<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { NPageHeader, NCard, NTag, NSpace, NButton, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

interface Task {
  id: string
  title: string
  status: string
  assigned_to: string | null
  cat_name: string
  cat_emoji: string
  start: number      // day offset from project start
  duration: number   // days
  color: string
}

const message = useMessage()
const tasks = ref<Task[]>([])
const today = ref(0) // simulated "today" marker

// Demo 任务数据 — 完整的项目甘特图
const DEMO_TASKS: Task[] = [
  { id: '1', title: 'NATS 消息总线集成', status: 'done', assigned_to: 'core_dev', cat_name: '英短蓝猫', cat_emoji: '🐱', start: 0, duration: 3, color: '#2ecc71' },
  { id: '2', title: 'Watchdog 健康检测', status: 'done', assigned_to: 'backend', cat_name: '缅因猫', cat_emoji: '🐱', start: 2, duration: 4, color: '#2ecc71' },
  { id: '3', title: 'Dashboard Vue3 基础框架', status: 'done', assigned_to: 'frontend', cat_name: '橘猫', cat_emoji: '🐱', start: 1, duration: 5, color: '#2ecc71' },
  { id: '4', title: 'Rust Daemon CLI', status: 'done', assigned_to: 'core_dev', cat_name: '英短蓝猫', cat_emoji: '🐱', start: 3, duration: 3, color: '#2ecc71' },
  { id: '5', title: 'Python Agent SDK', status: 'active', assigned_to: 'core_dev', cat_name: '英短蓝猫', cat_emoji: '🐱', start: 6, duration: 5, color: '#3498db' },
  { id: '6', title: 'Kanban 看板视图', status: 'active', assigned_to: 'frontend', cat_name: '橘猫', cat_emoji: '🐱', start: 6, duration: 4, color: '#3498db' },
  { id: '7', title: '代码评审工作流', status: 'reviewing', assigned_to: 'reviewer', cat_name: '玄猫', cat_emoji: '🖤', start: 8, duration: 3, color: '#9b59b6' },
  { id: '8', title: 'Bug 追踪动画系统', status: 'active', assigned_to: 'frontend', cat_name: '橘猫', cat_emoji: '🐱', start: 10, duration: 3, color: '#3498db' },
  { id: '9', title: 'Agent 间通信协议', status: 'pending', assigned_to: 'backend', cat_name: '缅因猫', cat_emoji: '🐱', start: 11, duration: 4, color: '#f39c12' },
  { id: '10', title: 'CI/CD 自动部署', status: 'pending', assigned_to: 'devops', cat_name: '三花猫', cat_emoji: '🐱', start: 13, duration: 3, color: '#f39c12' },
  { id: '11', title: 'L4 记忆系统', status: 'pending', assigned_to: 'pm', cat_name: '暹罗猫', cat_emoji: '🐱', start: 15, duration: 5, color: '#e74c3c' },
  { id: '12', title: '多 Agent 协同测试', status: 'pending', assigned_to: 'qa', cat_name: '波斯猫', cat_emoji: '🐱', start: 16, duration: 4, color: '#e74c3c' },
]

const totalDays = computed(() => Math.max(...DEMO_TASKS.map(t => t.start + t.duration)) + 1)

// 按猫咪分组
const catGroups = computed(() => {
  const groups: Record<string, Task[]> = {}
  for (const t of tasks.value) {
    const key = t.cat_name
    if (!groups[key]) groups[key] = []
    groups[key].push(t)
  }
  return groups
})

const statusEmoji: Record<string, string> = {
  done: '✅', active: '🔵', reviewing: '🔍', pending: '⏳', blocked: '🚫', failed: '❌',
}

async function refresh() {
  try {
    const res = await fetch('/api/tasks')
    const data = await res.json()
    if (data.tasks?.length > 0) {
      // 如果有真实数据，用真实数据
      tasks.value = data.tasks.map((t: any, i: number) => ({
        ...t,
        cat_name: t.assigned_to || '未分配',
        cat_emoji: '🐱',
        start: i * 2,
        duration: 3 + Math.floor(Math.random() * 4),
        color: '#3498db',
      }))
    } else {
      throw new Error('no data')
    }
  } catch {
    // Demo 模式
    tasks.value = DEMO_TASKS
    today.value = 9 // 模拟第 9 天
    message.info('🎮 Demo 模式 — 展示示例甘特图')
  }
}

onMounted(refresh)
</script>

<template>
  <div class="gantt-page">
    <n-page-header title="📊 甘特图" subtitle="猫咪团队任务时间线">
      <template #extra>
        <n-button @click="refresh" round>🔄 刷新</n-button>
      </template>
    </n-page-header>

    <n-card style="margin-top: 24px" class="gantt-card">
      <!-- 时间标尺 -->
      <div class="time-ruler">
        <div class="ruler-label">任务</div>
        <div class="ruler-days">
          <div v-for="d in totalDays" :key="d" class="ruler-day" :class="{ today: d - 1 === today }">
            Day {{ d - 1 }}
          </div>
        </div>
      </div>

      <!-- 今日线 -->
      <div class="today-line" :style="{ left: `calc(200px + ${today} * 60px + 30px)` }">
        <span class="today-label">📍 今天</span>
      </div>

      <!-- 任务行 -->
      <div v-for="task in tasks" :key="task.id" class="gantt-row">
        <div class="row-label">
          <span class="row-emoji">{{ task.cat_emoji }}</span>
          <div class="row-info">
            <span class="row-title">{{ task.title }}</span>
            <span class="row-meta">{{ task.cat_name }} · {{ statusEmoji[task.status] }} {{ task.status }}</span>
          </div>
        </div>
        <div class="row-bar-area">
          <div
            class="bar"
            :class="[`status-${task.status}`]"
            :style="{
              left: `${task.start * 60}px`,
              width: `${task.duration * 60 - 4}px`,
              background: task.color,
            }"
          >
            <span class="bar-label">{{ task.duration }}d</span>
          </div>
        </div>
      </div>

      <!-- 图例 -->
      <div class="legend">
        <span class="legend-item"><span class="legend-dot" style="background: #2ecc71"></span> done</span>
        <span class="legend-item"><span class="legend-dot" style="background: #3498db"></span> active</span>
        <span class="legend-item"><span class="legend-dot" style="background: #9b59b6"></span> reviewing</span>
        <span class="legend-item"><span class="legend-dot" style="background: #f39c12"></span> pending</span>
        <span class="legend-item"><span class="legend-dot" style="background: #e74c3c"></span> blocked</span>
      </div>
    </n-card>
  </div>
