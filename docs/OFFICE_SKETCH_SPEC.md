# Office Sketch Style Spec

> Catcoding dashboard Office 场景 · 手绘涂鸦风格资源规格
> 版本 0.1 · 2026-04-18 · 随 `views/Office.vue` + `components/CatSprite.vue` 同步更新

冻结本规格是为了之后加新品种、新场景、夜间田园模式时不产生规格漂移。
本文件不是使用说明,是**生产约束**——新资源必须对齐这里列出的尺寸、描边、配色、动画时序。

---

## 1. 风格定位

- **唯一样式车道**:笔记本速写 (notebook sketch) + 马克笔线稿
- 背景:米色纸 (#fbf5e3) + 浅网格 (24px 间距, 0.4px stroke, #c7b98a @ 35%)
- 字体:`'Patrick Hand', 'Comic Sans MS', 'Kalam', cursive`(手写)
- 边框:2px 黑描边 (#2a2a2a) + 4px 偏移阴影盒 (`4px 4px 0 #2a2a2a`)
- **禁止**:CSS 渐变填充、emoji-as-placeholder、Material/Tailwind default shadow

---

## 2. 配色

### 通用

| 用途 | HEX | 备注 |
|---|---|---|
| 墨线 | `#2a2a2a` | 所有 stroke 默认色 |
| 纸张底色 | `#fbf5e3` | stage 背景、白板内嵌纸 |
| 纸张更浅 | `#fdfcf4` | 白板主体 |
| 网格线 | `#c7b98a` | 仅背景网格使用 |

### 地板

| 瓦片 | fill | 纹理 stroke |
|---|---|---|
| 草地 | `#b9d89e` | 小草 `#4d7a3a` 0.7px,粉花 `#d88fb7` |
| 木地毯 | `#e0b88a` | 木纹 `#7a5a2a` 0.5-0.6px @ 50% |

### 家具 / 道具

| 元素 | 主填充 | 辅色 |
|---|---|---|
| 办公桌面 | `#e8c892` | 侧面 `#c99f5a` / 更深 `#b88a46` |
| 显示器外壳 | `#f5efdf` | 屏幕代码 绿 `#3aa35a` · 橙 `#c77a5a` · 灰 `#6a7a8a` · 静态灰 `#c0b090` |
| 键盘 | `#cfc6b0` | 按键面 `#f5efdf`,闪烁高亮 `#ffd866` |
| 马克杯 | `#d97a5a` | 把手线 `#8a4a2a` |
| 蒸汽 | `#b0b0b0` @ 60% |  |
| 地毯(lounge) | `#ecb6c8` | 花纹 `#a64a7a` @ 50% |
| 食碗 | `#c6a07a` | 鱼身 `#eac78a` |
| 盆栽 | 陶盆 `#c97a58` · 泥土 `#8a4a2a` · 叶 `#5a9a3a` / `#6aa34a` / `#7ab35a` |
| 饮水机 | 蓝顶 `#a0c8d8` / `#8ab8d0` · 机身 `#e8e4d0` · 红龙头 `#c95a3a` · 蓝龙头 `#3a7ac9` |
| 垃圾桶 | `#6a7a8a` / 口 `#8a9aa8` |
| 便签 | 黄 `#ffe08a` · 绿 `#a8d8a0` · 粉 `#f0a8b8` |
| 毛线球 | 粉 `#c97a98` · 蓝 `#7a9ac9` |
| 猫抓柱 | `#b88a56` / 底座 `#8a6a46` · 羽毛 `#c9a06a` |

### 品种填充(`CatSprite` BREED_META)

| breed | fill | pattern |
|---|---|---|
| siamese | `#e8d7b8` | `points` (深面罩 `#3a2a22` 点两爪) |
| british_blue | `#8c9aa8` | `none` |
| orange_tabby | `#e89a4e` | `tabby` (3 条波纹) |
| maine_coon | `#8a6a4e` | `tabby` |
| black | `#2a2a2a` | `tuxedo` (白胸腹 `#fdf6e0`) |
| abyssinian | `#b87a4a` | `tabby` |
| dragon_li | `#a89068` | `tabby` |
| owl | `#9a8058` | `none` |
| fox | `#d87040` | `none` |
| panda | `#fafafa` | `tuxedo` |
| generic | `#d8c8a8` | `none` |

---

## 3. 等距投影

- **瓦片尺寸**:96 × 48 px (2:1 菱形)
- **投影公式**:`screenX = (x − y) × 48`, `screenY = (x + y) × 24 − z`
- 世界网格:8 列 × 8 行(当前)
- 瓦片 SVG viewBox `0 0 96 48`;`wobblyDiamond(seed)` 抖动 ±1px 保持手绘感

### 各资源定位锚点

| 资源 | 放置层高 z | viewBox | DOM 尺寸 | margin 偏移 |
|---|---|---|---|---|
| tile 地砖 | 0 | 96×48 | 96×48 | -48 / -24 |
| desk 办公桌 | 10 | 80×64 | 80×64 | -40 / -40 |
| lounge 地毯+道具 | 4 | 200×100 | 200×100 | -100 / -50 |
| prop 场景小物 | 0–40 (按需) | 自定 | 64×60 | -32 / -40 |
| cat sprite | 24(工位上)/0(lounge) | 80×80 | 80×80 | -40 / -60 |

层高 z 是 **屏幕向上偏移**,用于让高于地面的物体(桌/椅/猫)不被后面的 tile 盖住——越高的物体 z 越大。

---

## 4. 描边与形态

- 主轮廓 stroke-width:
  - 猫主体(body/head/tail/ears): **1.6px**
  - 家具外轮廓: **1.2–1.4px**
  - 道具/细节: **1.0–1.2px**
  - 发丝纹理(木纹/胡须/草尖): **0.5–0.8px**
- `stroke-linejoin: round`, `stroke-linecap: round`(手绘感必需,禁 square)
- 所有闭合 path 必须手写 wobbly(每个锚点 ±0.5–2px 抖动)——不要用工整几何

### 品种区分约定(添加新猫时遵循)

1. 改 `CatSprite.vue` → `BREED_META`:加 `fill` + `pattern: 'none' | 'tabby' | 'points' | 'tuxedo'`
2. 如果需要新 pattern,在 sprite template `<g class="pattern" v-else-if="...">` 里加一个分支
3. 添加对应 `role → breed` 映射到 `Office.vue` `ROLE_TO_BREED`
4. 不要改 viewBox 或主体 path(品种之间共享骨架)

---

## 5. Cat Sprite 合约

- **viewBox**: `0 0 80 80`
- **props**: `breed: Breed`, `state: State`, `size?: number | string`(默认 80), `glow?: boolean`(默认 false,夜间工位开)
- **State**: `'working' | 'idle' | 'sleeping' | 'eating' | 'playing'`
- 主 `body` path 随 state 切换(sleeping 时替换为蜷缩 path)
- `tail` path 随 state 切换
- 眼睛 sleeping 时 `scaleY(0.15)` 闭眼

### 状态动画时序(keyframe / 时长 / 缓动)

| state | 动画链 | 时长 | 缓动 |
|---|---|---|---|
| working | `nod` (head) + `type-bob` (body) | 0.6s | ease-in-out infinite |
| working | `key-press` (10 键错峰) | 0.9s | ease-in-out infinite |
| idle | `tail-flick` | 2.4s | ease-in-out infinite |
| sleeping | `breathe` (body) | 3s | ease-in-out infinite |
| sleeping | `zzz-float` ×3 错峰 (0 / 0.5s / 1s) | 2.8s | ease-in-out infinite |
| eating | `munch` (head) | 0.8s | ease-in-out infinite |
| eating | `crumb-fall` ×3 错峰 (0 / 0.3 / 0.7s) | 1.4s | ease-out infinite |
| playing | `pounce` (head) + `tail-flick` (tail) | 0.8s / 0.5s | ease-in-out infinite |

### 场景级动画(Office.vue)

| 元素 | keyframe | 时长 |
|---|---|---|
| 显示器代码条 | `code-scroll` (steps 4) | 1.2s |
| 马克杯蒸汽 | `steam-rise` | 2.2s |
| 逗猫棒羽毛 | `feather-sway` | 2.6s |
| 毛线球堆 | `yarn-bob` | 3.2s |

**约定**:新增动画时,单元素 ≤2 条并行 keyframe,时长选 `0.5–3.2s` 之间,避免页面"抖得慌"。

---

## 6. 扩展流程

### 加一只新品种

1. `CatSprite.vue`: 在 `Breed` 联合类型追加名字
2. `BREED_META`: 填 `fill` + `pattern`
3. 若 pattern 不足以表达(异国短毛那种脸型),加一个 `<g class="pattern" v-else-if="...">` 分支,**不要**改 body/head path
4. `Office.vue` `ROLE_TO_BREED` 补映射
5. 记得同步 `docs/OFFICE_SKETCH_SPEC.md` 的品种表

### 加一个新场景道具

1. `Office.vue` `Prop['kind']` 加 kind
2. SVG 模板里加 `<g v-else-if="p.kind === '...'">` 分支——严格遵守本 spec 的描边/配色
3. `sceneProps` 数组追加一项,选空 tile 位置
4. 如需动画,timer 区间遵循 §5 约定

### 未来:夜间田园模式(保留槽位)

- 背景色切为深卡其 `#2a3326`,纸网格改蓝紫 `#5a6a8a`
- 草地 fill → 深绿 `#3a6a3a`,加萤火虫 `<circle>` 小光点
- desk → 露营桌/篝火,显示器 → 笔记本电脑 **(已实现 2026-04-18)**
- sprite 眼睛可改成圆珠笔点 + 猫眼发光(`filter: drop-shadow`) **(已实现 2026-04-18 — `glow` prop)**
- **不要**切掉描边和手绘 wobbly 路径——夜间仍是 sketch 车道,只是换调色板

### 夜间篝火工作站(已实现)

当 `theme === 'night'` 时,desk SVG 走 `<template v-else>` 分支渲染露营布景:
- 树桩座椅(背景层):`ellipse` + 侧面阴影 path,两圈年轮;填色 `#6a4a2a` / 侧面 `#4a3018` / 顶面 `#8a6a4a`,描边改 `palette.ink` 米色(`#e8e2d0`)在深底可见
- 笔记本(石头上):上下两段 path 拼 L 形侧视,屏幕 4 行代码条;active 时 `code-scroll` keyframe 驱动,idle 时灰色静态
- 篝火:4 颗石头环 + 两根交叉木柴 + 火焰双层(外 `#ffb040` + 内 `#ffe070`),`flame-flicker` 22ms/17ms 反向缩放;3 颗余烬 `ember-float` 2s 错峰上飘
- 地面火光 `fire-halo`:工位周围 30×10 橙色 ellipse,`halo-pulse` 2.6s 呼吸
- 棉花糖签:active 时左侧加一根插着棉花糖的小棍(纯装饰)
- idle 篝火:无火焰,改为一缕灰色烟线 `#6a6a7a` @ 50%

**描边规则**:日景 desk 用 `#2a2a2a` 黑线,夜景 camping 全部改 `#e8e2d0` 米线(纸本反色依然 sketch 风)。
