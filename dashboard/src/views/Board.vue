<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { NPageHeader, NSpace, NCard, NTag, NBadge, NButton, NScrollbar, NEmpty, NAvatar, NModal, NInput, NForm, NFormItem, useMessage } from 'naive-ui'
import { Task, TaskStatus, KANBAN_COLUMNS, STATUS_CONFIG, CatCodingApi, AGENT_ROLES_FIXED } from '@/api/types'
import CatAvatar from '@/components/CatAvatar.vue'
import EasterEgg from '@/components/EasterEgg.vue'
import { useResponsive } from '@/composables/useResponsive'

const { isMobile, kanbanMode } = useResponsive()

const api = new CatCodingApi()
const message = useMessage()
const tasks = ref<Task[]>([])
const loading = ref(false)
const showCreateTask = ref(false)
const newTask = ref({ title: '', description: '', role: '' })
const easterEggRef = ref<InstanceType<typeof EasterEgg> | null>(null)
const dragOverColumn = ref<string | null>(null)
const draggedTask = ref<Task | null>(null)

// 任务完成计数（用于彩蛋触发）
const doneCount = computed(() => tasks.value.filter(t => t.status === 'done').length)
const failCount = computed(() => tasks.value.filter(t => t.status === 'failed').length)

async function fetchTasks() {
  loading.value = true
  try {
    tasks.value = await api.getTasks()
    checkEasterEggs()
  } catch {
    // Mock data
    tasks.value = [
      { id: '1', title: '实现用户登录', description: 'JWT 认证 + OAuth2', status: 'active', assigned_to: 'core_dev', depends_on: [], created_at: new Date().toISOString(), updated_at: new Date().toISOString(), artifacts: [] },
      { id: '2', title: 'Vue 3 看板', description: 'Naive UI 猫咪主题', status: 'reviewing', assigned_to: 'frontend', depends_on: [], created_at: new Date().toISOString(), updated_at: new Date().toISOString(), artifacts: [] },
      { id: '3', title: 'NATS 路由优化', description: 'Rust 后端性能调优', status: 'ready', assigned_to: 'backend', depends_on: [], created_at: new Date().toISOString(), updated_at: new Date().toISOString(), artifacts: [] },
      { id: '4', title: '部署脚本 v2', description: 'CI/CD 增强', status: 'done', assigned_to: 'deploy', depends_on: [], created_at: new Date().toISOString(), updated_at: new Date().toISOString(), artifacts: [] },
      { id: '5', title: '单元测试补全', description: '覆盖率 > 80%', status: 'pending', assigned_to: 'tester', depends_on: ['3'], created_at: new Date().toISOString(), updated_at: new Date().toISOString(), artifacts: [] },
      { id: '6', title: '代码审查 #12', description: '玄猫在扫荡', status: 'active', assigned_to: 'reviewer', depends_on: [], created_at: new Date().toISOString(), updated_at: new Date().toISOString(), artifacts: [] },
    ]
  } finally {
    loading.value = false
  }
}

function getTasksByStatus(statuses: TaskStatus[]) {
  return tasks.value.filter(t => statuses.includes(t.status))
}

async function handleStatusChange(taskId: string, newStatus: TaskStatus) {
  try {
    await api.updateTaskStatus(taskId, newStatus)
    message.success('状态已更新 ✨')
    fetchTasks()
  } catch {
    // Mock 本地更新
    const task = tasks.value.find(t => t.id === taskId)
    if (task) task.status = newStatus
    checkEasterEggs()
  }
}

// 拖拽交互
function onDragStart(e: DragEvent, task: Task) {
  draggedTask.value = task
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', task.id)
  }
}

function onDragOver(e: DragEvent, columnKey: string) {
  e.preventDefault()
  dragOverColumn.value = columnKey
}

function onDragLeave() {
  dragOverColumn.value = null
}

function onDrop(e: DragEvent, statuses: TaskStatus[]) {
  e.preventDefault()
  dragOverColumn.value = null
  if (draggedTask.value && statuses.length > 0) {
    handleStatusChange(draggedTask.value.id, statuses[0])
    draggedTask.value = null
  }
}

// 彩蛋检测
function checkEasterEggs() {
  // 连续完成 10 个
  if (doneCount.value >= 10) {
    easterEggRef.value?.triggerEgg('streak_10')
  }
  // 100 个 bug
  if (failCount.value >= 100) {
    easterEggRef.value?.triggerEgg('bugs_100')
  }
  // 全部完成
  if (tasks.value.length > 0 && tasks.value.every(t => t.status === 'done')) {
    easterEggRef.value?.triggerEgg('complete')
  }
}

function getAgentForTask(assignedTo: string | null) {
  if (!assignedTo) return null
  return (AGENT_ROLES_FIXED as any)[assignedTo]
}

// 猫爪点击特效
const pawPrints = ref<{ id: number; x: number; y: number }[]>([])
let pawId = 0
function addPawPrint(e: MouseEvent) {
  const id = ++pawId
  pawPrints.value.push({ id, x: e.clientX, y: e.clientY })
  setTimeout(() => {
    pawPrints.value = pawPrints.value.filter(p => p.id !== id)
  }, 1000)
}

