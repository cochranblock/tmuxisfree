<!-- Unlicense — cochranblock.org -->

# Proof of Artifacts

*Visual and structural evidence that this project works, ships, and is real.*

> This is not a demo repo. This is production software running a fleet of AI agents from a single terminal.

## Architecture

```
┌─────────────────────────────────────────────────┐
│  tmux session "c2"                              │
│                                                 │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐        │
│  │ window 0 │  │ window 1 │  │ window 2 │  ...  │
│  │ C2 (hub) │  │ kova     │  │ cochran  │        │
│  └─────────┘  └─────────┘  └─────────┘        │
│       │            │             │               │
│       └──── tmuxisfree binary ───┘               │
│             dispatch / peek / unblock            │
│             backlog / sponge / qa                │
└─────────────────────────────────────────────────┘
       │
  ~/.tmuxisfree/backlog/*.stack  (file-backed LIFO)
```

## Build Output

| Metric | Value |
|--------|-------|
| Binary size (release) | 493KB (opt-level='z', LTO, strip, panic=abort) |
| Lines of code | 668 (single file) |
| Functions | 31 |
| Types/Enums | 1 (Cmd enum with 15 variants) |
| Subcommands | 15 (init, dispatch, broadcast, sponge, status, peek, unblock, qa, layout, mobile, desktop, focus, home, push, pop, backlog, clear, drain) |
| Direct dependencies | 5 (clap, serde, serde_json, anyhow, dirs) |
| Optional dependencies | 1 (exopack — test gate) |
| Infrastructure cost | $0 |
| External services | Zero |
| Database | Zero — file-backed stacks only |
| Cloud dependencies | Zero |
| Language | Rust 2024 edition |
| Total commits | 10 |
| Release profile | opt-level='z', lto=true, codegen-units=1, panic='abort', strip=true |
| Tests | Via exopack triple_sims gate (optional feature) |
| QA | `tmuxisfree qa` broadcasts build+clippy+status to all panes |

## Named Inventions & Techniques

| Name | Type | Description |
|------|------|-------------|
| Sponge Mesh Broadcast | Invention | Rate-limit-aware retry mesh — first pass saturates, retries with exponential backoff, self-healing fleet broadcast |
| Backlog Stack Pattern | Invention | File-backed LIFO per pane — push/pop/drain decouples task creation from execution, filesystem is the queue |
| Unblock Daemon | Technique | Auto-approves permission prompts, plan prompts (sends "1" then Enter), flushes pasted text, handles rate limits — with per-window cooldowns |
| Dispatch with Retry | Technique | 10-attempt dispatch with 3s base delay, exponential backoff on rate limit, pasted text flush, permission auto-approve |
| Mobile/Desktop Mode | Technique | Runtime tmux reconfiguration — bottom bar + hidden idle windows (mobile) vs top bar + all visible (desktop) |
| Focus/Home Navigation | Technique | Single-pane focus with optional auto-return command, C2 hub pattern |
| Fleet QA Sweep | Technique | Broadcast build+clippy+status to all panes, collect pass/fail |

## Comparisons

| Feature | tmuxisfree | LangChain | CrewAI |
|---------|-----------|-----------|--------|
| Binary size | 493KB | ~200MB (Python + deps) | ~150MB |
| Language | Rust | Python | Python |
| Infrastructure | tmux (pre-installed) | Docker, Redis, etc. | Docker, etc. |
| Agent isolation | tmux panes (OS-level) | Python threads | Python threads |
| Rate limit handling | Sponge mesh + backoff | Manual | Manual |
| Task queue | File-backed stacks | Redis/Celery | In-memory |
| Cloud dependency | Zero | AWS/GCP/Azure | AWS/GCP/Azure |
| Setup time | `cargo install` | pip install + config + infra | pip install + config |

## How to Verify

```bash
# Clone, build, run. That's it.
git clone https://github.com/cochranblock/tmuxisfree
cd tmuxisfree
cargo build --release
ls -lh target/release/tmuxisfree   # 493KB
cp target/release/tmuxisfree ~/bin/

# Fleet operations
tmuxisfree status -s c2            # see what's running
tmuxisfree dispatch kova "cargo check"  # send task to a pane
tmuxisfree broadcast "git pull"    # broadcast to all panes
tmuxisfree push kova "fix the bug" # queue a task
tmuxisfree drain kova              # auto-dispatch backlog
```

---

*Part of the [CochranBlock](https://cochranblock.org) zero-cloud architecture. All source under the Unlicense.*
