//! Bluetooth cli commands
//! crawl bluetooth                      - Shows status & help commands
//! crawl bluetooth --status             - Shows status
//! crawl bluetooth --list               - List paired bluetooth devices
//! crawl bluetooth --scan --timeout=30  - Scan for available bluetooth devices with timeout
//! crawl bluetooth --connect=ADDRESS    - Connect bluetooth device
//! crawl bluetooth --disconnect=ADDRESS - Disconnect connected bluetooth device
//! crawl bluetooth --pair=ADDRESS       - Pair to bluetooth device
//! crawl bluetooth --remove=ADDRESS     - Remove paired bluetooth device
//! crawl bluetooth --power=on           - Enable bluetooth adapter
//! crawl bluetooth --power=off          - Disable bluetooth adapter
//! crawl bluetooth --alias=NAME         - Set alias name for bluetooth adapter

use anyhow::{Result, bail};
use clap::Args;
use crawl_ipc::commands::CrawlCommand;

use crate::{CrawlClient, output};

#[derive(Args)]
pub struct BtArgs {
    /// Show bluetooth status
    #[arg(long)]
    pub status: bool,
    /// List paired devices
    #[arg(long)]
    pub list: bool,
    /// Scan for devices (with optional timeout in seconds)
    #[arg(long)]
    pub scan: bool,
    /// Scan timeout in seconds
    #[arg(long, requires = "scan")]
    pub timeout: Option<u64>,
    /// Connect to device by address
    #[arg(long)]
    pub connect: Option<String>,
    /// Disconnect device by address
    #[arg(long)]
    pub disconnect: Option<String>,
    /// Pair with device by address
    #[arg(long)]
    pub pair: Option<String>,
    /// Remove paired device by address
    #[arg(long)]
    pub remove: Option<String>,
    /// Power adapter on/off
    #[arg(long, value_parser = ["on", "off"])]
    pub power: Option<String>,
    /// Set trust for device by address
    #[arg(long)]
    pub trust: Option<String>,
    /// Trust value (true/false)
    #[arg(long)]
    pub trusted: Option<bool>,
    /// Set alias for device
    #[arg(long)]
    pub alias: Option<String>,
    /// Device address for alias (use with --alias)
    #[arg(long, requires = "alias")]
    pub alias_address: Option<String>,
    /// Set discoverable on/off
    #[arg(long, value_parser = ["on", "off"])]
    pub discoverable: Option<String>,
    /// Set pairable on/off
    #[arg(long, value_parser = ["on", "off"])]
    pub pairable: Option<String>,
}

pub async fn run(client: CrawlClient, args: BtArgs, json_mode: bool) -> Result<()> {
    // crawl bluetooth (no args) - show status & help
    if !args.status
        && !args.list
        && !args.scan
        && args.connect.is_none()
        && args.disconnect.is_none()
        && args.pair.is_none()
        && args.remove.is_none()
        && args.power.is_none()
        && args.trust.is_none()
        && args.alias.is_none()
        && args.discoverable.is_none()
        && args.pairable.is_none()
    {
        return show_status_and_help(client, json_mode).await;
    }

    if args.status {
        let res = client.command(CrawlCommand::BluetoothStatus).await?;
        output::handle_format(&res, json_mode, |val| {
            output::print_value(val, true);
            Ok(())
        })
    } else if args.list {
        let res = client.command(CrawlCommand::BluetoothDevices).await?;
        output::handle_format(&res, json_mode, |val| {
            output::print_value(val, true);
            Ok(())
        })
    } else if args.scan {
        let res = client
            .command(CrawlCommand::BluetoothScan {
                timeout: args.timeout,
            })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok("Bluetooth scan started");
            Ok(())
        })
    } else if let Some(address) = args.connect {
        let res = client
            .command(CrawlCommand::BluetoothConnect { address })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok("Connected");
            Ok(())
        })
    } else if let Some(address) = args.disconnect {
        let res = client
            .command(CrawlCommand::BluetoothDisconnect { address })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok("Disconnected");
            Ok(())
        })
    } else if args.power.is_some() {
        let enabled = args.power.as_deref() == Some("on");
        let res = client
            .command(CrawlCommand::BluetoothPower { enabled })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            let msg = if enabled { "Powered on" } else { "Powered off" };
            output::print_ok(msg);
            Ok(())
        })
    } else if let Some(address) = args.pair {
        let res = client
            .command(CrawlCommand::BluetoothPair { address })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok("Paired");
            Ok(())
        })
    } else if let Some(address) = args.remove {
        let res = client
            .command(CrawlCommand::BluetoothRemove { address })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok("Removed");
            Ok(())
        })
    } else if let Some(address) = args.trust {
        let trusted = args.trusted.unwrap_or(true);
        let res = client
            .command(CrawlCommand::BluetoothTrust { address, trusted })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            let msg = if trusted { "Trusted" } else { "Untrusted" };
            output::print_ok(msg);
            Ok(())
        })
    } else if let Some(alias) = args.alias {
        let address = args
            .alias_address
            .clone()
            .ok_or_else(|| anyhow::anyhow!("--alias-address is required with --alias"))?;
        let res = client
            .command(CrawlCommand::BluetoothAlias { address, alias })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok("Alias set");
            Ok(())
        })
    } else if let Some(setting) = args.discoverable {
        let enabled = setting == "on";
        let res = client
            .command(CrawlCommand::BluetoothDiscoverable { enabled })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            let msg = if enabled {
                "Discoverable"
            } else {
                "Not discoverable"
            };
            output::print_ok(msg);
            Ok(())
        })
    } else if let Some(setting) = args.pairable {
        let enabled = setting == "on";
        let res = client
            .command(CrawlCommand::BluetoothPairable { enabled })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            let msg = if enabled { "Pairable" } else { "Not pairable" };
            output::print_ok(msg);
            Ok(())
        })
    } else {
        bail!("expected a bluetooth action")
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
                "crawl bluetooth --status".to_string(),
                "Show bluetooth status".to_string(),
            ],
            vec![
                "crawl bluetooth --list".to_string(),
                "List paired devices".to_string(),
            ],
            vec![
                "crawl bluetooth --scan --timeout=N".to_string(),
                "Scan for devices".to_string(),
            ],
            vec![
                "crawl bluetooth --connect=ADDRESS".to_string(),
                "Connect to device".to_string(),
            ],
            vec![
                "crawl bluetooth --disconnect=ADDRESS".to_string(),
                "Disconnect device".to_string(),
            ],
            vec![
                "crawl bluetooth --pair=ADDRESS".to_string(),
                "Pair with device".to_string(),
            ],
            vec![
                "crawl bluetooth --remove=ADDRESS".to_string(),
                "Remove paired device".to_string(),
            ],
            vec![
                "crawl bluetooth --power=on|off".to_string(),
                "Power adapter on/off".to_string(),
            ],
            vec![
                "crawl bluetooth --alias=NAME --alias-address=ADDR".to_string(),
                "Set device alias".to_string(),
            ],
            vec![
                "crawl bluetooth --discoverable=on|off".to_string(),
                "Set discoverable".to_string(),
            ],
            vec![
                "crawl bluetooth --pairable=on|off".to_string(),
                "Set pairable".to_string(),
            ],
        ];
        let renderable = output::CliRenderable::new(headers, rows);
        output::render_table(&renderable);
    }
    Ok(())
}

async fn show_status(client: CrawlClient, json_mode: bool) -> Result<()> {
    let res = client.command(CrawlCommand::BluetoothStatus).await?;
    output::handle_format(&res, json_mode, |val| {
        output::print_value(val, true);
        Ok(())
    })
}
