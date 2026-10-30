# AGENTS.md

This file is the shared contract for Codex, Claude Code, and other coding agents working in this repository.

## Project

`peep` is a zero-config Rust terminal dashboard (ratatui) that watches local AI coding agent session logs from Claude Code, Codex CLI, and Gemini CLI and renders them with pixel art.

- `src/collector/`: `jsonl_watcher.rs` (default local log watcher) and `http_server.rs` (optional hooks on port 3100)
- `src/protocol/`, `src/store/`, `src/tui/`: event parsing, state, and rendering
- `src/update.rs`: auto-update
- `Formula/`: Homebrew formula; releases run from `v*` tags
- `docs/superpowers/`: design specs and plans

## Commands

```bash
cargo check
cargo clippy -- -D warnings
cargo test
cargo run -- --mock                # preview the UI without real sessions
scripts/verify_launch_paths.sh     # tests, offline cargo install, launch paths
```

## Rules

- Run `cargo clippy -- -D warnings` and `cargo test` before committing; CI runs the same checks.
- Keep the default path local-only and zero-config: no API keys, no network calls, and no data leaving the machine. HTTP hooks stay optional.
- When a supported tool changes its log location or format, update the parser and the Supported AI Tools table in `README.md` together.
- Release by pushing a `v*` tag only when the user asks.
