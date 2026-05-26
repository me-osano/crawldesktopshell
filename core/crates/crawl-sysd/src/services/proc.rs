use async_trait::async_trait;
use tracing::{error, info};

use crawl_ipc::error_code;
use crawl_ipc::protocol::CrawlResponse;

use crate::proc;
use crate::services::models::{Service, error_response};
use crate::state::SharedState;

pub struct ProcService {
    state: SharedState,
}

impl ProcService {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Service for ProcService {
    fn name(&self) -> &'static str {
        "proc"
    }

    async fn start(&self) -> anyhow::Result<()> {
        info!("Starting proc service");
        let cfg = self.state.config.processes.clone();
        let tx = self.state.event_bus.sender();

        tokio::spawn(async move {
            if let Err(e) = proc::run(cfg, tx).await {
                error!(domain = "proc", "Proc service failed: {e:#}");
            }
        });

        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        info!("Stopping proc service");
        Ok(())
    }

    async fn handle(
        &self,
        method: &str,
        params: &serde_json::Value,
        id: Option<serde_json::Value>,
    ) -> Option<CrawlResponse> {
        let response = match method {
            "ProcList" => {
                let sort_by = params
                    .get("sort_by")
                    .and_then(|v| v.as_str())
                    .unwrap_or("cpu");
                let top = params.get("top").and_then(|v| v.as_u64()).unwrap_or(20) as usize;
                let procs = proc::list_processes(sort_by, top);
                CrawlResponse::success(id, serde_json::to_value(procs).unwrap_or_default())
            }
            "ProcFind" => match params.get("name").and_then(|v| v.as_str()) {
                Some(name) => {
                    let procs = proc::find_processes(name);
                    CrawlResponse::success(id, serde_json::to_value(procs).unwrap_or_default())
                }
                None => CrawlResponse::error(id, error_code::INVALID_PARAMS, "name required"),
            },
            "ProcKill" => {
                let pid = match params.get("pid").and_then(|v| v.as_u64()) {
                    Some(pid) => pid as u32,
                    None => {
                        return Some(CrawlResponse::error(
                            id,
                            error_code::INVALID_PARAMS,
                            "pid required",
                        ));
                    }
                };
                let force = params
                    .get("force")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                match proc::kill_process(pid, force) {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => {
                        let msg: String = e.to_string();
                        error_response(id, msg)
                    }
                }
            }
            "ProcWatch" => {
                let pid = match params.get("pid").and_then(|v| v.as_u64()) {
                    Some(pid) => pid as u32,
                    None => {
                        return Some(CrawlResponse::error(
                            id,
                            error_code::INVALID_PARAMS,
                            "pid required",
                        ));
                    }
                };
                match proc::watch_pid(pid).await {
                    Ok(name) => CrawlResponse::success(
                        id,
                        serde_json::json!({"exited": true, "name": name}),
                    ),
                    Err(e) => {
                        let msg: String = e.to_string();
                        error_response(id, msg)
                    }
                }
            }
            _ => return None,
        };

        Some(response)
    }
}