onMounted(fetchTasks)
</script>

<template>
  <div class="board-page" :class="{ mobile: isMobile }" @click="addPawPrint">
    <n-page-header title="🐱 任务看板" subtitle="猫咪团队的做菜进度">
      <template #extra>
        <n-space>
          <n-button @click="fetchTasks" :loading="loading" round :size="isMobile ? 'small' : 'medium'">
            🔄
            <span v-if="!isMobile"> 刷新</span>
          </n-button>
          <n-button type="primary" @click="showCreateTask = true" round :size="isMobile ? 'small' : 'medium'">
            ➕
            <span v-if="!isMobile"> 创建任务</span>
          </n-button>
        </n-space>
      </template>
    </n-page-header>

    <div class="kanban-container" :class="'mode-' + kanbanMode">
      <div
        v-for="col in KANBAN_COLUMNS"
        :key="col.key"
        class="kanban-column"
        :class="{ 'drag-over': dragOverColumn === col.key }"
        @dragover="onDragOver($event, col.key)"
        @dragleave="onDragLeave"
        @drop="onDrop($event, col.statuses as TaskStatus[])"
      >
        <div class="column-header">
          <n-space align="center">
            <span class="emoji">{{ col.emoji }}</span>
            <span class="title">{{ col.title }}</span>
            <n-badge :value="getTasksByStatus(col.statuses as TaskStatus[]).length" :max="99" color="#f5a623" />
          </n-space>
        </div>

        <n-scrollbar class="column-content" style="max-height: 60vh">
          <n-space vertical size="medium">
            <div
              v-for="task in getTasksByStatus(col.statuses as TaskStatus[])"
              :key="task.id"
              class="task-card"
              draggable="true"
              @dragstart="onDragStart($event, task)"
            >
              <!-- 猫耳朵装饰 -->
              <div class="cat-ears">
                <div class="ear ear-left"></div>
                <div class="ear ear-right"></div>
              </div>

              <div class="task-header">
                <span class="task-id">#{{ task.id.slice(0, 4) }}</span>
                <n-tag
                  :color="{ textColor: STATUS_CONFIG[task.status].color, borderColor: STATUS_CONFIG[task.status].color }"
                  size="small"
                  round
                >
                  {{ STATUS_CONFIG[task.status].emoji }} {{ STATUS_CONFIG[task.status].label }}
                </n-tag>
              </div>

              <div class="task-body">
                <div class="task-title">{{ task.title }}</div>
                <div class="task-desc">{{ task.description }}</div>
              </div>

              <div class="task-footer">
                <n-space align="center" justify="space-between">
                  <n-space v-if="task.assigned_to" align="center" size="small">
                    <CatAvatar
                      :emoji="getAgentForTask(task.assigned_to)?.emoji || '🐱'"
                      :name="getAgentForTask(task.assigned_to)?.name || task.assigned_to"
                      :status="task.status === 'active' ? 'working' : task.status === 'done' ? 'done' : task.status === 'failed' ? 'error' : 'idle'"
                      size="small"
                      :show-animation="task.status === 'active'"
                    />
                    <span class="agent-name">{{ getAgentForTask(task.assigned_to)?.name || task.assigned_to }}</span>
                  </n-space>
                  <span v-else class="unassigned">🐱 未分配</span>

                  <!-- 快速推进按钮 -->
                  <n-button
                    v-if="task.status !== 'done' && task.status !== 'failed'"
                    size="tiny"
                    type="primary"
                    quaternary
                    @click.stop="handleStatusChange(task.id, task.status === 'active' ? 'reviewing' : task.status === 'reviewing' ? 'done' : 'active')"
                  >
                    推进 →
                  </n-button>
                </n-space>
              </div>
            </div>

            <n-empty
              v-if="getTasksByStatus(col.statuses as TaskStatus[]).length === 0"
              description="这里没有猫咪的任务 🐱"
              class="empty-col"
            />
          </n-space>
        </n-scrollbar>
      </div>
    </div>

    <!-- 创建任务对话框 -->
    <n-modal v-model:show="showCreateTask" preset="dialog" title="🐱 创建新任务">
      <n-form :model="newTask">
        <n-form-item label="任务标题">
          <n-input v-model:value="newTask.title" placeholder="例如：实现登录功能" />
        </n-form-item>
        <n-form-item label="任务描述">
          <n-input v-model:value="newTask.description" type="textarea" placeholder="详细描述..." />
        </n-form-item>
        <n-form-item label="分配给">
          <n-space>
            <n-tag
              v-for="(info, role) in AGENT_ROLES_FIXED"
              :key="role"
              :type="newTask.role === role ? 'warning' : 'default'"
              checkable
              :checked="newTask.role === role"
              @update:checked="newTask.role = role as string"
              round
            >
              {{ (info as any).emoji }} {{ (info as any).name }}
            </n-tag>
          </n-space>
        </n-form-item>
      </n-form>
      <template #action>
        <n-button type="primary" @click="showCreateTask = false">创建 🐾</n-button>
      </template>
    </n-modal>

    <!-- 彩蛋组件 -->
    <EasterEgg ref="easterEggRef" />

    <!-- 猫爪点击特效 -->
    <div class="paw-prints">
      <div
        v-for="paw in pawPrints"
        :key="paw.id"
        class="paw-print"
        :style="{ left: paw.x + 'px', top: paw.y + 'px' }"
      >
        🐾
      </div>
    </div>
  </div>
