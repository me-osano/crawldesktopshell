//! Network state tracking and diff logic.

use crawl_ipc::events::NetEvent;
use crawl_ipc::types::NetStatus;

#[derive(Clone)]
pub struct NetworkSnapshot {
    pub status: NetStatus,
}

pub struct NetworkState {
    last: Option<NetworkSnapshot>,
    last_connected_ssid: Option<String>,
    last_connected_iface: Option<String>,
}

impl NetworkState {
    pub fn new() -> Self {
        Self {
            last: None,
            last_connected_ssid: None,
            last_connected_iface: None,
        }
    }

    pub fn set_snapshot(&mut self, snapshot: NetworkSnapshot) {
        self.last_connected_ssid = snapshot.status.active_ssid.clone();
        self.last_connected_iface = snapshot
            .status
            .interfaces
            .iter()
            .find(|i| i.state == "activated" || i.state == "ip-config" || i.state == "ip-check")
            .map(|i| i.name.clone())
            .or_else(|| {
                // Fall back to first non-disconnected interface
                snapshot.status.interfaces.first().map(|i| i.name.clone())
            });
        self.last = Some(snapshot);
    }

    pub fn diff_events(&self, snapshot: &NetworkSnapshot) -> Vec<NetEvent> {
        let mut events = Vec::new();
        let status = &snapshot.status;

        let active_iface = status.interfaces.first().map(|i| i.name.clone());
        let active_ssid = status.active_ssid.clone();

        if let Some(prev) = &self.last {
            let prev_status = &prev.status;
            if prev_status.connectivity != status.connectivity {
                events.push(NetEvent::ConnectivityChanged {
                    state: status.connectivity.clone(),
                });
            }
            if prev_status.wifi_enabled != status.wifi_enabled {
                events.push(if status.wifi_enabled {
                    NetEvent::WifiEnabled
                } else {
                    NetEvent::WifiDisabled
                });
            }
            if prev_status.mode != status.mode {
                events.push(NetEvent::ModeChanged {
                    mode: status.mode.clone(),
                });
            }
        } else {
            events.push(NetEvent::ConnectivityChanged {
                state: status.connectivity.clone(),
            });
            events.push(if status.wifi_enabled {
                NetEvent::WifiEnabled
            } else {
                NetEvent::WifiDisabled
            });
            events.push(NetEvent::ModeChanged {
                mode: status.mode.clone(),
            });
        }

        if active_ssid != self.last_connected_ssid || active_iface != self.last_connected_iface {
            match (active_ssid, active_iface) {
                (Some(ssid), Some(iface)) => {
                    events.push(NetEvent::Connected {
                        ssid: Some(ssid),
                        iface,
                    });
                }
                (None, Some(iface)) => {
                    events.push(NetEvent::Disconnected { iface });
                }
                _ => {}
            }
        }

        events
    }
}

impl Default for NetworkState {
    fn default() -> Self {
        Self::new()
    }
}
