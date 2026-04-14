# 🐱 CatCoding

> 多 Agent 协同软件开发框架 — 让 AI 像猫咪团队一样协作做菜（写代码）

## 项目定位

CatCoding 是一个**独立于任何 AI agent 框架**的多 Agent 协同软件开发框架。

通过 Adapter 接口层支持多种 AI 编程 agent（Hermes Agent、Claude Code、Codex、OpenCode 等），将多个 AI agent 组织成高效协作的开发团队。

## 核心理念

1. **用户定义角色 → 框架自动组装** — 框架适配用户，不是用户适配框架
2. **约定优于配置**（Convention over Configuration）
3. **渐进式复杂度**（Progressive Disclosure）
4. **零配置启动**（Zero-config Start）
5. **框架与 AI agent 解耦** — 换 agent 不换框架

## 🐾 猫咪团队

| 猫咪 | 角色 | 职责 |
|------|------|------|
| 🐱 暹罗猫 | PM Agent | 需求分析、任务拆分、进度汇报 |
| 🐱 英短蓝猫 | Core Dev | 核心功能开发 |
| 🐱 橘猫 | Frontend | 前端开发、UI 实现 |
| 🐱 缅因猫 | Backend | 后端开发、API 实现 |
| 🐱 布偶猫 | UI/UX | 设计实现 |
| 🐱 玄猫 | Code Review | 代码审查、找 bug |
| 🐱 阿比西尼亚猫 | Test | 测试用例、自动化测试 |
| 🐱 狸花猫 | Deploy | CI/CD、部署 |
| 🦉 猫头鹰 | Watchdog | 进程监管（Daemon 内置） |
| 🦊 狐狸 | Tech Scout | 文档搜索、趋势追踪 |
| 🐼 大熊猫 | 吉祥物 | 品牌大使，不承担职责 |

## 品牌比喻

- **写代码 = 做菜** 🍳（厨房 = 项目代码库）
- **Bug = 老鼠** 🐭（藏在代码里的问题）
- **修复 Bug = 烹饪老鼠** （处理问题）
- **代码审查 = 品尝** （确认修复质量）

## 技术栈

| 层 | 技术 | 语言 |
|---|---|---|
| Daemon 核心 | tokio + axum | Rust |
| 消息总线 | NATS (JetStream) | Rust/Go |
| 状态存储 | NATS KV + SQLite | Rust |
| Agent 实现 | 各 Adapter 定义 | Python/任意 |
| 前端 Dashboard | Vue 3 + Vite + Naive UI + ECharts | TypeScript |
| CLI | clap | Rust |

## 快速开始

```bash
# 初始化项目
catcoding init

# 启动守护进程
catcoding serve

# 查看状态
catcoding status

# 向猫咪团队发送指令
catcoding command "帮我实现一个登录功能"
```

## 架构

```
┌──────────────────────────────────────────────────┐
│                   用户层                           │
│   浏览器 Dashboard (Vue 3)    CLI (catcoding)    │
└──────────────┼───────────────────────┼────────────┘
               │ HTTP/WebSocket        │ HTTP
┌──────────────┼───────────────────────┼────────────┐
│              ▼                       ▼             │
│     ┌─────────────────────────────────────┐       │
│     │      CatCoding Daemon (Rust)     │       │
│     │  ┌─────────┐ ┌──────────┐ ┌───────┐ │       │
│     │  │Watchdog │ │Scheduler │ │Router │ │       │
│     │  │猫头鹰   │ │暹罗猫PM  │ │消息路由│ │       │
│     │  └─────────┘ └──────────┘ └───────┘ │       │
│     │  ┌─────────┐ ┌──────────┐           │       │
│     │  │API Axum │ │StateManager│          │       │
│     │  │HTTP+WS  │ │状态持久化  │          │       │
│     │  └─────────┘ └──────────┘           │       │
│     └──────────────────┬──────────────────┘       │
│     ┌──────────────────┼──────────────────┐       │
│     │    Agent Adapter 接口层              │       │
│     │  ┌──────┐┌───────┐┌──────┐┌──────┐  │       │
│     │  │Hermes││Claude ││Open  ││Codex │  │       │
│     │  │Agent ││Code   ││Claw  ││      │  │       │
│     │  └──────┘└───────┘└──────┘└──────┘  │       │
│     └─────────────────────────────────────┘       │
│  ┌────────────────────────────────────────────┐   │
│  │              NATS 消息总线                   │   │
│  │  Streams · Pub/Sub · JetStream · KV Store  │   │
│  └────────────────────────────────────────────┘   │
└────────────────────────────────────────────────────┘
```

## 许可证

MIT
