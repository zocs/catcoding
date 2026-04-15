use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    /// 任务结果
    TaskResult,
    /// 状态更新
    StatusUpdate,
    /// 请求
    Request,
    /// 告警
    Alert,
    /// 心跳
    Heartbeat,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageStatus {
    Completed,
    Failed,
    Blocked,
    Progress,
}

/// Agent 间通信消息格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub msg_id: String,
    pub msg_type: MessageType,
    pub from: String,
    pub to: String,
    pub timestamp: String,
    pub task_id: Option<String>,
    pub status: Option<MessageStatus>,
    pub progress_percent: Option<u8>,
    pub artifacts: Vec<String>,
    pub summary: String,
    pub details: Option<String>,
    pub blockers: Vec<String>,
}

impl AgentMessage {
    pub fn new(from: &str, to: &str, msg_type: MessageType, summary: &str) -> Self {
        Self {
            msg_id: uuid::Uuid::new_v4().to_string(),
            msg_type,
            from: from.to_string(),
            to: to.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            task_id: None,
            status: None,
            progress_percent: None,
            artifacts: Vec::new(),
            summary: summary.to_string(),
            details: None,
            blockers: Vec::new(),
        }
    }

    pub fn with_task(mut self, task_id: &str) -> Self {
        self.task_id = Some(task_id.to_string());
        self
    }

    pub fn with_status(mut self, status: MessageStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_progress(mut self, percent: u8) -> Self {
        self.progress_percent = Some(percent);
        self
    }
}

/// 消息路由器 — 基于 NATS 的消息路由
///
/// 通信通道：
/// - 任务队列: NATS Stream "tasks.{role}" — 持久化，支持消费者组
/// - 实时通信: NATS Pub/Sub "agent.{id}.progress" — 实时推送
/// - 心跳: NATS Pub/Sub "agent.heartbeat" — Watchdog 订阅
/// - 日志流: NATS Stream "logs.{project_id}" — 持久化+实时
/// - 告警: NATS Pub/Sub "watchdog.alert" — PM Agent 订阅
pub struct MessageRouter {
    /// 消息回调处理器
    handlers: HashMap<String, Box<dyn Fn(AgentMessage) + Send + Sync>>,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// 注册消息处理器
    pub fn register_handler(
        &mut self,
        subject: &str,
        handler: Box<dyn Fn(AgentMessage) + Send + Sync>,
    ) {
        self.handlers.insert(subject.to_string(), handler);
    }

    /// 发送消息到指定 subject
    pub async fn publish(&self, subject: &str, message: &AgentMessage) -> Result<()> {
        let payload = serde_json::to_vec(message)?;
        tracing::debug!(
            "Routing message [{}] → {} ({} bytes)",
            subject,
            message.to,
            payload.len()
        );
        // TODO: 实际通过 NATS 发送
        // let client = async_nats::connect("nats://localhost:4222").await?;
        // client.publish(subject, payload.into()).await?;
        Ok(())
    }

    /// 订阅 subject
    pub async fn subscribe(&self, subject: &str) -> Result<()> {
        tracing::info!("Subscribing to: {}", subject);
        // TODO: 实际通过 NATS 订阅
        Ok(())
    }

    /// 生成 NATS subject
    pub fn task_subject(role: &str) -> String {
        format!("tasks.{}", role)
    }

    pub fn progress_subject(agent_id: &str) -> String {
        format!("agent.{}.progress", agent_id)
    }

    pub fn heartbeat_subject() -> &'static str {
        "agent.heartbeat"
    }

    pub fn alert_subject() -> &'static str {
        "watchdog.alert"
    }

    pub fn log_subject(project_id: &str) -> String {
        format!("logs.{}", project_id)
    }
}
