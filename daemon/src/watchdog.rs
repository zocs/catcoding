use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Watchdog 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchdogConfig {
    /// Agent 发心跳间隔（秒）
    pub heartbeat_interval: u64,
    /// 3 次没心跳 = 判死（秒）
    pub heartbeat_timeout: u64,
    /// 单任务默认超时（秒）
    pub task_timeout_default: u64,
    /// 编码任务超时（秒）
    pub task_timeout_coding: u64,
    /// 审核任务超时（秒）
    pub task_timeout_review: u64,
    /// 单 agent 最大内存（MB）
    pub max_memory_mb: u64,
    /// CPU 告警阈值（%）
    pub max_cpu_percent: u64,
    /// 单 agent 最多重启次数
    pub max_restart: u32,
    /// 重启间隔冷却（秒）
    pub restart_cooldown: u64,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            heartbeat_interval: 5,
            heartbeat_timeout: 15,
            task_timeout_default: 600,
            task_timeout_coding: 1200,
            task_timeout_review: 300,
            max_memory_mb: 2048,
            max_cpu_percent: 80,
            max_restart: 3,
            restart_cooldown: 10,
        }
    }
}

/// 恢复策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// 等 5 秒（可能是临时抖动）
    GracePeriod,
    /// 读最后日志、内存、CPU，判断原因
    Diagnose,
    /// 恢复上下文，继续跑
    Resume,
    /// 保存快照 → kill → 启动 → 注入上下文
    Restart,
    /// 通知 PM Agent → 通知用户
    Escalate,
}

/// Agent 监控信息
#[derive(Debug, Clone)]
pub struct AgentMonitor {
    pub agent_id: String,
    pub last_heartbeat: DateTime<Utc>,
    pub restart_count: u32,
    pub pid: Option<u32>,
    pub memory_mb: u64,
    pub cpu_percent: f64,
}

/// 猫头鹰 Watchdog — 进程监管模块
///
/// 三重检测机制：
/// 1. 管道 EOF（即时，<1ms）→ 进程崩溃/被 kill
/// 2. 心跳超时（中速，15s）→ 进程死循环/GC 卡死
/// 3. /proc 轮询（兜底，5s）→ 僵尸进程/特殊情况
pub struct Watchdog {
    config: WatchdogConfig,
    agents: Arc<RwLock<HashMap<String, AgentMonitor>>>,
}

impl Watchdog {
    pub fn new(config: WatchdogConfig) -> Self {
        Self {
            config,
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册要监控的 Agent
    pub async fn register(&self, agent_id: String, pid: Option<u32>) -> Result<()> {
        let id = agent_id.clone();
        let monitor = AgentMonitor {
            agent_id: id,
            last_heartbeat: Utc::now(),
            restart_count: 0,
            pid,
            memory_mb: 0,
            cpu_percent: 0.0,
        };
        self.agents.write().await.insert(agent_id.clone(), monitor);
        tracing::info!("🦉 Watchdog 已注册监控 Agent: {}", agent_id);
        Ok(())
    }

    /// 更新 Agent 心跳
    pub async fn heartbeat(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.write().await;
        if let Some(monitor) = agents.get_mut(agent_id) {
            monitor.last_heartbeat = Utc::now();
        }
        Ok(())
    }

    /// 检查所有 Agent 状态
    pub async fn check_all(&self) -> Vec<(String, RecoveryAction)> {
        let agents = self.agents.read().await;
        let now = Utc::now();
        let mut actions = Vec::new();

        for (agent_id, monitor) in agents.iter() {
            let elapsed = (now - monitor.last_heartbeat).num_seconds() as u64;

            let action = if elapsed > self.config.heartbeat_timeout {
                if monitor.restart_count >= self.config.max_restart {
                    tracing::warn!(
                        "🦉 Agent {} 超过重启上限 ({}次)，通知 PM",
                        agent_id,
                        self.config.max_restart
                    );
                    RecoveryAction::Escalate
                } else {
                    tracing::warn!(
                        "🦉 Agent {} 心跳超时 ({}s > {}s)，准备重启",
                        agent_id,
                        elapsed,
                        self.config.heartbeat_timeout
                    );
                    RecoveryAction::Restart
                }
            } else if elapsed > self.config.heartbeat_timeout / 2 {
                RecoveryAction::Diagnose
            } else if elapsed > self.config.heartbeat_interval * 2 {
                RecoveryAction::GracePeriod
            } else {
                continue; // 正常，跳过
            };

            actions.push((agent_id.clone(), action));
        }

        actions
    }

    /// 启动监控循环
    pub async fn start_monitoring(self: Arc<Self>) {
        tracing::info!("🦉 Watchdog 监控循环启动，间隔 {}s", self.config.heartbeat_interval);
        let mut interval = tokio::time::interval(
            std::time::Duration::from_secs(self.config.heartbeat_interval),
        );

        loop {
            interval.tick().await;
            let actions = self.check_all().await;

            for (agent_id, action) in actions {
                match action {
                    RecoveryAction::GracePeriod => {
                        tracing::info!("🦉 Agent {} 进入 GRACE_PERIOD", agent_id);
                    }
                    RecoveryAction::Diagnose => {
                        tracing::info!("🦉 Agent {} 正在诊断...", agent_id);
                        // TODO: 读取 /proc 信息
                    }
                    RecoveryAction::Restart => {
                        tracing::warn!("🦉 Agent {} 执行 RESTART", agent_id);
                        // TODO: 保存快照 → kill → 启动 → 注入上下文
                    }
                    RecoveryAction::Escalate => {
                        tracing::error!("🦉 Agent {} 需要 ESCALATE（通知 PM）", agent_id);
                        // TODO: 通知 PM Agent
                    }
                    _ => {}
                }
            }
        }
    }
}
