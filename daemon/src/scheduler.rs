use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use crate::adapter::{AgentContext, AgentLifecycleManager};
use crate::state::{Task, TaskStatus};
use crate::watchdog::Watchdog;

/// 调度器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// 调度检查间隔（秒）
    pub check_interval: u64,
    /// 最大并发任务数
    pub max_concurrent_tasks: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            check_interval: 2,
            max_concurrent_tasks: 10,
        }
    }
}

/// Agent 空闲状态
#[derive(Debug, Clone)]
pub struct AgentSlot {
    pub agent_id: String,
    pub role: String,
    pub is_idle: bool,
}

/// 调度器 — 任务调度 + 依赖检查门控 + 自动 Agent 管理
pub struct Scheduler {
    config: SchedulerConfig,
    /// 待调度任务队列
    pub pending_queue: Arc<RwLock<Vec<Task>>>,
    /// 空闲 Agent 列表
    pub idle_agents: Arc<RwLock<HashMap<String, AgentSlot>>>,
    /// Agent 生命周期管理器（用于实际 spawn + dispatch）
    lifecycle_manager: Arc<Mutex<AgentLifecycleManager>>,
    /// Watchdog（用于注册监控）
    watchdog: Option<Arc<Watchdog>>,
}

impl Scheduler {
    pub fn new(config: SchedulerConfig, lifecycle_manager: Arc<Mutex<AgentLifecycleManager>>) -> Self {
        Self {
            config,
            pending_queue: Arc::new(RwLock::new(Vec::new())),
            idle_agents: Arc::new(RwLock::new(HashMap::new())),
            lifecycle_manager,
            watchdog: None,
        }
    }

    /// 设置 Watchdog（用于注册 Agent 监控）
    pub fn with_watchdog(mut self, watchdog: Arc<Watchdog>) -> Self {
        self.watchdog = Some(watchdog);
        self
    }

    /// 添加任务到调度队列
    pub async fn enqueue(&self, task: Task) -> Result<()> {
        tracing::info!("Task enqueued: {} ({})", task.title, task.id);
        self.pending_queue.write().await.push(task);
        Ok(())
    }

    /// 注册空闲 Agent
    pub async fn register_agent(&self, agent_id: String, role: String) -> Result<()> {
        let slot = AgentSlot {
            agent_id: agent_id.clone(),
            role,
            is_idle: true,
        };
        self.idle_agents
            .write()
            .await
            .insert(agent_id.clone(), slot);
        tracing::info!("Agent {} registered to scheduler", agent_id);
        Ok(())
    }

    /// 获取队列长度
    pub async fn queue_len(&self) -> usize {
        self.pending_queue.read().await.len()
    }

    /// 获取空闲 Agent 数量
    pub async fn idle_agent_count(&self) -> usize {
        self.idle_agents.read().await.len()
    }

    /// 确保指定角色有可用 Agent，没有则自动 spawn
    pub async fn ensure_agent_for_role(
        &self,
        role: &str,
        project_id: &str,
    ) -> Result<Option<String>> {
        // 检查是否已有空闲 agent
        {
            let agents = self.idle_agents.read().await;
            if let Some(agent) = agents.values().find(|a| a.is_idle && a.role == role) {
                return Ok(Some(agent.agent_id.clone()));
            }
        }

        // 没有空闲 agent → 自动 spawn
        let agent_id = format!("{}-{}", role, uuid::Uuid::new_v4().to_string()[..8].to_string());
        tracing::info!(
            "🐱 Auto-spawning Agent: role={}, agent_id={}",
            role,
            agent_id
        );

        let context = AgentContext::new(&agent_id, role, project_id, "");

        let handle = {
            let mut lm = self.lifecycle_manager.lock().await;
            lm.spawn_agent("hermes", context).await?
        };

        // 注册到 watchdog
        if let Some(ref wd) = self.watchdog {
            let _ = wd.register(agent_id.clone(), handle.pid).await;
        }

        // 注册为调度空闲 agent
        self.register_agent(agent_id.clone(), role.to_string())
            .await?;

        Ok(Some(agent_id))
    }

