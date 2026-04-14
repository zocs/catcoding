<script setup lang="ts">
/**
 * CatAvatarSVG — 每个角色独特的 SVG 猫咪头像
 * 体现角色性格特点，支持动画状态
 */
import { computed } from 'vue'

interface Props {
  role: string
  status?: 'idle' | 'active' | 'busy' | 'error'
  size?: number
  animated?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  status: 'idle',
  size: 64,
  animated: false,
})

const statusColor = computed(() => {
  switch (props.status) {
    case 'active': return '#34d399'
    case 'busy':   return '#f5a623'
    case 'error':  return '#ef4444'
    default:       return '#6b7280'
  }
})
</script>

<template>
  <div
    class="cat-svg-avatar"
    :class="[status, { animated }]"
    :style="{ width: size + 'px', height: size + 'px' }"
  >
    <!-- ═══ 暹罗猫 PM — 尖脸、深色耳朵、杏仁眼 ═══ -->
    <svg v-if="role === 'pm'" viewBox="0 0 100 100" class="cat-svg">
      <!-- 脸 -->
      <ellipse cx="50" cy="55" rx="32" ry="30" fill="#F5E6D3"/>
      <!-- 耳朵（深色 — 暹罗特征） -->
      <polygon points="22,35 32,8 42,32" fill="#3D2B1F"/>
      <polygon points="58,32 68,8 78,35" fill="#3D2B1F"/>
      <polygon points="26,33 33,14 40,31" fill="#D4A574"/>
      <polygon points="60,31 67,14 74,33" fill="#D4A574"/>
      <!-- 面部深色区域（暹罗重点色） -->
      <ellipse cx="50" cy="62" rx="22" ry="18" fill="#D4A574" opacity="0.6"/>
      <!-- 眼睛 — 杏仁形，透着精明 -->
      <ellipse cx="38" cy="50" rx="7" ry="5" fill="white"/>
      <ellipse cx="62" cy="50" rx="7" ry="5" fill="white"/>
      <ellipse cx="39" cy="50" rx="4" ry="4.5" fill="#2D5A27"/>
      <ellipse cx="63" cy="50" rx="4" ry="4.5" fill="#2D5A27"/>
      <ellipse cx="39" cy="50" rx="2.5" ry="3" fill="#111"/>
      <ellipse cx="63" cy="50" rx="2.5" ry="3" fill="#111"/>
      <circle cx="37" cy="48" r="1.5" fill="white" opacity="0.8"/>
      <circle cx="61" cy="48" r="1.5" fill="white" opacity="0.8"/>
      <!-- 鼻子 -->
      <path d="M47,58 L50,62 L53,58" fill="#3D2B1F"/>
      <!-- 嘴 — 自信微笑 -->
      <path d="M44,64 Q50,68 56,64" fill="none" stroke="#3D2B1F" stroke-width="1.5" stroke-linecap="round"/>
      <!-- 胡须 -->
      <line x1="18" y1="56" x2="34" y2="58" stroke="#C4A882" stroke-width="0.8"/>
      <line x1="16" y1="62" x2="33" y2="62" stroke="#C4A882" stroke-width="0.8"/>
      <line x1="66" y1="58" x2="82" y2="56" stroke="#C4A882" stroke-width="0.8"/>
      <line x1="67" y1="62" x2="84" y2="62" stroke="#C4A882" stroke-width="0.8"/>
      <!-- 王冠（PM 的标志） -->
      <polygon points="42,24 45,16 50,22 55,16 58,24" fill="#FFD700" stroke="#DAA520" stroke-width="0.8"/>
      <circle cx="50" cy="18" r="2" fill="#FF6B6B"/>
    </svg>

    <!-- ═══ 英短蓝猫 Core Dev — 圆脸、胖腮、沉稳 ═══ -->
    <svg v-else-if="role === 'core_dev'" viewBox="0 0 100 100" class="cat-svg">
      <!-- 大圆脸 -->
      <circle cx="50" cy="55" r="32" fill="#8B9DAF"/>
      <!-- 胖腮 -->
      <ellipse cx="32" cy="60" rx="14" ry="12" fill="#9BAFBD"/>
      <ellipse cx="68" cy="60" rx="14" ry="12" fill="#9BAFBD"/>
      <!-- 耳朵 -->
      <polygon points="24,32 30,6 40,28" fill="#6B7D8F"/>
      <polygon points="60,28 70,6 76,32" fill="#6B7D8F"/>
      <polygon points="28,30 32,12 38,27" fill="#A8B8C8"/>
      <polygon points="62,27 68,12 72,30" fill="#A8B8C8"/>
      <!-- 眼睛 — 大而圆，铜色（英短特征） -->
      <ellipse cx="38" cy="48" rx="8" ry="8" fill="#FFFAF0"/>
      <ellipse cx="62" cy="48" rx="8" ry="8" fill="#FFFAF0"/>
      <ellipse cx="38" cy="48" rx="5" ry="5.5" fill="#D4851F"/>
      <ellipse cx="62" cy="48" rx="5" ry="5.5" fill="#D4851F"/>
      <ellipse cx="38" cy="48" rx="3" ry="3.5" fill="#111"/>
      <ellipse cx="62" cy="48" rx="3" ry="3.5" fill="#111"/>
      <circle cx="36" cy="46" r="2" fill="white" opacity="0.9"/>
      <circle cx="60" cy="46" r="2" fill="white" opacity="0.9"/>
      <!-- 鼻子 -->
      <ellipse cx="50" cy="58" rx="4" ry="3" fill="#D4A0A0"/>
      <!-- 嘴 — 憨厚 -->
      <path d="M46,62 Q50,65 54,62" fill="none" stroke="#5A6B7A" stroke-width="1.5" stroke-linecap="round"/>
      <!-- 胡须 -->
      <line x1="16" y1="55" x2="30" y2="57" stroke="#A8B8C8" stroke-width="0.8"/>
      <line x1="14" y1="62" x2="29" y2="62" stroke="#A8B8C8" stroke-width="0.8"/>
      <line x1="70" y1="57" x2="84" y2="55" stroke="#A8B8C8" stroke-width="0.8"/>
      <line x1="71" y1="62" x2="86" y2="62" stroke="#A8B8C8" stroke-width="0.8"/>
      <!-- 键盘（核心开发者标志） -->
      <rect x="36" y="76" width="28" height="8" rx="2" fill="#4A5568"/>
      <rect x="39" y="78" width="4" height="3" rx="0.5" fill="#A0AEC0"/>
      <rect x="45" y="78" width="4" height="3" rx="0.5" fill="#A0AEC0"/>
      <rect x="51" y="78" width="4" height="3" rx="0.5" fill="#A0AEC0"/>
      <rect x="57" y="78" width="4" height="3" rx="0.5" fill="#A0AEC0"/>
    </svg>

    <!-- ═══ 橘猫 Frontend — 温暖橙色、大笑脸 ═══ -->
    <svg v-else-if="role === 'frontend'" viewBox="0 0 100 100" class="cat-svg">
      <!-- 脸 -->
      <ellipse cx="50" cy="55" rx="32" ry="30" fill="#F5A623"/>
      <!-- 橘色条纹 -->
      <path d="M30,40 Q35,35 40,40" fill="none" stroke="#E8941A" stroke-width="2" opacity="0.5"/>
      <path d="M60,40 Q65,35 70,40" fill="none" stroke="#E8941A" stroke-width="2" opacity="0.5"/>
      <!-- 耳朵 -->
      <polygon points="22,35 30,8 40,30" fill="#E8941A"/>
      <polygon points="60,30 70,8 78,35" fill="#E8941A"/>
      <polygon points="26,33 32,14 38,30" fill="#FFCC80"/>
      <polygon points="62,30 68,14 74,33" fill="#FFCC80"/>
      <!-- 眼睛 — 开心眯眼 -->
      <path d="M32,48 Q38,42 44,48" fill="none" stroke="#3D2B1F" stroke-width="2.5" stroke-linecap="round"/>
      <path d="M56,48 Q62,42 68,48" fill="none" stroke="#3D2B1F" stroke-width="2.5" stroke-linecap="round"/>
      <!-- 鼻子 -->
      <path d="M47,56 L50,60 L53,56" fill="#E87040"/>
      <!-- 大笑脸 -->
      <path d="M40,62 Q50,72 60,62" fill="none" stroke="#3D2B1F" stroke-width="2" stroke-linecap="round"/>
      <!-- 腮红 -->
      <ellipse cx="32" cy="58" rx="6" ry="4" fill="#FF8A80" opacity="0.4"/>
      <ellipse cx="68" cy="58" rx="6" ry="4" fill="#FF8A80" opacity="0.4"/>
      <!-- 胡须 -->
      <line x1="18" y1="56" x2="32" y2="58" stroke="#E8C880" stroke-width="0.8"/>
      <line x1="16" y1="62" x2="31" y2="62" stroke="#E8C880" stroke-width="0.8"/>
      <line x1="68" y1="58" x2="82" y2="56" stroke="#E8C880" stroke-width="0.8"/>
      <line x1="69" y1="62" x2="84" y2="62" stroke="#E8C880" stroke-width="0.8"/>
      <!-- 画笔（前端标志） -->
      <line x1="76" y1="72" x2="84" y2="88" stroke="#FF6B6B" stroke-width="3" stroke-linecap="round"/>
      <circle cx="84" cy="89" r="3" fill="#4ECDC4"/>
    </svg>

    <!-- ═══ 缅因猫 Backend — 大而威严、耳簇毛 ═══ -->
    <svg v-else-if="role === 'backend'" viewBox="0 0 100 100" class="cat-svg">
      <!-- 大脸 -->
      <ellipse cx="50" cy="56" rx="34" ry="32" fill="#6B5B4D"/>
      <!-- 毛茸茸的脸颊 -->
      <ellipse cx="30" cy="58" rx="10" ry="14" fill="#7D6E5F"/>
      <ellipse cx="70" cy="58" rx="10" ry="14" fill="#7D6E5F"/>
      <!-- 大耳朵（带簇毛） -->
      <polygon points="18,38 26,2 42,30" fill="#5A4A3C"/>
      <polygon points="58,30 74,2 82,38" fill="#5A4A3C"/>
      <polygon points="22,36 28,10 38,30" fill="#8B7B6B"/>
      <polygon points="62,30 72,10 78,36" fill="#8B7B6B"/>
      <!-- 耳簇毛 -->
      <line x1="28" y1="12" x2="24" y2="6" stroke="#6B5B4D" stroke-width="1.5"/>
      <line x1="30" y1="10" x2="32" y2="4" stroke="#6B5B4D" stroke-width="1.5"/>
      <line x1="70" y1="10" x2="68" y2="4" stroke="#6B5B4D" stroke-width="1.5"/>
      <line x1="72" y1="12" x2="76" y2="6" stroke="#6B5B4D" stroke-width="1.5"/>
      <!-- 眼睛 — 威严大眼 -->
      <ellipse cx="38" cy="50" rx="7" ry="6" fill="#FFFAF0"/>
      <ellipse cx="62" cy="50" rx="7" ry="6" fill="#FFFAF0"/>
      <ellipse cx="38" cy="50" rx="4.5" ry="5" fill="#7B6B00"/>
      <ellipse cx="62" cy="50" rx="4.5" ry="5" fill="#7B6B00"/>
      <ellipse cx="38" cy="50" rx="2.5" ry="3.5" fill="#111"/>
      <ellipse cx="62" cy="50" rx="2.5" ry="3.5" fill="#111"/>
      <circle cx="36" cy="48" r="1.5" fill="white" opacity="0.9"/>
      <circle cx="60" cy="48" r="1.5" fill="white" opacity="0.9"/>
      <!-- 鼻子 -->
      <path d="M47,58 L50,62 L53,58" fill="#3D2B1F"/>
      <!-- 严肃的嘴 -->
      <line x1="44" y1="65" x2="56" y2="65" stroke="#3D2B1F" stroke-width="1.5" stroke-linecap="round"/>
      <!-- 鬃毛（缅因特征） -->
      <path d="M18,40 Q14,50 16,65" fill="none" stroke="#7D6E5F" stroke-width="3"/>
      <path d="M82,40 Q86,50 84,65" fill="none" stroke="#7D6E5F" stroke-width="3"/>
      <!-- 胡须 -->
      <line x1="14" y1="55" x2="30" y2="58" stroke="#8B7B6B" stroke-width="1"/>
      <line x1="12" y1="62" x2="28" y2="62" stroke="#8B7B6B" stroke-width="1"/>
      <line x1="70" y1="58" x2="86" y2="55" stroke="#8B7B6B" stroke-width="1"/>
      <line x1="72" y1="62" x2="88" y2="62" stroke="#8B7B6B" stroke-width="1"/>
      <!-- 盾牌（后端守护标志） -->
      <path d="M44,78 L50,74 L56,78 L56,86 Q50,92 44,86 Z" fill="#4A5568" stroke="#2D3748" stroke-width="1"/>
      <text x="50" y="85" text-anchor="middle" fill="#A0AEC0" font-size="6" font-weight="bold">API</text>
    </svg>

    <!-- ═══ 玄猫 Reviewer — 纯黑、锐利绿眼 ═══ -->
    <svg v-else-if="role === 'reviewer'" viewBox="0 0 100 100" class="cat-svg">
      <!-- 黑脸 -->
      <ellipse cx="50" cy="55" rx="31" ry="30" fill="#1A1A2E"/>
      <!-- 耳朵 -->
      <polygon points="24,34 30,6 40,28" fill="#111122"/>
      <polygon points="60,28 70,6 76,34" fill="#111122"/>
      <polygon points="28,32 32,12 38,28" fill="#2A2A3E"/>
      <polygon points="62,28 68,12 72,32" fill="#2A2A3E"/>
      <!-- 发光的绿眼睛（关键特征！） -->
      <ellipse cx="38" cy="48" rx="7" ry="6" fill="#00FF88" opacity="0.3"/>
      <ellipse cx="62" cy="48" rx="7" ry="6" fill="#00FF88" opacity="0.3"/>
      <ellipse cx="38" cy="48" rx="6" ry="5" fill="#00FF88"/>
      <ellipse cx="62" cy="48" rx="6" ry="5" fill="#00FF88"/>
      <!-- 竖瞳 -->
      <ellipse cx="38" cy="48" rx="2" ry="5" fill="#111"/>
      <ellipse cx="62" cy="48" rx="2" ry="5" fill="#111"/>
      <circle cx="36" cy="46" r="1.5" fill="white" opacity="0.7"/>
      <circle cx="60" cy="46" r="1.5" fill="white" opacity="0.7"/>
      <!-- 鼻子 -->
      <path d="M47,58 L50,61 L53,58" fill="#4A4A5E"/>
      <!-- 冷峻的嘴 -->
      <path d="M46,64 Q50,66 54,64" fill="none" stroke="#4A4A5E" stroke-width="1.2" stroke-linecap="round"/>
      <!-- 胡须 -->
      <line x1="16" y1="56" x2="32" y2="58" stroke="#3A3A4E" stroke-width="0.6"/>
      <line x1="14" y1="62" x2="31" y2="62" stroke="#3A3A4E" stroke-width="0.6"/>
      <line x1="68" y1="58" x2="84" y2="56" stroke="#3A3A4E" stroke-width="0.6"/>
      <line x1="69" y1="62" x2="86" y2="62" stroke="#3A3A4E" stroke-width="0.6"/>
      <!-- 放大镜（审查标志） -->
      <circle cx="78" cy="78" r="8" fill="none" stroke="#00FF88" stroke-width="2" opacity="0.6"/>
      <line x1="84" y1="84" x2="90" y2="90" stroke="#00FF88" stroke-width="2.5" stroke-linecap="round" opacity="0.6"/>
    </svg>

    <!-- ═══ 阿比西尼亚猫 Tester — 跳色花纹、好奇大眼 ═══ -->
    <svg v-else-if="role === 'tester'" viewBox="0 0 100 100" class="cat-svg">
      <!-- 脸 -->
      <ellipse cx="50" cy="55" rx="30" ry="28" fill="#D4853F"/>
      <!-- 跳色花纹（阿比特征 — 每根毛多色段） -->
      <ellipse cx="50" cy="55" rx="26" ry="24" fill="#C67A35" opacity="0.5"/>
      <ellipse cx="50" cy="55" rx="20" ry="18" fill="#D4853F" opacity="0.4"/>
      <!-- 大而尖的耳朵（好奇） -->
      <polygon points="22,36 26,2 38,28" fill="#B56A2F"/>
      <polygon points="62,28 74,2 78,36" fill="#B56A2F"/>
      <polygon points="26,34 28,10 36,28" fill="#E8A060"/>
      <polygon points="64,28 72,10 74,34" fill="#E8A060"/>
      <!-- 好奇的大眼睛 -->
      <ellipse cx="38" cy="48" rx="8" ry="8" fill="#FFFAF0"/>
      <ellipse cx="62" cy="48" rx="8" ry="8" fill="#FFFAF0"/>
      <ellipse cx="38" cy="48" rx="5.5" ry="6" fill="#8B6914"/>
      <ellipse cx="62" cy="48" rx="5.5" ry="6" fill="#8B6914"/>
      <ellipse cx="38" cy="48" rx="3" ry="4" fill="#111"/>
      <ellipse cx="62" cy="48" rx="3" ry="4" fill="#111"/>
      <circle cx="36" cy="46" r="2" fill="white" opacity="0.9"/>
      <circle cx="60" cy="46" r="2" fill="white" opacity="0.9"/>
      <!-- 鼻子 -->
      <path d="M47,58 L50,61 L53,58" fill="#8B4513"/>
      <!-- 好奇张嘴 -->
      <ellipse cx="50" cy="66" rx="4" ry="3" fill="#FFB6A0"/>
      <ellipse cx="50" cy="65" rx="3" ry="2" fill="#8B4513"/>
      <!-- 胡须 -->
      <line x1="18" y1="55" x2="32" y2="57" stroke="#E8C080" stroke-width="0.8"/>
      <line x1="16" y1="62" x2="31" y2="62" stroke="#E8C080" stroke-width="0.8"/>
      <line x1="68" y1="57" x2="82" y2="55" stroke="#E8C080" stroke-width="0.8"/>
      <line x1="69" y1="62" x2="84" y2="62" stroke="#E8C080" stroke-width="0.8"/>
      <!-- Bug 放大镜（测试标志） -->
      <text x="78" y="82" font-size="16">🐛</text>
    </svg>

    <!-- ═══ 狸花猫 Deploy — 条纹花纹、独立 ═══ -->
    <svg v-else-if="role === 'deploy'" viewBox="0 0 100 100" class="cat-svg">
      <!-- 脸 -->
      <ellipse cx="50" cy="55" rx="31" ry="29" fill="#8B8B5A"/>
      <!-- 条纹（狸花特征） -->
      <path d="M30,35 Q35,45 30,55" fill="none" stroke="#6B6B3A" stroke-width="3" opacity="0.6"/>
      <path d="M38,30 Q42,42 38,55" fill="none" stroke="#6B6B3A" stroke-width="2.5" opacity="0.5"/>
      <path d="M62,30 Q58,42 62,55" fill="none" stroke="#6B6B3A" stroke-width="2.5" opacity="0.5"/>
      <path d="M70,35 Q65,45 70,55" fill="none" stroke="#6B6B3A" stroke-width="3" opacity="0.6"/>
      <!-- M 额纹（狸花特征） -->
      <path d="M38,32 L42,26 L46,32 L50,26 L54,32 L58,26 L62,32" fill="none" stroke="#6B6B3A" stroke-width="2"/>
      <!-- 耳朵 -->
      <polygon points="24,34 30,6 40,28" fill="#7A7A4A"/>
      <polygon points="60,28 70,6 76,34" fill="#7A7A4A"/>
      <polygon points="28,32 32,12 38,28" fill="#A0A070"/>
      <polygon points="62,28 68,12 72,32" fill="#A0A070"/>
      <!-- 眼睛 — 自信 -->
      <ellipse cx="38" cy="48" rx="7" ry="6" fill="#FFFAF0"/>
      <ellipse cx="62" cy="48" rx="7" ry="6" fill="#FFFAF0"/>
      <ellipse cx="38" cy="48" rx="4" ry="4.5" fill="#5A8B00"/>
      <ellipse cx="62" cy="48" rx="4" ry="4.5" fill="#5A8B00"/>
      <ellipse cx="38" cy="48" rx="2.5" ry="3" fill="#111"/>
      <ellipse cx="62" cy="48" rx="2.5" ry="3" fill="#111"/>
      <circle cx="36" cy="46" r="1.5" fill="white" opacity="0.8"/>
      <circle cx="60" cy="46" r="1.5" fill="white" opacity="0.8"/>
      <!-- 鼻子 -->
      <path d="M47,58 L50,62 L53,58" fill="#5A5A3A"/>
      <!-- 嘴 -->
      <path d="M45,64 Q50,67 55,64" fill="none" stroke="#5A5A3A" stroke-width="1.3" stroke-linecap="round"/>
      <!-- 胡须 -->
      <line x1="16" y1="55" x2="32" y2="57" stroke="#A0A070" stroke-width="0.8"/>
      <line x1="14" y1="62" x2="30" y2="62" stroke="#A0A070" stroke-width="0.8"/>
      <line x1="68" y1="57" x2="84" y2="55" stroke="#A0A070" stroke-width="0.8"/>
      <line x1="70" y1="62" x2="86" y2="62" stroke="#A0A070" stroke-width="0.8"/>
      <!-- 火箭（部署标志） -->
      <text x="46" y="82" font-size="16">🚀</text>
    </svg>

    <!-- ═══ 猫头鹰 Watchdog — 大圆眼、不睡觉 ═══ -->
    <svg v-else-if="role === 'watchdog'" viewBox="0 0 100 100" class="cat-svg">
      <!-- 脸 -->
      <circle cx="50" cy="52" r="30" fill="#8B6914"/>
      <!-- 羽毛纹 -->
      <ellipse cx="50" cy="52" rx="26" ry="26" fill="#A07B1A" opacity="0.5"/>
      <!-- 耳羽 -->
      <polygon points="20,30 30,8 38,28" fill="#6B5010"/>
      <polygon points="62,28 70,8 80,30" fill="#6B5010"/>
      <!-- 面盘（猫头鹰特征 — 心形） -->
      <path d="M50,30 Q30,35 30,55 Q30,72 50,75 Q70,72 70,55 Q70,35 50,30" fill="#D4A850"/>
      <!-- 超大圆眼睛（永远不睡觉！） -->
      <circle cx="38" cy="48" r="10" fill="#FFFAF0"/>
      <circle cx="62" cy="48" r="10" fill="#FFFAF0"/>
      <circle cx="38" cy="48" r="7" fill="#FF8C00"/>
      <circle cx="62" cy="48" r="7" fill="#FF8C00"/>
      <circle cx="38" cy="48" r="4" fill="#111"/>
      <circle cx="62" cy="48" r="4" fill="#111"/>
      <circle cx="36" cy="45" r="2.5" fill="white" opacity="0.9"/>
      <circle cx="60" cy="45" r="2.5" fill="white" opacity="0.9"/>
      <!-- 喙 -->
      <path d="M47,58 L50,64 L53,58" fill="#FF8C00"/>
      <!-- 监控符号 -->
      <circle cx="78" cy="78" r="7" fill="none" stroke="#34d399" stroke-width="2"/>
      <circle cx="78" cy="78" r="3" fill="#34d399"/>
      <line x1="78" y1="71" x2="78" y2="68" stroke="#34d399" stroke-width="1.5"/>
    </svg>

    <!-- ═══ 狐狸 Tech Scout — 尖脸、聪明 ═══ -->
    <svg v-else-if="role === 'tech_scout'" viewBox="0 0 100 100" class="cat-svg">
      <!-- 尖脸 -->
      <path d="M20,40 Q20,20 50,18 Q80,20 80,40 L75,70 Q50,80 25,70 Z" fill="#E8650A"/>
      <!-- 白色面颊 -->
      <path d="M30,45 Q50,55 70,45 L65,70 Q50,75 35,70 Z" fill="#FFF5E0"/>
      <!-- 大尖耳朵 -->
      <polygon points="22,38 28,2 42,30" fill="#E8650A"/>
      <polygon points="58,30 72,2 78,38" fill="#E8650A"/>
      <polygon points="26,36 30,10 40,30" fill="#1A1A2E"/>
      <polygon points="60,30 70,10 74,36" fill="#1A1A2E"/>
      <!-- 聪明的眼睛 -->
      <ellipse cx="38" cy="46" rx="6" ry="5.5" fill="#FFFAF0"/>
      <ellipse cx="62" cy="46" rx="6" ry="5.5" fill="#FFFAF0"/>
      <ellipse cx="38" cy="46" rx="3.5" ry="4" fill="#8B6508"/>
      <ellipse cx="62" cy="46" rx="3.5" ry="4" fill="#8B6508"/>
      <ellipse cx="38" cy="46" rx="2" ry="2.5" fill="#111"/>
      <ellipse cx="62" cy="46" rx="2" ry="2.5" fill="#111"/>
      <circle cx="36" cy="44" r="1.5" fill="white" opacity="0.9"/>
      <circle cx="60" cy="44" r="1.5" fill="white" opacity="0.9"/>
      <!-- 黑鼻尖 -->
      <ellipse cx="50" cy="56" rx="4" ry="3" fill="#1A1A2E"/>
      <!-- 微笑 -->
      <path d="M44,60 Q50,64 56,60" fill="none" stroke="#1A1A2E" stroke-width="1.2" stroke-linecap="round"/>
      <!-- 搜索图标 -->
      <circle cx="78" cy="78" r="6" fill="none" stroke="#E8650A" stroke-width="2"/>
      <line x1="82" y1="82" x2="88" y2="88" stroke="#E8650A" stroke-width="2.5" stroke-linecap="round"/>
    </svg>

    <!-- ═══ 大熊猫 Mascot — 黑白、可爱 ═══ -->
    <svg v-else-if="role === 'mascot'" viewBox="0 0 100 100" class="cat-svg">
      <!-- 白脸 -->
      <circle cx="50" cy="55" r="32" fill="white"/>
      <!-- 黑耳朵 -->
      <circle cx="26" cy="28" r="12" fill="#1A1A2E"/>
      <circle cx="74" cy="28" r="12" fill="#1A1A2E"/>
      <!-- 黑眼圈（熊猫特征！） -->
      <ellipse cx="36" cy="48" rx="10" ry="8" fill="#1A1A2E"/>
      <ellipse cx="64" cy="48" rx="10" ry="8" fill="#1A1A2E"/>
      <!-- 白眼睛 -->
      <ellipse cx="36" cy="48" rx="5" ry="5" fill="white"/>
      <ellipse cx="64" cy="48" rx="5" ry="5" fill="white"/>
      <!-- 黑瞳 -->
      <circle cx="36" cy="48" r="3" fill="#111"/>
      <circle cx="64" cy="48" r="3" fill="#111"/>
      <circle cx="35" cy="46" r="1.2" fill="white"/>
      <circle cx="63" cy="46" r="1.2" fill="white"/>
      <!-- 黑鼻子 -->
      <ellipse cx="50" cy="58" rx="5" ry="3.5" fill="#1A1A2E"/>
      <!-- 微笑 -->
      <path d="M44,62 Q50,67 56,62" fill="none" stroke="#1A1A2E" stroke-width="1.5" stroke-linecap="round"/>
      <!-- 腮红 -->
      <ellipse cx="34" cy="60" rx="5" ry="3" fill="#FFB6C1" opacity="0.5"/>
      <ellipse cx="66" cy="60" rx="5" ry="3" fill="#FFB6C1" opacity="0.5"/>
      <!-- 竹子 -->
      <line x1="80" y1="60" x2="80" y2="95" stroke="#228B22" stroke-width="3"/>
      <line x1="80" y1="70" x2="88" y2="64" stroke="#228B22" stroke-width="2"/>
      <ellipse cx="89" cy="62" rx="4" ry="6" fill="#32CD32" transform="rotate(-20 89 62)"/>
    </svg>

    <!-- 默认 fallback -->
    <svg v-else viewBox="0 0 100 100" class="cat-svg">
      <circle cx="50" cy="55" r="30" fill="#ccc"/>
      <text x="50" y="60" text-anchor="middle" font-size="30">🐱</text>
    </svg>

    <!-- 状态指示器 -->
    <div class="status-dot" :style="{ background: statusColor }"></div>
  </div>
</template>

<style scoped>
.cat-svg-avatar {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.1);
  overflow: visible;
}

.cat-svg {
  width: 100%;
  height: 100%;
}

/* 状态指示器 */
.status-dot {
  position: absolute;
  bottom: 2px;
  right: 2px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 2px solid white;
  transition: background 0.3s;
}

/* active 呼吸动画 */
.cat-svg-avatar.active.animated .cat-svg {
  animation: breathe 2s ease-in-out infinite;
}

@keyframes breathe {
  0%, 100% { transform: scale(1); }
  50%      { transform: scale(1.06); }
}

/* busy 闪烁 */
.cat-svg-avatar.busy.animated .status-dot {
  animation: busyBlink 0.8s ease-in-out infinite;
}

@keyframes busyBlink {
  0%, 100% { opacity: 1; }
  50%      { opacity: 0.3; }
}

/* error 摇晃 */
.cat-svg-avatar.error.animated .cat-svg {
  animation: errorShake 0.5s ease-in-out;
}

@keyframes errorShake {
  0%, 100% { transform: translateX(0); }
  25%      { transform: translateX(-4px); }
  75%      { transform: translateX(4px); }
}
</style>
