# 🐱 CatCoding

> 让 AI 像猫咪团队一样协作做菜（写代码）

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)

CatCoding 是一个**独立于任何 AI agent 框架**的多 Agent 协同软件开发框架。通过 Adapter 接口支持多种 AI 编程 agent，将多个 AI agent 组织成高效协作的开发团队。

## ✨ 特性

- 🐱 **猫咪形象系统** — 每个 Agent 都有独特的猫咪角色
- 🦉 **Watchdog 守护** — 三重检测，自动恢复
- 📋 **可视化看板** — 实时监控 Agent 工作状态
- 🔄 **多 Adapter 支持** — Hermes、Claude Code、Codex
- 🧠 **L4 记忆系统** — 自动学习，技能结晶
- 🐛 **Bug = 老鼠** — 有趣的 Bug 分级系统

## 🏗️ 架构

```
┌─────────────────────────────────────────┐
│            用户层                        │
│   Dashboard (Vue 3)    CLI              │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│      CatCoding Daemon (Rust)            │
│  ┌─────────┐ ┌─────────┐ ┌──────────┐  │
│  │ Watchdog│ │Scheduler│ │  Router  │  │
│  └─────────┘ └─────────┘ └──────────┘  │
│  ┌─────────────────────────────────┐   │
│  │        Adapter 层               │   │
│  │  Hermes │ Claude │ Codex       │   │
│  └─────────────────────────────────┘   │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│         Python Agent SDK                │
│  PM(暹罗猫) │ Dev(英短) │ Review(玄猫)  │
└─────────────────────────────────────────┘
```

## 🚀 快速开始

### 安装

```bash
# 一键安装（即将支持）
curl -fsSL https://catcoding.org/install.sh | bash

# 或从源码编译
git clone https://github.com/catcoding-dev/catcoding.git
cd catcoding
cargo build --release
```

### 初始化项目

```bash
# 在项目目录中初始化
cd your-project
catcoding init

# 这会创建 .agent.yaml 配置文件
```

### 启动 Daemon

```bash
# 启动守护进程
catcoding serve

# Daemon 会启动：
# - HTTP API (默认: 127.0.0.1:9527)
# - Dashboard (默认: 0.0.0.0:8080)
```

### 访问 Dashboard

打开浏览器访问 `http://localhost:8080/dashboard`

## 🐱 猫咪团队

| 角色 | 猫咪 | 职责 |
|------|------|------|
| PM | 🐱 暹罗猫 | 需求分析、任务拆分、进度监控 |
| 核心开发 | 🐱 英短蓝猫 | 代码实现 |
| 代码审查 | 🐱 玄猫 | 代码审查、Bug 检测 |
| 测试 | 🐱 阿比西尼亚猫 | 测试用例、质量保障 |
| 技术侦察 | 🦊 狐狸 | 技术调研、文档搜集 |

## 🐛 Bug 分级（老鼠系统）

- 🐭 **小老鼠** — 简单 bug（拼写错误、语法问题）
- 🐀 **大老鼠** — 顽固 bug（逻辑错误、边界情况）
- 🦇 **蝙蝠** — 异常刁钻的 bug（时序问题、竞态条件）
- 🐉 **恶龙** — 架构级 bug（系统性问题，需要重构）

## 📁 配置文件

在项目根目录创建 `.agent.yaml`：

```yaml
project:
  name: "my-project"

agents:
  pm:
    enabled: true
    adapter: "hermes"
  core_dev:
    enabled: true
    adapter: "claude-code"

watchdog:
  heartbeat_timeout: 30
  max_restarts: 3
```

## 🔌 支持的 Adapter

| Adapter | 状态 | 说明 |
|---------|------|------|
| Hermes | ✅ 已实现 | 基于 hermes-agent |
| Claude Code | ✅ 已实现 | 基于 claude CLI |
| Codex | ✅ 已实现 | 基于 codex CLI |
| OpenCode | 🔜 计划中 | - |

## 🛠️ 开发

### 构建

```bash
# Debug 构建
cargo build

# Release 构建
cargo build --release

# 运行测试
cargo test
```

### 项目结构

```
catcoding/
├── daemon/           # Rust Daemon 核心
│   └── src/
│       ├── api/      # HTTP API
│       ├── adapter/  # Agent Adapter
│       ├── watchdog.rs
│       └── scheduler.rs
├── cli/              # CLI 工具
├── agents/           # Python Agent SDK
│   ├── base/         # 基础 Agent
│   ├── pm/           # PM Agent
│   └── reviewer/     # Review Agent
├── dashboard/        # Vue 3 前端
└── config/           # 配置文件
```

## 📊 Dashboard 功能

- 📋 **看板视图** — 任务状态一目了然
- 📊 **甘特图** — 时间线视图
- 🐱 **猫咪面板** — Agent 状态监控
- 🍳 **厨房日志** — 实时日志流
- 💬 **指令输入** — 下发指令

## 🎮 彩蛋系统

连续完成任务、抓到 Bug、项目完成时会触发特殊彩蛋！

- 🐯 **老虎模式** — 连续 10 个任务成功
- 🐱⬛ **传奇奶牛猫** — 抓到 100 个 Bug
- 🐼 **大熊猫庆祝** — 项目 100% 完成

## 📝 License

MIT License

## 🔗 链接

- 🌐 官网: [catcoding.org](https://catcoding.org)（即将上线）
- 📦 GitHub: [github.com/catcoding-dev/catcoding](https://github.com/catcoding-dev/catcoding)

---

**CatCoding** — 让 AI 像猫咪团队一样协作做菜 🐱🍳
