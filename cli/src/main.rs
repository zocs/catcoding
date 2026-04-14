use anyhow::Result;
use clap::{Parser, Subcommand};

/// 🐱 CatCoding — 多 Agent 协同软件开发框架
///
/// 让 AI 像猫咪团队一样协作做菜（写代码）
#[derive(Parser)]
#[command(name = "catcoding", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 🐱 初始化项目（创建 .catcoding 目录和配置）
    Init {
        /// 项目名称
        #[arg(short, long)]
        name: Option<String>,
    },

    /// 🦉 启动 Daemon 守护进程
    Serve {
        /// 监听端口
        #[arg(short, long, default_value = "19800")]
        port: u16,
        /// 前台运行（不 daemonize）
        #[arg(short, long)]
        foreground: bool,
    },

    /// 📋 查看项目/Agent/任务状态
    Status {
        /// 输出格式
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// 📜 查看日志
    Logs {
        /// 跟踪输出
        #[arg(short, long)]
        follow: bool,
        /// 显示行数
        #[arg(short, long, default_value = "100")]
        tail: usize,
    },

    /// 📨 向猫咪团队发送指令
    Command {
        /// 指令内容
        message: Vec<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => {
            let project_name = name.unwrap_or_else(|| {
                std::env::current_dir()
                    .ok()
                    .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                    .unwrap_or_else(|| "my-project".to_string())
            });
            println!("🐱 初始化 CatCoding 项目: {}", project_name);

            // 创建 .catcoding 目录
            let config_dir = std::path::Path::new(".catcoding");
            std::fs::create_dir_all(config_dir)?;

            // 创建默认 roles.yaml
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
            println!("🐾 猫咪团队已准备好！运行 `catcoding serve` 启动");
        }

        Commands::Serve { port, foreground } => {
            println!("🦉 Starting CatCoding Daemon...");
            println!("📡 Port: {}", port);
            println!("🌐 Dashboard: http://localhost:{}", port);

            // 查找 catcoding-daemon 可执行文件
            let daemon_path = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("catcoding-daemon")))
                .filter(|p| p.exists());

            let daemon_path = match daemon_path {
                Some(p) => p,
                None => {
                    // 尝试在 PATH 中查找
                    match which::which("catcoding-daemon") {
                        Ok(p) => p,
                        Err(_) => {
                            eprintln!("❌ catcoding-daemon not found!");
                            eprintln!("💡 Build it with: cargo build --release");
                            std::process::exit(1);
                        }
                    }
                }
            };

            // 构建参数
            let mut cmd = std::process::Command::new(&daemon_path);
            cmd.env("API_PORT", port.to_string());

            if foreground {
                // 前台运行
                println!("🏃 Running in foreground...");
                let status = cmd.status()?;
                std::process::exit(status.code().unwrap_or(1));
            } else {
                // 后台运行
                println!("💡 Use --foreground to run in foreground");
                let child = cmd.spawn()?;
                println!("✅ Daemon started (PID: {})", child.id());
                println!("🌐 Dashboard: http://localhost:{}", port);
            }
        }

        Commands::Status { format } => {
            println!("📋 CatCoding 状态");
            println!("格式: {}", format);
            // TODO: 通过 API 获取状态
            println!("⚠️  请先运行 `catcoding serve` 启动 Daemon");
        }

        Commands::Logs { follow, tail } => {
            println!("📜 日志 (tail={}, follow={})", tail, follow);
            // TODO: 通过 SSE 流获取日志
            println!("⚠️  请先运行 `catcoding serve` 启动 Daemon");
        }

        Commands::Command { message } => {
            let msg = message.join(" ");
            println!("📨 发送指令: {}", msg);
            // TODO: 通过 API 发送指令
            println!("⚠️  请先运行 `catcoding serve` 启动 Daemon");
        }
    }

    Ok(())
}
