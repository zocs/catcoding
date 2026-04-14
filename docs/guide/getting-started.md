# Getting Started

## Prerequisites

- **Rust** 1.75+ ([install](https://rustup.rs))
- **Python** 3.10+ (for Agent SDK)
- **Node.js** 18+ (for Dashboard)
- **NATS Server** (auto-started or [install](https://docs.nats.io/running-a-nats-service/introduction))

## Installation

### From Source

```bash
git clone https://github.com/zocs/catcoding.git
cd catcoding
cargo build --release

# Binaries will be at:
# target/release/catcoding-daemon
# target/release/catcoding
```

### One-liner (Coming Soon)

```bash
curl -fsSL https://catcoding.org/install.sh | bash
```

## Quick Start

### 1. Initialize a Project

```bash
cd your-project
catcoding init
```

This creates a `.agent.yaml` configuration file:

```yaml
project:
  name: "my-project"

agents:
  pm:
    enabled: true
    adapter: "hermes"
  core_dev:
    enabled: true
    adapter: "claude-code"

watchdog:
  heartbeat_timeout: 30
  max_restarts: 3
```

### 2. Start the Daemon

```bash
catcoding serve
```

The daemon starts:
- **HTTP API** at `http://127.0.0.1:9527`
- **Dashboard** at `http://localhost:8080`

### 3. Open Dashboard

Open `http://localhost:8080` in your browser.

## Configuration

See [`.agent.yaml.example`](../agent.yaml.example) for all available options.

### Adapters

| Adapter | Config Key | Requirements |
|---------|-----------|--------------|
| Hermes | `hermes` | hermes-agent installed |
| Claude Code | `claude-code` | `claude` CLI installed |
| Codex | `codex` | `codex` CLI installed |

### Watchdog Settings

```yaml
watchdog:
  heartbeat_timeout: 30    # seconds before agent considered dead
  max_restarts: 3           # max restart attempts per agent
  compile_check: true       # verify code compiles
  api_call_tracking: true   # track API call durations
```

## Next Steps

- [Architecture Overview](../README.md#architecture)
- [Agent Roles](https://github.com/zocs/catcoding#-agent-team)
- [Dashboard Guide](../dashboard/README.md)
- [Configuration Reference](../agent.yaml.example)
