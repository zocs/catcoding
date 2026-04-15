use anyhow::Result;
use futures_util::StreamExt;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod adapter;
mod api;
mod cascade;
mod db;
mod ipc;
mod memory;
mod rollback;
mod router;
mod scheduler;
mod skin;
mod state;
mod watchdog;

use adapter::AgentLifecycleManager;
use api::ApiState;
use db::Database;
use memory::MemoryManager;
use scheduler::{Scheduler, SchedulerConfig};
use skin::cats::CatSkin;
use skin::Skin;
use state::StateManager;
use tokio::sync::Mutex;
use watchdog::{Watchdog, WatchdogConfig};

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
        "🐱 CatCoding Daemon v{} 启动中...",
        env!("CARGO_PKG_VERSION")
    );

    // 皮肤系统
    let skin = CatSkin::new();
    tracing::info!("🐾 {}", skin.info().motto);
    for role in skin.roles() {
        tracing::info!("  {} {} — {}", role.emoji, role.name, role.description);
    }

    // 数据库（SQLite 冷存储）
    let db_path =
        std::env::var("DB_PATH").unwrap_or_else(|_| ".catcoding/catcoding.db".to_string());
    std::fs::create_dir_all(".catcoding")?;
    let db = Arc::new(Database::new(&db_path)?);
    db.init_schema().await?;
    tracing::info!("💾 SQLite 数据库: {}", db_path);

    // 状态管理器（内存 + SQLite）
    let state_manager = Arc::new(StateManager::new().with_db(db.clone()));
    tracing::info!("💾 状态管理器已初始化（热状态 + 冷存储）");

    // Watchdog（猫头鹰）
    let watchdog_config = WatchdogConfig::default();
    let watchdog = Arc::new(Watchdog::new(watchdog_config.clone()));
    tracing::info!(
        "🦉 猫头鹰（Watchdog）已就位 — 心跳间隔: {}s, 超时: {}s",
        watchdog_config.heartbeat_interval,
        watchdog_config.heartbeat_timeout
    );
    let watchdog_clone = watchdog.clone();
    tokio::spawn(async move { watchdog_clone.start_monitoring().await });

    // Agent 生命周期管理器
    let lifecycle_manager = Arc::new(Mutex::new(AgentLifecycleManager::new()));
    tracing::info!("🐱 Agent 生命周期管理器已初始化");

    // 调度器
    let scheduler_config = SchedulerConfig::default();
    let scheduler = Arc::new(Scheduler::new(scheduler_config.clone(), lifecycle_manager.clone()));
    tracing::info!(
        "📋 调度器已启动 — 检查间隔: {}s, 最大并发: {}",
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

    // NATS 连接
    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://127.0.0.1:4222".to_string());
    match async_nats::connect(&nats_url).await {
        Ok(client) => {
            tracing::info!("📡 已连接 NATS: {}", nats_url);
            // 订阅 Agent 心跳
            let watchdog_for_heartbeat = watchdog.clone();
            let mut sub = client.subscribe("agent.heartbeat").await?;
            tokio::spawn(async move {
                while let Some(msg) = sub.next().await {
                    if let Ok(data) = serde_json::from_slice::<serde_json::Value>(&msg.payload) {
                        if let Some(agent_id) = data.get("agent_id").and_then(|v| v.as_str()) {
                            let _ = watchdog_for_heartbeat.heartbeat(agent_id).await;
                        }
                    }
                }
            });
        }
        Err(e) => {
            tracing::warn!("⚠️  NATS 连接失败 ({}): 使用内存模式", e);
            tracing::info!("💡 提示: 确保 NATS Server 在 {} 运行", nats_url);
        }
    }

    // 从 SQLite 加载历史数据
    state_manager.load_from_db("default").await?;

    // L4 记忆系统
    let memory_dir =
        std::env::var("MEMORY_DIR").unwrap_or_else(|_| ".catcoding/memory".to_string());
    let memory_manager = Arc::new(MemoryManager::new(&memory_dir)?);
    tracing::info!("🧠 L4 记忆系统已初始化: {}", memory_dir);
    tracing::info!("  L1 索引: {} 行", memory_manager.l1.line_count());
    tracing::info!("  L2 事实: {} 条", memory_manager.l2.count());
    tracing::info!("  L3 技能: {} 个", memory_manager.l3.count());
    tracing::info!("  L4 会话: {} 条", memory_manager.l4.count());

    // API 服务器
    let api_state = Arc::new(ApiState {
        project_id: "default".to_string(),
        state_manager: state_manager.clone(),
        scheduler: scheduler.clone(),
        watchdog: watchdog.clone(),
        lifecycle_manager: lifecycle_manager.clone(),
    });

    let host = std::env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = std::env::var("API_PORT")
        .unwrap_or_else(|_| "19800".to_string())
        .parse()
        .unwrap_or(19800);

    let app = api::create_router(api_state);
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("✅ Daemon 启动完成！");
    tracing::info!("🌐 Dashboard: http://{}", addr);
    tracing::info!("📡 API: http://{}/api", addr);
    tracing::info!("💾 数据库: {}", db_path);
    tracing::info!("按 Ctrl+C 停止");

    axum::serve(listener, app).await?;
    Ok(())
}
