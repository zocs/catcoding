<script setup lang="ts">
import { ref, nextTick, onMounted } from 'vue'
import { NCard, NInput, NButton } from 'naive-ui'

const terminalRef = ref<HTMLElement>()
const input = ref('')
const lines = ref<{ text: string; type: 'cmd' | 'output' | 'error' | 'cat' }[]>([])

const catResponses: Record<string, string> = {
  'cat --help': `🐱 CatCoding CLI v0.1.0
Usage: catcoding <command> [options]

Commands:
  init          Initialize a project with cat agents
  serve         Start the daemon + dashboard
  status        Show agent status
  command       Send a command to the team
  mice          List all bugs (mice) in the project
  feed          Feed the cat agents 🐟

Options:
  -v, --version Show version
  -h, --help    Show this help`,
  'catcoding status': `🐱 Agent Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
PM (Siamese)     ✅ Active    → Reviewing requirements
Dev (British)    ✅ Active    → Writing auth.rs
Reviewer (Bombay) 💤 Idle     
Tester (Abyss.)  ⏳ Waiting   → 3 tests pending
Scout (Fox)      ✅ Active    → Reading docs
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
NATS: 🟢 Connected  |  Tasks: 5 active, 12 done`,
  'catcoding mice': `🐭 Bug Report
━━━━━━━━━━━━━━━━━━━━━━━━
[CRITICAL] 🐉 恶龙 — Memory leak in daemon/src/state.rs:42
[HIGH]     🐀 大老鼠 — Race condition in scheduler
[MEDIUM]   🐀 大老鼠 — Missing error handling in API
[LOW]      🐭 小老鼠 — Typo in README.md
━━━━━━━━━━━━━━━━━━━━━━━━
Total: 4 mice | Caught: 0 | Hunting: 1`,
  'catcoding feed': `🐟 Feeding the team...
🐱 PM: *munch munch* "Thanks! Back to reviewing!"
🐱 Dev: *nom nom* "Fuel for more Rust code!"
🐱 Reviewer: *sniffs* "I'll eat after I finish this review..."
🐱 Tester: *catches fish mid-air* "Testing is hungry work!"
🐱 Scout: *shares with the team* "We eat together!"

All agents energized! +50% motivation ⚡`,
  'catcoding serve': `🚀 Starting CatCoding Daemon...
📡 NATS: Connected to nats://127.0.0.1:4222
💾 SQLite: Database ready at .catcoding/state.db
🌐 HTTP API: Listening on 127.0.0.1:9527
📊 Dashboard: http://localhost:8080
🐱 Agent Pool: 5 agents initialized

Ready to collaborate! Use 'catcoding command' to send tasks.`,
  'ls': `agents/   dashboard/   daemon/   cli/   config/
README.md  LICENSE  Makefile  PROGRESS.md`,
  'whoami': `zocs — CatCoding Project Owner 🐱`,
  'neofetch': `       _╓▄▄▄▄╖_          zocs@catcoding
     ╓███▀▀▀▀███╖        ────────────────
    ▐██▌  ◕  ◕▐██▌       OS: WSL Ubuntu 24.04
    ▐██▌   ▲   ▐██▌       Kernel: Linux 6.x
     ╙███▄▄▄▄███▀        Shell: bash
       ▀▀▀▀▀▀            Terminal: CatCoding Dashboard
                         CPU: Xiaomi Laptop
     CatCoding v0.1.0    Memory: 12GB / 32GB
`
}

function execute() {
  const cmd = input.value.trim()
  if (!cmd) return
  
  lines.value.push({ text: `$ ${cmd}`, type: 'cmd' })
  input.value = ''

  const response = catResponses[cmd]
  if (response) {
    lines.value.push({ text: response, type: cmd.includes('cat') ? 'cat' : 'output' })
  } else if (cmd === 'clear') {
    lines.value = []
  } else if (cmd.startsWith('catcoding ')) {
    lines.value.push({ text: `🐱 *tilts head* Unknown command: "${cmd}". Try "cat --help"`, type: 'cat' })
  } else {
    lines.value.push({ text: `bash: ${cmd}: command not found. Try "cat --help" for CatCoding commands.`, type: 'error' })
  }

  nextTick(() => {
    if (terminalRef.value) {
      terminalRef.value.scrollTop = terminalRef.value.scrollHeight
    }
  })
}

onMounted(() => {
  lines.value.push(
    { text: '🐱 Welcome to CatCoding Terminal!', type: 'cat' },
    { text: 'Type "cat --help" to see available commands.', type: 'output' },
    { text: 'Try: catcoding status | catcoding mice | catcoding feed | neofetch', type: 'output' },
    { text: '', type: 'output' }
  )
})
</script>

<template>
  <n-card title="🐱 Terminal Demo" class="terminal-card">
    <div class="terminal" ref="terminalRef">
      <div
        v-for="(line, i) in lines"
        :key="i"
        :class="['line', line.type]"
      >
        <pre>{{ line.text }}</pre>
      </div>
      <div class="input-line">
        <span class="prompt">$ </span>
        <input
          v-model="input"
          @keydown.enter="execute"
          class="terminal-input"
          placeholder="type a command..."
          autofocus
        />
      </div>
    </div>
    <template #footer>
      <div class="quick-commands">
        <n-button size="small" @click="input = 'cat --help'; execute()">cat --help</n-button>
        <n-button size="small" @click="input = 'catcoding status'; execute()">status</n-button>
        <n-button size="small" @click="input = 'catcoding mice'; execute()">mice</n-button>
        <n-button size="small" @click="input = 'catcoding feed'; execute()">feed 🐟</n-button>
        <n-button size="small" @click="input = 'neofetch'; execute()">neofetch</n-button>
        <n-button size="small" @click="input = 'catcoding serve'; execute()">serve</n-button>
      </div>
    </template>
  </n-card>
</template>

<style scoped>
.terminal-card {
  font-family: 'Cascadia Code', 'Fira Code', monospace;
}

.terminal {
  background: var(--cc-bg-card);
  border-radius: 8px;
  padding: 16px;
  min-height: 400px;
  max-height: 500px;
  overflow-y: auto;
  color: var(--cc-fg);
  font-size: 13px;
  line-height: 1.5;
}

.line pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
}

.line.cmd { color: var(--cc-blue); }
.line.output { color: var(--cc-fg); }
.line.error { color: var(--cc-error); }
.line.cat { color: var(--cc-success); }

.input-line {
  display: flex;
  align-items: center;
  margin-top: 4px;
}

.prompt {
  color: var(--cc-blue);
  margin-right: 8px;
}

.terminal-input {
  background: transparent;
  border: none;
  color: var(--cc-fg);
  font-family: inherit;
  font-size: inherit;
  flex: 1;
  outline: none;
}

.quick-commands {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
</style>
