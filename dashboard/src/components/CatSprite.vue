<template>
  <div class="cat-sprite" :class="[state, `breed-${breed}`, { glow }]" :title="breed + ' · ' + state">
    <svg viewBox="0 0 80 80" xmlns="http://www.w3.org/2000/svg" class="sprite-svg">
      <!-- shadow -->
      <ellipse cx="40" cy="72" rx="18" ry="3" class="shadow" />

      <!-- tail: wiggles in idle/working -->
      <path class="tail" :d="tailPath" />

      <!-- body -->
      <path class="body" :d="bodyPath" />

      <!-- breed pattern (tabby stripes, tuxedo bib, etc.) -->
      <g class="pattern" v-if="pattern === 'tabby'">
        <path d="M 28 48 Q 34 46 40 48" />
        <path d="M 28 54 Q 34 52 40 54" />
        <path d="M 44 48 Q 50 46 56 48" />
      </g>
      <g class="pattern" v-else-if="pattern === 'points'">
        <!-- siamese dark mask/paws -->
        <path d="M 30 32 Q 34 30 38 32 Q 34 34 30 32 Z" fill="#3a2a22" />
        <ellipse cx="24" cy="64" rx="5" ry="3" fill="#3a2a22" />
        <ellipse cx="56" cy="64" rx="5" ry="3" fill="#3a2a22" />
      </g>
      <g class="pattern" v-else-if="pattern === 'tuxedo'">
        <path d="M 34 42 Q 40 56 46 42 L 44 60 L 36 60 Z" fill="#fdf6e0" />
      </g>

      <!-- head -->
      <g class="head">
        <!-- ears -->
        <path class="ear" d="M 24 28 L 20 14 L 32 22 Z" />
        <path class="ear" d="M 56 28 L 60 14 L 48 22 Z" />
        <!-- inner ears -->
        <path class="ear-inner" d="M 25 25 L 24 18 L 30 22 Z" />
        <path class="ear-inner" d="M 55 25 L 56 18 L 50 22 Z" />
        <!-- face -->
        <path class="face" d="M 22 28 Q 40 18 58 28 Q 60 42 40 44 Q 20 42 22 28 Z" />
        <!-- eyes -->
        <g class="eyes">
          <ellipse class="eye" cx="32" cy="32" rx="2.2" ry="3" />
          <ellipse class="eye" cx="48" cy="32" rx="2.2" ry="3" />
          <circle class="pupil" cx="32" cy="32" r="0.8" />
          <circle class="pupil" cx="48" cy="32" r="0.8" />
        </g>
        <!-- nose + mouth -->
        <path class="nose" d="M 38 36 Q 40 38 42 36 Z" />
        <path class="mouth" d="M 40 38 Q 38 41 36 40 M 40 38 Q 42 41 44 40" />
        <!-- whiskers -->
        <g class="whiskers">
          <path d="M 20 38 L 30 38" />
          <path d="M 20 40 L 30 39" />
          <path d="M 50 38 L 60 38" />
          <path d="M 50 39 L 60 40" />
        </g>
      </g>

      <!-- state-specific overlays -->
      <g class="zzz" v-if="state === 'sleeping'">
        <text x="58" y="18" class="zzz-text z1">z</text>
        <text x="62" y="12" class="zzz-text z2 small">z</text>
        <text x="66" y="6"  class="zzz-text z3 small">z</text>
      </g>
      <g class="keyboard" v-if="state === 'working'">
        <!-- keyboard base -->
        <rect x="20" y="60" width="40" height="7" rx="1.5" class="kbd" />
        <!-- 10 keys, each with its own flash delay (computed from index) -->
        <rect v-for="k in 10" :key="k"
              :x="21 + (k - 1) * 3.8" y="61.5" width="3" height="4" rx="0.4"
              class="kbd-key" :style="{ animationDelay: `${((k * 73) % 10) * 0.09}s` }" />
      </g>
      <g class="bowl" v-if="state === 'eating'">
        <path d="M 28 62 Q 40 72 52 62 L 50 66 Q 40 70 30 66 Z" class="bowl-shape" />
        <path d="M 36 60 L 40 58 L 44 60" class="fish-tail" />
        <!-- crumbs -->
        <circle cx="30" cy="54" r="0.8" class="crumb c1" />
        <circle cx="48" cy="52" r="0.8" class="crumb c2" />
        <circle cx="40" cy="50" r="0.6" class="crumb c3" />
      </g>
      <g class="ball" v-if="state === 'playing'">
        <circle cx="62" cy="64" r="4" class="yarn" />
        <path d="M 60 62 Q 64 64 62 66" class="yarn-line" />
        <!-- trailing yarn string -->
        <path d="M 62 64 Q 54 58 48 62" class="yarn-trail" />
      </g>
    </svg>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import type { Breed, SpriteState } from '../api/catBreed'

