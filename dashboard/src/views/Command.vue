<template>
  <div class="command">
    <n-page-header title="💬 指令输入" subtitle="直接对猫咪团队说话">
      <template #extra>
        <n-tag type="info">
          <template #icon><span>💡</span></template>
          输入指令，猫咪团队将自动执行
        </n-tag>
      </template>
    </n-page-header>

    <n-card style="margin-top: 24px">
      <!-- 指令输入 -->
      <n-input-group>
        <n-input
          v-model:value="command"
          type="textarea"
          :autosize="{ minRows: 3, maxRows: 8 }"
          placeholder="例如：帮我实现一个登录功能，需要 Vue3 前端和 FastAPI 后端..."
          @keydown.ctrl.enter="sendCommand"
        />
      </n-input-group>

      <div class="input-actions">
        <n-space>
          <n-button @click="clearCommand">
            <template #icon><span>🧹</span></template>
            清空
          </n-button>
          <n-button type="primary" :loading="sending" @click="sendCommand">
            <template #icon><span>🚀</span></template>
            发送指令 (Ctrl+Enter)
          </n-button>
        </n-space>
      </div>
    </n-card>

    <!-- 快捷指令 -->
    <n-card title="⚡ 快捷指令" style="margin-top: 24px">
      <n-space>
        <n-button v-for="cmd in quickCommands" :key="cmd.label" @click="useQuickCommand(cmd)">
          {{ cmd.emoji }} {{ cmd.label }}
        </n-button>
      </n-space>
    </n-card>

    <!-- 历史记录 -->
    <n-card title="📜 历史指令" style="margin-top: 24px">
      <n-list v-if="history.length > 0">
        <n-list-item v-for="(item, index) in history" :key="index">
          <n-thing>
            <template #header>
              <span class="history-time">{{ item.time }}</span>
            </template>
            {{ item.command }}
          </n-thing>
          <template #suffix>
            <n-button size="tiny" @click="reuseCommand(item.command)">
              复用
            </n-button>
          </template>
        </n-list-item>
      </n-list>
      <n-empty v-else description="暂无历史指令" />
    </n-card>

    <!-- 执行结果 -->
    <n-card v-if="result" title="📋 执行结果" style="margin-top: 24px">
      <n-alert :type="result.success ? 'success' : 'error'" :title="result.message">
        <template v-if="result.details">
          <pre class="result-details">{{ result.details }}</pre>
        </template>
      </n-alert>
    </n-card>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useMessage } from 'naive-ui'

interface HistoryItem {
  time: string
  command: string
}

interface QuickCommand {
  label: string
  emoji: string
  command: string
}

const message = useMessage()
const command = ref('')
const sending = ref(false)
const history = ref<HistoryItem[]>([])
const result = ref<{ success: boolean; message: string; details?: string } | null>(null)

const quickCommands: QuickCommand[] = [
  { label: '实现登录功能', emoji: '🔐', command: '帮我实现一个登录功能，需要前端登录表单和后端 JWT 认证' },
  { label: '创建 API 接口', emoji: '🔌', command: '帮我创建一个 RESTful API 接口，支持 CRUD 操作' },
  { label: '编写测试', emoji: '🧪', command: '帮我为当前项目编写单元测试和集成测试' },
  { label: '代码审查', emoji: '🔍', command: '审查当前代码的质量和安全性' },
  { label: '优化性能', emoji: '⚡', command: '分析并优化当前项目的性能瓶颈' },
  { label: '部署项目', emoji: '🚀', command: '帮我配置和部署当前项目' },
]

function clearCommand() {
  command.value = ''
  result.value = null
}

function useQuickCommand(cmd: QuickCommand) {
  command.value = cmd.command
}

function reuseCommand(cmd: string) {
  command.value = cmd
}

async function sendCommand() {
  if (!command.value.trim()) {
    message.warning('请输入指令')
    return
  }

  sending.value = true
  result.value = null

  try {
    const res = await fetch('/api/command', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        command: command.value,
        args: [],
      }),
    })

    const data = await res.json()

    result.value = {
      success: true,
      message: data.message || '指令已接收',
      details: JSON.stringify(data, null, 2),
    }

    // 添加到历史
    history.value.unshift({
      time: new Date().toLocaleString('zh-CN'),
      command: command.value,
    })

    if (history.value.length > 10) {
      history.value.pop()
    }

    message.success('指令已发送给猫咪团队')
  } catch (e) {
    result.value = {
      success: false,
      message: '发送失败',
      details: String(e),
    }
    message.error('发送失败')
  } finally {
    sending.value = false
  }
}
</script>

<style scoped>
.command {
  padding: 16px;
}

.input-actions {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
}

.history-time {
  font-size: 12px;
  color: #888;
}

.result-details {
  background: #1e1e2e;
  padding: 12px;
  border-radius: 8px;
  overflow-x: auto;
  font-size: 12px;
  margin-top: 8px;
}
</style>
