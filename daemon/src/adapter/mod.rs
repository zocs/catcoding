pub mod hermes;
// pub mod claude;
// pub mod opencode;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Agent 上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    pub agent_id: String,
    pub role: String,
    pub project_id: String,
    pub task_description: String,
    pub working_dir: String,
}

/// Agent 句柄
#[derive(Debug, Clone)]
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
/// 实现此 trait 即可接入新的 AI agent（Hermes, Claude Code, Codex, OpenCode 等）
/// 框架不与任何 AI agent 锁死
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
