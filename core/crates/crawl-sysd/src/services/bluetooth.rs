use async_trait::async_trait;
use tracing::{error, info};

use crawl_ipc::error_code;
use crawl_ipc::protocol::CrawlResponse;

use crate::bluetooth;
use crate::bluetooth::BluetoothRuntime;
use crate::services::models::{Service, error_response};
use crate::state::SharedState;

pub struct BluetoothService {
    state: SharedState,
}

impl BluetoothService {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }

    fn runtime(&self) -> Option<&BluetoothRuntime> {
        self.state.bluetooth.as_deref()
    }

    fn bt_unavailable(id: Option<serde_json::Value>) -> CrawlResponse {
        error_response(id, "Bluetooth runtime unavailable".to_string())
    }
}

#[async_trait]
impl Service for BluetoothService {
    fn name(&self) -> &'static str {
        "bluetooth"
    }

    async fn start(&self) -> anyhow::Result<()> {
        let Some(runtime) = self.state.bluetooth.clone() else {
            info!("Bluetooth runtime not available, skipping bluetooth service");
            return Ok(());
        };
        info!("Starting bluetooth service");
        let cfg = self.state.config.bluetooth.clone();
        let tx = self.state.event_bus.sender();

        tokio::spawn(async move {
            if let Err(e) = bluetooth::run(runtime, cfg, tx).await {
                error!(domain = "bluetooth", "Bluetooth service failed: {e:#}");
            }
        });

        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        info!("Stopping bluetooth service");

        if let Some(rt) = self.runtime() {
            bluetooth::stop_scan(rt).await;
            rt.unregister_agent().await;
        }

        Ok(())
    }

    async fn handle(
        &self,
        method: &str,
        params: &serde_json::Value,
        id: Option<serde_json::Value>,
    ) -> Option<CrawlResponse> {
        let rt = match self.runtime() {
            Some(rt) => rt,
            None => return Some(Self::bt_unavailable(id)),
        };
        let response = match method {
            "BluetoothStatus" => match bluetooth::get_status(rt).await {
                Ok(status) => {
                    CrawlResponse::success(id, serde_json::to_value(status).unwrap_or_default())
                }
                Err(e) => error_response(id, e.to_string()),
            },
            "BluetoothDevices" => match bluetooth::get_devices(rt).await {
                Ok(devices) => {
                    CrawlResponse::success(id, serde_json::to_value(devices).unwrap_or_default())
                }
                Err(e) => error_response(id, e.to_string()),
            },
            "BluetoothScan" => {
                let timeout = params
                    .get("timeout")
                    .and_then(|v| v.as_u64())
                    .or(Some(self.state.config.bluetooth.scan_timeout_secs));
                match bluetooth::scan(rt, timeout).await {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e.to_string()),
                }
            }
            "BluetoothScanStop" => {
                bluetooth::stop_scan(rt).await;
                CrawlResponse::success(id, serde_json::json!({"ok": true}))
            }
            "BluetoothConnect" => match params.get("address").and_then(|v| v.as_str()) {
                Some(addr) => match bluetooth::connect(rt, addr).await {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e.to_string()),
                },
                None => CrawlResponse::error(id, error_code::INVALID_PARAMS, "address required"),
            },
            "BluetoothDisconnect" => match params.get("address").and_then(|v| v.as_str()) {
                Some(addr) => match bluetooth::disconnect(rt, addr).await {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e.to_string()),
                },
                None => CrawlResponse::error(id, error_code::INVALID_PARAMS, "address required"),
            },
            "BluetoothPower" => match params.get("enabled").and_then(|v| v.as_bool()) {
                Some(on) => match bluetooth::set_powered(rt, on).await {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e.to_string()),
                },
                None => CrawlResponse::error(id, error_code::INVALID_PARAMS, "enabled required"),
            },
            "BluetoothPair" => match params.get("address").and_then(|v| v.as_str()) {
                Some(addr) => match bluetooth::pair(rt, addr).await {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e.to_string()),
                },
                None => CrawlResponse::error(id, error_code::INVALID_PARAMS, "address required"),
            },
            "BluetoothRemove" => match params.get("address").and_then(|v| v.as_str()) {
                Some(addr) => match bluetooth::remove_device(rt, addr).await {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e.to_string()),
                },
                None => CrawlResponse::error(id, error_code::INVALID_PARAMS, "address required"),
            },
            "BluetoothTrust" => match (
                params.get("address").and_then(|v| v.as_str()),
                params.get("trusted").and_then(|v| v.as_bool()),
            ) {
                (Some(addr), Some(trusted)) => {
                    match bluetooth::set_trusted(rt, addr, trusted).await {
                        Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                        Err(e) => error_response(id, e.to_string()),
                    }
                }
                _ => {
                    CrawlResponse::error(id, error_code::INVALID_PARAMS, "address/trusted required")
                }
            },
            "BluetoothAlias" => match (
                params.get("address").and_then(|v| v.as_str()),
                params.get("alias").and_then(|v| v.as_str()),
            ) {
                (Some(addr), Some(alias)) => match bluetooth::set_alias(rt, addr, alias).await {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e.to_string()),
                },
                _ => CrawlResponse::error(id, error_code::INVALID_PARAMS, "address/alias required"),
            },
            "BluetoothDiscoverable" => match params.get("enabled").and_then(|v| v.as_bool()) {
                Some(on) => match bluetooth::set_discoverable(rt, on).await {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e.to_string()),
                },
                None => CrawlResponse::error(id, error_code::INVALID_PARAMS, "enabled required"),
            },
            "BluetoothPairable" => match params.get("enabled").and_then(|v| v.as_bool()) {
                Some(on) => match bluetooth::set_pairable(rt, on).await {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e.to_string()),
                },
                None => CrawlResponse::error(id, error_code::INVALID_PARAMS, "enabled required"),
            },
            _ => return None,
        };

        Some(response)
    }
}
