/// 记忆系统 — L4 分层记忆
///
/// 借鉴 GenericAgent 的四层架构：
/// - L1: 极简索引层（≤30行，场景→定位指针）
/// - L2: 事实库层（环境事实：路径、配置）
/// - L3: 记录库层（SOP、脚本、结晶的 Skill）
/// - L4: 历史会话层（归档）

pub mod l1_index;
pub mod l2_facts;
pub mod l3_skills;
pub mod l4_sessions;
pub mod crystallizer;

pub use l1_index::L1Index;
pub use l2_facts::L2Facts;
pub use l3_skills::{L3Skills, Skill};
pub use l4_sessions::L4Sessions;
pub use crystallizer::SkillCrystallizer;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub layer: MemoryLayer,
    pub created_at: String,
    pub verified: bool,
    pub source_task: Option<String>,
}

/// 记忆层级
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MemoryLayer {
    L1, // 索引
    L2, // 事实
    L3, // 技能/SOP
    L4, // 会话归档
}

/// 记忆管理器
///
/// 核心原则：
/// 1. No Execution, No Memory — 只有验证过的才记入
/// 2. Sanctity of Verified Data — 验证过的数据神圣不可删改
/// 3. No Volatile State — 禁止存储易变状态
/// 4. Minimum Sufficient Pointer — 上层只留指针
pub struct MemoryManager {
    pub l1: L1Index,
    pub l2: L2Facts,
    pub l3: L3Skills,
    pub l4: L4Sessions,
    pub crystallizer: SkillCrystallizer,
}

impl MemoryManager {
    /// 初始化记忆系统
    pub fn new(memory_dir: &str) -> Result<Self> {
        std::fs::create_dir_all(memory_dir)?;
        std::fs::create_dir_all(format!("{}/L3", memory_dir))?;
        std::fs::create_dir_all(format!("{}/L4_sessions", memory_dir))?;

        Ok(Self {
            l1: L1Index::new(&format!("{}/L1_index.txt", memory_dir))?,
            l2: L2Facts::new(&format!("{}/L2_facts.txt", memory_dir))?,
            l3: L3Skills::new(&format!("{}/L3", memory_dir))?,
            l4: L4Sessions::new(&format!("{}/L4_sessions", memory_dir))?,
            crystallizer: SkillCrystallizer::new(),
        })
    }

    /// 查询记忆（从 L1 开始，逐层查找）
    pub fn query(&self, scene: &str) -> Option<String> {
        // 先查 L1 索引
        if let Some(pointer) = self.l1.lookup(scene) {
            // 指向 L2 或 L3
            if pointer.starts_with("L3:") {
                let skill_name = &pointer[3..];
                if let Some(skill) = self.l3.get(skill_name) {
                    return Some(skill.to_context());
                }
            } else {
                // 查 L2
                if let Some(fact) = self.l2.get(&pointer) {
                    return Some(fact);
                }
            }
        }
        None
    }

    /// 写入记忆（L2/L3 → 更新 L1 索引）
    pub fn write_fact(&mut self, key: &str, value: &str, source_task: &str) -> Result<()> {
        // 写入 L2
        self.l2.insert(key, value)?;

        // 更新 L1 索引（低频→第二层关键词，高频→第一层 key→value）
        self.l1.add_keyword(key);

        tracing::info!("💾 写入记忆 L2: {} (来源: {})", key, source_task);
        Ok(())
    }

    /// 结晶 Skill
    pub fn crystallize_skill(
        &mut self,
        task_id: &str,
        task_summary: &str,
        execution_steps: Vec<String>,
    ) -> Result<String> {
        let skill = self.crystallizer.crystallize(task_id, task_summary, execution_steps)?;
        let skill_name = skill.name.clone();

        // 写入 L3
        self.l3.insert(&skill)?;

        // 更新 L1 索引
        self.l1.add_mapping(&skill.trigger_scene, &format!("L3:{}", skill_name));

        tracing::info!("✨ Skill 结晶完成: {} (来源任务: {})", skill_name, task_id);
        Ok(skill_name)
    }

    /// 获取状态摘要
    pub fn status_summary(&self) -> serde_json::Value {
        serde_json::json!({
            "L1_index_lines": self.l1.line_count(),
            "L2_facts_count": self.l2.count(),
            "L3_skills_count": self.l3.count(),
            "L4_sessions_count": self.l4.count(),
            "core_axioms": [
                "No Execution, No Memory",
                "Sanctity of Verified Data",
                "No Volatile State",
                "Minimum Sufficient Pointer"
            ]
        })
    }
}
