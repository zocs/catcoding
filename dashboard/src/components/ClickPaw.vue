<template>
  <div class="click-paw-layer" @click="handleClick">
    <slot />
    <TransitionGroup name="paw">
      <div
        v-for="paw in paws"
        :key="paw.id"
        class="paw-print"
        :style="{ left: paw.x + 'px', top: paw.y + 'px' }"
      >
        <svg viewBox="0 0 32 32" width="24" height="24" fill="currentColor">
          <!-- 猫爪：四个小肉垫 + 一个大肉垫 -->
          <ellipse cx="10" cy="10" rx="3.5" ry="4" />
          <ellipse cx="22" cy="10" rx="3.5" ry="4" />
          <ellipse cx="6" cy="19" rx="3" ry="3.5" />
          <ellipse cx="26" cy="19" rx="3" ry="3.5" />
          <path d="M16 28c-5 0-9-3.5-9-7s2-5 4.5-5.5c1-.2 2.2.2 3 1 .8-.8 2-1.2 3-1 2.5.5 4.5 2 4.5 5.5s-4 7-9 7z" />
        </svg>
      </div>
    </TransitionGroup>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface Paw {
  id: number
  x: number
  y: number
}

const paws = ref<Paw[]>([])
let counter = 0

// 限流：300ms 内只触发一次
let lastTime = 0

function handleClick(e: MouseEvent) {
  const now = Date.now()
  if (now - lastTime < 300) return
  lastTime = now

  const id = counter++
  const target = e.currentTarget as HTMLElement
  const rect = target.getBoundingClientRect()
  const x = e.clientX - rect.left - 12 // 居中 (24px/2)
  const y = e.clientY - rect.top - 12

  paws.value.push({ id, x, y })

  // 400ms 后移除
  setTimeout(() => {
    paws.value = paws.value.filter(p => p.id !== id)
  }, 450)
}
</script>

<style scoped>
.click-paw-layer {
  position: relative;
  width: 100%;
  height: 100%;
}

.paw-print {
  position: absolute;
  pointer-events: none;
  color: var(--cc-accent);
  opacity: 0;
  transform: scale(0.3) rotate(-15deg);
  animation: pawPop 400ms var(--cc-ease-out) forwards;
  z-index: 999;
  filter: drop-shadow(0 0 4px rgba(34, 197, 94, 0.3));
}

@keyframes pawPop {
  0% {
    opacity: 0;
    transform: scale(0.3) rotate(-15deg);
  }
  30% {
    opacity: 0.7;
    transform: scale(1.1) rotate(5deg);
  }
  60% {
    opacity: 0.5;
    transform: scale(1) rotate(0deg) translateY(-8px);
  }
  100% {
    opacity: 0;
    transform: scale(0.8) rotate(5deg) translateY(-20px);
  }
}

@media (prefers-reduced-motion: reduce) {
  .paw-print {
    animation: none;
    display: none;
  }
}
</style>
