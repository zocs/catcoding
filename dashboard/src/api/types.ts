/**
 * CatCoding Dashboard — TypeScript 类型定义
 * 供 Gemini 3 Flash 生成 UI/UX 时直接引用
 */

// ━━━ API 响应类型 ━━━

export interface ApiResponse<T> {
  data?: T
  error?: string
  message?: string
}

export interface ServiceInfo {
  name: string
  version: string
  motto: string
  project: string
  agents: number
  tasks: number
  endpoints: {
    health: string
    projects: string
    agents: string
    tasks: string
    command: string
    dashboard: string
  }
}

export interface HealthStatus {
  status: 'ok' | 'degraded' | 'down'
  version: string
  project: string
  uptime: string
}

export interface WatchdogStatus {
  healthy: boolean
  checks: {
    nats: { status: string; latency_ms: number }
    agents: { status: string; active: number }
    disk: { status: string; usage_pct: number }
  }
  uptime_seconds: number
}

// ━━━ 数据模型 ━━━

/** 任务状态 — 8 态状态机 */
export type TaskStatus =
  | 'pending'      // 等待调度
  | 'blocked'      // 被依赖阻塞
  | 'ready'        // 就绪，可执行
  | 'active'       // 执行中
  | 'reviewing'    // 审查中
  | 'done'         // 完成
  | 'rollbacked'   // 已回滚
  | 'failed'       // 失败

export interface Task {
  id: string
  title: string
  description: string
  status: TaskStatus
  assigned_to: string | null  // Agent 角色名
  depends_on: string[]        // 依赖的 Task ID
  created_at: string          // ISO 8601
  updated_at: string
  artifacts: string[]         // 产出文件路径
}

/** Agent 状态 */
export type AgentStatus = 'idle' | 'active' | 'busy' | 'error'

/** Agent 运行模式 */
export type AgentMode = 'resident' | 'on_demand' | 'decorative'

export interface Agent {
  role: string          // 唯一标识: pm, core_dev, frontend, ...
  name: string          // 显示名: 暹罗猫, 英短蓝猫, ...
  emoji: string         // 头像: 🐱, 🦉, 🦊, 🐼
  status: AgentStatus
  description: string
  mode: AgentMode
  current_task: string | null
}

/** 项目 */
export interface Project {
  id: string
  name: string
  tasks: Task[]
  agents: Agent[]
}

/** 创建任务请求 */
export interface CreateTaskRequest {
  title: string
  description?: string
  role?: string
}

/** 更新状态请求 */
export interface UpdateStatusRequest {
  status: TaskStatus
}

/** 命令请求 */
export interface CommandRequest {
  command: string
  args?: string[]
}

// ━━━ Agent 角色常量 ━━━

export const AGENT_ROLES = {
  pm:         { emoji: '🐱', name: '暹罗猫',       mode: 'resident'   },
  core_dev:   { emoji: '🐱', name: '英短蓝猫',     mode: 'on_demand'  },
  frontend:   { emoji: '🐱', name: '橘猫',         mode: 'on_demand'  },
  backend:    { emoji: '🐱', name: '缅因猫',       mode: 'on_demand'  },
  reviewer:   { emoji: '🐱', name: '玄猫',         mode: 'resident'   },
  tester:     { emoji: '🐱', name: '阿比西尼亚猫', mode: 'on_demand'  },
  deploy:     { emoji: '🐱', name: '狸花猫',       mode: 'on_demand'  },
  watchdog:   { emoji: '🦉', name: '猫头鹰',       mode: 'resident'   },
  tech_scout: { emoji: '🦊', name: '狐狸',         mode: 'resident'   },
  mascot:     { emoji: '🐼', name: '大熊猫',       mode: 'decorative' },
} as const

export type AgentRole = keyof typeof AGENT_ROLES