const props = withDefaults(defineProps<{
  breed?: Breed
  state?: SpriteState
  size?: number | string
  glow?: boolean
}>(), {
  breed: 'generic',
  state: 'idle',
  size: 80,
  glow: false,
})

const cssSize = computed(() => typeof props.size === 'number' ? `${props.size}px` : props.size)

// breed → fill color + pattern hint
const BREED_META: Record<Breed, { fill: string; pattern: 'none' | 'tabby' | 'points' | 'tuxedo' }> = {
  siamese:      { fill: '#e8d7b8', pattern: 'points' },
  british_blue: { fill: '#8c9aa8', pattern: 'none' },
  orange_tabby: { fill: '#e89a4e', pattern: 'tabby' },
  maine_coon:   { fill: '#8a6a4e', pattern: 'tabby' },
  black:        { fill: '#2a2a2a', pattern: 'tuxedo' },
  abyssinian:   { fill: '#b87a4a', pattern: 'tabby' },
  dragon_li:    { fill: '#a89068', pattern: 'tabby' },
  owl:          { fill: '#9a8058', pattern: 'none' },
  fox:          { fill: '#d87040', pattern: 'none' },
  panda:        { fill: '#fafafa', pattern: 'tuxedo' },
  generic:      { fill: '#d8c8a8', pattern: 'none' },
}

const pattern = computed(() => BREED_META[props.breed].pattern)
const fillColor = computed(() => BREED_META[props.breed].fill)

// body path changes with state (curled up when sleeping, compact sit otherwise)
const bodyPath = computed(() => {
  if (props.state === 'sleeping') {
    return 'M 18 58 Q 14 70 30 72 Q 50 74 62 68 Q 68 58 58 54 Q 40 48 22 52 Q 16 54 18 58 Z'
  }
  return 'M 26 44 Q 22 62 32 68 Q 40 70 48 68 Q 58 62 54 44 Q 40 40 26 44 Z'
})

const tailPath = computed(() => {
  if (props.state === 'sleeping') return 'M 62 68 Q 72 66 70 72 Q 66 76 60 72'
  if (props.state === 'playing') return 'M 54 56 Q 68 48 72 58 Q 70 66 64 60'
  return 'M 54 56 Q 66 50 68 62 Q 70 70 62 64'
})
</script>

<style scoped>
.cat-sprite {
  display: inline-block;
  width: v-bind(cssSize);
  height: v-bind(cssSize);
  position: relative;
}
.sprite-svg {
  width: 100%;
  height: 100%;
  overflow: visible;
}

