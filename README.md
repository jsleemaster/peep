# peep

> Peep into your AI coding agents

Terminal dashboard that monitors AI coding agents in real-time. Watch your agents work with pixel-art characters, conversation timelines, and multi-project support.

```
peep
 ●1 ◐0 │ 🐔1 🐣1 │ tokens:168.8k │ $0.00         q:quit j/k:scroll [,]:project
┌──────────────────────┬─────────────────────────────────────────────────────────┐
│ refactored-badger LEAD│ conversation                                           │
│                      │  방금 ▶ auth module 리팩토링 해줘                       │
│     🐔               │  방금 ├─🐔 네, auth module을 확인하겠습니다.            │
│                      │  방금 │ ├─⬤ Read src/auth.ts                           │
│ HP ████████░░ 83%    │  방금 │ └─⬤ Read src/config.ts                         │
│                      │  방금 ├─🐔 리팩토링을 시작합니다.                       │
│ ── party (1) ──────  │  방금 │ ├─⬤ Edit src/auth.ts                           │
│  🥚 sub-worker      │  방금 │ └─⬤ Bash npm test                              │
│     hatching...      │                                                         │
└──────────────────────┴─────────────────────────────────────────────────────────┘
```

## Features

- **Pixel art characters** — Leader agent is a mother hen, sub-agents start as eggs and grow into chicks
- **Conversation timeline** — See prompts, responses, and tool calls in a threaded view
- **Multi-project** — Switch between projects with `[` `]` keys
- **Multi-AI support** — Claude Code, Codex CLI, Gemini CLI, OpenCode (auto-detected)
- **Status dots** — Orange (running), green (success), red (error)
- **HP bar** — Context window usage as health bar
- **Dark/Light theme** — Auto-detects or set with `--theme`
- **Zero config** — Just run `peep`, it auto-discovers AI session logs

## Supported AI Tools

| Tool | Detection | Status |
|------|-----------|--------|
| Claude Code | `~/.claude/projects/**/*.jsonl` | ✅ Full support |
| Codex CLI | `~/.codex/sessions/**/*.jsonl` | ✅ Auto-detected |
| Gemini CLI | `~/.gemini/logs/sessions/` | ✅ Auto-detected |
| OpenCode | `.opencode/logs/` | 🔜 Coming soon |

## Installation

### Download binary (easiest)

Download the latest release for your platform:

```bash
# macOS (Apple Silicon)
curl -L https://github.com/jsleemaster/peep/releases/latest/download/peep-macos-arm64.tar.gz | tar xz
sudo mv peep /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/jsleemaster/peep/releases/latest/download/peep-macos-intel.tar.gz | tar xz
sudo mv peep /usr/local/bin/

# Linux (x86_64)
curl -L https://github.com/jsleemaster/peep/releases/latest/download/peep-linux-x86_64.tar.gz | tar xz
sudo mv peep /usr/local/bin/
```

### Homebrew (macOS / Linux)

```bash
brew tap jsleemaster/tap
brew install peep
```

### Cargo (from source)

Requires [Rust](https://rustup.rs/) 1.75+:

```bash
cargo install --git https://github.com/jsleemaster/peep
```

## Usage

```bash
# Just run it — auto-discovers AI session logs
peep

# Light theme
peep --theme light

# Custom HTTP hook port
peep --port 4000

# Disable JSONL file watcher (HTTP hooks only)
peep --no-jsonl

# Demo with mock data
peep --mock
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `q` | Quit |
| `j` / `k` | Scroll conversation |
| `[` / `]` | Switch project |
| `h` / `l` | Focus left/right panel |
| `f` | Filter events |
| `g` / `G` | Scroll to top/bottom |
| `Enter` | Agent detail overlay |

## How It Works

peep monitors AI agent activity through two channels:

1. **JSONL file watcher** (default) — Watches `~/.claude/projects/`, `~/.codex/sessions/`, `~/.gemini/logs/sessions/` for new log entries. Zero configuration needed.

2. **HTTP hooks** (optional) — Listens on port 3100 for POST requests. Configure in your AI tool's hook settings for real-time events.

### Agent Growth System

Sub-agents evolve based on their usage count:

| Stage | Usage | Character |
|-------|-------|-----------|
| Egg | 0-4 | 🥚 Brand new |
| Cracking | 5-9 | 🪺 Warming up |
| Peeking | 10-19 | 🐥 Head poking out |
| Chick | 20+ | 🐣 Fully hatched |
| Done | completed | ⭐ Trophy |

### Claude Code HTTP Hooks (Optional)

For real-time events, add hooks to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PostToolUse": [{
      "type": "command",
      "command": "curl -s -X POST http://localhost:3100/hook -H 'Content-Type: application/json' -d \"$CLAUDE_HOOK_EVENT\""
    }]
  }
}
```

## Configuration

Optional config file at `~/.config/peep/config.toml`:

```toml
[server]
port = 3100
bind = "127.0.0.1"

[watcher]
enabled = true
# watch_dir = "~/.claude/projects"

[tui]
tick_rate = 100
```

## Requirements

- **OS**: macOS, Linux (Windows via WSL)
- **Terminal**: Any terminal with 256-color or truecolor support (iTerm2, Terminal.app, Alacritty, Kitty, WezTerm, etc.)
- **Rust**: 1.75+ (for building from source)

## License

MIT
