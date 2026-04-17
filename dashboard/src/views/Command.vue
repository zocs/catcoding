<script setup lang="ts">
// Command.vue — 📨 指令中心：向 daemon 下达指令
import { ref, onMounted, nextTick } from 'vue'
import { NPageHeader, NCard, NInput, NButton, NSpace, NTag, NEmpty, useMessage } from 'naive-ui'
import { useI18n } from 'vue-i18n'
import { CatCodingApi } from '@/api/types'
import { useResponsive } from '@/composables/useResponsive'

const { t } = useI18n()
const { isMobile } = useResponsive()
const api = new CatCodingApi()
const message = useMessage()

interface HistoryEntry {
  id: number
  at: string
  command: string
  ok: boolean
  response: string
}

const command = ref('')
const sending = ref(false)
const history = ref<HistoryEntry[]>([])
const inputRef = ref<any>(null)
let nextId = 1

async function send() {
  const cmd = command.value.trim()
  if (!cmd) return
  sending.value = true
  const entry: HistoryEntry = {
    id: nextId++,
    at: new Date().toLocaleTimeString(),
    command: cmd,
    ok: false,
    response: '',
  }
  try {
    await api.sendCommand(cmd)
    entry.ok = true
    entry.response = t('command.sent', 'sent')
    message.success(t('command.sent', 'sent'))
  } catch (e: any) {
    entry.ok = false
    entry.response = e?.message || t('command.failed', 'failed')
    message.error(entry.response)
  } finally {
    history.value.unshift(entry)
    command.value = ''
    sending.value = false
    await nextTick()
    inputRef.value?.focus?.()
  }
}

function pickPreset(preset: string) {
  command.value = preset
  inputRef.value?.focus?.()
}

const PRESETS: string[] = [
  'status',
  'list agents',
  'list tasks',
  'health',
  'restart reviewer',
]

onMounted(() => {
  inputRef.value?.focus?.()
})
</script>

<template>
  <div class="command-page" :class="{ mobile: isMobile }">
    <n-page-header :title="'📨 ' + t('command.title')"
                   :subtitle="t('command.subtitle')" />

    <n-card :bordered="false" class="input-card">
      <n-space vertical size="medium">
        <n-input
          ref="inputRef"
          v-model:value="command"
          :placeholder="t('command.inputPlaceholder')"
          :disabled="sending"
          @keyup.enter="send"
          clearable
          size="large"
        />
        <n-space>
          <n-button type="primary" :loading="sending" :disabled="!command.trim()" round @click="send">
            🐾 {{ t('command.send') }}
          </n-button>
          <n-tag
            v-for="preset in PRESETS"
            :key="preset"
            checkable
            round
            @click="pickPreset(preset)"
          >
            {{ preset }}
          </n-tag>
        </n-space>
      </n-space>
    </n-card>

    <n-card :bordered="false" class="history-card">
      <template #header>
        <span>📜 {{ t('command.history') }}</span>
      </template>
      <n-empty v-if="history.length === 0" :description="t('command.noHistory', 'No commands yet — send one above.')" />
      <ul v-else class="history-list">
        <li v-for="item in history" :key="item.id" class="history-entry" :class="{ ok: item.ok, fail: !item.ok }">
          <div class="row-1">
            <span class="ts">{{ item.at }}</span>
            <n-tag :type="item.ok ? 'success' : 'error'" size="small" round>
              {{ item.ok ? '✓' : '✗' }}
            </n-tag>
            <code class="cmd">{{ item.command }}</code>
          </div>
          <div class="resp">{{ item.response }}</div>
        </li>
      </ul>
    </n-card>
  </div>
</template>

<style scoped>
.command-page {
  min-height: 100vh;
  padding: 16px;
  color: var(--cc-fg);
}
.command-page.mobile {
  padding: 8px;
}
.input-card,
.history-card {
  margin-top: 16px;
  border-radius: 16px;
}
.history-list {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.history-entry {
  padding: 10px 12px;
  border-radius: 10px;
  border: 1px dashed var(--cc-border);
  background: var(--cc-bg);
}
.history-entry.ok {
  border-color: rgba(34, 197, 94, 0.4);
}
.history-entry.fail {
  border-color: rgba(239, 68, 68, 0.4);
}
.row-1 {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
}
.ts {
  font-family: monospace;
  opacity: 0.55;
  font-size: 12px;
}
.cmd {
  font-family: monospace;
  font-size: 13px;
  color: var(--cc-orange);
  word-break: break-all;
}
.resp {
  font-size: 12px;
  opacity: 0.7;
  margin-top: 4px;
  padding-left: 20px;
}
</style>
