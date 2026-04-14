<template>
  <n-config-provider :theme="darkTheme">
    <n-message-provider>
      <n-layout has-sider style="height: 100vh">
        <!-- 侧边栏 -->
        <n-layout-sider
          bordered
          :collapsed-width="64"
          :width="240"
          collapse-mode="width"
          show-trigger
        >
          <div class="logo">
            <span class="logo-icon">🐱</span>
            <span v-if="!collapsed" class="logo-text">CatCoding</span>
          </div>
          <n-menu
            v-model:value="activeKey"
            :collapsed="collapsed"
            :collapsed-width="64"
            :collapsed-icon-size="22"
            :options="menuOptions"
            @update:value="handleMenuSelect"
          />
        </n-layout-sider>

        <!-- 主内容区 -->
        <n-layout>
          <n-layout-header bordered style="height: 64px; padding: 0 24px">
            <div class="header-content">
              <n-breadcrumb>
                <n-breadcrumb-item>🐱 CatCoding</n-breadcrumb-item>
                <n-breadcrumb-item>{{ currentPageTitle }}</n-breadcrumb-item>
              </n-breadcrumb>
              <div class="header-right">
                <n-tag type="success" size="small">
                  <template #icon>
                    <n-icon><span>🟢</span></n-icon>
                  </template>
                  Daemon 运行中
                </n-tag>
              </div>
            </div>
          </n-layout-header>

          <n-layout-content content-style="padding: 24px;">
            <router-view />
          </n-layout-content>
        </n-layout>
      </n-layout>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { ref, computed, h } from 'vue'
import { useRouter } from 'vue-router'
import { darkTheme } from 'naive-ui'
import type { MenuOption } from 'naive-ui'

const router = useRouter()
const collapsed = ref(false)
const activeKey = ref('board')

const currentPageTitle = computed(() => {
  const titles: Record<string, string> = {
    board: '看板',
    gantt: '甘特图',
    agents: '猫咪面板',
    logs: '厨房日志',
    command: '指令输入',
  }
  return titles[activeKey.value] || '看板'
})

const menuOptions: MenuOption[] = [
  {
    label: '看板视图',
    key: 'board',
    icon: () => h('span', '📋'),
  },
  {
    label: '甘特图',
    key: 'gantt',
    icon: () => h('span', '📊'),
  },
  {
    label: '猫咪面板',
    key: 'agents',
    icon: () => h('span', '🐱'),
  },
  {
    label: '厨房日志',
    key: 'logs',
    icon: () => h('span', '🍳'),
  },
  {
    label: '指令输入',
    key: 'command',
    icon: () => h('span', '💬'),
  },
]

function handleMenuSelect(key: string) {
  activeKey.value = key
  router.push(`/${key}`)
}
</script>

<style scoped>
.logo {
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 18px;
  font-weight: bold;
}

.logo-icon {
  font-size: 28px;
}

.logo-text {
  color: #e94560;
}

.header-content {
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 16px;
}
</style>
