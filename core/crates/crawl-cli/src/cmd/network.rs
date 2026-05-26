//! Network cli commands
//! crawl network                              - Shows status & help commands
//! crawl network --status                     - Shows network status
//! crawl network --power=on                   - Enable networking
//! crawl network --power=off                  - Disable networking
//! crawl network --wifi-list                  - List WiFi networks
//! crawl network --wifi-scan                  - Scan for WiFi networks
//! crawl network --wifi-details               - Show WiFi connection details
//! crawl network --wifi-connect=ssid --password=passwd - Connect to WiFi
//! crawl network --wifi-disconnect            - Disconnect WiFi (currently connected)
//! crawl network --wifi-forget=ssid           - Forget WiFi connection
//! crawl network --eth-list                   - List ethernet interfaces
//! crawl network --eth-details=iface          - Show ethernet interface details
//! crawl network --eth-connect=iface          - Connect to ethernet interface
//! crawl network --eth-disconnect=iface       - Disconnect ethernet interface
//! crawl network --hotspot                    - Show hotspot status
//! crawl network --hotspot-start=ssid         - Start hotspot (with optional --hotspot-password, --hotspot-iface, etc.)
//! crawl network --hotspot-stop               - Stop hotspot

use anyhow::{Result, bail};
use clap::Args;
use crawl_ipc::commands::CrawlCommand;
use serde_json::json;

use crate::output::CliRenderable;
use crate::{CrawlClient, output};

#[derive(Args)]
pub struct NetworkArgs {
    /// Show network status
    #[arg(long)]
    pub status: bool,
    /// Enable/disable networking (on/off)
    #[arg(long, value_parser = ["on", "off"])]
    pub power: Option<String>,
    /// List WiFi networks
    #[arg(long)]
    pub wifi_list: bool,
    /// Scan for WiFi networks
    #[arg(long)]
    pub wifi_scan: bool,
    /// Show WiFi connection details
    #[arg(long)]
    pub wifi_details: bool,
    /// Connect to WiFi by SSID
    #[arg(long)]
    pub wifi_connect: Option<String>,
    /// WiFi password (use with --wifi-connect)
    #[arg(long)]
    pub password: Option<String>,
    /// Disconnect WiFi by SSID
    #[arg(long)]
    pub wifi_disconnect: bool,
    /// Forget WiFi connection by SSID
    #[arg(long)]
    pub wifi_forget: Option<String>,
    /// List ethernet interfaces
    #[arg(long)]
    pub eth_list: bool,
    /// Show ethernet interface details
    #[arg(long)]
    pub eth_details: Option<String>,
    /// Connect to ethernet interface
    #[arg(long)]
    pub eth_connect: Option<String>,
    /// Disconnect ethernet interface
    #[arg(long)]
    pub eth_disconnect: Option<String>,
    /// Show hotspot status
    #[arg(long)]
    pub hotspot: bool,
    /// Start hotspot with SSID (use additional options for config)
    #[arg(long)]
    pub hotspot_start: Option<String>,
    /// Hotspot password (use with --hotspot-start)
    #[arg(long)]
    pub hotspot_password: Option<String>,
    /// Hotspot interface (use with --hotspot-start)
    #[arg(long)]
    pub hotspot_iface: Option<String>,
    /// Hotspot band (use with --hotspot-start)
    #[arg(long)]
    pub hotspot_band: Option<String>,
    /// Hotspot channel (use with --hotspot-start)
    #[arg(long)]
    pub hotspot_channel: Option<u32>,
    /// Hotspot backend: networkmanager or hostapd (use with --hotspot-start)
    #[arg(long)]
    pub hotspot_backend: Option<String>,
    /// Stop hotspot
    #[arg(long)]
    pub hotspot_stop: bool,
}

