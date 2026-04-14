<template>
  <div class="gantt">
    <n-page-header title="📊 甘特图" subtitle="任务时间线">
      <template #extra>
        <n-button @click="refresh">
          <template #icon><span>🔄</span></template>
          刷新
        </n-button>
      </template>
    </n-page-header>

    <n-card style="margin-top: 24px">
      <div class="gantt-chart">
        <div v-if="tasks.length === 0" class="empty-state">
          <n-empty description="暂无任务数据" />
        </div>
        <div v-else class="timeline">
          <div v-for="task in tasks" :key="task.id" class="timeline-item">
            <div class="timeline-dot" :class="getStatusClass(task.status)"></div>
            <div class="timeline-content">
              <div class="timeline-header">
                <span class="task-title">{{ task.title }}</span>
                <n-tag size="tiny" :type="getStatusTagType(task.status)">
                  {{ getStatusEmoji(task.status) }} {{ task.status }}
                </n-tag>
              </div>
              <div class="timeline-meta">
                <span>{{ task.assigned_to || '未分配' }}</span>
                <span>{{ formatDate(task.created_at) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useMessage } from 'naive-ui'

interface Task {
  id: string
  title: string
  status: string
  assigned_to: string | null
  created_at: string
}

const message = useMessage()
const tasks = ref<Task[]>([])

const statusEmoji: Record<string, string> = {
  pending: '⏳',
  blocked: '🚫',
  ready: '🟡',
  active: '🔵',
  reviewing: '🔍',
  done: '✅',
  failed: '❌',
}

function getStatusEmoji(status: string) {
  return statusEmoji[status] || '❓'
}

function getStatusClass(status: string) {
  return `status-${status}`
}

function getStatusTagType(status: string) {
  const types: Record<string, string> = {
    pending: 'default',
    blocked: 'error',
    ready: 'warning',
    active: 'info',
    reviewing: 'success',
    done: 'success',
    failed: 'error',
  }
  return (types[status] || 'default') as any
}

function formatDate(dateStr: string) {
  if (!dateStr) return ''
  const date = new Date(dateStr)
  return date.toLocaleString('zh-CN', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

async function refresh() {
  try {
    const res = await fetch('/api/tasks')
    const data = await res.json()
    tasks.value = data.tasks || []
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
.gantt {
  padding: 16px;
}

.timeline {
  padding: 16px 0;
}

.timeline-item {
  display: flex;
  gap: 16px;
  padding: 12px 0;
  border-left: 2px solid #333;
  margin-left: 8px;
  padding-left: 24px;
  position: relative;
}

.timeline-dot {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  position: absolute;
  left: -9px;
  top: 16px;
}

.status-pending { background: #666; }
.status-blocked { background: #e94560; }
.status-ready { background: #f1c40f; }
.status-active { background: #3498db; }
.status-reviewing { background: #9b59b6; }
.status-done { background: #2ecc71; }
.status-failed { background: #e74c3c; }

.timeline-content {
  flex: 1;
}

.timeline-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.task-title {
  font-weight: bold;
}

.timeline-meta {
  display: flex;
  gap: 16px;
  font-size: 12px;
  color: #888;
  margin-top: 4px;
}

.empty-state {
  padding: 48px;
  display: flex;
  justify-content: center;
}
</style>
