/// i18n — 国际化支持
/// 
/// 支持中英双语，根据系统语言自动切换
/// 默认英文

use std::sync::OnceLock;

static LANG: OnceLock<Language> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    En,
    Zh,
}

impl Language {
    /// 从系统环境检测语言
    pub fn detect() -> Self {
        // 优先检查 LANG 环境变量
        if let Ok(lang) = std::env::var("LANG") {
            if lang.starts_with("zh") {
                return Language::Zh;
            }
        }
        
        // 检查 LC_ALL
        if let Ok(lang) = std::env::var("LC_ALL") {
            if lang.starts_with("zh") {
                return Language::Zh;
            }
        }
        
        // 检查 LC_MESSAGES
        if let Ok(lang) = std::env::var("LC_MESSAGES") {
            if lang.starts_with("zh") {
                return Language::Zh;
            }
        }
        
        Language::En
    }
}

/// 获取当前语言
pub fn lang() -> Language {
    *LANG.get_or_init(Language::detect)
}

/// 设置语言
pub fn set_lang(l: Language) {
    let _ = LANG.set(l);
}

/// 翻译宏
#[macro_export]
macro_rules! t {
    ($key:expr) => {
        $crate::i18n::translate($key)
    };
}

/// 翻译函数
pub fn translate(key: &str) -> &'static str {
    match lang() {
        Language::En => translate_en(key),
        Language::Zh => translate_zh(key),
    }
}

fn translate_en(key: &str) -> &'static str {
    match key {
        // CLI
        "cli.init" => "Initialize CatCoding project",
        "cli.serve" => "Start Daemon",
        "cli.status" => "Show status",
        "cli.logs" => "Show logs",
        "cli.command" => "Send command to team",
        "cli.version" => "CatCoding — Multi-Agent Collaborative Development Framework",
        
        // Daemon
        "daemon.starting" => "Starting CatCoding Daemon...",
        "daemon.started" => "Daemon started!",
        "daemon.stopped" => "Daemon stopped.",
        "daemon.port" => "Port",
        "daemon.dashboard" => "Dashboard",
        "daemon.api" => "API",
        "daemon.database" => "Database",
        
        // Components
        "watchdog.name" => "Watchdog (Owl)",
        "watchdog.heartbeat" => "Heartbeat interval",
        "watchdog.timeout" => "Timeout",
        "scheduler.name" => "Scheduler",
        "scheduler.check_interval" => "Check interval",
        "scheduler.max_concurrent" => "Max concurrent",
        "memory.name" => "L4 Memory System",
        "memory.l1" => "L1 Index",
        "memory.l2" => "L2 Facts",
        "memory.l3" => "L3 Skills",
        "memory.l4" => "L4 Sessions",
        
        // Status
        "status.agents" => "Agents",
        "status.tasks" => "Tasks",
        "status.running" => "Running",
        "status.idle" => "Idle",
        
        // Errors
        "error.nats_connect" => "NATS connection failed",
        "error.nats_hint" => "Hint: Ensure NATS Server is running at",
        "error.daemon_not_running" => "Daemon is not running. Start it with `catcoding serve`",
        
        // Success
        "success.init" => "Project initialized!",
        "success.config" => "Config created",
        "success.team_ready" => "Cat team is ready! Run `catcoding serve` to start",
        
        // Dashboard
        "dashboard.title" => "CatCoding Dashboard",
        "dashboard.kanban" => "Kanban Board",
        "dashboard.gantt" => "Gantt Chart",
        "dashboard.agents" => "Cat Panel",
        "dashboard.logs" => "Kitchen Logs",
        "dashboard.command" => "Command Input",
        "dashboard.language" => "Language",
        "dashboard.lang_en" => "English",
        "dashboard.lang_zh" => "中文",
        "dashboard.daemon_running" => "Daemon Running",
        
        _ => key,
    }
}

fn translate_zh(key: &str) -> &'static str {
    match key {
        // CLI
        "cli.init" => "初始化 CatCoding 项目",
        "cli.serve" => "启动 Daemon 守护进程",
        "cli.status" => "查看状态",
        "cli.logs" => "查看日志",
        "cli.command" => "向猫咪团队发送指令",
        "cli.version" => "🐱 CatCoding — 多 Agent 协同开发框架",
        
        // Daemon
        "daemon.starting" => "🐱 CatCoding Daemon 启动中...",
        "daemon.started" => "✅ Daemon 启动完成！",
        "daemon.stopped" => "Daemon 已停止",
        "daemon.port" => "端口",
        "daemon.dashboard" => "Dashboard",
        "daemon.api" => "API",
        "daemon.database" => "数据库",
        
        // Components
        "watchdog.name" => "🦉 猫头鹰（Watchdog）",
        "watchdog.heartbeat" => "心跳间隔",
        "watchdog.timeout" => "超时",
        "scheduler.name" => "📋 调度器",
        "scheduler.check_interval" => "检查间隔",
        "scheduler.max_concurrent" => "最大并发",
        "memory.name" => "🧠 L4 记忆系统",
        "memory.l1" => "L1 索引",
        "memory.l2" => "L2 事实",
        "memory.l3" => "L3 技能",
        "memory.l4" => "L4 会话",
        
        // Status
        "status.agents" => "Agent",
        "status.tasks" => "任务",
        "status.running" => "运行中",
        "status.idle" => "空闲",
        
        // Errors
        "error.nats_connect" => "⚠️ NATS 连接失败",
        "error.nats_hint" => "💡 提示: 确保 NATS Server 在",
        "error.daemon_not_running" => "⚠️ Daemon 未运行，请先执行 `catcoding serve`",
        
        // Success
        "success.init" => "✅ 项目初始化完成！",
        "success.config" => "✅ 配置文件已创建",
        "success.team_ready" => "🐾 猫咪团队已准备好！运行 `catcoding serve` 启动",
        
        // Dashboard
        "dashboard.title" => "CatCoding Dashboard",
        "dashboard.kanban" => "看板视图",
        "dashboard.gantt" => "甘特图",
        "dashboard.agents" => "猫咪面板",
        "dashboard.logs" => "厨房日志",
        "dashboard.command" => "指令输入",
        "dashboard.language" => "语言",
        "dashboard.lang_en" => "English",
        "dashboard.lang_zh" => "中文",
        "dashboard.daemon_running" => "Daemon 运行中",
        
        _ => key,
    }
}
