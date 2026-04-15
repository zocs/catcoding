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

/// 🐱 CatCoding — Multi-Agent Collaborative Dev Framework
#[derive(Parser)]
#[command(name = "catcoding", version, about, long_about = None)]
struct Cli {
    /// Daemon address
    #[arg(short, long, default_value = DEFAULT_DAEMON, env = "CATCODING_DAEMON")]
    daemon: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// 🐱 Initialize project
    Init {
        #[arg(short, long)]
        name: Option<String>,
    },

    /// 🦉 Start daemon
    Serve {
        #[arg(short, long, default_value = "19800")]
        port: u16,
        #[arg(short, long)]
        foreground: bool,
    },

    /// 📋 Show status
    Status,

    /// 📨 Send command to team
    Command {
        message: Vec<String>,
    },

    /// 🐭 List bugs
    Mice,

    /// 🐟 Feed cat agents
    Feed {
        agent: Option<String>,
    },

    /// 📜 View logs
    Logs {
        #[arg(short, long)]
        follow: bool,
        #[arg(short, long, default_value = "50")]
        tail: usize,
    },

    /// 🖥️  Interactive shell
    Shell,
}

// ━━━ API Types ━━━

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
        .with_context(|| format!("Cannot reach daemon at {}", url))?;
    Ok(resp.into_json()?)
}

fn api_post(base: &str, path: &str, body: serde_json::Value) -> Result<serde_json::Value> {
    let url = format!("{}{}", base, path);
    let resp = ureq::post(&url)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string())
        .with_context(|| format!("Cannot reach daemon at {}", url))?;
    Ok(resp.into_json()?)
}

fn check_daemon(base: &str) -> Result<()> {
    let _: HealthResp = api_get(base, "/api/health").with_context(|| {
        format!(
            "Daemon is not running! Start it with: catcoding serve\nAddress: {}",
            base
        )
    })?;
    Ok(())
}

// ━━━ Tab Completer ━━━

const COMMANDS: &[&str] = &[
    "status", "command", "mice", "feed", "logs", "serve", "init", "shell", "help", "exit", "quit",
];

struct CatHelper;

impl Helper for CatHelper {}

impl Highlighter for CatHelper {}

impl Validator for CatHelper {
    fn validate(
        &self,
        _ctx: &mut rustyline::validate::ValidationContext,
    ) -> rustyline::Result<rustyline::validate::ValidationResult> {
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
            return Some(" type a command (status/mice/feed/...) or help".to_string());
        }
        None
    }
}

// ━━━ Interactive Shell ━━━

fn run_shell(daemon: &str) -> Result<()> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(rustyline::CompletionType::List)
        .build();

    let helper = CatHelper;
    let mut rl = Editor::<CatHelper, DefaultHistory>::with_config(config)?;
    rl.set_helper(Some(helper));

    let history_path = dirs::home_dir().map(|h| h.join(".catcoding_history"));
    if let Some(ref path) = history_path {
        let _ = rl.load_history(path);
    }

    println!("{}", "🐱 CatCoding Interactive Shell".bright_yellow().bold());
    println!(
        "  Type {} for commands, {} to exit",
        "help".bright_green(),
        "exit".bright_red()
    );
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
                        println!("🐾 Bye!");
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
                        let agent = if args.is_empty() {
                            None
                        } else {
                            Some(args.join(" "))
                        };
                        if let Err(e) = cmd_feed(daemon, agent) {
                            eprintln!("❌ {}", e);
                        }
                    }
                    "command" => {
                        if args.is_empty() {
                            eprintln!("Usage: command <message>");
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
                    "health" => match api_get::<HealthResp>(daemon, "/api/health") {
                        Ok(h) => println!("● Daemon {} — {}", h.version, h.status.bright_green()),
                        Err(e) => eprintln!("❌ {}", e),
                    },
                    other => {
                        println!(
                            "❓ Unknown command: {} (type {} for help)",
                            other.bright_red(),
                            "help".bright_green()
                        );
                    }
                }
                println!();
            }
            Err(ReadlineError::Interrupted) => {
                println!("🐾 Ctrl+C — type {} to exit", "exit".bright_red());
            }
            Err(ReadlineError::Eof) => {
                println!("🐾 Bye!");
                break;
            }
            Err(err) => {
                eprintln!("❌ Readline error: {}", err);
                break;
            }
        }
    }

    if let Some(ref path) = history_path {
        let _ = rl.save_history(path);
    }

    Ok(())
}

