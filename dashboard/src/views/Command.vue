<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { ref, onMounted, nextTick } from 'vue'
import { NPageHeader, NCard, NInput, NButton, NSpace, NTag, NLog, NProgress, useMessage } from 'naive-ui'
import { CatCodingApi, AGENT_ROLES_FIXED } from '@/api/types'
import BugTracker from '@/components/BugTracker.vue'
import EasterEgg from '@/components/EasterEgg.vue'
const { t } = useI18n()


const api = new CatCodingApi()
const message = useMessage()
const command = ref('')
const logs = ref<string[]>([])
const loading = ref(false)
const easterEggRef = ref<InstanceType<typeof EasterEgg> | null>(null)
const logContainer = ref<HTMLElement | null>(null)

// Bug 追踪状态
interface Bug {
  id: string
  level: 'mouse' | 'big_mouse' | 'bat' | 'dragon'
  title: string
  file: string
  status: 'alive' | 'chasing' | 'caught' | 'escaped'
  discoveredAt: number
}
const bugs = ref<Bug[]>([])
const reviewActive = ref(false)
const reviewProgress = ref(0)
const currentReviewer = ref('')

// 模拟的代码评审流程
const reviewSteps = [
  { delay: 500,  msg: t('command.demo_calling'), emoji: '🐱' },
  { delay: 1200, msg: t('command.demo_arrived'), emoji: '👀' },
  { delay: 2000, msg: t('command.demo_scanning', { file: 'daemon/src/api/mod.rs' }), emoji: '📂' },
  { delay: 2800, msg: t('command.demo_scanning', { file: 'daemon/src/scheduler.rs' }), emoji: '📂' },
  { delay: 3500, msg: t('command.demo_scanning', { file: 'agents/base/agent.py' }), emoji: '📂' },
  { delay: 4200, msg: t('command.demo_scanning', { file: 'dashboard/src/views/Board.vue' }), emoji: '📂' },
  // 发现 Bug
  { delay: 5000, msg: t('command.demo_found_mouse'), emoji: '🚨', bug: { id: 'b1', level: 'mouse' as const, title: '未处理的 unwrap() 调用', file: 'daemon/src/api/mod.rs:45' } },
  { delay: 5800, msg: t('command.demo_found_rat'), emoji: '🚨', bug: { id: 'b2', level: 'big_mouse' as const, title: '缺少错误处理的 HTTP 请求', file: 'daemon/src/scheduler.rs:112' } },
  { delay: 6500, msg: t('command.demo_found_mouse'), emoji: '🔍', bug: { id: 'b3', level: 'mouse' as const, title: '未使用的 import', file: 'agents/base/agent.py:3' } },
  { delay: 7500, msg: t('command.demo_found_rat'), emoji: '🚨', bug: { id: 'b4', level: 'bat' as const, title: 'N+1 查询问题', file: 'daemon/src/db.rs:67' } },
  // 开始抓老鼠
  { delay: 8500, msg: '🐾 Target locked...hunting!', emoji: '🏃' },
  { delay: 9200, msg: '🐱💨 Caught b1!', emoji: '🎯', catchBug: 'b1' },
  { delay: 9800, msg: '🐱💨 Caught b3!', emoji: '🎯', catchBug: 'b3' },
  // 指派修复
  { delay: 10500, msg: '📋 Generating report...', emoji: '📝' },
  { delay: 11200, msg: '📨 Assign @core_dev to fix b2', emoji: '🐱' },
  { delay: 11800, msg: '📨 Assign @backend to fix b4', emoji: '🐱' },
  // 英短修复
  { delay: 12500, msg: '🐱 @core_dev fixing b2...', emoji: '🔧' },
  { delay: 13200, msg: '✅ @core_dev fixed b2!', emoji: '🎯', catchBug: 'b2' },
  // 缅因猫修复
  { delay: 14000, msg: '🐱 @backend fixing b4...', emoji: '🔧' },
  { delay: 15000, msg: '✅ @backend fixed b4!', emoji: '🎯', catchBug: 'b4' },
  // 报告
  { delay: 16000, msg: '📊 ===== Bug Report =====', emoji: '📋' },
  { delay: 16200, msg: '   Files scanned: 4', emoji: '  ' },
  { delay: 16400, msg: '   Bugs found: 4', emoji: '  ' },
  { delay: 16600, msg: '   Fixed: 4', emoji: '  ' },
  { delay: 16800, msg: '   Escaped: 0', emoji: '  ' },
  { delay: 17000, msg: '   Status: ✅ All cleared!', emoji: '  ' },
  { delay: 17500, msg: '🎉 Review complete! Good work team!', emoji: '🐱' },
]

