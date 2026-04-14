use anyhow::Result;
use chrono::Utc;

use super::l3_skills::Skill;

/// Skill 结晶器
///
/// 将成功的执行路径转化为可复用的 Skill
/// 核心原则: "No Execution, No Memory" — 只有验证过的才结晶
pub struct SkillCrystallizer {
    /// 最小步骤数（少于此数不结晶）
    min_steps: usize,
}

impl SkillCrystallizer {
    pub fn new() -> Self {
        Self { min_steps: 3 }
    }

    /// 结晶执行路径为 Skill
    pub fn crystallize(
        &self,
        task_id: &str,
        task_summary: &str,
        execution_steps: Vec<String>,
    ) -> Result<Skill> {
        if execution_steps.len() < self.min_steps {
            return Err(anyhow::anyhow!(
                "执行步骤太少 ({} < {})，不值得结晶",
                execution_steps.len(),
                self.min_steps
            ));
        }

        // 从任务摘要提取触发场景
        let trigger_scene = self.extract_trigger_scene(task_summary);

        // 从执行步骤提取关键词
        let trigger_keywords = self.extract_keywords(&execution_steps);

        // 分析步骤，识别前置条件和坑点
        let prerequisites = self.extract_prerequisites(&execution_steps);
        let pitfalls = self.extract_pitfalls(&execution_steps);

        // 生成 Skill 名称
        let name = self.generate_skill_name(task_summary);

        let skill = Skill {
            name: name.clone(),
            trigger_scene,
            trigger_keywords,
            prerequisites,
            steps: execution_steps,
            pitfalls,
            source_task: task_id.to_string(),
            created_at: Utc::now().to_rfc3339(),
            success_count: 1,
        };

        Ok(skill)
    }

    /// 从任务摘要提取触发场景
    fn extract_trigger_scene(&self, summary: &str) -> String {
        // 简化版：取前50个字符作为场景描述
        let scene = if summary.len() > 50 {
            &summary[..50]
        } else {
            summary
        };
        scene.to_string()
    }

    /// 从执行步骤提取关键词
    fn extract_keywords(&self, steps: &[String]) -> Vec<String> {
        let mut keywords = Vec::new();

        // 提取工具名
        for step in steps {
            if step.contains("shell(") || step.contains("terminal(") {
                keywords.push("shell".to_string());
            }
            if step.contains("file_read") || step.contains("file_write") {
                keywords.push("file".to_string());
            }
            if step.contains("browser_") {
                keywords.push("browser".to_string());
            }
            if step.contains("web_search") {
                keywords.push("search".to_string());
            }
        }

        // 提取常见任务类型
        let all_steps = steps.join(" ").to_lowercase();
        if all_steps.contains("install") || all_steps.contains("apt") || all_steps.contains("pip") {
            keywords.push("install".to_string());
        }
        if all_steps.contains("compile")
            || all_steps.contains("cargo")
            || all_steps.contains("make")
        {
            keywords.push("compile".to_string());
        }
        if all_steps.contains("test") || all_steps.contains("pytest") {
            keywords.push("test".to_string());
        }
        if all_steps.contains("deploy") || all_steps.contains("systemctl") {
            keywords.push("deploy".to_string());
        }

        keywords.sort();
        keywords.dedup();
        keywords
    }

    /// 提取前置条件
    fn extract_prerequisites(&self, steps: &[String]) -> Vec<String> {
        let mut prereqs = Vec::new();

        for step in steps {
            let lower = step.to_lowercase();
            if lower.contains("require") || lower.contains("need") || lower.contains("前提") {
                prereqs.push(step.clone());
            }
        }

        prereqs
    }

    /// 提取坑点
    fn extract_pitfalls(&self, steps: &[String]) -> Vec<String> {
        let mut pitfalls = Vec::new();

        for step in steps {
            let lower = step.to_lowercase();
            if lower.contains("⚠️")
                || lower.contains("注意")
                || lower.contains("坑")
                || lower.contains("error")
                || lower.contains("fail")
            {
                pitfalls.push(step.clone());
            }
        }

        pitfalls
    }

    /// 生成 Skill 名称
    fn generate_skill_name(&self, summary: &str) -> String {
        // 简化：取前几个词，用下划线连接
        let words: Vec<&str> = summary
            .split_whitespace()
            .take(4)
            .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|w| !w.is_empty())
            .collect();

        if words.is_empty() {
            format!("skill_{}", Utc::now().timestamp())
        } else {
            words.join("_").to_lowercase()
        }
    }

    /// 分析工具调用序列，判断是否值得结晶
    pub fn should_crystallize(&self, steps: &[String], success: bool) -> bool {
        if !success {
            return false;
        }

        if steps.len() < self.min_steps {
            return false;
        }

        // 检查是否有实质性操作（不只是读取）
        let has_write = steps.iter().any(|s| {
            s.contains("file_write")
                || s.contains("write_file")
                || s.contains("patch")
                || s.contains("shell(")
        });

        has_write
    }
}
