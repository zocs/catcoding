# CatCoding UI/UX 规范文档

> 为 Gemini 3 Flash UI/UX 生成提供完整 API 接口、数据模型和设计约束
> 版本: 0.1.0 | 更新: 2026-04-15

---

## 1. 技术栈

| 层 | 技术 | 备注 |
|---|------|------|
| 前端框架 | Vue 3.4 + TypeScript 5.3 | Composition API |
| UI 组件库 | **Naive UI 2.38** | 必须使用 naive-ui 组件 |
| 状态管理 | Pinia 2.1 | 全局状态 |
| 图表 | ECharts 5.5 + vue-echarts | 甘特图/监控 |
| 路由 | Vue Router 4.3 | History mode |
| 构建 | Vite 5 | |
| 后端 | Rust (Axum 0.7) | REST API |
| 消息总线 | NATS JetStream | Agent 通信 |

---

## 2. 路由结构

```
/           → 重定向到 /board
/board      → 任务看板 (Kanban)    ← 核心页面
/gantt      → 甘特图 (Gantt)
/agents     → Agent 状态面板
/logs       → 实时日志流
/command    → 命令终端
```

---

## 3. API 接口规范

### 3.1 基础信息

```
Base URL: http://localhost:9800
Content-Type: application/json
```

### 3.2 端点列表

#### GET `/` — 服务信息
```json
{
  "name": "CatCoding Daemon",
  "version": "0.1.0",
  "motto": "🐱 让 AI 像猫咪团队一样协作做菜！",
  "project": "my-project",
  "agents": 8,
  "tasks": 12,
  "endpoints": {
    "health": "/api/health",
    "projects": "/api/projects",
    "agents": "/api/agents",
    "tasks": "/api/tasks",
    "command": "/api/command",
    "dashboard": "/dashboard"
  }
}
```

#### GET `/api/health` — 健康检查
```json
{
  "status": "ok",
  "version": "0.1.0",
  "project": "my-project",
  "uptime": "2h 34m"
}
```

#### GET `/api/projects` — 列出项目
```json
{
  "projects": [
    {
      "id": "proj-uuid",
      "name": "My App",
      "task_count": 12,
      "agent_count": 8
    }
  ]
}
```

#### GET `/api/projects/{id}` — 项目详情
```json
{
  "id": "proj-uuid",
  "name": "My App",
  "tasks": [/* Task 对象数组 */],
  "agents": [/* Agent 对象数组 */]
}
```

#### GET `/api/agents` — 列出所有 Agent
```json
{
  "agents": [
    {
      "role": "pm",
      "name": "暹罗猫",
      "emoji": "🐱",
      "status": "active",
      "description": "聪明、爱指挥、话多 → 全局观、调度、汇报",
      "mode": "resident",
      "current_task": "task-uuid-123"
    }
  ]
}
```

#### GET `/api/tasks` — 列出所有任务
```json
{
  "tasks": [
    {
      "id": "task-uuid",
      "title": "实现用户登录",
      "description": "JWT 认证 + OAuth2",
      "status": "active",
      "assigned_to": "core_dev",
      "depends_on": ["task-uuid-001"],
      "created_at": "2026-04-15T00:40:00Z",
      "updated_at": "2026-04-15T01:23:00Z",
      "artifacts": ["src/auth.rs", "src/middleware.rs"]
    }
  ]
}
```

#### POST `/api/tasks` — 创建任务
请求:
```json
{
  "title": "实现用户登录",
  "description": "JWT 认证 + OAuth2",
  "role": "core_dev"
}
```
响应 (201):
```json
{
  "id": "new-task-uuid",
  "title": "实现用户登录",
  "status": "pending",
  "message": "任务已创建，等待调度..."
}
```

#### POST `/api/tasks/{id}/status` — 更新任务状态
请求:
```json
{
  "status": "active"  // pending|blocked|ready|active|reviewing|done|rollbacked|failed
}
```
响应:
```json
{
  "id": "task-uuid",
  "status": "active",
  "message": "状态已更新"
}
```

#### POST `/api/command` — 执行命令
请求:
```json
{
  "command": "deploy",
  "args": ["--env", "staging"]
}
```

#### GET `/api/watchdog` — Watchdog 状态
```json
{
  "healthy": true,
  "checks": {
    "nats": { "status": "ok", "latency_ms": 2 },
    "agents": { "status": "ok", "active": 3 },
    "disk": { "status": "ok", "usage_pct": 45 }
  },
  "uptime_seconds": 9240
}
```

