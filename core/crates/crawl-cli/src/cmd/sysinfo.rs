use anyhow::Result;
use clap::Args;
use crawl_ipc::commands::CrawlCommand;

use crate::{CrawlClient, output};

#[derive(Args)]
pub struct SysinfoArgs {
    /// Show OS info
    #[arg(long)]
    pub os: bool,
    /// Show compositor info
    #[arg(long)]
    pub compositor: bool,
    /// Show session info
    #[arg(long)]
    pub session: bool,
    /// Show hardware info
    #[arg(long)]
    pub hardware: bool,
    /// Show display/monitor info
    #[arg(long)]
    pub display: bool,
}

pub async fn run(client: CrawlClient, args: SysinfoArgs, json_mode: bool) -> Result<()> {
    let res = client.command(CrawlCommand::Sysinfo).await?;

    output::handle_format(&res, json_mode, |val| {
        let show_all =
            !args.os && !args.compositor && !args.session && !args.hardware && !args.display;

        if show_all || args.os {
            if let Some(os) = val.get("os") {
                output::print_header("OS");
                print_field(os, "name", "Name");
                print_field(os, "kernel", "Kernel");
                print_field(os, "pretty_name", "Pretty name");
                print_field(os, "hostname", "Hostname");
                print_field(os, "id", "ID");
            }
        }

        if show_all || args.compositor {
            if let Some(compositor) = val.get("compositor") {
                output::print_header("Compositor");
                print_field(compositor, "type", "Type");
                print_field(compositor, "name", "Name");
            }
        }

        if show_all || args.session {
            if let Some(session) = val.get("session") {
                output::print_header("Session");
                print_field(session, "type", "Type");
                print_field(session, "user", "User");
                print_field(session, "seat", "Seat");
                print_field(session, "home", "Home");
                print_field(session, "shell", "Shell");
                print_field(session, "terminal", "Terminal");
                if let Some(uptime) = session.get("uptime_seconds").and_then(|v| v.as_u64()) {
                    output::print_info(&format!("Uptime: {}s", uptime));
                }
            }
        }

        if show_all || args.hardware {
            if let Some(hw) = val.get("hardware") {
                output::print_header("Hardware");
                print_field(hw, "cpu_model", "CPU");
                print_field(hw, "cpu_cores", "Cores");
                if let Some(mem) = hw.get("memory_total").and_then(|v| v.as_u64()) {
                    output::print_info(&format!("Memory: {} MB", mem / 1024 / 1024));
                }
                print_field(hw, "gpu", "GPU");
                if let Some(disk_total) = hw.get("disk_total").and_then(|v| v.as_u64()) {
                    let disk_used = hw.get("disk_used").and_then(|v| v.as_u64()).unwrap_or(0);
                    output::print_info(&format!(
                        "Disk: {} used / {} total MB",
                        disk_used / 1024 / 1024,
                        disk_total / 1024 / 1024
                    ));
                }
            }
        }

        if show_all || args.display {
            if let Some(display) = val.get("display") {
                output::print_header("Displays");
                if let Some(monitors) = display.get("monitors").and_then(|v| v.as_array()) {
                    for mon in monitors {
                        let name = mon.get("name").and_then(|v| v.as_str()).unwrap_or("?");
                        let w = mon.get("width").and_then(|v| v.as_u64()).unwrap_or(0);
                        let h = mon.get("height").and_then(|v| v.as_u64()).unwrap_or(0);
                        let rate = mon
                            .get("refresh_rate")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        output::print_info(&format!("{name}: {w}x{h} @ {rate}Hz"));
                    }
                }
            }
        }

        Ok(())
    })
}

fn print_field(val: &serde_json::Value, field: &str, label: &str) {
    if let Some(v) = val.get(field) {
        output::print_info(&format!("{}: {}", label, v));
    }
}
