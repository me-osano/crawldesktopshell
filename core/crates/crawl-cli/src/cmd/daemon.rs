use anyhow::Result;
use clap::{Args, Subcommand};
use crawl_ipc::commands::CrawlCommand;

use crate::{CrawlClient, output};

#[derive(Args)]
pub struct DaemonArgs {
    #[command(subcommand)]
    pub action: DaemonAction,
}

#[derive(Subcommand)]
pub enum DaemonAction {
    Ping,
    /// List registered services
    List,
    /// Register a service by name
    Register {
        name: String,
    },
    /// Unregister a running service by name
    Unregister {
        name: String,
    },
}

pub async fn run(client: CrawlClient, args: DaemonArgs, json_mode: bool) -> Result<()> {
    match args.action {
        DaemonAction::Ping => {
            let res = client.command(CrawlCommand::DaemonPing).await?;
            output::handle_format(&res, json_mode, |_| {
                output::print_ok("Daemon is alive");
                Ok(())
            })
        }
        DaemonAction::List => {
            let res = client.command(CrawlCommand::ServiceList).await?;
            output::handle_format(&res, json_mode, |value| {
                if let Some(services) = value.get("services").and_then(|v| v.as_array()) {
                    output::print_info("Registered services:");
                    for svc in services {
                        if let Some(name) = svc.as_str() {
                            output::print_item(&format!("- {name}"));
                        }
                    }
                }
                Ok(())
            })
        }
        DaemonAction::Register { name } => {
            let res = client
                .command(CrawlCommand::ServiceRegister { name: name.clone() })
                .await?;
            output::handle_format(&res, json_mode, |_| {
                output::print_ok(&format!("Service '{name}' registered"));
                Ok(())
            })
        }
        DaemonAction::Unregister { name } => {
            let res = client
                .command(CrawlCommand::ServiceUnregister { name: name.clone() })
                .await?;
            output::handle_format(&res, json_mode, |value| {
                if let Some(removed) = value.get("removed").and_then(|v| v.as_bool()) {
                    if removed {
                        output::print_ok(&format!("Service '{name}' unregistered"));
                    } else {
                        output::print_info(&format!("Service '{name}' was not registered"));
                    }
                }
                Ok(())
            })
        }
    }
}