</template>

<style scoped>
.gantt-page { padding: 16px; }

.gantt-card {
  border-radius: 16px;
  overflow: hidden;
}

/* 时间标尺 */
.time-ruler {
  display: flex;
  border-bottom: 2px solid #eee;
  padding-bottom: 8px;
  margin-bottom: 8px;
  position: sticky;
  top: 0;
  background: white;
  z-index: 2;
}

.ruler-label {
  width: 200px;
  min-width: 200px;
  font-weight: bold;
  font-size: 13px;
  color: #888;
  display: flex;
  align-items: center;
}

.ruler-days {
  display: flex;
  gap: 0;
  flex: 1;
  overflow-x: auto;
}

.ruler-day {
  width: 60px;
  min-width: 60px;
  font-size: 11px;
  color: #aaa;
  text-align: center;
  border-left: 1px solid #f0f0f0;
  padding: 4px 0;
}

.ruler-day.today {
  color: #f5a623;
  font-weight: bold;
  background: rgba(245, 166, 35, 0.08);
}

/* 今日线 */
.today-line {
  position: absolute;
  top: 40px;
  bottom: 20px;
  width: 2px;
  background: #f5a623;
  z-index: 1;
  pointer-events: none;
  opacity: 0.6;
}

.today-label {
  position: absolute;
  top: -18px;
  left: -16px;
  font-size: 11px;
  white-space: nowrap;
  color: #f5a623;
}

/* 任务行 */
.gantt-row {
  display: flex;
  align-items: center;
  border-bottom: 1px solid #f5f5f5;
  min-height: 48px;
  transition: background 0.2s;
}

.gantt-row:hover {
  background: rgba(245, 166, 35, 0.04);
}

.row-label {
  width: 200px;
  min-width: 200px;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
}

.row-emoji { font-size: 20px; }

.row-info {
  display: flex;
  flex-direction: column;
}

.row-title {
  font-size: 13px;
  font-weight: 600;
  line-height: 1.3;
}

.row-meta {
  font-size: 11px;
  color: #aaa;
}

/* 甘特条 */
.row-bar-area {
  flex: 1;
  position: relative;
  height: 32px;
  overflow-x: auto;
}

.bar {
  position: absolute;
  height: 28px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  cursor: pointer;
  top: 2px;
}

.bar:hover {
  transform: scaleY(1.15);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  filter: brightness(1.1);
}

.bar.status-done { opacity: 0.7; }
.bar.status-active {
  animation: barPulse 2s ease-in-out infinite;
}
.bar.status-reviewing {
  animation: barPulse 1.5s ease-in-out infinite;
  border: 2px dashed rgba(255, 255, 255, 0.5);
}

@keyframes barPulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(255, 255, 255, 0.2); }
  50%      { box-shadow: 0 0 12px 2px rgba(255, 255, 255, 0.15); }
}

.bar-label {
  color: white;
  font-size: 11px;
  font-weight: bold;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}

/* 图例 */
.legend {
  display: flex;
  gap: 16px;
  margin-top: 20px;
  padding-top: 12px;
  border-top: 1px solid #eee;
  flex-wrap: wrap;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #888;
}

.legend-dot {
  width: 12px;
  height: 12px;
  border-radius: 3px;
}

/* 响应式 */
@media (max-width: 768px) {
  .ruler-label, .row-label { width: 120px; min-width: 120px; }
  .row-title { font-size: 11px; }
  .ruler-day { width: 50px; min-width: 50px; font-size: 10px; }
  .bar { height: 22px; }
  .bar-label { font-size: 9px; }
}

@media (max-width: 480px) {
  .ruler-label, .row-label { width: 80px; min-width: 80px; }
  .row-emoji { display: none; }
  .row-meta { display: none; }
}
</style>
