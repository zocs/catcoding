<template>
  <n-layout has-sider style="position: absolute; inset: 0">
    <n-layout-sider
      bordered
      collapse-mode="width"
      :collapsed-width="64"
      :width="220"
      :collapsed="collapsed"
      show-trigger
      @collapse="collapsed = true"
      @expand="collapsed = false"
      class="dashboard-sider"
    >
      <div class="sidebar-brand">
        <span class="brand-icon">🐱</span>
        <span v-if="!collapsed" class="brand-text">CatCoding</span>
      </div>
      <n-menu
        v-model:value="activeKey"
        :collapsed="collapsed"
        :collapsed-width="64"
        :collapsed-icon-size="22"
        :options="menuOptions"
        @update:value="handleMenuSelect"
      />
      <div class="sidebar-footer" v-if="!collapsed">
        <LangSwitch />
        <ThemeSwitch />
      </div>
    </n-layout-sider>
    <n-layout class="dashboard-main">
      <n-layout-content class="dashboard-content">
        <router-view />
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>

<script setup lang="ts">
import { ref, h, computed, inject, type Ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  NLayout, NLayoutSider, NLayoutContent, NMenu,
  NIcon
} from 'naive-ui'
import type { MenuOption } from 'naive-ui'
import {
  PeopleOutline,
  BarChartOutline,
  TerminalOutline,
  CodeSlashOutline,
  GridOutline
} from '@vicons/ionicons5'
import LangSwitch from '../components/LangSwitch.vue'
import ThemeSwitch from '../components/ThemeSwitch.vue'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const isDark = inject<Ref<boolean>>('isDark')!

const collapsed = ref(false)

function renderIcon(icon: any) {
  return () => h(NIcon, null, { default: () => h(icon) })
}

const menuOptions = computed<MenuOption[]>(() => [
  { label: t('nav.agents'), key: 'agents', icon: renderIcon(PeopleOutline) },
  { label: t('nav.gantt'), key: 'gantt', icon: renderIcon(BarChartOutline) },
  { label: t('nav.terminal'), key: 'terminal', icon: renderIcon(TerminalOutline) },
  { label: t('nav.commands'), key: 'commands', icon: renderIcon(CodeSlashOutline) },
  { label: t('nav.board'), key: 'board', icon: renderIcon(GridOutline) },
])

const activeKey = computed(() => {
  const name = route.name as string
  return name || 'agents'
})

function handleMenuSelect(key: string) {
  router.push({ name: key })
}
</script>

<style>
/* Light theme */
.dashboard-sider {
  background: #fef9f0 !important;
  border-right: 1px solid #f0e6d6 !important;
}
.dashboard-main {
  background: #faf8f5;
}
.dashboard-content {
  background: #faf8f5 !important;
}
.sidebar-brand {
  color: #5a4a3a;
}

/* Dark theme */
[data-theme="dark"] .dashboard-sider {
  background: #1e1e2e !important;
  border-right: 1px solid #313244 !important;
}
[data-theme="dark"] .dashboard-main {
  background: #181825;
}
[data-theme="dark"] .dashboard-content {
  background: #181825 !important;
}
[data-theme="dark"] .sidebar-brand {
  color: #cdd6f4;
}

/* Shared */
.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 20px 16px;
  border-bottom: 1px solid #f0e6d6;
}
[data-theme="dark"] .sidebar-brand {
  border-bottom: 1px solid #313244;
}
.brand-icon {
  font-size: 28px;
}
.brand-text {
  font-size: 18px;
  font-weight: 700;
}
.sidebar-footer {
  position: absolute;
  bottom: 16px;
  left: 16px;
  display: flex;
  gap: 8px;
}
</style>
