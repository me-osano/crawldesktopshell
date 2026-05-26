use async_trait::async_trait;
use tracing::{error, info};

use crawl_ipc::protocol::CrawlResponse;

use crate::services::models::Service;
use crate::state::SharedState;
use crate::sysmon;

pub struct SysmonService {
    state: SharedState,
}

impl SysmonService {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Service for SysmonService {
    fn name(&self) -> &'static str {
        "sysmon"
    }

    async fn start(&self) -> anyhow::Result<()> {
        info!("Starting sysmon service");
        let cfg = self.state.config.sysmon.clone();
        let tx = self.state.event_bus.sender();

        tokio::spawn(async move {
            if let Err(e) = sysmon::run(cfg, tx).await {
                error!(domain = "sysmon", "Sysmon service failed: {e:#}");
            }
        });

        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        info!("Stopping sysmon service");
        Ok(())
    }

    async fn handle(
        &self,
        method: &str,
        _params: &serde_json::Value,
        id: Option<serde_json::Value>,
    ) -> Option<CrawlResponse> {
        let response = match method {
            "SysmonCpu" => {
                let cpu = sysmon::get_cpu();
                CrawlResponse::success(id, serde_json::to_value(cpu).unwrap_or_default())
            }
            "SysmonMem" => {
                let mem = sysmon::get_mem();
                CrawlResponse::success(id, serde_json::to_value(mem).unwrap_or_default())
            }
            "SysmonDisks" => {
                let disks = sysmon::get_disks();
                CrawlResponse::success(id, serde_json::to_value(disks).unwrap_or_default())
            }
            "SysmonNet" => {
                let net = sysmon::get_net();
                CrawlResponse::success(id, serde_json::to_value(net).unwrap_or_default())
            }
            "SysmonGpu" => {
                let gpu = sysmon::get_gpu();
                CrawlResponse::success(id, serde_json::to_value(gpu).unwrap_or_default())
            }
            _ => return None,
        };

        Some(response)
    }
}
