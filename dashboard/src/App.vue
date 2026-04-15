<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'
import { NConfigProvider, NLayout, NLayoutSider, NLayoutHeader, NLayoutContent, NMenu, NMessageProvider, NButton, NDrawer, NDrawerContent, darkTheme } from 'naive-ui'
import { h, Component } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'
import { DashboardOutlined, TableOutlined, TeamOutlined, BarChartOutlined, CodeOutlined, MenuOutlined } from '@vicons/antd'
import { NIcon } from 'naive-ui'
import ThemeSwitch from './components/ThemeSwitch.vue'
import LangSwitch from './components/LangSwitch.vue'
import { useI18n } from 'vue-i18n'
import './theme.css'

const { t } = useI18n()

function renderIcon(icon: Component) {
  return () => h(NIcon, null, { default: () => h(icon) })
}

const route = useRoute()
const windowWidth = ref(window.innerWidth)
const showMobileMenu = ref(false)
const isDark = ref(false)

const isMobile = computed(() => windowWidth.value < 768)
const isTablet = computed(() => windowWidth.value >= 768 && windowWidth.value < 1024)
const collapsed = ref(false)

function onResize() {
  windowWidth.value = window.innerWidth
  if (isMobile.value) collapsed.value = true
}

onMounted(() => {
  window.addEventListener('resize', onResize)
  onResize()
  
  // Listen for theme changes
  window.addEventListener('theme-change', ((e: CustomEvent) => {
    isDark.value = e.detail === 'dark'
  }) as EventListener)
  
  // Initial detection
  const saved = localStorage.getItem('catcoding-theme')
  if (saved === 'dark') isDark.value = true
  else if (saved === 'system') {
    isDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches
  }
})

onUnmounted(() => {
  window.removeEventListener('resize', onResize)
})

const menuOptions = computed(() => [
  { label: () => h(RouterLink, { to: '/dashboard' }, { default: () => t('nav.dashboard') }), key: 'dashboard', icon: renderIcon(DashboardOutlined) },
  { label: () => h(RouterLink, { to: '/board' }, { default: () => t('nav.board') }), key: 'board', icon: renderIcon(TableOutlined) },
  { label: () => h(RouterLink, { to: '/agents' }, { default: () => t('nav.agents') }), key: 'agents', icon: renderIcon(TeamOutlined) },
  { label: () => h(RouterLink, { to: '/gantt' }, { default: () => '甘特图 / Gantt' }), key: 'gantt', icon: renderIcon(BarChartOutlined) },
  { label: () => h(RouterLink, { to: '/command' }, { default: () => t('nav.command') }), key: 'command', icon: renderIcon(CodeOutlined) },
])

const lightThemeOverrides = {
  common: {
    primaryColor: '#f5a623',
    primaryColorHover: '#ffb347',
    primaryColorPressed: '#d48806',
    borderRadius: '12px',
    fontFamily: "'Noto Sans SC', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
  }
}

const darkThemeOverrides = {
  common: {
    primaryColor: '#e0af68',
    primaryColorHover: '#f0c68a',
    primaryColorPressed: '#c49545',
    borderRadius: '12px',
    fontFamily: "'Noto Sans SC', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
    bodyColor: '#0f111a',
    cardColor: '#1a1b2e',
    modalColor: '#1a1b2e',
    popoverColor: '#24273a',
    tableColor: '#1a1b2e',
    inputColor: '#1e2030',
    borderColor: '#3b3f52',
    dividerColor: '#3b3f52',
    hoverColor: '#2a2d42',
  }
}

const currentTheme = computed(() => isDark.value ? darkTheme : null)
const currentOverrides = computed(() => isDark.value ? darkThemeOverrides : lightThemeOverrides)

function navigateTo() {
  showMobileMenu.value = false
}
</script>