---

## 4. 数据模型

### 4.1 Task（任务）— 8 态状态机

```
字段            类型              说明
──────────────────────────────────────────────────
id              string            UUID
title           string            任务标题
description     string            任务描述
status          TaskStatus        状态 (见下)
assigned_to     string?           分配的 Agent 角色
depends_on      string[]          依赖的任务 ID
created_at      DateTime          ISO 8601
updated_at      DateTime          ISO 8601
artifacts       string[]          产出的文件列表
```

**TaskStatus 状态机流转**:
```
                    ┌──────────┐
                    │ pending  │ ← 初始状态
                    └────┬─────┘
                         │ 调度
                    ┌────▼─────┐
              ┌─────│  ready   │─────┐
              │     └──────────┘     │
              │ 有依赖               │ 无依赖
         ┌────▼─────┐          ┌────▼─────┐
         │ blocked  │          │  active  │
         └──────────┘          └────┬─────┘
                                    │ 完成
                               ┌────▼─────┐
                               │reviewing │
                               └────┬─────┘
                              ┌─────┴─────┐
                         通过 │           │ 失败
                        ┌────▼─────┐ ┌───▼─────┐
                        │   done   │ │ failed  │
                        └──────────┘ └─────────┘
                         ┌──────────┐
                         │rollbacked│ ← 回滚状态
                         └──────────┘
```

### 4.2 Agent（代理）

```
字段              类型           说明
──────────────────────────────────────────────
role              string        角色标识 (pm, core_dev, ...)
name              string        显示名 (猫品种名)
emoji             string        头像 emoji
status            string        idle|active|busy|error
description       string        角色描述
mode              string        resident|on_demand|decorative
current_task      string?       当前执行的任务 ID
```

### 4.3 Agent 角色列表 (猫系皮肤)

| 角色 | Emoji | 猫名 | 模式 | 职责 |
|------|-------|------|------|------|
| pm | 🐱 | 暹罗猫 | resident | 全局观、调度、汇报 |
| core_dev | 🐱 | 英短蓝猫 | on_demand | 核心功能开发 |
| frontend | 🐱 | 橘猫 | on_demand | 用户界面实现 |
| backend | 🐱 | 缅因猫 | on_demand | 后端、API |
| reviewer | 🐱 | 玄猫 | resident | Code Review |
| tester | 🐱 | 阿比西尼亚猫 | on_demand | 测试用例编写 |
| deploy | 🐱 | 狸花猫 | on_demand | CI/CD、部署 |
| watchdog | 🦉 | 猫头鹰 | resident | 守护进程监控 |
| tech_scout | 🦊 | 狐狸 | resident | 技术侦察 |
| mascot | 🐼 | 大熊猫 | decorative | Logo/可爱 |

---

## 5. UI 设计约束

### 5.1 组件库 — 必须使用 Naive UI

```vue
<!-- 标准组件使用 -->
<n-page-header>    <!-- 页面标题 -->
<n-card>           <!-- 卡片容器 -->
<n-tag>            <!-- 标签 -->
<n-button>         <!-- 按钮 -->
<n-badge>          <!-- 徽章 -->
<n-modal>          <!-- 弹窗 -->
<n-form>           <!-- 表单 -->
<n-input>          <!-- 输入框 -->
<n-space>          <!-- 间距布局 -->
<n-thing>          <!-- 物品描述 -->
<n-data-table>     <!-- 数据表格 -->
<n-timeline>       <!-- 时间线 -->
<n-log>            <!-- 日志流 -->
<n-progress>       <!-- 进度条 -->
```

### 5.2 页面结构模板

每个页面遵循:
```vue
<template>
  <div class="page-name">
    <!-- 1. 页面头部 -->
    <n-page-header title="页面标题" subtitle="副标题">
      <template #extra><!-- 操作按钮 --></template>
    </n-page-header>

    <!-- 2. 主内容区 -->
    <!-- Kanban / Grid / Table / Timeline -->

    <!-- 3. 弹窗/抽屉 -->
    <n-modal v-model:show="visible">
      <!-- 表单/详情 -->
    </n-modal>
  </div>
</template>
```

### 5.3 设计 Token

