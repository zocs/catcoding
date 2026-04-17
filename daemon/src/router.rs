use anyhow::Result;
use async_nats::Client;
use serde::{Deserialize, Serialize};

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    TaskResult,
    StatusUpdate,
    Request,
    Alert,
    Heartbeat,
}

/// 任务状态（消息侧）
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

/// 消息路由器 — 真 NATS Pub/Sub 实现
///
/// 若 NATS 不可达，`client` 为 `None`，`publish_*` 静默成功、返回 `Ok(())`，
/// 调用方不用特判。订阅方法在 `client` 为 `None` 时直接报错。
///
/// 通道（约定）：
/// - `tasks.{role}`          — 任务队列（Stream，持久化）
/// - `agent.{id}.progress`   — Agent 进度推送（Pub/Sub）
/// - `agent.{id}.xp`         — XP / 等级变更（Pub/Sub）
/// - `agent.heartbeat`       — 心跳（Watchdog 订阅）
/// - `logs.{project_id}`     — 日志流（Stream）
/// - `watchdog.alert`        — Watchdog 告警
pub struct MessageRouter {
    client: Option<Client>,
}

impl MessageRouter {
    pub fn new(client: Option<Client>) -> Self {
        Self { client }
    }

    pub fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    /// 发送一条 `AgentMessage` 到指定 subject
    pub async fn publish(&self, subject: &str, message: &AgentMessage) -> Result<()> {
        let payload = serde_json::to_vec(message)?;
        self.publish_bytes(subject, payload).await
    }

    /// 发送任意 JSON 到指定 subject
    pub async fn publish_json(
        &self,
        subject: &str,
        value: &serde_json::Value,
    ) -> Result<()> {
        let payload = serde_json::to_vec(value)?;
        self.publish_bytes(subject, payload).await
    }

    /// 底层字节发送
    pub async fn publish_bytes(&self, subject: &str, payload: Vec<u8>) -> Result<()> {
        let Some(client) = self.client.as_ref() else {
            tracing::trace!(
                "NATS offline, skip publish [{}] ({} bytes)",
                subject,
                payload.len()
            );
            return Ok(());
        };

        let subject_owned: String = subject.to_string();
        let n = payload.len();
        client
            .publish(subject_owned.clone(), payload.into())
            .await
            .map_err(|e| anyhow::anyhow!("NATS publish to {} failed: {}", subject_owned, e))?;
        tracing::debug!("NATS publish [{}] ({} bytes)", subject_owned, n);
        Ok(())
    }

    /// 订阅 subject；订阅结果由调用方自行 `while let Some(msg) = sub.next().await`
    pub async fn subscribe(&self, subject: &str) -> Result<async_nats::Subscriber> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("NATS not connected — cannot subscribe to {}", subject))?;
        let sub = client
            .subscribe(subject.to_string())
            .await
            .map_err(|e| anyhow::anyhow!("NATS subscribe to {} failed: {}", subject, e))?;
        tracing::info!("NATS subscribed: {}", subject);
        Ok(sub)
    }

    /// 生成 NATS subject
    pub fn task_subject(role: &str) -> String {
        format!("tasks.{}", role)
    }

    pub fn progress_subject(agent_id: &str) -> String {
        format!("agent.{}.progress", agent_id)
    }

    pub fn xp_subject(agent_id: &str) -> String {
        format!("agent.{}.xp", agent_id)
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
