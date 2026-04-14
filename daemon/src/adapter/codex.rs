use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;

use super::{AgentAdapter, AgentContext, AgentHandle, AgentOutput, HealthStatus};

/// OpenAI Codex 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexConfig {
    /// codex 可执行文件路径
    pub binary_path: String,
    /// OpenAI API Key（可选，也可通过环境变量设置）
    pub api_key: Option<String>,
    /// 模型选择
    pub model: Option<String>,
    /// 额外参数
    pub extra_args: Vec<String>,
}

impl Default for CodexConfig {
    fn default() -> Self {
        Self {
            binary_path: "codex".to_string(),
            api_key: None,
            model: Some("o4-mini".to_string()),
            extra_args: Vec::new(),
        }
    }
}

/// OpenAI Codex Adapter
///
/// 通过 codex CLI 与 OpenAI Codex 交互
/// 通信方式：stdin/stdout
pub struct CodexAdapter {
    config: CodexConfig,
}

impl CodexAdapter {
    pub fn new(config: Option<CodexConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }

    /// 构建命令参数
    fn build_args(&self, context: &AgentContext) -> Vec<String> {
        let mut args = vec![
            "exec".to_string(),  // 执行模式
            "--quiet".to_string(),  // 静默模式
        ];

        if let Some(model) = &self.config.model {
            args.push("--model".to_string());
            args.push(model.clone());
        }

        args.extend(self.config.extra_args.clone());

        // 添加指令
        let instruction = format!(
            "项目: {}\n角色: {}\n任务: {}\n工作目录: {}",
            context.project_id,
            context.role,
            context.task_description,
            context.working_dir
        );
        args.push("--instruction".to_string());
        args.push(instruction);

        args
    }
}

#[async_trait]
impl AgentAdapter for CodexAdapter {
    fn name(&self) -> &str {
        "codex"
    }

    async fn spawn(&self, context: AgentContext) -> Result<AgentHandle> {
        tracing::info!(
            "🤖 启动 Codex Agent: role={}, project={}",
            context.role,
            context.project_id
        );

        let args = self.build_args(&context);
        tracing::debug!("codex 命令: {} {:?}", self.config.binary_path, args);

        let mut cmd = Command::new(&self.config.binary_path);
        cmd.args(&args)
            .env("OPENAI_PROJECT_ID", &context.project_id)
            .env("OPENAI_AGENT_ROLE", &context.role)
            .env("OPENAI_WORKDIR", &context.working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // 如果配置了 API Key，设置环境变量
        if let Some(api_key) = &self.config.api_key {
            cmd.env("OPENAI_API_KEY", api_key);
        }

        let child = cmd.spawn()?;
        let pid = child.id();
        tracing::info!("✅ Codex Agent 已启动, PID: {:?}", pid);

        // 将子进程存储以便后续管理
        std::mem::forget(child);

        Ok(AgentHandle {
            agent_id: context.agent_id,
            pid,
            adapter_type: "codex".to_string(),
        })
    }

    async fn send_task(&self, handle: &AgentHandle, task_description: &str) -> Result<()> {
        tracing::info!("📝 向 Codex Agent {} 发送任务", handle.agent_id);
        // Codex 使用一次性执行模式，任务在 spawn 时已传入
        Ok(())
    }

    async fn get_output(&self, handle: &AgentHandle) -> Result<Option<AgentOutput>> {
        // 简化实现
        Ok(Some(AgentOutput {
            agent_id: handle.agent_id.clone(),
            output_type: "info".to_string(),
            content: "Codex 执行中...".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }))
    }

    async fn stop(&self, handle: &AgentHandle) -> Result<()> {
        tracing::info!("🛑 停止 Codex Agent {}", handle.agent_id);
        if let Some(pid) = handle.pid {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }
        Ok(())
    }

    async fn health_check(&self, handle: &AgentHandle) -> Result<HealthStatus> {
        if let Some(pid) = handle.pid {
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