function addLog(msg: string, type: 'info' | 'success' | 'error' | 'command' | 'cat' = 'info') {
  const time = new Date().toLocaleTimeString()
  let prefix = `[${time}] `
  if (type === 'command') prefix += '🐱 > '
  if (type === 'cat') prefix += '🐾 '
  logs.value.push(`${prefix}${msg}`)
  // 自动滚动到底部
  nextTick(() => {
    const el = logContainer.value?.querySelector('.n-log')
    if (el) el.scrollTop = el.scrollHeight
  })
}

async function handleExecute() {
  if (!command.value.trim()) return

  const cmd = command.value.trim()
  addLog(cmd, 'command')
  command.value = ''

  if (cmd === '/code-review' || cmd === '/review') {
    await runCodeReview()
  } else if (cmd === '/magic') {
    easterEggRef.value?.triggerEgg('magic')
    addLog('✨ Magic cat appears!', 'cat')
  } else if (cmd === '/loaf') {
    easterEggRef.value?.triggerEgg('all_idle')
    addLog('🐱🍞 All cats in loaf mode!', 'cat')
  } else if (cmd === '/tiger') {
    easterEggRef.value?.triggerEgg('streak_10')
    addLog('🐯 Tiger power activated!', 'cat')
  } else {
    loading.value = true
    try {
      await api.sendCommand(cmd)
      addLog(`Command "${cmd}" sent.`, 'success')
    } catch {
      addLog('Execution failed, but cats never give up!', 'error')
    } finally {
      loading.value = false
    }
  }
}

async function runCodeReview() {
  reviewActive.value = true
  bugs.value = []
  reviewProgress.value = 0
  currentReviewer.value = 'Black Cat'
  loading.value = true

  for (const step of reviewSteps) {
    await new Promise(resolve => setTimeout(resolve, step.delay - (reviewSteps.indexOf(step) > 0 ? reviewSteps[reviewSteps.indexOf(step) - 1].delay : 0)))

    addLog(step.msg, step.emoji === '🚨' ? 'error' : step.emoji === '🎯' ? 'success' : 'info')

    // 发现 Bug
    if (step.bug) {
      bugs.value.push({
        ...step.bug,
        status: 'alive',
        discoveredAt: Date.now(),
      })
    }

    // 抓到 Bug
    if (step.catchBug) {
      const bug = bugs.value.find(b => b.id === step.catchBug)
      if (bug) {
        bug.status = 'chasing'
        await new Promise(resolve => setTimeout(resolve, 400))
        bug.status = 'caught'
      }
    }

    // 更新进度
    reviewProgress.value = Math.min(100, (reviewSteps.indexOf(step) / reviewSteps.length) * 100)
  }

  // 检查彩蛋
  if (bugs.value.every(b => b.status === 'caught')) {
    await new Promise(resolve => setTimeout(resolve, 500))
    if (bugs.value.some(b => b.level === 'bat')) {
      easterEggRef.value?.triggerEgg('bugs_100')
    }
  }

  reviewActive.value = false
  loading.value = false
}

function handleCatchBug(bugId: string) {
  const bug = bugs.value.find(b => b.id === bugId)
  if (bug && bug.status === 'alive') {
    bug.status = 'chasing'
    addLog(`🐾 You caught ${bug.title}! Good cat!`, 'cat')
    setTimeout(() => {
      if (bug) bug.status = 'caught'
    }, 600)
  }
}

onMounted(() => {
  addLog('🐱 CatCoding terminal ready.', 'cat')
  addLog('Type /code-review to start auto review', 'info')
  addLog('Easter eggs: /magic /loaf /tiger', 'info')
})
</script>

