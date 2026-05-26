//! Theme CLI — query, set, and dynamically generate themes.
//!
//! Usage:
//!   crawl theme --get                          Show current theme
//!   crawl theme --list                         List available static variants
//!   crawl theme --set mocha                    Set variant (mocha, latte, nord, ...)
//!   crawl theme --set mocha --accent mauve     Set variant + accent
//!
//! Dynamic generation from a seed color:
//!   crawl theme --generate "#cba6f7"           TonalSpot, dark
//!   crawl theme --generate "#cba6f7" --scheme content --mode light
//!
//! Dynamic generation from a wallpaper:
//!   crawl theme --image ~/wallpaper.png        Extract seed, TonalSpot, dark
//!   crawl theme --image ~/wallpaper.png --scheme vibrant --mode light
//!
//! Scheme types: tonal_spot, content, monochrome, vibrant, faithful,
//!               fruit_salad, rainbow, muted
//! Modes: dark (default), light
//!
//! Generated themes persist the seed hex + scheme type + mode in theme.toml
//! so the daemon can reproduce the palette on restart without re-quantizing.

use anyhow::{Result, bail};
use clap::Args;
use crawl_ipc::commands::CrawlCommand;
use std::path::PathBuf;

use crate::{CrawlClient, output};

#[derive(Args)]
pub struct ThemeArgs {
    /// Show current theme
    #[arg(long)]
    pub get: bool,
    /// List available theme variants
    #[arg(long)]
    pub list: bool,
    /// Set a theme variant (mocha, latte, frappe, macchiato, nord, tokyo-night, gruvbox-light, gruvbox-dark, kanagawa-light, kanagawa-dark, rose-pine-light, rose-pine-dark, or custom name)
    #[arg(long)]
    pub set: Option<String>,
    /// Set accent color (rosewater, mauve, blue, or hex like #cba6f7)
    #[arg(long)]
    pub accent: Option<String>,
    /// Generate a dynamic theme from a hex color (e.g. #cba6f7)
    #[arg(long)]
    pub generate: Option<String>,
    /// Generate a dynamic theme from an image file
    #[arg(long)]
    pub image: Option<PathBuf>,
    /// Dynamic scheme type: tonal_spot, content, monochrome, vibrant, faithful, fruit_salad, rainbow, muted
    #[arg(long, default_value = "tonal_spot")]
    pub scheme: Option<String>,
    /// Color scheme mode: dark or light (for --generate/--image)
    #[arg(long)]
    pub mode: Option<String>,
    /// Output the response as JSON
    #[arg(long)]
    pub json_output: bool,
}

pub async fn run(client: CrawlClient, args: ThemeArgs, json_mode: bool) -> Result<()> {
    let has_set = args.set.is_some() || args.generate.is_some() || args.image.is_some();

    if !args.get && !args.list && !has_set {
        return show_status_and_help(client, json_mode).await;
    }

    if args.list {
        let res = client.command(CrawlCommand::ThemeList).await?;
        output::handle_format(&res, json_mode, |val| {
            if let Some(variants) = val.as_array() {
                output::print_header("Available Theme Variants");
                for name in variants {
                    output::print_item(&name.as_str().unwrap_or(""));
                }
            }
            Ok(())
        })
    } else if args.get {
        let res = client.command(CrawlCommand::ThemeGet).await?;
        output::handle_format(&res, json_mode, |val| {
            output::print_value(val, true);
            Ok(())
        })
    } else if let Some(name) = args.set {
        let cmd = CrawlCommand::ThemeSet {
            name: name.clone(),
            accent: args.accent.clone(),
        };
        let res = client.command(cmd).await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok(&format!("Theme set to variant: {}", name));
            if let Some(ref a) = args.accent {
                output::print_ok(&format!("Accent: {}", a));
            }
            Ok(())
        })
    } else if let Some(color) = args.generate {
        let res = client
            .command(CrawlCommand::ThemeGenerate {
                color: color.clone(),
                scheme: args.scheme.clone(),
                mode: args.mode.clone(),
            })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok(&format!("Generated theme from color: {}", color));
            Ok(())
        })
    } else if let Some(image_path) = args.image {
        if !image_path.exists() {
            bail!("Image file not found: {}", image_path.display());
        }
        let res = client
            .command(CrawlCommand::ThemeGenerateFromImage {
                path: image_path.to_string_lossy().to_string(),
                scheme: args.scheme.clone(),
                mode: args.mode.clone(),
            })
            .await?;
        output::handle_format(&res, json_mode, |_| {
            output::print_ok(&format!(
                "Generated theme from image: {}",
                image_path.display()
            ));
            Ok(())
        })
    } else {
        bail!("expected one of --get/--list/--set")
    }
}

async fn show_status_and_help(client: CrawlClient, json_mode: bool) -> Result<()> {
    let _ = show_status(client, json_mode).await;

    if !json_mode {
        output::print_header("\nAvailable Commands");
        let headers = vec!["Command".to_string(), "Description".to_string()];
        let rows = vec![
            vec![
                "crawl theme --get".to_string(),
                "Show current theme".to_string(),
            ],
            vec![
                "crawl theme --list".to_string(),
                "List available variants".to_string(),
            ],
            vec![
                "crawl theme --set=VAR".to_string(),
                "Set a theme variant".to_string(),
            ],
            vec![
                "crawl theme --set=VAR --accent=COLOR".to_string(),
                "Set variant with accent".to_string(),
            ],
            vec![
                "crawl theme --generate=COLOR [--scheme=TYPE] [--mode=MODE]".to_string(),
                "Generate dynamic theme from hex color".to_string(),
            ],
            vec![
                "crawl theme --image=PATH [--scheme=TYPE] [--mode=MODE]".to_string(),
                "Generate dynamic theme from image".to_string(),
            ],
        ];
        let renderable = output::CliRenderable::new(headers, rows);
        output::render_table(&renderable);
    }
    Ok(())
}

async fn show_status(client: CrawlClient, json_mode: bool) -> Result<()> {
    let res = client.command(CrawlCommand::ThemeGet).await?;
    output::handle_format(&res, json_mode, |val| {
        output::print_value(val, true);
        Ok(())
    })
}
