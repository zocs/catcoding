use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::history::DefaultHistory;
use rustyline::validate::Validator;
use rustyline::{Config, Editor, Helper};
use serde::Deserialize;
use std::borrow::Cow;

const DEFAULT_DAEMON: &str = "http://127.0.0.1:19800";

/// 🐱 CatCoding — 多 Agent 协同软件开发框架
#[derive(Parser)]
#[command(name = "catc", version, about, long_about = None)]
struct Cli {
    /// Daemon 地址
    #[arg(short, long, default_value = DEFAULT_DAEMON, env = "CATCODING_DAEMON")]
    daemon: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 🐱 初始化项目
    Init {
        #[arg(short, long)]
        name: Option<String>,
    },

    /// 🦉 启动 Daemon
    Serve {
        #[arg(short, long, default_value = "19800")]
        port: u16,
        #[arg(short, long)]
        foreground: bool,
    },

    /// 📋 查看状态
    Status,

    /// 📨 发送指令
    Command {
        message: Vec<String>,
    },

    /// 🐭 列出 Bug
    Mice,

    /// 🐟 投喂猫咪
    Feed {
        agent: Option<String>,
    },

    /// 📜 查看日志
    Logs {
        #[arg(short, long)]
        follow: bool,
        #[arg(short, long, default_value = "50")]
        tail: usize,
    },

    /// 🖥️  启动交互式 Shell
    Shell,
}

// ━━━ API 类型 ━━━

#[derive(Debug, Deserialize)]
struct HealthResp {
    status: String,
    version: String,
}

#[derive(Debug, Deserialize)]
struct AgentInfo {
    id: String,
    role: String,
    name: String,
    status: String,
    #[serde(default)]
    current_task: Option<String>,
    #[serde(default)]
    tasks_completed: u32,
}

#[derive(Debug, Deserialize)]
struct AgentsResp {
    agents: Vec<AgentInfo>,
}

#[derive(Debug, Deserialize)]
struct TaskInfo {
    id: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    assigned_to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TasksResp {
    tasks: Vec<TaskInfo>,
}

// ━━━ HTTP Helper ━━━

fn api_get<T: serde::de::DeserializeOwned>(base: &str, path: &str) -> Result<T> {
    let url = format!("{}{}", base, path);
    let resp = ureq::get(&url)
        .call()
        .with_context(|| format!("无法连接 Daemon: {}", url))?;
    Ok(resp.into_json()?)
}

fn api_post(base: &str, path: &str, body: serde_json::Value) -> Result<serde_json::Value> {
    let url = format!("{}{}", base, path);
    let resp = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .with_context(|| format!("无法连接 Daemon: {}", url))?;
    Ok(resp.into_json()?)
}

fn check_daemon(base: &str) -> Result<()> {
    let _: HealthResp = api_get(base, "/api/health")
        .with_context(|| format!("Daemon 未运行！启动: catc serve\n地址: {}", base))?;
    Ok(())
}

// ━━━ Tab 补全器 ━━━

const COMMANDS: &[&str] = &[
    "status", "command", "mice", "feed", "logs", "serve", "init", "shell", "help", "exit", "quit",
];

struct CatHelper;

impl Helper for CatHelper {}

impl Highlighter for CatHelper {}

impl Validator for CatHelper {
    fn validate(&self, _ctx: &mut rustyline::validate::ValidationContext) -> rustyline::Result<rustyline::validate::ValidationResult> {
        Ok(rustyline::validate::ValidationResult::Valid(None))
    }
}

impl Completer for CatHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let start = line[..pos].rfind(' ').map_or(0, |i| i + 1);
        let word = &line[start..pos];

        let matches: Vec<Pair> = COMMANDS
            .iter()
            .filter(|cmd| cmd.starts_with(word))
            .map(|cmd| Pair {
                display: cmd.to_string(),
                replacement: cmd.to_string(),
            })
            .collect();

        Ok((start, matches))
    }
}

impl Hinter for CatHelper {
    type Hint = String;
    fn hint(&self, line: &str, _pos: usize, _ctx: &rustyline::Context<'_>) -> Option<String> {
        if line.is_empty() {
            return Some(" 输入命令 (status/mice/feed/...) 或 help".to_string());
        }
        None
    }
}

// ━━━ 交互式 Shell ━━━

