<script setup lang="ts">
/**
 * 猫咪剪影背景特效
 * - 漂浮的猫爪印、小鱼干、毛线球
 * - 鼠标跟随粒子（二次曲线轨迹）
 * - 低开销：requestAnimationFrame + 离屏检测
 */
import { ref, onMounted, onUnmounted } from 'vue'

interface Particle {
  x: number
  y: number
  vx: number
  vy: number
  size: number
  opacity: number
  rotation: number
  rotSpeed: number
  type: 'paw' | 'fish' | 'yarn' | 'star'
  life: number
  maxLife: number
}

interface MouseTrail {
  x: number
  y: number
  age: number
  size: number
  color: string
  // 二次曲线参数
  a: number
  b: number
  startX: number
  startY: number
  progress: number
  speed: number
}

const canvas = ref<HTMLCanvasElement | null>(null)
let ctx: CanvasRenderingContext2D | null = null
let animId = 0
let particles: Particle[] = []
let trails: MouseTrail[] = []
let mouseX = 0
let mouseY = 0
let lastTrailTime = 0

// 猫爪 SVG path 简化为绘制函数
function drawPaw(c: CanvasRenderingContext2D, x: number, y: number, size: number, opacity: number) {
  c.save()
  c.globalAlpha = opacity
  c.fillStyle = '#f5a623'
  c.translate(x, y)

  // 掌垫
  c.beginPath()
  c.ellipse(0, size * 0.2, size * 0.45, size * 0.35, 0, 0, Math.PI * 2)
  c.fill()

  // 四个趾垫
  const toes = [
    [-size * 0.35, -size * 0.2, size * 0.2],
    [-size * 0.12, -size * 0.4, size * 0.2],
    [size * 0.12, -size * 0.4, size * 0.2],
    [size * 0.35, -size * 0.2, size * 0.2],
  ]
  for (const [tx, ty, ts] of toes) {
    c.beginPath()
    c.ellipse(tx as number, ty as number, ts as number, ts as number * 0.85, 0, 0, Math.PI * 2)
    c.fill()
  }

  c.restore()
}

function drawFish(c: CanvasRenderingContext2D, x: number, y: number, size: number, opacity: number) {
  c.save()
  c.globalAlpha = opacity
  c.translate(x, y)

  // 鱼身
  c.fillStyle = '#ff8a65'
  c.beginPath()
  c.ellipse(0, 0, size * 0.6, size * 0.3, 0, 0, Math.PI * 2)
  c.fill()

  // 尾巴
  c.beginPath()
  c.moveTo(size * 0.5, 0)
  c.lineTo(size * 0.9, -size * 0.3)
  c.lineTo(size * 0.9, size * 0.3)
  c.closePath()
  c.fill()

  // 眼睛
  c.fillStyle = '#333'
  c.beginPath()
  c.arc(-size * 0.25, -size * 0.05, size * 0.07, 0, Math.PI * 2)
  c.fill()

  c.restore()
}

function drawYarn(c: CanvasRenderingContext2D, x: number, y: number, size: number, opacity: number) {
  c.save()
  c.globalAlpha = opacity
  c.translate(x, y)
  c.strokeStyle = '#e91e63'
  c.lineWidth = size * 0.12
  c.lineCap = 'round'

  // 毛线球
  c.beginPath()
  c.arc(0, 0, size * 0.4, 0, Math.PI * 2)
  c.stroke()

  // 缠绕线
  for (let i = 0; i < 3; i++) {
    c.beginPath()
    const offset = (i - 1) * size * 0.15
    c.arc(offset, 0, size * 0.35, -0.8, 0.8)
    c.stroke()
  }

  // 线头
  c.beginPath()
  c.moveTo(size * 0.35, size * 0.15)
  c.quadraticCurveTo(size * 0.6, size * 0.4, size * 0.5, size * 0.6)
  c.stroke()

  c.restore()
}

function drawStar(c: CanvasRenderingContext2D, x: number, y: number, size: number, opacity: number) {
  c.save()
  c.globalAlpha = opacity
  c.fillStyle = '#ffd54f'
  c.translate(x, y)

  c.beginPath()
  for (let i = 0; i < 5; i++) {
    const angle = (i * 4 * Math.PI) / 5 - Math.PI / 2
    const r = i === 0 ? size * 0.4 : size * 0.4
    if (i === 0) c.moveTo(Math.cos(angle) * r, Math.sin(angle) * r)
    else c.lineTo(Math.cos(angle) * r, Math.sin(angle) * r)
  }
  c.closePath()
  c.fill()

  c.restore()
}

const drawFns = { paw: drawPaw, fish: drawFish, yarn: drawYarn, star: drawStar }

