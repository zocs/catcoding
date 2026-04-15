use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::l3_skills::{L3Skills, Skill};

/// 渐进式 Skill 加载器
///
/// 按需加载 Skill，而非全量加载，减少 token 消耗
pub struct ProgressiveLoader {
    /// L1 索引（场景→Skill 指针）
    index: HashMap<String, Vec<String>>,
    /// 已加载的 Skill 缓存
    loaded_cache: HashMap<String, Skill>,
    /// 加载统计
    stats: LoaderStats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoaderStats {
    pub total_searches: u64,
    pub cache_hits: u64,
    pub skills_loaded: u64,
    pub tokens_saved: u64,
}

impl ProgressiveLoader {
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
            loaded_cache: HashMap::new(),
            stats: LoaderStats::default(),
        }
    }

    /// 从 L1 索引文件加载场景映射
    pub fn load_index(&mut self, index_path: &Path) -> Result<()> {
        if !index_path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(index_path)?;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // 格式: scene → skill1,skill2,skill3
            if let Some((scene, skills_str)) = line.split_once("→") {
                let scene = scene.trim().to_string();
                let skills: Vec<String> = skills_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                self.index.insert(scene, skills);
            }
        }

        tracing::info!("Progressive loader: loaded {} scene indexes", self.index.len());
        Ok(())
    }

    /// 根据任务类型和关键词搜索相关 Skill
    pub fn search_relevant_skills(
        &mut self,
        task_type: &str,
        keywords: &[String],
        skills_db: &L3Skills,
    ) -> Result<Vec<Skill>> {
        self.stats.total_searches += 1;

        let mut relevant_skill_names: Vec<String> = Vec::new();

        // 1. 精确匹配场景
        if let Some(skills) = self.index.get(task_type) {
            relevant_skill_names.extend(skills.clone());
        }

        // 2. 模糊匹配关键词
        for (scene, skills) in &self.index {
            for keyword in keywords {
                if scene.contains(keyword.as_str()) {
                    relevant_skill_names.extend(skills.clone());
                }
            }
        }

        // 3. 去重
        relevant_skill_names.sort();
        relevant_skill_names.dedup();

        // 4. 加载（优先从缓存）
        let mut result = Vec::new();
        for name in &relevant_skill_names {
            if let Some(cached) = self.loaded_cache.get(name) {
                result.push(cached.clone());
                self.stats.cache_hits += 1;
            } else {
                // 从 L3 加载
                if let Some(skill) = skills_db.get(name) {
                    self.loaded_cache.insert(name.clone(), skill.clone());
                    result.push(skill.clone());
                    self.stats.skills_loaded += 1;
                }
            }
        }

        tracing::debug!(
            "Search complete: scene={}, keywords={:?}, found={} skills",
            task_type,
            keywords,
            result.len()
        );

        Ok(result)
    }

    /// 更新 L1 索引（任务完成后调用）
    pub fn update_index(&mut self, task_type: &str, used_skills: &[String]) {
        let entry = self.index.entry(task_type.to_string()).or_default();
        for skill in used_skills {
            if !entry.contains(skill) {
                entry.push(skill.clone());
            }
        }
    }

    /// 保存索引到文件
    pub fn save_index(&self, index_path: &Path) -> Result<()> {
        let mut content =
            String::from("# CatCoding Skill Index\n# Format: scene → skill1,skill2,skill3\n\n");

        let mut scenes: Vec<_> = self.index.keys().collect();
        scenes.sort();

        for scene in scenes {
            if let Some(skills) = self.index.get(scene) {
                if !skills.is_empty() {
                    content.push_str(&format!("{} → {}\n", scene, skills.join(",")));
                }
            }
        }

        std::fs::write(index_path, content)?;
        tracing::info!("Index saved: {} scenes", self.index.len());
        Ok(())
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> &LoaderStats {
        &self.stats
    }

    /// 估算节省的 token 数
    pub fn estimate_token_savings(&self, avg_skill_tokens: u64) -> u64 {
        // 假设全量加载会加载所有 Skill
        let total_skills = self.index.values().map(|v| v.len()).sum::<usize>() as u64;
        let actually_loaded = self.stats.skills_loaded;

        if total_skills > actually_loaded {
            (total_skills - actually_loaded) * avg_skill_tokens
        } else {
            0
        }
    }
}
