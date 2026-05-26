use anyhow::Result;
use clap::Args;
use crawl_ipc::commands::CrawlCommand;

use crate::{CrawlClient, output};

#[derive(Args)]
pub struct StatusArgs;

pub async fn run(client: CrawlClient, _args: StatusArgs, json_mode: bool) -> Result<()> {
    let res = client.command(CrawlCommand::Status).await?;
    output::handle_format(&res, json_mode, |val| {
        if json_mode {
            output::print_value(val, true);
            return Ok(());
        }

        let audio = val.get("audio");
        let bluetooth = val.get("bluetooth");
        let brightness = val.get("brightness");
        let network = val.get("network");

        output::print_header("Audio");
        render_status_block(audio);
        output::print_header("Bluetooth");
        render_status_block(bluetooth);
        output::print_header("Brightness");
        render_status_block(brightness);
        output::print_header("Network");
        render_status_block(network);
        Ok(())
    })
}

fn render_status_block(section: Option<&serde_json::Value>) {
    let Some(section) = section else {
        output::print_err("missing status data");
        return;
    };
    let ok = section.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
    let error = section.get("error").and_then(|v| v.as_str());

    if ok {
        output::print_ok("ok");
    } else if let Some(err) = error {
        output::print_err(err);
    } else {
        output::print_err("unknown error");
    }
}
