use std::sync::Arc;

use crawl_ipc::error_code;
use crawl_ipc::protocol::CrawlResponse;

use crate::services::audio::AudioService;
use crate::services::bluetooth::BluetoothService;
use crate::services::brightness::BrightnessService;
use crate::services::models::ServiceRegistry;
use crate::services::network::NetworkService;
use crate::state::SharedState;

pub async fn handle(
    state: SharedState,
    registry: Arc<ServiceRegistry>,
    method: String,
    params: serde_json::Value,
    id: Option<serde_json::Value>,
) -> CrawlResponse {
    match method.as_str() {
        "DaemonPing" => CrawlResponse::success(id, serde_json::json!({"ok": true})),

        "ServiceList" => {
            let services = registry.services.read().await;
            let list: Vec<&str> = services.keys().map(|k| k.as_str()).collect();
            CrawlResponse::success(id, serde_json::json!({"ok": true, "services": list}))
        }

        "ServiceRegister" => match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => {
                let name = name.to_string();
                match name.as_str() {
                    "audio" => {
                        registry
                            .register(Arc::new(AudioService::new(state.clone())))
                            .await;
                        CrawlResponse::success(
                            id,
                            serde_json::json!({"ok": true, "registered": true, "name": name}),
                        )
                    }
                    "bluetooth" => {
                        registry
                            .register(Arc::new(BluetoothService::new(state.clone())))
                            .await;
                        CrawlResponse::success(
                            id,
                            serde_json::json!({"ok": true, "registered": true, "name": name}),
                        )
                    }
                    "brightness" => {
                        registry
                            .register(Arc::new(BrightnessService::new(state.clone())))
                            .await;
                        CrawlResponse::success(
                            id,
                            serde_json::json!({"ok": true, "registered": true, "name": name}),
                        )
                    }
                    "network" => {
                        registry
                            .register(Arc::new(NetworkService::new(state.clone())))
                            .await;
                        CrawlResponse::success(
                            id,
                            serde_json::json!({"ok": true, "registered": true, "name": name}),
                        )
                    }
                    _ => CrawlResponse::error(
                        id,
                        error_code::INVALID_PARAMS,
                        &format!("Unknown service: {name}"),
                    ),
                }
            }
            None => CrawlResponse::error(id, error_code::INVALID_PARAMS, "name required"),
        },

        "ServiceUnregister" => match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => {
                let name = name.to_string();
                match registry.unregister(&name).await {
                    Some(service) => {
                        if let Err(err) = service.stop().await {
                            return CrawlResponse::error(
                                id,
                                error_code::INTERNAL_ERROR,
                                &format!("Failed to stop service: {err}"),
                            );
                        }
                        CrawlResponse::success(
                            id,
                            serde_json::json!({"ok": true, "removed": true, "name": name}),
                        )
                    }
                    None => CrawlResponse::success(
                        id,
                        serde_json::json!({"ok": true, "removed": false, "name": name}),
                    ),
                }
            }
            None => CrawlResponse::error(id, error_code::INVALID_PARAMS, "name required"),
        },

        _ => CrawlResponse::error(id, error_code::INVALID_PARAMS, "Unknown daemon method"),
    }
}
