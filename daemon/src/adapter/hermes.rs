use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

use super::{AgentAdapter, AgentContext, AgentHandle, AgentOutput, HealthStatus};

/// Hermes Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HermesConfig {
    /// Python 解释器路径
    pub python_path: String,
    /// Agent SDK 根目录
    pub agents_dir: String,
}

impl Default for HermesConfig {
    fn default() -> Self {
        Self {
            python_path: "python3".to_string(),
            agents_dir: "agents".to_string(),
        }
    }
}

/// 运行中的 Agent 进程状态
struct RunningAgent {
    child: Child,
    stdout_reader: Arc<Mutex<BufReader<tokio::process::ChildStdout>>>,
}

/// Hermes Agent Adapter
///
/// 直接 spawn Python Agent 进程，通过 stdin/stdout JSON 通信
pub struct HermesAdapter {
    config: HermesConfig,
    /// 运行中的 Agent 进程（持有 stdin/stdout handle）
    running: Arc<Mutex<std::collections::HashMap<String, RunningAgent>>>,
}

impl HermesAdapter {
    pub fn new(config: Option<HermesConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            running: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    /// 获取 Agent 启动命令
    fn build_command(&self, context: &AgentContext) -> (String, Vec<String>) {
        let runner = format!("{}/run_agent.py", self.config.agents_dir);
        let args = vec![
            context.role.clone(),
            "--agent-id".to_string(),
            context.agent_id.clone(),
            "--project-id".to_string(),
            context.project_id.clone(),
            "--workdir".to_string(),
            context.working_dir.clone(),
        ];
        (
            self.config.python_path.clone(),
            std::iter::once(runner).chain(args).collect(),
        )
    }
}

#[async_trait]
impl AgentAdapter for HermesAdapter {
    fn name(&self) -> &str {
        "hermes"
    }

    async fn spawn(&self, context: AgentContext) -> Result<AgentHandle> {
        tracing::info!(
            "Starting Python Agent: role={}, project={}, agent_id={}",
            context.role,
            context.project_id,
            context.agent_id
        );

        let (python, args) = self.build_command(&context);

        let mut child = Command::new(&python)
            .args(&args)
            .env("AGENT_ID", &context.agent_id)
            .env("PROJECT_ID", &context.project_id)
            .env("WORKDIR", &context.working_dir)
            .env("ROLE", &context.role)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let pid = child.id();
        let stdout = child.stdout.take().unwrap();
        let stdout_reader = Arc::new(Mutex::new(BufReader::new(stdout)));

        let agent_id = context.agent_id.clone();

        // 保存运行中进程
        self.running.lock().await.insert(
            agent_id.clone(),
            RunningAgent {
                child,
                stdout_reader: stdout_reader.clone(),
            },
        );

        tracing::info!("Python Agent {} started, PID: {:?}", agent_id, pid);

        Ok(AgentHandle {
            agent_id,
            pid,
            adapter_type: "hermes".to_string(),
        })
    }

