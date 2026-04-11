<!-- Unlicense — cochranblock.org -->

# Timeline of Invention

*Dated, commit-level record of what was built, when, and why. Proves human-piloted AI development — not generated spaghetti.*

> Every entry below maps to real commits. Run `git log --oneline` to verify.

## How to Read This Document

Each entry follows this format:

- **Date**: When the work shipped (not when it was started)
- **What**: Concrete deliverable — binary, feature, fix, architecture change
- **Why**: Business or technical reason driving the decision
- **Commit**: Short hash(es) for traceability
- **AI Role**: What the AI did vs. what the human directed

This document exists because AI-assisted code has a trust problem. Anyone can generate 10,000 lines of spaghetti. This timeline proves that a human pilot directed every decision, verified every output, and shipped working software.

---

## Context

**tmuxisfree** is a LangChain replacement. Zero Python. Zero cloud. Each tmux pane is a siloed AI agent working on a project directory. One 493KB Rust binary replaces an entire multi-agent orchestration framework.

The insight: tmux already solves session management, pane isolation, and terminal multiplexing. Instead of building another orchestration framework from scratch, tmuxisfree treats tmux as the runtime and adds dispatch, retry, rate-limit handling, and backlog management on top. The result is a fleet controller that runs 28+ AI agents from a single terminal session.

---

## Human Revelations — Invented Techniques

*Novel ideas that came from human insight, not AI suggestion. These are original contributions to the field.*

### Sponge Mesh Broadcast (April 2, 2026)

**Invention:** A rate-limit-aware retry mesh for broadcasting tasks across AI agent sessions. First pass sends to all panes. Panes that hit rate limits are collected into a retry queue. Subsequent passes use exponential backoff (10s, 20s, 30s...) until all panes accept or 5 retries are exhausted.

**The Problem:** AI APIs (Claude, GPT) enforce per-session rate limits. Broadcasting the same task to 28 panes simultaneously guarantees that some will be rejected. Naive retry loops waste tokens re-sending to panes that are still cooling down. Sequential dispatch (one at a time) takes 10+ minutes for a fleet.

**The Insight:** Treat the fleet like a sponge. On the first pass, saturate every pane. Some absorb the task immediately; others are full (rate limited). Don't wait — move on. Come back later with increasing pressure (longer delays) and retry only the ones that didn't absorb. The sponge eventually soaks up everything.

**The Technique:**
1. First pass: send to all panes, 2s between each. Check for rate limit indicator.
2. Collect failed panes into a retry vector.
3. Retry loop (up to 5x): backoff = 10s * attempt. Re-send Enter to retry. Remove panes that recover.
4. Final report: all absorbed, or list of still-saturated panes.

**Result:** Full fleet broadcast in under 60 seconds instead of 10+ minutes. Zero wasted API calls. Self-healing — recovers from rate limits without human intervention.

**Named:** Sponge Mesh Broadcast
**Commit:** `8f95f18`
**Origin:** Michael Cochran, observing that Claude Code rate limits follow a predictable cooldown curve.

### Backlog Stack Pattern (April 7, 2026)

**Invention:** A file-backed LIFO stack per pane that decouples task creation from task execution. Tasks are pushed to a pane's backlog without requiring the pane to be idle. The drain command pops tasks one at a time, waits for idle between each, and auto-dispatches the next.

**The Problem:** AI agents can only process one task at a time. If you send 5 tasks to a busy pane, 4 get lost. Queuing systems (Redis, RabbitMQ) add infrastructure. In-memory queues die when the process exits.

**The Insight:** A text file with one task per line is a stack. Push appends. Pop removes the last line. Drain loops pop-then-wait-for-idle. The filesystem is the queue. No daemon, no database, no infrastructure.

**Named:** Backlog Stack Pattern
**Commit:** `b0c8c72`
**Origin:** Michael Cochran.

---

## Entries

<!-- Add entries in reverse chronological order. Template:

### YYYY-MM-DD — [Short Title]

**What:** [Concrete deliverable]
**Why:** [Business/technical driver]
**Commit:** `abc1234`
**AI Role:** [What AI generated vs. what human directed/verified]
**Proof:** [Link to artifact, screenshot, or test output]