fn print_help() {
    println!("{}", "🐱 Available commands:".bright_yellow().bold());
    println!("  {:<16} {}", "status".bright_green(), "Show agent & task status");
    println!("  {:<16} {}", "mice".bright_green(), "List all bugs");
    println!("  {:<16} {}", "feed [agent]".bright_green(), "Feed cat agents");
    println!("  {:<16} {}", "command <msg>".bright_green(), "Send command to team");
    println!("  {:<16} {}", "logs".bright_green(), "Show recent logs");
    println!("  {:<16} {}", "health".bright_green(), "Check daemon health");
    println!("  {:<16} {}", "help".bright_green(), "Show this help");
    println!("  {:<16} {}", "exit".bright_red(), "Exit shell");
    println!();
    println!(
        "  {} Up/Down: history | Tab: autocomplete",
        "⌨️".bright_cyan()
    );
}

// ━━━ Main ━━━

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => run_shell(&cli.daemon),
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

// ━━━ Subcommand Implementations ━━━

fn cmd_init(name: Option<String>) -> Result<()> {
    let project_name = name.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "my-project".to_string())
    });

    println!(
        "🐱 Initializing CatCoding project: {}",
        project_name.bright_yellow()
    );
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
    println!("✅ Created .catcoding/roles.yaml");
    println!("🐾 Run {} to start", "catcoding serve".bright_green());
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
                eprintln!("❌ catcoding-daemon not found! Build with: cargo build --release");
                std::process::exit(1);
            }
        },
    };

    let mut cmd = std::process::Command::new(&daemon_path);
    cmd.env("API_PORT", port.to_string());

    if foreground {
        println!("🦉 Starting daemon in foreground (port={})...", port);
        let status = cmd.status()?;
        std::process::exit(status.code().unwrap_or(1));
    } else {
        let child = cmd.spawn()?;
        println!("✅ Daemon started (PID: {})", child.id());
        println!("🌐 Dashboard: http://localhost:{}", port);
    }
    Ok(())
}

fn cmd_status(daemon: &str) -> Result<()> {
    check_daemon(daemon)?;

    let health: HealthResp = api_get(daemon, "/api/health")?;
    println!(
        "● Daemon {} — {}",
        health.version,
        "online".bright_green()
    );
    println!();

    let agents: AgentsResp = api_get(daemon, "/api/agents")?;
    if agents.agents.is_empty() {
        println!("🐱 No agents running");
    } else {
        println!("{}", "🐱 Agent Status".bright_yellow().bold());
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
    println!(
        "📋 {} active, {} done, {} total",
        active.to_string().bright_cyan(),
        done.to_string().bright_green(),
        tasks.tasks.len()
    );

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
    let bugs: Vec<_> = tasks
        .tasks
        .iter()
        .filter(|t| t.status == "blocked" || t.status == "failed")
        .collect();

    if bugs.is_empty() {
        println!("🐭 No bugs found! 🎉");
    } else {
        println!(
            "🐭 {} bug(s):",
            bugs.len().to_string().bright_red().bold()
        );
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
        println!("🐱 No agents to feed");
        return Ok(());
    }

    match agent {
        Some(name) => {
            if let Some(a) = agents.agents.iter().find(|a| a.name == name || a.id == name) {
                println!("🐟 Feeding {}...", a.name.bright_yellow());
            } else {
                println!("❌ Agent not found: {}", name);
            }
        }
        None => {
            for a in &agents.agents {
                let food = match a.role.as_str() {
                    "mascot" => "🎋",
                    "tech_scout" => "🫐",
                    "watchdog" => "🐭",
                    _ => "🐟",
                };
                println!("  {} {} — fed!", food, a.name);
            }
        }
    }
    Ok(())
}

fn cmd_logs(_daemon: &str, _follow: bool, tail: usize) -> Result<()> {
    println!("📜 Logs (tail={})", tail);
    println!(
        "  Use {} to see daemon terminal output",
        "catcoding serve -f".bright_green()
    );
    Ok(())
}
