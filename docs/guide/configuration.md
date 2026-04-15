# 配置参考

## `.agent.yaml` 完整配置

```yaml
# ═══ 项目配置 ═══
project:
  name: "my-project"          # 项目名称
  root: "."                   # 项目根目录（相对于 .agent.yaml）
  language: "auto"            # auto | rust | python | typescript | go

# ═══ Agent 配置 ═══
agents:
  # 常驻 Agent — 始终在线
  pm:
    enabled: true
    adapter: "hermes"         # hermes | claude-code | codex
    model: "claude-opus-4.6"  # 可选：指定模型
    priority: 1               # 调度优先级

  reviewer:
    enabled: true
    adapter: "hermes"

  watchdog:
    enabled: true
    adapter: "hermes"

  tech_scout:
    enabled: true
    adapter: "hermes"

  # 按需 Agent — 任务触发
  core_dev:
    enabled: true
    adapter: "claude-code"

  frontend:
    enabled: true
    adapter: "claude-code"

  backend:
    enabled: true
    adapter: "claude-code"

  tester:
    enabled: true
    adapter: "codex"

  deploy:
    enabled: true
    adapter: "claude-code"

  # 吉祥物
  mascot:
    enabled: true             # 设为 false 可关闭吉祥物

# ═══ Watchdog 配置 ═══
watchdog:
  heartbeat_timeout: 30       # Agent 心跳超时（秒）
  max_restarts: 3             # 单个 Agent 最大重启次数
  restart_cooldown: 10        # 重启冷却时间（秒）
  compile_check: true         # 编译检查
  api_call_tracking: true     # API 调用追踪
  proc_polling: true          # /proc 轮询监控

# ═══ 消息总线 ═══
nats:
  url: "nats://127.0.0.1:4222"  # NATS 连接地址
  jetstream: true                # 启用 JetStream 持久化

# ═══ 调度器 ═══
scheduler:
  strategy: "dependency"      # dependency | round-robin | priority
  max_concurrent: 3           # 最大并行任务数
  task_timeout: 3600          # 单任务超时（秒）

# ═══ Dashboard ═══
dashboard:
  port: 8080                  # Dashboard 端口
  host: "0.0.0.0"             # 监听地址
  cors: true                  # 允许跨域

# ═══ 数据库 ═══
database:
  path: ".catcoding/state.db" # SQLite 数据库路径
  wal_mode: true              # WAL 模式（提升并发性能）

# ═══ 日志 ═══
logging:
  level: "info"               # debug | info | warn | error
  file: ".catcoding/catcoding.log"
  max_size_mb: 100            # 日志文件最大大小
```

## 环境变量

| 变量 | 说明 | 必需 |
|------|------|------|
| `ANTHROPIC_API_KEY` | Claude API Key | 使用 claude-code 时 |
| `OPENAI_API_KEY` | OpenAI API Key | 使用 codex 时 |
| `NATS_URL` | NATS 地址（覆盖配置） | 否 |
| `CATCODING_LOG_LEVEL` | 日志级别（覆盖配置） | 否 |
| `CATCODING_DB_PATH` | 数据库路径（覆盖配置） | 否 |

## 命令行参考

```bash
# 初始化项目
catcoding init [--template basic|full]

# 启动守护进程
catcoding serve [--port 8080] [--config .agent.yaml]

# 查看状态
catcoding status

# 查看 Agent 列表
catcoding agents list

# 手动触发任务
catcoding task create "实现用户登录功能"

# 查看日志
catcoding logs [--follow] [--agent pm]
```
