use anyhow::Result;
use clap::Args;
use crawl_ipc::commands::CrawlCommand;

use crate::{CrawlClient, output};

#[derive(Args)]
pub struct HealthArgs;

pub async fn run(client: CrawlClient, _args: HealthArgs, json_mode: bool) -> Result<()> {
    let res = client.command(CrawlCommand::Health).await?;
    output::handle_format(&res, json_mode, |val| {
        output::print_value(val, true);
        Ok(())
    })
}
