<script setup lang="ts">
import { inject, type Ref } from 'vue'
import { NButtonGroup, NButton, NPopover, NSpace } from 'naive-ui'
import { PartlySunnyOutline, MoonOutline, DesktopOutline } from '@vicons/ionicons5'

type ThemeMode = 'light' | 'dark' | 'system'

const themeMode = inject<Ref<ThemeMode>>('themeMode')!
const setThemeMode = inject<(m: ThemeMode) => void>('setThemeMode')!

const icons = { light: PartlySunnyOutline, dark: MoonOutline, system: DesktopOutline }
const labels = { light: 'Light', dark: 'Dark', system: 'System' }
</script>

<template>
  <n-popover trigger="click" placement="top" :show-arrow="false">
    <template #trigger>
      <n-button quaternary circle size="small" :title="`Theme: ${labels[themeMode]}`">
        <template #icon>
          <component :is="icons[themeMode]" />
        </template>
      </n-button>
    </template>
    <n-space vertical :size="4">
      <n-button
        v-for="t in (['light', 'dark', 'system'] as ThemeMode[])"
        :key="t"
        block
        :type="themeMode === t ? 'primary' : 'default'"
        :quaternary="themeMode !== t"
        size="small"
        @click="setThemeMode(t)"
      >
        <template #icon>
          <component :is="icons[t]" />
        </template>
        {{ labels[t] }}
      </n-button>
    </n-space>
  </n-popover>
</template>
