use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

use super::{AgentAdapter, AgentContext, AgentHandle, AgentOutput, HealthStatus};

/// Hermes Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesConfig {
    /// hermes-agent 可执行文件路径
    pub binary_path: String,
    /// 默认 profile
    pub profile: Option<String>,
    /// 额外参数
    pub extra_args: Vec<String>,
}

impl Default for HermesConfig {
    fn default() -> Self {
        Self {
            binary_path: "hermes".to_string(),
            profile: None,
            extra_args: Vec::new(),
        }
    }
}

/// Hermes Agent Adapter
///
/// 通过 hermes CLI 与 Hermes Agent 交互
/// 通信方式：stdin/stdout JSON-RPC
pub struct HermesAdapter {
    config: HermesConfig,
}

impl HermesAdapter {
    pub fn new(config: Option<HermesConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }

    /// 构建命令参数
    fn build_args(&self, context: &AgentContext) -> Vec<String> {
        let mut args = vec![
            "chat".to_string(),
            "--query".to_string(),
            context.task_description.clone(),
            "--worktree".to_string(),  // 隔离工作目录
        ];

        if let Some(profile) = &self.config.profile {
            args.push("--profile".to_string());
            args.push(profile.clone());
        }

        args.extend(self.config.extra_args.clone());
        args
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

        let args = self.build_args(&context);
        tracing::debug!("hermes 命令: {} {:?}", self.config.binary_path, args);

        let child = Command::new(&self.config.binary_path)
            .args(&args)
            .env("HERMES_PROJECT_ID", &context.project_id)
            .env("HERMES_AGENT_ROLE", &context.role)
            .env("HERMES_WORKDIR", &context.working_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let pid = child.id();

        tracing::info!("✅ Hermes Agent 已启动, PID: {:?}", pid);

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
            &task_description[..task_description.len().min(80)]
        );

        // TODO: 通过 stdin 发送 JSON-RPC 消息
        // 当前使用 one-shot 模式，后续支持交互模式

        Ok(())
    }

    async fn get_output(&self, _handle: &AgentHandle) -> Result<Option<AgentOutput>> {
        // TODO: 从 stdout 读取 Agent 输出
        // 需要持有 Child 进程的 stdout handle
        Ok(None)
    }

    async fn stop(&self, handle: &AgentHandle) -> Result<()> {
        tracing::info!("🛑 停止 Hermes Agent: {}", handle.agent_id);

        if let Some(pid) = handle.pid {
            // 优雅停止：先 SIGTERM，等待 5s，再 SIGKILL
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
            tracing::info!("📤 已发送 SIGTERM 给进程 {}", pid);

            // 等待进程退出
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            // 检查是否还活着
            let proc_path = format!("/proc/{}", pid);
            if std::path::Path::new(&proc_path).exists() {
                tracing::warn!("⚠️ 进程 {} 未退出，发送 SIGKILL", pid);
                unsafe {
                    libc::kill(pid as i32, libc::SIGKILL);
                }
            }
        }

        Ok(())
    }

    async fn health_check(&self, handle: &AgentHandle) -> Result<HealthStatus> {
        match handle.pid {
            Some(pid) => {
                let proc_path = format!("/proc/{}", pid);
                if std::path::Path::new(&proc_path).exists() {
                    // 检查进程状态
                    let status = std::fs::read_to_string(format!("{}/status", proc_path))
                        .unwrap_or_default();

                    if status.contains("State:\tZ") {
                        // Zombie 进程
                        Ok(HealthStatus::Unhealthy {
                            reason: format!("进程 {} 是僵尸进程", pid),
                        })
                    } else {
                        Ok(HealthStatus::Healthy)
                    }
                } else {
                    Ok(HealthStatus::Unhealthy {
                        reason: format!("进程 {} 不存在", pid),
                    })
                }
            }
            None => Ok(HealthStatus::Degraded {
                reason: "无 PID 信息".to_string(),
            }),
        }
    }
}

/// Agent 生命周期管理器
///
/// 管理所有 Agent 的启动、监控、停止
pub struct AgentLifecycleManager {
    adapters: std::collections::HashMap<String, Box<dyn AgentAdapter>>,
    handles: std::collections::HashMap<String, AgentHandle>,
}

impl AgentLifecycleManager {
    pub fn new() -> Self {
        let mut manager = Self {
            adapters: std::collections::HashMap::new(),
            handles: std::collections::HashMap::new(),
        };

        // 注册默认 adapter
        manager.register_adapter("hermes", Box::new(HermesAdapter::new(None)));

        manager
    }

    /// 注册 Adapter
    pub fn register_adapter(&mut self, name: &str, adapter: Box<dyn AgentAdapter>) {
        self.adapters.insert(name.to_string(), adapter);
    }

    /// 启动 Agent
    pub async fn spawn_agent(
        &mut self,
        adapter_name: &str,
        context: AgentContext,
    ) -> Result<AgentHandle> {
        let adapter = self.adapters.get(adapter_name)
            .ok_or_else(|| anyhow::anyhow!("未找到 Adapter: {}", adapter_name))?;

        let handle = adapter.spawn(context.clone()).await?;
        self.handles.insert(context.agent_id.clone(), handle.clone());

        Ok(handle)
    }

    /// 发送任务
    pub async fn send_task(&self, agent_id: &str, task: &str) -> Result<()> {
        let handle = self.handles.get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("未找到 Agent: {}", agent_id))?;

        let adapter = self.adapters.get(&handle.adapter_type)
            .ok_or_else(|| anyhow::anyhow!("未找到 Adapter: {}", handle.adapter_type))?;

        adapter.send_task(handle, task).await
    }

    /// 停止 Agent
    pub async fn stop_agent(&self, agent_id: &str) -> Result<()> {
        let handle = self.handles.get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("未找到 Agent: {}", agent_id))?;

        let adapter = self.adapters.get(&handle.adapter_type)
            .ok_or_else(|| anyhow::anyhow!("未找到 Adapter: {}", handle.adapter_type))?;

        adapter.stop(handle).await
    }

    /// 健康检查
    pub async fn check_health(&self, agent_id: &str) -> Result<HealthStatus> {
        let handle = self.handles.get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("未找到 Agent: {}", agent_id))?;

        let adapter = self.adapters.get(&handle.adapter_type)
            .ok_or_else(|| anyhow::anyhow!("未找到 Adapter: {}", handle.adapter_type))?;

        adapter.health_check(handle).await
    }

    /// 列出所有 Agent
    pub fn list_agents(&self) -> Vec<&AgentHandle> {
        self.handles.values().collect()
    }

    /// 停止所有 Agent
    pub async fn stop_all(&self) -> Result<()> {
        for (agent_id, handle) in &self.handles {
            if let Some(adapter) = self.adapters.get(&handle.adapter_type) {
                tracing::info!("🛑 停止 Agent: {}", agent_id);
                let _ = adapter.stop(handle).await;
            }
        }
        Ok(())
    }
}
