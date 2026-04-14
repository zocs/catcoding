use anyhow::Result;
use async_trait::async_trait;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

use super::{AgentAdapter, AgentContext, AgentHandle, AgentOutput, HealthStatus};

/// Hermes Agent Adapter
///
/// 通过 hermes-agent CLI 与 Hermes Agent 交互
/// 通信方式：stdin/stdout JSON-RPC
pub struct HermesAdapter {
    /// hermes-agent 可执行文件路径
    binary_path: String,
}

impl HermesAdapter {
    pub fn new(binary_path: Option<String>) -> Self {
        Self {
            binary_path: binary_path.unwrap_or_else(|| "hermes-agent".to_string()),
        }
    }
}

#[async_trait]
impl AgentAdapter for HermesAdapter {
    fn name(&self) -> &str {
        "hermes"
    }

    async fn spawn(&self, context: AgentContext) -> Result<AgentHandle> {
        tracing::info!(
            "🐱 启动 Hermes Agent: role={}, project={}",
            context.role,
            context.project_id
        );

        let child = Command::new(&self.binary_path)
            .arg("--role")
            .arg(&context.role)
            .arg("--project")
            .arg(&context.project_id)
            .arg("--workdir")
            .arg(&context.working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let pid = child.id();

        // 子进程由 Watchdog 管理，这里暂不持有
        // TODO: 将 child 存入进程池，由 Watchdog 监控

        Ok(AgentHandle {
            agent_id: context.agent_id,
            pid,
            adapter_type: "hermes".to_string(),
        })
    }

    async fn send_task(&self, handle: &AgentHandle, task_description: &str) -> Result<()> {
        tracing::info!(
            "📨 发送任务给 Hermes Agent {}: {}",
            handle.agent_id,
            &task_description[..task_description.len().min(50)]
        );
        // TODO: 通过 stdin 发送 JSON-RPC 消息
        Ok(())
    }

    async fn get_output(&self, _handle: &AgentHandle) -> Result<Option<AgentOutput>> {
        // TODO: 从 stdout 读取 Agent 输出
        Ok(None)
    }

    async fn stop(&self, handle: &AgentHandle) -> Result<()> {
        tracing::info!("🛑 停止 Hermes Agent: {}", handle.agent_id);
        if let Some(pid) = handle.pid {
            // 优雅停止：先 SIGTERM，等待 5s，再 SIGKILL
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }
        Ok(())
    }

    async fn health_check(&self, handle: &AgentHandle) -> Result<HealthStatus> {
        match handle.pid {
            Some(pid) => {
                // 检查 /proc/{pid} 是否存在
                let proc_path = format!("/proc/{}", pid);
                if std::path::Path::new(&proc_path).exists() {
                    Ok(HealthStatus::Healthy)
                } else {
                    Ok(HealthStatus::Unhealthy {
                        reason: format!("进程 {} 不存在", pid),
                    })
                }
            }
            None => Ok(HealthStatus::Unhealthy {
                reason: "无 PID".to_string(),
            }),
        }
    }
}