// 带描述的完整角色定义
export const AGENT_ROLES_FIXED = {
  pm:         { emoji: '🐱', name: '暹罗猫',       mode: 'resident',   desc: '负责需求分析、任务拆分、进度汇报与全局决策。' },
  core_dev:   { emoji: '🐱', name: '英短蓝猫',     mode: 'on_demand',  desc: '稳重可靠，负责项目核心逻辑与系统架构的开发。' },
  frontend:   { emoji: '🐱', name: '橘猫',         mode: 'on_demand',  desc: '亲和力强，专注于用户界面实现与交互体验优化。' },
  backend:    { emoji: '🐱', name: '缅因猫',       mode: 'on_demand',  desc: '力量强大，负责高性能后端服务、API 与数据库设计。' },
  reviewer:   { emoji: '🐱', name: '玄猫',         mode: 'resident',   desc: '目光敏锐，负责代码审查，专门负责"抓老鼠"。' },
  tester:     { emoji: '🐱', name: '阿比西尼亚猫', mode: 'on_demand',  desc: '充满好奇心，负责编写测试用例与自动化测试流程。' },
  deploy:     { emoji: '🐱', name: '狸花猫',       mode: 'on_demand',  desc: '适应力极强，负责 CI/CD 流程与云端部署。' },
  watchdog:   { emoji: '🦉', name: '猫头鹰',       mode: 'resident',   desc: '永不睡眠，负责守护进程监控与系统自动恢复。' },
  tech_scout: { emoji: '🦊', name: '狐狸',         mode: 'resident',   desc: '技术侦察兵，负责文档搜索、趋势追踪与方案对比。' },
  mascot:     { emoji: '🐼', name: '大熊猫',       mode: 'decorative', desc: '品牌大使，负责卖萌和提升团队士气。' },
} as const

// ━━━ 状态 → 显示映射 ━━━

export const STATUS_CONFIG: Record<TaskStatus, { label: string; emoji: string; color: string; kanban: string }> = {
  pending:    { label: '待办',  emoji: '📋', color: '#909399', kanban: 'todo'     },
  blocked:    { label: '阻塞',  emoji: '🚫', color: '#e6a23c', kanban: 'todo'     },
  ready:      { label: '就绪',  emoji: '🔵', color: '#409eff', kanban: 'ready'    },
  active:     { label: '进行中', emoji: '🔨', color: '#f5a623', kanban: 'doing'    },
  reviewing:  { label: '审查中', emoji: '🔍', color: '#e6a23c', kanban: 'doing'    },
  done:       { label: '完成',  emoji: '✅', color: '#67c23a', kanban: 'done'     },
  rollbacked: { label: '回滚',  emoji: '⏪', color: '#909399', kanban: 'done'     },
  failed:     { label: '失败',  emoji: '❌', color: '#f56c6c', kanban: 'todo'     },
}

/** 看板列定义 */
export const KANBAN_COLUMNS = [
  { key: 'todo',   title: '待办',   emoji: '📋', statuses: ['pending', 'blocked', 'failed'] as TaskStatus[] },
  { key: 'ready',  title: '就绪',   emoji: '🔵', statuses: ['ready'] as TaskStatus[] },
  { key: 'doing',  title: '进行中', emoji: '🔨', statuses: ['active', 'reviewing'] as TaskStatus[] },
  { key: 'done',   title: '已完成', emoji: '✅', statuses: ['done', 'rollbacked'] as TaskStatus[] },
] as const

// ━━━ API 客户端 ━━━

export class CatCodingApi {
  // 自动检测：如果在 daemon 嵌入的 dashboard 内，用当前 origin；否则用配置的地址
  private baseUrl: string

  constructor(baseUrl?: string) {
    if (baseUrl) {
      this.baseUrl = baseUrl
    } else if (typeof window !== 'undefined' && window.location) {
      // 在浏览器中 — 用当前 origin（daemon 嵌入 dashboard 时）
      this.baseUrl = window.location.origin
    } else {
      this.baseUrl = 'http://127.0.0.1:19800'
    }
  }

  async health(): Promise<HealthStatus> {
    return fetch(`${this.baseUrl}/api/health`).then(r => r.json())
  }

  async getTasks(): Promise<Task[]> {
    return fetch(`${this.baseUrl}/api/tasks`).then(r => r.json()).then(d => d.tasks)
  }

  async createTask(req: CreateTaskRequest): Promise<{ id: string; status: string }> {
    return fetch(`${this.baseUrl}/api/tasks`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(req),
    }).then(r => r.json())
  }

  async updateTaskStatus(id: string, status: TaskStatus): Promise<void> {
    await fetch(`${this.baseUrl}/api/tasks/${id}/status`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ status }),
    })
  }

  async getAgents(): Promise<Agent[]> {
    return fetch(`${this.baseUrl}/api/agents`).then(r => r.json()).then(d => d.agents)
  }

  async getWatchdog(): Promise<WatchdogStatus> {
    return fetch(`${this.baseUrl}/api/watchdog`).then(r => r.json())
  }

  async sendCommand(command: string, args?: string[]): Promise<void> {
    await fetch(`${this.baseUrl}/api/command`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ command, args }),
    })
  }
}
