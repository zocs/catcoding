use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use rust_embed::{Embed, RustEmbed};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::broadcast;

use crate::adapter::AgentLifecycleManager;
use crate::log_buffer::LogBuffer;
use crate::memory::MemoryManager;
use crate::scheduler::Scheduler;
use crate::state::{StateManager, Task, TaskStatus};
use crate::watchdog::Watchdog;
use crate::xp::{XpEngine, XpEvent};

/// 嵌入式 Dashboard 静态文件
#[derive(RustEmbed)]
#[folder = "../dashboard/dist/"]
struct DashboardAssets;

/// API 服务器状态
pub struct ApiState {
    pub project_id: String,
    pub state_manager: Arc<StateManager>,
    pub scheduler: Arc<Scheduler>,
    pub watchdog: Arc<Watchdog>,
    pub lifecycle_manager: Arc<tokio::sync::Mutex<AgentLifecycleManager>>,
    /// WebSocket broadcast channel
    pub ws_tx: broadcast::Sender<String>,
    /// Log ring buffer
    pub log_buffer: Arc<LogBuffer>,
    /// L4 Memory system
    pub memory_manager: Arc<MemoryManager>,
    /// Daemon start time, for `/api/health` uptime.
    pub started_at: Instant,
    /// XP persistence engine (awards XP on task status transitions).
    pub xp_engine: Arc<XpEngine>,
    /// Optional NATS router: real pub/sub when connected.
    pub router: Arc<crate::router::MessageRouter>,
    /// Cold storage handle (for reading xp_log etc).
    pub db: Option<Arc<crate::db::Database>>,
}

/// 命令请求
#[derive(Deserialize)]
struct CommandRequest {
    command: String,
    args: Option<Vec<String>>,
}

/// Watchdog 状态
async fn watchdog_status(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    let summary = state.watchdog.status_summary().await;
    Json(summary)
}

/// Recent logs
async fn list_logs(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    let entries = state.log_buffer.get_recent(200);
    Json(json!({
        "logs": entries,
        "total": state.log_buffer.count(),
    }))
}

/// 创建任务请求
#[derive(Deserialize)]
struct CreateTaskRequest {
    title: String,
    description: Option<String>,
    role: Option<String>,
}

/// 创建 Axum 路由
pub fn create_router(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/api/health", get(health_handler))
        .route("/api/projects", get(list_projects))
        .route("/api/projects/{id}", get(get_project))
        .route("/api/agents", get(list_agents))
        .route("/api/tasks", get(list_tasks).post(create_task))
        .route("/api/tasks/{id}", get(get_task))
        .route("/api/tasks/{id}/status", post(update_task_status))
        .route("/api/command", post(execute_command))
        .route("/api/permission/check", post(permission_check))
        .route("/api/watchdog", get(watchdog_status))
        .route("/api/logs", get(list_logs))
        .route("/api/memory/status", get(memory_status))
        .route("/api/memory/search", get(memory_search))
        .route("/api/agents/{id}/xp-log", get(agent_xp_log))
        .route("/ws", get(ws_handler))
        .route("/dashboard", get(dashboard_index))
        .route("/dashboard/{*path}", get(dashboard_handler))
        .route("/assets/{*path}", get(dashboard_handler))
        .with_state(state)
}

/// 根路由
async fn root_handler(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    let agents = state.state_manager.get_project(&state.project_id).await;
    let agent_count = agents.as_ref().map(|p| p.agents.len()).unwrap_or(0);
    let task_count = agents.as_ref().map(|p| p.tasks.len()).unwrap_or(0);

    Json(json!({
        "name": "CatCoding Daemon",
        "version": env!("CARGO_PKG_VERSION"),
        "motto": "Let AI collaborate like a cat team!",
        "project": state.project_id,
        "agents": agent_count,
        "tasks": task_count,
        "endpoints": {
            "health": "/api/health",
            "projects": "/api/projects",
            "agents": "/api/agents",
            "tasks": "/api/tasks",
            "command": "/api/command",
            "dashboard": "/dashboard"
        }
    }))
}

/// 健康检查
async fn health_handler(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "project": state.project_id,
        "uptime_secs": state.started_at.elapsed().as_secs(),
    }))
}

