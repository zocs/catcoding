<template>
  <div class="office-page" :class="`theme-${theme}`">
    <div class="office-header">
      <h2>{{ theme === 'night' ? '🌙' : '🏢' }} {{ t('nav.office') }}</h2>
      <span class="subtitle">{{ theme === 'night' ? 'pastoral night · catcoding camp' : 'notebook sketch · catcoding team HQ' }}</span>
      <span class="live-badge" :class="{ 'is-live': live, 'is-dead': !live, 'is-ws': wsConnected }">
        <span class="dot" />
        {{ wsConnected ? 'LIVE · ws' : (live ? 'LIVE · poll' : (lastError ? 'OFFLINE' : '...')) }}
      </span>
      <button class="theme-btn" @click="toggleTheme" :title="theme === 'day' ? 'switch to night' : 'switch to day'">
        {{ theme === 'day' ? '☀️ day' : '🌙 night' }}
      </button>
    </div>

    <div class="stage" :class="`stage-${theme}`">
      <svg class="paper-grid" width="100%" height="100%" preserveAspectRatio="none">
        <defs>
          <pattern id="paperGrid" width="24" height="24" patternUnits="userSpaceOnUse">
            <path d="M 24 0 L 0 0 0 24" fill="none" :stroke="theme === 'night' ? '#5a6a8a' : '#c7b98a'" stroke-width="0.4" />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#paperGrid)" />
      </svg>

      <!-- night-only: moon + stars -->
      <svg v-if="theme === 'night'" class="sky" viewBox="0 0 600 120" preserveAspectRatio="none" xmlns="http://www.w3.org/2000/svg">
        <!-- moon -->
        <circle cx="540" cy="50" r="26" fill="#f5efdf" stroke="#2a2a2a" stroke-width="1.4" />
        <path d="M 552 38 Q 540 50 552 62 Q 538 60 538 50 Q 538 40 552 38 Z" fill="#d8d0b8" stroke="#2a2a2a" stroke-width="0.8" opacity="0.7" />
        <!-- stars -->
        <g stroke="#fbf5e3" stroke-width="0.8" fill="#fbf5e3">
          <path d="M 80 30 L 82 34 L 86 36 L 82 38 L 80 42 L 78 38 L 74 36 L 78 34 Z" class="star s1" />
          <path d="M 200 60 L 201 62 L 204 63 L 201 64 L 200 66 L 199 64 L 196 63 L 199 62 Z" class="star s2" />
          <path d="M 320 40 L 322 44 L 326 46 L 322 48 L 320 52 L 318 48 L 314 46 L 318 44 Z" class="star s3" />
          <path d="M 440 70 L 441 72 L 444 73 L 441 74 L 440 76 L 439 74 L 436 73 L 439 72 Z" class="star s4" />
          <circle cx="150" cy="90" r="1" class="star s5" />
          <circle cx="380" cy="20" r="1" class="star s6" />
        </g>
      </svg>

      <div class="iso-world">
        <!-- hand-drawn tiles (SVG per tile, wobbly stroke + seeded texture) -->
        <svg
          v-for="tile in tiles"
          :key="`t-${tile.x}-${tile.y}`"
          class="tile"
          :style="tileStyle(tile.x, tile.y)"
          viewBox="0 0 96 48"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            :d="wobblyDiamond(tile.seed)"
            :fill="tile.carpet ? palette.carpet : palette.grass"
            :stroke="palette.ink"
            stroke-width="1.2"
            stroke-linejoin="round"
          />
          <!-- carpet: wood-grain hatching -->
          <g v-if="tile.carpet" :stroke="palette.carpetStroke" stroke-width="0.5" opacity="0.5" fill="none">
            <path :d="`M ${20 + tile.seed % 6} 18 Q 48 ${14 + tile.seed % 3} ${70 + tile.seed % 5} 22`" />
            <path :d="`M ${24 + tile.seed % 4} 28 Q 48 ${30 - tile.seed % 3} ${72 + tile.seed % 4} 24`" />
            <line x1="36" y1="14" x2="38" y2="20" opacity="0.6" />
            <line x1="58" y1="26" x2="60" y2="32" opacity="0.6" />
          </g>
          <!-- grass: tiny tufts (seeded count 2-3) + night fireflies -->
          <g v-else :stroke="palette.grassStroke" stroke-width="0.7" fill="none" stroke-linecap="round">
            <path :d="`M ${30 + tile.seed % 8} 22 L ${30 + tile.seed % 8} 18 M ${32 + tile.seed % 8} 22 L ${33 + tile.seed % 8} 17 M ${34 + tile.seed % 8} 22 L ${36 + tile.seed % 8} 18`" />
            <path v-if="tile.seed % 3 === 0" :d="`M ${54 + tile.seed % 6} 30 L ${54 + tile.seed % 6} 26 M ${56 + tile.seed % 6} 30 L ${57 + tile.seed % 6} 25`" />
            <circle v-if="tile.seed % 5 === 0 && theme === 'day'" :cx="62 + tile.seed % 4" :cy="20 + tile.seed % 3" r="0.8" fill="#d88fb7" stroke="none" />
            <circle v-if="tile.seed % 7 === 0 && theme === 'night'" :cx="48 + tile.seed % 10" :cy="22 + tile.seed % 5" r="1.2" fill="#ffe488" stroke="none" class="firefly" :style="{ animationDelay: `${(tile.seed % 10) * 0.2}s` }" />
          </g>
        </svg>

        <!-- scene props on empty tiles -->
        <svg v-for="(p, i) in sceneProps" :key="`p-${i}`"
             class="prop" :style="tileStyle(p.x, p.y, p.z || 0)"
             :viewBox="p.vb" xmlns="http://www.w3.org/2000/svg">
          <!-- potted plant -->
          <g v-if="p.kind === 'plant'">
            <path d="M 20 30 L 22 50 L 34 50 L 36 30 Z" fill="#c97a58" stroke="#2a2a2a" stroke-width="1.2" />
            <ellipse cx="28" cy="30" rx="10" ry="2" fill="#8a4a2a" stroke="#2a2a2a" stroke-width="1" />
            <path d="M 28 30 Q 18 18 22 8 Q 28 14 28 28" fill="#6aa34a" stroke="#2a2a2a" stroke-width="1.2" />
            <path d="M 28 30 Q 38 16 36 6 Q 28 14 28 28" fill="#7ab35a" stroke="#2a2a2a" stroke-width="1.2" />
            <path d="M 28 30 Q 28 12 32 4 Q 30 18 28 28" fill="#5a9a3a" stroke="#2a2a2a" stroke-width="1.2" />
          </g>
          <!-- water cooler -->
          <g v-else-if="p.kind === 'cooler'">
            <path d="M 16 20 L 40 20 L 38 26 L 18 26 Z" fill="#a0c8d8" stroke="#2a2a2a" stroke-width="1.2" />
            <rect x="18" y="26" width="20" height="20" fill="#e8e4d0" stroke="#2a2a2a" stroke-width="1.2" />
            <ellipse cx="28" cy="20" rx="12" ry="3" fill="#8ab8d0" stroke="#2a2a2a" stroke-width="1.2" />
            <rect x="25" y="34" width="6" height="3" fill="#c95a3a" stroke="#2a2a2a" stroke-width="0.8" />
            <rect x="25" y="40" width="6" height="3" fill="#3a7ac9" stroke="#2a2a2a" stroke-width="0.8" />
          </g>
          <!-- trash bin -->
          <g v-else-if="p.kind === 'trash'">
            <path d="M 14 22 L 38 22 L 34 46 L 18 46 Z" fill="#6a7a8a" stroke="#2a2a2a" stroke-width="1.2" />
            <path d="M 20 26 L 21 42 M 26 26 L 26 42 M 32 26 L 31 42" stroke="#2a2a2a" stroke-width="0.6" fill="none" />
            <ellipse cx="26" cy="22" rx="13" ry="2" fill="#8a9aa8" stroke="#2a2a2a" stroke-width="1.2" />
            <path d="M 18 20 Q 26 14 34 20" stroke="#2a2a2a" stroke-width="1.2" fill="none" />
            <!-- crumpled paper sticking out -->
            <path d="M 20 18 Q 22 12 28 14 Q 32 16 28 18 Z" fill="#fbf5e3" stroke="#2a2a2a" stroke-width="0.8" />
          </g>
          <!-- whiteboard -->
          <g v-else-if="p.kind === 'board'">
            <rect x="4" y="4" width="56" height="34" fill="#fdfcf4" stroke="#2a2a2a" stroke-width="1.4" />
            <rect x="4" y="38" width="56" height="3" fill="#c9b090" stroke="#2a2a2a" stroke-width="1" />
            <text x="9" y="12" font-family="Patrick Hand, cursive" font-size="6" fill="#3a7ac9">TODO</text>
            <!-- dynamic task lines -->
            <g v-if="boardTasks.length">
              <g v-for="(task, ti) in boardTasks" :key="task.id" :transform="`translate(0, ${16 + ti * 5})`">
                <!-- checkbox: filled for active/reviewing, empty for ready/pending -->
                <rect x="9" y="-3.5" width="3" height="3" fill="none" stroke="#2a2a2a" stroke-width="0.6" />
                <path v-if="task.status === 'active' || task.status === 'reviewing'"
                      d="M 9.3 -2 L 10.5 -0.8 L 11.7 -3.2" stroke="#3aa35a" stroke-width="0.9" fill="none" />
                <text x="14" y="-1" font-family="Patrick Hand, cursive" font-size="3.4"
                      :fill="taskLineColor(task.status)">{{ truncateTask(task.title, 22) }}</text>
              </g>
            </g>
            <!-- fallback when no tasks -->
            <g v-else>
              <path d="M 9 18 L 54 18" stroke="#c95a3a" stroke-width="0.6" />
              <path d="M 9 22 L 40 22" stroke="#2a2a2a" stroke-width="0.6" />
              <path d="M 9 26 L 48 26" stroke="#2a2a2a" stroke-width="0.6" />
              <path d="M 9 30 L 34 30" stroke="#6a7a8a" stroke-width="0.6" stroke-dasharray="2 1" />
            </g>
            <!-- paw doodle -->
            <g transform="translate(50, 28)" stroke="#2a2a2a" stroke-width="0.6" fill="#f5a0b8">
              <circle cx="0" cy="2" r="2" />
              <circle cx="-3" cy="-1" r="1" />
              <circle cx="0" cy="-3" r="1" />
              <circle cx="3" cy="-1" r="1" />
            </g>
          </g>
          <!-- sticky note pile -->
          <g v-else-if="p.kind === 'sticky'">
            <rect x="6" y="10" width="20" height="20" fill="#ffe08a" stroke="#2a2a2a" stroke-width="1" transform="rotate(-4 16 20)" />
            <rect x="9" y="8"  width="20" height="20" fill="#a8d8a0" stroke="#2a2a2a" stroke-width="1" transform="rotate(3 19 18)" />
            <rect x="7" y="6"  width="20" height="20" fill="#f0a8b8" stroke="#2a2a2a" stroke-width="1" transform="rotate(-1 17 16)" />
            <path d="M 11 14 L 22 14 M 11 17 L 20 17 M 11 20 L 23 20" stroke="#2a2a2a" stroke-width="0.5" opacity="0.6" />
          </g>
        </svg>

        <!-- hand-drawn desks — theme-aware: office desk (day) vs camping campfire (night) -->
        <svg
          v-for="(desk, i) in desks"
          :key="`d-${i}`"
          class="desk"
          :class="{ 'desk--active': deskActive(i), 'desk--night': theme === 'night' }"
          :style="tileStyle(desk.x, desk.y, 10)"
          viewBox="0 0 80 64"
          xmlns="http://www.w3.org/2000/svg"
        >
          <template v-if="theme === 'day'">
            <!-- office desk top (isometric diamond) -->
            <path
              d="M 40 26 L 74 42 L 40 58 L 6 42 Z"
              fill="#e8c892" stroke="#2a2a2a" stroke-width="1.4" stroke-linejoin="round"
            />
            <path d="M 40 58 L 6 42 L 6 46 L 40 62 Z" fill="#c99f5a" stroke="#2a2a2a" stroke-width="1" />
            <path d="M 40 58 L 74 42 L 74 46 L 40 62 Z" fill="#b88a46" stroke="#2a2a2a" stroke-width="1" />
            <!-- monitor -->
            <rect x="34" y="8" width="20" height="16" rx="2" fill="#f5efdf" stroke="#2a2a2a" stroke-width="1.4" />
            <g v-if="deskActive(i)" class="monitor-code">
              <path d="M 37 12 L 47 12" stroke="#3aa35a" stroke-width="0.9" />
              <path d="M 37 15 L 51 15" stroke="#6a7a8a" stroke-width="0.9" />
              <path d="M 37 18 L 45 18" stroke="#c77a5a" stroke-width="0.9" />
              <path d="M 37 21 L 49 21" stroke="#6a7a8a" stroke-width="0.9" />
            </g>
            <g v-else>
              <path d="M 37 12 L 51 12 M 37 16 L 49 16 M 37 20 L 47 20" stroke="#c0b090" stroke-width="0.8" />
            </g>
            <rect x="42" y="24" width="4" height="3" fill="#2a2a2a" />
            <!-- paper sheet -->
            <path d="M 14 40 L 24 38 L 26 46 L 16 48 Z" fill="#fbf5e3" stroke="#2a2a2a" stroke-width="0.8" />
            <line x1="16" y1="42" x2="23" y2="41" stroke="#2a2a2a" stroke-width="0.3" opacity="0.5" />
            <line x1="16" y1="44" x2="23" y2="43" stroke="#2a2a2a" stroke-width="0.3" opacity="0.5" />
            <!-- mug + steam -->
            <ellipse cx="58" cy="34" rx="4" ry="2" fill="#d97a5a" stroke="#2a2a2a" stroke-width="1" />
            <path d="M 58 32 Q 60 30 60 34" stroke="#8a4a2a" stroke-width="0.6" fill="none" />
            <g v-if="deskActive(i)" class="steam">
              <path d="M 56 30 Q 58 26 56 22" stroke="#b0b0b0" stroke-width="0.6" fill="none" opacity="0.6" />
              <path d="M 60 30 Q 58 26 60 22" stroke="#b0b0b0" stroke-width="0.6" fill="none" opacity="0.6" />
            </g>
          </template>
          <template v-else>
            <!-- night camping workstation: tree-stump seat + laptop-on-rock + campfire -->
            <!-- ground halo: soft firelight glow (active only) -->
            <ellipse v-if="deskActive(i)" cx="40" cy="52" rx="30" ry="10" fill="#f5a860" opacity="0.18" class="fire-halo" />
            <!-- tree stump seat (center-back) -->
            <ellipse cx="40" cy="42" rx="14" ry="5" fill="#6a4a2a" stroke="#e8e2d0" stroke-width="1.2" />
            <path d="M 26 42 L 26 50 Q 40 54 54 50 L 54 42" fill="#4a3018" stroke="#e8e2d0" stroke-width="1.2" />
            <ellipse cx="40" cy="42" rx="10" ry="3" fill="#8a6a4a" stroke="#e8e2d0" stroke-width="0.8" />
            <!-- tree rings -->
            <ellipse cx="40" cy="42" rx="6" ry="1.8" fill="none" stroke="#4a3018" stroke-width="0.6" />
            <ellipse cx="40" cy="42" rx="3" ry="0.9" fill="none" stroke="#4a3018" stroke-width="0.6" />
            <!-- laptop on stump -->
            <g transform="translate(30, 30)">
              <path d="M 0 12 L 20 12 L 22 16 L -2 16 Z" fill="#3a3a4a" stroke="#e8e2d0" stroke-width="1.2" />
              <path d="M 2 0 L 18 0 L 20 12 L 0 12 Z" fill="#2a2a3a" stroke="#e8e2d0" stroke-width="1.2" />
              <!-- screen -->
              <g v-if="deskActive(i)" class="laptop-code">
                <path d="M 5 4 L 13 4" stroke="#5ac97a" stroke-width="0.7" />
                <path d="M 5 6 L 16 6" stroke="#7a9ac9" stroke-width="0.7" />
                <path d="M 5 8 L 12 8" stroke="#c97a5a" stroke-width="0.7" />
                <path d="M 5 10 L 15 10" stroke="#7a9ac9" stroke-width="0.7" />
              </g>
              <g v-else>
                <path d="M 5 4 L 15 4 M 5 6 L 14 6 M 5 8 L 16 8 M 5 10 L 12 10" stroke="#6a6a7a" stroke-width="0.5" />
              </g>
            </g>
            <!-- campfire (front-right of desk) -->
            <g transform="translate(56, 48)">
              <!-- stone ring -->
              <ellipse cx="-6" cy="4" rx="2.5" ry="1.5" fill="#8a8a9a" stroke="#e8e2d0" stroke-width="0.8" />
              <ellipse cx="-2" cy="5" rx="2.5" ry="1.5" fill="#6a6a7a" stroke="#e8e2d0" stroke-width="0.8" />
              <ellipse cx="3"  cy="5" rx="2.5" ry="1.5" fill="#7a7a8a" stroke="#e8e2d0" stroke-width="0.8" />
              <ellipse cx="7"  cy="4" rx="2.5" ry="1.5" fill="#8a8a9a" stroke="#e8e2d0" stroke-width="0.8" />
              <!-- logs criss-cross -->
              <path d="M -5 2 L 6 4 L 5 6 L -6 4 Z" fill="#5a3a18" stroke="#e8e2d0" stroke-width="0.6" />
              <path d="M -4 1 L 5 -1 L 6 1 L -3 3 Z" fill="#6a4a28" stroke="#e8e2d0" stroke-width="0.6" transform="rotate(20)" />
              <!-- flames (active: flicker) -->
              <g v-if="deskActive(i)" class="flame">
                <path d="M 0 0 Q -4 -4 -2 -8 Q 0 -12 2 -8 Q 4 -4 0 0 Z" fill="#ffb040" stroke="#2a2a2a" stroke-width="0.8" class="flame-outer" />
                <path d="M 0 -2 Q -2 -4 -1 -6 Q 0 -9 1 -6 Q 2 -4 0 -2 Z" fill="#ffe070" stroke="#c77a1a" stroke-width="0.4" class="flame-inner" />
                <!-- embers floating up -->
                <circle cx="-3" cy="-10" r="0.5" fill="#ff7030" class="ember e1" />
                <circle cx="2"  cy="-12" r="0.5" fill="#ffa048" class="ember e2" />
                <circle cx="4"  cy="-8"  r="0.4" fill="#ff5020" class="ember e3" />
              </g>
              <g v-else>
                <!-- cold fire: just wisps of smoke -->
                <path d="M 0 -2 Q -2 -6 0 -10" stroke="#6a6a7a" stroke-width="0.6" fill="none" opacity="0.5" />
              </g>
            </g>
            <!-- marshmallow on stick (decorative, only when active) -->
            <g v-if="deskActive(i)" transform="translate(12, 38)">
              <line x1="0" y1="0" x2="16" y2="-6" stroke="#6a4a2a" stroke-width="0.8" />
              <ellipse cx="0" cy="0" rx="2" ry="1.5" fill="#f5efdf" stroke="#e8e2d0" stroke-width="0.6" />
            </g>
          </template>
        </svg>

        <!-- lounge: rug + bowl + props -->
        <svg class="lounge" :style="tileStyle(5.5, 5.5, 4)" viewBox="0 0 200 100" xmlns="http://www.w3.org/2000/svg">
          <path
            d="M 100 10 Q 180 40 190 50 Q 180 70 100 90 Q 20 70 10 50 Q 20 30 100 10 Z"
            fill="#ecb6c8" stroke="#2a2a2a" stroke-width="1.4" opacity="0.9"
          />
          <g stroke="#a64a7a" stroke-width="0.6" opacity="0.5" fill="none">
            <path d="M 40 40 Q 60 36 80 44" />
            <path d="M 90 60 Q 110 54 130 62" />
            <path d="M 140 44 Q 160 40 170 50" />
          </g>
          <!-- food bowl with fish -->
          <g transform="translate(140, 58)">
            <path d="M 0 0 Q 24 16 48 0 L 44 10 Q 24 18 4 10 Z" fill="#c6a07a" stroke="#2a2a2a" stroke-width="1.2" />
            <path d="M 16 -2 L 24 -6 L 32 -2" stroke="#2a2a2a" stroke-width="1" fill="#eac78a" />
          </g>
          <!-- tuna can -->
          <g transform="translate(28, 64)">
            <ellipse cx="10" cy="4" rx="10" ry="3" fill="#d0c8b0" stroke="#2a2a2a" stroke-width="1" />
            <rect x="0" y="0" width="20" height="4" fill="#d0c8b0" stroke="#2a2a2a" stroke-width="1" />
            <text x="4" y="3" font-family="Patrick Hand" font-size="3" fill="#2a2a2a">TUNA</text>
          </g>
          <!-- yarn ball pile -->
          <g transform="translate(80, 70)" class="yarn-pile">
            <circle cx="0" cy="0" r="5" fill="#c97a98" stroke="#2a2a2a" stroke-width="1" />
            <path d="M -3 -2 Q 0 0 3 -3 M -4 1 Q 0 3 4 0" stroke="#2a2a2a" stroke-width="0.5" fill="none" />
            <circle cx="10" cy="3" r="4" fill="#7a9ac9" stroke="#2a2a2a" stroke-width="1" />
          </g>
          <!-- scratching post -->
          <g transform="translate(170, 20)">
            <rect x="0" y="0" width="6" height="40" fill="#b88a56" stroke="#2a2a2a" stroke-width="1" />
            <path d="M 0 10 L 6 12 M 0 20 L 6 22 M 0 30 L 6 32" stroke="#6a4a2a" stroke-width="0.5" />
            <ellipse cx="3" cy="42" rx="8" ry="2" fill="#8a6a46" stroke="#2a2a2a" stroke-width="1" />
            <!-- dangling feather toy -->
            <line x1="3" y1="0" x2="-6" y2="-6" stroke="#2a2a2a" stroke-width="0.6" class="feather-string" />
            <path d="M -6 -6 Q -10 -10 -8 -14 Q -4 -10 -6 -6 Z" fill="#c9a06a" stroke="#2a2a2a" stroke-width="0.8" class="feather" />
          </g>
        </svg>

        <!-- pair sparkle between playing cat pairs -->
        <svg
          v-for="p in pairs"
          :key="`pair-${p.a}-${p.b}`"
          class="pair-spark"
          :style="tileStyle(p.cx, p.cy, 48)"
          viewBox="0 0 40 40"
          xmlns="http://www.w3.org/2000/svg"
        >
          <g class="spark-heart">
            <path d="M 20 14 Q 14 6 10 14 Q 10 22 20 30 Q 30 22 30 14 Q 26 6 20 14 Z"
                  fill="#e89ab8" stroke="#2a2a2a" stroke-width="1.4" stroke-linejoin="round" />
          </g>
          <g class="spark-stars" stroke="#2a2a2a" stroke-width="1" fill="#ffe488">
            <path d="M 6 10 L 7 12 L 9 13 L 7 14 L 6 16 L 5 14 L 3 13 L 5 12 Z" class="s-a" />
            <path d="M 34 10 L 35 12 L 37 13 L 35 14 L 34 16 L 33 14 L 31 13 L 33 12 Z" class="s-b" />
            <path d="M 30 28 L 31 30 L 33 31 L 31 32 L 30 34 L 29 32 L 27 31 L 29 30 Z" class="s-c" />
          </g>
        </svg>

        <!-- cats -->
        <div
          v-for="(slot, i) in catSlots"
          :key="`c-${i}`"
          class="cat-slot"
          :style="tileStyle(slot.x, slot.y, 24)"
          :title="slot.agent.name + ' · ' + slot.spriteState"
          @click="onCatClick(slot.agent)"
        >
          <div class="cat-anchor">
            <!-- task bubble when working -->
            <div
              v-if="slot.spriteState === 'working' && slot.agent.current_task"
              class="task-bubble"
              :title="slot.agent.current_task"
            >
              <span class="bubble-label">📋</span>
              <span class="bubble-text">{{ truncate(slot.agent.current_task, 22) }}</span>
            </div>
            <CatSprite :breed="slot.breed" :state="slot.spriteState" :glow="theme === 'night'" />
            <div class="name-tag">{{ slot.agent.name }}</div>
          </div>
        </div>
      </div>
    </div>

    <div class="legend">
      <span>🌿 floor</span>
      <span>🟫 carpet</span>
      <span>🖥 desk</span>
      <span>🐟 lounge</span>
      <small>tile {{ TILE_W }}×{{ TILE_H }} · grid {{ COLS }}×{{ ROWS }}</small>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import CatSprite from '../components/CatSprite.vue'
