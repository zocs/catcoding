use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use rust_embed::RustEmbed;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

use crate::cascade::CascadeHandler;
use crate::rollback::RollbackManager;
use crate::scheduler::Scheduler;
use crate::state::{StateManager, Task, TaskStatus};
use crate::watchdog::Watchdog;

/// 嵌入式 Dashboard 静态文件
#[derive(RustEmbed)]
#[folder = "/home/zocs/devs/catcoding/dashboard/dist/"]
struct DashboardAssets;

/// API 服务器状态
pub struct ApiState {
    pub project_id: String,
    pub state_manager: Arc<StateManager>,
    pub scheduler: Arc<Scheduler>,
    pub watchdog: Arc<Watchdog>,
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
        .route("/api/watchdog", get(watchdog_status))
        .route("/dashboard", get(dashboard_index))
        .route("/dashboard/{*path}", get(dashboard_handler))
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
        "motto": "🐱 让 AI 像猫咪团队一样协作做菜！",
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
        "uptime": "TODO"
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
async fn get_project(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> Response {
    match state.state_manager.get_project(&id).await {
        Some(p) => Json(json!({
            "id": p.id,
            "name": p.name,
            "tasks": p.tasks.values().collect::<Vec<_>>(),
            "agents": p.agents.values().collect::<Vec<_>>()
        })).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "项目不存在" })),
        ).into_response(),
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
async fn get_task(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
) -> Response {
    let project = state.state_manager.get_project(&state.project_id).await;
    match project.and_then(|p| p.tasks.get(&id).cloned()) {
        Some(task) => Json(json!(task)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "任务不存在" })),
        ).into_response(),
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
    let _ = state.state_manager.add_task(&state.project_id, task.clone()).await;

    // 加入调度队列
    let _ = state.scheduler.enqueue(task).await;

    tracing::info!("📋 创建任务: {} ({})", title, task_id);

    (
        StatusCode::CREATED,
        Json(json!({
            "id": task_id,
            "title": title,
            "status": "pending",
            "message": "任务已创建，等待调度..."
        })),
    )
}

/// 更新任务状态
async fn update_task_status(
    State(state): State<Arc<ApiState>>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let status_str = body.get("status")
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
                Json(json!({ "error": "无效的状态" })),
            ).into_response()
        }
    };

    match state.state_manager.update_task_status(
        &state.project_id,
        &id,
        new_status.clone(),
    ).await {
        Ok(_) => {
            tracing::info!("📋 任务 {} 状态更新为 {:?}", id, new_status);
            Json(json!({
                "id": id,
                "status": status_str,
                "message": "状态已更新"
            })).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ).into_response(),
    }
}

/// 执行命令
async fn execute_command(
    State(_state): State<Arc<ApiState>>,
    Json(req): Json<CommandRequest>,
) -> impl IntoResponse {
    tracing::info!("📨 收到命令: {} {:?}", req.command, req.args);
    Json(json!({
        "result": "ok",
        "command": req.command,
        "message": "命令已接收，正在处理..."
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
            ).into_response()
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
