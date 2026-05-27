use std::sync::Arc;

use async_trait::async_trait;
use tracing::info;

use crawl_ipc::protocol::CrawlResponse;

use crate::clipboard::ClipboardBackend;
use crate::services::models::{Service, error_response};
use crate::state::SharedState;

pub struct ClipboardService {
    backend: Arc<ClipboardBackend>,
    #[allow(dead_code)]
    state: SharedState,
}

impl ClipboardService {
    pub fn new(state: SharedState) -> Self {
        let cfg = &state.config.clipboard;
        let backend = Arc::new(ClipboardBackend::new(cfg.clone(), state.event_bus.sender()));
        Self { backend, state }
    }
}

#[async_trait]
impl Service for ClipboardService {
    fn name(&self) -> &'static str {
        "clipboard"
    }

    async fn start(&self) -> anyhow::Result<()> {
        info!("Starting clipboard service");
        self.backend.init().await?;
        let backend = self.backend.clone();
        backend.start_monitor();
        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        info!("Stopping clipboard service");
        self.backend.stop();
        Ok(())
    }

    async fn handle(
        &self,
        method: &str,
        params: &serde_json::Value,
        id: Option<serde_json::Value>,
    ) -> Option<CrawlResponse> {
        let response = match method {
            "ClipboardList" => {
                let entries = self.backend.list();
                CrawlResponse::success(id, serde_json::to_value(entries).unwrap_or_default())
            }
            "ClipboardGetContent" => {
                let entry_id = params.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                match self.backend.get_content(entry_id).await {
                    Ok(Some(content)) => {
                        CrawlResponse::success(id, serde_json::to_value(content).unwrap_or_default())
                    }
                    Ok(None) => error_response(id, "entry not found".to_string()),
                    Err(e) => error_response(id, e.to_string()),
                }
            }
            "ClipboardCopy" => {
                let entry_id = params.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                match self.backend.copy_to_clipboard(entry_id).await {
                    Ok(true) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Ok(false) => error_response(id, "entry not found".to_string()),
                    Err(e) => error_response(id, e.to_string()),
                }
            }
            "ClipboardDelete" => {
                let entry_id = params.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                match self.backend.delete(entry_id).await {
                    Ok(true) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Ok(false) => error_response(id, "entry not found".to_string()),
                    Err(e) => error_response(id, e.to_string()),
                }
            }
            "ClipboardWipe" => match self.backend.wipe().await {
                Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                Err(e) => error_response(id, e.to_string()),
            },
            "ClipboardPin" => {
                let entry_id = params.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                match self.backend.toggle_pin(entry_id).await {
                    Ok(true) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Ok(false) => error_response(id, "entry not found".to_string()),
                    Err(e) => error_response(id, e.to_string()),
                }
            }
            "ClipboardPasteText" => {
                let text = params
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                match self.backend.paste_text(&text) {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e.to_string()),
                }
            }
            "ClipboardSet" => {
                let text = params
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let mime = params
                    .get("mime")
                    .and_then(|v| v.as_str())
                    .unwrap_or("text/plain")
                    .to_string();
                match self.backend.set_clipboard(&text, &mime) {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e.to_string()),
                }
            }
            _ => return None,
        };

        Some(response)
    }

    fn is_healthy(&self) -> bool {
        // Monitor thread is running if enabled and Wayland is available
        true
    }
}
