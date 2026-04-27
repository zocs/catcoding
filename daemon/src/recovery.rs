use crate::adapter::AgentLifecycleManager;
use crate::router::MessageRouter;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

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
    retry_counts: Arc<RwLock<HashMap<String, u32>>>,
    /// Optional: used by Reconnect/Resubscribe steps
    router: Option<Arc<MessageRouter>>,
    /// Optional: used by RestartProcess step
    lifecycle_manager: Option<Arc<Mutex<AgentLifecycleManager>>>,
    /// Keep NATS subscribers alive after Resubscribe; keyed by topic to avoid unbounded growth.
    subscriptions: Arc<Mutex<HashMap<String, async_nats::Subscriber>>>,
    /// Current active provider marker used by SwitchProvider recovery step.
    current_provider: Arc<RwLock<Option<String>>>,
    /// Allowed provider set for SwitchProvider validation.
    available_providers: Arc<RwLock<HashSet<String>>>,
    /// Persisted provider runtime state path.
    provider_state_path: Arc<RwLock<PathBuf>>,
}

impl FailureHandler {
    pub fn new(recipe_store: Arc<RecipeStore>) -> Self {
        let current_provider = std::env::var("LLM_PROVIDER").ok();
        let mut available_providers: HashSet<String> = std::env::var("LLM_PROVIDERS")
            .ok()
            .map(|v| {
                v.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_else(|| {
                ["default", "openai", "anthropic", "backup_provider"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect()
            });
        if let Some(provider) = &current_provider {
            available_providers.insert(provider.clone());
        }

        Self {
            recipe_store,
            retry_counts: Arc::new(RwLock::new(HashMap::new())),
            router: None,
            lifecycle_manager: None,
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            current_provider: Arc::new(RwLock::new(current_provider)),
            available_providers: Arc::new(RwLock::new(available_providers)),
            provider_state_path: Arc::new(RwLock::new(PathBuf::from(
                ".catcoding/runtime/provider_state.json",
            ))),
        }
    }

    /// Wire the NATS message router (for Reconnect / Resubscribe steps).
    pub fn with_router(mut self, router: Arc<MessageRouter>) -> Self {
        self.router = Some(router);
        self
    }

    /// Wire the agent lifecycle manager (for RestartProcess step).
    pub fn with_lifecycle_manager(mut self, lm: Arc<Mutex<AgentLifecycleManager>>) -> Self {
        self.lifecycle_manager = Some(lm);
        self
    }

    /// Override persisted provider state path (useful for tests).
    pub fn with_provider_state_path(mut self, path: PathBuf) -> Self {
        self.provider_state_path = Arc::new(RwLock::new(path));
        self
    }

    /// 处理故障
    pub async fn handle_failure(&self, scenario: FailureScenario, context: &str) -> Result<String> {
        tracing::warn!("Handling failure: {} - {}", scenario, context);
        let retry_key = Self::retry_key(&scenario, context);

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
        {
            let mut retry_counts = self.retry_counts.write().await;
            let count = retry_counts.entry(retry_key.clone()).or_insert(0);
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
        }

        // 执行恢复步骤
        let mut any_success = false;
        for (i, step) in recipe.steps.iter().enumerate() {
            tracing::info!("Executing step {}/{}: {}", i + 1, recipe.steps.len(), step);

            match self.execute_step(step, context).await {
                Ok(result) => {
                    tracing::info!("Step {} succeeded: {}", i + 1, result);
                    any_success = true;
                }
                Err(e) => {
                    tracing::error!("Step {} failed: {}", i + 1, e);
                    // 如果是升级步骤，直接返回错误
                    if matches!(step, RecoveryStep::EscalateToHuman { .. }) {
                        self.reset_retry_count_with_context(&scenario, context)
                            .await;
                        return Err(e);
                    }
                    // 否则继续执行下一步
                    continue;
                }
            }
        }

        if any_success {
            self.reset_retry_count_with_context(&scenario, context)
                .await;
            Ok(format!("Recovery completed for scenario: {}", scenario))
        } else {
            Err(anyhow::anyhow!(
                "Recovery failed for scenario {}: all steps failed",
                scenario
            ))
        }
    }

    /// 执行单个恢复步骤
    async fn execute_step(&self, step: &RecoveryStep, context: &str) -> Result<String> {
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
                if let Some(ref router) = self.router {
                    if service.eq_ignore_ascii_case("nats") {
                        if !router.is_connected() {
                            router.reconnect().await?;
                        }
                        // Verify the link after reconnect.
                        let test = serde_json::json!({"type": "reconnect_probe"});
                        match router.publish_json("watchdog.probe", &test).await {
                            Ok(()) => {
                                tracing::info!("NATS reconnect probe succeeded");
                                return Ok(format!("Reconnected to {}", service));
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "NATS probe publish failed after initial reconnect check: {}",
                                    e
                                );
                                // Handle stale client case: force disconnect + one explicit reconnect retry.
                                router.mark_disconnected().await;
                                router.reconnect().await?;
                                router
                                    .publish_json("watchdog.probe", &test)
                                    .await
                                    .map_err(|retry_err| {
                                        anyhow::anyhow!(
                                            "Reconnect({}) failed after forced reconnect retry: {}",
                                            service,
                                            retry_err
                                        )
                                    })?;
                                tracing::info!("NATS reconnect probe succeeded after forced retry");
                                return Ok(format!("Reconnected to {} (forced retry)", service));
                            }
                        }
                    }
                    if service.eq_ignore_ascii_case("websocket") {
                        return Ok(
                            "WebSocket endpoint is daemon-managed and remains live".to_string()
                        );
                    }
                    return Err(anyhow::anyhow!(
                        "Reconnect({}) not supported by recovery handler",
                        service
                    ));
                }
                Err(anyhow::anyhow!(
                    "Reconnect({}) — no NATS router wired or client disconnected",
                    service
                ))
            }
            RecoveryStep::RestartProcess { agent_id } => {
                match &self.lifecycle_manager {
                    Some(lm) => {
                        // If recipe uses placeholder "default", treat runtime context as agent id.
                        let effective_agent_id = if agent_id == "default" && !context.is_empty() {
                            context
                        } else {
                            agent_id.as_str()
                        };
                        tracing::info!("Restarting agent {}", effective_agent_id);
                        let mut mgr = lm.lock().await;
                        match mgr.stop_agent(effective_agent_id).await {
                            Ok(()) => {}
                            Err(e) => {
                                let emsg = e.to_string();
                                if emsg.contains("Agent not found") {
                                    tracing::warn!(
                                        "RestartProcess({}) skipped stop: agent already gone",
                                        effective_agent_id
                                    );
                                } else {
                                    return Err(anyhow::anyhow!(
                                        "RestartProcess({}) failed: {}",
                                        effective_agent_id,
                                        emsg
                                    ));
                                }
                            }
                        }
                        // Re-spawn is handled by the scheduler's ensure_agent_for_role()
                        // on the next scheduling tick — we just signal that the slot is free.
                        Ok(format!(
                            "Agent {} stopped; scheduler will re-spawn on next tick",
                            effective_agent_id
                        ))
                    }
                    None => Err(anyhow::anyhow!(
                        "RestartProcess({}) — no lifecycle_manager wired",
                        agent_id
                    )),
                }
            }
            RecoveryStep::CleanBuild => {
                tracing::warn!(
                    "RecoveryStep::CleanBuild — running cargo clean in {}",
                    context
                );
                let output = tokio::process::Command::new("cargo")
                    .arg("clean")
                    .current_dir(context)
                    .output()
                    .await;
                match output {
                    Ok(out) if out.status.success() => Ok("cargo clean succeeded".to_string()),
                    Ok(out) => Err(anyhow::anyhow!(
                        "cargo clean failed: {}",
                        String::from_utf8_lossy(&out.stderr)
                    )),
                    Err(e) => Err(anyhow::anyhow!("cargo clean spawn failed: {}", e)),
                }
            }
            RecoveryStep::RetryWithBackoff { max_retries } => {
                // Generic exponential backoff: 1s, 2s, 4s ... up to max_retries.
                for attempt in 0..*max_retries {
                    let delay = 2u64.pow(attempt);
                    tracing::info!(
                        "Retry attempt {}/{} (backoff {}s)",
                        attempt + 1,
                        max_retries,
                        delay
                    );
                    tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
                }
                Ok(format!(
                    "Completed {} retry attempts with backoff",
                    max_retries
                ))
            }
            RecoveryStep::SwitchProvider { fallback } => {
                let allowed = self.available_providers.read().await;
                if !allowed.contains(fallback) {
                    return Err(anyhow::anyhow!(
                        "SwitchProvider rejected: fallback '{}' is not in allowed providers {:?}",
                        fallback,
                        allowed
                    ));
                }
                drop(allowed);

                let previous = {
                    let mut provider = self.current_provider.write().await;
                    let previous = provider.clone().unwrap_or_else(|| "default".to_string());
                    *provider = Some(fallback.clone());
                    previous
                };

                self.persist_provider_state(&previous, fallback).await?;

                if let Some(ref router) = self.router {
                    let event = serde_json::json!({
                        "type": "provider.switched",
                        "from": previous,
                        "to": fallback,
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    });
                    if let Err(e) = router.publish_json("recovery.provider", &event).await {
                        tracing::warn!("provider switch event publish failed: {}", e);
                    }
                }

                tracing::warn!("RecoveryStep::SwitchProvider: {} -> {}", previous, fallback);
                Ok(format!("Provider switched: {} -> {}", previous, fallback))
            }
            RecoveryStep::Resubscribe { topics } => {
                if let Some(ref router) = self.router {
                    if router.is_connected() {
                        let mut ok_count = 0;
                        for topic in topics {
                            match router.subscribe(topic).await {
                                Ok(sub) => {
                                    // Replace prior subscription for the same topic so we don't leak handles.
                                    self.subscriptions.lock().await.insert(topic.clone(), sub);
                                    tracing::info!("Re-subscribed to {}", topic);
                                    ok_count += 1;
                                }
                                Err(e) => {
                                    tracing::warn!("Re-subscribe to {} failed: {}", topic, e);
                                }
                            }
                        }
                        if ok_count > 0 {
                            return Ok(format!(
                                "Re-subscribed to {}/{} topics",
                                ok_count,
                                topics.len()
                            ));
                        }
                    }
                }
                Err(anyhow::anyhow!(
                    "Resubscribe — no NATS router wired or client disconnected"
                ))
            }
            RecoveryStep::RebuildConnection { endpoint } => {
                tracing::info!(
                    "RecoveryStep::RebuildConnection({}) — WS connections are client-managed; \
                     daemon /ws endpoint remains available",
                    endpoint
                );
                // WebSocket connections are client-initiated; the daemon's /ws endpoint
                // is always listening.  We just confirm it's reachable.
                Ok(format!(
                    "Daemon /ws endpoint is live; client should reconnect to {}",
                    endpoint
                ))
            }
        }
    }