/// 列出项目
async fn list_projects(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    let project = state.state_manager.get_project(&state.project_id).await;
    match project {
        Some(p) => Json(json!({
            "projects": [{
                "id": p.id,
                "name": p.name,
                "task_count": p.tasks.len(),
                "agent_count": p.agents.len()
            }]
        })),
        None => Json(json!({ "projects": [] })),
    }
}

/// 获取项目详情
async fn get_project(State(state): State<Arc<ApiState>>, Path(id): Path<String>) -> Response {
    match state.state_manager.get_project(&id).await {
        Some(p) => Json(json!({
            "id": p.id,
            "name": p.name,
            "tasks": p.tasks.values().collect::<Vec<_>>(),
            "agents": p.agents.values().collect::<Vec<_>>()
        }))
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Project not found" })),
        )
            .into_response(),
    }
}

/// 列出 Agent
async fn list_agents(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    let project = state.state_manager.get_project(&state.project_id).await;
    let agents: Vec<_> = project
        .map(|p| p.agents.values().cloned().collect())
        .unwrap_or_default();
    Json(json!({ "agents": agents }))
}

/// 列出任务
async fn list_tasks(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    let project = state.state_manager.get_project(&state.project_id).await;
    let tasks: Vec<_> = project
        .map(|p| p.tasks.values().cloned().collect())
        .unwrap_or_default();
    Json(json!({ "tasks": tasks }))
}

/// 获取单个任务
async fn get_task(State(state): State<Arc<ApiState>>, Path(id): Path<String>) -> Response {
    let project = state.state_manager.get_project(&state.project_id).await;
    match project.and_then(|p| p.tasks.get(&id).cloned()) {
        Some(task) => Json(json!(task)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Task not found" })),
        )
            .into_response(),
    }
}

/// 创建任务
async fn create_task(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<CreateTaskRequest>,
) -> impl IntoResponse {
    let task = Task::new(
        &req.title,
        req.description.as_deref().unwrap_or(""),
        req.role.as_deref(),
    );
    let task_id = task.id.clone();
    let title = task.title.clone();

    // 保存到状态管理器
    let _ = state
        .state_manager
        .add_task(&state.project_id, task.clone())
        .await;

    // 加入调度队列
    let _ = state.scheduler.enqueue(task).await;

    tracing::info!("Task created: {} ({})", title, task_id);

    (
        StatusCode::CREATED,
        Json(json!({
            "id": task_id,
            "title": title,
            "status": "pending",
            "message": "Task created, waiting for scheduling..."
        })),
    )
}