function spawnFloatingParticle(w: number, h: number) {
  const types: Particle['type'][] = ['paw', 'fish', 'yarn', 'star']
  particles.push({
    x: Math.random() * w,
    y: h + 30,
    vx: (Math.random() - 0.5) * 0.4,
    vy: -(0.3 + Math.random() * 0.5),
    size: 8 + Math.random() * 14,
    opacity: 0.08 + Math.random() * 0.12,
    rotation: Math.random() * Math.PI * 2,
    rotSpeed: (Math.random() - 0.5) * 0.01,
    type: types[Math.floor(Math.random() * types.length)],
    life: 0,
    maxLife: 400 + Math.random() * 300,
  })
}

function spawnMouseTrail(x: number, y: number) {
  const colors = ['#f5a623', '#ff8a65', '#e91e63', '#ffd54f', '#81c784']
  // 二次曲线：向上抛物线 + 随机水平偏移
  const dir = Math.random() > 0.5 ? 1 : -1
  trails.push({
    x,
    y,
    age: 0,
    size: 3 + Math.random() * 6,
    color: colors[Math.floor(Math.random() * colors.length)],
    a: -(0.005 + Math.random() * 0.008), // 抛物线开口向下
    b: (Math.random() - 0.5) * 3,
    startX: x,
    startY: y,
    progress: 0,
    speed: 0.02 + Math.random() * 0.02,
  })
}

function animate() {
  if (!ctx || !canvas.value) return
  const c = ctx
  const w = canvas.value.width
  const h = canvas.value.height

  c.clearRect(0, 0, w, h)

  // 更新 & 绘制漂浮粒子
  for (let i = particles.length - 1; i >= 0; i--) {
    const p = particles[i]
    p.x += p.vx
    p.y += p.vy
    p.rotation += p.rotSpeed
    p.life++

    // 淡入淡出
    const fadeIn = Math.min(p.life / 60, 1)
    const fadeOut = Math.max(1 - (p.life - p.maxLife + 60) / 60, 0)
    const alpha = p.opacity * fadeIn * (p.life > p.maxLife - 60 ? fadeOut : 1)

    if (p.life > p.maxLife || p.y < -30) {
      particles.splice(i, 1)
      continue
    }

    c.save()
    c.translate(p.x, p.y)
    c.rotate(p.rotation)
    drawFns[p.type](c, 0, 0, p.size, alpha)
    c.restore()
  }

  // 更新 & 绘制鼠标轨迹（二次曲线）
  for (let i = trails.length - 1; i >= 0; i--) {
    const t = trails[i]
    t.progress += t.speed
    t.age++

    if (t.progress > 1 || t.age > 80) {
      trails.splice(i, 1)
      continue
    }

    // 二次曲线位置
    const dx = t.progress * 60 * (t.b > 0 ? 1 : -1)
    const dy = -(t.progress * 80)
    const curveY = t.a * dx * dx // 抛物线

    const px = t.startX + dx + t.b * t.progress * 20
    const py = t.startY + dy + curveY

    const alpha = (1 - t.progress) * 0.7

    c.save()
    c.globalAlpha = alpha
    c.fillStyle = t.color
    c.beginPath()

    // 小猫爪印形状
    const s = t.size * (1 - t.progress * 0.3)
    // 掌垫
    c.ellipse(px, py + s * 0.15, s * 0.4, s * 0.3, 0, 0, Math.PI * 2)
    c.fill()
    // 两个小趾
    c.beginPath()
    c.arc(px - s * 0.25, py - s * 0.15, s * 0.15, 0, Math.PI * 2)
    c.fill()
    c.beginPath()
    c.arc(px + s * 0.25, py - s * 0.15, s * 0.15, 0, Math.PI * 2)
    c.fill()

    c.restore()
  }

  // 保持漂浮粒子数量
  if (particles.length < 15 && Math.random() < 0.03) {
    spawnFloatingParticle(w, h)
  }

  animId = requestAnimationFrame(animate)
}

function onMouseMove(e: MouseEvent) {
  const now = Date.now()
  if (now - lastTrailTime < 50) return // 节流 50ms
  lastTrailTime = now

  if (!canvas.value) return
  const rect = canvas.value.getBoundingClientRect()
  mouseX = e.clientX - rect.left
  mouseY = e.clientY - rect.top

  spawnMouseTrail(mouseX, mouseY)
}

function onResize() {
  if (!canvas.value) return
  canvas.value.width = window.innerWidth
  canvas.value.height = window.innerHeight
}

onMounted(() => {
  if (!canvas.value) return
  ctx = canvas.value.getContext('2d')
  onResize()

  // 初始粒子
  for (let i = 0; i < 8; i++) {
    spawnFloatingParticle(canvas.value.width, canvas.value.height)
  }

  window.addEventListener('mousemove', onMouseMove)
  window.addEventListener('resize', onResize)
  animId = requestAnimationFrame(animate)
})

onUnmounted(() => {
  cancelAnimationFrame(animId)
  window.removeEventListener('mousemove', onMouseMove)
  window.removeEventListener('resize', onResize)
})
</script>

<template>
  <canvas
    ref="canvas"
    class="cat-bg-canvas"
  />
</template>

<style scoped>
.cat-bg-canvas {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  pointer-events: none;
  z-index: 0;
  opacity: 0.85;
}
</style>
