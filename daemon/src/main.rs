mod adapter;
mod api;
mod ipc;
mod router;
mod scheduler;
mod skin;
mod state;
mod watchdog;

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// CatCoding Daemon — 多 Agent 协同开发框架核心守护进程
///
/// 职责：调度、监控、协调多个 AI Agent 共同完成软件开发任务
/// 技术栈：Rust (tokio + axum + NATS + SQLite)
/// 目标：内存 < 10MB，CPU 空闲 < 1%

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "catcoding=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🐱 CatCoding Daemon v{} 启动中...", env!("CARGO_PKG_VERSION"));
    tracing::info!("🐾 暹罗猫（PM）已就位，准备接收任务！");
    tracing::info!("🦉 猫头鹰（Watchdog）开始监视...");

    // TODO: Phase 1 - 启动各子模块
    // - state: 初始化 SQLite + NATS KV
    // - router: 连接 NATS，建立消息通道
    // - scheduler: 启动任务调度循环
    // - watchdog: 启动进程监控
    // - api: 启动 Axum HTTP/WebSocket 服务器 (端口 19800)

    tracing::info!("✅ Daemon 启动完成，监听端口 19800");
    tracing::info!("🌐 Dashboard: http://localhost:19800");
    tracing::info!("按 Ctrl+C 停止");

    // 保持运行（后续替换为实际的服务器启动）
    tokio::signal::ctrl_c().await?;
    tracing::info!("🐱 收到停止信号，所有猫咪下班！");

    Ok(())
}
