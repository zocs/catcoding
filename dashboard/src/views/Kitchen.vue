<script setup lang="ts">
// Kitchen.vue — 🐭 厨房视图：Bug (老鼠) 处理流程
// 依据计划书 §11.2 烹饪老鼠比喻：将失败/阻塞的任务按 Bug 等级 (🐭 小/🐀 大/🦇 蝙蝠/🐉 恶龙) 归类展示。
import { ref, onMounted, computed } from 'vue'
import { NPageHeader, NCard, NEmpty, NSpace, NTag, NButton, useMessage } from 'naive-ui'
import { Task, CatCodingApi } from '@/api/types'
import { useI18n } from 'vue-i18n'
import { useResponsive } from '@/composables/useResponsive'

const { t } = useI18n()
const { isMobile } = useResponsive()
const api = new CatCodingApi()
const message = useMessage()

const tasks = ref<Task[]>([])
const loading = ref(false)

type BugTier = 'small' | 'big' | 'bat' | 'dragon'

async function fetchTasks() {
  loading.value = true
  try {
    tasks.value = await api.getTasks()
  } catch {
    tasks.value = []
    message.warning(t('command.failed'))
  } finally {
    loading.value = false
  }
}

// Bug 分级：按描述关键词粗分
function classifyBug(task: Task): BugTier {
  const desc = (task.description || '').toLowerCase()
  const title = (task.title || '').toLowerCase()
  if (desc.includes('arch') || title.includes('refactor') || title.includes('架构')) return 'dragon'
  if (desc.includes('race') || desc.includes('timing') || desc.includes('环境')) return 'bat'
  if (task.status === 'failed') return 'big'
  return 'small'
}

const buckets = computed(() => {
  const bugs = tasks.value.filter(t => t.status === 'failed' || t.status === 'blocked' || t.status === 'rollbacked')
  return {
    small:  bugs.filter(t => classifyBug(t) === 'small'),
    big:    bugs.filter(t => classifyBug(t) === 'big'),
    bat:    bugs.filter(t => classifyBug(t) === 'bat'),
    dragon: bugs.filter(t => classifyBug(t) === 'dragon'),
  }
})

const total = computed(() => Object.values(buckets.value).reduce((s, arr) => s + arr.length, 0))

function tierTitle(tier: BugTier): string {
  return t(`kitchen.tier_${tier}`)
}
function tierChipLabel(tier: BugTier, count: number): string {
  return `${t(`kitchen.tier_${tier}_label`)} ${count}`
}

onMounted(fetchTasks)
</script>

<template>
  <div class="kitchen-page" :class="{ mobile: isMobile }">
    <n-page-header :title="'🍳 ' + t('kitchen.title')"
                   :subtitle="t('kitchen.subtitle')">
      <template #extra>
        <n-button @click="fetchTasks" :loading="loading" round :size="isMobile ? 'small' : 'medium'">
          🔄 {{ t('gantt.refresh') }}
        </n-button>
      </template>
    </n-page-header>

    <n-space align="center" size="small" style="margin: 16px 0">
      <n-tag :bordered="false" type="info" size="large">{{ t('kitchen.total', { n: total }) }}</n-tag>
      <n-tag :bordered="false" size="medium">{{ tierChipLabel('small', buckets.small.length) }}</n-tag>
      <n-tag :bordered="false" size="medium" type="warning">{{ tierChipLabel('big', buckets.big.length) }}</n-tag>
      <n-tag :bordered="false" size="medium" type="warning">{{ tierChipLabel('bat', buckets.bat.length) }}</n-tag>
      <n-tag :bordered="false" size="medium" type="error">{{ tierChipLabel('dragon', buckets.dragon.length) }}</n-tag>
    </n-space>

    <div v-if="total === 0">
      <n-empty :description="t('kitchen.empty')" />
    </div>

    <div v-else class="bug-sections">
      <n-card
        v-for="tier in (['dragon','bat','big','small'] as BugTier[])"
        :key="tier"
        v-show="buckets[tier].length > 0"
        :title="tierTitle(tier)"
        :bordered="false"
        class="bug-card"
      >
        <div class="bug-list">
          <div v-for="task in buckets[tier]" :key="task.id" class="bug-item">
            <div class="bug-title">{{ task.title }}</div>
            <div class="bug-meta">
              <span class="bug-id">#{{ task.id.slice(0,8) }}</span>
              <n-tag size="small" :type="task.status === 'failed' ? 'error' : 'warning'" round>{{ task.status }}</n-tag>
              <span v-if="task.assigned_to" class="bug-agent">@{{ task.assigned_to }}</span>
            </div>
            <div class="bug-desc" v-if="task.description">{{ task.description }}</div>
          </div>
        </div>
      </n-card>
    </div>
  </div>
</template>

<style scoped>
.kitchen-page { min-height: 100vh; padding: 16px; color: var(--cc-fg); }
.kitchen-page.mobile { padding: 8px; }
.bug-sections { display: flex; flex-direction: column; gap: 16px; margin-top: 16px; }
.bug-card { border-radius: 16px; }
.bug-list { display: flex; flex-direction: column; gap: 12px; }
.bug-item { padding: 12px; border: 1px dashed var(--cc-border); border-radius: 10px; }
.bug-title { font-weight: 600; margin-bottom: 6px; }
.bug-meta { display: flex; gap: 8px; align-items: center; font-size: 12px; opacity: 0.8; }
.bug-id { font-family: monospace; opacity: 0.6; }
.bug-agent { color: var(--cc-orange); font-weight: 500; }
.bug-desc { margin-top: 6px; font-size: 13px; opacity: 0.75; line-height: 1.5; }
</style>
