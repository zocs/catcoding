<template>
  <div class="logs-page">
    <div class="logs-header">
      <h2>📜 {{ t('logs.title') }}</h2>
      <div class="logs-controls">
        <n-button size="small" @click="fetchLogs" :loading="loading">
          🔄 {{ t('logs.refresh') }}
        </n-button>
        <n-switch v-model:value="autoRefresh" size="small">
          <template #checked>{{ t('logs.auto') }}</template>
          <template #unchecked>{{ t('logs.auto') }}</template>
        </n-switch>
        <n-select
          v-model:value="levelFilter"
          :options="levelOptions"
          size="small"
          style="width: 120px"
          :placeholder="t('logs.filterLevel')"
        />
      </div>
    </div>

    <div class="logs-body" ref="logsContainer">
      <div v-if="filteredLogs.length === 0" class="logs-empty">
        📭 {{ t('logs.noLogs') }}
      </div>
      <div
        v-for="(log, i) in filteredLogs"
        :key="i"
        class="log-entry"
        :class="'log-' + log.level.toLowerCase()"
      >
        <span class="log-time">{{ formatTime(log.timestamp) }}</span>
        <span class="log-level" :class="'level-' + log.level.toLowerCase()">{{ log.level }}</span>
        <span class="log-target">{{ log.target }}</span>
        <span class="log-msg">{{ cleanMsg(log.message) }}</span>
      </div>
    </div>

    <div class="logs-footer">
      {{ filteredLogs.length }} / {{ logs.length }} {{ t('logs.entries') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, nextTick, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { NButton, NSwitch, NSelect } from 'naive-ui'

const { t } = useI18n()

interface LogEntry {
  timestamp: string
  level: string
  target: string
  message: string
}

const logs = ref<LogEntry[]>([])
const loading = ref(false)
const autoRefresh = ref(true)
const levelFilter = ref<string | null>(null)
const logsContainer = ref<HTMLElement | null>(null)
let refreshTimer: ReturnType<typeof setInterval> | null = null

const levelOptions = [
  { label: 'All', value: null },
  { label: 'ERROR', value: 'ERROR' },
  { label: 'WARN', value: 'WARN' },
  { label: 'INFO', value: 'INFO' },
  { label: 'DEBUG', value: 'DEBUG' },
]

const filteredLogs = computed(() => {
  if (!levelFilter.value) return logs.value
  return logs.value.filter(l => l.level === levelFilter.value)
})

async function fetchLogs() {
  loading.value = true
  try {
    const resp = await fetch('/api/logs')
    const data = await resp.json()
    logs.value = data.logs || []
    await nextTick()
    scrollToBottom()
  } catch (e) {
    console.error('Failed to fetch logs:', e)
  } finally {
    loading.value = false
  }
}

function scrollToBottom() {
  if (logsContainer.value) {
    logsContainer.value.scrollTop = logsContainer.value.scrollHeight
  }
}

function formatTime(ts: string): string {
  try {
    const d = new Date(ts)
    return d.toLocaleTimeString('en-US', { hour12: false })
  } catch {
    return ts
  }
}

function cleanMsg(msg: string): string {
  // Remove ANSI escape codes and surrounding quotes from debug output
  return msg.replace(/\x1b\[[0-9;]*m/g, '').replace(/^"|"$/g, '')
}

watch(autoRefresh, (on) => {
  if (on) {
    refreshTimer = setInterval(fetchLogs, 3000)
  } else if (refreshTimer) {
    clearInterval(refreshTimer)
    refreshTimer = null
  }
})

onMounted(() => {
  fetchLogs()
  if (autoRefresh.value) {
    refreshTimer = setInterval(fetchLogs, 3000)
  }
})

onUnmounted(() => {
  if (refreshTimer) {
    clearInterval(refreshTimer)
  }
})
</script>

<style scoped>
.logs-page {
  display: flex;
  flex-direction: column;
  height: calc(100vh - 80px);
}
.logs-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}
.logs-header h2 {
  margin: 0;
  font-size: 20px;
}
.logs-controls {
  display: flex;
  gap: 12px;
  align-items: center;
}
.logs-body {
  flex: 1;
  overflow-y: auto;
  background: #1e1e2e;
  border-radius: 8px;
  padding: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 13px;
  line-height: 1.6;
}
.logs-empty {
  color: #666;
  text-align: center;
  padding: 40px;
}
.log-entry {
  display: flex;
  gap: 8px;
  padding: 2px 0;
  border-bottom: 1px solid #2a2a3a;
}
.log-entry:hover {
  background: #2a2a3a;
}
.log-time {
  color: #666;
  min-width: 70px;
  flex-shrink: 0;
}
.log-level {
  min-width: 50px;
  flex-shrink: 0;
  font-weight: bold;
}
.level-error { color: #f38ba8; }
.level-warn  { color: #fab387; }
.level-info  { color: #a6e3a1; }
.level-debug { color: #89b4fa; }
.level-trace { color: #cba6f7; }
.log-target {
  color: #585b70;
  min-width: 140px;
  flex-shrink: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.log-msg {
  color: #cdd6f4;
  word-break: break-all;
}
.logs-footer {
  margin-top: 8px;
  color: #666;
  font-size: 12px;
  text-align: right;
}
</style>
