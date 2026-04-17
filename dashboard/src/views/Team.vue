<script setup lang="ts">
// Team.vue — 🐾 团队视图：按项目分组查看 Agent + 等级
import { ref, onMounted, computed } from 'vue'
import { NPageHeader, NCard, NButton, NEmpty, NTag, useMessage } from 'naive-ui'
import { Agent, CatCodingApi } from '@/api/types'
import CatAvatar from '@/components/CatAvatar.vue'
import XpBadge from '@/components/XpBadge.vue'
import { useI18n } from 'vue-i18n'
import { useResponsive } from '@/composables/useResponsive'
import { useAgentRoles } from '@/composables/useAgentRoles'

const { t } = useI18n()
const { isMobile } = useResponsive()
const { getRole } = useAgentRoles()
const api = new CatCodingApi()
const message = useMessage()
const agents = ref<Agent[]>([])
const loading = ref(false)

async function fetchAgents() {
  loading.value = true
  try {
    agents.value = await api.getAgents()
  } catch {
    agents.value = []
    message.warning(t('command.failed'))
  } finally {
    loading.value = false
  }
}

const groups = computed(() => {
  const m = new Map<string, Agent[]>()
  for (const a of agents.value) {
    const list = m.get(a.role) || []
    list.push(a)
    m.set(a.role, list)
  }
  return Array.from(m.entries()).sort((a, b) => a[0].localeCompare(b[0]))
})

onMounted(fetchAgents)
</script>

<template>
  <div class="team-page" :class="{ mobile: isMobile }">
    <n-page-header :title="'🐾 ' + t('team.title')"
                   :subtitle="t('team.subtitle')">
      <template #extra>
        <n-button @click="fetchAgents" :loading="loading" round :size="isMobile ? 'small' : 'medium'">
          🔄 {{ t('gantt.refresh') }}
        </n-button>
      </template>
    </n-page-header>

    <n-empty v-if="agents.length === 0" :description="t('team.empty')" />

    <div v-else class="team-groups">
      <n-card
        v-for="[role, list] in groups"
        :key="role"
        :title="`${getRole(role).emoji} ${getRole(role).name} (${list.length})`"
        :bordered="false"
        class="team-card"
      >
        <div class="members">
          <div v-for="agent in list" :key="agent.id ?? agent.role" class="member">
            <CatAvatar :emoji="getRole(role).emoji" :name="agent.id ?? agent.role" :status="(agent.status as any) || 'idle'" size="small" />
            <div class="member-info">
              <div class="member-id">{{ (agent.id ?? agent.role).slice(0, 14) }}</div>
              <XpBadge :level="agent.level ?? 1" :xp="agent.xp ?? 0" />
              <div v-if="agent.current_task" class="current">🔧 #{{ agent.current_task }}</div>
              <n-tag v-if="(agent.restart_count ?? 0) > 0" size="tiny" type="warning" style="margin-top: 4px">
                {{ t('team.restart_count', { n: agent.restart_count }) }}
              </n-tag>
            </div>
          </div>
        </div>
      </n-card>
    </div>
  </div>
</template>

<style scoped>
.team-page { min-height: 100vh; padding: 16px; }
.team-page.mobile { padding: 8px; }
.team-groups { display: grid; grid-template-columns: repeat(auto-fill, minmax(360px, 1fr)); gap: 16px; margin-top: 16px; }
.team-card { border-radius: 16px; }
.members { display: flex; flex-direction: column; gap: 12px; }
.member { display: flex; gap: 12px; align-items: center; padding: 8px; border-radius: 8px; background: var(--cc-bg); }
.member-info { flex: 1; min-width: 0; }
.member-id { font-family: monospace; font-size: 12px; opacity: 0.75; margin-bottom: 4px; }
.current { font-size: 11px; color: var(--cc-orange); margin-top: 4px; }
</style>
