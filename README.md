# tmuxisfree

**LangChain is dead. tmux is free.**

AI fleet orchestration via tmux + Claude Code. Each pane is a siloed AI agent working on a project directory. Zero Python. Zero cloud. Zero dependencies beyond tmux and claude.

## What it does

You have 15 projects. You open 15 tmux panes. Each pane runs `claude` in a project directory. tmuxisfree dispatches tasks, monitors progress, retries on rate limits, approves permission prompts, and reports status. That's it.

No LangChain. No Python. No vector databases. No prompt chains. No API wrappers. Just tmux + claude + a 300 KB Rust binary.

## Install

```
cargo install tmuxisfree
```

## Usage

```bash
# Initialize a fleet
tmuxisfree init --config fleet.toml

# Dispatch a task to one pane
tmuxisfree dispatch pixel-forge "fix the build error"

# Broadcast to all panes
tmuxisfree broadcast "update docs and push"

# Sponge mesh — handles rate limits automatically
tmuxisfree sponge "run cargo audit and fix vulnerabilities"

# Check fleet status
tmuxisfree status

# Peek at a pane
tmuxisfree peek pixel-forge

# Start the unblock daemon (auto-approves prompts)
tmuxisfree unblock

# QA sweep — build + clippy + test all panes
tmuxisfree qa

# Export fleet layout
tmuxisfree layout
```

## Fleet Config (fleet.toml)

```toml
session = "c2"

[[pane]]
name = "pixel-forge"
dir = "~/pixel-forge"

[[pane]]
name = "cochranblock"
dir = "~/cochranblock"

[[pane]]
name = "kova"
dir = "~/kova"
```

## Architecture

- **Dispatch**: Send task → retry on rate limit → flush pasted text → approve permissions → verify accepted
- **Broadcast**: Staggered dispatch to all panes (default 5s between each)
- **Sponge mesh**: First pass hits everyone, rate-limited panes get retried with exponential backoff
- **Unblock daemon**: Polls every 3s, auto-approves permission prompts + flushes stuck text
- **QA sweep**: Broadcasts compile + clippy + test to all panes

## Why not LangChain?

| | LangChain | tmuxisfree |
|---|-----------|------------|
| Language | Python | Rust |
| Size | 50+ MB with deps | 300 KB |
| Agents | API wrappers | Full Claude Code instances |
| Tool access | Limited | Everything (filesystem, git, ssh, cargo) |
| Isolation | Shared process | Separate tmux panes |
| Cost | API tokens per call | Claude subscription (flat rate) |
| Setup | pip install, .env, chains, prompts | tmux + claude |

## License

Unlicense — public domain. cochranblock.org