</template>

<style scoped>
.board-page {
  min-height: 100vh;
  position: relative;
  cursor: url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><text y="18" font-size="18">🐾</text></svg>'), auto;
}

.kanban-container {
  display: flex;
  gap: 16px;
  margin-top: 24px;
  overflow-x: auto;
  padding-bottom: 16px;
}

/* 移动端：看板列垂直堆叠 */
.kanban-container.mode-stack {
  flex-direction: column;
  overflow-x: visible;
}

.kanban-container.mode-stack .kanban-column {
  min-width: 0;
  width: 100%;
}

/* 平板：水平滚动 */
.kanban-container.mode-scroll {
  gap: 12px;
}

.kanban-container.mode-scroll .kanban-column {
  min-width: 260px;
  flex: 0 0 260px;
}

/* Kanban column */
.kanban-column {
  min-width: 280px;
  flex: 1;
  background: var(--cc-bg-card);
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  border: 2px solid var(--cc-border);
  transition: all 0.3s ease;
  position: relative;
}

.kanban-column.drag-over {
  border-color: var(--cc-orange);
  box-shadow: 0 0 20px rgba(245, 166, 35, 0.3);
  transform: scale(1.02);
}

.column-header {
  padding: 16px;
  border-bottom: 2px dashed var(--cc-border);
  background: rgba(255, 255, 255, 0.6);
  border-radius: 16px 16px 0 0;
}

.column-header .title {
  font-weight: bold;
  font-size: 16px;
}

.column-content {
  flex: 1;
  padding: 12px;
}

/* Task card */
.task-card {
  background: var(--cc-bg);
  border-radius: 12px;
  padding: 14px;
  border: 1px solid var(--cc-border);
  cursor: grab;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
}

.task-card:hover {
  transform: translateY(-4px) rotate(0.5deg);
  box-shadow: 0 8px 24px rgba(245, 166, 35, 0.15);
  border-color: var(--cc-orange);
}

.task-card:active {
  cursor: grabbing;
  transform: scale(0.98);
}

/* 猫耳朵 CSS */
.cat-ears {
  position: absolute;
  top: -10px;
  left: 0;
  right: 0;
  display: flex;
  justify-content: space-between;
  padding: 0 20px;
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.3s;
}

.task-card:hover .cat-ears {
  opacity: 1;
}

.ear {
  width: 0;
  height: 0;
  border-left: 8px solid transparent;
  border-right: 8px solid transparent;
  border-bottom: 12px solid var(--cc-orange);
}

.ear-left {
  transform: rotate(-15deg);
}

.ear-right {
  transform: rotate(15deg);
}

.task-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.task-id {
  font-family: monospace;
  opacity: 0.55;
  font-size: 12px;
}

.task-body {
  margin-bottom: 12px;
}

.task-title {
  font-weight: 600;
  font-size: 14px;
  margin-bottom: 4px;
}

.task-desc {
  font-size: 13px;
  opacity: 0.75;
  line-height: 1.5;
}

.task-footer {
  border-top: 1px dashed var(--cc-border);
  padding-top: 10px;
}

.agent-name {
  font-size: 13px;
  color: var(--cc-orange);
  font-weight: 500;
}

.unassigned {
  font-size: 12px;
  opacity: 0.5;
  font-style: italic;
}

.empty-col {
  padding: 24px 0;
}

/* 猫爪点击特效 */
.paw-prints {
  position: fixed;
  top: 0;
  left: 0;
  pointer-events: none;
  z-index: 9999;
}

.paw-print {
  position: absolute;
  font-size: 20px;
  animation: pawFade 1s ease-out forwards;
  pointer-events: none;
}

@keyframes pawFade {
  0% {
    opacity: 1;
    transform: scale(1) rotate(0deg);
  }
  50% {
    opacity: 0.6;
    transform: scale(1.3) rotate(15deg);
  }
  100% {
    opacity: 0;
    transform: scale(0.5) rotate(30deg) translateY(20px);
  }
}

/* 全局呼吸动画 */
@keyframes breathe {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.05); }
}

/* ═══ 移动端覆盖 ═══ */
.mobile .board-page {
  padding: 8px;
}

.mobile .kanban-container {
  margin-top: 12px;
  gap: 10px;
}

.mobile .kanban-column {
  border-radius: 12px;
}

.mobile .column-header {
  padding: 10px 12px;
}

.mobile .column-header .title {
  font-size: 14px;
}

.mobile .column-content {
  padding: 8px;
}

.mobile .task-card {
  padding: 10px;
}

.mobile .task-title {
  font-size: 13px;
}

.mobile .task-desc {
  font-size: 12px;
}

.mobile .cat-ears {
  display: none; /* 移动端太小显示不了耳朵 */
}
</style>
