pub mod claude;
pub mod codex;
pub mod hermes;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use hermes::AgentLifecycleManager;

/// Agent 上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub agent_id: String,
    pub role: String,
    pub project_id: String,
    pub task_description: String,
    pub working_dir: String,
}

impl AgentContext {
    pub fn new(agent_id: &str, role: &str, project_id: &str, task: &str) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            role: role.to_string(),
            project_id: project_id.to_string(),
            task_description: task.to_string(),
            working_dir: ".".to_string(),
        }
    }

    pub fn with_working_dir(mut self, dir: &str) -> Self {
        self.working_dir = dir.to_string();
        self
    }
}

/// Agent 句柄
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHandle {
    pub agent_id: String,
    pub pid: Option<u32>,
    pub adapter_type: String,
}

/// Agent 输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub agent_id: String,
    pub output_type: String, // "stdout", "stderr", "progress", "result"
    pub content: String,
    pub timestamp: String,
}

/// 健康状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

/// Adapter 接口 — 支持多种 AI Agent 框架
///
/// 实现此 trait 即可接入新的 AI agent
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    /// 获取 Adapter 名称
    fn name(&self) -> &str;

    /// 启动 Agent 进程
    async fn spawn(&self, context: AgentContext) -> Result<AgentHandle>;

    /// 发送任务给 Agent
    async fn send_task(&self, handle: &AgentHandle, task_description: &str) -> Result<()>;

    /// 获取 Agent 输出流
    async fn get_output(&self, handle: &AgentHandle) -> Result<Option<AgentOutput>>;

    /// 停止 Agent
    async fn stop(&self, handle: &AgentHandle) -> Result<()>;

    /// 健康检查
    async fn health_check(&self, handle: &AgentHandle) -> Result<HealthStatus>;
}
