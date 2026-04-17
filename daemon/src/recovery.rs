use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 故障场景
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FailureScenario {
    /// NATS 断连
    NatsDisconnect,
    /// Agent 超时
    AgentTimeout,
    /// 编译失败
    CompileFailure,
    /// API 限流
    ApiRateLimit,
    /// Dashboard 断连
    DashboardDisconnect,
    /// 自定义场景
    Custom(String),
}

impl std::fmt::Display for FailureScenario {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NatsDisconnect => write!(f, "NATS_DISCONNECT"),
            Self::AgentTimeout => write!(f, "AGENT_TIMEOUT"),
            Self::CompileFailure => write!(f, "COMPILE_FAILURE"),
            Self::ApiRateLimit => write!(f, "API_RATE_LIMIT"),
            Self::DashboardDisconnect => write!(f, "DASHBOARD_DISCONNECT"),
            Self::Custom(name) => write!(f, "CUSTOM_{}", name.to_uppercase()),
        }
    }
}

/// 恢复步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStep {
    /// 重连服务
    Reconnect { service: String },
    /// 重启进程
    RestartProcess { agent_id: String },
    /// 清理构建
    CleanBuild,
    /// 指数退避重试
    RetryWithBackoff { max_retries: u32 },
    /// 切换 provider
    SwitchProvider { fallback: String },
    /// 升级到人工处理
    EscalateToHuman { reason: String },
    /// 等待一段时间
    Wait { seconds: u64 },
    /// 重新订阅
    Resubscribe { topics: Vec<String> },
    /// 重建连接
    RebuildConnection { endpoint: String },
}

impl std::fmt::Display for RecoveryStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Reconnect { service } => write!(f, "RECONNECT_{}", service.to_uppercase()),
            Self::RestartProcess { agent_id } => write!(f, "RESTART_{}", agent_id),
            Self::CleanBuild => write!(f, "CLEAN_BUILD"),
            Self::RetryWithBackoff { max_retries } => {
                write!(f, "RETRY_WITH_BACKOFF(max={})", max_retries)
            }
            Self::SwitchProvider { fallback } => write!(f, "SWITCH_TO_{}", fallback),
            Self::EscalateToHuman { reason } => write!(f, "ESCALATE: {}", reason),
            Self::Wait { seconds } => write!(f, "WAIT_{}s", seconds),
            Self::Resubscribe { topics } => write!(f, "RESUBSCRIBE({})", topics.join(",")),
            Self::RebuildConnection { endpoint } => write!(f, "REBUILD_{}", endpoint),
        }
    }
}

/// 升级策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationPolicy {
    /// 通知人工处理
    AlertHuman,
    /// 记录并继续
    LogAndContinue,
    /// 中止操作
    Abort,
}

impl std::fmt::Display for EscalationPolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlertHuman => write!(f, "ALERT_HUMAN"),
            Self::LogAndContinue => write!(f, "LOG_AND_CONTINUE"),
            Self::Abort => write!(f, "ABORT"),
        }
    }
}

/// 恢复配方
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryRecipe {
    /// 故障场景
    pub scenario: FailureScenario,
    /// 恢复步骤列表
    pub steps: Vec<RecoveryStep>,
    /// 升级策略
    pub escalation_policy: EscalationPolicy,
    /// 最大重试次数
    pub max_retries: u32,
    /// 描述
    pub description: String,
}

/// 恢复配方存储
pub struct RecipeStore {
    recipes: Arc<RwLock<HashMap<FailureScenario, RecoveryRecipe>>>,
}

