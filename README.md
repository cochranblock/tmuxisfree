# tmuxisfree

**LangChain is dead. tmux is free.**

AI fleet orchestration via tmux + Claude Code. Each pane is a siloed AI agent working on a project directory. Zero Python. Zero cloud. Zero dependencies beyond tmux and claude.

Part of the [CochranBlock](https://cochranblock.org) ecosystem — veteran-owned, zero-cloud infrastructure.

## What it does

You have 15 projects. You open 15 tmux panes. Each pane runs `claude` in a project directory. tmuxisfree dispatches tasks, monitors progress, retries on rate limits, approves permission prompts, and reports status. That's it.

No LangChain. No Python. No vector databases. No prompt chains. No API wrappers. Just tmux + claude + a ~500 KB Rust binary.

## Install

```bash
# From source (not yet on crates.io)
git clone https://github.com/cochranblock/tmuxisfree
cd tmuxisfree
cargo install --path .
```

## Usage

```bash
# Dispatch a task to one pane (retry + backoff)
tmuxisfree dispatch pixel-forge "fix the build error"

# Broadcast to all panes (staggered)
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

# Mobile/desktop mode switching
tmuxisfree mobile    # compact bottom bar, hide idle
tmuxisfree desktop   # full top bar, all visible

# Focus a window (auto-return to C2 when done)
tmuxisfree focus cochranblock -c "cargo build"
tmuxisfree home      # return to C2

# Task backlog (queue work for later)
tmuxisfree push cochranblock "fix nav CSS"
tmuxisfree push cochranblock "add dark mode"
tmuxisfree backlog             # show all backlogs
tmuxisfree pop cochranblock    # pop top task and dispatch
tmuxisfree drain cochranblock  # auto-dispatch all, wait for idle between
tmuxisfree clear cochranblock  # clear backlog
```

## Fleet Config

Define your fleet in `fleet.toml`:

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

Then spin it up:

```bash
tmuxisfree init -c fleet.toml
```

Creates the tmux session, one window per pane, `cd`s to each dir, and starts `claude` in every pane (2s stagger to avoid rate limits). Skips any pane whose directory doesn't exist.

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
| Size | 50+ MB with deps | ~500 KB |
| Agents | API wrappers | Full Claude Code instances |
| Tool access | Limited | Everything (filesystem, git, ssh, cargo) |
| Isolation | Shared process | Separate tmux panes |
| Cost | API tokens per call | Claude subscription (flat rate) |
| Setup | pip install, .env, chains, prompts | tmux + claude |

## Part of the CochranBlock ecosystem

tmuxisfree is one piece of a zero-cloud stack built entirely in Rust:

- **[kova](https://github.com/cochranblock/kova)** — Augment engine. Local AI, distributed C2, tokenized commands.
- **[approuter](https://github.com/cochranblock/approuter)** — Reverse proxy + Cloudflare tunnel manager. One binary replaces nginx + certbot + cloudflared.
- **[pixel-forge](https://github.com/cochranblock/pixel-forge)** — AI pixel art generator. Deterministic sprites from text prompts.
- **[cochranblock](https://github.com/cochranblock/cochranblock)** — The company site. 15MB binary serves everything. [$10/month total infrastructure](https://cochranblock.org/openbooks).

**Need a fractional CTO who ships?** [cochranblock.org/deploy](https://cochranblock.org/deploy)

## Live stats

- [Velocity dashboard](https://cochranblock.org/codeskillz) — commit activity across all repos
- [Traffic analytics](https://cochranblock.org/barz) — live Cloudflare + git data
- [Open books](https://cochranblock.org/openbooks) — real costs, public

## License

Unlicense — public domain. [cochranblock.org](https://cochranblock.org)