pub async fn run(client: CrawlClient, args: NetworkArgs, json_mode: bool) -> Result<()> {
    // crawl network (no args) - show status & help
    if !args.status
        && args.power.is_none()
        && !args.wifi_list
        && !args.wifi_scan
        && !args.wifi_details
        && args.wifi_connect.is_none()
        && !args.wifi_disconnect
        && args.wifi_forget.is_none()
        && !args.eth_list
        && args.eth_connect.is_none()
        && args.eth_disconnect.is_none()
        && args.eth_details.is_none()
        && !args.hotspot
        && args.hotspot_start.is_none()
        && !args.hotspot_stop
    {
        return show_status_and_help(client, json_mode).await;
    }

    if args.status {
        let res = client.command(CrawlCommand::NetworkStatus).await?;
        output::handle_format(&res, json_mode, |val| {
            render_network_status(val);
            Ok(())
        })
    } else if let Some(power) = args.power {
        let enabled = power == "on";
        let res = client
            .command(CrawlCommand::NetworkEnable { enabled })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            let msg = if enabled {
                "Networking enabled"
            } else {
                "Networking disabled"
            };
            output::print_ok(msg);
            Ok(())
        })
    } else if args.wifi_list {
        let res = client.command(CrawlCommand::WifiList).await?;
        output::handle_format(&res, json_mode, |val| {
            render_wifi_list(val);
            Ok(())
        })
    } else if args.wifi_scan {
        let res = client.command(CrawlCommand::WifiScan).await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok("WiFi scan requested");
            Ok(())
        })
    } else if args.wifi_details {
        let res = client.command(CrawlCommand::WifiDetails).await?;
        output::handle_format(&res, json_mode, |val| {
            render_wifi_details(val);
            Ok(())
        })
    } else if let Some(ssid) = args.wifi_connect {
        let res = client
            .command(CrawlCommand::WifiConnect {
                ssid,
                password: args.password,
            })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok("WiFi connect requested");
            Ok(())
        })
    } else if args.wifi_disconnect {
        let res = client.command(CrawlCommand::WifiDisconnect).await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok("WiFi disconnected");
            Ok(())
        })
    } else if let Some(ssid) = args.wifi_forget {
        let res = client.command(CrawlCommand::WifiForget { ssid }).await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok("WiFi connection removed");
            Ok(())
        })
    } else if args.eth_list {
        let res = client.command(CrawlCommand::EthernetList).await?;
        output::handle_format(&res, json_mode, |val| {
            render_ethernet_list(val);
            Ok(())
        })
    } else if let Some(iface) = args.eth_details {
        let res = client
            .command(CrawlCommand::EthernetDetails { iface: Some(iface) })
            .await?;
        output::handle_format(&res, json_mode, |val| {
            render_ethernet_details(val);
            Ok(())
        })
    } else if let Some(iface) = args.eth_connect {
        let res = client
            .command(CrawlCommand::EthernetConnect { iface: Some(iface) })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok("Ethernet connect requested");
            Ok(())
        })
    } else if let Some(iface) = args.eth_disconnect {
        let res = client
            .command(CrawlCommand::EthernetDisconnect { iface: Some(iface) })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok("Ethernet disconnected");
            Ok(())
        })
    } else if args.hotspot {
        let res = client.command(CrawlCommand::HotspotStatus).await?;
        output::handle_format(&res, json_mode, |val| {
            render_hotspot_status(val);
            Ok(())
        })
    } else if let Some(ssid) = args.hotspot_start {
        let backend = match args.hotspot_backend.as_deref() {
            Some("networkmanager") => Some("networkmanager"),
            Some("hostapd") => Some("hostapd"),
            Some(other) => return Err(anyhow::anyhow!("unsupported hotspot backend: {other}")),
            None => None,
        };

        let config = json!({
            "ssid": ssid,
            "password": args.hotspot_password,
            "iface": args.hotspot_iface,
            "band": args.hotspot_band,
            "channel": args.hotspot_channel,
            "backend": backend,
        });
        let res = client
            .command(CrawlCommand::HotspotStart {
                config: serde_json::from_value(config)?,
            })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok("Hotspot started");
            Ok(())
        })
    } else if args.hotspot_stop {
        let res = client.command(CrawlCommand::HotspotStop).await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok("Hotspot stopped");
            Ok(())
        })
    } else {
        bail!("expected a network action")
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
                "crawl network --status".to_string(),
                "Show network status".to_string(),
            ],
            vec![
                "crawl network --power=on|off".to_string(),
                "Enable/disable networking".to_string(),
            ],
            vec![
                "crawl network --wifi-list".to_string(),
                "List WiFi networks".to_string(),
            ],
            vec![
                "crawl network --wifi-scan".to_string(),
                "Scan WiFi networks".to_string(),
            ],
            vec![
                "crawl network --wifi-details".to_string(),
                "Show WiFi details".to_string(),
            ],
            vec![
                "crawl network --wifi-connect=SSID --password=PASS".to_string(),
                "Connect to WiFi".to_string(),
            ],
            vec![
                "crawl network --wifi-disconnect".to_string(),
                "Disconnect WiFi".to_string(),
            ],
            vec![
                "crawl network --wifi-forget=SSID".to_string(),
                "Forget WiFi connection".to_string(),
            ],
            vec![
                "crawl network --eth-list".to_string(),
                "List ethernet interfaces".to_string(),
            ],
            vec![
                "crawl network --eth-details=IFACE".to_string(),
                "Show ethernet interface details".to_string(),
            ],
            vec![
                "crawl network --eth-connect=IFACE".to_string(),
                "Connect ethernet".to_string(),
            ],
            vec![
                "crawl network --eth-disconnect=IFACE".to_string(),
                "Disconnect ethernet".to_string(),
            ],
            vec![
                "crawl network --hotspot".to_string(),
                "Show hotspot status".to_string(),
            ],
            vec![
                "crawl network --hotspot-start=SSID --hotspot-password=PASS".to_string(),
                "Start hotspot".to_string(),
            ],
            vec![
                "crawl network --hotspot-stop".to_string(),
                "Stop hotspot".to_string(),
            ],
        ];
        let renderable = CliRenderable::new(headers, rows);
        output::render_table(&renderable);
    }
    Ok(())
}