    /// 检查任务依赖是否全部满足
    fn check_dependencies(task: &Task, completed: &std::collections::HashSet<String>) -> bool {
        if task.depends_on.is_empty() {
            return true;
        }
        task.depends_on.iter().all(|dep| completed.contains(dep))
    }

    /// 调度一轮：检查队列，分配任务
    pub async fn schedule_once(
        &self,
        state_manager: &crate::state::StateManager,
        project_id: &str,
    ) -> Result<Vec<(String, Task)>> {
        let mut queue = self.pending_queue.write().await;
        let agents = self.idle_agents.read().await;
        let mut assignments = Vec::new();
        let mut remaining = Vec::new();

        // 获取已完成任务集合
        let project = state_manager.get_project(project_id).await;
        let completed: std::collections::HashSet<String> = project
            .as_ref()
            .map(|p| {
                p.tasks
                    .iter()
                    .filter(|(_, t)| t.status == TaskStatus::Done)
                    .map(|(id, _)| id.clone())
                    .collect()
            })
            .unwrap_or_default();

        for task in queue.drain(..) {
            if assignments.len() >= self.config.max_concurrent_tasks {
                remaining.push(task);
                continue;
            }

            if !Self::check_dependencies(&task, &completed) {
                tracing::debug!("Task {} dependencies not met, re-queued", task.id);
                remaining.push(task);
                continue;
            }

            // 找到对应角色的空闲 Agent
            let assigned_role = task.assigned_to.as_deref().unwrap_or("core_dev");
            let agent_id = agents
                .values()
                .find(|a| a.is_idle && a.role == assigned_role)
                .map(|a| a.agent_id.clone());

            match agent_id {
                Some(id) => {
                    tracing::info!(
                        "Assigning task [{}] → Agent {} ({})",
                        task.title,
                        id,
                        assigned_role
                    );
                    assignments.push((id, task));
                }
                None => {
                    // 没有对应角色的 Agent → 自动 spawn 并分配
                    remaining.push(task);
                }
            }
        }

        *queue = remaining;
        Ok(assignments)
    }

    /// 启动调度循环 — 自动 spawn Agent + 分发任务
    pub async fn start_scheduling(
        self: Arc<Self>,
        state_manager: Arc<crate::state::StateManager>,
        project_id: String,
    ) {
        tracing::info!("Scheduler started, check interval {}s", self.config.check_interval);
        let mut interval =
            tokio::time::interval(std::time::Duration::from_secs(self.config.check_interval));

        loop {
            interval.tick().await;

            // 预处理：检查是否有 pending 任务需要对应角色但没有空闲 agent
            {
                let queue = self.pending_queue.read().await;
                let agents = self.idle_agents.read().await;
                let mut roles_needed: std::collections::HashSet<String> =
                    std::collections::HashSet::new();

                for task in queue.iter() {
                    let role = task.assigned_to.as_deref().unwrap_or("core_dev");
                    let has_idle = agents
                        .values()
                        .any(|a| a.is_idle && a.role == role);
                    if !has_idle {
                        roles_needed.insert(role.to_string());
                    }
                }

                // 释放 locks 后 spawn
                drop(agents);
                drop(queue);

                for role in roles_needed {
                    if let Err(e) = self.ensure_agent_for_role(&role, &project_id).await {
                        tracing::warn!("Auto-spawn failed for role {}: {}", role, e);
                    }
                }
            }

            // 执行调度
            match self.schedule_once(&state_manager, &project_id).await {
                Ok(assignments) => {
                    for (agent_id, task) in assignments {
                        tracing::info!("Dispatching task [{}] → {}", task.title, agent_id);

                        // 通过生命周期管理器实际发送任务给 Agent
                        let lm = self.lifecycle_manager.lock().await;
                        match lm.send_task(&agent_id, &task.description).await {
                            Ok(()) => {
                                tracing::info!("Task sent to Agent {}", agent_id);
                                // 更新任务状态为 active
                                let _ = state_manager
                                    .update_task_status(
                                        &project_id,
                                        &task.id,
                                        TaskStatus::Active,
                                    )
                                    .await;
                            }
                            Err(e) => {
                                tracing::error!("Task dispatch failed: {}, re-queued", e);
                                self.pending_queue.write().await.push(task);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Scheduler error: {}", e);
                }
            }
        }
    }
}
