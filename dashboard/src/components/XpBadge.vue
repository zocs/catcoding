<script setup lang="ts">
// XpBadge.vue — Agent 等级徽章 + XP 进度条
// 依据计划书 §4.6: Lv1 实习猫 → Lv5 专家猫
import { computed } from 'vue'

interface Props {
  level: number
  xp: number
  compact?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  compact: false,
})

const XP_THRESHOLDS = [0, 50, 200, 500, 1000]
const MAX_LEVEL = 5

const LEVEL_TITLES = [
  '',              // 0 (unused)
  '实习猫',         // Lv1
  '初级猫',         // Lv2
  '熟手猫',         // Lv3
  '资深猫',         // Lv4
  '专家猫',         // Lv5
]

const LEVEL_COLORS = [
  '',
  '#94a3b8',  // Lv1 灰
  '#22c55e',  // Lv2 绿
  '#3b82f6',  // Lv3 蓝
  '#a855f7',  // Lv4 紫
  '#f5a623',  // Lv5 橙 (金)
]

const clampedLevel = computed(() => Math.max(1, Math.min(MAX_LEVEL, props.level)))

const title = computed(() => LEVEL_TITLES[clampedLevel.value])
const color = computed(() => LEVEL_COLORS[clampedLevel.value])

const currentBase = computed(() => XP_THRESHOLDS[clampedLevel.value - 1] ?? 0)
const nextTarget = computed(() => {
  const idx = clampedLevel.value
  return XP_THRESHOLDS[Math.min(idx, XP_THRESHOLDS.length - 1)]
})

const isMaxed = computed(() => clampedLevel.value >= MAX_LEVEL)

const progressPct = computed(() => {
  if (isMaxed.value) return 100
  const span = nextTarget.value - currentBase.value
  if (span <= 0) return 0
  const pct = ((props.xp - currentBase.value) / span) * 100
  return Math.max(0, Math.min(100, pct))
})

const stars = computed(() => '★'.repeat(clampedLevel.value) + '☆'.repeat(MAX_LEVEL - clampedLevel.value))
</script>

<template>
  <div class="xp-badge" :class="{ compact }">
    <div class="level-row">
      <span class="level-chip" :style="{ background: color + '20', color: color, borderColor: color }">
        Lv{{ clampedLevel }}
        <span v-if="!compact" class="title">· {{ title }}</span>
      </span>
      <span v-if="!compact" class="stars" :style="{ color }">{{ stars }}</span>
    </div>
    <div class="xp-row">
      <div class="bar" :aria-label="`${xp} / ${nextTarget} XP`">
        <div class="fill" :style="{ width: progressPct + '%', background: color }" />
      </div>
      <span class="xp-text">
        <template v-if="isMaxed">{{ xp }} XP · MAX</template>
        <template v-else>{{ xp }} / {{ nextTarget }} XP</template>
      </span>
    </div>
  </div>
</template>

<style scoped>
.xp-badge {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
}

.xp-badge.compact {
  gap: 2px;
  font-size: 11px;
}

.level-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.level-chip {
  padding: 1px 8px;
  border-radius: 10px;
  border: 1px solid;
  font-weight: 600;
  font-size: 11px;
  white-space: nowrap;
}

.level-chip .title {
  font-weight: 500;
  opacity: 0.85;
}

.stars {
  letter-spacing: 1px;
  font-size: 10px;
}

.xp-row {
  display: flex;
  align-items: center;
  gap: 6px;
}

.bar {
  flex: 1;
  height: 4px;
  background: var(--cc-border);
  border-radius: 2px;
  overflow: hidden;
  min-width: 60px;
}

.xp-badge.compact .bar {
  height: 3px;
  min-width: 40px;
}

.fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.5s ease, background 0.3s ease;
}

.xp-text {
  font-family: monospace;
  opacity: 0.7;
  white-space: nowrap;
  font-size: 10px;
}
</style>
