use anyhow::{Result, bail};
use clap::Args;
use crawl_ipc::commands::CrawlCommand;

use crate::{CrawlClient, output};

#[derive(Args)]
pub struct SysmonArgs {
    #[arg(long)]
    pub cpu: bool,
    #[arg(long)]
    pub mem: bool,
    #[arg(long)]
    pub disks: bool,
    #[arg(long)]
    pub net: bool,
    #[arg(long)]
    pub gpu: bool,
}

pub async fn run(client: CrawlClient, args: SysmonArgs, json_mode: bool) -> Result<()> {
    if !args.cpu && !args.mem && !args.disks && !args.net && !args.gpu {
        bail!("expected a sysmon action: --cpu, --mem, --disks, --net, or --gpu");
    }

    if args.cpu {
        let res = client.command(CrawlCommand::SysmonCpu).await?;
        output::handle_format(&res, json_mode, |val| {
            output::print_value(val, true);
            Ok(())
        })
    } else if args.mem {
        let res = client.command(CrawlCommand::SysmonMem).await?;
        output::handle_format(&res, json_mode, |val| {
            output::print_value(val, true);
            Ok(())
        })
    } else if args.disks {
        let res = client.command(CrawlCommand::SysmonDisks).await?;
        output::handle_format(&res, json_mode, |val| {
            output::print_value(val, true);
            Ok(())
        })
    } else if args.net {
        let res = client.command(CrawlCommand::SysmonNet).await?;
        output::handle_format(&res, json_mode, |val| {
            output::print_value(val, true);
            Ok(())
        })
    } else {
        let res = client.command(CrawlCommand::SysmonGpu).await?;
        output::handle_format(&res, json_mode, |val| {
            output::print_value(val, true);
            Ok(())
        })
    }
}