import { CatCodingApi, type Agent, type AgentStatus, type Task } from '../api/types'
import { ROLE_TO_BREED } from '../api/catBreed'

const { t } = useI18n()
const router = useRouter()
const api = new CatCodingApi()

const theme = ref<'day' | 'night'>(
  (localStorage.getItem('office-theme') as 'day' | 'night') || 'day'
)
function toggleTheme() {
  theme.value = theme.value === 'day' ? 'night' : 'day'
  localStorage.setItem('office-theme', theme.value)
}

const palette = computed(() => theme.value === 'night'
  ? {
      grass: '#3a6a3a', grassStroke: '#2a4a2a',
      carpet: '#8a6a4e', carpetStroke: '#4a3a1a',
      ink: '#e8e2d0',
    }
  : {
      grass: '#b9d89e', grassStroke: '#4d7a3a',
      carpet: '#e0b88a', carpetStroke: '#7a5a2a',
      ink: '#2a2a2a',
    })

// polling cadence: tighter when WS is down, loose when WS fills in the gaps
const POLL_MS_WITH_WS = 15000
const POLL_MS_WITHOUT_WS = 3000
let refreshTimer: ReturnType<typeof setInterval> | null = null
let ws: WebSocket | null = null
let wsReconnectTimer: ReturnType<typeof setTimeout> | null = null
let wsBackoff = 1000

