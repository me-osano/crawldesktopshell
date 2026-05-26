use async_trait::async_trait;
use tracing::info;

use crawl_ipc::protocol::CrawlResponse;

use crate::idle::IdleController;
use crate::services::models::{Service, error_response};
use crate::state::SharedState;

pub struct IdleService {
    controller: IdleController,
}

impl IdleService {
    pub fn new(state: SharedState) -> Self {
        Self {
            controller: IdleController::new(state.event_bus.sender()),
        }
    }
}

#[async_trait]
impl Service for IdleService {
    fn name(&self) -> &'static str {
        "idle"
    }

    async fn start(&self) -> anyhow::Result<()> {
        info!("Starting idle service");
        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        info!("Stopping idle service");
        self.controller.uninhibit().await;
        Ok(())
    }

    async fn handle(
        &self,
        method: &str,
        params: &serde_json::Value,
        id: Option<serde_json::Value>,
    ) -> Option<CrawlResponse> {
        let response = match method {
            "IdleInhibit" => match self.controller.inhibit().await {
                Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                Err(e) => error_response(id, e.to_string()),
            },
            "IdleUninhibit" => {
                self.controller.uninhibit().await;
                CrawlResponse::success(id, serde_json::json!({"ok": true}))
            }
            "IdleStatus" => {
                let status = self.controller.status().await;
                CrawlResponse::success(id, serde_json::to_value(status).unwrap_or_default())
            }
            "IdleInhibitWithTimeout" => {
                let seconds = params.get("seconds").and_then(|v| v.as_u64()).unwrap_or(0);
                if seconds == 0 {
                    error_response(id, "missing or invalid 'seconds' parameter".to_string())
                } else {
                    match self.controller.inhibit_with_timeout(seconds).await {
                        Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                        Err(e) => error_response(id, e.to_string()),
                    }
                }
            }
            _ => return None,
        };

        Some(response)
    }
}