<template>
  <n-config-provider :theme="currentTheme" :theme-overrides="currentOverrides">
    <n-message-provider>
      <!-- ═══ 桌面布局 (>= 768px) ═══ -->
      <n-layout v-if="!isMobile" has-sider position="absolute">
        <n-layout-sider
          bordered
          collapse-mode="width"
          :collapsed-width="64"
          :width="200"
          :collapsed="collapsed"
          show-trigger
          :breakpoint="isTablet ? 'md' : undefined"
          @update:collapsed="collapsed = $event"
          class="sider"
        >
          <div class="logo" :class="{ collapsed }">
            <span class="logo-emoji">🐱</span>
            <span v-if="!collapsed" class="logo-text">CatCoding</span>
          </div>
          <n-menu
            :options="menuOptions"
            :collapsed="collapsed"
            :collapsed-width="64"
            :collapsed-icon-size="22"
            :indent="24"
          />
          <div v-if="!collapsed" class="sider-footer">
            <LangSwitch />
            <ThemeSwitch />
          </div>
        </n-layout-sider>
        <n-layout>
          <n-layout-header bordered class="header">
            <div class="header-content">
              <span class="header-title">
                🐱 CatCoding <span class="version">v0.1.0</span>
              </span>
              <div class="header-actions">
                <span class="header-motto">{{ isDark ? '🌙 Night coding mode' : '☀️ Happy coding!' }}</span>
                <ThemeSwitch v-if="collapsed" />
                <LangSwitch v-if="collapsed" />
              </div>
            </div>
          </n-layout-header>
          <n-layout-content class="content">
            <router-view />
          </n-layout-content>
        </n-layout>
      </n-layout>

      <!-- ═══ 移动布局 (< 768px) ═══ -->
      <n-layout v-else position="absolute">
        <n-layout-header bordered class="mobile-header">
          <div class="mobile-header-content">
            <span class="logo-emoji">🐱</span>
            <span class="header-title">CatCoding</span>
            <div class="mobile-actions">
              <ThemeSwitch />
              <n-button quaternary circle @click="showMobileMenu = true">
                <template #icon><MenuOutlined /></template>
              </n-button>
            </div>
          </div>
        </n-layout-header>
        <n-layout-content class="mobile-content">
          <router-view />
        </n-layout-content>
      </n-layout>

      <!-- 移动端侧拉菜单 -->
      <n-drawer v-model:show="showMobileMenu" :width="240" placement="left">
        <n-drawer-content title="🐱 CatCoding" :native-scrollbar="false">
          <n-menu
            :options="menuOptions"
            :indent="16"
            @update:value="navigateTo"
          />
          <div style="padding: 16px; display: flex; gap: 8px;">
            <LangSwitch />
          </div>
        </n-drawer-content>
      </n-drawer>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
/* ═══ 全局重置 ═══ */
* { margin: 0; padding: 0; box-sizing: border-box; }

html, body, #app {
  height: 100%;
  font-family: 'Noto Sans SC', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}
</style>

<style scoped>
/* ═══ 侧边栏 ═══ */
.sider {
  background: var(--cat-sider-bg) !important;
  transition: background 0.3s;
}

.logo {
  height: 64px;
  display: flex;
  align-items: center;
  padding: 0 20px;
  gap: 10px;
  overflow: hidden;
}

.logo.collapsed {
  justify-content: center;
  padding: 0;
}

.logo-emoji {
  font-size: 28px;
  animation: catWave 3s ease-in-out infinite;
}

@keyframes catWave {
  0%, 100% { transform: rotate(0deg); }
  25%      { transform: rotate(10deg); }
  75%      { transform: rotate(-10deg); }
}

.logo-text {
  font-size: 18px;
  font-weight: bold;
  color: var(--cat-primary);
  white-space: nowrap;
}

.sider-footer {
  padding: 12px 16px;
  display: flex;
  gap: 8px;
  align-items: center;
  justify-content: center;
  border-top: 1px solid var(--cat-border);
  margin-top: auto;
  position: absolute;
  bottom: 0;
  width: 100%;
  background: inherit;
}

/* ═══ 桌面头部 ═══ */
.header {
  height: 56px;
  display: flex;
  align-items: center;
  padding: 0 24px;
  background: var(--cat-header-bg);
  backdrop-filter: blur(8px);
  transition: background 0.3s;
}

.header-content {
  display: flex;
  align-items: center;
  gap: 16px;
  width: 100%;
}

.header-title {
  font-weight: 600;
  font-size: 15px;
  color: var(--cat-text);
}

.version {
  font-size: 11px;
  opacity: 0.4;
  font-weight: normal;
}

.header-actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 8px;
}

.header-motto {
  font-size: 12px;
  opacity: 0.5;
  color: var(--cat-text-muted);
}

/* ═══ 内容区 ═══ */
.content {
  padding: 20px;
  background: var(--cat-bg);
  min-height: calc(100vh - 56px);
  transition: background 0.3s;
}

/* ═══ 移动头部 ═══ */
.mobile-header {
  height: 56px;
  background: var(--cat-header-bg);
  backdrop-filter: blur(8px);
}

.mobile-header-content {
  display: flex;
  align-items: center;
  padding: 0 16px;
  height: 100%;
  gap: 10px;
}

.mobile-header-content .header-title {
  flex: 1;
  font-weight: bold;
}

.mobile-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.mobile-content {
  padding: 12px;
  background: var(--cat-bg);
  min-height: calc(100vh - 56px);
}

/* ═══ 响应式 ═══ */
@media (max-width: 480px) {
  .content, .mobile-content { padding: 8px; }
}
@media (min-width: 481px) and (max-width: 768px) {
  .content, .mobile-content { padding: 12px; }
}
@media (min-width: 769px) and (max-width: 1024px) {
  .content { padding: 16px; }
}
</style>