impl RecipeStore {
    pub fn new() -> Self {
        Self {
            recipes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加配方
    pub async fn add_recipe(&self, recipe: RecoveryRecipe) {
        let mut recipes = self.recipes.write().await;
        recipes.insert(recipe.scenario.clone(), recipe);
    }

    /// 获取配方
    pub async fn get_recipe(&self, scenario: &FailureScenario) -> Option<RecoveryRecipe> {
        let recipes = self.recipes.read().await;
        recipes.get(scenario).cloned()
    }

    /// 初始化默认配方
    pub async fn init_default_recipes(&self) {
        // NATS 断连配方
        self.add_recipe(RecoveryRecipe {
            scenario: FailureScenario::NatsDisconnect,
            steps: vec![
                RecoveryStep::Reconnect {
                    service: "nats".to_string(),
                },
                RecoveryStep::Resubscribe {
                    topics: vec![
                        "tasks.>".to_string(),
                        "agent.>".to_string(),
                        "watchdog.alert".to_string(),
                    ],
                },
                RecoveryStep::Wait { seconds: 2 },
                RecoveryStep::EscalateToHuman {
                    reason: "NATS reconnection failed".to_string(),
                },
            ],
            escalation_policy: EscalationPolicy::AlertHuman,
            max_retries: 3,
            description: "NATS connection lost, try to reconnect".to_string(),
        })
        .await;

        // Agent 超时配方
        self.add_recipe(RecoveryRecipe {
            scenario: FailureScenario::AgentTimeout,
            steps: vec![
                RecoveryStep::Wait { seconds: 5 },
                RecoveryStep::RestartProcess {
                    agent_id: "default".to_string(),
                },
                RecoveryStep::EscalateToHuman {
                    reason: "Agent timeout, restart failed".to_string(),
                },
            ],
            escalation_policy: EscalationPolicy::AlertHuman,
            max_retries: 2,
            description: "Agent process timeout, try to restart".to_string(),
        })
        .await;

        // 编译失败配方
        self.add_recipe(RecoveryRecipe {
            scenario: FailureScenario::CompileFailure,
            steps: vec![
                RecoveryStep::CleanBuild,
                RecoveryStep::RetryWithBackoff { max_retries: 3 },
                RecoveryStep::EscalateToHuman {
                    reason: "Build failed after clean".to_string(),
                },
            ],
            escalation_policy: EscalationPolicy::LogAndContinue,
            max_retries: 2,
            description: "Compilation failed, try clean build".to_string(),
        })
        .await;

        // API 限流配方
        self.add_recipe(RecoveryRecipe {
            scenario: FailureScenario::ApiRateLimit,
            steps: vec![
                RecoveryStep::RetryWithBackoff { max_retries: 5 },
                RecoveryStep::SwitchProvider {
                    fallback: "backup_provider".to_string(),
                },
                RecoveryStep::EscalateToHuman {
                    reason: "All providers rate limited".to_string(),
                },
            ],
            escalation_policy: EscalationPolicy::AlertHuman,
            max_retries: 3,
            description: "API rate limit hit, try backoff and switch provider".to_string(),
        })
        .await;

        // Dashboard 断连配方
        self.add_recipe(RecoveryRecipe {
            scenario: FailureScenario::DashboardDisconnect,
            steps: vec![
                RecoveryStep::Reconnect {
                    service: "websocket".to_string(),
                },
                RecoveryStep::Wait { seconds: 3 },
                RecoveryStep::RebuildConnection {
                    endpoint: "/ws".to_string(),
                },
            ],
            escalation_policy: EscalationPolicy::LogAndContinue,
            max_retries: 2,
            description: "Dashboard WebSocket disconnected, try to reconnect".to_string(),
        })
        .await;

        tracing::info!("Initialized default recovery recipes");
    }
}

/// 故障处理器
pub struct FailureHandler {
    recipe_store: Arc<RecipeStore>,
    retry_counts: Arc<RwLock<HashMap<FailureScenario, u32>>>,
}

impl FailureHandler {
    pub fn new(recipe_store: Arc<RecipeStore>) -> Self {
        Self {
            recipe_store,
            retry_counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 处理故障
    pub async fn handle_failure(&self, scenario: FailureScenario, context: &str) -> Result<String> {
        tracing::warn!("Handling failure: {} - {}", scenario, context);

        // 获取配方
        let recipe = match self.recipe_store.get_recipe(&scenario).await {
            Some(r) => r,
            None => {
                tracing::error!("No recipe found for scenario: {}", scenario);
                return Err(anyhow::anyhow!(
                    "No recipe found for scenario: {}",
                    scenario
                ));
            }
        };

        // 检查重试次数
        let mut retry_counts = self.retry_counts.write().await;
        let count = retry_counts.entry(scenario.clone()).or_insert(0);
        *count += 1;

        if *count > recipe.max_retries {
            tracing::error!(
                "Max retries ({}) reached for scenario: {}",
                recipe.max_retries,
                scenario
            );
            return Err(anyhow::anyhow!(
                "Max retries reached for scenario: {}",
                scenario
            ));
        }

        // 执行恢复步骤
        for (i, step) in recipe.steps.iter().enumerate() {
            tracing::info!("Executing step {}/{}: {}", i + 1, recipe.steps.len(), step);

            match self.execute_step(step, context).await {
                Ok(result) => {
                    tracing::info!("Step {} succeeded: {}", i + 1, result);
                    // 如果是最后一步，重置重试计数
                    if i == recipe.steps.len() - 1 {
                        retry_counts.remove(&scenario);
                    }
                }
                Err(e) => {
                    tracing::error!("Step {} failed: {}", i + 1, e);
                    // 如果是升级步骤，直接返回错误
                    if matches!(step, RecoveryStep::EscalateToHuman { .. }) {
                        retry_counts.remove(&scenario);
                        return Err(e);
                    }
                    // 否则继续执行下一步
                    continue;
                }
            }
        }

        Ok(format!("Recovery completed for scenario: {}", scenario))
    }

    /// 执行单个恢复步骤
    ///
    /// 说明：大部分步骤仍需依赖外部资源才能真正生效（NATS client、lifecycle manager、
    /// provider 池等）。当前实现把"尚未接线"的步骤**显式返回错误**而不是假装成功
    /// `sleep + Ok`。调用方会继续走下一条 step 或升级到 EscalateToHuman——这样上报
    /// 的状态和实际行为一致。
    async fn execute_step(&self, step: &RecoveryStep, _context: &str) -> Result<String> {
        match step {
            RecoveryStep::Wait { seconds } => {
                tracing::info!("Waiting for {} seconds", seconds);
                tokio::time::sleep(std::time::Duration::from_secs(*seconds)).await;
                Ok(format!("Waited for {} seconds", seconds))
            }
            RecoveryStep::EscalateToHuman { reason } => {
                tracing::error!("Escalating to human: {}", reason);
                Err(anyhow::anyhow!("Escalated to human: {}", reason))
            }
            RecoveryStep::Reconnect { service } => {
                tracing::warn!("RecoveryStep::Reconnect({}) not wired to NATS client yet", service);
                Err(anyhow::anyhow!(
                    "Reconnect({}) not implemented — requires NATS client handle from main.rs",
                    service
                ))
            }
            RecoveryStep::RestartProcess { agent_id } => {
                tracing::warn!(
                    "RecoveryStep::RestartProcess({}) not wired to AgentLifecycleManager yet",
                    agent_id
                );
                Err(anyhow::anyhow!(
                    "RestartProcess({}) not implemented — requires lifecycle_manager handle",
                    agent_id
                ))
            }
            RecoveryStep::CleanBuild => {
                tracing::warn!("RecoveryStep::CleanBuild not wired to build system yet");
                Err(anyhow::anyhow!(
                    "CleanBuild not implemented — requires workspace path + cargo invocation"
                ))
            }
            RecoveryStep::RetryWithBackoff { max_retries } => {
                tracing::warn!(
                    "RecoveryStep::RetryWithBackoff(max={}) has no target to retry",
                    max_retries
                );
                Err(anyhow::anyhow!(
                    "RetryWithBackoff not implemented — requires retry target closure"
                ))
            }
            RecoveryStep::SwitchProvider { fallback } => {
                tracing::warn!(
                    "RecoveryStep::SwitchProvider({}) not wired to provider pool yet",
                    fallback
                );
                Err(anyhow::anyhow!(
                    "SwitchProvider({}) not implemented — requires provider registry",
                    fallback
                ))
            }
            RecoveryStep::Resubscribe { topics } => {
                tracing::warn!(
                    "RecoveryStep::Resubscribe({:?}) not wired to NATS client yet",
                    topics
                );
                Err(anyhow::anyhow!(
                    "Resubscribe not implemented — requires NATS client handle"
                ))
            }
            RecoveryStep::RebuildConnection { endpoint } => {
                tracing::warn!(
                    "RecoveryStep::RebuildConnection({}) not wired to WebSocket manager yet",
                    endpoint
                );
                Err(anyhow::anyhow!(
                    "RebuildConnection({}) not implemented — requires WS state handle",
                    endpoint
                ))
            }
        }
    }

    /// 重置重试计数
    pub async fn reset_retry_count(&self, scenario: &FailureScenario) {
        let mut retry_counts = self.retry_counts.write().await;
        retry_counts.remove(scenario);
    }

    /// 获取重试计数
    pub async fn get_retry_count(&self, scenario: &FailureScenario) -> u32 {
        let retry_counts = self.retry_counts.read().await;
        *retry_counts.get(scenario).unwrap_or(&0)
    }
}