const live = ref(false)
const wsConnected = ref(false)
const lastError = ref<string | null>(null)

const TILE_W = 96
const TILE_H = 48
const COLS = 8
const ROWS = 8

interface Tile { x: number; y: number; carpet?: boolean; seed: number }
const tiles: Tile[] = []
for (let y = 0; y < ROWS; y++) {
  for (let x = 0; x < COLS; x++) {
    tiles.push({
      x, y,
      carpet: x >= 4 && y >= 4,
      seed: (x * 31 + y * 17) % 100,
    })
  }
}

const desks = [
  { x: 1, y: 1 }, { x: 3, y: 1 }, { x: 5, y: 1 },
  { x: 1, y: 3 }, { x: 3, y: 3 }, { x: 5, y: 3 },
]

// decorative props on empty tiles
interface Prop { kind: 'plant' | 'cooler' | 'trash' | 'board' | 'sticky'; x: number; y: number; z?: number; vb: string }
const sceneProps: Prop[] = [
  { kind: 'plant',  x: 7, y: 0, z: 30, vb: '0 0 56 56' },
  { kind: 'board',  x: 0, y: 0, z: 40, vb: '0 0 64 44' },
  { kind: 'cooler', x: 7, y: 2, z: 36, vb: '0 0 56 56' },
  { kind: 'trash',  x: 0, y: 2, z: 30, vb: '0 0 52 56' },
  { kind: 'sticky', x: 7, y: 7, z: 20, vb: '0 0 36 36' },
  { kind: 'plant',  x: 0, y: 7, z: 30, vb: '0 0 56 56' },
]

