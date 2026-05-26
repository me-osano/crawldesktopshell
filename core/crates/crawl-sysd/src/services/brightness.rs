use async_trait::async_trait;
use tokio::sync::Mutex;

use crawl_ipc::events::{BrightnessEvent, CrawlEvent};
use crawl_ipc::protocol::CrawlResponse;

use crate::brightness;
use crate::services::models::{Service, error_response};
use crate::state::SharedState;

pub struct BrightnessService {
    state: SharedState,
    backlight: Mutex<Option<brightness::Backlight>>,
}

impl BrightnessService {
    pub fn new(state: SharedState) -> Self {
        Self {
            state,
            backlight: Mutex::new(None),
        }
    }

    async fn get_backlight(
        &self,
    ) -> Result<tokio::sync::MutexGuard<'_, Option<brightness::Backlight>>, CrawlResponse> {
        let mut guard = self.backlight.lock().await;
        if guard.is_some() {
            return Ok(guard);
        }
        let cfg = self.state.config.brightness.clone();
        match brightness::Backlight::open_async(&cfg).await {
            Ok(b) => {
                *guard = Some(b);
                Ok(guard)
            }
            Err(e) => Err(error_response(None, e.to_string())),
        }
    }

    async fn emit_event(&self, status: &crawl_ipc::types::BrightnessStatus) {
        self.state
            .event_bus
            .publish(CrawlEvent::Brightness(BrightnessEvent::Changed {
                status: status.clone(),
            }));
    }
}

#[async_trait]
impl Service for BrightnessService {
    fn name(&self) -> &'static str {
        "brightness"
    }

    async fn start(&self) -> anyhow::Result<()> {
        let cfg = self.state.config.brightness.clone();
        match brightness::Backlight::open_async(&cfg).await {
            Ok(b) => {
                match b.status_async().await {
                    Ok(status) => {
                        self.emit_event(&status).await;
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "brightness status read failed on start");
                    }
                }
                *self.backlight.lock().await = Some(b);
            }
            Err(e) => {
                tracing::warn!(error = %e, "brightness backlight unavailable on start");
            }
        }
        Ok(())
    }

    async fn handle(
        &self,
        method: &str,
        params: &serde_json::Value,
        id: Option<serde_json::Value>,
    ) -> Option<CrawlResponse> {
        let response = match method {
            "BrightnessGet" => match self.get_backlight().await {
                Ok(guard) => {
                    if let Some(ref b) = *guard {
                        match b.status_async().await {
                            Ok(status) => CrawlResponse::success(
                                id,
                                serde_json::to_value(status).unwrap_or_default(),
                            ),
                            Err(e) => error_response(id, e.to_string()),
                        }
                    } else {
                        error_response(id, "no backlight device available".to_string())
                    }
                }
                Err(e) => e,
            },
            "BrightnessSet" => {
                let value = params.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                let cfg = self.state.config.brightness.clone();
                match self.get_backlight().await {
                    Ok(mut guard) => {
                        if let Some(ref b) = *guard {
                            match b.set_percent_async(value, &cfg).await {
                                Ok(status) => {
                                    self.emit_event(&status).await;
                                    CrawlResponse::success(
                                        id,
                                        serde_json::to_value(status).unwrap_or_default(),
                                    )
                                }
                                Err(e) => {
                                    *guard = None;
                                    error_response(id, e.to_string())
                                }
                            }
                        } else {
                            error_response(id, "no backlight device available".to_string())
                        }
                    }
                    Err(e) => e,
                }
            }
            "BrightnessInc" => {
                let value = params.get("value").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                let cfg = self.state.config.brightness.clone();
                match self.get_backlight().await {
                    Ok(mut guard) => {
                        if let Some(ref b) = *guard {
                            match b.adjust_percent_async(value, &cfg).await {
                                Ok(status) => {
                                    self.emit_event(&status).await;
                                    CrawlResponse::success(
                                        id,
                                        serde_json::to_value(status).unwrap_or_default(),
                                    )
                                }
                                Err(e) => {
                                    *guard = None;
                                    error_response(id, e.to_string())
                                }
                            }
                        } else {
                            error_response(id, "no backlight device available".to_string())
                        }
                    }
                    Err(e) => e,
                }
            }
            "BrightnessDec" => {
                let value = params.get("value").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32;
                let cfg = self.state.config.brightness.clone();
                match self.get_backlight().await {
                    Ok(mut guard) => {
                        if let Some(ref b) = *guard {
                            match b.adjust_percent_async(-value, &cfg).await {
                                Ok(status) => {
                                    self.emit_event(&status).await;
                                    CrawlResponse::success(
                                        id,
                                        serde_json::to_value(status).unwrap_or_default(),
                                    )
                                }
                                Err(e) => {
                                    *guard = None;
                                    error_response(id, e.to_string())
                                }
                            }
                        } else {
                            error_response(id, "no backlight device available".to_string())
                        }
                    }
                    Err(e) => e,
                }
            }
            _ => return None,
        };

        Some(response)
    }
}
