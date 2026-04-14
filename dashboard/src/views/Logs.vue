<template>
  <div class="logs">
    <n-page-header title="🍳 厨房日志" subtitle="实时 Bug 处理流程">
      <template #extra>
        <n-button @click="clearLogs">
          <template #icon><span>🧹</span></template>
          清空
        </n-button>
        <n-button type="primary" @click="toggleAutoScroll">
          <template #icon><span>{{ autoScroll ? '⏸️' : '▶️' }}</span></template>
          {{ autoScroll ? '暂停' : '自动滚动' }}
        </n-button>
      </template>
    </n-page-header>

    <n-card style="margin-top: 24px">
      <div class="log-container" ref="logContainer">
        <div v-for="(log, index) in logs" :key="index" class="log-entry" :class="log.level">
          <span class="log-time">{{ log.time }}</span>
          <span class="log-level">{{ getLevelEmoji(log.level) }}</span>
          <span class="log-source">{{ log.source }}</span>
          <span class="log-message">{{ log.message }}</span>
        </div>
        <div v-if="logs.length === 0" class="empty-logs">
          <n-empty description="暂无日志，等待猫咪团队开工..." />
        </div>
      </div>
    </n-card>

    <!-- Bug 处理统计 -->
    <div class="bug-stats">
      <n-card title="🐭 老鼠统计">
        <n-space>
          <n-statistic label="小老鼠 🐭" :value="bugStats.mouse" />
          <n-statistic label="大老鼠 🐀" :value="bugStats.rat" />
          <n-statistic label="蝙蝠 🦇" :value="bugStats.bat" />
          <n-statistic label="恶龙 🐉" :value="bugStats.dragon" />
        </n-space>
      </n-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'

interface LogEntry {
  time: string
  level: string
  source: string
  message: string
}

const logs = ref<LogEntry[]>([])
const autoScroll = ref(true)
const logContainer = ref<HTMLElement | null>(null)

const bugStats = ref({
  mouse: 0,
  rat: 0,
  bat: 0,
  dragon: 0,
})

function getLevelEmoji(level: string) {
  const emojis: Record<string, string> = {
    info: 'ℹ️',
    warn: '⚠️',
    error: '❌',
    debug: '🐛',
    success: '✅',
  }
  return emojis[level] || '📝'
}

function addLog(entry: LogEntry) {
  logs.value.push(entry)
  if (logs.value.length > 1000) {
    logs.value.shift()
  }
  if (autoScroll.value) {
    nextTick(() => {
      if (logContainer.value) {
        logContainer.value.scrollTop = logContainer.value.scrollHeight
      }
    })
  }
}

function clearLogs() {
  logs.value = []
}

function toggleAutoScroll() {
  autoScroll.value = !autoScroll.value
}

function formatTime() {
  return new Date().toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })
}

// 模拟日志（实际应从 SSE 接收）
let logInterval: number | null = null

onMounted(() => {
  // 添加示例日志
  addLog({ time: formatTime(), level: 'info', source: 'Daemon', message: '🐱 CatCoding 启动' })
  addLog({ time: formatTime(), level: 'success', source: 'NATS', message: '📡 已连接 NATS: nats://127.0.0.1:4222' })
  addLog({ time: formatTime(), level: 'info', source: 'Watchdog', message: '🦉 猫头鹰已就位' })

  // 模拟日志流
  logInterval = window.setInterval(() => {
    const sources = ['PM', 'Dev', 'Reviewer', 'Tester', 'TechScout']
    const messages = [
      '📋 分析需求中...',
      '🔧 实现功能...',
      '🔍 审查代码...',
      '🧪 运行测试...',
      '📚 搜索文档...',
    ]
    const levels = ['info', 'debug', 'success', 'warn']

    if (Math.random() > 0.7) {
      addLog({
        time: formatTime(),
        level: levels[Math.floor(Math.random() * levels.length)],
        source: sources[Math.floor(Math.random() * sources.length)],
        message: messages[Math.floor(Math.random() * messages.length)],
      })
    }
  }, 3000)
})

onUnmounted(() => {
  if (logInterval) {
    clearInterval(logInterval)
  }
})
</script>

<style scoped>
.logs {
  padding: 16px;
}

.log-container {
  height: 400px;
  overflow-y: auto;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 13px;
  background: #0d0d14;
  padding: 16px;
  border-radius: 8px;
}

.log-entry {
  display: flex;
  gap: 12px;
  padding: 4px 0;
  border-bottom: 1px solid #1a1a2e;
}

.log-entry.warn {
  color: #f1c40f;
}

.log-entry.error {
  color: #e74c3c;
}

.log-entry.success {
  color: #2ecc71;
}

.log-time {
  color: #666;
  min-width: 80px;
}

.log-level {
  min-width: 24px;
}

.log-source {
  color: #9b59b6;
  min-width: 80px;
}

.log-message {
  flex: 1;
}

.empty-logs {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 100%;
}

.bug-stats {
  margin-top: 24px;
}
</style>
