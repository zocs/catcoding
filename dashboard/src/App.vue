<template>
  <n-message-provider>
    <n-dialog-provider>
      <div style="height: 100vh; position: relative">
        <router-view />
      </div>
    </n-dialog-provider>
  </n-message-provider>
</template>

<script setup lang="ts">
import { NMessageProvider, NDialogProvider } from 'naive-ui'
</script>

<style>
html, body, #app {
  margin: 0;
  padding: 0;
  height: 100%;
}
</style>
