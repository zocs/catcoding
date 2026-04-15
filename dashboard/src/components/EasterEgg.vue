<template>
  <div class="easter-eggs" v-if="activeEgg">
    <n-modal v-model:show="showModal" preset="card" :title="activeEgg.title" style="width: 400px">
      <div class="egg-content">
        <div class="egg-emoji">{{ activeEgg.emoji }}</div>
        <p class="egg-message">{{ activeEgg.message }}</p>
      </div>
      <template #footer>
        <n-button @click="dismissEgg">{{ t('easterEgg.dismiss') }}</n-button>
      </template>
    </n-modal>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { ref, watch } from 'vue'
const { t } = useI18n()


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
    title: t('easterEgg.tiger_title'),
    message: t('easterEgg.tiger_msg'),
    trigger: 'streak_10',
  },
  {
    id: 'legendary_cat',
    emoji: '🐱⬛',
    title: t('easterEgg.cow_title'),
    message: t('easterEgg.cow_msg'),
    trigger: 'bugs_100',
  },
  {
    id: 'rat_king',
    emoji: '🐭👑',
    title: t('easterEgg.mouseKing_title'),
    message: t('easterEgg.mouseKing_msg'),
    trigger: 'dragon_bug',
  },
  {
    id: 'vampire_bat',
    emoji: '🦇',
    title: t('easterEgg.bat_title'),
    message: 'Same bug 3 times — try a different approach',
    trigger: 'recurring_bug',
  },
  {
    id: 'cat_loaf',
    emoji: '🐱🍞',
    title: 'All cats entered loaf mode!',
    message: 'All agents idle 5+ min — everyone curled up',
    trigger: 'all_idle',
  },
  {
    id: 'nine_lives',
    emoji: '🐱🐈‍⬛',
    title: 'Nine lives used up!',
    message: 'Agent restarted 9 times by watchdog — last chance',
    trigger: 'restart_9',
  },
  {
    id: 'panda',
    emoji: '🐼',
    title: 'Panda celebration! 🎉🎋',
    message: 'Project 100% complete — full-screen panda!',
    trigger: 'complete',
  },
  {
    id: 'magic_cat',
    emoji: '🐱🎩',
    title: '✨ A little magic...',
    message: 'Magic cat — random fun animation!',
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
