<script setup lang="ts">
import { ref, onMounted, nextTick } from 'vue'
import { NPageHeader, NCard, NInput, NButton, NSpace, NTag, NLog, NProgress, useMessage } from 'naive-ui'
import { CatCodingApi, AGENT_ROLES_FIXED } from '@/api/types'
import BugTracker from '@/components/BugTracker.vue'
import EasterEgg from '@/components/EasterEgg.vue'

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
  { delay: 500,  msg: '📡 正在呼叫 @reviewer (玄猫)...', emoji: '🐱' },
  { delay: 1200, msg: '🖤 玄猫已就位，开始扫描代码仓库...', emoji: '👀' },
  { delay: 2000, msg: '🔍 扫描 daemon/src/api/mod.rs...', emoji: '📂' },
  { delay: 2800, msg: '🔍 扫描 daemon/src/scheduler.rs...', emoji: '📂' },
  { delay: 3500, msg: '🔍 扫描 agents/base/agent.py...', emoji: '📂' },
  { delay: 4200, msg: '🔍 扫描 dashboard/src/views/Board.vue...', emoji: '📂' },
  // 发现 Bug
  { delay: 5000, msg: '🐭 发现！ 小老鼠级: 未处理的 unwrap() 调用', emoji: '🚨', bug: { id: 'b1', level: 'mouse' as const, title: '未处理的 unwrap() 调用', file: 'daemon/src/api/mod.rs:45' } },
  { delay: 5800, msg: '🐀 发现！ 大老鼠级: 缺少错误处理的 HTTP 请求', emoji: '🚨', bug: { id: 'b2', level: 'big_mouse' as const, title: '缺少错误处理的 HTTP 请求', file: 'daemon/src/scheduler.rs:112' } },
  { delay: 6500, msg: '🐭 发现！ 小老鼠级: 未使用的 import', emoji: '🔍', bug: { id: 'b3', level: 'mouse' as const, title: '未使用的 import', file: 'agents/base/agent.py:3' } },
  { delay: 7500, msg: '🦇 发现！ 蝙蝠级: N+1 查询问题（第三次出现）', emoji: '🚨', bug: { id: 'b4', level: 'bat' as const, title: 'N+1 查询问题', file: 'daemon/src/db.rs:67' } },
  // 开始抓老鼠
  { delay: 8500, msg: '🐾 玄猫锁定目标...开始追捕！', emoji: '🏃' },
  { delay: 9200, msg: '🐱💨 玄猫追上了 b1！', emoji: '🎯', catchBug: 'b1' },
  { delay: 9800, msg: '🐱💨 玄猫追上了 b3！', emoji: '🎯', catchBug: 'b3' },
  // 指派修复
  { delay: 10500, msg: '📋 玄猫生成《老鼠检测报告》...', emoji: '📝' },
  { delay: 11200, msg: '📨 指派 @core_dev (英短蓝猫) 修复大老鼠 b2', emoji: '🐱' },
  { delay: 11800, msg: '📨 指派 @backend (缅因猫) 修复蝙蝠 b4', emoji: '🐱' },
  // 英短修复
  { delay: 12500, msg: '🐱 英短蓝猫开始修复 b2...', emoji: '🔧' },
  { delay: 13200, msg: '✅ 英短蓝猫修复了 b2！', emoji: '🎯', catchBug: 'b2' },
  // 缅因猫修复
  { delay: 14000, msg: '🐱 缅因猫开始修复 b4 (蝙蝠很难缠)...', emoji: '🔧' },
  { delay: 15000, msg: '✅ 缅因猫终于抓住了 b4！蝙蝠被消灭！', emoji: '🎯', catchBug: 'b4' },
  // 报告
  { delay: 16000, msg: '📊 ===== 老鼠检测报告 =====', emoji: '📋' },
  { delay: 16200, msg: '   扫描文件: 4', emoji: '  ' },
  { delay: 16400, msg: '   发现 Bug: 4 (小老鼠×2, 大老鼠×1, 蝙蝠×1)', emoji: '  ' },
  { delay: 16600, msg: '   已修复: 4', emoji: '  ' },
  { delay: 16800, msg: '   逃走: 0', emoji: '  ' },
  { delay: 17000, msg: '   状态: ✅ 全部清除！', emoji: '  ' },
  { delay: 17500, msg: '🎉 代码评审完成！猫咪团队干得漂亮！', emoji: '🐱' },
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
    addLog('✨ 魔术猫出现！变变变...', 'cat')
  } else if (cmd === '/loaf') {
    easterEggRef.value?.triggerEgg('all_idle')
    addLog('🐱🍞 全体猫咪进入 loaf 模式！', 'cat')
  } else if (cmd === '/tiger') {
    easterEggRef.value?.triggerEgg('streak_10')
    addLog('🐯 猫咪觉醒了老虎之力！', 'cat')
  } else {
    loading.value = true
    try {
      await api.sendCommand(cmd)
      addLog(`指令 "${cmd}" 已下达。`, 'success')
    } catch {
      addLog(`执行失败，但猫咪不会放弃！`, 'error')
    } finally {
      loading.value = false
    }
  }
}