fn run_shell(daemon: &str) -> Result<()> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(rustyline::CompletionType::List)
        .build();

    let helper = CatHelper;
    let mut rl = Editor::<CatHelper, DefaultHistory>::with_config(config)?;
    rl.set_helper(Some(helper));

    // 加载历史
    let history_path = dirs::home_dir()
        .map(|h| h.join(".catcoding_history"));
    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    println!("{}", "🐱 CatCoding 交互式终端".bright_yellow().bold());
    println!("  输入 {} 查看可用命令，{} 退出", "help".bright_green(), "exit".bright_red());
    println!();

    loop {
        let prompt = format!("{} ", "🐱 >".bright_green());
        match rl.readline(&prompt) {
            Ok(line) => {
                let line = line.trim().to_string();
                if line.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(&line);

                let parts: Vec<&str> = line.split_whitespace().collect();
                let cmd = parts.first().copied().unwrap_or("");
                let args = &parts[1..];

                match cmd {
                    "exit" | "quit" | "q" => {
                        println!("🐾 再见！");
                        break;
                    }
                    "help" | "?" => print_help(),
                    "status" => {
                        if let Err(e) = cmd_status(daemon) {
                            eprintln!("❌ {}", e);
                        }
                    }
                    "mice" => {
                        if let Err(e) = cmd_mice(daemon) {
                            eprintln!("❌ {}", e);
                        }
                    }
                    "feed" => {
                        let agent = if args.is_empty() { None } else { Some(args.join(" ")) };
                        if let Err(e) = cmd_feed(daemon, agent) {
                            eprintln!("❌ {}", e);
                        }
                    }
                    "command" => {
                        if args.is_empty() {
                            eprintln!("用法: command <消息>");
                        } else {
                            let msg: Vec<String> = args.iter().map(|s| s.to_string()).collect();
                            if let Err(e) = cmd_command(daemon, msg) {
                                eprintln!("❌ {}", e);
                            }
                        }
                    }
                    "logs" => {
                        if let Err(e) = cmd_logs(daemon, false, 50) {
                            eprintln!("❌ {}", e);
                        }
                    }
                    "health" => {
                        match api_get::<HealthResp>(daemon, "/api/health") {
                            Ok(h) => println!("● Daemon {} — {}", h.version, h.status.bright_green()),
                            Err(e) => eprintln!("❌ {}", e),
                        }
                    }
                    other => {
                        println!("❓ 未知命令: {} (输入 help 查看可用命令)", other.bright_red());
                    }
                }
                println!();
            }
            Err(ReadlineError::Interrupted) => {
                println!("🐾 Ctrl+C — 输入 exit 退出");
            }
            Err(ReadlineError::Eof) => {
                println!("🐾 再见！");
                break;
            }
            Err(err) => {
                eprintln!("❌ 读取输入错误: {}", err);
                break;
            }
        }
    }

    // 保存历史
    if let Some(ref path) = history_path {
        let _ = rl.save_history(path);
    }

    Ok(())
}

fn print_help() {
    println!("{}", "🐱 可用命令:".bright_yellow().bold());
    println!("  {:<16} {}", "status".bright_green(), "查看 Agent 和任务状态");
    println!("  {:<16} {}", "mice".bright_green(), "列出所有 Bug");
    println!("  {:<16} {}", "feed [agent]".bright_green(), "投喂猫咪");
    println!("  {:<16} {}", "command <msg>".bright_green(), "发送指令给团队");
    println!("  {:<16} {}", "logs".bright_green(), "查看日志");
    println!("  {:<16} {}", "health".bright_green(), "检查 Daemon 状态");
    println!("  {:<16} {}", "help".bright_green(), "显示帮助");
    println!("  {:<16} {}", "exit".bright_red(), "退出");
    println!();
    println!("  {} 上下键: 历史记录 | Tab: 自动补全", "⌨️".bright_cyan());
}

// ━━━ 主函数 ━━━

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            // 无子命令 → 启动交互式 shell
            run_shell(&cli.daemon)
        }
        Some(cmd) => match cmd {
            Commands::Init { name } => cmd_init(name),
            Commands::Serve { port, foreground } => cmd_serve(port, foreground),
            Commands::Status => cmd_status(&cli.daemon),
            Commands::Command { message } => cmd_command(&cli.daemon, message),
            Commands::Mice => cmd_mice(&cli.daemon),
            Commands::Feed { agent } => cmd_feed(&cli.daemon, agent),
            Commands::Logs { follow, tail } => cmd_logs(&cli.daemon, follow, tail),
            Commands::Shell => run_shell(&cli.daemon),
        },
    }
}

// ━━━ 子命令实现 ━━━

fn cmd_init(name: Option<String>) -> Result<()> {
    let project_name = name.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "my-project".to_string())
    });

    println!("🐱 初始化 CatCoding 项目: {}", project_name.bright_yellow());
    let config_dir = std::path::Path::new(".catcoding");
    std::fs::create_dir_all(config_dir)?;

    let roles_yaml = format!(
        r#"project:
  name: "{}"
  tech_stack: []
  auto_detect: true

agents:
  - role: pm
    auto: true
  - role: core_dev
    auto: true
  - role: reviewer
    auto: true
    trigger: on_task_complete

coordination:
  mode: auto
"#,
        project_name
    );
    std::fs::write(config_dir.join("roles.yaml"), roles_yaml)?;
    println!("✅ 创建 .catcoding/roles.yaml");
    println!("🐾 运行 {} 启动", "catc serve".bright_green());
    Ok(())
}

