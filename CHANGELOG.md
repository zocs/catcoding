# Changelog

所有重要的更改都会记录在此文件中。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

## [0.1.0] - 2026-04-15

### 🎉 首次发布

#### ✨ 新功能

**核心框架 (Rust Daemon)**
- 🦉 Watchdog 三重检测系统（管道EOF、心跳超时、/proc轮询）
- 📋 Scheduler 任务调度器（依赖门控）
- 🔄 NATS 消息路由
- 💾 SQLite 持久化存储
- 🧠 L4 记忆系统（事实、技能、会话、索引）
- 📊 状态管理器（8态状态机）
- ⏪ Rollback 管理器
- 🔗 级联依赖处理

**Adapter 系统**
- 🐱 Hermes Adapter — 支持 hermes-agent
- 🎭 Claude Code Adapter — 支持 claude CLI
- 🤖 Codex Adapter — 支持 codex CLI

**Python Agent SDK**
- 🐱 PM Agent（暹罗猫）— 需求分析、任务拆分
- 🐱 Review Agent（玄猫）— 代码审查、Bug 分级
- 🐱 Test Agent（阿比西尼亚猫）— 自动测试
- 🐱 Tech Scout（狐狸）— 技术调研

**Dashboard (Vue 3)**
- 📋 看板视图 — 任务状态管理
- 📊 甘特图 — 时间线视图
- 🐱 猫咪面板 — Agent 状态监控
- 🍳 厨房日志 — 实时日志流
- 💬 指令输入 — 下发指令
- 🎨 深色主题 — Naive UI

**品牌系统**
- 🐱 10种猫咪角色形象
- 🐛 Bug 分级系统（老鼠系统）
- 🎮 彩蛋系统（老虎、传奇奶牛猫、九命猫等）

**CLI 工具**
- `catcoding init` — 初始化项目
- `catcoding serve` — 启动 Daemon
- `catcoding status` — 查看状态
- `catcoding logs` — 查看日志
- `catcoding command` — 发送指令

#### 📦 安装方式

- 🍺 Homebrew: `brew install catcoding`
- 📥 一键安装: `curl -fsSL https://catcoding.org/install.sh | bash`
- 🔧 源码编译: `cargo build --release`

#### 📝 文档

- 完整的 README.md
- 配置文件示例 (agent.yaml.example)
- Homebrew Formula

---

## [计划中]

### 0.2.0 - 多 Agent 协作增强

- [ ] WebSocket 实时通信
- [ ] Agent 间协作协议
- [ ] 更智能的 PM Agent（LLM 集成）
- [ ] 技术侦察兵实际搜索功能

### 0.3.0 - 生产就绪

- [ ] API 认证
- [ ] 限流保护
- [ ] 更多 Adapter（OpenCode、Superpowers）
- [ ] Kubernetes 部署支持

### 1.0.0 - 稳定版

- [ ] 完整的测试覆盖
- [ ] 性能优化
- [ ] 社区插件系统
- [ ] 多语言支持

---

**CatCoding** — 让 AI 像猫咪团队一样协作做菜 🐱🍳