/// 更新任务状态
async fn update_task_status(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let status_str = body
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("active");

    let new_status = match status_str {
        "pending" => TaskStatus::Pending,
        "blocked" => TaskStatus::Blocked,
        "ready" => TaskStatus::Ready,
        "active" => TaskStatus::Active,
        "reviewing" => TaskStatus::Reviewing,
        "done" => TaskStatus::Done,
        "rollbacked" => TaskStatus::Rollbacked,
        "failed" => TaskStatus::Failed,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Invalid status" })),
            )
                .into_response()
        }
    };

    // Capture prior status so we can decide XP event (was it a retry?).
    let prior_task = state.state_manager.get_task(&state.project_id, &id).await;
    let prior_status = prior_task.as_ref().map(|t| t.status.clone());

    match state
        .state_manager
        .update_task_status(&state.project_id, &id, new_status.clone())
        .await
    {
        Ok(_) => {
            tracing::info!("Task {} status updated to {:?}", id, new_status);

            // ═══ XP event hook ═══
            // Award XP on status transitions. Only applies when the task has
            // an assigned agent role + an actual agent running that role.
            if let Some(task) = state.state_manager.get_task(&state.project_id, &id).await {
                let event = xp_event_for_transition(prior_status.as_ref(), &new_status);
                if let (Some(ev), Some(role)) = (event, task.assigned_to.as_deref()) {
                    // Find an agent with that role (first match).
                    let project = state.state_manager.get_project(&state.project_id).await;
                    let agent_id = project.and_then(|p| {
                        p.agents
                            .values()
                            .find(|a| a.role == role)
                            .map(|a| a.id.clone())
                    });
                    if let Some(aid) = agent_id {
                        match state
                            .xp_engine
                            .apply(&state.project_id, &aid, Some(&task.id), &ev)
                            .await
                        {
                            Ok(Some(outcome)) => {
                                // Broadcast XP event on WebSocket for real-time dashboard update.
                                let ws_msg = json!({
                                    "type": "xp.update",
                                    "agent_id": aid,
                                    "task_id": task.id,
                                    "event": ev.reason(),
                                    "delta": outcome.delta,
                                    "old_xp": outcome.old_xp,
                                    "new_xp": outcome.new_xp,
                                    "old_level": outcome.old_level,
                                    "new_level": outcome.new_level,
                                    "leveled_up": outcome.leveled_up,
                                });
                                let _ = state.ws_tx.send(ws_msg.to_string());

                                // Also publish to NATS router if connected.
                                let _ = state
                                    .router
                                    .publish_json(
                                        &format!("agent.{}.xp", aid),
                                        &ws_msg,
                                    )
                                    .await;
                            }
                            Ok(None) => {
                                tracing::debug!("XP: agent {} not found, skipped", aid);
                            }
                            Err(e) => {
                                tracing::warn!("XP apply failed: {}", e);
                            }
                        }
                    }
                }
            }

            // Publish task status update to NATS.
            let _ = state
                .router
                .publish_json(
                    &crate::router::MessageRouter::log_subject(&state.project_id),
                    &json!({
                        "type": "task.status",
                        "task_id": id,
                        "status": status_str,
                    }),
                )
                .await;

            // Broadcast agent live-status over WS so dashboards can react without polling.
            // Derive dashboard-style status ('active' | 'idle' | 'busy' | 'error') from task transition.
            let agent_status = match new_status {
                TaskStatus::Active => Some("active"),
                TaskStatus::Reviewing | TaskStatus::Blocked => Some("busy"),
                TaskStatus::Done | TaskStatus::Rollbacked => Some("idle"),
                TaskStatus::Failed => Some("error"),
                _ => None,
            };
            if let (Some(status_str_ws), Some(task)) = (
                agent_status,
                state.state_manager.get_task(&state.project_id, &id).await,
            ) {
                if let Some(role) = task.assigned_to.as_deref() {
                    let project = state.state_manager.get_project(&state.project_id).await;
                    let agent_id = project.and_then(|p| {
                        p.agents
                            .values()
                            .find(|a| a.role == role)
                            .map(|a| a.id.clone())
                    });
                    if let Some(aid) = agent_id {
                        let ws_msg = json!({
                            "type": "agent.status",
                            "agent_id": aid,
                            "role": role,
                            "status": status_str_ws,
                            "task_id": task.id,
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                        });
                        let _ = state.ws_tx.send(ws_msg.to_string());
                        let _ = state
                            .router
                            .publish_json(&format!("agent.{}.status", aid), &ws_msg)
                            .await;
                    }
                }
            }

            Json(json!({
                "id": id,
                "status": status_str,
                "message": "Status updated"
            }))
            .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

/// Decide which XP event, if any, a status transition should emit.
fn xp_event_for_transition(prev: Option<&TaskStatus>, new: &TaskStatus) -> Option<XpEvent> {
    match (prev, new) {
        // First successful completion — review passed on the first try.
        (Some(TaskStatus::Reviewing), TaskStatus::Done) => Some(XpEvent::ReviewPassedFirst),
        // Task completed without a review stage.
        (_, TaskStatus::Done) => Some(XpEvent::TaskCompleted),
        // Task failed outright.
        (_, TaskStatus::Failed) => Some(XpEvent::TaskFailed),
        // Task rolled back — treat as failed retry.
        (_, TaskStatus::Rollbacked) => Some(XpEvent::TaskFailed),
        _ => None,
    }
}

/// 执行命令
async fn execute_command(
    State(_state): State<Arc<ApiState>>,
    Json(req): Json<CommandRequest>,
) -> impl IntoResponse {
    tracing::info!("Received command: {} {:?}", req.command, req.args);
    Json(json!({
        "result": "ok",
        "command": req.command,
        "message": "Command received, processing..."
    }))
}

/// Dashboard 首页
async fn dashboard_index() -> Response {
    match DashboardAssets::get("index.html") {
        Some(content) => Html(content.data.into_owned()).into_response(),
        None => (StatusCode::NOT_FOUND, "Dashboard not built").into_response(),
    }
}

/// 提供 Dashboard 静态文件
async fn dashboard_handler(Path(path): Path<String>) -> Response {
    let path = if path.is_empty() { "index.html" } else { &path };

    match DashboardAssets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path)
                .first_or_octet_stream()
                .to_string();

            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime)],
                content.data.into_owned(),
            )
                .into_response()
        }
        None => {
            // SPA fallback: 返回 index.html
            match DashboardAssets::get("index.html") {
                Some(content) => Html(content.data.into_owned()).into_response(),
                None => (StatusCode::NOT_FOUND, "Dashboard not built").into_response(),
            }
        }
    }
}

