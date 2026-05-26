//! Brightness cli commands
//! crawl brightness                        - Shows status & help commands
//! crawl brightness --status               - Shows status
//! crawl brightness --get                  - Get brightness value of the default device
//! crawl brightness --set=70               - Handle setting brightness on default device
//! crawl brightness --set=70               - Handle setting brightness on default device
//! crawl brightness --inc=5                - Increase brightness by percent
//! crawl brightness --dec=5                - Decrease brightness by percent

use anyhow::{Result, bail};
use clap::Args;
use crawl_ipc::commands::CrawlCommand;

use crate::{CrawlClient, output};

#[derive(Args)]
pub struct BrightnessArgs {
    /// Show brightness status
    #[arg(long)]
    pub status: bool,
    /// Get brightness value of default device
    #[arg(long)]
    pub get: bool,
    /// Set brightness (0-100)
    #[arg(long)]
    pub set: Option<f32>,
    /// Increase brightness by percent
    #[arg(long)]
    pub inc: Option<f32>,
    /// Decrease brightness by percent
    #[arg(long)]
    pub dec: Option<f32>,
}

pub async fn run(client: CrawlClient, args: BrightnessArgs, json_mode: bool) -> Result<()> {
    // crawl brightness (no args) - show status & help
    if !args.status && !args.get && args.set.is_none() && args.inc.is_none() && args.dec.is_none() {
        return show_status_and_help(client, json_mode).await;
    }

    if args.status || args.get {
        let res = client.command(CrawlCommand::BrightnessGet).await?;
        output::handle_format(&res, json_mode, |val| {
            output::print_value(val, true);
            Ok(())
        })
    } else if let Some(value) = args.set {
        let res = client
            .command(CrawlCommand::BrightnessSet { value })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok(&format!("Brightness set to {}%", value));
            Ok(())
        })
    } else if let Some(value) = args.inc {
        let res = client
            .command(CrawlCommand::BrightnessInc { value })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok(&format!("Brightness increased by {}%", value));
            Ok(())
        })
    } else if let Some(value) = args.dec {
        let res = client
            .command(CrawlCommand::BrightnessDec { value })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok(&format!("Brightness decreased by {}%", value));
            Ok(())
        })
    } else {
        bail!("expected one of --status/--get/--set/--inc/--dec")
    }
}

async fn show_status_and_help(client: CrawlClient, json_mode: bool) -> Result<()> {
    // Try to show status, but don't fail if daemon isn't running
    let _ = show_status(client, json_mode).await;

    if !json_mode {
        output::print_header("\nAvailable Commands");
        let headers = vec!["Command".to_string(), "Description".to_string()];
        let rows = vec![
            vec![
                "crawl brightness --status".to_string(),
                "Show brightness status".to_string(),
            ],
            vec![
                "crawl brightness --get".to_string(),
                "Get brightness value".to_string(),
            ],
            vec![
                "crawl brightness --set=N".to_string(),
                "Set brightness (0-100)".to_string(),
            ],
            vec![
                "crawl brightness --inc=N".to_string(),
                "Increase brightness by N%".to_string(),
            ],
            vec![
                "crawl brightness --dec=N".to_string(),
                "Decrease brightness by N%".to_string(),
            ],
        ];
        let renderable = output::CliRenderable::new(headers, rows);
        output::render_table(&renderable);
    }
    Ok(())
}

async fn show_status(client: CrawlClient, json_mode: bool) -> Result<()> {
    let res = client.command(CrawlCommand::BrightnessGet).await?;
    output::handle_format(&res, json_mode, |val| {
        output::print_value(val, true);
        Ok(())
    })
}
