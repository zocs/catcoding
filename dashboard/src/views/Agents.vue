<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { NPageHeader, NCard, NTag, NSpace, NButton, NEmpty, useMessage } from 'naive-ui'
import { Agent, CatCodingApi, AGENT_ROLES_FIXED } from '@/api/types'
import CatAvatarSVG from '@/components/CatAvatarSVG.vue'
import EasterEgg from '@/components/EasterEgg.vue'

const api = new CatCodingApi()
const message = useMessage()
const agents = ref<Agent[]>([])
const loading = ref(false)
const easterEggRef = ref<InstanceType<typeof EasterEgg> | null>(null)
const selectedAgent = ref<string | null>(null)
const feedCount = ref<Record<string, number>>({})
const windowWidth = ref(window.innerWidth)

// 响应式列数
const gridCols = computed(() => {
  if (windowWidth.value < 480) return 1
  if (windowWidth.value < 768) return 2
  if (windowWidth.value < 1024) return 3
  return 4
})

function onResize() {
  windowWidth.value = window.innerWidth
}

onMounted(() => {
  window.addEventListener('resize', onResize)
  fetchAgents()
})

onUnmounted(() => {
  window.removeEventListener('resize', onResize)
})

async function fetchAgents() {
  loading.value = true
  try {
    agents.value = await api.getAgents()
  } catch {
    agents.value = Object.entries(AGENT_ROLES_FIXED).map(([role, info]) => ({
      role,
      name: info.name,
      emoji: info.emoji,
      status: ['active', 'active', 'idle', 'idle', 'idle', 'busy', 'idle', 'active', 'active', 'idle'][Math.floor(Math.random() * 10)] as any,
      description: info.desc,
      mode: info.mode as any,
      current_task: Math.random() > 0.6 ? `task-${Math.floor(Math.random() * 100)}` : null,
    }))
  } finally {
    loading.value = false
  }
}

// "投喂"互动
function feedAgent(role: string) {
  feedCount.value[role] = (feedCount.value[role] || 0) + 1
  message.success(`🐟 投喂了 ${getAgentInfo(role)?.name}！(第 ${feedCount.value[role]} 条小鱼干)`)

  // 投喂 5 次触发彩蛋
  if (feedCount.value[role] === 5) {
    easterEggRef.value?.triggerEgg('magic')
  }
  // 所有 agent 都被投喂过
  const allFed = Object.keys(AGENT_ROLES_FIXED).every(r => (feedCount.value[r] || 0) > 0)
  if (allFed) {
    easterEggRef.value?.triggerEgg('streak_10')
  }
}

function getAgentInfo(role: string) {
  return (AGENT_ROLES_FIXED as any)[role]
}

function getStatusLabel(status: string) {
  const map: Record<string, string> = { idle: '😴 休息中', active: '🔧 工作中', busy: '⚡ 繁忙', error: '🙀 出错了' }
  return map[status] || status
}

function getStatusType(status: string) {
  const map: Record<string, 'default' | 'success' | 'warning' | 'error'> = { idle: 'default', active: 'success', busy: 'warning', error: 'error' }
  return map[status] || 'default'
}

// 按模式分组
const residents = computed(() => agents.value.filter(a => getAgentInfo(a.role)?.mode === 'resident'))
const onDemand = computed(() => agents.value.filter(a => getAgentInfo(a.role)?.mode === 'on_demand'))
const decorative = computed(() => agents.value.filter(a => getAgentInfo(a.role)?.mode === 'decorative'))
</script>

