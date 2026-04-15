<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { NCard, NStatistic, NSpace, NTag, NProgress, NList, NListItem, NThing, useMessage } from 'naive-ui'
import { Task, Agent, WatchdogStatus, CatCodingApi, STATUS_CONFIG } from '@/api/types'
import CatAvatarSVG from '@/components/CatAvatarSVG.vue'
import CatBackground from '@/components/CatBackground.vue'

const api = new CatCodingApi()
const message = useMessage()
const tasks = ref<Task[]>([])
const agents = ref<Agent[]>([])
const watchdog = ref<WatchdogStatus | null>(null)
const loading = ref(false)

async function fetchData() {
  loading.value = true
  try {
    const [t, a, w] = await Promise.all([api.getTasks(), api.getAgents(), api.getWatchdog()])
    tasks.value = t
    agents.value = a
    watchdog.value = w
  } catch {
    // Mock
    tasks.value = [
      { id: '1', title: '实现用户登录', description: 'JWT', status: 'active', assigned_to: 'core_dev', depends_on: [], created_at: '', updated_at: '', artifacts: [] },
      { id: '2', title: '看板优化', description: 'UI', status: 'done', assigned_to: 'frontend', depends_on: [], created_at: '', updated_at: '', artifacts: [] },
      { id: '3', title: 'API 文档', description: 'Docs', status: 'pending', assigned_to: null, depends_on: [], created_at: '', updated_at: '', artifacts: [] },
    ] as Task[]
    agents.value = [
      { role: 'pm', name: '暹罗猫', emoji: '🐱', status: 'active', description: '', mode: 'resident', current_task: '1' },
      { role: 'core_dev', name: '英短蓝猫', emoji: '🐱', status: 'active', description: '', mode: 'on_demand', current_task: '1' },
      { role: 'frontend', name: '橘猫', emoji: '🐱', status: 'idle', description: '', mode: 'on_demand', current_task: null },
    ] as Agent[]
    watchdog.value = { healthy: true, checks: { nats: { status: 'ok', latency_ms: 2 }, agents: { status: 'ok', active: 3 }, disk: { status: 'ok', usage_pct: 45 } }, uptime_seconds: 3600 }
  } finally {
    loading.value = false
  }
}

function statusCount(status: string) {
  return tasks.value.filter(t => t.status === status).length
}

function activeAgents() {
  return agents.value.filter(a => a.status === 'active').length
}

onMounted(fetchData)
</script>

<template>
  <div class="dashboard-page">
    <CatBackground />
    <div class="page-title">
      <span class="title-emoji">🏠</span>
      <span class="title-text">CatCoding 总览</span>
    </div>

    <!-- 统计卡片 — 响应式 4→2→1 列 -->
    <div class="stats-grid">
      <n-card class="stat-card">
        <n-statistic label="📋 总任务" :value="tasks.length" />
      </n-card>
      <n-card class="stat-card">
        <n-statistic label="🐱 活跃猫咪" :value="activeAgents()" />
      </n-card>
      <n-card class="stat-card">
        <n-statistic label="✅ 已完成" :value="statusCount('done')" />
      </n-card>
      <n-card class="stat-card">
        <n-statistic label="⏱️ 运行时间" :value="watchdog ? (watchdog.uptime_seconds / 3600).toFixed(1) + 'h' : '0h'" />
      </n-card>
    </div>

    <!-- 详情区 — 响应式 2→1 列 -->
    <div class="detail-grid">
      <!-- 任务分布 -->
      <n-card title="📊 任务状态分布" class="detail-card">
        <div v-for="(cfg, status) in STATUS_CONFIG" :key="status" class="status-bar">
          <div class="status-label">
            <span>{{ cfg.emoji }} {{ cfg.label }}</span>
            <span class="status-count">{{ statusCount(status) }}</span>
          </div>
          <n-progress
            type="line"
            :percentage="tasks.length > 0 ? (statusCount(status) / tasks.length) * 100 : 0"
            :color="cfg.color"
            :show-indicator="false"
            :height="8"
          />
        </div>
      </n-card>

      <!-- 猫咪状态 -->
      <n-card title="🐱 活跃猫咪" class="detail-card">
        <div class="agent-mini-list">
          <div v-for="agent in agents.slice(0, 6)" :key="agent.role" class="agent-mini">
            <CatAvatarSVG :role="agent.role" :status="agent.status" :size="40" />
            <div class="agent-mini-info">
              <span class="agent-mini-name">{{ agent.name }}</span>
              <span class="agent-mini-task" v-if="agent.current_task">
                🔧 #{{ agent.current_task }}
              </span>
            </div>
          </div>
        </div>
      </n-card>
    </div>

    <!-- Watchdog -->
    <n-card v-if="watchdog" title="🦉 系统健康" class="watchdog-card">
      <n-space align="center" justify="space-between">
        <n-tag :type="watchdog.healthy ? 'success' : 'error'" round size="large">
          {{ watchdog.healthy ? '✅ 健康' : '❌ 异常' }}
        </n-tag>
        <n-space>
          <n-tag size="small">NATS: {{ watchdog.checks.nats.latency_ms }}ms</n-tag>
          <n-tag size="small">磁盘: {{ watchdog.checks.disk.usage_pct }}%</n-tag>
        </n-space>
      </n-space>
    </n-card>
  </div>
</template>

<style scoped>
.dashboard-page {
  max-width: 1200px;
  margin: 0 auto;
  position: relative;
  z-index: 1;
}

.page-title {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 24px;
}

.title-emoji {
  font-size: 28px;
}

.title-text {
  font-size: 22px;
  font-weight: bold;
}

/* ═══ 统计网格 ═══ */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 16px;
  margin-bottom: 20px;
}

.stat-card {
  border-radius: 16px;
  text-align: center;
}

@media (max-width: 768px) {
  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 10px;
  }
}

@media (max-width: 480px) {
  .stats-grid {
    grid-template-columns: 1fr;
    gap: 8px;
  }
}

/* ═══ 详情网格 ═══ */
.detail-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
  margin-bottom: 20px;
}

.detail-card {
  border-radius: 16px;
}

@media (max-width: 768px) {
  .detail-grid {
    grid-template-columns: 1fr;
  }
}

/* 状态条 */
.status-bar {
  margin-bottom: 12px;
}

.status-label {
  display: flex;
  justify-content: space-between;
  margin-bottom: 4px;
  font-size: 13px;
}

.status-count {
  font-weight: bold;
  color: var(--cat-primary);
}

/* Agent 迷你列表 */
.agent-mini-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.agent-mini {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px;
  border-radius: 10px;
  transition: background 0.2s;
}

.agent-mini:hover {
  background: #faf7f2;
}

.agent-mini-info {
  display: flex;
  flex-direction: column;
}

.agent-mini-name {
  font-weight: 500;
  font-size: 13px;
}

.agent-mini-task {
  font-size: 12px;
  opacity: 0.75;
}

/* Watchdog */
.watchdog-card {
  border-radius: 16px;
}
</style>
