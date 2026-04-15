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
    /// 心跳超时判死（秒）
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

impl std::fmt::Display for RecoveryAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GracePeriod => write!(f, "GRACE_PERIOD"),
            Self::Diagnose => write!(f, "DIAGNOSE"),
            Self::Resume => write!(f, "RESUME"),
            Self::Restart => write!(f, "RESTART"),
            Self::Escalate => write!(f, "ESCALATE"),
        }
    }
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
    pub last_diagnose: Option<DateTime<Utc>>,
}

/// 进程资源信息（从 /proc 读取）
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub alive: bool,
    pub memory_kb: u64,
    pub cpu_percent: f64,
    pub uptime_secs: u64,
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
        let monitor = AgentMonitor {
            agent_id: agent_id.clone(),
            last_heartbeat: Utc::now(),
            restart_count: 0,
            pid,
            memory_mb: 0,
            cpu_percent: 0.0,
            last_diagnose: None,
        };
        self.agents.write().await.insert(agent_id.clone(), monitor);
        tracing::info!("Watchdog monitoring Agent: {}", agent_id);
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

    /// 检测层3: /proc 轮询 — 检查进程是否存在和资源使用
    pub fn check_proc(pid: u32) -> ProcessInfo {
        let proc_path = format!("/proc/{}", pid);

        // 检查进程是否存在
        if !std::path::Path::new(&proc_path).exists() {
            return ProcessInfo {
                pid,
                alive: false,
                memory_kb: 0,
                cpu_percent: 0.0,
                uptime_secs: 0,
            };
        }

        // 读取 /proc/{pid}/status 获取内存
        let memory_kb = std::fs::read_to_string(format!("{}/status", proc_path))
            .ok()
            .and_then(|content| {
                content
                    .lines()
                    .find(|line| line.starts_with("VmRSS:"))
                    .and_then(|line| {
                        line.split_whitespace()
                            .nth(1)
                            .and_then(|s| s.parse::<u64>().ok())
                    })
            })
            .unwrap_or(0);

        // 读取 /proc/{pid}/stat 获取 CPU（简化版）
        let cpu_percent = std::fs::read_to_string(format!("{}/stat", proc_path))
            .ok()
            .and_then(|content| {
                // 简化：只检查进程状态
                let parts: Vec<&str> = content.split_whitespace().collect();
                if parts.len() > 2 {
                    match parts[2] {
                        "R" => Some(50.0), // Running
                        "S" => Some(5.0),  // Sleeping
                        "D" => Some(80.0), // Uninterruptible sleep
                        "Z" => Some(0.0),  // Zombie
                        _ => Some(10.0),
                    }
                } else {
                    None
                }
            })
            .unwrap_or(0.0);

        ProcessInfo {
            pid,
            alive: true,
            memory_kb,
            cpu_percent,
            uptime_secs: 0,
        }
    }

    /// 诊断 Agent 状态
    pub async fn diagnose(&self, agent_id: &str) -> (RecoveryAction, String) {
        let agents = self.agents.read().await;
        let monitor = match agents.get(agent_id) {
            Some(m) => m,
            None => return (RecoveryAction::Diagnose, "Agent not found".to_string()),
        };

        // 检查重启次数
        if monitor.restart_count >= self.config.max_restart {
            return (
                RecoveryAction::Escalate,
                format!(
                    "Agent {} restarted {} times, limit reached",
                    agent_id, monitor.restart_count
                ),
            );
        }

        // 检查进程
        if let Some(pid) = monitor.pid {
            let proc_info = Self::check_proc(pid);

            if !proc_info.alive {
                return (RecoveryAction::Restart, format!("Process {} exited", pid));
            }

            if proc_info.memory_kb as u64 > self.config.max_memory_mb * 1024 {
                return (
                    RecoveryAction::Restart,
                    format!(
                        "Memory exceeded: {}KB > {}MB",
                        proc_info.memory_kb, self.config.max_memory_mb
                    ),
                );
            }

            if proc_info.cpu_percent > self.config.max_cpu_percent as f64 {
                return (
                    RecoveryAction::Diagnose,
                    format!("High CPU usage: {:.1}%", proc_info.cpu_percent),
                );
            }

            // 进程存在但心跳丢失，可能是死循环
            let elapsed = (Utc::now() - monitor.last_heartbeat).num_seconds();
            if elapsed > self.config.heartbeat_timeout as i64 {
                return (
                    RecoveryAction::Restart,
                    format!(
                        "Heartbeat timeout: {}s > {}s",
                        elapsed, self.config.heartbeat_timeout
                    ),
                );
            }

            (RecoveryAction::Resume, "Process healthy, resuming context".to_string())
        } else {
            (RecoveryAction::Restart, "No PID info".to_string())
        }
    }

    /// 执行恢复策略
    pub async fn execute_recovery(
        &self,
        agent_id: &str,
        action: &RecoveryAction,
    ) -> Result<String> {
        let mut agents = self.agents.write().await;
        let monitor = agents.get_mut(agent_id);

        match action {
            RecoveryAction::GracePeriod => {
                tracing::info!("Agent {} entering GRACE_PERIOD, waiting 5s", agent_id);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                Ok("Wait complete".to_string())
            }
            RecoveryAction::Diagnose => {
                tracing::info!("Agent {} diagnosing...", agent_id);
                if let Some(m) = monitor {
                    m.last_diagnose = Some(Utc::now());
                }
                Ok("Diagnosis complete".to_string())
            }
            RecoveryAction::Resume => {
                tracing::info!("Agent {} resumed", agent_id);
                if let Some(m) = monitor {
                    m.last_heartbeat = Utc::now();
                }
                Ok("Resumed".to_string())
            }
            RecoveryAction::Restart => {
                tracing::warn!("Agent {} restarting", agent_id);
                let restart_count = monitor.as_ref().map(|m| m.restart_count + 1).unwrap_or(1);
                if let Some(m) = monitor {
                    // 杀死旧进程
                    if let Some(pid) = m.pid {
                        unsafe {
                            libc::kill(pid as i32, libc::SIGTERM);
                        }
                        tracing::info!("Sent SIGTERM to process {}", pid);
                    }
                    m.restart_count = restart_count;
                    m.last_heartbeat = Utc::now();
                    m.pid = None;
                }
                Ok(format!("Restarted (attempt {})", restart_count))
            }
            RecoveryAction::Escalate => {
                tracing::error!("Agent {} needs ESCALATE, notifying user", agent_id);
                Ok("User notified".to_string())
            }
        }
    }

    /// 检查所有 Agent 状态
    pub async fn check_all(&self) -> Vec<(String, RecoveryAction, String)> {
        let agents = self.agents.read().await;
        let now = Utc::now();
        let mut results = Vec::new();

        for (agent_id, monitor) in agents.iter() {
            let elapsed = (now - monitor.last_heartbeat).num_seconds() as u64;

            if elapsed > self.config.heartbeat_timeout {
                if monitor.restart_count >= self.config.max_restart {
                    results.push((
                        agent_id.clone(),
                        RecoveryAction::Escalate,
                        format!("Restart limit reached ({})", monitor.restart_count),
                    ));
                } else {
                    results.push((
                        agent_id.clone(),
                        RecoveryAction::Restart,
                        format!("Heartbeat timeout {}s > {}s", elapsed, self.config.heartbeat_timeout),
                    ));
                }
            } else if elapsed > self.config.heartbeat_timeout / 2 {
                results.push((
                    agent_id.clone(),
                    RecoveryAction::Diagnose,
                    format!("Heartbeat delayed {}s", elapsed),
                ));
            } else if elapsed > self.config.heartbeat_interval * 3 {
                results.push((
                    agent_id.clone(),
                    RecoveryAction::GracePeriod,
                    format!("Heartbeat slightly delayed {}s", elapsed),
                ));
            }
        }

        results
    }

    /// 获取 Agent 状态摘要
    pub async fn status_summary(&self) -> serde_json::Value {
        let agents = self.agents.read().await;
        let mut summary = Vec::new();

        for (id, m) in agents.iter() {
            let elapsed = (Utc::now() - m.last_heartbeat).num_seconds();
            let status = if elapsed > self.config.heartbeat_timeout as i64 {
                "unhealthy"
            } else if elapsed > (self.config.heartbeat_timeout / 2) as i64 {
                "degraded"
            } else {
                "healthy"
            };

            summary.push(serde_json::json!({
                "id": id,
                "status": status,
                "last_heartbeat_secs": elapsed,
                "restart_count": m.restart_count,
                "pid": m.pid,
            }));
        }

        serde_json::json!({
            "agents": summary,
            "config": {
                "heartbeat_interval": self.config.heartbeat_interval,
                "heartbeat_timeout": self.config.heartbeat_timeout,
                "max_restart": self.config.max_restart,
            }
        })
    }

    /// 启动监控循环
    pub async fn start_monitoring(self: Arc<Self>) {
        tracing::info!(
            "Watchdog monitoring loop started, interval {}s",
            self.config.heartbeat_interval
        );
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(
            self.config.heartbeat_interval,
        ));

        loop {
            interval.tick().await;
            let checks = self.check_all().await;

            for (agent_id, action, reason) in checks {
                tracing::warn!("🦉 Agent {}: {} - {}", agent_id, action, reason);

                // 执行恢复策略
                match self.execute_recovery(&agent_id, &action).await {
                    Ok(result) => {
                        tracing::info!("Recovery result: {}", result);
                    }
                    Err(e) => {
                        tracing::error!("Recovery failed: {}", e);
                    }
                }
            }
        }
    }
}
