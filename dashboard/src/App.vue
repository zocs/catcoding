<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { NConfigProvider, NLayout, NLayoutSider, NLayoutHeader, NLayoutContent, NMenu, NMessageProvider, NButton, NSpace, NDrawer, NDrawerContent } from 'naive-ui'
import { h, Component } from 'vue'
import { RouterLink, RouterView, useRoute } from 'vue-router'
import { DashboardOutlined, TableOutlined, TeamOutlined, BarChartOutlined, CodeOutlined, MenuOutlined } from '@vicons/antd'
import { NIcon } from 'naive-ui'

function renderIcon(icon: Component) {
  return () => h(NIcon, null, { default: () => h(icon) })
}

const route = useRoute()
const windowWidth = ref(window.innerWidth)
const showMobileMenu = ref(false)

const isMobile = computed(() => windowWidth.value < 768)
const isTablet = computed(() => windowWidth.value >= 768 && windowWidth.value < 1024)
const collapsed = ref(false)

function onResize() {
  windowWidth.value = window.innerWidth
  if (isMobile.value) {
    collapsed.value = true
  }
}

onMounted(() => {
  window.addEventListener('resize', onResize)
  onResize()
})

onUnmounted(() => {
  window.removeEventListener('resize', onResize)
})

const menuOptions = [
  {
    label: () => h(RouterLink, { to: '/dashboard' }, { default: () => '首页' }),
    key: 'dashboard',
    icon: renderIcon(DashboardOutlined)
  },
  {
    label: () => h(RouterLink, { to: '/board' }, { default: () => '看板' }),
    key: 'board',
    icon: renderIcon(TableOutlined)
  },
  {
    label: () => h(RouterLink, { to: '/agents' }, { default: () => '猫咪' }),
    key: 'agents',
    icon: renderIcon(TeamOutlined)
  },
  {
    label: () => h(RouterLink, { to: '/gantt' }, { default: () => '甘特图' }),
    key: 'gantt',
    icon: renderIcon(BarChartOutlined)
  },
  {
    label: () => h(RouterLink, { to: '/command' }, { default: () => '终端' }),
    key: 'command',
    icon: renderIcon(CodeOutlined)
  }
]

const themeOverrides = {
  common: {
    primaryColor: '#f5a623',
    primaryColorHover: '#ffb347',
    primaryColorPressed: '#d48806',
    borderRadius: '12px',
    fontFamily: "'Noto Sans SC', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif",
  }
}

function navigateTo(path: string) {
  showMobileMenu.value = false
}
</script>

<template>
  <n-config-provider :theme-overrides="themeOverrides">
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
        </n-layout-sider>
        <n-layout>
          <n-layout-header bordered class="header">
            <div class="header-content">
              <span class="header-title">
                🐱 CatCoding <span class="version">v0.1.0</span>
              </span>
              <span class="header-motto">让 AI 像猫咪团队一样协作做菜！</span>
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
            <n-button quaternary circle @click="showMobileMenu = true">
              <template #icon><MenuOutlined /></template>
            </n-button>
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
        </n-drawer-content>
      </n-drawer>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
/* ═══ 全局重置 ═══ */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  height: 100%;
  font-family: 'Noto Sans SC', -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

/* ═══ 全局滚动条美化 ═══ */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}

::-webkit-scrollbar-thumb {
  background: #d4c5a9;
  border-radius: 3px;
}

::-webkit-scrollbar-track {
  background: transparent;
}
</style>

<style scoped>
/* ═══ 侧边栏 ═══ */
.sider {
  background: linear-gradient(180deg, #faf7f2 0%, #f5f0e8 100%) !important;
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
  color: #f5a623;
  white-space: nowrap;
}

/* ═══ 桌面头部 ═══ */
.header {
  height: 56px;
  display: flex;
  align-items: center;
  padding: 0 24px;
  background: rgba(255, 255, 255, 0.9);
  backdrop-filter: blur(8px);
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
}

.version {
  font-size: 11px;
  opacity: 0.4;
  font-weight: normal;
}

.header-motto {
  font-size: 12px;
  opacity: 0.5;
  margin-left: auto;
}

/* ═══ 内容区 ═══ */
.content {
  padding: 20px;
  background: #faf8f5;
  min-height: calc(100vh - 56px);
}

/* ═══ 移动头部 ═══ */
.mobile-header {
  height: 56px;
  background: rgba(255, 255, 255, 0.95);
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

.mobile-content {
  padding: 12px;
  background: #faf8f5;
  min-height: calc(100vh - 56px);
}

/* ═══ 响应式内容区 ═══ */
@media (max-width: 480px) {
  .content, .mobile-content {
    padding: 8px;
  }
}

@media (min-width: 481px) and (max-width: 768px) {
  .content, .mobile-content {
    padding: 12px;
  }
}

@media (min-width: 769px) and (max-width: 1024px) {
  .content {
    padding: 16px;
  }
}
</style>
