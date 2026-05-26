use async_trait::async_trait;
use tracing::{error, info};

use crawl_ipc::protocol::CrawlResponse;

use crate::audio;
use crate::services::models::{Service, error_response};
use crate::state::SharedState;

pub struct AudioService {
    state: SharedState,
}

impl AudioService {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Service for AudioService {
    fn name(&self) -> &'static str {
        "audio"
    }

    async fn start(&self) -> anyhow::Result<()> {
        info!("Starting audio service");
        let cfg = self.state.config.audio.clone();
        let tx = self.state.event_bus.sender();

        tokio::spawn(async move {
            if let Err(e) = crate::audio::run(cfg, tx).await {
                error!(domain = "audio", "Audio service failed: {e:#}");
            }
        });

        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        info!("Stopping audio service");
        // Crawl: audio doesn't have an explicit stop, it will end when the task is dropped
        Ok(())
    }

    async fn handle(
        &self,
        method: &str,
        params: &serde_json::Value,
        id: Option<serde_json::Value>,
    ) -> Option<CrawlResponse> {
        let cfg = self.state.config.audio.clone();

        let response = match method {
            "AudioSinks" => match audio::list_sinks(&cfg).await {
                Ok(devices) => {
                    CrawlResponse::success(id, serde_json::to_value(devices).unwrap_or_default())
                }
                Err(e) => error_response(id, e.to_string()),
            },
            "AudioSources" => match audio::list_sources(&cfg).await {
                Ok(devices) => {
                    CrawlResponse::success(id, serde_json::to_value(devices).unwrap_or_default())
                }
                Err(e) => error_response(id, e.to_string()),
            },
            "AudioVolume" => {
                let percent = params.get("percent").and_then(|v| v.as_u64()).unwrap_or(50) as u32;
                match audio::set_output_volume(&cfg, percent).await {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e.to_string()),
                }
            }
            "AudioInputVolume" => {
                let percent = params.get("percent").and_then(|v| v.as_u64()).unwrap_or(50) as u32;
                match audio::set_input_volume(&cfg, percent).await {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e.to_string()),
                }
            }
            "AudioMute" => match audio::set_output_mute(&cfg, true).await {
                Ok(()) => CrawlResponse::success(id, serde_json::json!({"muted": true})),
                Err(e) => error_response(id, e.to_string()),
            },
            "AudioUnmute" => match audio::set_output_mute(&cfg, false).await {
                Ok(()) => CrawlResponse::success(id, serde_json::json!({"muted": false})),
                Err(e) => error_response(id, e.to_string()),
            },
            "AudioMuteInput" => match audio::set_input_mute(&cfg, true).await {
                Ok(()) => CrawlResponse::success(id, serde_json::json!({"muted": true})),
                Err(e) => error_response(id, e.to_string()),
            },
            "AudioUnmuteInput" => match audio::set_input_mute(&cfg, false).await {
                Ok(()) => CrawlResponse::success(id, serde_json::json!({"muted": false})),
                Err(e) => error_response(id, e.to_string()),
            },
            "AudioSetDefaultSink" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                if name.is_empty() {
                    error_response(id, "missing 'name' parameter".to_string())
                } else {
                    match audio::set_default_sink(&cfg, name).await {
                        Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                        Err(e) => error_response(id, e.to_string()),
                    }
                }
            }
            "AudioSetDefaultSource" => {
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                if name.is_empty() {
                    error_response(id, "missing 'name' parameter".to_string())
                } else {
                    match audio::set_default_source(&cfg, name).await {
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