fn cmd_serve(port: u16, foreground: bool) -> Result<()> {
    let daemon_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("catcoding-daemon")))
        .filter(|p| p.exists());

    let daemon_path = match daemon_path {
        Some(p) => p,
        None => match which::which("catcoding-daemon") {
            Ok(p) => p,
            Err(_) => {
                eprintln!("❌ catcoding-daemon 未找到！构建: cargo build --release");
                std::process::exit(1);
            }
        },
    };

    let mut cmd = std::process::Command::new(&daemon_path);
    cmd.env("API_PORT", port.to_string());

    if foreground {
        println!("🦉 前台启动 Daemon (port={})...", port);
        let status = cmd.status()?;
        std::process::exit(status.code().unwrap_or(1));
    } else {
        let child = cmd.spawn()?;
        println!("✅ Daemon PID: {}", child.id());
        println!("🌐 Dashboard: http://localhost:{}", port);
    }
    Ok(())
}

fn cmd_status(daemon: &str) -> Result<()> {
    check_daemon(daemon)?;

    let health: HealthResp = api_get(daemon, "/api/health")?;
    println!("● Daemon {} — {}", health.version, "在线".bright_green());
    println!();

    let agents: AgentsResp = api_get(daemon, "/api/agents")?;
    if agents.agents.is_empty() {
        println!("🐱 暂无 Agent 运行");
    } else {
        println!("{}", "🐱 Agent 状态".bright_yellow().bold());
        for a in &agents.agents {
            let s = match a.status.as_str() {
                "active" => a.status.bright_green(),
                "idle" => a.status.bright_yellow(),
                "error" => a.status.bright_red(),
                _ => a.status.normal(),
            };
            let task = a.current_task.as_deref().unwrap_or("—");
            println!("  {} {} [{}] {}", a.name, a.role.dimmed(), s, task);
        }
    }
    println!();

    let tasks: TasksResp = api_get(daemon, "/api/tasks")?;
    let active = tasks.tasks.iter().filter(|t| t.status != "done").count();
    let done = tasks.tasks.len() - active;
    println!("📋 {} 活跃, {} 完成, 共 {}", active.to_string().bright_cyan(), done.to_string().bright_green(), tasks.tasks.len());

    Ok(())
}

fn cmd_command(daemon: &str, message: Vec<String>) -> Result<()> {
    check_daemon(daemon)?;
    let msg = message.join(" ");
    println!("📨 {}", msg.bright_cyan());
    let body = serde_json::json!({ "command": msg });
    let resp = api_post(daemon, "/api/command", body)?;
    println!("✅ {}", resp);
    Ok(())
}

fn cmd_mice(daemon: &str) -> Result<()> {
    check_daemon(daemon)?;
    let tasks: TasksResp = api_get(daemon, "/api/tasks")?;
    let bugs: Vec<_> = tasks.tasks.iter()
        .filter(|t| t.status == "blocked" || t.status == "failed")
        .collect();

    if bugs.is_empty() {
        println!("🐭 没有发现 Bug！🎉");
    } else {
        println!("🐭 {} 个 Bug:", bugs.len().to_string().bright_red().bold());
        for t in &bugs {
            let id = &t.id[..8.min(t.id.len())];
            println!("  🐛 [{}] {} — {}", id.bright_red(), t.status, t.summary);
        }
    }
    Ok(())
}

fn cmd_feed(daemon: &str, agent: Option<String>) -> Result<()> {
    check_daemon(daemon)?;
    let agents: AgentsResp = api_get(daemon, "/api/agents")?;

    if agents.agents.is_empty() {
        println!("🐱 没有 Agent 可投喂");
        return Ok(());
    }

    match agent {
        Some(name) => {
            if let Some(a) = agents.agents.iter().find(|a| a.name == name || a.id == name) {
                println!("🐟 投喂 {}...", a.name.bright_yellow());
            } else {
                println!("❌ 找不到: {}", name);
            }
        }
        None => {
            for a in &agents.agents {
                let food = match a.role.as_str() {
                    "mascot" => "🎋", "tech_scout" => "🫐", "watchdog" => "🐭", _ => "🐟",
                };
                println!("  {} {}", food, a.name);
            }
        }
    }
    Ok(())
}

fn cmd_logs(_daemon: &str, _follow: bool, tail: usize) -> Result<()> {
    println!("📜 日志 (tail={})", tail);
    println!("  用 {} 查看 daemon 终端输出", "catc serve -f".bright_green());
    Ok(())
}
