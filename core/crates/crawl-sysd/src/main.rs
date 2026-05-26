mod audio;
mod bluetooth;
mod brightness;
mod config;
mod daemon;
mod idle;
mod network;
mod notification;
mod proc;
mod scheduler;
mod services;
mod state;
mod sysinfo;
mod sysmon;

use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::daemon::Daemon;
pub use crate::scheduler::{Scheduler, ShouldRun};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::load()?;

    // Logging — CRAWL_LOG=debug or default to info
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("CRAWL_LOG")
                .unwrap_or_else(|_| EnvFilter::new(&config.daemon.log_level)),
        )
        .with_target(false)
        .compact()
        .init();

    info!("crawl-sysd starting");

    let mut daemon = Daemon::new(config).await?;
    // Register services
    daemon.register_services().await;
    // Configure child daemon IPC routing
    daemon.configure_router();
    // Run (blocks until shutdown)
    daemon.run().await?;

    Ok(())
}