function tileStyle(x: number, y: number, z = 0) {
  const sx = (x - y) * (TILE_W / 2)
  const sy = (x + y) * (TILE_H / 2) - z
  return { transform: `translate(${sx}px, ${sy}px)` }
}

// wobbly diamond path — seed jitters corners for hand-drawn feel
function wobblyDiamond(seed: number): string {
  const j = (s: number) => ((s * 9301 + 49297) % 233280) / 233280
  const w = (s: number) => (j(s) - 0.5) * 2
  return `M ${48 + w(seed)} ${0 + w(seed + 1)}
          L ${96 + w(seed + 2)} ${24 + w(seed + 3)}
          L ${48 + w(seed + 4)} ${48 + w(seed + 5)}
          L ${0 + w(seed + 6)}  ${24 + w(seed + 7)} Z`
}

const agents = ref<Agent[]>([])
const tasks = ref<Task[]>([])

// top 4 "actionable" tasks for the whiteboard (active > reviewing > ready > pending/blocked)
const boardTasks = computed(() => {
  const order: Record<string, number> = { active: 0, reviewing: 1, ready: 2, pending: 3, blocked: 4, done: 9, rollbacked: 9, failed: 9 }
  return [...tasks.value]
    .filter((t) => (order[t.status] ?? 9) < 9)
    .sort((a, b) => (order[a.status] ?? 9) - (order[b.status] ?? 9))
    .slice(0, 4)
})

