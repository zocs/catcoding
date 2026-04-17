<script setup lang="ts">
import { useI18n } from 'vue-i18n'
const { t } = useI18n()

const title = t('command.title')
const calling = t('command.demo_calling')
</script>

<template>
  <div style="padding: 24px; color: white;">
    <h1>TEST</h1>
    <p>{{ title }}</p>
    <p>{{ calling }}</p>
  </div>
</template>
