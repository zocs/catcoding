<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { ref, computed } from 'vue'
const { t } = useI18n()


interface Bug {
  id: string
  level: 'mouse' | 'big_mouse' | 'bat' | 'dragon'
  title: string
  file: string
  status: 'alive' | 'chasing' | 'caught' | 'escaped'
  discoveredAt: number
}

const props = defineProps<{
  bugs: Bug[]
}>()

const emit = defineEmits<{
  catch: [bugId: string]
}>()

const BUG_CONFIG: Record<string, { emoji: string; label: string; color: string; size: string }> = {
  mouse:      { emoji: '🐭', label: t('bugHunter.mouse'), color: '#909399', size: '24px' },
  big_mouse:  { emoji: '🐀', label: t('bugHunter.rat'), color: '#e6a23c', size: '32px' },
  bat:        { emoji: '🦇', label: t('bugHunter.bat'),   color: '#7b68ee', size: '36px' },
  dragon:     { emoji: '🐉', label: t('bugHunter.dragon'),   color: '#f56c6c', size: '48px' },
}

const activeBugs = computed(() => props.bugs.filter(b => b.status === 'alive' || b.status === 'chasing'))
const caughtBugs = computed(() => props.bugs.filter(b => b.status === 'caught'))
const escapedBugs = computed(() => props.bugs.filter(b => b.status === 'escaped'))

function catchBug(bug: Bug) {
  emit('catch', bug.id)
}
</script>

<template>
  <div class="bug-tracker">
    <!-- 统计条 -->
    <div class="bug-stats">
      <div class="stat alive">
        <span class="icon">🐛</span>
        <span class="count">{{ activeBugs.length }}</span>
        <span class="label">{{ t("bugHunter.pending") }}</span>
      </div>
      <div class="stat caught">
        <span class="icon">🐱</span>
        <span class="count">{{ caughtBugs.length }}</span>
        <span class="label">{{ t("bugHunter.caught") }}</span>
      </div>
      <div class="stat escaped">
        <span class="icon">💨</span>
        <span class="count">{{ escapedBugs.length }}</span>
        <span class="label">{{ t("bugHunter.escaped") }}</span>
      </div>
    </div>

    <!-- 活跃的 Bug（带动画） -->
    <div class="bug-arena">
      <TransitionGroup name="bug" tag="div" class="bug-list">
        <div
          v-for="bug in activeBugs"
          :key="bug.id"
          class="bug-item"
          :class="[bug.level, bug.status]"
          @click="catchBug(bug)"
        >
          <!-- Bug 动画图标 -->
          <div class="bug-icon" :style="{ fontSize: BUG_CONFIG[bug.level].size }">
            <span class="bug-emoji" :class="`anim-${bug.level}`">
              {{ BUG_CONFIG[bug.level].emoji }}
            </span>
            <!-- 追赶中的猫爪 -->
            <span v-if="bug.status === 'chasing'" class="cat-paw">🐾</span>
          </div>

          <div class="bug-info">
            <div class="bug-header">
              <span class="bug-level" :style="{ color: BUG_CONFIG[bug.level].color }">
                {{ BUG_CONFIG[bug.level].label }}
              </span>
              <span class="bug-file">{{ bug.file }}</span>
            </div>
            <div class="bug-title">{{ bug.title }}</div>
          </div>

          <!-- 抓捕按钮 -->
          <div class="catch-btn" @click.stop="catchBug(bug)">
            🐾 {{ t("bugHunter.caught") }}!
          </div>
        </div>
      </TransitionGroup>
    </div>

    <!-- 已抓到的 Bug 展示 -->
    <div v-if="caughtBugs.length > 0" class="caught-section">
      <div class="section-title">🐱 {{ t("bugHunter.title") }}</div>
      <div class="caught-grid">
        <div v-for="bug in caughtBugs" :key="bug.id" class="caught-bug">
          <span class="caught-emoji">{{ BUG_CONFIG[bug.level].emoji }}</span>
          <span class="caught-x">❌</span>
          <span class="caught-title">{{ bug.title }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.bug-tracker {
  margin: 16px 0;
}

/* 统计条 */
.bug-stats {
  display: flex;
  gap: 16px;
  margin-bottom: 16px;
}

.stat {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-radius: 20px;
  font-size: 14px;
}

.stat.alive  { background: var(--cc-bg-card); }
.stat.caught { background: var(--cc-bg-card); }
.stat.escaped { background: var(--cc-bg-card); }

.stat .count {
  font-weight: bold;
  font-size: 18px;
}

/* Bug 竞技场 */
.bug-arena {
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  border-radius: 16px;
  padding: 16px;
  min-height: 100px;
  position: relative;
  overflow: hidden;
}

/* 背景星星 */
.bug-arena::before {
  content: '✨ ⭐ ✨';
  position: absolute;
  top: 10px;
  right: 20px;
  font-size: 12px;
  opacity: 0.3;
  animation: twinkle 2s ease-in-out infinite;
}

@keyframes twinkle {
  0%, 100% { opacity: 0.3; }
  50% { opacity: 0.8; }
}

.bug-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.bug-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.3s;
  border: 1px solid transparent;
}

