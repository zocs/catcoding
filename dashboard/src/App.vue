<script setup lang="ts">
import { ref, onMounted, provide } from 'vue'
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  darkTheme
} from 'naive-ui'
import type { GlobalTheme } from 'naive-ui'

type ThemeMode = 'light' | 'dark' | 'system'

const themeMode = ref<ThemeMode>(
  (localStorage.getItem('catcoding-theme') as ThemeMode) || 'system'
)
const isDark = ref(false)

function detectSystem(): boolean {
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

function applyTheme() {
  isDark.value = themeMode.value === 'dark'
    || (themeMode.value === 'system' && detectSystem())
  document.documentElement.setAttribute(
    'data-theme',
    isDark.value ? 'dark' : 'light'
  )
}

function setThemeMode(mode: ThemeMode) {
  themeMode.value = mode
  localStorage.setItem('catcoding-theme', mode)
  applyTheme()
}

onMounted(() => {
  applyTheme()
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (themeMode.value === 'system') applyTheme()
  })
})

// Provide theme controls to child components (ThemeSwitch, LangSwitch)
provide('themeMode', themeMode)
provide('isDark', isDark)
provide('setThemeMode', setThemeMode)
</script>

<template>
  <n-config-provider :theme="isDark ? darkTheme : null">
    <n-message-provider>
      <n-dialog-provider>
        <div style="height: 100vh; position: relative">
          <router-view />
        </div>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
}
</style>
