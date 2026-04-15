# 🐱 CatCoding

> 让 AI 像猫咪团队一样协作写代码

**[🎮 在线演示](https://demo.catcoding.org)** · **[English](README.md)** · **[快速开始](docs/guide/getting-started-zh.md)** · **[官网](https://catcoding.org)**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![CI](https://github.com/zocs/catcoding/actions/workflows/ci.yml/badge.svg)](https://github.com/zocs/catcoding/actions/workflows/ci.yml)

CatCoding 是一个**独立于任何 AI agent 框架**的多 Agent 协同软件开发框架。通过 Adapter 接口支持多种 AI 编程 agent（Hermes、Claude Code、Codex 等），将多个 AI agent 组织成高效协作的开发团队。

## ✨ 核心特性

- 🐱 **猫咪形象系统** — 每个 Agent 都有独特的猫咪角色和 SVG 头像
- 🦉 **Watchdog 守护** — 三重检测框架，自动恢复故障
- 📋 **可视化看板** — 实时看板、甘特图、Agent 状态监控
- 🔄 **多 Adapter** — 即插即用：Hermes、Claude Code、Codex、OpenClaw
- 🧠 **L4 记忆系统** — 四层分级：索引 → 事实 → 技能 → 会话
- 🐛 **Bug = 老鼠** — 有趣的 Bug 分级系统（抓老鼠 = 修 Bug）

## 🏗️ 架构

```
┌──────────────────────────────────────────────┐
│                    用户层                     │
│         Dashboard (Vue 3)      CLI           │
└────────────────────┬─────────────────────────┘
                     │
┌────────────────────▼─────────────────────────┐
│          CatCoding Daemon (Rust)              │
│  ┌───────────┐ ┌───────────┐ ┌────────────┐  │
│  │  Watchdog │ │ Scheduler │ │   Router   │  │
│  │  三重检测  │ │ 依赖门控   │ │  NATS 消息  │  │
│  └───────────┘ └───────────┘ └────────────┘  │
│  ┌──────────────────────────────────────┐    │
│  │             Adapter 层               │    │
│  │    Hermes  │  Claude Code  │  Codex  │    │
│  └──────────────────────────────────────┘    │
└────────────────────┬─────────────────────────┘
                     │
┌────────────────────▼─────────────────────────┐
│            Python Agent SDK                   │
│  PM (暹罗猫) │ Dev (英短) │ Review (玄猫)     │
└──────────────────────────────────────────────┘
```

## 🚀 快速开始

完整指南见 **[快速开始文档](docs/guide/getting-started.md)**。

```bash
# 一键安装（即将支持）
curl -fsSL https://catcoding.org/install.sh | bash

# 或从源码编译
git clone https://github.com/zocs/catcoding.git
cd catcoding && cargo build --release

# 在项目中初始化
cd your-project && catcoding init

# 启动 Daemon + Dashboard
catcoding serve
# → Dashboard: http://localhost:8080
# → API: http://127.0.0.1:9527
```

## 🐱 猫咪团队

| 角色 | 猫咪 | 职责 |
|------|------|------|
| PM | 🐱 暹罗猫 | 需求分析、任务拆分、进度监控 |
| 核心开发 | 🐱 英短蓝猫 | 代码实现 |
| 代码审查 | 🐱 玄猫 | 代码审查、Bug 检测 |
| 测试 | 🐱 阿比西尼亚猫 | 测试用例、质量保障 |
| 技术侦察 | 🦊 狐狸 | 技术调研、文档搜集 |

## 🔌 支持的 Adapter

| Adapter | 状态 | 说明 |
|---------|------|------|
| Hermes | ✅ | hermes-agent 集成 |
| Claude Code | ✅ | claude CLI 集成 |
| Codex | ✅ | codex CLI 集成 |
| OpenCode | 🔜 | 计划中 |

## 🛠️ 开发

```bash
cargo build              # Debug 构建
cargo build --release    # Release 构建
cargo test               # 运行测试
make ci                  # 完整 CI 流水线
```

## 📝 License

[MIT License](LICENSE)