async fn show_status(client: CrawlClient, json_mode: bool) -> Result<()> {
    let res = client.command(CrawlCommand::NetworkStatus).await?;
    output::handle_format(&res, json_mode, |val| {
        render_network_status(val);
        Ok(())
    })
}

fn render_wifi_list(val: &serde_json::Value) {
    let Some(networks) = val.as_array() else {
        output::print_err("missing wifi list");
        return;
    };
    output::print_header("WiFi Networks");

    let headers = vec![
        "SSID".to_string(),
        "Signal".to_string(),
        "Security".to_string(),
        "Connected".to_string(),
        "Known".to_string(),
        "Band".to_string(),
        "BSSID".to_string(),
    ];
    let rows: Vec<Vec<String>> = networks
        .iter()
        .map(|net| {
            let ssid = net.get("ssid").and_then(|v| v.as_str()).unwrap_or("-");
            let signal = net.get("signal").and_then(|v| v.as_u64()).unwrap_or(0);
            let security = net.get("security").and_then(|v| v.as_str()).unwrap_or("-");
            let connected = net
                .get("connected")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let existing = net
                .get("existing")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let freq = net.get("frequency_mhz").and_then(|v| v.as_u64());
            let band = freq
                .map(frequency_band_label)
                .unwrap_or_else(|| "-".to_string());
            let bssid = net.get("bssid").and_then(|v| v.as_str()).unwrap_or("-");

            vec![
                ssid.to_string(),
                format!("{}%", signal),
                security.to_string(),
                if connected {
                    "yes".to_string()
                } else {
                    "".to_string()
                },
                if existing {
                    "yes".to_string()
                } else {
                    "".to_string()
                },
                band,
                bssid.to_string(),
            ]
        })
        .collect();
    let renderable = CliRenderable::new(headers, rows);
    output::render_table(&renderable);
}

fn render_wifi_details(val: &serde_json::Value) {
    output::print_header("WiFi Details");
    let headers = vec!["Property".to_string(), "Value".to_string()];
    let mut rows = Vec::new();

    let map = |key: &str| val.get(key).and_then(|v| v.as_str()).unwrap_or("-");
    let map_opt = |key: &str| {
        val.get(key)
            .and_then(|v| v.as_u64())
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string())
    };

    rows.push(vec!["SSID".to_string(), map("ssid").to_string()]);
    rows.push(vec!["Interface".to_string(), map("ifname").to_string()]);
    rows.push(vec!["Signal".to_string(), map_opt("signal")]);
    rows.push(vec!["Frequency".to_string(), map_opt("frequency_mhz")]);
    rows.push(vec!["Security".to_string(), map("security").to_string()]);
    rows.push(vec!["Band".to_string(), map("band").to_string()]);
    rows.push(vec!["Channel".to_string(), map_opt("channel")]);
    rows.push(vec!["BSSID".to_string(), map("bssid").to_string()]);
    rows.push(vec!["MAC".to_string(), map("mac").to_string()]);
    rows.push(vec!["IPv4".to_string(), map("ip4").to_string()]);
    rows.push(vec!["Gateway".to_string(), map("gateway4").to_string()]);
    rows.push(vec!["DNS".to_string(), join_list(val.get("dns4"))]);

    let renderable = CliRenderable::new(headers, rows);
    output::render_table(&renderable);
}

