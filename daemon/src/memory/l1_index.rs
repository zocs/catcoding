use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// L1 索引层 — 极简场景→定位指针
///
/// 体积限制：≤30行，<1k tokens
/// 内容：场景关键词→记忆定位映射
pub struct L1Index {
    file_path: String,
    /// 高频场景 key→value（直接给出定位）
    mappings: HashMap<String, String>,
    /// 低频场景关键词列表
    keywords: Vec<String>,
    /// 避坑规则
    rules: Vec<String>,
}

impl L1Index {
    pub fn new(file_path: &str) -> Result<Self> {
        let mut index = Self {
            file_path: file_path.to_string(),
            mappings: HashMap::new(),
            keywords: Vec::new(),
            rules: Vec::new(),
        };

        // 如果文件存在，加载内容
        if Path::new(file_path).exists() {
            index.load()?;
        } else {
            // 创建初始文件
            index.save()?;
        }

        Ok(index)
    }

    /// 加载 L1 索引
    fn load(&mut self) -> Result<()> {
        let content = fs::read_to_string(&self.file_path)?;
        let mut section = "";

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                if line.contains("MAPPINGS") {
                    section = "mappings";
                } else if line.contains("KEYWORDS") {
                    section = "keywords";
                } else if line.contains("RULES") {
                    section = "rules";
                }
                continue;
            }

            match section {
                "mappings" => {
                    if let Some((key, value)) = line.split_once("→") {
                        self.mappings.insert(key.trim().to_string(), value.trim().to_string());
                    }
                }
                "keywords" => {
                    self.keywords.push(line.to_string());
                }
                "rules" => {
                    self.rules.push(line.to_string());
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// 保存 L1 索引
    fn save(&self) -> Result<()> {
        let mut content = String::new();
        content.push_str("# L1 Index — 场景→定位指针 (≤30行)\n");
        content.push_str("# 核心原则: 上层只留指针，不写细节\n\n");

        content.push_str("## MAPPINGS (高频场景)\n");
        let mut mappings: Vec<_> = self.mappings.iter().collect();
        mappings.sort_by_key(|(k, _)| k.as_str());
        for (key, value) in mappings {
            content.push_str(&format!("{} → {}\n", key, value));
        }

        content.push_str("\n## KEYWORDS (低频场景)\n");
        for keyword in &self.keywords {
            content.push_str(&format!("{}\n", keyword));
        }

        content.push_str("\n## RULES (避坑准则)\n");
        for rule in &self.rules {
            content.push_str(&format!("- {}\n", rule));
        }

        fs::write(&self.file_path, content)?;
        Ok(())
    }

    /// 查找场景对应的定位
    pub fn lookup(&self, scene: &str) -> Option<String> {
        self.mappings.get(scene).cloned()
    }

    /// 添加映射（高频场景）
    pub fn add_mapping(&mut self, scene: &str, pointer: &str) {
        // 检查是否超过30行限制
        if self.total_lines() >= 30 {
            tracing::warn!("⚠️ L1 索引已达30行限制，跳过添加: {}", scene);
            return;
        }

        self.mappings.insert(scene.to_string(), pointer.to_string());
        let _ = self.save();
    }

    /// 添加关键词（低频场景）
    pub fn add_keyword(&mut self, keyword: &str) {
        if !self.keywords.contains(&keyword.to_string()) {
            self.keywords.push(keyword.to_string());
            let _ = self.save();
        }
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: &str) {
        if !self.rules.contains(&rule.to_string()) {
            self.rules.push(rule.to_string());
            let _ = self.save();
        }
    }

    /// 总行数
    pub fn line_count(&self) -> usize {
        self.mappings.len() + self.keywords.len() + self.rules.len()
    }

    fn total_lines(&self) -> usize {
        self.line_count() + 10 // 预留头部和分隔符
    }
}
