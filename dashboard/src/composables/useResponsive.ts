import { ref, computed, onMounted, onUnmounted } from 'vue'

/**
 * Vue 3 Composable: 响应式断点系统
 *
 * 设计原则：
 * - 移动优先 (Mobile First)
 * - 基于 CSS 行业标准断点 (Bootstrap/Tailwind/W3C)
 * - 单一数据源 (window.innerWidth)，所有组件共享
 * - 零依赖，纯 Composition API
 */

const windowWidth = ref(typeof window !== 'undefined' ? window.innerWidth : 1024)
const windowHeight = ref(typeof window !== 'undefined' ? window.innerHeight : 768)

// 防抖 resize handler — 30ms 平衡响应速度和性能
let resizeTimer: ReturnType<typeof setTimeout> | null = null
function onResize() {
  if (resizeTimer) clearTimeout(resizeTimer)
  resizeTimer = setTimeout(() => {
    windowWidth.value = window.innerWidth
    windowHeight.value = window.innerHeight
  }, 30)
}

let listenerCount = 0

export function useResponsive() {
  onMounted(() => {
    if (listenerCount === 0) {
      window.addEventListener('resize', onResize)
    }
    listenerCount++
    // 同步一次确保值正确
    windowWidth.value = window.innerWidth
    windowHeight.value = window.innerHeight
  })

  onUnmounted(() => {
    listenerCount--
    if (listenerCount === 0) {
      window.removeEventListener('resize', onResize)
      if (resizeTimer) clearTimeout(resizeTimer)
    }
  })

  // ═══ 断点 (行业标准) ═══
  const isMobile = computed(() => windowWidth.value < 768)
  const isTablet = computed(() => windowWidth.value >= 768 && windowWidth.value < 1024)
  const isDesktop = computed(() => windowWidth.value >= 1024)
  const isLargeDesktop = computed(() => windowWidth.value >= 1440)

  // ═══ 侧栏模式 ═══
  // 移动端：overlay 覆盖（不挤压内容）
  // 平板：icon-only 64px
  // 桌面：full 220px
  const sidebarMode = computed<'overlay' | 'collapsed' | 'full'>(() => {
    if (isMobile.value) return 'overlay'
    if (isTablet.value) return 'collapsed'
    return 'full'
  })

  // 侧栏是否默认折叠
  const sidebarDefaultCollapsed = computed(() => isMobile.value || isTablet.value)

  // 移动端侧栏宽度（overlay 模式用完整宽度）
  const sidebarWidth = computed(() => {
    if (isMobile.value) return 260  // overlay drawer 宽度
    return 220
  })

  const collapsedWidth = computed(() => {
    if (isMobile.value) return 0  // 移动端完全隐藏
    return 64
  })

  // ═══ 内容区宽度 ═══
  const contentWidth = computed(() => {
    const sider = isMobile.value ? 0 : (isTablet.value ? 64 : 220)
    return windowWidth.value - sider
  })

  // ═══ 网格列数 ═══
  // 基于内容区宽度而非窗口宽度，排除侧栏占用
  const gridCols = computed(() => {
    const w = contentWidth.value
    if (w < 400) return 1
    if (w < 640) return 2
    if (w < 900) return 3
    return 4
  })

  // 看板列模式
  const kanbanMode = computed<'stack' | 'scroll' | 'grid'>(() => {
    if (contentWidth.value < 640) return 'stack'   // 垂直堆叠
    if (contentWidth.value < 1024) return 'scroll'  // 水平滚动
    return 'grid'                                   // 正常排列
  })

  // ═══ UI 尺寸 ═══
  const avatarSize = computed(() => isMobile.value ? 48 : isTablet.value ? 64 : 80)
  const cardPadding = computed(() => isMobile.value ? '12px' : '24px 16px 12px')
  const pagePadding = computed(() => isMobile.value ? '8px' : '16px')

  return {
    windowWidth,
    windowHeight,
    isMobile,
    isTablet,
    isDesktop,
    isLargeDesktop,
    sidebarMode,
    sidebarDefaultCollapsed,
    sidebarWidth,
    collapsedWidth,
    contentWidth,
    gridCols,
    kanbanMode,
    avatarSize,
    cardPadding,
    pagePadding,
  }
}