    async fn send_task(&self, handle: &AgentHandle, task_description: &str) -> Result<()> {
        tracing::info!(
            "Sending task to Agent {}: {}",
            handle.agent_id,
            &task_description[..task_description.len().min(80)]
        );

        let mut running = self.running.lock().await;
        let agent = running
            .get_mut(&handle.agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent {} not running", handle.agent_id))?;

        // 构造任务分配消息（与 Python Agent 的 AgentMessage 兼容）
        let msg = serde_json::json!({
            "msg_id": uuid::Uuid::new_v4().to_string(),
            "type": "task.assign",
            "from": "daemon",
            "to": handle.agent_id,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "task_id": uuid::Uuid::new_v4().to_string(),
            "summary": task_description,
        });

        let line = format!("{}\n", msg);
        if let Some(stdin) = agent.child.stdin.as_mut() {
            stdin.write_all(line.as_bytes()).await?;
            stdin.flush().await?;
            tracing::debug!("Wrote {} bytes to Agent stdin", line.len());
        } else {
            anyhow::bail!("Agent {} stdin closed", handle.agent_id);
        }

        Ok(())
    }

    async fn get_output(&self, handle: &AgentHandle) -> Result<Option<AgentOutput>> {
        let mut running = self.running.lock().await;
        let agent = running
            .get_mut(&handle.agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent {} not running", handle.agent_id))?;

        let mut line = String::new();
        let n = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            agent.stdout_reader.lock().await.read_line(&mut line),
        )
        .await;

        match n {
            Ok(Ok(0)) => {
                // EOF — Agent 进程已退出
                tracing::info!("📭 Agent {} stdout EOF", handle.agent_id);
                Ok(None)
            }
            Ok(Ok(_)) => {
                let line = line.trim().to_string();
                if line.is_empty() {
                    return Ok(None);
                }

                // 解析 JSON 输出
                let output_type = if let Ok(val) = serde_json::from_str::<serde_json::Value>(&line)
                {
                    val.get("type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string()
                } else {
                    "raw".to_string()
                };

                Ok(Some(AgentOutput {
                    agent_id: handle.agent_id.clone(),
                    output_type,
                    content: line,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                }))
            }
            Ok(Err(e)) => {
                tracing::error!("Error reading Agent {} stdout: {}", handle.agent_id, e);
                Ok(None)
            }
            Err(_) => {
                // 超时 — 没有新输出
                Ok(None)
            }
        }
    }

    async fn stop(&self, handle: &AgentHandle) -> Result<()> {
        tracing::info!("Stopping Agent: {}", handle.agent_id);

        let mut running = self.running.lock().await;
        if let Some(mut agent) = running.remove(&handle.agent_id) {
            // 先发送停止消息
            if let Some(stdin) = agent.child.stdin.as_mut() {
                let stop_msg = serde_json::json!({
                    "type": "stop",
                    "from": "daemon",
                    "to": handle.agent_id,
                });
                let line = format!("{}\n", stop_msg);
                let _ = stdin.write_all(line.as_bytes()).await;
            }

            // 等待 3 秒
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;

            // 如果还活着就杀掉
            match agent.child.try_wait()? {
                Some(status) => {
                    tracing::info!("Agent {} exited: {}", handle.agent_id, status);
                }
                None => {
                    tracing::warn!("Agent {} did not exit, sending SIGKILL", handle.agent_id);
                    agent.child.kill().await?;
                }
            }
        }

        Ok(())
    }

    async fn health_check(&self, handle: &AgentHandle) -> Result<HealthStatus> {
        let running = self.running.lock().await;
        match running.get(&handle.agent_id) {
            Some(_agent) => {
                // 检查进程是否还活着
                match handle.pid {
                    Some(pid) => {
                        let proc_path = format!("/proc/{}", pid);
                        if std::path::Path::new(&proc_path).exists() {
                            Ok(HealthStatus::Healthy)
                        } else {
                            Ok(HealthStatus::Unhealthy {
                                reason: format!("Process {} not found", pid),
                            })
                        }
                    }
                    None => Ok(HealthStatus::Unhealthy {
                        reason: "No PID info available".to_string(),
                    }),
                }
            }
            None => Ok(HealthStatus::Unhealthy {
                reason: "Agent not running".to_string(),
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
        let adapter = self
            .adapters
            .get(adapter_name)
            .ok_or_else(|| anyhow::anyhow!("Adapter not found: {}", adapter_name))?;

        let handle = adapter.spawn(context.clone()).await?;
        self.handles
            .insert(context.agent_id.clone(), handle.clone());

        Ok(handle)
    }

    /// 发送任务
    pub async fn send_task(&self, agent_id: &str, task: &str) -> Result<()> {
        let handle = self
            .handles
            .get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        let adapter = self
            .adapters
            .get(&handle.adapter_type)
            .ok_or_else(|| anyhow::anyhow!("Adapter not found: {}", handle.adapter_type))?;

        adapter.send_task(handle, task).await
    }

    /// 读取输出
    pub async fn get_output(&self, agent_id: &str) -> Result<Option<AgentOutput>> {
        let handle = self
            .handles
            .get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        let adapter = self
            .adapters
            .get(&handle.adapter_type)
            .ok_or_else(|| anyhow::anyhow!("Adapter not found: {}", handle.adapter_type))?;

        adapter.get_output(handle).await
    }

    /// 停止 Agent
    pub async fn stop_agent(&self, agent_id: &str) -> Result<()> {
        let handle = self
            .handles
            .get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        let adapter = self
            .adapters
            .get(&handle.adapter_type)
            .ok_or_else(|| anyhow::anyhow!("Adapter not found: {}", handle.adapter_type))?;

        adapter.stop(handle).await
    }

    /// 健康检查
    pub async fn check_health(&self, agent_id: &str) -> Result<HealthStatus> {
        let handle = self
            .handles
            .get(agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        let adapter = self
            .adapters
            .get(&handle.adapter_type)
            .ok_or_else(|| anyhow::anyhow!("Adapter not found: {}", handle.adapter_type))?;

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
                tracing::info!("Stopping Agent: {}", agent_id);
                let _ = adapter.stop(handle).await;
            }
        }
        Ok(())
    }
}
