use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

use super::{AgentAdapter, AgentContext, AgentHandle, AgentOutput, HealthStatus};

/// Claude Code 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeConfig {
    /// claude 可执行文件路径
    pub binary_path: String,
    /// API Key（可选，也可通过环境变量设置）
    pub api_key: Option<String>,
    /// 模型选择
    pub model: Option<String>,
    /// 额外参数
    pub extra_args: Vec<String>,
}

impl Default for ClaudeCodeConfig {
    fn default() -> Self {
        Self {
            binary_path: "claude".to_string(),
            api_key: None,
            model: Some("claude-sonnet-4-20250514".to_string()),
            extra_args: Vec::new(),
        }
    }
}

/// Claude Code Adapter
///
/// 通过 claude CLI 与 Claude Code 交互
/// 通信方式：stdin/stdout JSON
pub struct ClaudeCodeAdapter {
    config: ClaudeCodeConfig,
}

impl ClaudeCodeAdapter {
    pub fn new(config: Option<ClaudeCodeConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }

    /// 构建命令参数
    fn build_args(&self, context: &AgentContext) -> Vec<String> {
        let mut args = vec![
            "--print".to_string(),  // 非交互模式
            "--output-format".to_string(),
            "json".to_string(),
        ];

        if let Some(model) = &self.config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        args.extend(self.config.extra_args.clone());

        // 最后添加 prompt
        let prompt = format!(
            "项目: {}\n角色: {}\n任务: {}\n工作目录: {}",
            context.project_id,
            context.role,
            context.task_description,
            context.working_dir
        );
        args.push(prompt);

        args
    }
}

#[async_trait]
impl AgentAdapter for ClaudeCodeAdapter {
    fn name(&self) -> &str {
        "claude-code"
    }

    async fn spawn(&self, context: AgentContext) -> Result<AgentHandle> {
        tracing::info!(
            "🎭 启动 Claude Code Agent: role={}, project={}",
            context.role,
            context.project_id
        );

        let args = self.build_args(&context);
        tracing::debug!("claude 命令: {} {:?}", self.config.binary_path, args);

        let mut cmd = Command::new(&self.config.binary_path);
        cmd.args(&args)
            .env("CLAUDE_PROJECT_ID", &context.project_id)
            .env("CLAUDE_AGENT_ROLE", &context.role)
            .env("CLAUDE_WORKDIR", &context.working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // 如果配置了 API Key，设置环境变量
        if let Some(api_key) = &self.config.api_key {
            cmd.env("ANTHROPIC_API_KEY", api_key);
        }

        let child = cmd.spawn()?;

        let pid = child.id();
        tracing::info!("✅ Claude Code Agent 已启动, PID: {:?}", pid);

        // 将子进程存储以便后续管理
        // 注意：这里需要一个全局的进程管理器，暂时简化处理
        std::mem::forget(child);

        Ok(AgentHandle {
            agent_id: context.agent_id,
            pid,
            adapter_type: "claude-code".to_string(),
        })
    }

    async fn send_task(&self, handle: &AgentHandle, task_description: &str) -> Result<()> {
        tracing::info!("📝 向 Claude Code Agent {} 发送任务", handle.agent_id);
        // Claude Code 使用一次性执行模式，任务在 spawn 时已传入
        // 这里可以实现重试或更新任务的逻辑
        Ok(())
    }

    async fn get_output(&self, handle: &AgentHandle) -> Result<Option<AgentOutput>> {
        // 简化实现：从 stdout 读取
        // 实际应该维护一个输出队列
        Ok(Some(AgentOutput {
            agent_id: handle.agent_id.clone(),
            output_type: "info".to_string(),
            content: "Claude Code 执行中...".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }

    async fn stop(&self, handle: &AgentHandle) -> Result<()> {
        tracing::info!("🛑 停止 Claude Code Agent {}", handle.agent_id);
        if let Some(pid) = handle.pid {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }
        Ok(())
    }

    async fn health_check(&self, handle: &AgentHandle) -> Result<HealthStatus> {
        if let Some(pid) = handle.pid {
            // 检查进程是否存在
            let exists = unsafe { libc::kill(pid as i32, 0) == 0 };
            if exists {
                Ok(HealthStatus::Healthy)
            } else {
                Ok(HealthStatus::Unhealthy {
                    reason: "进程不存在".to_string(),
                })
            }
        } else {
            Ok(HealthStatus::Degraded {
                reason: "无 PID 信息".to_string(),
            })
        }
    }
}