async function runCodeReview() {
  reviewActive.value = true
  bugs.value = []
  reviewProgress.value = 0
  currentReviewer.value = '玄猫'
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
    addLog(`🐾 你手动抓住了 ${bug.title}！好猫咪！`, 'cat')
    setTimeout(() => {
      if (bug) bug.status = 'caught'
    }, 600)
  }
}

onMounted(() => {
  addLog('🐱 CatCoding 终端已就绪。', 'cat')
  addLog('输入 /code-review 开启自动评审流', 'info')
  addLog('彩蛋指令: /magic /loaf /tiger', 'info')
})
</script>

<template>
  <div class="command-page">
    <n-page-header title="🐱 命令终端" subtitle="直接指挥猫咪团队">
      <template #extra>
        <n-tag v-if="reviewActive" type="warning" round>
          🔍 玄猫评审中...
        </n-tag>
      </template>
    </n-page-header>

    <n-space vertical size="large" class="main-content">
      <!-- 评审进度条 -->
      <n-card v-if="reviewActive" class="progress-card">
        <n-space align="center">
          <span class="reviewer-emoji">🖤</span>
          <span>玄猫正在评审...</span>
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
            placeholder="输入指令..."
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
            🐾 执行
          </n-button>
        </n-space>

        <!-- 快捷指令标签 -->
        <div class="hints">
          <n-tag size="small" quaternary round @click="command = '/code-review'">
            /code-review 🖤 代码评审
          </n-tag>
          <n-tag size="small" quaternary round @click="command = '/magic'">
            /magic ✨ 魔术猫
          </n-tag>
          <n-tag size="small" quaternary round @click="command = '/loaf'">
            /loaf 🍞 猫咪 loaf
          </n-tag>
          <n-tag size="small" quaternary round @click="command = '/tiger'">
            /tiger 🐯 老虎之力
          </n-tag>
          <n-tag size="small" quaternary round @click="command = 'status'">
            status 📊 状态
          </n-tag>
          <n-tag size="small" quaternary round @click="command = 'deploy'">
            deploy 🚀 部署
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
  border: 2px solid #f5a623;
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
  background: #1a1a2e;
  border-radius: 16px;
  border: 2px solid #2d2d44;
}

.terminal-log {
  height: 400px;
  color: #d4d4d4;
  font-family: 'Fira Code', 'JetBrains Mono', monospace;
}

.input-card {
  border-radius: 16px;
}

.prompt {
  color: #f5a623;
  font-weight: bold;
  margin-right: 8px;
  transition: color 0.3s;
}

.prompt.cat-active {
  color: #9c27b0;
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
