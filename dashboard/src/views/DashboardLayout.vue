<template>
  <n-layout has-sider style="position: absolute; inset: 0">
    <!-- Mobile overlay -->
    <div
      v-if="isMobile && !collapsed"
      class="sider-overlay"
      @click="collapsed = true"
    />

    <n-layout-sider
      bordered
      collapse-mode="width"
      :collapsed-width="collapsedWidth"
      :width="sidebarWidth"
      :collapsed="collapsed"
      :show-trigger="!isMobile"
      :native-scrollbar="false"
      :position="isMobile ? 'absolute' : 'static'"
      :style="isMobile ? { zIndex: 100, height: '100%' } : {}"
      class="dashboard-sider"
      @collapse="collapsed = true"
      @expand="collapsed = false"
    >
      <div class="sidebar-brand">
        <span class="brand-icon">🐱</span>
        <span v-if="!collapsed || isMobile" class="brand-text">CatCoding</span>
        <n-button
          v-if="isMobile && !collapsed"
          quaternary
          circle
          size="small"
          class="mobile-close"
          @click="collapsed = true"
        >
          ✕
        </n-button>
      </div>

      <n-menu
        v-model:value="activeKey"
        :collapsed="!isMobile && collapsed"
        :collapsed-width="collapsedWidth"
        :collapsed-icon-size="22"
        :options="menuOptions"
        @update:value="handleMenuSelect"
      />

      <!-- Bottom controls -->
      <div class="sidebar-footer" v-if="!collapsed || isMobile">
        <LangSwitch />
        <ThemeSwitch />
      </div>
    </n-layout-sider>

    <n-layout class="dashboard-main">
      <!-- Mobile top bar -->
      <div v-if="isMobile" class="mobile-topbar">
        <n-button quaternary circle size="small" @click="collapsed = false">
          <span style="font-size: 20px">☰</span>
        </n-button>
        <span class="topbar-title">🐱 {{ currentPageTitle }}</span>
      </div>

      <n-layout-content
        class="dashboard-content"
        :style="isMobile ? { paddingTop: '0' } : {}"
      >
        <router-view v-slot="{ Component }">
          <keep-alive>
            <component :is="Component" />
          </keep-alive>
        </router-view>
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>

<script setup lang="ts">
import { ref, h, computed, inject, watch, type Ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  NLayout, NLayoutSider, NLayoutContent, NMenu,
  NIcon, NButton
} from 'naive-ui'
import type { MenuOption } from 'naive-ui'
import {
  PeopleOutline,
  BarChartOutline,
  TerminalOutline,
  CodeSlashOutline,
  GridOutline,
  DocumentTextOutline,
  HardwareChipOutline
} from '@vicons/ionicons5'
import LangSwitch from '../components/LangSwitch.vue'
import ThemeSwitch from '../components/ThemeSwitch.vue'
import { useResponsive } from '../composables/useResponsive'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()
const isDark = inject<Ref<boolean>>('isDark')!

const {
  isMobile,
  isTablet,
  sidebarDefaultCollapsed,
  sidebarWidth,
  collapsedWidth,
} = useResponsive()

const collapsed = ref(sidebarDefaultCollapsed.value)

watch(sidebarDefaultCollapsed, (val) => {
  collapsed.value = val
})

function renderIcon(icon: any) {
  return () => h(NIcon, null, { default: () => h(icon) })
}

const menuOptions = computed<MenuOption[]>(() => [
  { label: t('nav.agents'), key: 'agents', icon: renderIcon(PeopleOutline) },
  { label: t('nav.gantt'), key: 'gantt', icon: renderIcon(BarChartOutline) },
  { label: t('nav.terminal'), key: 'terminal', icon: renderIcon(TerminalOutline) },
  { label: t('nav.commands'), key: 'commands', icon: renderIcon(CodeSlashOutline) },
  { label: t('nav.memory'), key: 'memory', icon: renderIcon(HardwareChipOutline) },
  { label: t('nav.logs'), key: 'logs', icon: renderIcon(DocumentTextOutline) },
  { label: t('nav.board'), key: 'board', icon: renderIcon(GridOutline) },
])

const activeKey = computed(() => {
  const name = route.name as string
  return name || 'agents'
})

const currentPageTitle = computed(() => {
  const option = menuOptions.value.find(o => o.key === activeKey.value)
  return (option?.label as string) || 'CatCoding'
})

function handleMenuSelect(key: string) {
  router.push({ name: key })
  if (isMobile.value) {
    collapsed.value = true
  }
}
</script>

<style>
/* ═══ Sider ═══ */
.dashboard-sider {
  background: var(--cc-bg-sider) !important;
  border-right: 1px solid var(--cc-border) !important;
  transition: width var(--cc-duration) var(--cc-ease),
              left var(--cc-duration) var(--cc-ease) !important;
}

.dashboard-main {
  background: var(--cc-bg);
}

.dashboard-content {
  background: var(--cc-bg) !important;
}

/* ═══ Brand ═══ */
.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 20px 16px;
  border-bottom: 1px solid var(--cc-border);
  position: relative;
}

.brand-icon {
  font-size: 28px;
}

.brand-text {
  font-size: 18px;
  font-weight: 700;
  color: var(--cc-fg);
}

.mobile-close {
  position: absolute;
  right: 12px;
  top: 50%;
  transform: translateY(-50%);
}

/* ═══ Footer Controls ═══ */
.sidebar-footer {
  position: absolute;
  bottom: 16px;
  left: 16px;
  display: flex;
  gap: 8px;
}

/* ═══ Mobile ═══ */
.sider-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.45);
  z-index: 99;
  animation: cc-fadeIn var(--cc-duration) var(--cc-ease);
}

@keyframes cc-fadeIn {
  from { opacity: 0; }
  to   { opacity: 1; }
}

.mobile-topbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: var(--cc-bg-sider);
  border-bottom: 1px solid var(--cc-border);
  min-height: 48px;
  position: sticky;
  top: 0;
  z-index: 50;
}

.topbar-title {
  font-size: 16px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  color: var(--cc-fg);
}

/* ═══ Responsive ═══ */
@media (max-width: 767px) {
  .dashboard-sider .n-layout-sider__border {
    display: none;
  }
}
</style>
