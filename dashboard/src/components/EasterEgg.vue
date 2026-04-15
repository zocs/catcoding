<template>
  <div class="easter-eggs" v-if="activeEgg">
    <n-modal v-model:show="showModal" preset="card" :title="activeEgg.title" style="width: 400px">
      <div class="egg-content">
        <div class="egg-emoji">{{ activeEgg.emoji }}</div>
        <p class="egg-message">{{ activeEgg.message }}</p>
      </div>
      <template #footer>
        <n-button @click="dismissEgg">知道了 🐱</n-button>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

interface EasterEgg {
  id: string
  emoji: string
  title: string
  message: string
  trigger: string
}

const props = defineProps<{
  trigger?: string
}>()

const showModal = ref(false)
const activeEgg = ref<EasterEgg | null>(null)

const eggs: EasterEgg[] = [
  {
    id: 'tiger',
    emoji: '🐯',
    title: '猫咪觉醒了老虎之力！',
    message: '连续完成 10 个任务无失败 — 全速前进！Agent 临时加速',
    trigger: 'streak_10',
  },
  {
    id: 'legendary_cat',
    emoji: '🐱⬛',
    title: '传奇奶牛猫现身了！🐱⬛',
    message: '抓到第 100 个 bug — 传说中这只猫曾一爪打掉过老鼠的耳朵，鼠辈闻风丧胆',
    trigger: 'bugs_100',
  },
  {
    id: 'rat_king',
    emoji: '🐭👑',
    title: '老鼠国王现身了...',
    message: '出现架构级 bug（恶龙）— 这需要全员出动',
    trigger: 'dragon_bug',
  },
  {
    id: 'vampire_bat',
    emoji: '🦇',
    title: '这只蝙蝠怎么打不死？！',
    message: '同一个 bug 复发 3 次 — 建议换一种思路',
    trigger: 'recurring_bug',
  },
  {
    id: 'cat_loaf',
    emoji: '🐱🍞',
    title: '全体猫咪进入了 loaf 模式！',
    message: '所有 agent 空闲超过 5 分钟 — 全员猫咪蜷缩动画',
    trigger: 'all_idle',
  },
  {
    id: 'nine_lives',
    emoji: '🐱🐈‍⬛',
    title: '九条命用完了！',
    message: 'Agent 被 watchdog 重启恰好 9 次 — 这是最后一次机会',
    trigger: 'restart_9',
  },
  {
    id: 'panda',
    emoji: '🐼',
    title: '大熊猫来庆祝了！🎉🎋',
    message: '项目整体进度达到 100% — 全屏熊猫动画',
    trigger: 'complete',
  },
  {
    id: 'magic_cat',
    emoji: '🐱🎩',
    title: '✨ 变个魔术...',
    message: '魔术猫出现 — 随机有趣的动画效果',
    trigger: 'magic',
  },
]

// 检查触发条件
watch(() => props.trigger, (trigger) => {
  if (trigger) {
    const egg = eggs.find(e => e.trigger === trigger)
    if (egg) {
      activeEgg.value = egg
      showModal.value = true
    }
  }
})

function dismissEgg() {
  showModal.value = false
  activeEgg.value = null
}

// 暴露方法供父组件调用
defineExpose({
  triggerEgg: (trigger: string) => {
    const egg = eggs.find(e => e.trigger === trigger)
    if (egg) {
      activeEgg.value = egg
      showModal.value = true
    }
  },
})
</script>

<style scoped>
.egg-content {
  text-align: center;
  padding: 24px;
}

.egg-emoji {
  font-size: 64px;
  margin-bottom: 16px;
  animation: bounce 1s ease-out;
}

.egg-message {
  color: var(--cc-fg-muted);
  font-size: 14px;
}

@keyframes bounce {
  0% { transform: scale(0); }
  50% { transform: scale(1.3); }
  100% { transform: scale(1); }
}
</style>
