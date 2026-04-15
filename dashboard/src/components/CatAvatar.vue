<template>
  <div class="cat-avatar" :class="[size, status]">
    <div class="avatar-container">
      <span class="emoji">{{ emoji }}</span>
      <div v-if="showStatus" class="status-indicator" :class="status"></div>
    </div>
    <div v-if="showName" class="avatar-name">{{ name }}</div>
    <div v-if="showAnimation" class="animation">
      <!-- 工作中动画 -->
      <div v-if="status === 'working'" class="working-animation">
        <span class="key">⌨️</span>
      </div>
      <!-- 等待中动画 -->
      <div v-else-if="status === 'idle'" class="idle-animation">
        <span class="zzz">💤</span>
      </div>
      <!-- 完成动画 -->
      <div v-else-if="status === 'done'" class="done-animation">
        <span class="check">✅</span>
      </div>
      <!-- 错误动画 -->
      <div v-else-if="status === 'error'" class="error-animation">
        <span class="confused">🙀</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Props {
  emoji: string
  name?: string
  status?: 'working' | 'idle' | 'done' | 'error' | 'restarting'
  size?: 'small' | 'medium' | 'large'
  showName?: boolean
  showStatus?: boolean
  showAnimation?: boolean
}

withDefaults(defineProps<Props>(), {
  name: '',
  status: 'idle',
  size: 'medium',
  showName: false,
  showStatus: true,
  showAnimation: false,
})
</script>

<style scoped>
.cat-avatar {
  display: inline-flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.avatar-container {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: linear-gradient(135deg, var(--cc-bg-sider) 0%, var(--cc-bg-input) 100%);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

/* 大小 */
.cat-avatar.small .avatar-container {
  width: 40px;
  height: 40px;
}

.cat-avatar.medium .avatar-container {
  width: 64px;
  height: 64px;
}

.cat-avatar.large .avatar-container {
  width: 96px;
  height: 96px;
}

.emoji {
  font-size: 24px;
}

.cat-avatar.small .emoji {
  font-size: 20px;
}

.cat-avatar.large .emoji {
  font-size: 40px;
}

/* 状态指示器 */
.status-indicator {
  position: absolute;
  bottom: 2px;
  right: 2px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 2px solid var(--cc-bg-sider);
}

.status-indicator.working {
  background: var(--cc-blue);
  animation: pulse 1.5s infinite;
}

.status-indicator.idle {
  background: var(--cc-fg-secondary);
}

.status-indicator.done {
  background: var(--cc-accent);
}

.status-indicator.error {
  background: var(--cc-red);
}

.status-indicator.restarting {
  background: var(--cc-yellow);
  animation: spin 1s linear infinite;
}

/* 名字 */
.avatar-name {
  font-size: 12px;
  color: var(--cc-fg-muted);
  text-align: center;
}

/* 动画 */
.animation {
  position: absolute;
  top: -20px;
  right: -10px;
}

.working-animation .key {
  font-size: 16px;
  animation: typing 0.5s infinite;
}

.idle-animation .zzz {
  font-size: 16px;
  animation: float 2s ease-in-out infinite;
}

.done-animation .check {
  font-size: 16px;
  animation: bounce 0.5s ease-out;
}

.error-animation .confused {
  font-size: 16px;
  animation: shake 0.5s ease-out;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@keyframes typing {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-3px); }
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-5px); }
}

@keyframes bounce {
  0% { transform: scale(0); }
  50% { transform: scale(1.2); }
  100% { transform: scale(1); }
}

@keyframes shake {
  0%, 100% { transform: translateX(0); }
  25% { transform: translateX(-5px); }
  75% { transform: translateX(5px); }
}
</style>
