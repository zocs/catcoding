use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

/// API 服务器状态
pub struct ApiState {
    pub project_id: String,
}

/// 项目信息
#[derive(Serialize)]
struct ProjectInfo {
    name: String,
    version: String,
    status: String,
    agents: Vec<AgentStatus>,
}

#[derive(Serialize)]
struct AgentStatus {
    id: String,
    role: String,
    status: String,
    current_task: Option<String>,
}

/// 命令请求
#[derive(Deserialize)]
struct CommandRequest {
    command: String,
    args: Option<Vec<String>>,
}

/// 创建 Axum 路由
pub fn create_router(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/", get(root_handler))
        .route("/api/health", get(health_handler))
        .route("/api/projects", get(list_projects))
        .route("/api/projects/:id", get(get_project))
        .route("/api/agents", get(list_agents))
        .route("/api/tasks", get(list_tasks))
        .route("/api/command", post(execute_command))
        .with_state(state)
}

/// 根路由 — Dashboard 入口
async fn root_handler() -> impl IntoResponse {
    Json(json!({
        "name": "CatCoding Daemon",
        "version": env!("CARGO_PKG_VERSION"),
        "dashboard": "http://localhost:19800",
        "motto": "🐱 让 AI 像猫咪团队一样协作做菜！"
    }))
}

/// 健康检查
async fn health_handler() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "uptime": "TODO",
        "memory_mb": "TODO"
    }))
}

/// 列出项目
async fn list_projects(State(state): State<Arc<ApiState>>) -> impl IntoResponse {
    Json(json!({
        "projects": [{
            "id": state.project_id,
            "name": "当前项目",
            "status": "active"
        }]
    }))
}

/// 获取项目详情
async fn get_project(
    State(_state): State<Arc<ApiState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> impl IntoResponse {
    Json(json!({
        "id": id,
        "name": "项目",
        "tasks": [],
        "agents": []
    }))
}

/// 列出 Agent
async fn list_agents(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    Json(json!({
        "agents": []
    }))
}

/// 列出任务
async fn list_tasks(State(_state): State<Arc<ApiState>>) -> impl IntoResponse {
    Json(json!({
        "tasks": []
    }))
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
