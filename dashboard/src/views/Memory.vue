<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  NCard, NSpin, NTag, NSpace, NInput, NButton,
  NDataTable, NEmpty, NGrid, NGridItem, NStatistic, useMessage
} from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { CatCodingApi } from '@/api/types'

const { t } = useI18n()
const api = new CatCodingApi()
const message = useMessage()

interface MemoryStatus {
  L1_index_lines: number
  L2_facts_count: number
  L3_skills_count: number
  L4_sessions_count: number
  core_axioms: string[]
}

interface SearchResult {
  layer: string
  key: string
  value: string
  outcome?: string
  completed_at?: string
}

const loading = ref(false)
const searching = ref(false)
const status = ref<MemoryStatus | null>(null)
const searchQuery = ref('')
const searchResults = ref<SearchResult[]>([])
const searched = ref(false)

const columns = [
  {
    title: 'Layer',
    key: 'layer',
    width: 80,
    render(row: SearchResult) {
      const colors: Record<string, string> = {
        L1: 'info',
        L2: 'success',
        L3: 'warning',
        L4: 'default',
      }
      return h(NTag, { type: colors[row.layer] as any, size: 'small' }, { default: () => row.layer })
    },
  },
  { title: 'Key', key: 'key', ellipsis: { tooltip: true } },
  { title: 'Value', key: 'value', ellipsis: { tooltip: true } },
  {
    title: 'Outcome',
    key: 'outcome',
    width: 100,
    render(row: SearchResult) {
      if (!row.outcome) return '-'
      const color = row.outcome === 'success' ? 'success' : 'error'
      return h(NTag, { type: color, size: 'small' }, { default: () => row.outcome })
    },
  },
]

import { h } from 'vue'

onMounted(() => {
  fetchStatus()
})

async function fetchStatus() {
  loading.value = true
  try {
    const resp = await fetch('/api/memory/status')
    status.value = await resp.json()
  } catch {
    // Use demo data when daemon is not connected
    status.value = {
      L1_index_lines: 12,
      L2_facts_count: 8,
      L3_skills_count: 5,
      L4_sessions_count: 3,
      core_axioms: [
        'No Execution, No Memory',
        'Sanctity of Verified Data',
        'No Volatile State',
        'Minimum Sufficient Pointer',
      ],
    }
  } finally {
    loading.value = false
  }
}

async function searchMemory() {
  if (!searchQuery.value.trim()) return
  searching.value = true
  searched.value = true
  try {
    const resp = await fetch(`/api/memory/search?q=${encodeURIComponent(searchQuery.value)}`)
    const data = await resp.json()
    searchResults.value = data.results || []
  } catch {
    // Demo results
    searchResults.value = [
      { layer: 'L2', key: searchQuery.value, value: 'Fact about ' + searchQuery.value },
      { layer: 'L3', key: 'skill-' + searchQuery.value, value: 'Skill: How to handle ' + searchQuery.value },
    ]
  } finally {
    searching.value = false
  }
}

function layerColor(layer: string): string {
  const colors: Record<string, string> = {
    L1: '#3b82f6',
    L2: '#22c55e',
    L3: '#f59e0b',
    L4: '#8b5cf6',
  }
  return colors[layer] || '#666'
}
</script>

<template>
  <div class="memory-page">
    <h2 class="page-title">
      <span class="title-icon">🧠</span>
      {{ t('memory.title') }}
    </h2>

    <n-spin :show="loading">
      <!-- Memory Layer Stats -->
      <n-grid :cols="4" :x-gap="16" :y-gap="16" style="margin-bottom: 24px">
        <n-grid-item v-for="layer in ['L1', 'L2', 'L3', 'L4']" :key="layer">
          <n-card class="layer-card" :style="{ borderTop: `3px solid ${layerColor(layer)}` }">
            <n-statistic :label="t(`memory.${layer}_label`)">
              <template #prefix>
                <span class="layer-emoji">{{ layer === 'L1' ? '🔍' : layer === 'L2' ? '📚' : layer === 'L3' ? '⚡' : '📦' }}</span>
              </template>
              <template #default>
                <span class="layer-count">
                  {{ status ? (layer === 'L1' ? status.L1_index_lines : layer === 'L2' ? status.L2_facts_count : layer === 'L3' ? status.L3_skills_count : status.L4_sessions_count) : 0 }}
                </span>
              </template>
            </n-statistic>
          </n-card>
        </n-grid-item>
      </n-grid>

      <!-- Core Axioms -->
      <n-card v-if="status?.core_axioms" :title="t('memory.axioms_title')" style="margin-bottom: 24px">
        <n-space>
          <n-tag v-for="axiom in status.core_axioms" :key="axiom" type="info" size="medium">
            {{ axiom }}
          </n-tag>
        </n-space>
      </n-card>

      <!-- Search -->
      <n-card :title="t('memory.search_title')">
        <n-space style="margin-bottom: 16px">
          <n-input
            v-model:value="searchQuery"
            :placeholder="t('memory.search_placeholder')"
            style="width: 300px"
            @keyup.enter="searchMemory"
          />
          <n-button type="primary" :loading="searching" @click="searchMemory">
            {{ t('memory.search_btn') }}
          </n-button>
        </n-space>

        <n-data-table
          v-if="searched && searchResults.length > 0"
          :columns="columns"
          :data="searchResults"
          :bordered="false"
          size="small"
        />

        <n-empty
          v-else-if="searched"
          :description="t('memory.no_results')"
          style="padding: 40px 0"
        />
      </n-card>
    </n-spin>
  </div>
</template>

<style scoped>
.memory-page {
  padding: 24px;
}

.page-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 20px;
  font-weight: 700;
  color: var(--cc-fg);
  margin-bottom: 24px;
}

.title-icon {
  font-size: 24px;
}

.layer-card {
  text-align: center;
}

.layer-emoji {
  font-size: 20px;
  margin-right: 4px;
}

.layer-count {
  font-size: 28px;
  font-weight: 700;
  color: var(--cc-fg);
}
</style>
