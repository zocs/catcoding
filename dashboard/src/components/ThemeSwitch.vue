<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import { NButtonGroup, NButton, NPopover, NSpace } from 'naive-ui'
import { PartlySunnyOutline, MoonOutline, DesktopOutline } from '@vicons/ionicons5'

type Theme = 'light' | 'dark' | 'system'

const theme = ref<Theme>((localStorage.getItem('catcoding-theme') as Theme) || 'system')
const actualTheme = ref<'light' | 'dark'>('light')

function detectSystemTheme(): 'light' | 'dark' {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

function applyTheme(t: Theme) {
  const actual = t === 'system' ? detectSystemTheme() : t
  actualTheme.value = actual
  document.documentElement.setAttribute('data-theme', actual)
  
  // Dispatch event for Naive UI
  window.dispatchEvent(new CustomEvent('theme-change', { detail: actual }))
}

function setTheme(t: Theme) {
  theme.value = t
  localStorage.setItem('catcoding-theme', t)
  applyTheme(t)
}

onMounted(() => {
  applyTheme(theme.value)
  
  // Listen for system theme changes
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (theme.value === 'system') {
      applyTheme('system')
    }
  })
})

watch(theme, (t) => applyTheme(t))

const icons = { light: PartlySunnyOutline, dark: MoonOutline, system: DesktopOutline }
const labels = { light: 'Light', dark: 'Dark', system: 'System' }
</script>

<template>
  <n-popover trigger="click" placement="bottom-end" :show-arrow="false">
    <template #trigger>
      <n-button quaternary circle size="small" :title="`Theme: ${labels[theme]}`">
        <template #icon>
          <component :is="icons[theme]" />
        </template>
      </n-button>
    </template>
    <n-space vertical :size="4">
      <n-button
        v-for="t in (['light', 'dark', 'system'] as Theme[])"
        :key="t"
        block
        :type="theme === t ? 'primary' : 'default'"
        :quaternary="theme !== t"
        size="small"
        @click="setTheme(t)"
      >
        <template #icon>
          <component :is="icons[t]" />
        </template>
        {{ labels[t] }}
      </n-button>
    </n-space>
  </n-popover>
</template>