```css
/* 主题色 — 猫系暖色调 */
--primary: #f5a623;      /* 琥珀金 — 主色 */
--success: #18a058;      /* 绿色 — done */
--warning: #f0a020;      /* 橙色 — active/reviewing */
--error: #d03050;        /* 红色 — failed */
--info: #2080f0;         /* 蓝色 — info */

/* 状态色 */
--status-pending:   #909399;  /* 灰色 */
--status-blocked:   #e6a23c;  /* 深黄 */
--status-ready:     #409eff;  /* 蓝色 */
--status-active:    #f5a623;  /* 琥珀 */
--status-reviewing: #e6a23c;  /* 橙色 */
--status-done:      #67c23a;  /* 绿色 */
--status-failed:    #f56c6c;  /* 红色 */
--status-rollbacked:#909399;  /* 灰色 */

/* 布局 */
--board-columns: 4~6;        /* 看板列数 */
--card-border-radius: 8px;
--page-max-width: 1440px;
--sidebar-width: 200px;
```

### 5.4 看板列定义

| 列 | Emoji | 状态映射 | 颜色 |
|----|-------|---------|------|
| 待办 | 📋 | pending, blocked | 灰 |
| 就绪 | 🔵 | ready | 蓝 |
| 进行中 | 🔨 | active, reviewing | 琥珀 |
| 已完成 | ✅ | done, rollbacked | 绿 |

### 5.5 Agent 面板布局

```
┌─────────────────────────────────────────┐
│ 🐱 猫咪面板          Agent 状态监控 [🔄] │
├─────────┬─────────┬─────────┬───────────┤
│ 🐱 暹罗猫│ 🐱 英短蓝│ 🐱 橘猫  │ 🐱 缅因猫  │
│ PM      │ 核心开发 │ 前端    │ 后端      │
│ ✅ idle │ 🔨 active│ ⏸ idle │ 🔨 active │
├─────────┼─────────┼─────────┼───────────┤
│ 🐱 玄猫  │ 🐱 阿比  │ 🐱 狸花猫│ 🦉 猫头鹰  │
│ 审查    │ 测试    │ 部署    │ 守护      │
│ ✅ idle │ ⏸ idle  │ ⏸ idle │ ✅ resident│
└─────────┴─────────┴─────────┴───────────┘
```

---

## 6. 文件输出规范

Gemini 生成的文件应为:

1. **`.vue` 单文件组件** — 标准 Vue 3 SFC 格式
2. **必须使用 Naive UI 组件** — `import { NCard, NButton } from 'naive-ui'`
3. **必须使用 Composition API** — `<script setup lang="ts">`
4. **使用 `defineProps` / `defineEmits`** 定义接口
5. **样式使用 `<style scoped>`**
6. **响应式使用 `ref` / `computed` / `watch`**

### 标准组件模板

```vue
<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { NCard, NButton, NTag, NBadge, useMessage } from 'naive-ui'

// Props
interface Props {
  apiUrl?: string
}
const props = withDefaults(defineProps<Props>(), {
  apiUrl: 'http://localhost:9800'
})

// Emits
const emit = defineEmits<{
  refresh: []
  select: [id: string]
}>()

// State
const loading = ref(false)
const data = ref([])

// API 调用
async function fetchData() {
  loading.value = true
  try {
    const res = await fetch(`${props.apiUrl}/api/tasks`)
    const json = await res.json()
    data.value = json.tasks
  } finally {
    loading.value = false
  }
}

onMounted(fetchData)
</script>

<template>
  <!-- 使用 Naive UI 组件 -->
  <n-card title="标题">
    <!-- 内容 -->
  </n-card>
</template>

<style scoped>
/* 使用设计 token */
.page { max-width: 1440px; margin: 0 auto; }
</style>
```

---

## 7. 现有页面参考

以下是已实现的页面结构，新设计应在这些基础上改进:

- **Board.vue** — 4列看板 (pending/ready/active/done)
- **Agents.vue** — 卡片网格展示 Agent 状态
- **Gantt.vue** — ECharts 甘特图
- **Logs.vue** — 实时日志流 (`n-log`)
- **Command.vue** — 命令输入终端

---

## 8. API 调用示例 (前端)

```typescript
// 获取任务列表
const tasks = await fetch('http://localhost:9800/api/tasks')
  .then(r => r.json())
  .then(d => d.tasks as Task[])

// 创建任务
await fetch('http://localhost:9800/api/tasks', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    title: '新功能',
    description: '描述',
    role: 'core_dev'
  })
})

// 更新状态
await fetch(`http://localhost:9800/api/tasks/${taskId}/status`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ status: 'active' })
})

// 获取 Agent 列表
const agents = await fetch('http://localhost:9800/api/agents')
  .then(r => r.json())
  .then(d => d.agents as Agent[])
```
