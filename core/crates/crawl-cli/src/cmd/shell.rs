//! Crawl Desktop Shell CLI commands
//!
//! Manages the quickshell-based desktop shell lifecycle and IPC bridge.
//!
//! # Usage
//! - `crawl shell`                    - Show status & help
//! - `crawl shell --status`           - Check if the shell is running
//! - `crawl shell --run`              - Start the shell
//! - `crawl shell --restart`          - Restart the shell
//! - `crawl shell --kill`             - Stop the shell
//! - `crawl shell --ipc <target> <function> [args...]` - Invoke quickshell IPC

use anyhow::{Context, Result, bail};
use clap::Args;
use std::process::{Command, Stdio};

use crate::{CrawlClient, output};

#[derive(Args)]
pub struct ShellArgs {
    /// Show shell status
    #[arg(long)]
    pub status: bool,

    /// Start the desktop shell (quickshell)
    #[arg(long)]
    pub run: bool,

    /// Restart the desktop shell
    #[arg(long)]
    pub restart: bool,

    /// Stop the desktop shell
    #[arg(long)]
    pub kill: bool,

    /// Invoke quickshell IPC: <target> <function> [args...]
    /// Example: crawl shell --ipc CrawlService getTheme
    #[arg(long, num_args = 2..)]
    pub ipc: Option<Vec<String>>,

    /// Quickshell config path (default: auto-detect)
    #[arg(long)]
    pub config: Option<String>,

    /// Quickshell binary path (default: auto-detect)
    #[arg(long)]
    pub binary: Option<String>,
}

pub async fn run(client: CrawlClient, args: ShellArgs, json_mode: bool) -> Result<()> {
    if !args.status && !args.run && !args.restart && !args.kill && args.ipc.is_none() {
        return show_status_and_help(client, json_mode).await;
    }

    if args.status {
        cmd_status(json_mode).await
    } else if args.run {
        cmd_run(&args, json_mode).await
    } else if args.restart {
        cmd_kill(json_mode).await?;
        cmd_run(&args, json_mode).await
    } else if args.kill {
        cmd_kill(json_mode).await
    } else if let Some(ipc_args) = args.ipc {
        cmd_ipc(&ipc_args, json_mode).await
    } else {
        bail!("expected one of --status/--run/--restart/--kill/--ipc")
    }
}

// ── Status ──────────────────────────────────────────────────────────────────

async fn cmd_status(json_mode: bool) -> Result<()> {
    let running = is_shell_running();
    let pid = find_shell_pid();

    if json_mode {
        let status = serde_json::json!({
            "running": running,
            "pid": pid,
        });
        println!("{}", serde_json::to_string_pretty(&status)?);
        return Ok(());
    }

    print_banner();
    output::print_header("Shell Status");
    if running {
        output::print_ok(&format!("quickshell is running (pid {})", pid.unwrap_or(0)));
    } else {
        output::print_err("quickshell is not running");
    }
    Ok(())
}

// ── Run ─────────────────────────────────────────────────────────────────────

fn resolve_shell_config() -> String {
    if let Ok(cfg) = std::env::var("CRAWL_SHELL_CONFIG") {
        return cfg;
    }
    // Standard quickshell config locations
    let candidates = [
        format!("{}/.config/quickshell/crawl", env_home()),
        format!("{}/.config/quickshell/crawl/shell.qml", env_home()),
        "/etc/crawl/shell.qml".to_string(),
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            if path.ends_with(".qml") {
                return path.clone();
            }
            let qml = format!("{}/shell.qml", path);
            if std::path::Path::new(&qml).exists() {
                return qml;
            }
            return path.clone();
        }
    }
    candidates[0].clone()
}

fn resolve_quickshell_binary() -> String {
    if let Ok(bin) = std::env::var("CRAWL_SHELL_BINARY") {
        return bin;
    }
    "quickshell".to_string()
}

async fn cmd_run(args: &ShellArgs, json_mode: bool) -> Result<()> {
    let binary = args
        .binary
        .clone()
        .or_else(|| std::env::var("CRAWL_SHELL_BINARY").ok())
        .unwrap_or_else(|| resolve_quickshell_binary());

    let config = args
        .config
        .clone()
        .or_else(|| std::env::var("CRAWL_SHELL_CONFIG").ok())
        .unwrap_or_else(|| resolve_shell_config());

    // Check if already running
    if is_shell_running() {
        if json_mode {
            println!("{}", serde_json::json!({"status": "already_running"}));
            return Ok(());
        }
        output::print_info("quickshell is already running");
        return Ok(());
    }

    let child = Command::new(&binary)
        .arg("--path")
        .arg(&config)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
        .with_context(|| format!("failed to launch {}", binary))?;

    if json_mode {
        let status = serde_json::json!({
            "status": "started",
            "pid": child.id(),
            "binary": binary,
            "config": config,
        });
        println!("{}", serde_json::to_string_pretty(&status)?);
        return Ok(());
    }

    output::print_ok(&format!(
        "quickshell started (pid {}) — config: {}",
        child.id(),
        config
    ));
    Ok(())
}

