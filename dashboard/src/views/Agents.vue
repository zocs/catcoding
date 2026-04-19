<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { NPageHeader, NCard, NTag, NSpace, NButton, NEmpty, useMessage } from 'naive-ui'
import { Agent, CatCodingApi } from '@/api/types'
import CatSprite from '@/components/CatSprite.vue'
import { breedFor, agentStatusToSpriteState } from '@/api/catBreed'
import EasterEgg from '@/components/EasterEgg.vue'
import XpBadge from '@/components/XpBadge.vue'
import { useI18n } from 'vue-i18n'
import { useResponsive } from '@/composables/useResponsive'
import { useAgentRoles, ROLE_META } from '@/composables/useAgentRoles'

const { t } = useI18n()
const { isMobile, gridCols, avatarSize, cardPadding } = useResponsive()
const { getRole } = useAgentRoles()

const api = new CatCodingApi()
const message = useMessage()
const agents = ref<Agent[]>([])
const loading = ref(false)
const easterEggRef = ref<InstanceType<typeof EasterEgg> | null>(null)
const selectedAgent = ref<string | null>(null)
const feedCount = ref<Record<string, number>>({})

// 吉祥物点击追踪
const mascotClicks = ref<Record<string, number>>({})
const mascotLastClick = ref<Record<string, number>>({})
const mascotMood = ref<Record<string, string>>({})
const showParticles = ref<Record<string, boolean>>({})

// 动物适配食物
const ANIMAL_FOOD: Record<string, { emoji: string; name: string }> = {
  mascot:     { emoji: '🎋', name: t('agentsExt.food_bamboo') },
  tech_scout: { emoji: '🫐', name: t('agentsExt.food_berry') },
  watchdog:   { emoji: '🐭', name: t('agentsExt.food_mouse') },
}

function getFoodEmoji(role: string): string {
  return ANIMAL_FOOD[role]?.emoji || '🐟'
}

function getFoodName(role: string): string {
  return ANIMAL_FOOD[role]?.name || t('agentsExt.food_fish')
}

// 吉祥物互动消息
const MASCOT_REACTIONS: Record<number, { msg: string; mood: string; trigger?: string }> = {
  1:  { msg: t('agentsExt.panda_eats'), mood: 'eating' },
  2:  { msg: t('agentsExt.panda_eats2'), mood: 'eating' },
  3:  { msg: t('agentsExt.panda_rolls'), mood: 'rolling' },
  5:  { msg: t('agentsExt.panda_handstand'), mood: 'handstand' },
  7:  { msg: t('agentsExt.panda_sleepy'), mood: 'sleeping' },
  8:  { msg: t('agentsExt.panda_sleeping'), mood: 'sleeping' },
  10: { msg: t('agentsExt.panda_party'), mood: 'party', trigger: 'panda_party' },
  12: { msg: t('agentsExt.panda_magic'), mood: 'magic' },
  15: { msg: t('agentsExt.panda_golden'), mood: 'golden', trigger: 'golden_panda' },
}

onMounted(() => {
  fetchAgents()
})

async function fetchAgents() {
  loading.value = true
  try {
    agents.value = await api.getAgents()
  } catch {
    // Mock: synthesize one of each role so the page still renders offline.
    agents.value = Object.keys(ROLE_META).map((role) => {
      const info = getRole(role)
      return {
        role,
        name: info.name,
        emoji: info.emoji,
        status: ['active', 'idle', 'idle', 'busy', 'active'][Math.floor(Math.random() * 5)] as any,
        description: info.desc,
        mode: info.mode as any,
        current_task: Math.random() > 0.6 ? `task-${Math.floor(Math.random() * 100)}` : null,
      }
    })
  } finally {
    loading.value = false
  }
}

// "投喂"互动
function feedAgent(role: string) {
  feedCount.value[role] = (feedCount.value[role] || 0) + 1
  message.success(t('agentsExt.feedSuccess', { name: getAgentInfo(role).name, count: feedCount.value[role] }))

  // 投喂 5 次触发彩蛋
  if (feedCount.value[role] === 5) {
    easterEggRef.value?.triggerEgg('magic')
  }
  // 所有 agent 都被投喂过
  const allFed = Object.keys(ROLE_META).every(r => (feedCount.value[r] || 0) > 0)
  if (allFed) {
    easterEggRef.value?.triggerEgg('streak_10')
  }
}

