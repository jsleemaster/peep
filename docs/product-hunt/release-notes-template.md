# peep v0.5.13

`peep` is a zero-config terminal dashboard for watching AI coding agents work in real time.

## Highlights

- Zero-config local log watching for Claude Code, Codex CLI, and Gemini CLI
- Pixel-art party view for the lead agent and sub-agents
- Project tabs and sub-agent focus mode
- Demo mode with `peep --mock`

## Install

### Homebrew

```bash
brew tap jsleemaster/tap
brew install peep
```

### Quick Demo

```bash
PEEP_NO_AUTO_UPDATE=1 peep --mock
```

### Direct Download

- macOS Apple Silicon: `peep-macos-arm64.tar.gz`
- macOS Intel: `peep-macos-intel.tar.gz`
- Linux x86_64: `peep-linux-x86_64.tar.gz`
- Linux arm64: `peep-linux-arm64.tar.gz`

## What To Try

- Run `peep` while an AI coding tool is active
- Switch projects with `[` and `]`
- Focus a sub-agent with `Enter`
- Return to the leader with `Esc`

## Notes

- Default path is local-only log watching
- Optional HTTP hooks still exist for lower-latency events
- Product Hunt launch assets live in `docs/product-hunt/`
