use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 会话归档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArchive {
    pub session_id: String,
    pub task_id: String,
    pub task_summary: String,
    pub started_at: String,
    pub completed_at: String,
    pub tools_used: Vec<String>,
    pub outcome: String, // "success" | "failed"
}

/// L4 会话层 — 历史会话归档
///
/// 职责：可追溯的执行记录
/// 由 scheduler 反射自动收集
pub struct L4Sessions {
    sessions_dir: String,
}

impl L4Sessions {
    pub fn new(sessions_dir: &str) -> Result<Self> {
        std::fs::create_dir_all(sessions_dir)?;
        Ok(Self {
            sessions_dir: sessions_dir.to_string(),
        })
    }

    /// 归档会话
    pub fn archive(&self, session: &SessionArchive) -> Result<String> {
        let filename = format!(
            "{}_{}.json",
            session.completed_at.replace(':', "-").replace('T', "_"),
            session.task_id
        );
        let path = format!("{}/{}", self.sessions_dir, filename);

        let json = serde_json::to_string_pretty(session)?;
        fs::write(&path, json)?;

        tracing::info!("📁 会话归档: {}", filename);
        Ok(filename)
    }

    /// 搜索历史会话
    pub fn search(&self, keyword: &str) -> Result<Vec<SessionArchive>> {
        let mut results = Vec::new();

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<SessionArchive>(&content) {
                        if session.task_summary.contains(keyword)
                            || session.outcome.contains(keyword)
                        {
                            results.push(session);
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// 获取会话数量
    pub fn count(&self) -> usize {
        fs::read_dir(&self.sessions_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
                    .count()
            })
            .unwrap_or(0)
    }

    /// 压缩旧会话（保留摘要，删除细节）
    pub fn compress_old_sessions(&self, keep_days: u32) -> Result<usize> {
        let mut compressed = 0;
        let now = chrono::Utc::now();

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<SessionArchive>(&content) {
                        if let Ok(completed) = chrono::DateTime::parse_from_rfc3339(&session.completed_at) {
                            let age_days = (now - completed.with_timezone(&chrono::Utc)).num_days();
                            if age_days > keep_days as i64 {
                                // 压缩：只保留摘要
                                let compressed_session = SessionArchive {
                                    session_id: session.session_id,
                                    task_id: session.task_id,
                                    task_summary: session.task_summary,
                                    started_at: session.started_at,
                                    completed_at: session.completed_at,
                                    tools_used: Vec::new(), // 清空细节
                                    outcome: session.outcome,
                                };
                                let json = serde_json::to_string_pretty(&compressed_session)?;
                                fs::write(&path, json)?;
                                compressed += 1;
                            }
                        }
                    }
                }
            }
        }

        if compressed > 0 {
            tracing::info!("🗜️ 压缩了 {} 个旧会话", compressed);
        }

        Ok(compressed)
    }
}