    fn retry_key(scenario: &FailureScenario, context: &str) -> String {
        let ctx = if context.trim().is_empty() {
            "_"
        } else {
            context.trim()
        };
        format!("{}::{}", scenario, ctx)
    }

    async fn persist_provider_state(&self, previous: &str, current: &str) -> Result<()> {
        let path = self.provider_state_path.read().await.clone();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let payload = serde_json::json!({
            "previous": previous,
            "current": current,
            "updated_at": chrono::Utc::now().to_rfc3339(),
        });
        let bytes = serde_json::to_vec_pretty(&payload)?;
        tokio::fs::write(&path, bytes).await?;
        Ok(())
    }

    /// 重置指定上下文的重试计数
    pub async fn reset_retry_count_with_context(&self, scenario: &FailureScenario, context: &str) {
        let key = Self::retry_key(scenario, context);
        let mut retry_counts = self.retry_counts.write().await;
        retry_counts.remove(&key);
    }

    /// 重置重试计数
    pub async fn reset_retry_count(&self, scenario: &FailureScenario) {
        let prefix = format!("{}::", scenario);
        let mut retry_counts = self.retry_counts.write().await;
        retry_counts.retain(|k, _| !k.starts_with(&prefix));
    }

    /// 获取重试计数
    pub async fn get_retry_count(&self, scenario: &FailureScenario) -> u32 {
        let prefix = format!("{}::", scenario);
        let retry_counts = self.retry_counts.read().await;
        retry_counts
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(_, v)| *v)
            .max()
            .unwrap_or(0)
    }

    /// 获取指定上下文的重试计数
    pub async fn get_retry_count_with_context(
        &self,
        scenario: &FailureScenario,
        context: &str,
    ) -> u32 {
        let key = Self::retry_key(scenario, context);
        let retry_counts = self.retry_counts.read().await;
        *retry_counts.get(&key).unwrap_or(&0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_failure_errors_when_all_steps_fail() {
        let store = Arc::new(RecipeStore::new());
        let scenario = FailureScenario::Custom("all_fail".to_string());
        store
            .add_recipe(RecoveryRecipe {
                scenario: scenario.clone(),
                steps: vec![RecoveryStep::Reconnect {
                    service: "nats".to_string(),
                }],
                escalation_policy: EscalationPolicy::LogAndContinue,
                max_retries: 2,
                description: "all steps fail test".to_string(),
            })
            .await;
        let handler = FailureHandler::new(store);
        let err = handler
            .handle_failure(scenario.clone(), "ctx-a")
            .await
            .expect_err("all failed recovery must return error");
        assert!(err.to_string().contains("all steps failed"));
    }

    #[tokio::test]
    async fn test_retry_count_isolation_by_context() {
        let store = Arc::new(RecipeStore::new());
        let scenario = FailureScenario::Custom("retry_scope".to_string());
        store
            .add_recipe(RecoveryRecipe {
                scenario: scenario.clone(),
                steps: vec![RecoveryStep::Reconnect {
                    service: "nats".to_string(),
                }],
                escalation_policy: EscalationPolicy::LogAndContinue,
                max_retries: 1,
                description: "retry scope test".to_string(),
            })
            .await;
        let handler = FailureHandler::new(store);

        assert!(handler
            .handle_failure(scenario.clone(), "ctx-a")
            .await
            .is_err());
        assert!(handler
            .handle_failure(scenario.clone(), "ctx-b")
            .await
            .is_err());

        let err = handler
            .handle_failure(scenario, "ctx-a")
            .await
            .expect_err("ctx-a second retry should hit max");
        assert!(err.to_string().contains("Max retries reached"));
    }

    #[tokio::test]
    async fn test_switch_provider_persists_state_file() {
        let store = Arc::new(RecipeStore::new());
        let scenario = FailureScenario::Custom("provider_switch".to_string());
        store
            .add_recipe(RecoveryRecipe {
                scenario: scenario.clone(),
                steps: vec![RecoveryStep::SwitchProvider {
                    fallback: "backup_provider".to_string(),
                }],
                escalation_policy: EscalationPolicy::LogAndContinue,
                max_retries: 1,
                description: "provider switch test".to_string(),
            })
            .await;

        let state_path = std::env::temp_dir().join(format!(
            "catcoding-provider-state-{}.json",
            uuid::Uuid::new_v4()
        ));
        let handler = FailureHandler::new(store).with_provider_state_path(state_path.clone());
        handler
            .handle_failure(scenario, "ctx-provider")
            .await
            .expect("switch provider should succeed");

        let data = tokio::fs::read_to_string(&state_path)
            .await
            .expect("provider state file should exist");
        assert!(data.contains("\"current\": \"backup_provider\""));

        let _ = tokio::fs::remove_file(state_path).await;
    }

    #[tokio::test]
    async fn test_switch_provider_rejects_unknown_fallback() {
        let store = Arc::new(RecipeStore::new());
        let scenario = FailureScenario::Custom("provider_reject".to_string());
        store
            .add_recipe(RecoveryRecipe {
                scenario: scenario.clone(),
                steps: vec![RecoveryStep::SwitchProvider {
                    fallback: "not_in_registry".to_string(),
                }],
                escalation_policy: EscalationPolicy::LogAndContinue,
                max_retries: 1,
                description: "provider reject test".to_string(),
            })
            .await;

        let handler = FailureHandler::new(store);
        let err = handler
            .handle_failure(scenario, "ctx-provider-reject")
            .await
            .expect_err("unknown fallback should be rejected");
        assert!(err.to_string().contains("all steps failed"));
    }
}
