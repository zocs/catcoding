use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Checkpoint — 任务检查点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub task_id: String,
    pub project_id: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub files: Vec<FileSnapshot>,
    pub metadata: HashMap<String, String>,
}

/// 文件快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSnapshot {
    pub path: String,
    pub content: String,
    pub checksum: String,
}

/// Rollback 管理器
///
/// 管理任务检查点，支持回滚到之前的状态
///
/// Rollback 后处理策略:
/// - A: 超时/网络 → RETRY_SAME（同 agent 重跑）| 上限 2 次
/// - B: 逻辑错误 → RETRY_DIFFERENT（换 agent + 注入失败上下文）| 上限 1 次
/// - C: 任务太大 → SPLIT_AND_RETRY（PM 拆分）
/// - D: 依赖缺失 → ESCALATE_PM
/// - E: 做不了 → ESCALATE_HUMAN
pub struct RollbackManager {
    checkpoints: Arc<RwLock<HashMap<String, Checkpoint>>>,
    checkpoint_dir: String,
}

/// 失败类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailureType {
    /// A: 超时/网络
    TimeoutOrNetwork,
    /// B: 逻辑错误
    LogicError,
    /// C: 任务太大
    TaskTooLarge,
    /// D: 依赖缺失
    MissingDependency,
    /// E: 做不了
    CannotComplete,
}

/// 恢复策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    /// 同 agent 重跑
    RetrySame { max_attempts: u32 },
    /// 换 agent + 注入失败上下文
    RetryDifferent { max_attempts: u32 },
    /// PM 拆分
    SplitAndRetry,
    /// 升级到 PM
    EscalatePM,
    /// 升级到用户
    EscalateHuman,
}

impl RollbackManager {
    pub fn new(checkpoint_dir: &str) -> Result<Self> {
        std::fs::create_dir_all(checkpoint_dir)?;

        Ok(Self {
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
            checkpoint_dir: checkpoint_dir.to_string(),
        })
    }

    /// 创建检查点
    pub async fn create_checkpoint(
        &self,
        task_id: &str,
        project_id: &str,
        description: &str,
        files: Vec<&str>,
    ) -> Result<String> {
        let checkpoint_id = uuid::Uuid::new_v4().to_string();

        // 快照文件内容
        let mut snapshots = Vec::new();
        for file_path in files {
            if let Ok(content) = std::fs::read_to_string(file_path) {
                let checksum = format!("{:x}", md5::compute(&content));
                snapshots.push(FileSnapshot {
                    path: file_path.to_string(),
                    content,
                    checksum,
                });
            }
        }

        let checkpoint = Checkpoint {
            id: checkpoint_id.clone(),
            task_id: task_id.to_string(),
            project_id: project_id.to_string(),
            description: description.to_string(),
            created_at: Utc::now(),
            files: snapshots,
            metadata: HashMap::new(),
        };

        // 保存到内存
        self.checkpoints
            .write()
            .await
            .insert(checkpoint_id.clone(), checkpoint.clone());

        // 持久化到磁盘
        let path = format!("{}/{}.json", self.checkpoint_dir, checkpoint_id);
        let json = serde_json::to_string_pretty(&checkpoint)?;
        std::fs::write(&path, json)?;

        tracing::info!("📸 创建检查点: {} (任务: {})", checkpoint_id, task_id);

        Ok(checkpoint_id)
    }

    /// 回滚到检查点
    pub async fn rollback(&self, checkpoint_id: &str) -> Result<Vec<String>> {
        let checkpoints = self.checkpoints.read().await;
        let checkpoint = checkpoints
            .get(checkpoint_id)
            .ok_or_else(|| anyhow::anyhow!("检查点不存在: {}", checkpoint_id))?;

        let mut restored_files = Vec::new();

        for snapshot in &checkpoint.files {
            // 恢复文件内容
            std::fs::write(&snapshot.path, &snapshot.content)?;
            restored_files.push(snapshot.path.clone());
            tracing::info!("🔄 恢复文件: {}", snapshot.path);
        }

        tracing::info!(
            "⏪ 回滚完成: {} (恢复 {} 个文件)",
            checkpoint_id,
            restored_files.len()
        );

        Ok(restored_files)
    }

    /// 获取检查点
    pub async fn get_checkpoint(&self, checkpoint_id: &str) -> Option<Checkpoint> {
        self.checkpoints.read().await.get(checkpoint_id).cloned()
    }

    /// 获取任务的所有检查点
    pub async fn get_task_checkpoints(&self, task_id: &str) -> Vec<Checkpoint> {
        self.checkpoints
            .read()
            .await
            .values()
            .filter(|cp| cp.task_id == task_id)
            .cloned()
            .collect()
    }

    /// 根据失败类型推荐恢复策略
    pub fn recommend_strategy(
        &self,
        failure_type: &FailureType,
        restart_count: u32,
    ) -> RecoveryStrategy {
        match failure_type {
            FailureType::TimeoutOrNetwork => {
                if restart_count < 2 {
                    RecoveryStrategy::RetrySame { max_attempts: 2 }
                } else {
                    RecoveryStrategy::RetryDifferent { max_attempts: 1 }
                }
            }
            FailureType::LogicError => {
                if restart_count < 1 {
                    RecoveryStrategy::RetryDifferent { max_attempts: 1 }
                } else {
                    RecoveryStrategy::EscalatePM
                }
            }
            FailureType::TaskTooLarge => RecoveryStrategy::SplitAndRetry,
            FailureType::MissingDependency => RecoveryStrategy::EscalatePM,
            FailureType::CannotComplete => RecoveryStrategy::EscalateHuman,
        }
    }

    /// 执行恢复策略
    pub async fn execute_recovery(
        &self,
        checkpoint_id: &str,
        strategy: &RecoveryStrategy,
    ) -> Result<String> {
        match strategy {
            RecoveryStrategy::RetrySame { max_attempts } => {
                // 回滚后重试
                self.rollback(checkpoint_id).await?;
                Ok(format!(
                    "已回滚，将在同 Agent 重试 (上限 {} 次)",
                    max_attempts
                ))
            }
            RecoveryStrategy::RetryDifferent { max_attempts } => {
                // 回滚后换 Agent
                self.rollback(checkpoint_id).await?;
                Ok(format!(
                    "已回滚，将换 Agent 重试 (上限 {} 次)",
                    max_attempts
                ))
            }
            RecoveryStrategy::SplitAndRetry => Ok("任务需拆分，通知 PM".to_string()),
            RecoveryStrategy::EscalatePM => Ok("升级到 PM Agent 处理".to_string()),
            RecoveryStrategy::EscalateHuman => Ok("升级到用户处理".to_string()),
        }
    }

    /// 清理旧检查点
    pub async fn cleanup_old_checkpoints(&self, keep_days: u32) -> Result<usize> {
        let now = Utc::now();
        let mut cleaned = 0;

        let checkpoints = self.checkpoints.read().await;
        for (id, checkpoint) in checkpoints.iter() {
            let age_days = (now - checkpoint.created_at).num_days();
            if age_days > keep_days as i64 {
                let path = format!("{}/{}.json", self.checkpoint_dir, id);
                if Path::new(&path).exists() {
                    std::fs::remove_file(&path)?;
                    cleaned += 1;
                }
            }
        }

        if cleaned > 0 {
            tracing::info!("🧹 清理了 {} 个旧检查点", cleaned);
        }

        Ok(cleaned)
    }
}
