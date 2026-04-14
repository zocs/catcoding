# CatCoding 演示 GIF 制作指南

## 🎬 演示内容

### GIF 1: 快速开始 (15秒)
```
1. 终端运行: catcoding init (2秒)
2. 终端运行: catcoding serve (3秒)
3. 浏览器打开 Dashboard (2秒)
4. 展示看板视图 (3秒)
5. 创建任务 (3秒)
6. 猫咪状态动画 (2秒)
```

### GIF 2: 多 Agent 协作 (20秒)
```
1. 创建3个任务 (4秒)
2. 分配给不同猫咪 (4秒)
3. 看板任务流转 (4秒)
4. 猫咪面板工作动画 (4秒)
5. Bug 老鼠系统 (4秒)
```

### GIF 3: 彩蛋展示 (10秒)
```
1. 连续完成任务 (3秒)
2. 触老虎彩蛋 (3秒)
3. 熊猫庆祝动画 (4秒)
```

## 🛠️ 录制工具

### Linux/WSL
```bash
# 使用 peek (GTK)
sudo apt install peek

# 或使用 byzanz (命令行)
byzanz-record -d 15 -x 0 -y 0 -w 800 -h 600 demo.gif

# 或使用 gifski + 录屏
sudo apt install gifski
```

### macOS
```bash
# 使用 Gifski (推荐)
brew install gifski
# 用 QuickTime 录屏，然后用 gifski 转换

# 或使用 Kap
brew install --cask kap
```

### Windows
```bash
# 使用 ScreenToGif
scoop install screentogif

# 或使用 LICEcap (WSL中不可用)
```

## 📐 录制参数

- **分辨率**: 1280x720 或 800x600
- **帧率**: 15fps
- **时长**: 10-20秒
- **文件大小**: < 5MB (GitHub README 限制)
- **循环**: 无限循环

## 🎨 美化建议

1. 使用深色终端主题（CatCoding 深色主题）
2. 字体大小 14-16px
3. 命令有轻微延迟，更自然
4. 鼠标点击有高亮效果
5. 添加字幕标注关键步骤

## 📁 输出位置

```
catcoding/
├── assets/
│   └── demos/
│       ├── quick-start.gif
│       ├── multi-agent.gif
│       └── easter-eggs.gif
└── README.md (引用这些 GIF)
```

## 📝 README 引用

```markdown
## 🎬 演示

### 快速开始
![Quick Start](assets/demos/quick-start.gif)

### 多 Agent 协作
![Multi Agent](assets/demos/multi-agent.gif)

### 彩蛋系统
![Easter Eggs](assets/demos/easter-eggs.gif)
```
