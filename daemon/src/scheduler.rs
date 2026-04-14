use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::{Task, TaskStatus};

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

/// 调度器 — 任务调度 + 依赖检查门控
///
/// 职责：
/// - 从任务队列读取新任务
/// - 检查任务依赖链是否就绪
/// - 就绪任务分配给对应角色的空闲 agent
/// - 不就绪的任务放回队列尾部，不浪费 agent token
pub struct Scheduler {
    config: SchedulerConfig,
    /// 待调度任务队列
    pub pending_queue: Arc<RwLock<Vec<Task>>>,
    /// 空闲 Agent 列表
    pub idle_agents: Arc<RwLock<HashMap<String, AgentSlot>>>,
}

impl Scheduler {
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            config,
            pending_queue: Arc::new(RwLock::new(Vec::new())),
            idle_agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加任务到调度队列
    pub async fn enqueue(&self, task: Task) -> Result<()> {
        tracing::info!("📋 任务入队: {} ({})", task.title, task.id);
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
        self.idle_agents.write().await.insert(agent_id.clone(), slot);
        tracing::info!("🐱 Agent {} 已注册到调度器", agent_id);
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
                tracing::debug!("⏳ 任务 {} 依赖未满足，放回队列", task.id);
                remaining.push(task);
                continue;
            }

            // 找到对应角色的空闲 Agent
            let assigned_role = task.assigned_to.as_deref().unwrap_or("core_dev");
            let available = agents.values().find(|a| a.is_idle && a.role == assigned_role);

            match available {
                Some(agent) => {
                    tracing::info!(
                        "🐱 分配任务 [{}] → Agent {} ({})",
                        task.title,
                        agent.agent_id,
                        agent.role
                    );
                    assignments.push((agent.agent_id.clone(), task));
                }
                None => {
                    // 没有对应角色的 Agent，任务留在队列
                    remaining.push(task);
                }
            }
        }

        *queue = remaining;
        Ok(assignments)
    }

    /// 启动调度循环
    pub async fn start_scheduling(
        self: Arc<Self>,
        state_manager: Arc<crate::state::StateManager>,
        project_id: String,
    ) {
        tracing::info!(
            "📋 调度器启动，检查间隔 {}s",
            self.config.check_interval
        );
        let mut interval = tokio::time::interval(
            std::time::Duration::from_secs(self.config.check_interval),
        );

        loop {
            interval.tick().await;
            match self.schedule_once(&state_manager, &project_id).await {
                Ok(assignments) => {
                    for (agent_id, task) in assignments {
                        tracing::info!("🚀 任务 [{}] 已分配给 {}", task.title, agent_id);
                        // TODO: 通过 Router 发送任务给 Agent
                    }
                }
                Err(e) => {
                    tracing::error!("调度器错误: {}", e);
                }
            }
        }
    }
}
