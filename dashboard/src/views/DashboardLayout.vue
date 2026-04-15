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
      style="background: #1a1a2e"
    >
      <n-menu
        v-model:value="activeKey"
        :collapsed="collapsed"
        :collapsed-width="64"
        :collapsed-icon-size="22"
        :options="menuOptions"
        @update:value="handleMenuSelect"
      />
    </n-layout-sider>
    <n-layout>
      <n-layout-content style="padding: 24px; background: #0f0f1a">
        <router-view />
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>

<script setup lang="ts">
import { ref, h, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  NLayout, NLayoutSider, NLayoutContent, NMenu,
  NIcon
} from 'naive-ui'
import type { MenuOption } from 'naive-ui'
import {
  HomeOutline,
  PeopleOutline,
  BarChartOutline,
  TerminalOutline,
  CodeSlashOutline,
  PawOutline
} from '@vicons/ionicons5'

const router = useRouter()
const route = useRoute()
const { t } = useI18n()

const collapsed = ref(false)

function renderIcon(icon: any) {
  return () => h(NIcon, null, { default: () => h(icon) })
}

const menuOptions = computed<MenuOption[]>(() => [
  { label: t('nav.home'), key: 'home', icon: renderIcon(HomeOutline) },
  { label: t('nav.agents'), key: 'agents', icon: renderIcon(PeopleOutline) },
  { label: t('nav.gantt'), key: 'gantt', icon: renderIcon(BarChartOutline) },
  { label: t('nav.terminal'), key: 'terminal', icon: renderIcon(TerminalOutline) },
  { label: t('nav.commands'), key: 'commands', icon: renderIcon(CodeSlashOutline) },
  { label: t('nav.cat'), key: 'cat', icon: renderIcon(PawOutline) },
])

const activeKey = computed(() => {
  const name = route.name as string
  if (!name) return 'home'
  return name
})

function handleMenuSelect(key: string) {
  router.push({ name: key })
}
</script>
