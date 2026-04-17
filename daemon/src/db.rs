use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::state::{AgentInfo, AgentStatus, Task, TaskStatus};

/// SQLite 数据库 — 冷存储（历史记录、审计、回滚）
pub struct Database {
    conn: Arc<Mutex<Connection>>,
    db_path: String,
}

impl Database {
    /// 创建数据库实例（不初始化 Schema，需要调用 init_schema）
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: db_path.to_string(),
        })
    }

    /// 异步初始化数据库 Schema
    pub async fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT,
                status TEXT NOT NULL DEFAULT 'pending',
                assigned_to TEXT,
                depends_on TEXT DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                artifacts TEXT DEFAULT '[]'
            );

            CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                role TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'idle',
                current_task TEXT,
                last_heartbeat TEXT NOT NULL,
                restart_count INTEGER DEFAULT 0,
                level INTEGER NOT NULL DEFAULT 1,
                xp INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS task_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id TEXT NOT NULL,
                old_status TEXT,
                new_status TEXT NOT NULL,
                changed_at TEXT NOT NULL,
                details TEXT
            );

            CREATE TABLE IF NOT EXISTS xp_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                agent_id TEXT NOT NULL,
                task_id TEXT,
                delta INTEGER NOT NULL,
                reason TEXT NOT NULL,
                old_xp INTEGER NOT NULL,
                new_xp INTEGER NOT NULL,
                old_level INTEGER NOT NULL,
                new_level INTEGER NOT NULL,
                timestamp TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project_id);
            CREATE INDEX IF NOT EXISTS idx_agents_project ON agents(project_id);
            CREATE INDEX IF NOT EXISTS idx_history_task ON task_history(task_id);
            CREATE INDEX IF NOT EXISTS idx_xp_log_agent ON xp_log(agent_id);
        ",
        )?;

        // Idempotent migration for pre-existing DBs created before level/xp columns were added.
        // ALTER TABLE ADD COLUMN fails if the column already exists; we swallow that specific error.
        for stmt in [
            "ALTER TABLE agents ADD COLUMN level INTEGER NOT NULL DEFAULT 1",
            "ALTER TABLE agents ADD COLUMN xp INTEGER NOT NULL DEFAULT 0",
        ] {
            if let Err(e) = conn.execute(stmt, []) {
                let msg = e.to_string();
                if !msg.contains("duplicate column name") {
                    return Err(e.into());
                }
            }
        }

        tracing::info!("SQLite schema initialized: {}", self.db_path);
        Ok(())
    }

    /// 插入任务
    pub async fn insert_task(&self, project_id: &str, task: &Task) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO tasks (id, project_id, title, description, status, assigned_to, depends_on, created_at, updated_at, artifacts)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                task.id,
                project_id,
                task.title,
                task.description,
                task.status.as_str(),
                task.assigned_to,
                serde_json::to_string(&task.depends_on)?,
                task.created_at.to_rfc3339(),
                task.updated_at.to_rfc3339(),
                serde_json::to_string(&task.artifacts)?,
            ],
        )?;

        // 记录历史
        conn.execute(
            "INSERT INTO task_history (task_id, old_status, new_status, changed_at, details)
             VALUES (?1, NULL, ?2, ?3, 'task_created')",
            params![task.id, task.status.as_str(), Utc::now().to_rfc3339()],
        )?;

        Ok(())
    }

    /// 更新任务状态
    pub async fn update_task_status(
        &self,
        project_id: &str,
        task_id: &str,
        status: &str,
    ) -> Result<()> {
        let conn = self.conn.lock().await;

        // 获取旧状态
        let old_status: String = conn
            .query_row(
                "SELECT status FROM tasks WHERE id = ?1 AND project_id = ?2",
                params![task_id, project_id],
                |row| row.get(0),
            )
            .map_err(|e| anyhow::anyhow!("Task {}/{} not found: {}", project_id, task_id, e))?;

        // 更新状态
        let rows = conn.execute(
            "UPDATE tasks SET status = ?1, updated_at = ?2 WHERE id = ?3 AND project_id = ?4",
            params![status, Utc::now().to_rfc3339(), task_id, project_id],
        )?;
        if rows == 0 {
            return Err(anyhow::anyhow!(
                "UPDATE affected 0 rows for task {}/{}",
                project_id,
                task_id
            ));
        }

        // 记录历史
        conn.execute(
            "INSERT INTO task_history (task_id, old_status, new_status, changed_at, details)
             VALUES (?1, ?2, ?3, ?4, 'status_update')",
            params![task_id, old_status, status, Utc::now().to_rfc3339()],
        )?;

        Ok(())
    }

    /// 获取项目的所有任务
    pub async fn get_tasks(&self, project_id: &str) -> Result<Vec<Task>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, title, description, status, assigned_to, depends_on, created_at, updated_at, artifacts
             FROM tasks WHERE project_id = ?1"
        )?;

        let tasks = stmt.query_map(params![project_id], |row| {
            let id: String = row.get(0)?;
            let title: String = row.get(1)?;
            let description: String = row.get(2)?;
            let status_str: String = row.get(3)?;
            let assigned_to: Option<String> = row.get(4)?;
            let depends_on_json: String = row.get(5)?;
            let created_at_str: String = row.get(6)?;
            let updated_at_str: String = row.get(7)?;
            let artifacts_json: String = row.get(8)?;

            let depends_on: Vec<String> =
                serde_json::from_str(&depends_on_json).unwrap_or_default();
            let artifacts: Vec<String> = serde_json::from_str(&artifacts_json).unwrap_or_default();
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(Task {
                id,
                title,
                description,
                status: TaskStatus::from_str(&status_str),
                assigned_to,
                depends_on,
                created_at,
                updated_at,
                artifacts,
            })
        })?;

        let mut result = Vec::new();
        for task in tasks {
            result.push(task?);
        }
        Ok(result)
    }

    /// 插入 Agent
    pub async fn insert_agent(&self, project_id: &str, agent: &AgentInfo) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT OR REPLACE INTO agents (id, project_id, role, status, current_task, last_heartbeat, restart_count, level, xp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                agent.id,
                project_id,
                agent.role,
                agent.status.as_str(),
                agent.current_task,
                agent.last_heartbeat.to_rfc3339(),
                agent.restart_count,
                agent.level,
                agent.xp,
            ],
        )?;
        Ok(())
    }

    /// 获取项目的所有 Agent
    pub async fn get_agents(&self, project_id: &str) -> Result<Vec<AgentInfo>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, role, status, current_task, last_heartbeat, restart_count, level, xp
             FROM agents WHERE project_id = ?1",
        )?;

        let agents = stmt.query_map(params![project_id], |row| {
            let id: String = row.get(0)?;
            let role: String = row.get(1)?;
            let status_str: String = row.get(2)?;
            let current_task: Option<String> = row.get(3)?;
            let last_heartbeat_str: String = row.get(4)?;
            let restart_count: u32 = row.get(5)?;
            let level: u32 = row.get(6)?;
            let xp: u32 = row.get(7)?;

            let status = match status_str.as_str() {
                "working" => AgentStatus::Working,
                "done" => AgentStatus::Done,
                "error" => AgentStatus::Error,
                "restarting" => AgentStatus::Restarting,
                _ => AgentStatus::Idle,
            };

            let last_heartbeat = DateTime::parse_from_rfc3339(&last_heartbeat_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(AgentInfo {
                id,
                role,
                status,
                current_task,
                last_heartbeat,
                restart_count,
                level,
                xp,
            })
        })?;

        let mut result = Vec::new();
        for agent in agents {
            result.push(agent?);
        }
        Ok(result)
    }

    /// 记录一条 XP 变更
    pub async fn insert_xp_log(
        &self,
        agent_id: &str,
        task_id: Option<&str>,
        delta: i32,
        reason: &str,
        old_xp: u32,
        new_xp: u32,
        old_level: u32,
        new_level: u32,
    ) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "INSERT INTO xp_log (agent_id, task_id, delta, reason, old_xp, new_xp, old_level, new_level, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                agent_id,
                task_id,
                delta,
                reason,
                old_xp,
                new_xp,
                old_level,
                new_level,
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// 读取某 agent 的 XP 历史（最近 `limit` 条，默认 50）
    pub async fn get_xp_log(
        &self,
        agent_id: &str,
        limit: u32,
    ) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT task_id, delta, reason, old_xp, new_xp, old_level, new_level, timestamp
             FROM xp_log WHERE agent_id = ?1 ORDER BY timestamp DESC LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![agent_id, limit], |row| {
            let task_id: Option<String> = row.get(0)?;
            let delta: i32 = row.get(1)?;
            let reason: String = row.get(2)?;
            let old_xp: u32 = row.get(3)?;
            let new_xp: u32 = row.get(4)?;
            let old_level: u32 = row.get(5)?;
            let new_level: u32 = row.get(6)?;
            let ts: String = row.get(7)?;
            Ok(serde_json::json!({
                "task_id": task_id,
                "delta": delta,
                "reason": reason,
                "old_xp": old_xp,
                "new_xp": new_xp,
                "old_level": old_level,
                "new_level": new_level,
                "timestamp": ts,
            }))
        })?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// 获取任务历史
    pub async fn get_task_history(&self, task_id: &str) -> Result<Vec<(String, String, String)>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT old_status, new_status, changed_at FROM task_history WHERE task_id = ?1 ORDER BY changed_at DESC"
        )?;

        let history = stmt.query_map(params![task_id], |row| {
            let old: Option<String> = row.get(0)?;
            let new: String = row.get(1)?;
            let time: String = row.get(2)?;
            Ok((old.unwrap_or_default(), new, time))
        })?;

        let mut result = Vec::new();
        for h in history {
            result.push(h?);
        }
        Ok(result)
    }
}
