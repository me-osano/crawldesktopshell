//! Audio cli commands
//! crawl audio                      - Shows status & help commands
//! crawl audio --status             - Shows status
//! crawl audio --volume=70          - Handles output volume by default
//! crawl audio --list               - List both output & input devices
//! crawl audio --output             - List output devices and their details
//! crawl audio --output --volume=70 - Handles output volume
//! crawl audio --output --mute      - Toggles mute on default output device
//! crawl audio --output --unmute    - Unmutes default output device
//! crawl audio --input              - List input devices
//! crawl audio --input --volume=70  - Handles input volume
//! crawl audio --input --mute       - Toggle mute on default input device
//! crawl audio --input --unmute     - Unmutes default input device

use crate::{
    CrawlClient,
    output::{self, CliRenderable},
};
use anyhow::{Result, bail};
use clap::Args;
use crawl_ipc::commands::CrawlCommand;

#[derive(Args)]
pub struct AudioArgs {
    /// Show audio status
    #[arg(long)]
    pub status: bool,
    /// Set volume (0-100)
    #[arg(long)]
    pub volume: Option<u32>,
    /// Target output devices (speakers)
    #[arg(long)]
    pub output: bool,
    /// List input devices (microphones)
    #[arg(long)]
    pub input: bool,
    /// Toggle mute
    #[arg(long)]
    pub mute: bool,
    /// Unmute
    #[arg(long)]
    pub unmute: bool,
    /// List all devices (output & input)
    #[arg(long)]
    pub list: bool,
}

pub async fn run(client: CrawlClient, args: AudioArgs, json_mode: bool) -> Result<()> {
    // crawl audio (no args) - show status & help
    if !args.status
        && !args.input
        && !args.mute
        && !args.unmute
        && args.volume.is_none()
        && !args.list
        && !args.output
    {
        return show_status_and_help(client, json_mode).await;
    }

    // Determine device type
    let device = if args.input {
        Some("input")
    } else if args.output {
        Some("output")
    } else {
        None
    };

    if args.status {
        show_status(client, json_mode).await
    } else if let Some(vol) = args.volume {
        let command = if device == Some("input") {
            CrawlCommand::AudioInputVolume { percent: vol }
        } else {
            CrawlCommand::AudioVolume { percent: vol }
        };
        let res = client.command(command).await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok(&format!("Volume set to {}%", vol));
            Ok(())
        })
    } else if args.mute {
        let command = if device == Some("input") {
            CrawlCommand::AudioMuteInput
        } else {
            CrawlCommand::AudioMute
        };
        let res = client.command(command).await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_info("Muted");
            Ok(())
        })
    } else if args.unmute {
        let command = if device == Some("input") {
            CrawlCommand::AudioUnmuteInput
        } else {
            CrawlCommand::AudioUnmute
        };
        let res = client.command(command).await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_info("Unmuted");
            Ok(())
        })
    } else if args.list {
        // List both output and input devices
        list_all_devices(client, json_mode).await
    } else if args.output {
        let res = client.command(CrawlCommand::AudioSinks).await?;
        output::handle_format(&res, json_mode, |val| {
            render_audio_table(val, "Output Devices (Speakers)");
            Ok(())
        })
    } else {
        bail!("expected one of --status/--volume/--input/--mute/--unmute/--list")
    }
}

async fn show_status_and_help(client: CrawlClient, json_mode: bool) -> Result<()> {
    show_status(client, json_mode).await?;

    if !json_mode {
        output::print_header("\nAvailable Commands");
        let headers = vec!["Command".to_string(), "Description".to_string()];
        let rows = vec![
            vec![
                "crawl audio --status".to_string(),
                "Show audio status".to_string(),
            ],
            vec![
                "crawl audio --volume=N".to_string(),
                "Set output volume (0-100)".to_string(),
            ],
            vec![
                "crawl audio --output --volume=N".to_string(),
                "Set output volume".to_string(),
            ],
            vec![
                "crawl audio --input --volume=N".to_string(),
                "Set input volume".to_string(),
            ],
            vec![
                "crawl audio --output --mute".to_string(),
                "Mute output".to_string(),
            ],
            vec![
                "crawl audio --output --unmute".to_string(),
                "Unmute output".to_string(),
            ],
            vec![
                "crawl audio --input --mute".to_string(),
                "Mute input".to_string(),
            ],
            vec![
                "crawl audio --input --unmute".to_string(),
                "Unmute input".to_string(),
            ],
            vec![
                "crawl audio --list".to_string(),
                "List all devices".to_string(),
            ],
        ];
        let renderable = CliRenderable::new(headers, rows);
        output::render_table(&renderable);
    }
    Ok(())
}

async fn show_status(client: CrawlClient, json_mode: bool) -> Result<()> {
    let res = client.command(CrawlCommand::AudioSinks).await?;
    output::handle_format(&res, json_mode, |val| {
        if let Some(devices) = val.as_array() {
            if let Some(default) = devices
                .iter()
                .find(|d| d["is_default"].as_bool().unwrap_or(false))
            {
                let name = default["name"].as_str().unwrap_or("-");
                let vol = default["volume_percent"].as_u64().unwrap_or(0);
                let muted = default["muted"].as_bool().unwrap_or(false);

                let headers = vec!["Property".to_string(), "Value".to_string()];
                let rows = vec![
                    vec!["Default".to_string(), name.to_string()],
                    vec!["Volume".to_string(), format!("{}%", vol)],
                    vec!["Muted".to_string(), muted.to_string()],
                ];
                let renderable = CliRenderable::new(headers, rows);
                output::render_table(&renderable);
            }
        }
        Ok(())
    })
}

async fn list_all_devices(client: CrawlClient, json_mode: bool) -> Result<()> {
    let res = client.command(CrawlCommand::AudioSinks).await?;
    output::handle_format(&res, json_mode, |val| {
        render_audio_table(val, "Output Devices (Speakers)");
        Ok(())
    })?;

    let res = client.command(CrawlCommand::AudioSources).await?;
    output::handle_format(&res, json_mode, |val| {
        render_audio_table(val, "Input Devices (Microphones)");
        Ok(())
    })
}

fn render_audio_table(val: &serde_json::Value, title: &str) {
    if let Some(devices) = val.as_array() {
        output::print_header(title);

        let headers = vec![
            "Name".to_string(),
            "Volume".to_string(),
            "Muted".to_string(),
            "Default".to_string(),
        ];
        let rows: Vec<Vec<String>> = devices
            .iter()
            .map(|dev| {
                let name = dev["name"].as_str().unwrap_or("?");
                let vol = dev["volume_percent"].as_u64().unwrap_or(0);
                let muted = dev["muted"].as_bool().unwrap_or(false);
                let default = dev["is_default"].as_bool().unwrap_or(false);
                vec![
                    name.to_string(),
                    format!("{}%", vol),
                    if muted {
                        "Yes".to_string()
                    } else {
                        "No".to_string()
                    },
                    if default {
                        "✓".to_string()
                    } else {
                        "".to_string()
                    },
                ]
            })
            .collect();

        let renderable = CliRenderable::new(headers, rows);
        output::render_table(&renderable);
    }
}
