use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 任务状态（8 态状态机）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    /// ⏳ 创建，等待依赖
    Pending,
    /// 🚫 依赖未满足
    Blocked,
    /// 🟡 等待分配
    Ready,
    /// 🔵 烹饪中 🍳
    Active,
    /// 🔍 品尝中
    Reviewing,
    /// ✅ 美味！
    Done,
    /// 🔄 回锅重做
    Rollbacked,
    /// ❌ 需要帮助
    Failed,
}

/// 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub assigned_to: Option<String>, // agent role
    pub depends_on: Vec<String>,     // task IDs
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub artifacts: Vec<String>,
}

/// Agent 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    /// 正在工作
    Working,
    /// 等待中（打盹）
    Idle,
    /// 已完成任务
    Done,
    /// 出错了
    Error,
    /// 被重启中
    Restarting,
}

/// Agent 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub role: String,
    pub status: AgentStatus,
    pub current_task: Option<String>,
    pub last_heartbeat: DateTime<Utc>,
    pub restart_count: u32,
}

/// 项目状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    pub id: String,
    pub name: String,
    pub tasks: HashMap<String, Task>,
    pub agents: HashMap<String, AgentInfo>,
}

/// 状态管理器
pub struct StateManager {
    /// 热状态（NATS KV 替代：内存 HashMap + 持久化到 SQLite）
    projects: Arc<RwLock<HashMap<String, ProjectState>>>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            projects: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建项目
    pub async fn create_project(&self, id: String, name: String) -> Result<()> {
        let state = ProjectState {
            id: id.clone(),
            name,
            tasks: HashMap::new(),
            agents: HashMap::new(),
        };
        self.projects.write().await.insert(id, state);
        Ok(())
    }

    /// 添加任务
    pub async fn add_task(&self, project_id: &str, task: Task) -> Result<()> {
        let mut projects = self.projects.write().await;
        if let Some(project) = projects.get_mut(project_id) {
            project.tasks.insert(task.id.clone(), task);
        }
        Ok(())
    }

    /// 更新任务状态
    pub async fn update_task_status(
        &self,
        project_id: &str,
        task_id: &str,
        status: TaskStatus,
    ) -> Result<()> {
        let mut projects = self.projects.write().await;
        if let Some(project) = projects.get_mut(project_id) {
            if let Some(task) = project.tasks.get_mut(task_id) {
                task.status = status;
                task.updated_at = Utc::now();
            }
        }
        Ok(())
    }

    /// 注册 Agent
    pub async fn register_agent(&self, project_id: &str, agent: AgentInfo) -> Result<()> {
        let mut projects = self.projects.write().await;
        if let Some(project) = projects.get_mut(project_id) {
            project.agents.insert(agent.id.clone(), agent);
        }
        Ok(())
    }

    /// 获取项目状态（只读快照）
    pub async fn get_project(&self, project_id: &str) -> Option<ProjectState> {
        self.projects.read().await.get(project_id).cloned()
    }
}
