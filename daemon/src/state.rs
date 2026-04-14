use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::db::Database;

/// 任务状态（8 态状态机）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Blocked,
    Ready,
    Active,
    Reviewing,
    Done,
    Rollbacked,
    Failed,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Blocked => "blocked",
            Self::Ready => "ready",
            Self::Active => "active",
            Self::Reviewing => "reviewing",
            Self::Done => "done",
            Self::Rollbacked => "rollbacked",
            Self::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "blocked" => Self::Blocked,
            "ready" => Self::Ready,
            "active" => Self::Active,
            "reviewing" => Self::Reviewing,
            "done" => Self::Done,
            "rollbacked" => Self::Rollbacked,
            "failed" => Self::Failed,
            _ => Self::Pending,
        }
    }
}

/// 任务定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub assigned_to: Option<String>,
    pub depends_on: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub artifacts: Vec<String>,
}

impl Task {
    pub fn new(title: &str, description: &str, role: Option<&str>) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.to_string(),
            description: description.to_string(),
            status: TaskStatus::Pending,
            assigned_to: role.map(String::from),
            depends_on: Vec::new(),
            created_at: now,
            updated_at: now,
            artifacts: Vec::new(),
        }
    }

    pub fn with_dependency(mut self, task_id: &str) -> Self {
        self.depends_on.push(task_id.to_string());
        self
    }

    pub fn with_artifact(mut self, path: &str) -> Self {
        self.artifacts.push(path.to_string());
        self
    }
}

/// Agent 状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Working,
    Idle,
    Done,
    Error,
    Restarting,
}

impl AgentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Working => "working",
            Self::Idle => "idle",
            Self::Done => "done",
            Self::Error => "error",
            Self::Restarting => "restarting",
        }
    }
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

impl AgentInfo {
    pub fn new(id: &str, role: &str) -> Self {
        Self {
            id: id.to_string(),
            role: role.to_string(),
            status: AgentStatus::Idle,
            current_task: None,
            last_heartbeat: Utc::now(),
            restart_count: 0,
        }
    }
}

/// 项目状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectState {
    pub id: String,
    pub name: String,
    pub tasks: HashMap<String, Task>,
    pub agents: HashMap<String, AgentInfo>,
}

/// 状态管理器 — 内存热状态 + SQLite 冷存储
pub struct StateManager {
    /// 热状态（内存）
    projects: Arc<RwLock<HashMap<String, ProjectState>>>,
    /// 冷存储（SQLite）
    db: Option<Arc<Database>>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            projects: Arc::new(RwLock::new(HashMap::new())),
            db: None,
        }
    }

    /// 设置数据库（持久化）
    pub fn with_db(mut self, db: Arc<Database>) -> Self {
        self.db = Some(db);
        self
    }

    /// 确保项目存在
    async fn ensure_project(&self, project_id: &str) {
        let mut projects = self.projects.write().await;
        if !projects.contains_key(project_id) {
            projects.insert(project_id.to_string(), ProjectState {
                id: project_id.to_string(),
                name: project_id.to_string(),
                tasks: HashMap::new(),
                agents: HashMap::new(),
            });
        }
    }

    /// 添加任务（内存 + SQLite）
    pub async fn add_task(&self, project_id: &str, task: Task) -> Result<()> {
        self.ensure_project(project_id).await;

        // 写入内存
        {
            let mut projects = self.projects.write().await;
            if let Some(project) = projects.get_mut(project_id) {
                project.tasks.insert(task.id.clone(), task.clone());
            }
        }

        // 写入 SQLite
        if let Some(db) = &self.db {
            db.insert_task(project_id, &task).await?;
            tracing::debug!("💾 任务 {} 已持久化到 SQLite", task.id);
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
        // 更新内存
        {
            let mut projects = self.projects.write().await;
            if let Some(project) = projects.get_mut(project_id) {
                if let Some(task) = project.tasks.get_mut(task_id) {
                    task.status = status.clone();
                    task.updated_at = Utc::now();
                }
            }
        }

        // 更新 SQLite
        if let Some(db) = &self.db {
            db.update_task_status(project_id, task_id, status.as_str()).await?;
        }

        Ok(())
    }

    /// 注册 Agent
    pub async fn register_agent(&self, project_id: &str, agent: AgentInfo) -> Result<()> {
        self.ensure_project(project_id).await;

        {
            let mut projects = self.projects.write().await;
            if let Some(project) = projects.get_mut(project_id) {
                project.agents.insert(agent.id.clone(), agent.clone());
            }
        }

        if let Some(db) = &self.db {
            db.insert_agent(project_id, &agent).await?;
        }

        Ok(())
    }

    /// 获取项目
    pub async fn get_project(&self, project_id: &str) -> Option<ProjectState> {
        self.projects.read().await.get(project_id).cloned()
    }

    /// 获取任务
    pub async fn get_task(&self, project_id: &str, task_id: &str) -> Option<Task> {
        self.projects.read().await
            .get(project_id)
            .and_then(|p| p.tasks.get(task_id).cloned())
    }

    /// 从 SQLite 加载历史数据
    pub async fn load_from_db(&self, project_id: &str) -> Result<()> {
        if let Some(db) = &self.db {
            let tasks = db.get_tasks(project_id).await?;
            let agents = db.get_agents(project_id).await?;

            self.ensure_project(project_id).await;
            let mut projects = self.projects.write().await;
            if let Some(project) = projects.get_mut(project_id) {
                for task in tasks {
                    project.tasks.insert(task.id.clone(), task);
                }
                for agent in agents {
                    project.agents.insert(agent.id.clone(), agent);
                }
                tracing::info!("💾 从 SQLite 加载了 {} 个任务和 {} 个 Agent",
                    project.tasks.len(), project.agents.len());
            }
        }
        Ok(())
    }
}