<template>
  <div class="command-page">
    <n-page-header :title="'🐱 ' + t('command.title')" :subtitle="t('command.subtitle')">
      <template #extra>
        <n-tag v-if="reviewActive" type="warning" round>
          🔍 {{ t("command.autoReview") }}...
        </n-tag>
      </template>
    </n-page-header>

    <n-space vertical size="large" class="main-content">
      <!-- 评审进度条 -->
      <n-card v-if="reviewActive" class="progress-card">
        <n-space align="center">
          <span class="reviewer-emoji">🖤</span>
          <span>{{ t("command.autoReview") }}...</span>
          <n-progress
            type="line"
            :percentage="reviewProgress"
            :show-indicator="false"
            style="flex: 1; min-width: 200px;"
            color="#f5a623"
          />
        </n-space>
      </n-card>

      <!-- Bug 追踪器（有 Bug 时显示） -->
      <BugTracker v-if="bugs.length > 0" :bugs="bugs" @catch="handleCatchBug" />

      <!-- 终端日志 -->
      <n-card class="terminal-card" ref="logContainer">
        <n-log
          :lines="logs"
          :font-size="13"
          class="terminal-log"
          trim
        />
      </n-card>

      <!-- 输入区 -->
      <n-card class="input-card">
        <n-space>
          <n-input
            v-model:value="command"
            :placeholder="t('command.inputPlaceholder')"
            @keyup.enter="handleExecute"
            style="width: 500px"
            :disabled="loading"
          >
            <template #prefix>
              <span class="prompt" :class="{ 'cat-active': reviewActive }">
                {{ reviewActive ? '🖤 >' : '🐱 >' }}
              </span>
            </template>
          </n-input>
          <n-button type="primary" @click="handleExecute" :loading="loading" round>
            🐾 {{ t("command.send") }}
          </n-button>
        </n-space>

        <!-- 快捷指令标签 -->
        <div class="hints">
          <n-tag size="small" quaternary round @click="command = '/code-review'">
            /code-review 🖤 Code Review
          </n-tag>
          <n-tag size="small" quaternary round @click="command = '/magic'">
            /magic ✨ Magic Cat
          </n-tag>
          <n-tag size="small" quaternary round @click="command = '/loaf'">
            /loaf 🍞 Loaf Mode
          </n-tag>
          <n-tag size="small" quaternary round @click="command = '/tiger'">
            /tiger 🐯 Tiger Power
          </n-tag>
          <n-tag size="small" quaternary round @click="command = 'status'">
            status 📊 Status
          </n-tag>
          <n-tag size="small" quaternary round @click="command = 'deploy'">
            deploy 🚀 Deploy
          </n-tag>
        </div>
      </n-card>
    </n-space>

    <!-- 彩蛋 -->
    <EasterEgg ref="easterEggRef" />
  </div>
</template>

<style scoped>
.command-page {
  min-height: 100vh;
}

.main-content {
  margin-top: 24px;
}

.progress-card {
  border-radius: 12px;
  border: 2px solid var(--cc-orange);
  animation: progressPulse 2s ease-in-out infinite;
}

@keyframes progressPulse {
  0%, 100% { box-shadow: 0 0 0 0 rgba(245, 166, 35, 0.2); }
  50%      { box-shadow: 0 0 20px 4px rgba(245, 166, 35, 0.1); }
}

.reviewer-emoji {
  font-size: 24px;
  animation: catBreathe 1.5s ease-in-out infinite;
}

@keyframes catBreathe {
  0%, 100% { transform: scale(1); }
  50%      { transform: scale(1.1); }
}

.terminal-card {
  background: var(--cc-bg-card);
  border-radius: 16px;
  border: 2px solid var(--cc-bg-input);
}

.terminal-log {
  height: 400px;
  color: var(--cc-fg);
  font-family: 'Fira Code', 'JetBrains Mono', monospace;
}

.input-card {
  border-radius: 16px;
}

.prompt {
  color: var(--cc-orange);
  font-weight: bold;
  margin-right: 8px;
  transition: color 0.3s;
}

.prompt.cat-active {
  color: var(--cc-purple);
  animation: catBreathe 1s ease-in-out infinite;
}

.hints {
  margin-top: 12px;
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.hints .n-tag {
  cursor: pointer;
  transition: all 0.2s;
}

.hints .n-tag:hover {
  transform: scale(1.05);
  background: rgba(245, 166, 35, 0.1);
}
</style>
