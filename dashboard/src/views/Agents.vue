<template>
  <div class="agents">
    <n-page-header title="🐱 猫咪面板" subtitle="Agent 状态监控">
      <template #extra>
        <n-button @click="refresh">
          <template #icon><span>🔄</span></template>
          刷新
        </n-button>
      </template>
    </n-page-header>

    <!-- Agent 卡片 -->
    <div class="agent-grid">
      <n-card v-for="agent in agents" :key="agent.role" class="agent-card" hoverable>
        <template #cover>
          <div class="agent-avatar" :class="agent.status">
            <span class="avatar-emoji">{{ agent.emoji }}</span>
            <span class="status-dot" :class="agent.status"></span>
          </div>
        </template>
        <n-thing>
          <template #header>
            <span class="agent-name">{{ agent.name }}</span>
          </template>
          <template #description>
            <span class="agent-role">{{ agent.role }}</span>
          </template>
          {{ agent.description }}
        </n-thing>
        <template #footer>
          <n-space justify="space-between">
            <n-tag size="small" :type="getStatusType(agent.status)">
              {{ getStatusText(agent.status) }}
            </n-tag>
            <span v-if="agent.current_task" class="current-task">
              🔧 {{ agent.current_task }}
            </span>
          </n-space>
        </template>
      </n-card>
    </div>

    <!-- Watchdog 状态 -->
    <n-card title="🦉 猫头鹰 Watchdog" style="margin-top: 24px">
      <n-descriptions bordered>
        <n-descriptions-item label="状态">
          <n-tag type="success">运行中</n-tag>
        </n-descriptions-item>
        <n-descriptions-item label="心跳间隔">
          {{ watchdogConfig.heartbeat_interval }}s
        </n-descriptions-item>
        <n-descriptions-item label="超时阈值">
          {{ watchdogConfig.heartbeat_timeout }}s
        </n-descriptions-item>
        <n-descriptions-item label="最大重启">
          {{ watchdogConfig.max_restart }} 次
        </n-descriptions-item>
      </n-descriptions>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useMessage } from 'naive-ui'

interface Agent {
  role: string
  name: string
  emoji: string
  description: string
  status: string
  current_task: string | null
}

const message = useMessage()

const agents = ref<Agent[]>([
  { role: 'pm', name: '暹罗猫', emoji: '🐱', description: '聪明、爱指挥、话多 → 全局观、调度、汇报', status: 'idle', current_task: null },
  { role: 'core_dev', name: '英短蓝猫', emoji: '🐱', description: '沉稳、可靠、高效 → 核心功能开发', status: 'idle', current_task: null },
  { role: 'frontend', name: '橘猫', emoji: '🐱', description: '温暖、亲和力强 → 用户界面实现', status: 'idle', current_task: null },
  { role: 'backend', name: '缅因猫', emoji: '🐱', description: '体型大、力量强 → 后端、API', status: 'idle', current_task: null },
  { role: 'reviewer', name: '玄猫', emoji: '🐱', description: '神秘、敏锐 → 找 bug（抓老鼠）', status: 'idle', current_task: null },
  { role: 'tester', name: '阿比西尼亚猫', emoji: '🐱', description: '好奇心强、爱探索 → 测试用例编写', status: 'idle', current_task: null },
  { role: 'watchdog', name: '猫头鹰', emoji: '🦉', description: '夜行、警觉、永远不睡觉 = 完美的守护进程', status: 'active', current_task: null },
  { role: 'tech_scout', name: '狐狸', emoji: '🦊', description: '聪明、敏捷、善于发现 = 技术侦察兵', status: 'idle', current_task: null },
  { role: 'mascot', name: '大熊猫', emoji: '🐼', description: '不干活，只负责可爱和 logo', status: 'decorative', current_task: null },
])

const watchdogConfig = ref({
  heartbeat_interval: 5,
  heartbeat_timeout: 15,
  max_restart: 3,
})

function getStatusType(status: string) {
  const types: Record<string, string> = {
    working: 'info',
    active: 'success',
    idle: 'default',
    error: 'error',
    restarting: 'warning',
    decorative: 'default',
  }
  return (types[status] || 'default') as any
}

function getStatusText(status: string) {
  const texts: Record<string, string> = {
    working: '🔵 工作中',
    active: '🟢 运行中',
    idle: '💤 空闲',
    error: '❌ 错误',
    restarting: '🔄 重启中',
    decorative: '🐼 装饰',
  }
  return texts[status] || '❓ 未知'
}

async function refresh() {
  try {
    const res = await fetch('/api/watchdog')
    const data = await res.json()
    watchdogConfig.value = data.config || watchdogConfig.value
    message.success('刷新成功')
  } catch (e) {
    message.error('刷新失败')
  }
}

onMounted(() => {
  refresh()
})
</script>

<style scoped>
.agents {
  padding: 16px;
}

.agent-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 16px;
  margin-top: 24px;
}

.agent-card {
  text-align: center;
}

.agent-avatar {
  padding: 24px;
  background: linear-gradient(135deg, #1e1e2e 0%, #2d2d44 100%);
  position: relative;
}

.avatar-emoji {
  font-size: 48px;
}

.status-dot {
  position: absolute;
  bottom: 8px;
  right: 8px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 2px solid #1e1e2e;
}

.status-dot.working, .status-dot.active { background: #2ecc71; }
.status-dot.idle { background: #666; }
.status-dot.error { background: #e74c3c; }
.status-dot.restarting { background: #f1c40f; }
.status-dot.decorative { background: #9b59b6; }

.agent-name {
  font-weight: bold;
  font-size: 16px;
}

.agent-role {
  color: #888;
  font-size: 12px;
}

.current-task {
  font-size: 12px;
  color: #888;
}
</style>