// 吉祥物互动 — 渐进式彩蛋
function interactMascot(role: string) {
  const now = Date.now()
  const last = mascotLastClick.value[role] || 0

  // 超过 5 秒没点击则重置计数
  if (now - last > 5000) {
    mascotClicks.value[role] = 0
  }

  mascotClicks.value[role] = (mascotClicks.value[role] || 0) + 1
  mascotLastClick.value[role] = now

  const count = mascotClicks.value[role]
  const reaction = MASCOT_REACTIONS[count]

  if (reaction) {
    mascotMood.value[role] = reaction.mood
    message.info(reaction.msg)

    // 触发 EasterEgg
    if (reaction.trigger) {
      easterEggRef.value?.triggerEgg(reaction.trigger)
    }

    // 粒子爆发
    showParticles.value[role] = true
    setTimeout(() => { showParticles.value[role] = false }, 1500)
  }

  // 超过 15 次循环金熊猫彩蛋
  if (count > 15 && count % 5 === 0) {
    message.success(t('agentsExt.golden_wink'))
    showParticles.value[role] = true
    setTimeout(() => { showParticles.value[role] = false }, 1500)
  }

  // 顺便也算投喂
  feedAgent(role)
}

function getAgentInfo(role: string) {
  return getRole(role)
}

function getStatusLabel(status: string) {
  const map: Record<string, string> = { idle: '😴 ' + t('agentsExt.idle'), active: '🔧 ' + t('agentsExt.working'), busy: '⚡ ' + t('agentsExt.busy'), error: '🙀 ' + t('agentsExt.error') }
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
  <div class="agents-page" :class="{ 'mobile': isMobile }">
    <n-page-header :title="'🐱 ' + t('agents.title')" :subtitle="t('agents.title')">
      <template #extra>
        <n-space>
          <n-button @click="fetchAgents" :loading="loading" round :size="isMobile ? 'small' : 'medium'">
            🔄 {{ t("agentsExt.refresh") }}
          </n-button>
        </n-space>
      </template>
    </n-page-header>

    <!-- 常驻 Agent -->
    <div class="agent-section" v-if="residents.length > 0">
      <div class="section-title">🏠 {{ t("agentsExt.section_resident") }}</div>
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
              <CatSprite
                :breed="breedFor(agent.role)"
                :state="agentStatusToSpriteState(agent.status)"
                :size="avatarSize"
              />
            </div>

            <div class="agent-info">
              <div class="agent-name-row">
                <span class="agent-name">{{ getAgentInfo(agent.role).name }}</span>
                <span class="agent-role-tag">@{{ agent.role }}</span>
              </div>
              <div class="agent-desc" v-if="!isMobile">{{ getAgentInfo(agent.role).desc }}</div>
              <div class="agent-status">
                <n-tag :type="getStatusType(agent.status)" size="small" round>
                  {{ getStatusLabel(agent.status) }}
                </n-tag>
              </div>
              <div v-if="agent.mode !== 'decorative'" class="xp-slot">
                <XpBadge :level="agent.level ?? 1" :xp="agent.xp ?? 0" compact />
              </div>
              <div v-if="agent.current_task" class="current-task">
                🔧 <span class="task-id">#{{ agent.current_task }}</span>
              </div>
            </div>
          </div>

          <!-- 投喂按钮 -->
          <div class="card-footer">
            <button class="feed-btn" @click.stop="feedAgent(agent.role)" :title="t('agentsExt.feed') + ' ' + getFoodName(agent.role)">
              {{ getFoodEmoji(agent.role) }} {{ feedCount[agent.role] || 0 }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 按需 Agent -->
    <div class="agent-section" v-if="onDemand.length > 0">
      <div class="section-title">⚡ {{ t("agentsExt.section_onDemand") }}</div>
      <div class="agent-grid" :style="{ '--cols': gridCols }">
        <div
          v-for="agent in onDemand"
          :key="agent.role"
          class="agent-card"
          :class="[agent.status]"
          @click="selectedAgent = selectedAgent === agent.role ? null : agent.role"
        >
          <div class="card-ears">
            <div class="card-ear left"></div>
            <div class="card-ear right"></div>
          </div>
          <div class="card-body">
            <CatSprite :breed="breedFor(agent.role)" :state="agentStatusToSpriteState(agent.status)" :size="isMobile ? 40 : 64" />
            <div class="agent-info">
              <div class="agent-name-row">
                <span class="agent-name">{{ getAgentInfo(agent.role).name }}</span>
                <span class="agent-role-tag">@{{ agent.role }}</span>
              </div>
              <div class="agent-status">
                <n-tag :type="getStatusType(agent.status)" size="small" round>
                  {{ getStatusLabel(agent.status) }}
                </n-tag>
              </div>
              <div class="xp-slot">
                <XpBadge :level="agent.level ?? 1" :xp="agent.xp ?? 0" compact />
              </div>
              <div v-if="agent.current_task" class="current-task">
                🔧 <span class="task-id">#{{ agent.current_task }}</span>
              </div>
            </div>
          </div>
          <div class="card-footer">
            <button class="feed-btn" @click.stop="feedAgent(agent.role)" :title="t('agentsExt.feed') + ' ' + getFoodName(agent.role)">
              {{ getFoodEmoji(agent.role) }} {{ feedCount[agent.role] || 0 }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- 吉祥物 -->
    <div class="agent-section" v-if="decorative.length > 0">
      <div class="section-title">🎋 {{ t("agentsExt.section_mascot") }}</div>
      <div class="agent-grid mascot-grid" :style="{ '--cols': Math.min(gridCols, 3) }">
        <div
          v-for="agent in decorative"
          :key="agent.role"
          class="agent-card mascot-card"
          @click="feedAgent(agent.role)"
        >
          <div class="mascot-wrapper">
            <CatSprite :breed="breedFor(agent.role)" state="playing" :size="isMobile ? 80 : 120" />
            <div class="mascot-name">{{ getAgentInfo(agent.role).name }}</div>
            <div class="agent-role-tag">@{{ agent.role }}</div>
            <div class="mascot-desc" v-if="!isMobile">{{ getAgentInfo(agent.role).desc }}</div>
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
  padding: 16px;
  padding-bottom: 40px;
}

.agents-page.mobile {
  padding: 8px;
}

.agent-section {
  margin-top: 28px;
}

.mobile .agent-section {
  margin-top: 16px;
}

.section-title {
  font-size: 18px;
  font-weight: bold;
  margin-bottom: 16px;
  padding-left: 8px;
  border-left: 4px solid var(--cc-orange);
}

.mobile .section-title {
  font-size: 15px;
  margin-bottom: 10px;
}

/* ═══ 响应式网格 ═══ */
.agent-grid {
  display: grid;
  grid-template-columns: repeat(var(--cols, 4), 1fr);
  gap: 16px;
}

.mobile .agent-grid {
  gap: 10px;
}

/* ═══ Agent Card ═══ */
.agent-card {
  background: var(--cc-bg-card);
  border-radius: 20px;
  border: 2px solid var(--cc-border);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  position: relative;
  overflow: visible;
  cursor: pointer;
}

.agent-card:hover {
  transform: translateY(-4px);
  box-shadow: var(--cc-shadow-hover);
  border-color: var(--cc-orange);
}

/* 移动端去掉 hover 上浮（触屏无 hover） */
.mobile .agent-card:hover {
  transform: none;
}

.mobile .agent-card:active {
  transform: scale(0.97);
  transition: transform 0.1s;
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

.mobile .card-ears {
  padding: 0 16px;
}

.card-ear {
  width: 0;
  height: 0;
  border-left: 10px solid transparent;
  border-right: 10px solid transparent;
  border-bottom: 14px solid var(--cc-orange);
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

.mobile .card-body {
  padding: 16px 10px 8px;
  gap: 6px;
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

.mobile .agent-name {
  font-size: 14px;
}

.agent-role-tag {
  font-family: monospace;
  font-size: 14px;
  font-weight: 600;
  opacity: 1;
  color: var(--cc-fg);
  background: rgba(249, 115, 22, 0.08);
  padding: 1px 8px;
  border-radius: 10px;
}

.mobile .agent-role-tag {
  font-size: 11px;
  padding: 1px 5px;
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

.mobile .agent-status {
  margin-top: 4px;
}

.current-task {
  margin-top: 8px;
  font-size: 13px;
  opacity: 0.85;
}

.xp-slot {
  margin-top: 8px;
  width: 100%;
}

.mobile .current-task {
  margin-top: 4px;
  font-size: 12px;
}

.current-task .task-id {
  color: var(--cc-orange);
  font-weight: bold;
  font-family: monospace;
}

/* 投喂按钮 */
.card-footer {
  padding: 8px 12px 12px;
  display: flex;
  justify-content: center;
}

.mobile .card-footer {
  padding: 4px 8px 8px;
}

.feed-btn {
  background: var(--cc-bg-input);
  border: 1px solid var(--cc-border);
  border-radius: 20px;
  padding: 6px 16px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.mobile .feed-btn {
  padding: 4px 12px;
  font-size: 12px;
}

.feed-btn:hover {
  transform: scale(1.1);
  background: var(--cc-bg-hover);
}

.feed-btn:active {
  transform: scale(0.95);
}

/* 吉祥物卡片 */
.mascot-card {
  background: var(--cc-bg-card);
  border-color: var(--cc-border);
}

.mascot-card:hover {
  border-color: var(--cc-orange);
}

.mascot-wrapper {
  padding: 24px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.mobile .mascot-wrapper {
  padding: 16px 10px;
  gap: 8px;
}

.mascot-name {
  font-weight: bold;
  font-size: 18px;
}

.mobile .mascot-name {
  font-size: 15px;
}

.mascot-desc {
  font-size: 13px;
  opacity: 0.7;
  text-align: center;
}
</style>