// ── Kill ────────────────────────────────────────────────────────────────────

async fn cmd_kill(json_mode: bool) -> Result<()> {
    if !is_shell_running() {
        if json_mode {
            println!("{}", serde_json::json!({"status": "not_running"}));
            return Ok(());
        }
        output::print_info("quickshell is not running");
        return Ok(());
    }

    let status = Command::new("pkill")
        .args(["-x", "quickshell"])
        .status()
        .with_context(|| "failed to run pkill")?;

    if json_mode {
        println!(
            "{}",
            serde_json::json!({"status": if status.success() { "killed" } else { "failed" }})
        );
        return Ok(());
    }

    if status.success() {
        output::print_ok("quickshell stopped");
    } else {
        output::print_err("failed to stop quickshell");
    }
    Ok(())
}

// ── IPC ─────────────────────────────────────────────────────────────────────

async fn cmd_ipc(args: &[String], json_mode: bool) -> Result<()> {
    let (target, function, rest) = match args {
        [t, f, rest @ ..] => (t.as_str(), f.as_str(), rest),
        _ => bail!("usage: --ipc <target> <function> [args...]"),
    };

    let mut cmd = Command::new("qs");
    cmd.args(["ipc", "call", target, function]);
    for arg in rest {
        cmd.arg(arg);
    }

    let output = cmd
        .output()
        .with_context(|| "failed to run qs — is quickshell installed?")?;

    if json_mode {
        let result = serde_json::json!({
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
            "success": output.status.success(),
        });
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        output::print_err(&format!("IPC call failed: {}", stderr.trim()));
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.trim().is_empty() {
        println!("{}", stdout.trim());
    }
    Ok(())
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn is_shell_running() -> bool {
    find_shell_pid().is_some()
}

fn find_shell_pid() -> Option<u32> {
    let output = Command::new("pgrep")
        .args(["-x", "quickshell"])
        .output()
        .ok()?;
    if output.status.success() {
        let pid_str = String::from_utf8_lossy(&output.stdout);
        pid_str.trim().parse::<u32>().ok()
    } else {
        None
    }
}

fn env_home() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
}

fn print_banner() {
    println!(
        r#"
        ____ ____      ___        ___     ____  ____  
       / ___|  _ \    / \ \      / / |   |  _ \/ ___| 
      | |   | |_) |  / _ \ \ /\ / /| |   | | | \___ \ 
      | |___|  _ <  / ___ \ V  V / | |___| |_| |___) |
       \____|_| \_\/_/   \_/\_/  |_____|____/|____/ 
"#
    );
}

async fn show_status_and_help(client: CrawlClient, json_mode: bool) -> Result<()> {
    // Show daemon status as context
    if let Ok(res) = client
        .command(crawl_ipc::commands::CrawlCommand::Status)
        .await
    {
        if !json_mode {
            print_banner();
            output::print_header("Daemon Status");
            if let Some(ok) = res.get("ok").and_then(|v| v.as_bool()) {
                if ok {
                    output::print_ok("daemon is connected");
                }
            }
        }
    }

    // Show shell status
    cmd_status(json_mode).await?;

    if !json_mode {
        output::print_header("Available Commands");
        let headers = vec!["Command".to_string(), "Description".to_string()];
        let rows = vec![
            vec![
                "crawl shell --status".to_string(),
                "Show whether quickshell is running".to_string(),
            ],
            vec![
                "crawl shell --run".to_string(),
                "Start the crawl desktop shell".to_string(),
            ],
            vec![
                "crawl shell --kill".to_string(),
                "Stop the crawl desktop shell".to_string(),
            ],
            vec![
                "crawl shell --restart".to_string(),
                "Restart the crawl desktop shell".to_string(),
            ],
            vec![
                "crawl shell --ipc <target> <fn> [args]".to_string(),
                "Invoke a quickshell IPC function".to_string(),
            ],
        ];
        let renderable = output::CliRenderable::new(headers, rows);
        output::render_table(&renderable);
    }
    Ok(())
}
