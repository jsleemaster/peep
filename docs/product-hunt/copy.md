# Product Hunt Copy

Source of truth for the launch copy checked against Product Hunt's submission guide on 2026-03-31:
<https://www.producthunt.com/launch/preparing-for-launch>

## Core Positioning

- Primary line: real-time terminal dashboard for AI coding agents
- Repeated proof points: zero config, local log watching, multi-agent visibility, playful but useful
- Avoid: language that makes `peep` sound like an agent control plane or hosted platform

## Submission Fields

- Product name: `peep`
- Tagline: `A terminal dashboard for watching AI coding agents live`
- Launch tags: `Developer Tools`, `Terminal Tools`, `Artificial Intelligence`
- URL: `https://github.com/jsleemaster/peep`
- Additional links:
  - `https://github.com/jsleemaster/peep/releases/latest`
  - `https://github.com/jsleemaster/homebrew-tap`

### Description

`peep` watches the local session logs from Claude Code, Codex, and Gemini and turns them into a live terminal dashboard. Follow the lead agent, inspect sub-agents, switch between projects, and watch the timeline update in real time without leaving your terminal or sending data to a cloud service.

## Full Description

`peep` is a zero-config terminal dashboard for people who already live inside AI coding tools.

Instead of opening raw JSONL logs or guessing what your agents are doing, `peep` gives you a live party view with pixel-art characters, a running conversation timeline, project tabs, and sub-agent focus mode. It reads the local logs that tools like Claude Code, Codex CLI, and Gemini CLI already write during normal use, so you can install it and start watching immediately.

What matters most in the launch copy:

- It is local-first.
- It is useful before it is cute.
- It is for people juggling multiple agents and multiple repos.
- It does not require hooks, API keys, or another hosted dashboard.

## Maker Comment

Built `peep` because raw AI agent logs are technically available but practically unreadable when several agents are working at once. I wanted a terminal-native way to see what the lead agent is doing, where sub-agents are spending time, and which repo is actually moving without leaving the shell.

If you use Claude Code, Codex, or Gemini heavily, I would love feedback on the parts that still feel noisy, the tools you want detected next, and the moments where a terminal dashboard becomes genuinely useful instead of decorative.

## First Comment

Hey Product Hunt, I'm the maker of `peep`.

`peep` is a terminal dashboard for watching AI coding agents work in real time. It reads the local session logs from tools like Claude Code, Codex, and Gemini, then turns them into a live timeline with project tabs, sub-agent focus mode, and pixel-art party members that reflect each agent's state.

I built it because once you have more than one agent running, the logs stop being a good interface. You can technically inspect everything, but it becomes hard to answer simple questions like:

- Which agent is actually active right now?
- Which project is moving?
- Is a sub-agent done, blocked, or waiting on me?
- Did the context window just spike?

`peep` is intentionally local-first and low-friction:

- zero-config JSONL watching
- no API keys
- no cloud dashboard
- quick demo mode with `peep --mock`

If you try it, the most useful feedback would be:

- which tools or log formats should be added next
- which timeline details are still missing
- where the terminal UI gets noisy under real workloads

## X / Community Posts

### Post 1

Shipping `peep` today on Product Hunt.

It turns local Claude Code / Codex / Gemini session logs into a live terminal dashboard with sub-agent focus, project tabs, and a running timeline.

Zero config. No API keys. No cloud sync.

`peep --mock` gives you a safe demo in seconds.

### Post 2

If you're running multiple AI coding agents, the logs are there but the visibility is bad.

`peep` is my attempt at fixing that with a terminal-native dashboard:

- live party view
- sub-agent focus
- multi-project tabs
- local log watching

Launch page: `https://github.com/jsleemaster/peep`

## FAQ Reply Templates

### What tools does it support today?

Today `peep` auto-detects local session logs from Claude Code, Codex CLI, and Gemini CLI. OpenCode is planned, but I am not marketing it as full support yet.

### Does it send my prompts or code anywhere?

No. The default workflow is local log watching only. `peep` reads the session files already written on your machine and renders them in the terminal.

### Do I need to change my AI tool setup first?

No for the default path. Install `peep` and run it normally. Optional HTTP hooks exist for lower-latency events, but they are not required.

### Can I try it without real logs?

Yes. `peep --mock` starts a synthetic demo so you can see the interface without pointing it at personal project logs.

### What should happen after launch?

The next priorities are more tool coverage, better visibility for larger agent swarms, and tighter signal around blocking states and context pressure.
