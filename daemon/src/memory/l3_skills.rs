use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Skill — 结晶的执行路径
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub trigger_scene: String,
    pub trigger_keywords: Vec<String>,
    pub prerequisites: Vec<String>,
    pub steps: Vec<String>,
    pub pitfalls: Vec<String>,
    pub source_task: String,
    pub created_at: String,
    pub success_count: u32,
}

impl Skill {
    /// 转换为上下文注入格式
    pub fn to_context(&self) -> String {
        let mut ctx = format!("# Skill: {}\n", self.name);

        if !self.prerequisites.is_empty() {
            ctx.push_str("\n## 前置条件\n");
            for p in &self.prerequisites {
                ctx.push_str(&format!("- {}\n", p));
            }
        }

        ctx.push_str("\n## 执行步骤\n");
        for (i, step) in self.steps.iter().enumerate() {
            ctx.push_str(&format!("{}. {}\n", i + 1, step));
        }

        if !self.pitfalls.is_empty() {
            ctx.push_str("\n## ⚠️ 已知坑点\n");
            for p in &self.pitfalls {
                ctx.push_str(&format!("- {}\n", p));
            }
        }

        ctx.push_str(&format!(
            "\n## 来源\n- 首次执行: {}\n- 成功次数: {}\n",
            self.created_at, self.success_count
        ));

        ctx
    }

    /// 转换为 Markdown 文件内容
    pub fn to_markdown(&self) -> String {
        let mut md = format!("# {}\n\n", self.name);
        md.push_str(&format!("> 来源任务: {}\n", self.source_task));
        md.push_str(&format!("> 创建时间: {}\n", self.created_at));
        md.push_str(&format!("> 成功次数: {}\n\n", self.success_count));

        md.push_str("## 触发条件\n");
        md.push_str(&format!("- 场景: {}\n", self.trigger_scene));
        for kw in &self.trigger_keywords {
            md.push_str(&format!("- 关键词: {}\n", kw));
        }

        if !self.prerequisites.is_empty() {
            md.push_str("\n## 前置条件\n");
            for p in &self.prerequisites {
                md.push_str(&format!("- {}\n", p));
            }
        }

        md.push_str("\n## 执行步骤\n");
        for (i, step) in self.steps.iter().enumerate() {
            md.push_str(&format!("{}. {}\n", i + 1, step));
        }

        if !self.pitfalls.is_empty() {
            md.push_str("\n## ⚠️ 已知坑点\n");
            for p in &self.pitfalls {
                md.push_str(&format!("- {}\n", p));
            }
        }

        md
    }
}

/// L3 技能库 — 结晶的 Skill + SOP
///
/// 职责：为特定任务保留可复用的执行路径
pub struct L3Skills {
    skills_dir: String,
    skills: HashMap<String, Skill>,
}

impl L3Skills {
    pub fn new(skills_dir: &str) -> Result<Self> {
        std::fs::create_dir_all(skills_dir)?;

        let mut manager = Self {
            skills_dir: skills_dir.to_string(),
            skills: HashMap::new(),
        };

        manager.load_all()?;
        Ok(manager)
    }

    /// 加载所有 Skill
    fn load_all(&mut self) -> Result<()> {
        for entry in fs::read_dir(&self.skills_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(skill) = serde_json::from_str::<Skill>(&content) {
                        self.skills.insert(skill.name.clone(), skill);
                    }
                }
            }
        }
        tracing::info!("📚 加载了 {} 个 Skill", self.skills.len());
        Ok(())
    }

    /// 获取 Skill
    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// 插入 Skill
    pub fn insert(&mut self, skill: &Skill) -> Result<()> {
        // 保存为 JSON
        let json_path = format!("{}/{}.json", self.skills_dir, skill.name);
        let json = serde_json::to_string_pretty(skill)?;
        fs::write(&json_path, json)?;

        // 保存为 Markdown（人类可读）
        let md_path = format!("{}/{}.md", self.skills_dir, skill.name);
        fs::write(&md_path, skill.to_markdown())?;

        self.skills.insert(skill.name.clone(), skill.clone());
        Ok(())
    }

    /// 搜索相关 Skill
    pub fn search(&self, keywords: &[String]) -> Vec<&Skill> {
        self.skills
            .values()
            .filter(|skill| {
                keywords.iter().any(|kw| {
                    skill.trigger_scene.contains(kw)
                        || skill.trigger_keywords.iter().any(|tk| tk.contains(kw))
                        || skill.steps.iter().any(|s| s.contains(kw))
                })
            })
            .collect()
    }

    /// Skill 数量
    pub fn count(&self) -> usize {
        self.skills.len()
    }

    /// 列出所有 Skill 名称
    pub fn list_names(&self) -> Vec<String> {
        self.skills.keys().cloned().collect()
    }
}
