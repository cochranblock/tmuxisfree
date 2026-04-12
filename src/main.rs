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
        #[arg(short = 'g', long, default_value = "5")]
        stagger: u64,
        /// Session name
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Sponge mesh broadcast — skip rate-limited panes, retry with backoff
    Sponge {
        message: String,
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Show fleet status — which panes are working, idle, or stuck
    Status {
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Peek at a pane's recent output
    Peek {
        window: String,
        #[arg(short, long, default_value = "20")]
        lines: usize,
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Start the unblock daemon — auto-approves permission prompts + flushes pasted text
    Unblock {
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
        /// Poll interval in seconds
        #[arg(short = 'i', long, default_value = "3")]
        interval: u64,
    },
    /// QA sweep — send compile + test + clippy to all panes
    Qa {
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Export fleet layout as markdown
    Layout {
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Switch to mobile mode — compact status, bottom bar, hide idle windows
    Mobile {
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Switch to desktop mode — full status bar, top position
    Desktop {
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Focus a project window, hide all others. Auto-return to C2 when command finishes.
    Focus {
        /// Window name or index
        window: String,
        /// Optional command to run (auto-returns to C2 on completion)
        #[arg(short, long)]
        cmd: Option<String>,
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Return to C2 and minimize all project windows
    Home {
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Push a task onto a pane's backlog stack
    Push {
        /// Window name
        window: String,
        /// Task description
        task: String,
    },
    /// Pop the top task from a pane's backlog and dispatch it
    Pop {
        /// Window name
        window: String,
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
    },
    /// Show a pane's backlog stack (top = next)
    Backlog {
        /// Window name (omit for all panes)
        window: Option<String>,
    },
    /// Clear a pane's backlog
    Clear {
        /// Window name
        window: String,
    },
    /// Drain: pop and dispatch all tasks from a pane's backlog (staggered, waits for idle)
    Drain {
        /// Window name
        window: String,
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
        /// Seconds to wait between polling for idle
        #[arg(short = 'i', long, default_value = "10")]
        interval: u64,
    },
    /// Scaffold a fleet.toml by scanning for git repos in a directory
    Scaffold {
        /// Directory to scan for git projects
        #[arg(short, long, default_value = "~")]
        dir: String,
        /// Output file
        #[arg(short, long, default_value = "fleet.toml")]
        output: String,
        /// Session name to embed in config
        #[arg(short, long, default_value = DEFAULT_SESSION)]
        session: String,
        /// Max depth to scan
        #[arg(long, default_value = "1")]
        depth: usize,
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
        Cmd::Mobile { session } => mode::f90(&session),
        Cmd::Desktop { session } => mode::f91(&session),
        Cmd::Focus { window, cmd, session } => focus::f100(&session, &window, cmd.as_deref()),
        Cmd::Home { session } => focus::f101(&session),
        Cmd::Push { window, task } => backlog::f110(&window, &task),
        Cmd::Pop { window, session } => backlog::f111(&window, &session),
        Cmd::Backlog { window } => backlog::f112(window.as_deref()),
        Cmd::Clear { window } => backlog::f113(&window),
        Cmd::Drain { window, session, interval } => backlog::f114(&window, &session, interval),
        Cmd::Scaffold { dir, output, session, depth } => scaffold::f120(&dir, &output, &session, depth),
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

/// f4: Check if pane is idle (at shell prompt or Claude Code empty prompt)
fn is_idle(session: &str, window: &str) -> bool {
    capture_pane(session, window, 30)
        .map(|s| {
            // Strip trailing blank lines (Claude Code leaves empty terminal space below prompt)
            let non_empty: Vec<&str> = s.lines().rev().skip_while(|l| l.trim().is_empty()).collect();
            non_empty.iter().take(6).any(|l| l.contains("❯"))
        })
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

/// f7: Check if pane has any approval prompt (permission, plan, trust, allow)
fn has_approval_prompt(session: &str, window: &str) -> bool {
    capture_pane(session, window, 15)
        .map(|s| {
            s.contains("Do you want to proceed")
                || s.contains("Would you like to proceed")
                || s.contains("Yes, auto-accept")
                || s.contains("Yes, and don")
                || s.contains("Yes, allow")
                || s.contains("Trust this folder")
                || s.contains("Esc to cancel")
        })
        .unwrap_or(false)
}

/// f8: Detect which option to pick. Returns "1" for most prompts,
/// "2" for "allow from this project" prompts (broader permission).
fn pick_option(session: &str, window: &str) -> &'static str {
    let text = capture_pane(session, window, 15).unwrap_or_default();
    // If there's a "Yes, allow" option that grants project-wide access, pick that
    if text.contains("2. Yes, allow") || text.contains("2. Yes, and don") {
        "2"
    } else {
        "1"
    }
}

mod init {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct FleetConfig {
        session: Option<String>,
        pane: Vec<PaneConfig>,
    }

    #[derive(Deserialize)]
    struct PaneConfig {
        name: String,
        dir: String,
    }

    pub fn expand_tilde(path: &str) -> String {
        if let Some(rest) = path.strip_prefix("~/") {
            if let Some(home) = dirs::home_dir() {
                return format!("{}/{}", home.display(), rest);
            }
        }
        path.to_string()
    }

    /// f0: Initialize fleet from config — parse fleet.toml, create session + windows, start claude
    pub fn f0(session_arg: &str, config: &str) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(config)
            .map_err(|e| anyhow::anyhow!("cannot read {}: {}", config, e))?;

        let fleet: FleetConfig = toml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("parse {}: {}", config, e))?;

        let session = fleet.session.as_deref().unwrap_or(session_arg);

        // Bail if session already exists
        let check = Command::new("tmux")
            .args(["has-session", "-t", session])
            .stderr(std::process::Stdio::null())
            .status()?;
        if check.success() {
            anyhow::bail!("session '{}' exists — kill first: tmux kill-session -t {}", session, session);
        }

        // Create session with window 0 as C2
        Command::new("tmux")
            .args(["new-session", "-d", "-s", session, "-n", "C2"])
            .status()?;
        eprintln!("[init] session '{}'", session);

        let mut created = 0u16;
        for pane in &fleet.pane {
            let dir = expand_tilde(&pane.dir);

            if !std::path::Path::new(&dir).is_dir() {
                eprintln!("[init] SKIP {} — not found: {}", pane.name, dir);
                continue;
            }

            Command::new("tmux")
                .args(["new-window", "-t", session, "-n", &pane.name, "-c", &dir])
                .status()?;

            // Start claude in this pane
            send_keys(session, &pane.name, "claude")?;
            created += 1;
            eprintln!("[init] w{} {} → {}", created, pane.name, dir);

            // Stagger 2s to avoid rate limits on claude startup
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        // Return to C2
        let _ = Command::new("tmux")
            .args(["select-window", "-t", &format!("{}:0", session)])
            .status();

        eprintln!("[init] fleet ready — {} panes", created);
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

            if has_approval_prompt(session, window) {
                let option = pick_option(session, window);
                let target = format!("{}:{}", session, window);
                let _ = Command::new("tmux").args(["send-keys", "-t", &target, option]).status();
                std::thread::sleep(std::time::Duration::from_millis(300));
                let _ = Command::new("tmux").args(["send-keys", "-t", &target, "Enter"]).status();
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

    /// Jittered sleep: base_ms ± 50%
    fn jittered_sleep(base_ms: u64) {
        let jitter = (base_ms / 2) as i64;
        let offset = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos() as i64 % (jitter * 2 + 1)) - jitter;
        let ms = (base_ms as i64 + offset).max(500) as u64;
        std::thread::sleep(std::time::Duration::from_millis(ms));
    }

    /// f30: Sponge mesh — circuit breaker + exponential backoff + jitter
    pub fn f30(session: &str, message: &str) -> anyhow::Result<()> {
        let windows = tmux(&["list-windows", "-t", session, "-F", "#{window_index}:#{window_name}"])?;
        let panes: Vec<String> = windows.lines()
            .filter_map(|l| {
                let idx = l.split(':').next().unwrap_or("0").to_string();
                if idx == "0" || l.contains("unblock") { None } else { Some(idx) }
            })
            .collect();

        let mut pending = panes.clone();
        let mut sent: Vec<String> = Vec::new();
        let mut consecutive_fails = 0u32;

        // First pass — circuit breaker: 3 consecutive rate limits = account-wide, stop
        for idx in &panes {
            if consecutive_fails >= 3 {
                eprintln!("[circuit breaker] account-wide rate limit detected — pausing first pass");
                break;
            }
            // Only send to idle panes — don't dump text into active/rate-limited Claude
            if !is_idle(session, idx) && !is_rate_limited(session, idx) {
                eprintln!("[w{}] busy — skipping", idx);
                continue;
            }
            send_keys(session, idx, message)?;
            jittered_sleep(3000);

            if is_rate_limited(session, idx) {
                consecutive_fails += 1;
                eprintln!("[w{}] rate limited ({} consecutive)", idx, consecutive_fails);
            } else {
                if has_pasted_text(session, idx) {
                    send_keys(session, idx, "")?;
                }
                consecutive_fails = 0;
                sent.push(idx.clone());
                eprintln!("[w{}] accepted", idx);
            }
        }

        // Build pending list: everything not sent
        pending.retain(|idx| !sent.contains(idx));

        if pending.is_empty() {
            eprintln!("sponge complete — all {} panes accepted", sent.len());
            return Ok(());
        }

        eprintln!("{} sent, {} pending — entering retry with exponential backoff", sent.len(), pending.len());

        // Retry passes — exponential backoff (30s, 60s, 120s, 240s, 480s)
        for retry in 0..5u32 {
            if pending.is_empty() { break; }
            let base_ms = 30_000 * 2u64.pow(retry);
            eprintln!("retry {} — backoff ~{}s before trying {} panes...", retry + 1, base_ms / 1000, pending.len());
            jittered_sleep(base_ms);

            // Trickle: one pane at a time with stagger, stop on consecutive fails
            consecutive_fails = 0;
            let mut newly_sent = Vec::new();
            for idx in &pending {
                if consecutive_fails >= 2 {
                    eprintln!("[circuit breaker] still rate limited — aborting retry {}", retry + 1);
                    break;
                }
                // Check if rate limit cleared before sending
                if is_rate_limited(session, idx) {
                    consecutive_fails += 1;
                    continue;
                }
                if !is_idle(session, idx) {
                    eprintln!("[w{}] busy — skip", idx);
                    continue;
                }
                send_keys(session, idx, message)?;
                jittered_sleep(5000);

                if is_rate_limited(session, idx) {
                    consecutive_fails += 1;
                    eprintln!("[w{}] still limited", idx);
                } else {
                    if has_pasted_text(session, idx) {
                        send_keys(session, idx, "")?;
                    }
                    consecutive_fails = 0;
                    newly_sent.push(idx.clone());
                    eprintln!("[w{}] accepted on retry {}", idx, retry + 1);
                }
            }
            pending.retain(|idx| !newly_sent.contains(idx));
            sent.extend(newly_sent);
        }

        if pending.is_empty() {
            eprintln!("sponge complete — all {} panes accepted", sent.len());
        } else {
            eprintln!("sponge done — {}/{} accepted, {} still pending: {:?}",
                sent.len(), sent.len() + pending.len(), pending.len(), pending);
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
    /// f50: Peek at pane — strips Claude Code chrome and tmux noise
    pub fn f50(session: &str, window: &str, lines: usize) -> anyhow::Result<()> {
        let content = capture_pane(session, window, lines)?;
        let cleaned: Vec<&str> = content
            .lines()
            .filter(|l| !is_chrome(l))
            .collect();
        // Strip trailing blank lines
        let trimmed: Vec<&str> = cleaned.into_iter().rev()
            .skip_while(|l| l.trim().is_empty())
            .collect::<Vec<_>>().into_iter().rev().collect();
        for line in &trimmed {
            println!("{}", line);
        }
        // Detect stale chrome filter — repeated identical non-empty lines = unstripped noise
        let mut counts = std::collections::HashMap::new();
        for line in &trimmed {
            let t = line.trim();
            if !t.is_empty() {
                *counts.entry(t).or_insert(0u32) += 1;
            }
        }
        if let Some((line, n)) = counts.iter().find(|(_, n)| **n >= 3) {
            let preview: String = line.chars().take(40).collect();
            eprintln!("[peek] WARNING: chrome filter may be stale — \"{}\" repeated {}x", preview, n);
        }
        Ok(())
    }

    /// Lines that are Claude Code banner/chrome or tmux noise
    fn is_chrome(line: &str) -> bool {
        let t = line.trim();
        if t.is_empty() { return false; } // keep blanks between real content
        // Box-drawing separator lines (────...)
        if t.chars().all(|c| c == '─' || c == '━') { return true; }
        // Claude Code banner art
        if t.contains('▐') || t.contains('▛') || t.contains('▜') || t.contains('▝') || t.contains('▘') { return true; }
        // Claude Code version + model line
        if t.contains("Claude Code v") { return true; }
        if t.contains("context") && (t.contains("Opus") || t.contains("Sonnet") || t.contains("Haiku")) { return true; }
        // Hints
        if t == "? for shortcuts" { return true; }
        if t.starts_with("Tip:") { return true; }
        false
    }
}

mod unblock {
    use super::*;
    /// f60: Unblock daemon — self-kills older instances, tracks cooldowns
    pub fn f60(session: &str, interval: u64) -> anyhow::Result<()> {
        use std::collections::HashMap;
        use std::time::Instant;

        // Gemini Man: write PID to lockfile, kill old instance via lockfile, take over
        let lockfile = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join(".kova")
            .join("unblock.pid");
        let my_pid = std::process::id();

        // Kill old instance from lockfile (not pgrep — avoids matching ourselves)
        if let Ok(old_pid_str) = std::fs::read_to_string(&lockfile) {
            if let Ok(old_pid) = old_pid_str.trim().parse::<u32>() {
                if old_pid != my_pid {
                    let _ = Command::new("kill").arg(old_pid.to_string()).status();
                    eprintln!("[unblock] killed old instance pid={}", old_pid);
                    std::thread::sleep(std::time::Duration::from_millis(500));
                }
            }
        }

        // Write our PID
        if let Some(parent) = lockfile.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&lockfile, my_pid.to_string());
        eprintln!("unblock daemon running ({}s interval, pid={})", interval, my_pid);

        // Cooldown: don't re-approve same window within 30s
        let mut cooldowns: HashMap<String, Instant> = HashMap::new();
        let cooldown_secs = 30;

        loop {
            let windows = tmux(&["list-windows", "-t", session, "-F", "#{window_index}"])?;
            for idx in windows.lines() {
                if idx == "0" { continue; }
                let now = Instant::now();

                if has_approval_prompt(session, idx) {
                    if let Some(last) = cooldowns.get(idx) {
                        if now.duration_since(*last).as_secs() < cooldown_secs {
                            continue;
                        }
                    }
                    let option = pick_option(session, idx);
                    let target = format!("{}:{}", session, idx);
                    let _ = Command::new("tmux").args(["send-keys", "-t", &target, option]).status();
                    std::thread::sleep(std::time::Duration::from_millis(300));
                    let _ = Command::new("tmux").args(["send-keys", "-t", &target, "Enter"]).status();
                    cooldowns.insert(idx.to_string(), now);
                    eprintln!("[w{}] approved (option {}, cooldown {}s)", idx, option, cooldown_secs);
                }
                if is_rate_limited(session, idx) {
                    if let Some(last) = cooldowns.get(&format!("rl_{}", idx)) {
                        if now.duration_since(*last).as_secs() < 60 {
                            continue; // back off 60s on rate limits
                        }
                    }
                    // Hit Enter to retry the last message
                    send_keys(session, idx, "")?;
                    cooldowns.insert(format!("rl_{}", idx), now);
                    eprintln!("[w{}] rate limited — retry (cooldown 60s)", idx);
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

mod mode {
    use super::*;

    /// f90: Mobile mode — fat-finger friendly, compact, bottom bar
    pub fn f90(session: &str) -> anyhow::Result<()> {
        let cmds: &[&[&str]] = &[
            // Status bar to bottom (thumb reachable)
            &["set", "-t", session, "-g", "status-position", "bottom"],
            // Double-height status bar (bigger tap targets)
            &["set", "-t", session, "-g", "status", "2"],
            // Compact left — just emoji + short label
            &["set", "-t", session, "-g", "status-left", "#[fg=#00d4aa,bold] 🚀 K "],
            // Compact right
            &["set", "-t", session, "-g", "status-right", "#[fg=#d4a017] %H:%M "],
            // Wider tab format for tap targets
            &["set", "-t", session, "-g", "window-status-format", "#[fg=#6b2fa0,bg=#0a0a0a]  #W  "],
            &["set", "-t", session, "-g", "window-status-current-format", "#[fg=#0a0a0a,bg=#00d4aa,bold]  #W  "],
            // Hide idle windows — only show C2 + windows with activity
            &["set", "-t", session, "-g", "window-status-format",
              "#{?window_activity_flag,#[fg=#d4a017,bg=#0a0a0a]  #W  ,#{?#{==:#I,0},#[fg=#6b2fa0,bg=#0a0a0a]  #W  ,}}"],
            // Tag mobile mode
            &["set", "-t", session, "-g", "@mobile", "on"],
        ];
        for args in cmds {
            let _ = Command::new("tmux").args(*args).status();
        }
        eprintln!("mobile mode — bottom bar, compact tabs, idle windows hidden");
        Ok(())
    }

    /// f91: Desktop mode — full status bar, top position, all windows visible
    pub fn f91(session: &str) -> anyhow::Result<()> {
        let cmds: &[&[&str]] = &[
            &["set", "-t", session, "-g", "status-position", "top"],
            &["set", "-t", session, "-g", "status", "1"],
            &["set", "-t", session, "-g", "status-left", "#[fg=#00d4aa,bold] KOVA "],
            &["set", "-t", session, "-g", "status-right", "#[fg=#6b2fa0]⬥ #[fg=#d4a017]%H:%M"],
            &["set", "-t", session, "-g", "window-status-format", "#[fg=#6b2fa0,bg=#0a0a0a] #W "],
            &["set", "-t", session, "-g", "window-status-current-format", "#[fg=#0a0a0a,bg=#00d4aa,bold] #W "],
            &["set", "-t", session, "-g", "@mobile", "off"],
        ];
        for args in cmds {
            let _ = Command::new("tmux").args(*args).status();
        }
        eprintln!("desktop mode — top bar, all windows visible");
        Ok(())
    }
}

mod focus {
    use super::*;

    /// f100: Focus a project window. Optional command auto-returns to C2.
    pub fn f100(session: &str, window: &str, cmd: Option<&str>) -> anyhow::Result<()> {
        // Select the target window
        let target = format!("{}:{}", session, window);
        Command::new("tmux")
            .args(["select-window", "-t", &target])
            .status()
            .map_err(|_| anyhow::anyhow!("window '{}' not found", window))?;

        if let Some(c) = cmd {
            // Run command, then auto-return to C2
            let wrapped = format!("{} ; tmux select-window -t {}:0", c, session);
            send_keys(session, window, &wrapped)?;
            eprintln!("[{}] dispatched — auto-return to C2 on completion", window);
        } else {
            eprintln!("[{}] focused — use `tmuxisfree home` to return", window);
        }
        Ok(())
    }

    /// f101: Return to C2 window
    pub fn f101(session: &str) -> anyhow::Result<()> {
        Command::new("tmux")
            .args(["select-window", "-t", &format!("{}:0", session)])
            .status()?;
        eprintln!("home — C2");
        Ok(())
    }
}

mod backlog {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn stack_dir() -> PathBuf {
        let dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".tmuxisfree/backlog");
        let _ = fs::create_dir_all(&dir);
        dir
    }

    fn stack_file(window: &str) -> PathBuf {
        stack_dir().join(format!("{}.stack", window))
    }

    fn read_stack(window: &str) -> Vec<String> {
        fs::read_to_string(stack_file(window))
            .unwrap_or_default()
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect()
    }

    fn write_stack(window: &str, tasks: &[String]) {
        let _ = fs::write(stack_file(window), tasks.join("\n"));
    }

    /// f110: Push task onto pane backlog (top of stack = last line)
    pub fn f110(window: &str, task: &str) -> anyhow::Result<()> {
        let mut stack = read_stack(window);
        stack.push(task.to_string());
        write_stack(window, &stack);
        eprintln!("[{}] pushed ({} in backlog)", window, stack.len());
        Ok(())
    }

    /// f111: Pop top task and dispatch to pane
    pub fn f111(window: &str, session: &str) -> anyhow::Result<()> {
        let mut stack = read_stack(window);
        if stack.is_empty() {
            eprintln!("[{}] backlog empty", window);
            return Ok(());
        }
        let task = stack.pop().unwrap();
        write_stack(window, &stack);
        eprintln!("[{}] popped: {} ({} remaining)", window, task, stack.len());
        dispatch::f10(session, window, &task)
    }

    /// f112: Show backlog (one pane or all)
    pub fn f112(window: Option<&str>) -> anyhow::Result<()> {
        if let Some(w) = window {
            let stack = read_stack(w);
            if stack.is_empty() {
                println!("[{}] (empty)", w);
            } else {
                println!("[{}] {} tasks:", w, stack.len());
                for (i, t) in stack.iter().rev().enumerate() {
                    let marker = if i == 0 { ">" } else { " " };
                    println!("  {} {}", marker, t);
                }
            }
        } else {
            let dir = stack_dir();
            let mut any = false;
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("stack") {
                        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("?");
                        let stack = read_stack(name);
                        if !stack.is_empty() {
                            any = true;
                            let top = stack.last().unwrap();
                            println!("[{}] {} tasks — next: {}", name, stack.len(), top);
                        }
                    }
                }
            }
            if !any {
                println!("all backlogs empty");
            }
        }
        Ok(())
    }

    /// f113: Clear a pane's backlog
    pub fn f113(window: &str) -> anyhow::Result<()> {
        let count = read_stack(window).len();
        let _ = fs::remove_file(stack_file(window));
        eprintln!("[{}] cleared {} tasks", window, count);
        Ok(())
    }

    /// f114: Drain — pop all tasks, dispatch one at a time, wait for idle between each
    pub fn f114(window: &str, session: &str, interval: u64) -> anyhow::Result<()> {
        loop {
            let stack = read_stack(window);
            if stack.is_empty() {
                eprintln!("[{}] backlog drained", window);
                return Ok(());
            }
            // Wait for pane to be idle before popping
            eprintln!("[{}] waiting for idle ({} remaining)...", window, stack.len());
            loop {
                if is_idle(session, window) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_secs(interval));
            }
            f111(window, session)?;
        }
    }
}

mod scaffold {
    use super::init::expand_tilde;
    use std::fs;
    use std::path::Path;

    /// f120: Scaffold fleet.toml by scanning for git repos
    pub fn f120(dir: &str, output: &str, session: &str, depth: usize) -> anyhow::Result<()> {
        let root = expand_tilde(dir);
        let root = Path::new(&root);
        if !root.is_dir() {
            anyhow::bail!("not a directory: {}", root.display());
        }

        let mut panes: Vec<(String, String)> = Vec::new();
        scan_git_repos(root, depth, 0, &mut panes)?;
        // Deduplicate by name — keep shortest path (closest to home)
        panes.sort_by(|a, b| a.1.len().cmp(&b.1.len()));
        let mut seen = std::collections::HashSet::new();
        panes.retain(|(name, _)| seen.insert(name.clone()));
        panes.sort_by(|a, b| a.0.cmp(&b.0));

        if panes.is_empty() {
            eprintln!("no git repos found in {} (depth={})", root.display(), depth);
            return Ok(());
        }

        let home = dirs::home_dir().unwrap_or_default();
        let home_str = home.to_string_lossy();

        let mut toml = format!("session = \"{}\"\n\n", session);
        for (name, path) in &panes {
            // Collapse home dir to ~
            let short = if path.starts_with(home_str.as_ref()) {
                format!("~{}", &path[home_str.len()..])
            } else {
                path.clone()
            };
            toml.push_str(&format!("[[pane]]\nname = \"{}\"\ndir = \"{}\"\n\n", name, short));
        }

        fs::write(output, &toml)?;
        eprintln!("[scaffold] {} repos → {}", panes.len(), output);
        for (name, _) in &panes {
            eprintln!("  {}", name);
        }
        Ok(())
    }

    fn scan_git_repos(dir: &Path, max_depth: usize, current: usize, out: &mut Vec<(String, String)>) -> anyhow::Result<()> {
        if current > max_depth { return Ok(()); }
        let entries = fs::read_dir(dir)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            if name.starts_with('.') { continue; }
            if path.join(".git").is_dir() {
                out.push((name, path.to_string_lossy().to_string()));
            } else if current < max_depth {
                scan_git_repos(&path, max_depth, current + 1, out)?;
            }
        }
        Ok(())
    }
}
