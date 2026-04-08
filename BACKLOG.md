# tmuxisfree Backlog

Prioritized. Top = most important.

1. **Implement `init` command** — parse fleet.toml config, create tmux session + windows, start claude in each pane. Currently a stub.
2. **Publish to crates.io** — `cargo install tmuxisfree` doesn't work yet. Needs metadata review, test that package builds clean from registry.
3. **Config file parsing** — fleet.toml format is designed but no parser exists. Needed for init.
4. **Pane detection hardening** — idle/rate-limit/permission detection relies on string matching (`❯`, `Rate limit`, `Pasted text`, `Do you want to proceed`). Fragile if Claude Code changes prompts.
5. **Error propagation in broadcast** — broadcast/sponge skip dispatcher (window 0) and unblock windows by name, but don't validate that target windows exist.
6. **Backlog persistence** — backlog uses plain text `.stack` files in `~/.tmuxisfree/backlog/`. Works, but no locking — concurrent push/pop could race.
7. **Tests** — no test suite yet. Binary compiles, commands are manually tested against live tmux sessions.
8. **Session flag docs** — README usage examples don't show `-s` flag for non-default sessions.
