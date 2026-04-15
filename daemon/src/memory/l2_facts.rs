use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// L2 事实库 — 环境事实（路径、配置、凭证）
///
/// 特征：
/// - 按 ## [SECTION] 组织
/// - 随环境扩展而膨胀（可接受）
/// - 变化时更新 L1 的相应映射
pub struct L2Facts {
    file_path: String,
    sections: HashMap<String, Vec<(String, String)>>,
}

impl L2Facts {
    pub fn new(file_path: &str) -> Result<Self> {
        let mut facts = Self {
            file_path: file_path.to_string(),
            sections: HashMap::new(),
        };

        if Path::new(file_path).exists() {
            facts.load()?;
        } else {
            facts.save()?;
        }

        Ok(facts)
    }

    /// 加载 L2 事实
    fn load(&mut self) -> Result<()> {
        let content = fs::read_to_string(&self.file_path)?;
        let mut current_section = String::from("GENERAL");

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("## [") && line.ends_with(']') {
                current_section = line[4..line.len() - 1].to_string();
                continue;
            }

            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once(": ") {
                self.sections
                    .entry(current_section.clone())
                    .or_default()
                    .push((key.trim().to_string(), value.trim().to_string()));
            }
        }

        Ok(())
    }

    /// 保存 L2 事实
    fn save(&self) -> Result<()> {
        let mut content = String::new();
        content.push_str("# L2 Facts — Environment\n");
        content.push_str("# Core: only verified environment-specific facts\n\n");

        let mut sections: Vec<_> = self.sections.keys().collect();
        sections.sort();

        for section in sections {
            content.push_str(&format!("## [{}]\n", section));
            if let Some(entries) = self.sections.get(section) {
                for (key, value) in entries {
                    content.push_str(&format!("{}: {}\n", key, value));
                }
            }
            content.push_str("\n");
        }

        fs::write(&self.file_path, content)?;
        Ok(())
    }

    /// 获取事实
    pub fn get(&self, key: &str) -> Option<String> {
        for entries in self.sections.values() {
            for (k, v) in entries {
                if k == key {
                    return Some(v.clone());
                }
            }
        }
        None
    }

    /// 插入事实
    pub fn insert(&mut self, key: &str, value: &str) -> Result<()> {
        let section = "GENERAL";
        self.sections
            .entry(section.to_string())
            .or_default()
            .push((key.to_string(), value.to_string()));
        self.save()
    }

    /// 按分区插入
    pub fn insert_to_section(&mut self, section: &str, key: &str, value: &str) -> Result<()> {
        self.sections
            .entry(section.to_string())
            .or_default()
            .push((key.to_string(), value.to_string()));
        self.save()
    }

    /// 事实数量
    pub fn count(&self) -> usize {
        self.sections.values().map(|v| v.len()).sum()
    }
}
