// Unlicense — cochranblock.org
// Contributors: GotEmCoach, KOVA, Claude Opus 4.6
//! tmuxisfree — AI orchestration via tmux + Claude Code.
//! LangChain replacement. Zero Python. Zero cloud.
//! Each pane = one AI agent siloed to a project directory.

use clap::Parser;
use std::process::Command;

const DEFAULT_SESSION: &str = "c2";

#[derive(Parser)]
#[command(name = "tmuxisfree", about = "AI fleet orchestration via tmux. LangChain is dead.")]
enum Cmd {
    /// Initialize a new fleet session with project panes
    Init {
        /// Session name
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
        /// Config file listing projects (one per line: name:/path/to/dir)
        #[arg(short, long, default_value = "fleet.toml")]
        config: String,
    },
    /// Dispatch a task to a single pane (with retry + backoff)
    Dispatch {
        /// Window name or index
        window: String,
        /// The task message
        message: String,
        /// Session name
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Broadcast a task to all panes (staggered)
    Broadcast {
        /// The task message
        message: String,
        /// Seconds between each pane
        #[arg(short, long, default_value = "5")]
        stagger: u64,
        /// Session name
        #[arg(short = 'S', long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Sponge mesh broadcast — skip rate-limited panes, retry with backoff
    Sponge {
        message: String,
        #[arg(short = 'S', long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Show fleet status — which panes are working, idle, or stuck
    Status {
        #[arg(short = 'S', long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Peek at a pane's recent output
    Peek {
        window: String,
        #[arg(short, long, default_value = "20")]
        lines: usize,
        #[arg(short = 'S', long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Start the unblock daemon — auto-approves permission prompts + flushes pasted text
    Unblock {
        #[arg(short = 'S', long, default_value = DEFAULT_SESSION)]
        session: String,
        /// Poll interval in seconds
        #[arg(short, long, default_value = "3")]
        interval: u64,
    },
    /// QA sweep — send compile + test + clippy to all panes
    Qa {
        #[arg(short = 'S', long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Export fleet layout as markdown
    Layout {
        #[arg(short = 'S', long, default_value = DEFAULT_SESSION)]
        session: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cmd = Cmd::parse();
    match cmd {
        Cmd::Init { session, config } => init::f0(&session, &config),
        Cmd::Dispatch { window, message, session } => dispatch::f10(&session, &window, &message),
        Cmd::Broadcast { message, stagger, session } => broadcast::f20(&session, &message, stagger),
        Cmd::Sponge { message, session } => sponge::f30(&session, &message),
        Cmd::Status { session } => status::f40(&session),
        Cmd::Peek { window, lines, session } => peek::f50(&session, &window, lines),
        Cmd::Unblock { session, interval } => unblock::f60(&session, interval),
        Cmd::Qa { session } => qa::f70(&session),
        Cmd::Layout { session } => layout::f80(&session),
    }
}

/// f1: Run a tmux command, return stdout
fn tmux(args: &[&str]) -> anyhow::Result<String> {
    let out = Command::new("tmux").args(args).output()?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// f2: Send keys to a pane
fn send_keys(session: &str, window: &str, msg: &str) -> anyhow::Result<()> {
    Command::new("tmux")
        .args(["send-keys", "-t", &format!("{}:{}", session, window), msg, "Enter"])
        .status()?;
    Ok(())
}

/// f3: Capture pane output
fn capture_pane(session: &str, window: &str, lines: usize) -> anyhow::Result<String> {
    let out = Command::new("tmux")
        .args(["capture-pane", "-t", &format!("{}:{}", session, window), "-p", "-S", &format!("-{}", lines)])
        .output()?;
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// f4: Check if pane is idle (at prompt)
fn is_idle(session: &str, window: &str) -> bool {
    capture_pane(session, window, 6)
        .map(|s| s.lines().rev().take(6).any(|l| l.contains("❯")))
        .unwrap_or(false)
}

/// f5: Check if pane has stuck pasted text
fn has_pasted_text(session: &str, window: &str) -> bool {
    capture_pane(session, window, 10)
        .map(|s| s.contains("Pasted text"))
        .unwrap_or(false)
}

/// f6: Check if pane hit rate limit
fn is_rate_limited(session: &str, window: &str) -> bool {
    capture_pane(session, window, 10)
        .map(|s| s.contains("Rate limit"))
        .unwrap_or(false)
}

/// f7: Check if pane has permission prompt
fn has_permission_prompt(session: &str, window: &str) -> bool {
    capture_pane(session, window, 10)
        .map(|s| s.contains("Do you want to proceed") || s.contains("Yes, and don't ask"))
        .unwrap_or(false)
}

/// f8: Check if pane has plan approval prompt (needs '1' not just Enter)
fn has_plan_prompt(session: &str, window: &str) -> bool {
    capture_pane(session, window, 10)
        .map(|s| {
            s.contains("Would you like to proceed")
                || s.contains("Yes, auto-accept edits")
                || s.contains("Yes, and don")
        })
        .unwrap_or(false)
}

mod init {
    use super::*;
    /// f0: Initialize fleet from config
    pub fn f0(_session: &str, _config: &str) -> anyhow::Result<()> {
        println!("init: not yet implemented");
        Ok(())
    }
}

mod dispatch {
    use super::*;
    /// f10: Dispatch with retry + backoff
    pub fn f10(session: &str, window: &str, message: &str) -> anyhow::Result<()> {
        let max_retries = 10;
        let base_delay = std::time::Duration::from_secs(3);

        for attempt in 1..=max_retries {
            send_keys(session, window, message)?;
            std::thread::sleep(base_delay);

            if has_pasted_text(session, window) {
                send_keys(session, window, "")?;
                std::thread::sleep(std::time::Duration::from_secs(2));
            }

            if is_rate_limited(session, window) {
                let backoff = base_delay * attempt;
                eprintln!("[w{}] rate limited — backing off {:?} (attempt {}/{})", window, backoff, attempt, max_retries);
                std::thread::sleep(backoff);
                send_keys(session, window, "")?;
                continue;
            }

            if has_permission_prompt(session, window) {
                send_keys(session, window, "")?;
                continue;
            }

            let pane = capture_pane(session, window, 8)?;
            if pane.contains("✻") || pane.contains("✶") || pane.contains("✽") || pane.contains("Bash") || pane.contains("Edit") || pane.contains("Read") {
                eprintln!("[w{}] accepted on attempt {}", window, attempt);
                return Ok(());
            }

            if is_idle(session, window) {
                send_keys(session, window, "")?;
                std::thread::sleep(base_delay);
            }
        }

        eprintln!("[w{}] sent after {} attempts", window, max_retries);
        Ok(())
    }
}

mod broadcast {
    use super::*;
    /// f20: Staggered broadcast
    pub fn f20(session: &str, message: &str, stagger: u64) -> anyhow::Result<()> {
        let windows = tmux(&["list-windows", "-t", session, "-F", "#{window_index}:#{window_name}"])?;
        for line in windows.lines() {
            let idx = line.split(':').next().unwrap_or("0");
            if idx == "0" { continue; } // skip dispatcher
            if line.contains("unblock") { continue; }
            dispatch::f10(session, idx, message)?;
            std::thread::sleep(std::time::Duration::from_secs(stagger));
        }
        eprintln!("broadcast complete");
        Ok(())
    }
}

mod sponge {
    use super::*;
    /// f30: Sponge mesh — skip rate limited, retry later
    pub fn f30(session: &str, message: &str) -> anyhow::Result<()> {
        let windows = tmux(&["list-windows", "-t", session, "-F", "#{window_index}:#{window_name}"])?;
        let mut failed: Vec<String> = Vec::new();

        // First pass
        for line in windows.lines() {
            let idx = line.split(':').next().unwrap_or("0").to_string();
            if idx == "0" || line.contains("unblock") { continue; }
            send_keys(session, &idx, message)?;
            std::thread::sleep(std::time::Duration::from_secs(2));
            if is_rate_limited(session, &idx) {
                eprintln!("[w{}] rate limited — will retry", idx);
                failed.push(idx);
            } else {
                if has_pasted_text(session, &idx) {
                    send_keys(session, &idx, "")?;
                }
                eprintln!("[w{}] sent", idx);
            }
        }

        // Retry pass
        for retry in 1..=5 {
            if failed.is_empty() { break; }
            let backoff = std::time::Duration::from_secs(10 * retry);
            eprintln!("retrying {} failed panes in {:?}...", failed.len(), backoff);
            std::thread::sleep(backoff);

            failed.retain(|idx| {
                send_keys(session, idx, "").ok();
                std::thread::sleep(std::time::Duration::from_secs(3));
                if is_rate_limited(session, idx) {
                    true
                } else {
                    eprintln!("[w{}] recovered on retry {}", idx, retry);
                    false
                }
            });
        }

        if failed.is_empty() {
            eprintln!("sponge broadcast complete — all panes accepted");
        } else {
            eprintln!("WARNING: {} panes still rate limited: {:?}", failed.len(), failed);
        }
        Ok(())
    }
}

mod status {
    use super::*;
    /// f40: Fleet status
    pub fn f40(session: &str) -> anyhow::Result<()> {
        let windows = tmux(&["list-windows", "-t", session, "-F", "#{window_index}:#{window_name}"])?;
        println!("{:<5} {:<22} {:<6}", "WIN", "NAME", "STATE");
        println!("{}", "-".repeat(35));
        for line in windows.lines() {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() < 2 { continue; }
            let (idx, name) = (parts[0], parts[1]);
            let state = if is_idle(session, idx) { "IDLE" } else { "WORK" };
            println!("{:<5} {:<22} {:<6}", idx, name, state);
        }
        Ok(())
    }
}

mod peek {
    use super::*;
    /// f50: Peek at pane
    pub fn f50(session: &str, window: &str, lines: usize) -> anyhow::Result<()> {
        let content = capture_pane(session, window, lines)?;
        print!("{}", content);
        Ok(())
    }
}

mod unblock {
    use super::*;
    /// f60: Unblock daemon
    pub fn f60(session: &str, interval: u64) -> anyhow::Result<()> {
        eprintln!("unblock daemon running ({}s interval)", interval);
        loop {
            let windows = tmux(&["list-windows", "-t", session, "-F", "#{window_index}"])?;
            for idx in windows.lines() {
                if idx == "0" { continue; }
                if has_plan_prompt(session, idx) {
                    send_keys(session, idx, "1")?;
                    eprintln!("[w{}] approved plan prompt", idx);
                } else if has_permission_prompt(session, idx) {
                    send_keys(session, idx, "")?;
                    eprintln!("[w{}] unblocked permission", idx);
                }
                if has_pasted_text(session, idx) {
                    send_keys(session, idx, "")?;
                    eprintln!("[w{}] flushed pasted text", idx);
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(interval));
        }
    }
}

mod qa {
    use super::*;
    /// f70: QA sweep
    pub fn f70(session: &str, ) -> anyhow::Result<()> {
        broadcast::f20(session, "QA: cargo build --release && cargo clippy -- -D warnings && git status. Report PASS or FAIL.", 5)
    }
}

mod layout {
    use super::*;
    /// f80: Export layout as markdown
    pub fn f80(session: &str) -> anyhow::Result<()> {
        let windows = tmux(&["list-windows", "-t", session, "-F", "#{window_index}|#{window_name}|#{pane_current_path}"])?;
        println!("| Window | Name | Directory |");
        println!("|--------|------|-----------|");
        for line in windows.lines() {
            let parts: Vec<&str> = line.splitn(3, '|').collect();
            if parts.len() >= 3 {
                println!("| {} | {} | {} |", parts[0], parts[1], parts[2]);
            }
        }
        Ok(())
    }
}