.bug-item:hover {
  background: rgba(255, 255, 255, 0.15);
  border-color: rgba(245, 166, 35, 0.5);
  transform: translateX(4px);
}

/* Bug 动画图标 */
.bug-icon {
  position: relative;
  min-width: 48px;
  text-align: center;
}

/* 小老鼠 — 快速乱窜 */
.anim-mouse {
  display: inline-block;
  animation: mouseRun 0.8s ease-in-out infinite;
}

@keyframes mouseRun {
  0%   { transform: translateX(0) rotate(0deg); }
  25%  { transform: translateX(-6px) rotate(-10deg); }
  50%  { transform: translateX(4px) rotate(5deg); }
  75%  { transform: translateX(-3px) rotate(-5deg); }
  100% { transform: translateX(0) rotate(0deg); }
}

/* 大老鼠 — 缓慢但威胁 */
.anim-big_mouse {
  display: inline-block;
  animation: bigMouseStomp 1.2s ease-in-out infinite;
}

@keyframes bigMouseStomp {
  0%, 100% { transform: scale(1); }
  50%      { transform: scale(1.15); }
}

/* 蝙蝠 — 盘旋飞行 */
.anim-bat {
  display: inline-block;
  animation: batFly 1.5s ease-in-out infinite;
}

@keyframes batFly {
  0%   { transform: translateY(0) rotate(0deg); }
  25%  { transform: translateY(-8px) rotate(10deg); }
  50%  { transform: translateY(-2px) rotate(-5deg); }
  75%  { transform: translateY(-10px) rotate(8deg); }
  100% { transform: translateY(0) rotate(0deg); }
}

/* 恶龙 — 喷火咆哮 */
.anim-dragon {
  display: inline-block;
  animation: dragonRoar 2s ease-in-out infinite;
}

@keyframes dragonRoar {
  0%, 100% { transform: scale(1) rotate(0deg); filter: brightness(1); }
  30%      { transform: scale(1.2) rotate(-5deg); filter: brightness(1.3); }
  60%      { transform: scale(1.1) rotate(3deg); filter: brightness(1.5); }
}

/* 追赶中的猫爪 */
.cat-paw {
  position: absolute;
  top: -8px;
  right: -8px;
  font-size: 18px;
  animation: pawSwing 0.5s ease-in-out infinite;
}

@keyframes pawSwing {
  0%, 100% { transform: rotate(-20deg); }
  50%      { transform: rotate(20deg); }
}

.bug-info {
  flex: 1;
  min-width: 0;
}

.bug-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.bug-level {
  font-weight: bold;
  font-size: 12px;
}

.bug-file {
  font-family: monospace;
  font-size: 11px;
  opacity: 0.5;
}

.bug-title {
  font-size: 13px;
  color: #eee;
}

.catch-btn {
  padding: 6px 12px;
  background: rgba(245, 166, 35, 0.2);
  border: 1px solid rgba(245, 166, 35, 0.5);
  border-radius: 20px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.catch-btn:hover {
  background: rgba(245, 166, 35, 0.5);
  transform: scale(1.1);
}

/* 已抓到的展示 */
.caught-section {
  margin-top: 16px;
  padding: 12px;
  background: var(--cc-bg-card);
  border-radius: 12px;
}

.section-title {
  font-weight: bold;
  margin-bottom: 8px;
}

.caught-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.caught-bug {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  background: white;
  border-radius: 16px;
  font-size: 12px;
  animation: caughtBounce 0.5s ease-out;
}

@keyframes caughtBounce {
  0%   { transform: scale(0); }
  50%  { transform: scale(1.2); }
  100% { transform: scale(1); }
}

.caught-x {
  font-size: 10px;
}

/* TransitionGroup 动画 */
.bug-enter-active {
  transition: all 0.5s ease;
}

.bug-leave-active {
  transition: all 0.3s ease;
}

.bug-enter-from {
  opacity: 0;
  transform: translateX(-30px) scale(0.8);
}

.bug-leave-to {
  opacity: 0;
  transform: translateX(30px) scale(0.5);
}
</style>
