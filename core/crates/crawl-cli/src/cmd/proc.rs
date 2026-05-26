use anyhow::{Result, bail};
use clap::Args;
use crawl_ipc::commands::CrawlCommand;

use crate::{CrawlClient, output};

#[derive(Args)]
pub struct ProcArgs {
    /// List processes (optionally sort_by and top)
    #[arg(long)]
    pub list: bool,
    #[arg(long, requires = "list")]
    pub sort_by: Option<String>,
    #[arg(long, requires = "list")]
    pub top: Option<u32>,
    /// Find processes by name
    #[arg(long)]
    pub find: Option<String>,
    /// Kill a process by PID
    #[arg(long)]
    pub kill: Option<u32>,
    /// Force kill (SIGKILL) — use with --kill
    #[arg(long, requires = "kill")]
    pub force: bool,
    /// Watch a PID until it exits
    #[arg(long)]
    pub watch: Option<u32>,
}

pub async fn run(client: CrawlClient, args: ProcArgs, json_mode: bool) -> Result<()> {
    if args.list {
        let res = client
            .command(CrawlCommand::ProcList {
                sort_by: args.sort_by,
                top: args.top,
            })
            .await?;
        output::handle_format(&res, json_mode, |val| {
            output::print_value(val, true);
            Ok(())
        })
    } else if let Some(name) = args.find {
        let res = client.command(CrawlCommand::ProcFind { name }).await?;
        output::handle_format(&res, json_mode, |val| {
            output::print_value(val, true);
            Ok(())
        })
    } else if let Some(pid) = args.kill {
        let force = Some(args.force);
        let res = client
            .command(CrawlCommand::ProcKill { pid, force })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok(&format!("Process {} killed", pid));
            Ok(())
        })
    } else if let Some(pid) = args.watch {
        let res = client.command(CrawlCommand::ProcWatch { pid }).await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok(&format!("Process {} exited", pid));
            Ok(())
        })
    } else {
        bail!("expected a proc action: --list, --find, --kill, or --watch");
    }
}