fn render_ethernet_list(val: &serde_json::Value) {
    let Some(interfaces) = val.as_array() else {
        output::print_err("missing ethernet list");
        return;
    };
    output::print_header("Ethernet Interfaces");

    let headers = vec![
        "Interface".to_string(),
        "Connected".to_string(),
        "IPv4".to_string(),
        "MAC".to_string(),
    ];
    let rows: Vec<Vec<String>> = interfaces
        .iter()
        .map(|iface| {
            let ifname = iface.get("ifname").and_then(|v| v.as_str()).unwrap_or("-");
            let connected = iface
                .get("connected")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let ip4 = iface.get("ip4").and_then(|v| v.as_str()).unwrap_or("-");
            let mac = iface.get("mac").and_then(|v| v.as_str()).unwrap_or("-");
            vec![
                ifname.to_string(),
                if connected {
                    "yes".to_string()
                } else {
                    "".to_string()
                },
                ip4.to_string(),
                mac.to_string(),
            ]
        })
        .collect();
    let renderable = CliRenderable::new(headers, rows);
    output::render_table(&renderable);
}

fn render_ethernet_details(val: &serde_json::Value) {
    output::print_header("Ethernet Details");
    let headers = vec!["Property".to_string(), "Value".to_string()];
    let mut rows = Vec::new();

    let map = |key: &str| val.get(key).and_then(|v| v.as_str()).unwrap_or("-");

    rows.push(vec!["Interface".to_string(), map("ifname").to_string()]);
    rows.push(vec!["Speed".to_string(), map("speed").to_string()]);
    rows.push(vec!["IPv4".to_string(), map("ipv4").to_string()]);
    rows.push(vec!["Gateway".to_string(), map("gateway4").to_string()]);
    rows.push(vec!["MAC".to_string(), map("mac").to_string()]);

    let renderable = CliRenderable::new(headers, rows);
    output::render_table(&renderable);
}

fn render_hotspot_status(val: &serde_json::Value) {
    output::print_header("Hotspot Status");
    let headers = vec!["Property".to_string(), "Value".to_string()];
    let mut rows = Vec::new();

    let map = |key: &str| val.get(key).and_then(|v| v.as_str()).unwrap_or("-");

    let active = val.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
    rows.push(vec![
        "Active".to_string(),
        if active {
            "yes".to_string()
        } else {
            "no".to_string()
        },
    ]);
    rows.push(vec!["SSID".to_string(), map("ssid").to_string()]);
    rows.push(vec!["Interface".to_string(), map("iface").to_string()]);
    rows.push(vec!["Band".to_string(), map("band").to_string()]);
    rows.push(vec![
        "Channel".to_string(),
        val.get("channel")
            .and_then(|v| v.as_u64())
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string()),
    ]);
    rows.push(vec!["Backend".to_string(), map("backend").to_string()]);

    let renderable = CliRenderable::new(headers, rows);
    output::render_table(&renderable);
}

fn render_network_status(val: &serde_json::Value) {
    output::print_header("Network Status");
    let headers = vec!["Property".to_string(), "Value".to_string()];
    let mut rows = Vec::new();

    let map = |key: &str| val.get(key).and_then(|v| v.as_str()).unwrap_or("-");
    let map_bool = |key: &str| val.get(key).and_then(|v| v.as_bool()).unwrap_or(false);

    rows.push(vec![
        "Connectivity".to_string(),
        map("connectivity").to_string(),
    ]);
    rows.push(vec![
        "WiFi Enabled".to_string(),
        bool_label(map_bool("wifi_enabled")),
    ]);
    rows.push(vec![
        "WiFi Available".to_string(),
        bool_label(map_bool("wifi_available")),
    ]);
    rows.push(vec![
        "Network Enabled".to_string(),
        bool_label(map_bool("network_enabled")),
    ]);
    rows.push(vec![
        "Ethernet Available".to_string(),
        bool_label(map_bool("ethernet_available")),
    ]);
    rows.push(vec!["Mode".to_string(), map("mode").to_string()]);
    rows.push(vec![
        "Active SSID".to_string(),
        map("active_ssid").to_string(),
    ]);

    let renderable = CliRenderable::new(headers, rows);
    output::render_table(&renderable);
}

fn bool_label(value: bool) -> String {
    if value {
        "yes".to_string()
    } else {
        "no".to_string()
    }
}

fn join_list(value: Option<&serde_json::Value>) -> String {
    let Some(value) = value else {
        return "-".to_string();
    };
    let Some(list) = value.as_array() else {
        return "-".to_string();
    };
    let items: Vec<String> = list
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    if items.is_empty() {
        "-".to_string()
    } else {
        items.join(", ")
    }
}

fn frequency_band_label(freq: u64) -> String {
    match freq {
        2400..=2499 => "2.4GHz".to_string(),
        5000..=5899 => "5GHz".to_string(),
        5925..=7125 => "6GHz".to_string(),
        _ => "-".to_string(),
    }
}