async function refresh() {
  try {
    const [a, ts] = await Promise.all([api.getAgents(), api.getTasks()])
    agents.value = a
    tasks.value = ts
    live.value = true
    lastError.value = null
    tickRoam()
  } catch (e) {
    live.value = false
    lastError.value = e instanceof Error ? e.message : String(e)
  }
}

function applyStatusPatch(role: string, status: AgentStatus, taskId?: string) {
  const idx = agents.value.findIndex(a => a.role === role)
  if (idx < 0) return
  agents.value = agents.value.map((a, i) =>
    i === idx ? { ...a, status, current_task: taskId ?? a.current_task } : a
  )
}

function schedulePoll() {
  if (refreshTimer) clearInterval(refreshTimer)
  const interval = wsConnected.value ? POLL_MS_WITH_WS : POLL_MS_WITHOUT_WS
  refreshTimer = setInterval(refresh, interval)
}

function connectWs() {
  if (ws) return
  const proto = location.protocol === 'https:' ? 'wss' : 'ws'
  try {
    ws = new WebSocket(`${proto}://${location.host}/ws`)
  } catch (e) {
    scheduleReconnect()
    return
  }
  ws.onopen = () => {
    wsConnected.value = true
    wsBackoff = 1000
    schedulePoll()
  }
  ws.onmessage = (ev) => {
    try {
      const msg = JSON.parse(ev.data)
      if (msg.type === 'agent.status' && msg.role && msg.status) {
        applyStatusPatch(msg.role as string, msg.status as AgentStatus, msg.task_id)
        live.value = true
      } else if (msg.type === 'xp.update') {
        // XP changed — agent xp/level fields may be stale; do a quick refresh
        refresh()
      }
    } catch {
      // non-JSON frame — ignore
    }
  }
  ws.onclose = () => {
    wsConnected.value = false
    ws = null
    schedulePoll()
    scheduleReconnect()
  }
  ws.onerror = () => {
    // let onclose handle reconnect; avoid double-scheduling
  }
}

