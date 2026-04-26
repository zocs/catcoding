use anyhow::Result;
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod adapter;
mod api;
mod cascade;
mod db;
mod ipc;
mod log_buffer;
mod memory;
mod permission;
mod recovery;
mod rollback;
mod router;
mod scheduler;
mod skin;
mod state;
mod watchdog;
mod xp;

use adapter::AgentLifecycleManager;
use api::ApiState;
use db::Database;
use memory::MemoryManager;
use recovery::{FailureHandler, FailureScenario, RecipeStore};
use router::MessageRouter;
use scheduler::{Scheduler, SchedulerConfig};
use skin::cats::CatSkin;
use skin::Skin;
use state::StateManager;
use watchdog::{Watchdog, WatchdogConfig};
use xp::XpEngine;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "catcoding=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!(
        "CatCoding Daemon v{} starting...",
        env!("CARGO_PKG_VERSION")
    );

    // Skin system
    let skin = CatSkin::new();
    tracing::info!("{}", skin.info().motto);
    for role in skin.roles() {
        tracing::info!("  {} {} - {}", role.emoji, role.name, role.description);
    }

    // Database (SQLite cold storage)
    let db_path =
        std::env::var("DB_PATH").unwrap_or_else(|_| ".catcoding/catcoding.db".to_string());
    std::fs::create_dir_all(".catcoding")?;
    let db = Arc::new(Database::new(&db_path)?);
    db.init_schema().await?;
    tracing::info!("SQLite database: {}", db_path);

    // State manager (memory + SQLite)
    let state_manager = Arc::new(StateManager::new().with_db(db.clone()));
    tracing::info!("State manager initialized (hot state + cold storage)");

    // Watchdog
    let watchdog_config = WatchdogConfig::default();
    let (watchdog, mut restart_rx) = Watchdog::new(watchdog_config.clone());
    let watchdog = Arc::new(watchdog);
    tracing::info!(
        "Watchdog started - heartbeat: {}s, timeout: {}s",
        watchdog_config.heartbeat_interval,
        watchdog_config.heartbeat_timeout
    );
    let watchdog_clone = watchdog.clone();
    tokio::spawn(async move { watchdog_clone.start_monitoring().await });

    // Agent lifecycle manager
    let lifecycle_manager = Arc::new(Mutex::new(AgentLifecycleManager::new()));
    tracing::info!("Agent lifecycle manager initialized");

    // NATS connection (optional — daemon degrades to in-memory-only if unreachable)
    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string());
    let nats_client: Option<async_nats::Client> = match async_nats::connect(&nats_url).await {
        Ok(client) => {
            tracing::info!("Connected to NATS: {}", nats_url);
            // Subscribe to agent heartbeats
            let watchdog_for_heartbeat = watchdog.clone();
            match client.subscribe("agent.heartbeat").await {
                Ok(mut sub) => {
                    tokio::spawn(async move {
                        while let Some(msg) = sub.next().await {
                            if let Ok(data) =
                                serde_json::from_slice::<serde_json::Value>(&msg.payload)
                            {
                                if let Some(agent_id) =
                                    data.get("agent_id").and_then(|v| v.as_str())
                                {
                                    let _ = watchdog_for_heartbeat.heartbeat(agent_id).await;
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    tracing::warn!("NATS subscribe 'agent.heartbeat' failed: {}", e);
                }
            }
            Some(client)
        }
        Err(e) => {
            tracing::warn!("NATS connection failed ({}): using in-memory mode", e);
            tracing::info!("Hint: ensure NATS Server is running at {}", nats_url);
            None
        }
    };

    // Message router — wraps NATS client (or no-op if disconnected)
    let router = Arc::new(MessageRouter::new(nats_client.clone()).with_nats_url(nats_url.clone()));
    tracing::info!(
        "Message router: {}",
        if router.is_connected() {
            "connected to NATS"
        } else {
            "offline (publishes become no-ops, subscribes error)"
        }
    );

    // Scheduler (with watchdog + router integration)
    let scheduler_config = SchedulerConfig::default();
    let scheduler = Arc::new(
        Scheduler::new(scheduler_config.clone(), lifecycle_manager.clone())
            .with_watchdog(watchdog.clone())
            .with_router(router.clone()),
    );
    // Load per-role max_concurrent from roles.yaml (if present)
    scheduler.load_role_limits(".catcoding/roles.yaml").await;
    tracing::info!(
        "Scheduler started - interval: {}s, max concurrent: {}",
        scheduler_config.check_interval,
        scheduler_config.max_concurrent_tasks
    );
    let scheduler_clone = scheduler.clone();
    let state_clone = state_manager.clone();
    tokio::spawn(async move {
        scheduler_clone
            .start_scheduling(state_clone, "default".to_string())
            .await;
    });

    // Load history from SQLite
    state_manager.load_from_db("default").await?;

    // L4 memory system
    let memory_dir =
        std::env::var("MEMORY_DIR").unwrap_or_else(|_| ".catcoding/memory".to_string());
    let memory_manager = Arc::new(MemoryManager::new(&memory_dir)?);
    tracing::info!("L4 memory system initialized: {}", memory_dir);
    tracing::info!("  L1 index: {} lines", memory_manager.l1.line_count());
    tracing::info!("  L2 facts: {}", memory_manager.l2.count());
    tracing::info!("  L3 skills: {}", memory_manager.l3.count());
    tracing::info!("  L4 sessions: {}", memory_manager.l4.count());

    // Recovery system
    let recipe_store = Arc::new(RecipeStore::new());
    recipe_store.init_default_recipes().await;
    let failure_handler = Arc::new(
        FailureHandler::new(recipe_store.clone())
            .with_router(router.clone())
            .with_lifecycle_manager(lifecycle_manager.clone()),
    );
    tracing::info!("Recovery system initialized (5 default recipes, router + lifecycle wired)");
    let failure_handler_for_watchdog = failure_handler.clone();
    tokio::spawn(async move {
        while let Some(agent_id) = restart_rx.recv().await {
            tracing::warn!("Watchdog requested restart recovery for agent {}", agent_id);
            match failure_handler_for_watchdog
                .handle_failure(FailureScenario::AgentTimeout, &agent_id)
                .await
            {
                Ok(msg) => tracing::info!("Recovery handler result for {}: {}", agent_id, msg),
                Err(e) => tracing::error!("Recovery handler failed for agent {}: {}", agent_id, e),
            }
        }
        tracing::warn!("Watchdog restart channel closed; recovery listener exiting");
    });
    let failure_handler_for_nats = failure_handler.clone();
    let router_for_nats = router.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        const NATS_RECOVERY_CONTEXT: &str = "daemon_nats";
        loop {
            interval.tick().await;
            if router_for_nats.is_connected() {
                failure_handler_for_nats
                    .reset_retry_count_with_context(
                        &FailureScenario::NatsDisconnect,
                        NATS_RECOVERY_CONTEXT,
                    )
                    .await;
                continue;
            }
            tracing::warn!("NATS disconnected; triggering recovery recipe");
            if let Err(e) = failure_handler_for_nats
                .handle_failure(FailureScenario::NatsDisconnect, NATS_RECOVERY_CONTEXT)
                .await
            {
                tracing::warn!("NATS recovery attempt failed: {}", e);
            }
        }
    });

    // Log buffer
    let log_buffer = std::sync::Arc::new(log_buffer::LogBuffer::new(500));

    // WebSocket broadcast channel
    let (ws_tx, _ws_rx) = broadcast::channel::<String>(100);

    // XP engine (state + SQLite persistence)
    let xp_engine = Arc::new(XpEngine::new(state_manager.clone(), Some(db.clone())));
    tracing::info!("XP engine initialized");

    // API server state
    let api_state = Arc::new(ApiState {
        project_id: "default".to_string(),
        state_manager: state_manager.clone(),
        scheduler: scheduler.clone(),
        watchdog: watchdog.clone(),
        lifecycle_manager: lifecycle_manager.clone(),
        ws_tx: ws_tx.clone(),
        log_buffer: log_buffer.clone(),
        memory_manager: memory_manager.clone(),
        started_at: std::time::Instant::now(),
        xp_engine: xp_engine.clone(),
        router: router.clone(),
        db: Some(db.clone()),
    });

    let host = std::env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = std::env::var("API_PORT")
        .unwrap_or_else(|_| "19800".to_string())
        .parse()
        .unwrap_or(19800);

    let app = api::create_router(api_state);
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Daemon ready");
    tracing::info!("  Dashboard: http://{}", addr);
    tracing::info!("  API: http://{}/api", addr);
    tracing::info!("  Database: {}", db_path);
    tracing::info!("Press Ctrl+C to stop");

    // Graceful shutdown: axum drains in-flight requests, then we clean up agents.
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // ── Post-shutdown cleanup ──
    tracing::info!("Shutting down — cleaning up agents...");
    let mut lm = lifecycle_manager.lock().await;
    match lm.stop_all().await {
        Ok(()) => tracing::info!("All agents stopped"),
        Err(e) => tracing::warn!("Agent cleanup error: {}", e),
    }

    tracing::info!("CatCoding Daemon stopped. Goodbye 🐱");
    Ok(())
}

/// Wait for SIGINT (Ctrl+C) or SIGTERM (systemd / kill).
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { tracing::info!("SIGINT (Ctrl+C) received"); },
        _ = terminate => { tracing::info!("SIGTERM received"); },
    }
}
