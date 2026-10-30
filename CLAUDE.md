# Claude Code

@AGENTS.md

# Claude Code Mapping

- `AGENTS.md` is the shared rule source for Codex and Claude Code. Change project rules there; keep this file to Claude-specific differences.
- Sessions target Opus 5.5 (`claude-opus-5-5`) at the default effort. Apply each rule as written and keep changes inside the requested scope.
- peep reads Claude Code session JSONL under `~/.claude/projects/`. When testing against real logs, quote only what the task needs and prefer `peep --mock` for UI checks.