function scheduleReconnect() {
  if (wsReconnectTimer) return
  wsReconnectTimer = setTimeout(() => {
    wsReconnectTimer = null
    wsBackoff = Math.min(wsBackoff * 2, 30000)
    connectWs()
  }, wsBackoff)
}

onMounted(() => {
  refresh()
  schedulePoll()
  connectWs()
  roamTimer = setInterval(tickRoam, 7000)
})

onBeforeUnmount(() => {
  if (refreshTimer) clearInterval(refreshTimer)
  refreshTimer = null
  if (roamTimer) clearInterval(roamTimer)
  roamTimer = null
  if (wsReconnectTimer) clearTimeout(wsReconnectTimer)
  wsReconnectTimer = null
  if (ws) {
    ws.onclose = null
    ws.onerror = null
    ws.onmessage = null
    ws.close()
    ws = null
  }
})

// role → breed (shared with Agents via api/catBreed.ts)

const catSlots = computed(() => {
  return agents.value.map((agent, i) => {
    const breed = (ROLE_TO_BREED[agent.role] || 'generic') as any
    const working = agent.status === 'active' || agent.status === 'busy'
    if (working) {
      const d = desks[i % desks.length]
      return { agent, breed, spriteState: 'working' as const, x: d.x, y: d.y - 0.7, deskIdx: i % desks.length }
    }
    // lounge cats: roam within lounge bounds; fall back to grid layout before first tick
    const loungeStates = ['sleeping', 'eating', 'playing', 'idle'] as const
    const defaultState = loungeStates[i % loungeStates.length]
    const r = roam.value[agent.role]
    const spriteState = r?.state ?? defaultState
    const lx = r?.x ?? (5 + (i % 3) * 0.7)
    const ly = r?.y ?? (5 + Math.floor(i / 3) * 0.7)
    return { agent, breed, spriteState, x: lx, y: ly, deskIdx: -1 }
  })
})

// ═══ lounge roaming ═══
// Each idle cat gets a randomized target within lounge bounds. CSS transition on
// .cat-slot smooths the glide. Retarget every 6–10s per cat (staggered).
interface RoamSlot { x: number; y: number; state: 'sleeping' | 'eating' | 'playing' | 'idle' }
const roam = ref<Record<string, RoamSlot>>({})
const pairs = ref<Array<{ a: string; b: string; cx: number; cy: number }>>([])
const LOUNGE_BOUNDS = { x0: 4.6, x1: 7.2, y0: 4.6, y1: 7.2 }

function rand(min: number, max: number) { return min + Math.random() * (max - min) }
function pickState(): RoamSlot['state'] {
  const r = Math.random()
  if (r < 0.4) return 'sleeping'
  if (r < 0.65) return 'playing'
  if (r < 0.85) return 'eating'
  return 'idle'
}

function tickRoam() {
  const idleAgents = agents.value.filter(
    (a) => a.status !== 'active' && a.status !== 'busy'
  )
  // drop stale roles (agents disappeared or went to desk)
  const keep = new Set(idleAgents.map((a) => a.role))
  for (const role of Object.keys(roam.value)) {
    if (!keep.has(role)) delete roam.value[role]
  }
  // seed new, mutate existing
  for (const agent of idleAgents) {
    const cur = roam.value[agent.role]
    if (!cur) {
      roam.value[agent.role] = {
        x: rand(LOUNGE_BOUNDS.x0, LOUNGE_BOUNDS.x1),
        y: rand(LOUNGE_BOUNDS.y0, LOUNGE_BOUNDS.y1),
        state: pickState(),
      }
      continue
    }
    if (Math.random() < 0.6) {
      cur.x = rand(LOUNGE_BOUNDS.x0, LOUNGE_BOUNDS.x1)
      cur.y = rand(LOUNGE_BOUNDS.y0, LOUNGE_BOUNDS.y1)
      if (Math.random() < 0.35) cur.state = pickState()
    }
  }

  // ~35% chance per tick: pair up two idle cats at the same spot, both playing.
  // Records pair into `pairs` so we can render a sparkle between them.
  pairs.value = []
  const idleRoles = Object.keys(roam.value)
  if (idleRoles.length >= 2 && Math.random() < 0.35) {
    const shuffled = [...idleRoles].sort(() => Math.random() - 0.5)
    const a = shuffled[0], b = shuffled[1]
    const cx = rand(LOUNGE_BOUNDS.x0 + 0.3, LOUNGE_BOUNDS.x1 - 0.3)
    const cy = rand(LOUNGE_BOUNDS.y0 + 0.3, LOUNGE_BOUNDS.y1 - 0.3)
    roam.value[a] = { x: cx - 0.25, y: cy, state: 'playing' }
    roam.value[b] = { x: cx + 0.25, y: cy, state: 'playing' }
    pairs.value.push({ a, b, cx, cy })
  }
}