/// Memory system status
async fn memory_status(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    let summary = state.memory_manager.status_summary();
    Json(summary)
}

/// Agent XP 审计日志
async fn agent_xp_log(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> Response {
    match &state.db {
        Some(db) => match db.get_xp_log(&id, 50).await {
            Ok(rows) => Json(json!({ "agent_id": id, "entries": rows })).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "Database not available" })),
        )
            .into_response(),
    }
}

/// Search memory by keyword
#[derive(Deserialize)]
struct MemorySearchQuery {
    q: Option<String>,
}

async fn memory_search(
    State(state): State<Arc<ApiState>>,
    axum::extract::Query(query): axum::extract::Query<MemorySearchQuery>,
) -> impl IntoResponse {
    let keyword = query.q.unwrap_or_default();
    let mut results = Vec::new();

    // Search L2 facts
    if let Some(fact) = state.memory_manager.l2.get(&keyword) {
        results.push(json!({
            "layer": "L2",
            "key": keyword,
            "value": fact,
        }));
    }

    // Search L3 skills
    if let Some(skill) = state.memory_manager.l3.get(&keyword) {
        results.push(json!({
            "layer": "L3",
            "key": keyword,
            "value": skill.to_context(),
        }));
    }

    // Search L4 sessions
    if let Ok(sessions) = state.memory_manager.l4.search(&keyword) {
        for s in sessions {
            results.push(json!({
                "layer": "L4",
                "key": s.task_id,
                "value": s.task_summary,
                "outcome": s.outcome,
                "completed_at": s.completed_at,
            }));
        }
    }

    Json(json!({
        "query": keyword,
        "results": results,
        "count": results.len(),
    }))
}

/// WebSocket 升级处理
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| ws_connection(socket, state))
}

/// WebSocket 连接处理
async fn ws_connection(socket: axum::extract::ws::WebSocket, state: Arc<ApiState>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.ws_tx.subscribe();

    // 发送欢迎消息
    let welcome = json!({
        "type": "connected",
        "message": "CatCoding WebSocket connected",
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });
    let _ = sender
        .send(axum::extract::ws::Message::Text(welcome.to_string().into()))
        .await;

    // 转发广播消息到 WebSocket
    let send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender
                .send(axum::extract::ws::Message::Text(msg.into()))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // 接收客户端消息（目前忽略）
    while let Some(Ok(_msg)) = receiver.next().await {
        // 可以处理客户端发来的消息
    }

    send_task.abort();
    tracing::info!("WebSocket client disconnected");
}

/// 权限检查请求
#[derive(Deserialize)]
struct PermissionCheckRequest {
    command: String,
}

/// 权限检查 API — 对 Bash 命令进行权限分级
async fn permission_check(Json(req): Json<PermissionCheckRequest>) -> impl IntoResponse {
    use crate::permission::{check_permission, classify_bash_command, PermissionConfig};

    let config = PermissionConfig::default();
    let level = classify_bash_command(&req.command);
    let (allowed, _, reason) = check_permission(&req.command, &config);

    Json(json!({
        "command": req.command,
        "level": level.to_string(),
        "allowed": allowed,
        "reason": reason,
    }))
}
