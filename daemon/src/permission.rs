use serde::{Deserialize, Serialize};

/// 权限等级 — 按风险分级，Bash 命令自动分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// 只读操作: cat, ls, grep, find, git status, etc.
    ReadOnly,
    /// 安全写入: git add, git commit, mkdir, touch, echo, etc.
    SafeWrite,
    /// 破坏性操作: rm, sudo, curl | bash, git reset --hard, etc.
    Destructive,
}

impl std::fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionLevel::ReadOnly => write!(f, "ReadOnly"),
            PermissionLevel::SafeWrite => write!(f, "SafeWrite"),
            PermissionLevel::Destructive => write!(f, "Destructive"),
        }
    }
}

/// 权限配置 — agent.yaml 中的 permissions 段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionConfig {
    /// 全局默认权限等级
    pub default_level: PermissionLevel,
    /// 是否允许破坏性操作
    pub allow_destructive: bool,
    /// Destructive 操作是否需要用户确认
    pub require_confirmation: bool,
    /// 是否允许 sudo
    pub sudo_allowed: bool,
}

impl Default for PermissionConfig {
    fn default() -> Self {
        Self {
            default_level: PermissionLevel::SafeWrite,
            allow_destructive: false,
            require_confirmation: true,
            sudo_allowed: false,
        }
    }
}

/// Bash 命令分类器 — 根据命令内容自动判断权限等级
///
/// # Examples
///
/// ```
/// use catcoding::permission::{classify_bash_command, PermissionLevel};
///
/// assert_eq!(classify_bash_command("ls -la"), PermissionLevel::ReadOnly);
/// assert_eq!(classify_bash_command("git add ."), PermissionLevel::SafeWrite);
/// assert_eq!(classify_bash_command("rm -rf /"), PermissionLevel::Destructive);
/// ```
pub fn classify_bash_command(cmd: &str) -> PermissionLevel {
    let cmd_lower = cmd.to_lowercase();

    // 破坏性操作模式 — 匹配命令开头或管道后第一个词
    let dangerous_prefixes = [
        "rm ",
        "rm -",
        "mkfs",
        "dd if=",
        "format ",
        "del ",
    ];
    let dangerous_exact = [
        "sudo",
        "format",
        "mkfs",
        "del",
    ];
    let dangerous_substrings = [
        "curl | bash",
        "curl | sh",
        "wget | bash",
        "wget | sh",
        "git reset --hard",
        "git clean -fd",
        "chmod 777",
        "chmod -r 777",
        "> /dev/",
        "drop table",
        "drop database",
        ":(){ :|:& };:", // fork bomb
    ];

    // 安全写入模式
    let safe_write_prefixes = [
        "git add",
        "git commit",
        "git push",
        "git merge",
        "git checkout -",
        "git branch",
        "mkdir ",
        "touch ",
        "echo ",
        "tee ",
        "cp ",
        "mv ",
        "rename ",
        "sed -i",
        "create ",
        "insert into",
        "update ",
        "cargo build",
        "cargo test",
        "npm install",
        "npm run",
        "pip install",
        "python -m",
    ];
    let safe_write_exact = [
        "write",
        "create",
    ];

    // 只读模式
    let readonly_prefixes = [
        "cat ",
        "ls",
        "grep",
        "find ",
        "head ",
        "tail ",
        "wc ",
        "sort ",
        "diff ",
        "git status",
        "git log",
        "git diff",
        "git show",
        "curl ",
        "wget ",
        "ping ",
        "ps ",
        "top ",
        "df ",
        "du ",
        "which ",
        "whereis ",
        "file ",
        "stat ",
        "tree ",
        "env ",
        "printenv",
        "uname",
        "hostname",
        "whoami",
        "id ",
    ];

    // 辅助：检查命令是否以模式开头
    let starts_with_pattern = |cmd: &str, patterns: &[&str]| -> bool {
        // 检查整条命令（处理管道：也检查 | 后的每个子命令）
        let parts: Vec<&str> = cmd.split('|').collect();
        for part in &parts {
            let trimmed = part.trim();
            for p in patterns {
                if trimmed.starts_with(p) {
                    return true;
                }
            }
        }
        false
    };

    // 辅助：检查命令中的某个子命令是否精确匹配
    let exact_match = |cmd: &str, patterns: &[&str]| -> bool {
        let parts: Vec<&str> = cmd.split(|c: char| c.is_whitespace() || c == '|').collect();
        for part in &parts {
            for p in patterns {
                if part == *p {
                    return true;
                }
            }
        }
        false
    };

    // 优先级: Destructive > SafeWrite > ReadOnly
    // curl/wget piping to shell — check for pipe to bash/sh anywhere
    if (cmd_lower.contains("curl") || cmd_lower.contains("wget"))
        && (cmd_lower.contains("| bash") || cmd_lower.contains("| sh"))
    {
        return PermissionLevel::Destructive;
    }

    // 检查破坏性子字符串模式（管道炸弹、重定向等）
    if dangerous_substrings.iter().any(|p| cmd_lower.contains(p)) {
        return PermissionLevel::Destructive;
    }

    if starts_with_pattern(&cmd_lower, &dangerous_prefixes) || exact_match(&cmd_lower, &dangerous_exact) {
        PermissionLevel::Destructive
    } else if starts_with_pattern(&cmd_lower, &safe_write_prefixes) || exact_match(&cmd_lower, &safe_write_exact) {
        PermissionLevel::SafeWrite
    } else if starts_with_pattern(&cmd_lower, &readonly_prefixes) {
        PermissionLevel::ReadOnly
    } else {
        // 未知命令默认 SafeWrite（保守策略）
        PermissionLevel::SafeWrite
    }
}

