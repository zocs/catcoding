<script setup lang="ts">
import { ref, onMounted, provide, nextTick, computed } from 'vue'
import {
  NConfigProvider,
  NMessageProvider,
  NDialogProvider,
  NGlobalStyle,
  darkTheme
} from 'naive-ui'
import type { GlobalThemeOverrides } from 'naive-ui'
import ClickPaw from './components/ClickPaw.vue'

type ThemeMode = 'light' | 'dark' | 'system'

const themeMode = ref<ThemeMode>(
  (localStorage.getItem('catcoding-theme') as ThemeMode) || 'system'
)
const isDark = ref(false)

const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')

function canDetectSystemTheme(): boolean {
  const lightQuery = window.matchMedia('(prefers-color-scheme: light)')
  return mediaQuery.matches || lightQuery.matches
}

function applyTheme() {
  if (themeMode.value === 'dark') {
    isDark.value = true
  } else if (themeMode.value === 'light') {
    isDark.value = false
  } else {
    if (canDetectSystemTheme()) {
      isDark.value = mediaQuery.matches
    } else {
      isDark.value = true
    }
  }
  document.documentElement.setAttribute('data-theme', isDark.value ? 'dark' : 'light')
}

function setThemeMode(mode: ThemeMode) {
  themeMode.value = mode
  localStorage.setItem('catcoding-theme', mode)
  applyTheme()
}

// Naive UI theme overrides — matches CSS design tokens
const darkOverrides: GlobalThemeOverrides = {
  common: {
    bodyColor: '#0F172A',
    cardColor: '#1B2336',
    modalColor: '#1E293B',
    popoverColor: '#1E293B',
    inputColor: '#272F42',
    primaryColor: '#22C55E',
    primaryColorHover: '#16A34A',
    primaryColorPressed: '#15803D',
    primaryColorSuppl: '#22C55E',
    infoColor: '#3B82F6',
    successColor: '#22C55E',
    warningColor: '#F59E0B',
    errorColor: '#EF4444',
    textColorBase: '#F8FAFC',
    textColor1: '#F8FAFC',
    textColor2: '#94A3B8',
    textColor3: '#64748B',
    borderColor: '#334155',
    dividerColor: '#334155',
    borderRadius: '16px',
    borderRadiusSmall: '8px',
    fontFamily: "'Fira Sans', -apple-system, BlinkMacSystemFont, system-ui, sans-serif",
    fontFamilyMono: "'Fira Code', 'JetBrains Mono', monospace",
  },
  Card: {
    borderRadius: '16px',
    color: '#1B2336',
    borderColor: '#334155',
  },
  Menu: {
    itemColorActive: 'rgba(34,197,94,0.1)',
    itemColorActiveHover: 'rgba(34,197,94,0.15)',
    itemTextColor: '#94A3B8',
    itemTextColorActive: '#22C55E',
    itemTextColorActiveHover: '#22C55E',
    itemIconColor: '#64748B',
    itemIconColorActive: '#22C55E',
    itemIconColorActiveHover: '#22C55E',
    borderRadius: '8px',
  },
  Input: {
    color: '#272F42',
    colorFocus: '#272F42',
    border: '1px solid #334155',
    borderFocus: '1px solid #22C55E',
    textColor: '#F8FAFC',
    placeholderColor: '#64748B',
    caretColor: '#22C55E',
  },
  Button: {
    borderRadiusMedium: '12px',
    borderRadiusSmall: '8px',
  },
  Tag: {
    borderRadius: '8px',
  },
  Layout: {
    colorEmbedded: '#1E293B',
    siderColor: '#1E293B',
  },
}

const lightOverrides: GlobalThemeOverrides = {
  common: {
    bodyColor: '#FAFBFC',
    cardColor: '#FFFFFF',
    modalColor: '#F1F5F9',
    popoverColor: '#FFFFFF',
    inputColor: '#F1F5F9',
    primaryColor: '#16A34A',
    primaryColorHover: '#15803D',
    primaryColorPressed: '#166534',
    primaryColorSuppl: '#16A34A',
    infoColor: '#2563EB',
    successColor: '#16A34A',
    warningColor: '#D97706',
    errorColor: '#DC2626',
    textColorBase: '#0F172A',
    textColor1: '#0F172A',
    textColor2: '#475569',
    textColor3: '#94A3B8',
    borderColor: '#E2E8F0',
    dividerColor: '#E2E8F0',
    borderRadius: '16px',
    borderRadiusSmall: '8px',
    fontFamily: "'Fira Sans', -apple-system, BlinkMacSystemFont, system-ui, sans-serif",
    fontFamilyMono: "'Fira Code', 'JetBrains Mono', monospace",
  },
  Card: {
    borderRadius: '16px',
    color: '#FFFFFF',
    borderColor: '#E2E8F0',
  },
  Menu: {
    itemColorActive: 'rgba(22,163,74,0.08)',
    itemColorActiveHover: 'rgba(22,163,74,0.12)',
    itemTextColor: '#475569',
    itemTextColorActive: '#16A34A',
    itemTextColorActiveHover: '#16A34A',
    itemIconColor: '#94A3B8',
    itemIconColorActive: '#16A34A',
    itemIconColorActiveHover: '#16A34A',
    borderRadius: '8px',
  },
  Input: {
    color: '#F1F5F9',
    colorFocus: '#F1F5F9',
    border: '1px solid #E2E8F0',
    borderFocus: '1px solid #16A34A',
    textColor: '#0F172A',
    placeholderColor: '#94A3B8',
    caretColor: '#16A34A',
  },
  Button: {
    borderRadiusMedium: '12px',
    borderRadiusSmall: '8px',
  },
  Tag: {
    borderRadius: '8px',
  },
  Layout: {
    colorEmbedded: '#F1F5F9',
    siderColor: '#F1F5F9',
  },
}

const themeOverrides = computed(() => isDark.value ? darkOverrides : lightOverrides)

onMounted(async () => {
  applyTheme()
  mediaQuery.addEventListener('change', () => {
    if (themeMode.value === 'system') applyTheme()
  })
  if (themeMode.value === 'system') {
    await nextTick()
    setTimeout(applyTheme, 100)
  }
})

provide('themeMode', themeMode)
provide('isDark', isDark)
provide('setThemeMode', setThemeMode)
</script>

<template>
  <n-config-provider :theme="isDark ? darkTheme : null" :theme-overrides="themeOverrides">
    <n-global-style />
    <n-message-provider>
      <n-dialog-provider>
        <div style="height: 100vh; position: relative">
          <ClickPaw>
        <router-view />
      </ClickPaw>
        </div>
      </n-dialog-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<style>
@import url('https://fonts.googleapis.com/css2?family=Fira+Code:wght@400;500;600;700&family=Fira+Sans:wght@300;400;500;600;700&display=swap');

html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: 'Fira Sans', -apple-system, BlinkMacSystemFont, system-ui, sans-serif;
}
</style>
