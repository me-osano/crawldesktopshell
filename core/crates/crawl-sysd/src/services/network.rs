use async_trait::async_trait;
use tracing::{error, info};

use crawl_ipc::error_code;
use crawl_ipc::protocol::CrawlResponse;

use crate::network;
use crate::services::models::{Service, error_response};
use crate::state::SharedState;

pub struct NetworkService {
    state: SharedState,
}

impl NetworkService {
    pub fn new(state: SharedState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl Service for NetworkService {
    fn name(&self) -> &'static str {
        "network"
    }

    async fn start(&self) -> anyhow::Result<()> {
        info!("Starting network service");
        let cfg = self.state.config.network.clone();
        let tx = self.state.event_bus.sender();

        tokio::spawn(async move {
            if let Err(e) = network::run(cfg, tx).await {
                error!(domain = "network", "Network service failed: {e:#}");
            }
        });

        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        info!("Stopping network service");
        Ok(())
    }

    async fn handle(
        &self,
        method: &str,
        params: &serde_json::Value,
        id: Option<serde_json::Value>,
    ) -> Option<CrawlResponse> {
        let response = match method {
            "NetworkStatus" => match network::get_status().await {
                Ok(status) => {
                    CrawlResponse::success(id, serde_json::to_value(status).unwrap_or_default())
                }
                Err(e) => error_response(id, e.to_string()),
            },
            "WifiList" => match network::list_wifi().await {
                Ok(networks) => {
                    CrawlResponse::success(id, serde_json::to_value(networks).unwrap_or_default())
                }
                Err(e) => error_response(id, e.to_string()),
            },
            "WifiScan" => match network::scan_wifi().await {
                Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                Err(e) => error_response(id, e.to_string()),
            },
            "WifiDetails" => match network::get_wifi_details().await {
                Ok(details) => {
                    CrawlResponse::success(id, serde_json::to_value(details).unwrap_or_default())
                }
                Err(e) => error_response(id, e.to_string()),
            },
            "WifiConnect" => match params.get("ssid").and_then(|v| v.as_str()) {
                Some(ssid) => {
                    let password = params.get("password").and_then(|v| v.as_str());
                    let security_key = params.get("security_key").and_then(|v| v.as_str());
                    let is_hidden = params
                        .get("is_hidden")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let identity = params.get("identity").and_then(|v| v.as_str());
                    let eap = params.get("eap").and_then(|v| v.as_str());
                    let phase2 = params.get("phase2").and_then(|v| v.as_str());
                    let anon_identity = params.get("anon_identity").and_then(|v| v.as_str());
                    let ca_cert = params.get("ca_cert").and_then(|v| v.as_str());
                    let cfg = network::WifiConnectConfig {
                        ssid: ssid.to_string(),
                        password: password.map(|s| s.to_string()),
                        security_key: security_key.map(|s| s.to_string()),
                        is_hidden,
                        identity: identity.map(|s| s.to_string()),
                        eap: eap.map(|s| s.to_string()),
                        phase2: phase2.map(|s| s.to_string()),
                        anon_identity: anon_identity.map(|s| s.to_string()),
                        ca_cert: ca_cert.map(|s| s.to_string()),
                    };
                    match network::connect_wifi(cfg).await {
                        Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                        Err(e) => error_response(id, e.to_string()),
                    }
                }
                None => CrawlResponse::error(id, error_code::INVALID_PARAMS, "ssid required"),
            },
            "WifiDisconnect" => match network::disconnect_wifi().await {
                Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                Err(e) => error_response(id, e.to_string()),
            },
            "WifiForget" => match params.get("ssid").and_then(|v| v.as_str()) {
                Some(ssid) => match network::delete_wifi_connection(ssid).await {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e.to_string()),
                },
                None => CrawlResponse::error(id, error_code::INVALID_PARAMS, "ssid required"),
            },
            "EthernetList" => match network::list_ethernet().await {
                Ok(interfaces) => {
                    CrawlResponse::success(id, serde_json::to_value(interfaces).unwrap_or_default())
                }
                Err(e) => error_response(id, e.to_string()),
            },
            "EthernetDetails" => {
                let iface = params.get("iface").and_then(|v| v.as_str());
                match network::get_ethernet_details(iface).await {
                    Ok(details) => CrawlResponse::success(
                        id,
                        serde_json::to_value(details).unwrap_or_default(),
                    ),
                    Err(e) => error_response(id, e.to_string()),
                }
            }
            "EthernetConnect" => {
                let iface = params.get("iface").and_then(|v| v.as_str());
                match network::connect_ethernet(iface).await {
                    Ok(ifname) => CrawlResponse::success(id, serde_json::json!({"ifname": ifname})),
                    Err(e) => error_response(id, e.to_string()),
                }
            }
            "EthernetDisconnect" => {
                let iface = params.get("iface").and_then(|v| v.as_str());
                match network::disconnect_ethernet(iface).await {
                    Ok(ifname) => CrawlResponse::success(id, serde_json::json!({"ifname": ifname})),
                    Err(e) => error_response(id, e.to_string()),
                }
            }
            "NetworkEnable" => match params.get("enabled").and_then(|v| v.as_bool()) {
                Some(enabled) => match network::set_network_enabled(enabled).await {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e.to_string()),
                },
                None => CrawlResponse::error(id, error_code::INVALID_PARAMS, "enabled required"),
            },
            "HotspotStatus" => match network::hotspot_status(None).await {
                Ok(status) => {
                    CrawlResponse::success(id, serde_json::to_value(status).unwrap_or_default())
                }
                Err(e) => error_response(id, e.to_string()),
            },
            "HotspotStart" => match params.get("config") {
                Some(config_value) => {
                    match serde_json::from_value::<crawl_ipc::types::HotspotConfig>(
                        config_value.clone(),
                    ) {
                        Ok(mut config) => {
                            if config.backend.is_none() {
                                config.backend = self.state.config.network.hotspot_backend.clone();
                            }
                            let use_virtual_iface = self.state.config.network.hotspot_virtual_iface;
                            match network::start_hotspot(&config, use_virtual_iface).await {
                                Ok(status) => CrawlResponse::success(
                                    id,
                                    serde_json::to_value(status).unwrap_or_default(),
                                ),
                                Err(e) => error_response(id, e.to_string()),
                            }
                        }
                        Err(e) => error_response(id, e.to_string()),
                    }
                }
                None => CrawlResponse::error(id, error_code::INVALID_PARAMS, "config required"),
            },
            "HotspotStop" => match network::stop_hotspot().await {
                Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                Err(e) => error_response(id, e.to_string()),
            },
            _ => return None,
        };

        Some(response)
    }
}
