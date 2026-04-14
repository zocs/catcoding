# 🐱 CatCoding

> Let AI cats collaborate like a dev team

**[🎮 Live Demo](https://demo.catcoding.org)** · **[中文版](README_zh-CN.md)** · **[Getting Started](docs/guide/getting-started.md)** · **[Website](https://catcoding.org)**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![CI](https://github.com/zocs/catcoding/actions/workflows/ci.yml/badge.svg)](https://github.com/zocs/catcoding/actions/workflows/ci.yml)

CatCoding is a **framework-agnostic multi-agent collaborative software development framework**. It organizes multiple AI coding agents into an efficient team through a pluggable Adapter interface, supporting Hermes, Claude Code, Codex, and more.

## ✨ Features

- 🐱 **Cat Persona System** — Each agent has a unique cat character with SVG avatars
- 🦉 **Watchdog Guardian** — Triple-detection framework with auto-recovery
- 📋 **Visual Dashboard** — Real-time Kanban, Gantt charts, agent status monitoring
- 🔄 **Multi-Adapter** — Pluggable adapters: Hermes, Claude Code, Codex, OpenClaw
- 🧠 **L4 Memory** — Four-layer memory: Index → Facts → Skills → Sessions
- 🐛 **Bug = Mouse** — Fun bug classification system (catch mice = fix bugs)

## 🏗️ Architecture

```
┌──────────────────────────────────────────────┐
│                    User Layer                 │
│         Dashboard (Vue 3)      CLI           │
└────────────────────┬─────────────────────────┘
                     │
┌────────────────────▼─────────────────────────┐
│          CatCoding Daemon (Rust)              │
│  ┌───────────┐ ┌───────────┐ ┌────────────┐  │
│  │  Watchdog │ │ Scheduler │ │   Router   │  │
│  │  Triple   │ │ Dep-gated │ │  NATS Msg  │  │
│  └───────────┘ └───────────┘ └────────────┘  │
│  ┌──────────────────────────────────────┐    │
│  │           Adapter Layer              │    │
│  │   Hermes  │  Claude Code  │  Codex   │    │
│  └──────────────────────────────────────┘    │
└────────────────────┬─────────────────────────┘
                     │
┌────────────────────▼─────────────────────────┐
│           Python Agent SDK                    │
│  PM (Siamese) │ Dev (British) │ Rev (Bombay)  │
└──────────────────────────────────────────────┘
```

## 🚀 Quick Start

See **[Getting Started](docs/guide/getting-started.md)** for full instructions.

```bash
# Install (coming soon)
curl -fsSL https://catcoding.org/install.sh | bash

# Or build from source
git clone https://github.com/zocs/catcoding.git
cd catcoding && cargo build --release

# Initialize in your project
cd your-project && catcoding init

# Start daemon + dashboard
catcoding serve
# → Dashboard: http://localhost:8080
# → API: http://127.0.0.1:9527
```

## 🐱 Agent Team

| Role | Cat | Responsibility |
|------|-----|----------------|
| PM | 🐱 Siamese | Requirements, task breakdown, progress tracking |
| Core Dev | 🐱 British Shorthair | Code implementation |
| Reviewer | 🐱 Bombay | Code review, bug detection |
| Tester | 🐱 Abyssinian | Test cases, quality assurance |
| Tech Scout | 🦊 Fox | Research, documentation gathering |

## 🔌 Supported Adapters

| Adapter | Status | Description |
|---------|--------|-------------|
| Hermes | ✅ | hermes-agent integration |
| Claude Code | ✅ | claude CLI integration |
| Codex | ✅ | codex CLI integration |
| OpenCode | 🔜 | Planned |

## 🛠️ Development

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo test               # Run tests
make ci                  # Full CI pipeline (lint + test + build)
```

## 📁 Project Structure

```
catcoding/
├── daemon/           # Rust Daemon core
│   └── src/
│       ├── api/      # HTTP API (Axum)
│       ├── adapter/  # Agent Adapters
│       ├── watchdog.rs
│       └── scheduler.rs
├── cli/              # CLI tool
├── agents/           # Python Agent SDK
│   ├── base/         # Base Agent
│   ├── pm/           # PM Agent
│   └── reviewer/     # Review Agent
├── dashboard/        # Vue 3 Dashboard
└── config/           # Configuration
```

## 📝 License

[MIT License](LICENSE)
