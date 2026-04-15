<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { NButtonGroup, NButton } from 'naive-ui'

const { locale } = useI18n()

function switchLang(lang: string) {
  locale.value = lang
  localStorage.setItem('catcoding-lang', lang)
}
</script>

<template>
  <div class="lang-switch">
    <NButtonGroup size="small">
      <NButton
        :type="locale === 'en' ? 'primary' : 'default'"
        :quaternary="locale !== 'en'"
        @click="switchLang('en')"
      >
        EN
      </NButton>
      <NButton
        :type="locale === 'zh' ? 'primary' : 'default'"
        :quaternary="locale !== 'zh'"
        @click="switchLang('zh')"
      >
        中文
      </NButton>
    </NButtonGroup>
  </div>
</template>

<style scoped>
.lang-switch {
  display: flex;
  align-items: center;
}
</style>
