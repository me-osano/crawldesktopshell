mod cmd;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
pub use crawl_ipc::client::CrawlClient;
use std::path::PathBuf;

fn default_socket_path() -> PathBuf {
    std::env::var("CRAWL_SOCKET")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let uid = std::env::var("UID")
                .ok()
                .and_then(|u| u.parse().ok())
                .unwrap_or(1000);
            PathBuf::from(format!("/run/user/{}/crawl.sock", uid))
        })
}

#[derive(Parser)]
#[command(
    name = "crawl",
    version,
    about = "System services CLI — display, brightness, wallpaper, audio",
    long_about = None,
)]
struct Cli {
    #[arg(long, short = 'j', global = true)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Audio(cmd::audio::AudioArgs),
    Bluetooth(cmd::bluetooth::BtArgs),
    Daemon(cmd::daemon::DaemonArgs),
    Brightness(cmd::brightness::BrightnessArgs),
    Network(cmd::network::NetworkArgs),
    Shell(cmd::shell::ShellArgs),
    Status(cmd::status::StatusArgs),
    Health(cmd::health::HealthArgs),
    Sysmon(cmd::sysmon::SysmonArgs),
    Proc(cmd::proc::ProcArgs),
    Sysinfo(cmd::sysinfo::SysinfoArgs),
    Theme(cmd::theme::ThemeArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = color_eyre::install();

    let cli = Cli::parse();

    let socket_path = default_socket_path();
    let client = CrawlClient::new(socket_path);
    let json_mode = cli.json;

    match cli.command {
        Commands::Audio(args) => cmd::audio::run(client, args, json_mode).await?,
        Commands::Bluetooth(args) => cmd::bluetooth::run(client, args, json_mode).await?,
        Commands::Daemon(args) => cmd::daemon::run(client, args, json_mode).await?,
        Commands::Brightness(args) => cmd::brightness::run(client, args, json_mode).await?,
        Commands::Network(args) => cmd::network::run(client, args, json_mode).await?,
        Commands::Shell(args) => cmd::shell::run(client, args, json_mode).await?,
        Commands::Status(args) => cmd::status::run(client, args, json_mode).await?,
        Commands::Health(args) => cmd::health::run(client, args, json_mode).await?,
        Commands::Sysmon(args) => cmd::sysmon::run(client, args, json_mode).await?,
        Commands::Proc(args) => cmd::proc::run(client, args, json_mode).await?,
        Commands::Sysinfo(args) => cmd::sysinfo::run(client, args, json_mode).await?,
        Commands::Theme(args) => cmd::theme::run(client, args, json_mode).await?,
    }

    Ok(())
}