let roamTimer: ReturnType<typeof setInterval> | null = null

// desk is "active" if any working cat sits at it
function deskActive(i: number): boolean {
  return catSlots.value.some(s => s.deskIdx === i)
}

function truncate(s: string, max: number): string {
  return s.length > max ? s.slice(0, max - 1) + '…' : s
}

function truncateTask(s: string, max: number): string {
  return truncate(s, max)
}

function taskLineColor(status: string): string {
  if (status === 'active') return '#3aa35a'
  if (status === 'reviewing') return '#c77a5a'
  if (status === 'blocked') return '#c95a5a'
  return '#2a2a2a'
}

function onCatClick(agent: Agent) {
  router.push({
    name: 'agents',
    query: {
      role: agent.role,
      ...(agent.current_task ? { task: agent.current_task } : {}),
    },
  })
}
</script>

<style scoped>
.office-page {
  padding: 20px;
  background: #f5efdf;
  min-height: 100%;
  color: #2a2a2a;
  font-family: 'Patrick Hand', 'Comic Sans MS', 'Kalam', cursive;
  transition: background 0.4s ease;
}
.office-page.theme-night {
  background: #2a3326;
  color: #e8e2d0;
}
.office-header h2 { margin: 0 0 2px; font-size: 22px; }
.subtitle { opacity: 0.6; font-size: 13px; }
.live-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  margin-left: 14px;
  padding: 2px 10px;
  font-size: 12px;
  border: 1.5px solid #2a2a2a;
  border-radius: 999px;
  background: #fbf5e3;
  vertical-align: middle;
}
.live-badge .dot {
  width: 8px; height: 8px;
  border-radius: 50%;
  border: 1px solid #2a2a2a;
}
.live-badge.is-live .dot { background: #5aa35a; animation: live-pulse 1.6s ease-in-out infinite; }
.live-badge.is-dead .dot { background: #c95a5a; }
.live-badge.is-ws .dot   { background: #3a7ac9; animation: live-pulse 1.2s ease-in-out infinite; }
.live-badge small { opacity: 0.55; }

@keyframes live-pulse { 0%,100%{ box-shadow: 0 0 0 0 rgba(90,163,90,0.6) } 50%{ box-shadow: 0 0 0 4px rgba(90,163,90,0) } }

.stage {
  position: relative;
  width: 100%;
  height: 620px;
  overflow: hidden;
  border-radius: 14px;
  border: 2px solid #2a2a2a;
  background: #fbf5e3;
  box-shadow: inset 0 0 40px rgba(160, 130, 70, 0.18), 4px 4px 0 #2a2a2a;
  margin: 12px 0;
  transition: background 0.4s ease, box-shadow 0.4s ease, border-color 0.4s ease;
}
.stage-night {
  background: #1a2a24;
  border-color: #d8d0b8;
  box-shadow: inset 0 0 60px rgba(20, 40, 30, 0.8), 4px 4px 0 #d8d0b8;
}
.sky {
  position: absolute;
  top: 0; left: 0;
  width: 100%;
  height: 140px;
  pointer-events: none;
}
.star { animation: star-twinkle 3s ease-in-out infinite; transform-origin: center; transform-box: fill-box; }
.s2 { animation-delay: 0.4s; }
.s3 { animation-delay: 0.9s; }
.s4 { animation-delay: 1.6s; }
.s5 { animation-delay: 2.1s; }
.s6 { animation-delay: 0.2s; }
.firefly { animation: firefly-glow 2.4s ease-in-out infinite; }

@keyframes star-twinkle { 0%,100%{ opacity: 0.4 } 50%{ opacity: 1 } }
@keyframes firefly-glow {
  0%,100%{ opacity: 0.2; filter: drop-shadow(0 0 0 #ffe488) }
  50%   { opacity: 1;   filter: drop-shadow(0 0 3px #ffe488) }
}

.theme-btn {
  margin-left: auto;
  padding: 3px 10px;
  font-family: inherit;
  font-size: 13px;
  background: #fbf5e3;
  color: #2a2a2a;
  border: 1.5px solid #2a2a2a;
  border-radius: 999px;
  cursor: pointer;
  box-shadow: 2px 2px 0 #2a2a2a;
}
.theme-btn:active { transform: translate(1px, 1px); box-shadow: 1px 1px 0 #2a2a2a; }
.theme-night .theme-btn { background: #2a3326; color: #e8e2d0; border-color: #e8e2d0; box-shadow: 2px 2px 0 #e8e2d0; }
.theme-night .live-badge { background: #2a3326; border-color: #e8e2d0; color: #e8e2d0; }
.theme-night .name-tag { background: #2a3326; border-color: #e8e2d0; color: #e8e2d0; }
.theme-night .legend { background: #2a3326; border-color: #e8e2d0; color: #e8e2d0; }
.theme-night .task-bubble { background: #e8e2d0; color: #2a2a2a; }
.theme-night .task-bubble::before { border-top-color: #e8e2d0; }
.paper-grid {
  position: absolute;
  inset: 0;
  opacity: 0.35;
  pointer-events: none;
}

.iso-world {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(0, -120px);
}

.tile {
  position: absolute;
  width: 96px;
  height: 48px;
  margin-left: -48px;
  margin-top: -24px;
}
.desk {
  position: absolute;
  width: 80px;
  height: 64px;
  margin-left: -40px;
  margin-top: -40px;
}
.prop {
  position: absolute;
  width: 64px;
  height: 60px;
  margin-left: -32px;
  margin-top: -40px;
  overflow: visible;
}
.desk--active .monitor-code path { animation: code-scroll 1.2s steps(4) infinite; transform-origin: left; }
.desk--active .steam path { animation: steam-rise 2.2s ease-in-out infinite; transform-origin: 58px 34px; transform-box: fill-box; }
/* night campfire desk */
.desk--night .fire-halo { animation: halo-pulse 2.6s ease-in-out infinite; transform-origin: center; transform-box: fill-box; }
.desk--night.desk--active .flame-outer { animation: flame-flicker 0.22s ease-in-out infinite; transform-origin: 0 0; transform-box: fill-box; }
.desk--night.desk--active .flame-inner { animation: flame-flicker 0.17s ease-in-out infinite reverse; transform-origin: 0 -2px; transform-box: fill-box; }
.desk--night.desk--active .ember.e1 { animation: ember-float 2.1s ease-in 0.1s infinite; }
.desk--night.desk--active .ember.e2 { animation: ember-float 2.4s ease-in 0.7s infinite; }
.desk--night.desk--active .ember.e3 { animation: ember-float 1.9s ease-in 1.3s infinite; }
.desk--night.desk--active .laptop-code path { animation: code-scroll 1.2s steps(4) infinite; transform-origin: left; }

@keyframes halo-pulse    { 0%,100%{ opacity: 0.15; transform: scaleX(0.95) } 50%{ opacity: 0.3; transform: scaleX(1.05) } }
@keyframes flame-flicker { 0%,100%{ transform: scaleY(1) scaleX(1) } 50%{ transform: scaleY(1.15) scaleX(0.9) } }
@keyframes ember-float   { 0%{ opacity: 1; transform: translate(0, 0) } 100%{ opacity: 0; transform: translate(1px, -10px) } }

.feather { animation: feather-sway 2.6s ease-in-out infinite; transform-origin: -6px -6px; transform-box: fill-box; }
.feather-string { animation: feather-sway 2.6s ease-in-out infinite; transform-origin: 3px 0; transform-box: fill-box; }
.yarn-pile { animation: yarn-bob 3.2s ease-in-out infinite; transform-origin: center; transform-box: fill-box; }

@keyframes code-scroll  { 0%{ transform: scaleX(0.4); opacity: 0.4 } 50%{ transform: scaleX(1); opacity: 1 } 100%{ transform: scaleX(0.6); opacity: 0.4 } }
@keyframes steam-rise   { 0%,100%{ transform: translateY(0); opacity: 0.3 } 50%{ transform: translateY(-3px); opacity: 0.7 } }
@keyframes feather-sway { 0%,100%{ transform: rotate(-8deg) } 50%{ transform: rotate(8deg) } }
@keyframes yarn-bob     { 0%,100%{ transform: translateY(0) } 50%{ transform: translateY(-1.5px) } }
.lounge {
  position: absolute;
  width: 200px;
  height: 100px;
  margin-left: -100px;
  margin-top: -50px;
  pointer-events: none;
}
.cat-slot {
  position: absolute;
  margin-left: -40px;
  margin-top: -60px;
  z-index: 10;
  text-align: center;
  cursor: pointer;
  transition: transform 4.5s cubic-bezier(0.45, 0.05, 0.55, 0.95);
}
.pair-spark {
  position: absolute;
  width: 80px;
  height: 80px;
  margin-left: -40px;
  margin-top: -80px;
  z-index: 9;
  pointer-events: none;
  overflow: visible;
}
.spark-heart { animation: heart-pop 1.2s ease-in-out infinite; transform-origin: 20px 20px; transform-box: fill-box; }
.spark-stars .s-a { animation: spark-twinkle 1.4s ease-in-out infinite; }
.spark-stars .s-b { animation: spark-twinkle 1.4s ease-in-out 0.3s infinite; }
.spark-stars .s-c { animation: spark-twinkle 1.4s ease-in-out 0.7s infinite; }
@keyframes heart-pop { 0%,100%{ transform: scale(0.9) } 50%{ transform: scale(1.1) } }
@keyframes spark-twinkle { 0%,100%{ opacity: 0.3; transform: scale(0.8) } 50%{ opacity: 1; transform: scale(1.1) } }
.cat-anchor {
  position: relative;
  transition: transform 0.12s ease-out;
}
.cat-slot:hover { z-index: 20; }
.cat-slot:hover .cat-anchor { transform: translateY(-2px) scale(1.04); }
.cat-slot:hover .name-tag { background: #ffe08a; }

.task-bubble {
  position: absolute;
  left: 50%;
  top: -18px;
  transform: translateX(-50%);
  padding: 2px 8px;
  font-size: 11px;
  background: #fbf5e3;
  border: 1.5px solid #2a2a2a;
  border-radius: 10px;
  white-space: nowrap;
  box-shadow: 2px 2px 0 #2a2a2a;
  pointer-events: none;
  max-width: 160px;
  overflow: hidden;
  text-overflow: ellipsis;
  display: flex;
  align-items: center;
  gap: 4px;
}
.task-bubble::after {
  content: '';
  position: absolute;
  bottom: -6px;
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  height: 0;
  border-left: 5px solid transparent;
  border-right: 5px solid transparent;
  border-top: 6px solid #2a2a2a;
}
.task-bubble::before {
  content: '';
  position: absolute;
  bottom: -4px;
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  height: 0;
  border-left: 4px solid transparent;
  border-right: 4px solid transparent;
  border-top: 4px solid #fbf5e3;
  z-index: 1;
}
.bubble-label { font-size: 10px; }
.bubble-text { max-width: 140px; overflow: hidden; text-overflow: ellipsis; }
.name-tag {
  display: inline-block;
  margin-top: -4px;
  padding: 1px 6px;
  font-size: 11px;
  background: #fbf5e3;
  border: 1px solid #2a2a2a;
  border-radius: 8px;
  white-space: nowrap;
}

.legend {
  display: flex;
  gap: 14px;
  font-size: 13px;
  opacity: 0.75;
  flex-wrap: wrap;
  padding: 6px 10px;
  background: #fbf5e3;
  border: 1.5px dashed #2a2a2a;
  border-radius: 10px;
  width: fit-content;
}
.legend small { margin-left: 12px; opacity: 0.55; }
</style>
