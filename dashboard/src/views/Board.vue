<template>
  <div class="board">
    <n-page-header title="📋 任务看板" subtitle="猫咪团队的任务进度">
      <template #extra>
        <n-button type="primary" @click="showCreateTask = true">
          <template #icon><span>➕</span></template>
          创建任务
        </n-button>
        <n-button @click="refreshTasks">
          <template #icon><span>🔄</span></template>
          刷新
        </n-button>
      </template>
    </n-page-header>

    <!-- 看板列 -->
    <div class="kanban">
      <div class="kanban-column" v-for="column in columns" :key="column.status">
        <div class="column-header">
          <span class="column-emoji">{{ column.emoji }}</span>
          <span class="column-title">{{ column.title }}</span>
          <n-badge :value="getColumnTasks(column.status).length" :max="99" />
        </div>
        <div class="column-content">
          <n-card
            v-for="task in getColumnTasks(column.status)"
            :key="task.id"
            size="small"
            class="task-card"
            hoverable
          >
            <template #header>
              <span class="task-title">{{ task.title }}</span>
            </template>
            <template #header-extra>
              <n-tag size="tiny" :type="getRoleTagType(task.assigned_to)">
                {{ getRoleEmoji(task.assigned_to) }} {{ task.assigned_to || '未分配' }}
              </n-tag>
            </template>
            <p class="task-desc">{{ task.description || '暂无描述' }}</p>
            <template #footer>
              <n-space justify="space-between">
                <n-button size="tiny" @click="viewTask(task)">详情</n-button>
                <n-button size="tiny" type="primary" @click="updateStatus(task)">
                  推进 →
                </n-button>
              </n-space>
            </template>
          </n-card>
        </div>
      </div>
    </div>

    <!-- 创建任务对话框 -->
    <n-modal v-model:show="showCreateTask" preset="dialog" title="🐱 创建任务">
      <n-form ref="formRef" :model="newTask">
        <n-form-item label="任务标题" path="title">
          <n-input v-model:value="newTask.title" placeholder="例如：实现登录功能" />
        </n-form-item>
        <n-form-item label="任务描述" path="description">
          <n-input
            v-model:value="newTask.description"
            type="textarea"
            placeholder="详细描述任务需求..."
          />
        </n-form-item>
        <n-form-item label="分配给" path="role">
          <n-select
            v-model:value="newTask.role"
            :options="roleOptions"
            placeholder="选择角色"
          />
        </n-form-item>
      </n-form>
      <template #action>
        <n-button @click="showCreateTask = false">取消</n-button>
        <n-button type="primary" @click="createTask">创建</n-button>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useMessage } from 'naive-ui'

interface Task {
  id: string
  title: string
  description: string
  status: string
  assigned_to: string | null
  created_at: string
}

const message = useMessage()
const tasks = ref<Task[]>([])
const showCreateTask = ref(false)
const newTask = ref({
  title: '',
  description: '',
  role: null as string | null,
})

const columns = [
  { status: 'pending', title: '排队中', emoji: '⏳' },
  { status: 'blocked', title: '被阻塞', emoji: '🚫' },
  { status: 'ready', title: '就绪', emoji: '🟡' },
  { status: 'active', title: '烹饪中', emoji: '🔵' },
  { status: 'reviewing', title: '品尝中', emoji: '🔍' },
  { status: 'done', title: '美味！', emoji: '✅' },
]

const roleOptions = [
  { label: '🐱 英短蓝猫 (Core Dev)', value: 'core_dev' },
  { label: '🐱 橘猫 (Frontend)', value: 'frontend' },
  { label: '🐱 缅因猫 (Backend)', value: 'backend' },
  { label: '🐱 玄猫 (Reviewer)', value: 'reviewer' },
  { label: '🐱 阿比西尼亚猫 (Tester)', value: 'tester' },
]

function getColumnTasks(status: string) {
  return tasks.value.filter((t) => t.status === status)
}

function getRoleEmoji(role: string | null) {
  const emojis: Record<string, string> = {
    core_dev: '🐱',
    frontend: '🐱',
    backend: '🐱',
    reviewer: '🐱',
    tester: '🐱',
  }
  return emojis[role || ''] || '❓'
}

function getRoleTagType(role: string | null) {
  const types: Record<string, string> = {
    core_dev: 'info',
    frontend: 'success',
    backend: 'warning',
    reviewer: 'error',
    tester: 'default',
  }
  return (types[role || ''] || 'default') as any
}

async function refreshTasks() {
  try {
    const res = await fetch('/api/tasks')
    const data = await res.json()
    tasks.value = data.tasks || []
    message.success('刷新成功')
  } catch (e) {
    message.error('刷新失败')
  }
}

async function createTask() {
  if (!newTask.value.title) {
    message.warning('请输入任务标题')
    return
  }

  try {
    const res = await fetch('/api/tasks', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(newTask.value),
    })
    const data = await res.json()
    message.success(`任务已创建: ${data.title}`)
    showCreateTask.value = false
    newTask.value = { title: '', description: '', role: null }
    refreshTasks()
  } catch (e) {
    message.error('创建失败')
  }
}

function viewTask(task: Task) {
  message.info(`查看任务: ${task.title}`)
}

async function updateStatus(task: Task) {
  const statusMap: Record<string, string> = {
    pending: 'ready',
    ready: 'active',
    active: 'reviewing',
    reviewing: 'done',
  }
  const nextStatus = statusMap[task.status]
  if (!nextStatus) return

  try {
    await fetch(`/api/tasks/${task.id}/status`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ status: nextStatus }),
    })
    message.success(`任务状态更新: ${nextStatus}`)
    refreshTasks()
  } catch (e) {
    message.error('更新失败')
  }
}

onMounted(() => {
  refreshTasks()
})
</script>

<style scoped>
.board {
  padding: 16px;
}

.kanban {
  display: flex;
  gap: 16px;
  margin-top: 24px;
  overflow-x: auto;
}

.kanban-column {
  min-width: 280px;
  max-width: 320px;
  flex: 1;
  background: #1e1e2e;
  border-radius: 12px;
  padding: 16px;
}

.column-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 16px;
  font-weight: bold;
}

.column-emoji {
  font-size: 20px;
}

.column-title {
  flex: 1;
}

.column-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.task-card {
  cursor: pointer;
}

.task-title {
  font-weight: bold;
}

.task-desc {
  font-size: 12px;
  color: #888;
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}
</style>