<template>
  <div class="agents-page">
    <n-page-header title="🐱 猫咪面板" subtitle="Agent 团队实时状态">
      <template #extra>
        <n-space>
          <n-button @click="fetchAgents" :loading="loading" round>
            🔄 刷新
          </n-button>
        </n-space>
      </template>
    </n-page-header>

    <!-- 常驻 Agent -->
    <div class="agent-section" v-if="residents.length > 0">
      <div class="section-title">🏠 常驻猫咪</div>
      <div class="agent-grid" :style="{ '--cols': gridCols }">
        <div
          v-for="agent in residents"
          :key="agent.role"
          class="agent-card"
          :class="[agent.status, { selected: selectedAgent === agent.role }]"
          @click="selectedAgent = selectedAgent === agent.role ? null : agent.role"
        >
          <!-- 猫耳朵装饰 -->
          <div class="card-ears">
            <div class="card-ear left"></div>
            <div class="card-ear right"></div>
          </div>

          <div class="card-body">
            <!-- SVG 头像 -->
            <div class="avatar-wrapper">
              <CatAvatarSVG
                :role="agent.role"
                :status="agent.status"
                :size="80"
                :animated="agent.status === 'active'"
              />
            </div>

            <div class="agent-info">
              <div class="agent-name-row">
                <span class="agent-name">{{ agent.name }}</span>
                <span class="agent-role-tag">@{{ agent.role }}</span>
              </div>
              <div class="agent-desc">{{ agent.description }}</div>
              <div class="agent-status">
                <n-tag :type="getStatusType(agent.status)" size="small" round>
                  {{ getStatusLabel(agent.status) }}
                </n-tag>
              </div>
              <div v-if="agent.current_task" class="current-task">
                🔧 正在处理: <span class="task-id">#{{ agent.current_task }}</span>
              </div>
            </div>
          </div>

          <!-- 投喂按钮 -->
          <div class="card-footer">
            <button class="feed-btn" @click.stop="feedAgent(agent.role)" title="投喂小鱼干">
              🐟 {{ feedCount[agent.role] || 0 }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 按需 Agent -->
    <div class="agent-section" v-if="onDemand.length > 0">
      <div class="section-title">⚡ 按需猫咪</div>
      <div class="agent-grid" :style="{ '--cols': gridCols }">
        <div
          v-for="agent in onDemand"
          :key="agent.role"
          class="agent-card compact"
          :class="[agent.status]"
          @click="selectedAgent = selectedAgent === agent.role ? null : agent.role"
        >
          <div class="card-ears">
            <div class="card-ear left"></div>
            <div class="card-ear right"></div>
          </div>
          <div class="card-body compact-body">
            <CatAvatarSVG :role="agent.role" :status="agent.status" :size="56" />
            <div class="compact-info">
              <span class="agent-name">{{ agent.name }}</span>
              <span class="agent-role-tag">@{{ agent.role }}</span>
              <n-tag :type="getStatusType(agent.status)" size="tiny" round>
                {{ getStatusLabel(agent.status) }}
              </n-tag>
              <div v-if="agent.current_task" class="current-task compact-task">
                🔧 #{{ agent.current_task }}
              </div>
            </div>
          </div>
          <div class="card-footer">
            <button class="feed-btn small" @click.stop="feedAgent(agent.role)">
              🐟 {{ feedCount[agent.role] || 0 }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 吉祥物 -->
    <div class="agent-section" v-if="decorative.length > 0">
      <div class="section-title">🎋 吉祥物</div>
      <div class="agent-grid mascot-grid" :style="{ '--cols': Math.min(gridCols, 3) }">
        <div
          v-for="agent in decorative"
          :key="agent.role"
          class="agent-card mascot-card"
          @click="feedAgent(agent.role)"
        >
          <div class="mascot-wrapper">
            <CatAvatarSVG :role="agent.role" status="idle" :size="120" :animated="true" />
            <div class="mascot-name">{{ agent.name }}</div>
            <div class="agent-role-tag">@{{ agent.role }}</div>
            <div class="mascot-desc">{{ agent.description }}</div>
          </div>
        </div>
      </div>
    </div>

    <EasterEgg ref="easterEggRef" />
  </div>
</template>

<style scoped>
.agents-page {
  min-height: 100vh;
  padding-bottom: 40px;
}

.agent-section {
  margin-top: 28px;
}

.section-title {
  font-size: 18px;
  font-weight: bold;
  margin-bottom: 16px;
  padding-left: 8px;
  border-left: 4px solid #f5a623;
}

/* ═══ 响应式网格 ═══ */
.agent-grid {
  display: grid;
  grid-template-columns: repeat(var(--cols, 4), 1fr);
  gap: 16px;
}

/* ═══ Agent 卡片 ═══ */
.agent-card {
  background: white;
  border-radius: 20px;
  border: 2px solid #f0ebe3;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: visible;
  cursor: pointer;
}

.agent-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 32px rgba(245, 166, 35, 0.15);
  border-color: #f5a623;
}

/* 猫耳朵 CSS 装饰 */
.card-ears {
  position: absolute;
  top: -12px;
  left: 0;
  right: 0;
  display: flex;
  justify-content: space-between;
  padding: 0 24px;
  pointer-events: none;
  z-index: 2;
}

.card-ear {
  width: 0;
  height: 0;
  border-left: 10px solid transparent;
  border-right: 10px solid transparent;
  border-bottom: 14px solid #f5a623;
  transition: transform 0.3s;
}

.card-ear.left  { transform: rotate(-10deg); }
.card-ear.right { transform: rotate(10deg); }

.agent-card:hover .card-ear.left  { transform: rotate(-20deg) translateY(-2px); }
.agent-card:hover .card-ear.right { transform: rotate(20deg) translateY(-2px); }

/* 卡片内部 */
.card-body {
  padding: 24px 16px 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.compact-body {
  flex-direction: row;
  gap: 12px;
  padding: 16px 12px 8px;
}

.avatar-wrapper {
  position: relative;
}

.agent-info {
  text-align: center;
  width: 100%;
}

.agent-name-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  flex-wrap: wrap;
}

.agent-name {
  font-weight: bold;
  font-size: 16px;
}

.agent-role-tag {
  font-family: monospace;
  font-size: 14px;
  font-weight: 600;
  opacity: 1;
  color: #5a4a3a;
  background: rgba(245, 166, 35, 0.08);
  padding: 1px 8px;
  border-radius: 10px;
}

.agent-desc {
  font-size: 13px;
  opacity: 0.75;
  line-height: 1.5;
  margin-top: 6px;
}

.agent-status {
  margin-top: 8px;
}

.current-task {
  margin-top: 8px;
  font-size: 13px;
  opacity: 0.85;
}

.current-task .task-id {
  color: #f5a623;
  font-weight: bold;
  font-family: monospace;
}

.compact-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.compact-task {
  margin-top: 2px;
}

/* 投喂按钮 */
.card-footer {
  padding: 8px 12px 12px;
  display: flex;
  justify-content: center;
}

.feed-btn {
  background: linear-gradient(135deg, #e8f5e9 0%, #fff8e1 100%);
  border: 1px solid #c8e6c9;
  border-radius: 20px;
  padding: 6px 16px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.feed-btn:hover {
  transform: scale(1.1);
  background: linear-gradient(135deg, #c8e6c9 0%, #ffecb3 100%);
}

.feed-btn:active {
  transform: scale(0.95);
}

.feed-btn.small {
  padding: 4px 10px;
  font-size: 11px;
}

/* 吉祥物卡片 */
.mascot-card {
  background: linear-gradient(135deg, #faf7f2 0%, #fff5f5 100%);
  border-color: #ffcdd2;
}

.mascot-card:hover {
  border-color: #ef9a9a;
}

.mascot-wrapper {
  padding: 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.mascot-name {
  font-size: 18px;
  font-weight: bold;
}

.mascot-desc {
  font-size: 12px;
  opacity: 0.5;
  text-align: center;
}

/* ═══ 响应式断点 ═══ */
@media (max-width: 480px) {
  .agent-grid {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .card-body {
    padding: 20px 12px 8px;
  }

  .agent-name {
    font-size: 14px;
  }

  .agent-desc {
    font-size: 11px;
  }
}

@media (min-width: 481px) and (max-width: 768px) {
  .agent-grid {
    grid-template-columns: repeat(2, 1fr);
    gap: 12px;
  }
}

@media (min-width: 769px) and (max-width: 1024px) {
  .agent-grid {
    grid-template-columns: repeat(3, 1fr);
  }
}
</style>