/// 检查命令是否被允许执行
///
/// 返回 (allowed, level, reason)
pub fn check_permission(cmd: &str, config: &PermissionConfig) -> (bool, PermissionLevel, String) {
    let level = classify_bash_command(cmd);

    match level {
        PermissionLevel::ReadOnly => (true, level, "Read-only operation".to_string()),
        PermissionLevel::SafeWrite => (true, level, "Safe write operation".to_string()),
        PermissionLevel::Destructive => {
            if !config.allow_destructive {
                (
                    false,
                    level,
                    "Destructive operations disabled by policy".to_string(),
                )
            } else if config.require_confirmation {
                (
                    false,
                    level,
                    "Destructive operation requires user confirmation".to_string(),
                )
            } else {
                (true, level, "Destructive operation allowed".to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_readonly() {
        assert_eq!(classify_bash_command("ls -la"), PermissionLevel::ReadOnly);
        assert_eq!(
            classify_bash_command("cat file.txt"),
            PermissionLevel::ReadOnly
        );
        assert_eq!(
            classify_bash_command("git status"),
            PermissionLevel::ReadOnly
        );
        assert_eq!(
            classify_bash_command("grep -r foo ."),
            PermissionLevel::ReadOnly
        );
        assert_eq!(
            classify_bash_command("find . -name '*.rs'"),
            PermissionLevel::ReadOnly
        );
    }

    #[test]
    fn test_classify_safe_write() {
        assert_eq!(
            classify_bash_command("git add ."),
            PermissionLevel::SafeWrite
        );
        assert_eq!(
            classify_bash_command("git commit -m 'test'"),
            PermissionLevel::SafeWrite
        );
        assert_eq!(
            classify_bash_command("mkdir new_dir"),
            PermissionLevel::SafeWrite
        );
        assert_eq!(
            classify_bash_command("touch file.txt"),
            PermissionLevel::SafeWrite
        );
        assert_eq!(
            classify_bash_command("cargo build --release"),
            PermissionLevel::SafeWrite
        );
        assert_eq!(
            classify_bash_command("npm install"),
            PermissionLevel::SafeWrite
        );
    }

    #[test]
    fn test_classify_destructive() {
        assert_eq!(
            classify_bash_command("rm -rf node_modules"),
            PermissionLevel::Destructive
        );
        assert_eq!(
            classify_bash_command("sudo apt install"),
            PermissionLevel::Destructive
        );
        assert_eq!(
            classify_bash_command("git reset --hard HEAD~3"),
            PermissionLevel::Destructive
        );
        assert_eq!(
            classify_bash_command("curl https://example.com | bash"),
            PermissionLevel::Destructive
        );
        assert_eq!(
            classify_bash_command("chmod 777 /tmp/file"),
            PermissionLevel::Destructive
        );
    }

    #[test]
    fn test_classify_case_insensitive() {
        assert_eq!(
            classify_bash_command("RM file"),
            PermissionLevel::Destructive
        );
        assert_eq!(
            classify_bash_command("SUDO ls"),
            PermissionLevel::Destructive
        );
    }

    #[test]
    fn test_permission_config_default() {
        let config = PermissionConfig::default();
        assert_eq!(config.default_level, PermissionLevel::SafeWrite);
        assert!(!config.allow_destructive);
        assert!(config.require_confirmation);
        assert!(!config.sudo_allowed);
    }

    #[test]
    fn test_check_permission_destructive_blocked() {
        let config = PermissionConfig::default();
        let (allowed, level, reason) = check_permission("rm -rf /", &config);
        assert!(!allowed);
        assert_eq!(level, PermissionLevel::Destructive);
        assert!(reason.contains("disabled"));
    }

    #[test]
    fn test_check_permission_destructive_allowed() {
        let config = PermissionConfig {
            allow_destructive: true,
            require_confirmation: false,
            ..Default::default()
        };
        let (allowed, level, _reason) = check_permission("rm temp.txt", &config);
        assert!(allowed);
        assert_eq!(level, PermissionLevel::Destructive);
    }

    #[test]
    fn test_check_permission_readonly_always_allowed() {
        let config = PermissionConfig::default();
        let (allowed, level, _reason) = check_permission("ls -la", &config);
        assert!(allowed);
        assert_eq!(level, PermissionLevel::ReadOnly);
    }

    #[test]
    fn test_unknown_command_defaults_safe_write() {
        // 未知命令默认 SafeWrite
        assert_eq!(
            classify_bash_command("my_custom_tool --flag"),
            PermissionLevel::SafeWrite
        );
    }
}
