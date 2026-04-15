# 快速开始

## 环境要求

- **Rust** 1.75+ ([安装](https://rustup.rs))
- **Python** 3.10+（Agent SDK）
- **Node.js** 18+（Dashboard）
- **NATS Server**（自动启动或[手动安装](https://docs.nats.io/running-a-nats-service/introduction)）

## 安装

### 从源码构建

```bash
git clone https://github.com/zocs/catcoding.git
cd catcoding
cargo build --release

# 编译产物:
# target/release/catcoding-daemon  — 守护进程
# target/release/catcoding         — CLI 工具
```

### 一键安装（即将推出）

```bash
curl -fsSL https://catcoding.org/install.sh | bash
```

## 三步启动

### 1. 初始化项目

```bash
cd your-project
catcoding init
```

这会创建 `.agent.yaml` 配置文件：

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

### 2. 启动守护进程

```bash
catcoding serve
```

启动后：
- **HTTP API** → `http://127.0.0.1:9527`
- **Dashboard** → `http://localhost:8080`

### 3. 打开 Dashboard

浏览器访问 `http://localhost:8080`，你将看到：

- 🏠 **总览** — 任务统计、猫咪状态、系统健康
- 📋 **看板** — 可拖拽的任务看板（支持创建、移动、删除）
- 🐱 **猫咪面板** — 10 只 AI 猫咪实时状态
- 📊 **甘特图** — 任务时间线可视化
- 🐛 **命令终端** — 直接指挥猫咪团队

## 配置详解

### 适配器

| 适配器 | 配置键 | 说明 |
|--------|--------|------|
| Hermes | `hermes` | 需要安装 hermes-agent |
| Claude Code | `claude-code` | 需要安装 `claude` CLI |
| Codex | `codex` | 需要安装 `codex` CLI |

### Watchdog 设置

```yaml
watchdog:
  heartbeat_timeout: 30    # Agent 无响应超时（秒）
  max_restarts: 3           # 最大重启次数
  compile_check: true       # 验证代码是否可编译
  api_call_tracking: true   # 追踪 API 调用耗时
```

### Agent 角色

| 角色 | 猫咪 | 模式 | 职责 |
|------|------|------|------|
| `pm` | 暹罗猫 🐱 | 常驻 | 需求分析、任务拆分、全局决策 |
| `core_dev` | 英短蓝猫 🐱 | 按需 | 核心逻辑与架构开发 |
| `frontend` | 橘猫 🐱 | 按需 | UI 实现与交互优化 |
| `backend` | 缅因猫 🐱 | 按需 | 后端服务、API、数据库 |
| `reviewer` | 玄猫 🐱 | 常驻 | 代码审查、"抓老鼠" |
| `tester` | 阿比西尼亚猫 🐱 | 按需 | 测试用例与自动化 |
| `deploy` | 狸花猫 🐱 | 按需 | CI/CD 与部署 |
| `watchdog` | 猫头鹰 🦉 | 常驻 | 进程监控与自动恢复 |
| `tech_scout` | 狐狸 🦊 | 常驻 | 文档搜索、趋势追踪 |
| `mascot` | 大熊猫 🐱 | 吉祥物 | 卖萌提升团队士气 |

## 下一步

- [架构概览](https://github.com/zocs/catcoding#-architecture)
- [配置参考](../config/agent.yaml.example)
- [Dashboard 使用指南](https://demo.catcoding.org)
- [贡献指南](https://github.com/zocs/catcoding/blob/main/docs/contributing)
- [English](../docs/guide/getting-started.md)