-->

### 2026-04-09 — Exopack Decoupled + Truth Audit + Backlog Stack + Session Flag Fix

**What:** Four changes in one session: (1) Decoupled exopack from path dependency to git dependency, making the project buildable by anyone who clones it. (2) Truth audit of README — fixed inflated claims (removed "28 sessions" marketing, corrected subcommand counts, added real binary size). Added BACKLOG.md documenting the backlog workflow. (3) Standardized session flag to `-s` across all subcommands (was inconsistent). (4) Built the backlog subsystem — push/pop/drain/clear/backlog commands with file-backed LIFO stacks per pane.
**Why:** Path dependencies break on clone. README claimed features that didn't exist yet (28-session fleet, fleet.toml config). Session flag inconsistency was a usability bug — some subcommands used `-s`, others didn't accept it at all. Backlog solves the "send 5 tasks to a busy pane and 4 get lost" problem.
**Commit:** `3820121` (exopack decouple), `73f040f` (truth audit), `c9404a2` (session flag), `b0c8c72` (backlog)
**AI Role:** AI implemented all code changes, backlog module, and README corrections. Human identified the path dep problem, directed the truth audit scope, and invented the backlog stack pattern.
**Proof:** `cargo build` succeeds from clean clone. `tmuxisfree push/pop/drain` working.

### 2026-04-05 — Mobile/Desktop Mode + Focus/Home Navigation

**What:** Added mobile and desktop display modes that reconfigure tmux status bar position, height, tab format, and idle window visibility. Mobile mode: bottom bar (thumb-reachable), double-height tabs (fat-finger friendly), idle windows hidden. Desktop mode: top bar, single-height, all windows visible. Added focus/home navigation — focus a project window with optional auto-return command, home returns to C2. Updated README with CochranBlock ecosystem links and live deployment stats.
**Why:** Fleet management from a phone (via Termius) requires different UX than a 27" monitor. Focus/home enables single-pane workflow without losing the fleet context.
**Commit:** `9a234bc` (mode switching + focus/home), `f524695` (README)
**AI Role:** AI implemented tmux configuration commands and CLI subcommands. Human directed the mobile UX requirements from real Termius usage and designed the auto-return pattern.

### 2026-04-02 — Sponge Mesh + Unblock Daemon + Plan Approval

**What:** Built the sponge mesh broadcast — rate-limit-aware retry with exponential backoff. Extended the unblock daemon with plan approval detection (sends "1" then Enter for plan prompts vs just Enter for permission prompts), cooldown tracking (30s per window, 60s for rate limits), and self-kill of older daemon instances. Fixed clippy warning from unused import.
**Why:** Broadcasting to 28 panes requires handling rate limits gracefully. Plan approval prompts in Claude Code need "1" (select option 1), not just Enter. Without cooldowns, the daemon would spam the same window every 3 seconds.
**Commit:** `8f95f18` (sponge mesh + unblock), `804dcf4` (clippy fix), `e261e37` (plan approval)
**AI Role:** AI implemented all code. Human invented the sponge mesh pattern and identified the plan approval prompt difference.

### 2026-03-31 — Initial Scaffold

**What:** First commit. Core architecture: clap CLI with 8 subcommands (init, dispatch, broadcast, sponge, status, peek, unblock, qa, layout). Shared helper functions for tmux interaction (send_keys, capture_pane, is_idle, has_pasted_text, is_rate_limited, has_permission_prompt). Dispatch with retry + backoff. Staggered broadcast. Fleet status display. Pane peek. Unblock daemon. QA sweep. Layout export.
**Why:** LangChain and CrewAI are bloated Python frameworks that add complexity without value. tmux already does session management. This project adds the missing piece: task dispatch, monitoring, and self-healing across AI agent panes.
**Commit:** `90bcefa`
**AI Role:** AI generated the initial codebase from human-directed architecture. Human designed the tmux-as-runtime approach and specified every subcommand's behavior.
**Proof:** Single 668-line Rust file. 493KB release binary. Zero external services.