/* hand-drawn marker look */
.sprite-svg :is(.body, .face, .tail, .ear) {
  stroke: #2a2a2a;
  stroke-width: 1.6;
  stroke-linejoin: round;
  stroke-linecap: round;
  fill: v-bind(fillColor);
}
.ear-inner { fill: #e89ab8; stroke: none; opacity: 0.7; }
.eye       { fill: #2a2a2a; stroke: none; }
.pupil     { fill: #fff; stroke: none; }
/* night glow: amber cat-eyes with slitted pupils + drop-shadow */
.cat-sprite.glow .eye {
  fill: #ffb040;
  filter: drop-shadow(0 0 2.5px rgba(255, 200, 80, 0.9));
}
.cat-sprite.glow .pupil {
  fill: #2a1a10;
  transform: scaleX(0.35);
  transform-origin: center;
  transform-box: fill-box;
}
.nose      { fill: #e89ab8; stroke: #2a2a2a; stroke-width: 1; }
.mouth, .whiskers path {
  stroke: #2a2a2a; stroke-width: 1; fill: none; stroke-linecap: round;
}
.whiskers path { opacity: 0.55; }
.pattern path, .pattern ellipse { stroke: #2a2a2a; stroke-width: 1; fill: none; opacity: 0.5; }
.shadow { fill: #2a2a2a; opacity: 0.12; }

/* state overlays */
.zzz-text { font-family: 'Patrick Hand', 'Comic Sans MS', cursive; font-size: 12px; fill: #2a2a2a; }
.zzz-text.small { font-size: 9px; }
.kbd { fill: #cfc6b0; stroke: #2a2a2a; stroke-width: 1.2; }
.kbd-key {
  fill: #f5efdf; stroke: #2a2a2a; stroke-width: 0.5;
  animation: key-press 0.9s ease-in-out infinite;
}
.bowl-shape { fill: #c6a07a; stroke: #2a2a2a; stroke-width: 1.2; }
.fish-tail  { stroke: #2a2a2a; stroke-width: 1; fill: #eac78a; }
.crumb      { fill: #c9a06a; stroke: #2a2a2a; stroke-width: 0.3; animation: crumb-fall 1.4s ease-out infinite; }
.crumb.c2   { animation-delay: 0.3s; }
.crumb.c3   { animation-delay: 0.7s; }
.yarn       { fill: #c97a98; stroke: #2a2a2a; stroke-width: 1.2; }
.yarn-line  { stroke: #2a2a2a; stroke-width: 0.8; fill: none; }
.yarn-trail { stroke: #c97a98; stroke-width: 0.7; fill: none; stroke-dasharray: 1.5 1.5; }

/* animations */
.cat-sprite.working .head  { animation: nod 0.6s ease-in-out infinite; transform-origin: 40px 40px; transform-box: fill-box; }
.cat-sprite.working .body  { animation: type-bob 0.6s ease-in-out infinite; transform-origin: 40px 60px; transform-box: fill-box; }
.cat-sprite.idle .tail     { animation: tail-flick 2.4s ease-in-out infinite; transform-origin: 54px 56px; transform-box: fill-box; }
.cat-sprite.sleeping .body { animation: breathe 3s ease-in-out infinite; transform-origin: 40px 62px; transform-box: fill-box; }
.cat-sprite.sleeping .eye  { transform: scaleY(0.15); transform-origin: center; }
.cat-sprite.sleeping .z1   { animation: zzz-float 2.8s ease-in-out infinite; }
.cat-sprite.sleeping .z2   { animation: zzz-float 2.8s ease-in-out 0.5s infinite; }
.cat-sprite.sleeping .z3   { animation: zzz-float 2.8s ease-in-out 1s infinite; }
.cat-sprite.sleeping .zzz-text { opacity: 0; }
.cat-sprite.eating .head   { animation: munch 0.8s ease-in-out infinite; transform-origin: 40px 40px; transform-box: fill-box; }
.cat-sprite.playing .tail  { animation: tail-flick 0.5s ease-in-out infinite; transform-origin: 54px 56px; transform-box: fill-box; }
.cat-sprite.playing .head  { animation: pounce 0.8s ease-in-out infinite; transform-origin: 40px 40px; transform-box: fill-box; }

@keyframes nod        { 0%,100%{ transform: translateY(0) } 50%{ transform: translateY(2px) } }
@keyframes type-bob   { 0%,100%{ transform: translateY(0) } 50%{ transform: translateY(1px) } }
@keyframes tail-flick { 0%,100%{ transform: rotate(0) } 50%{ transform: rotate(-12deg) } }
@keyframes breathe    { 0%,100%{ transform: scaleY(1) } 50%{ transform: scaleY(1.04) } }
@keyframes zzz-float  { 0%{ opacity: 0; transform: translate(0, 0) } 30%{ opacity: 1 } 100%{ opacity: 0; transform: translate(4px, -8px) } }
@keyframes munch      { 0%,100%{ transform: translateY(0) } 50%{ transform: translateY(3px) rotate(-3deg) } }
@keyframes pounce     { 0%,100%{ transform: translateY(0) } 50%{ transform: translateY(-3px) rotate(5deg) } }
@keyframes key-press  { 0%,80%,100%{ fill: #f5efdf; transform: translateY(0) } 85%{ fill: #ffd866; transform: translateY(0.5px) } }
@keyframes crumb-fall { 0%{ opacity: 1; transform: translateY(0) } 100%{ opacity: 0; transform: translateY(6px) } }
</style>
