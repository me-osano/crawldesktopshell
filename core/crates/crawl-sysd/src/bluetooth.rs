//! crawl-bluetooth: Bluetooth domain via BlueZ/bluer.
//!
//! Runs as a long-lived tokio task, publishing BtEvents to the broadcast channel.
//! All BlueZ communication is async via the bluer crate.

use crate::config::BluetoothConfig as Config;
use bluer::agent::Agent;
use bluer::{AdapterEvent, AdapterProperty, DeviceEvent, DeviceProperty};
use crawl_ipc::{
    events::{BtEvent, CrawlEvent},
    types::{BtDevice, BtStatus},
};
use futures_util::StreamExt;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{Mutex, broadcast, watch};
use tracing::{error, info, warn};

// ── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum BtError {
    #[error("BlueZ session error: {0}")]
    Session(#[from] bluer::Error),
    #[error("no default adapter found")]
    NoAdapter,
    #[error("invalid device address: {0}")]
    InvalidAddress(String),
}

#[derive(Clone)]
pub struct BluetoothRuntime {
    session: Arc<bluer::Session>,
    adapter: Arc<Mutex<Option<bluer::Adapter>>>,
    agent_handle: Arc<Mutex<Option<bluer::agent::AgentHandle>>>,
    scan_stop: Arc<Mutex<Option<watch::Sender<bool>>>>,
}

impl BluetoothRuntime {
    pub async fn new() -> Result<Self, BtError> {
        let session = bluer::Session::new().await?;
        Ok(Self {
            session: Arc::new(session),
            adapter: Arc::new(Mutex::new(None)),
            agent_handle: Arc::new(Mutex::new(None)),
            scan_stop: Arc::new(Mutex::new(None)),
        })
    }

    pub async fn unregister_agent(&self) {
        self.agent_handle.lock().await.take();
    }

    async fn adapter(&self) -> Result<bluer::Adapter, BtError> {
        {
            let guard = self.adapter.lock().await;
            if let Some(ref adapter) = *guard {
                if adapter.is_powered().await.is_ok() {
                    return Ok(adapter.clone());
                }
                info!("cached adapter stale, re-resolving");
            }
        }

        let adapter = self
            .session
            .default_adapter()
            .await
            .map_err(|_| BtError::NoAdapter)?;
        let mut guard = self.adapter.lock().await;
        *guard = Some(adapter.clone());
        Ok(adapter)
    }
}

// ── Domain runner ─────────────────────────────────────────────────────────────

/// Entry point called by crawl-daemon. Runs indefinitely, publishing events.
pub async fn run(
    runtime: Arc<BluetoothRuntime>,
    cfg: Config,
    tx: broadcast::Sender<CrawlEvent>,
) -> anyhow::Result<()> {
    info!("crawl-bluetooth starting");

    let adapter = runtime.adapter().await?;
    let agent_handle = register_agent(runtime.session.as_ref()).await?;
    *runtime.agent_handle.lock().await = Some(agent_handle);

    info!(adapter = %adapter.name(), "using Bluetooth adapter");

    if cfg.auto_enable && !adapter.is_powered().await? {
        adapter.set_powered(true).await?;
        info!("adapter powered on");
    }

    // Publish initial adapter state
    let powered = adapter.is_powered().await.unwrap_or(false);
    let discoverable = adapter.is_discoverable().await.unwrap_or(false);
    let pairable = adapter.is_pairable().await.unwrap_or(false);
    let _ = tx.send(CrawlEvent::Bluetooth(BtEvent::AdapterPowered {
        on: powered,
    }));
    let _ = tx.send(CrawlEvent::Bluetooth(BtEvent::AdapterDiscoverable {
        on: discoverable,
    }));
    let _ = tx.send(CrawlEvent::Bluetooth(BtEvent::AdapterPairable {
        on: pairable,
    }));

    let existing = adapter.device_addresses().await.unwrap_or_default();
    for addr in existing {
        if let Ok(dev) = adapter.device(addr) {
            let bt_dev = device_to_ipc(&dev).await;
            let _ = tx.send(CrawlEvent::Bluetooth(BtEvent::DeviceDiscovered {
                device: bt_dev,
            }));
            let tx2 = tx.clone();
            tokio::spawn(watch_device(dev, addr.to_string(), tx2));
        }
    }

    // Watch for adapter-level events (device added/removed + property changes)
    let mut adapter_events = adapter.events().await?;

    while let Some(event) = adapter_events.next().await {
        match event {
            AdapterEvent::DeviceAdded(addr) => {
                let dev_result = adapter.device(addr);
                match dev_result {
                    Ok(dev) => {
                        let bt_dev = device_to_ipc(&dev).await;
                        info!(address = %addr, name = ?bt_dev.name, "device discovered");
                        let _ = tx.send(CrawlEvent::Bluetooth(BtEvent::DeviceDiscovered {
                            device: bt_dev,
                        }));

                        // Watch device-level events in a subtask
                        let tx2 = tx.clone();
                        tokio::spawn(watch_device(dev, addr.to_string(), tx2));
                    }
                    Err(e) => warn!("failed to get device {addr}: {e}"),
                }
            }
            AdapterEvent::DeviceRemoved(addr) => {
                info!(address = %addr, "device removed");
                let _ = tx.send(CrawlEvent::Bluetooth(BtEvent::DeviceRemoved {
                    address: addr.to_string(),
                }));
            }
            AdapterEvent::PropertyChanged(prop) => {
                handle_adapter_property(&tx, prop).await;
            }
        }
    }

    Ok(())
}

async fn handle_adapter_property(tx: &broadcast::Sender<CrawlEvent>, prop: AdapterProperty) {
    match prop {
        AdapterProperty::Powered(on) => {
            let _ = tx.send(CrawlEvent::Bluetooth(BtEvent::AdapterPowered { on }));
        }
        AdapterProperty::Discoverable(on) => {
            let _ = tx.send(CrawlEvent::Bluetooth(BtEvent::AdapterDiscoverable { on }));
        }
        AdapterProperty::Pairable(on) => {
            let _ = tx.send(CrawlEvent::Bluetooth(BtEvent::AdapterPairable { on }));
        }
        AdapterProperty::Discovering(started) => {
            let evt = if started {
                BtEvent::ScanStarted
            } else {
                BtEvent::ScanStopped
            };
            let _ = tx.send(CrawlEvent::Bluetooth(evt));
        }
        _ => {}
    }
}

/// Watch property changes on a single Bluetooth device.
async fn watch_device(device: bluer::Device, _address: String, tx: broadcast::Sender<CrawlEvent>) {
    let mut events = match device.events().await {
        Ok(e) => e,
        Err(e) => {
            error!("device event stream failed: {e}");
            return;
        }
    };

    while let Some(event) = events.next().await {
        let DeviceEvent::PropertyChanged(prop) = event;
        match prop {
            DeviceProperty::Connected(connected) => {
                let bt_dev = device_to_ipc(&device).await;
                let evt = if connected {
                    BtEvent::DeviceConnected { device: bt_dev }
                } else {
                    BtEvent::DeviceDisconnected { device: bt_dev }
                };
                let _ = tx.send(CrawlEvent::Bluetooth(evt));
            }
            DeviceProperty::Rssi(_)
            | DeviceProperty::BatteryPercentage(_)
            | DeviceProperty::Paired(_)
            | DeviceProperty::Alias(_)
            | DeviceProperty::Name(_)
            | DeviceProperty::Icon(_) => {
                let bt_dev = device_to_ipc(&device).await;
                let _ = tx.send(CrawlEvent::Bluetooth(BtEvent::DeviceUpdated {
                    device: bt_dev,
                }));
            }
            _ => {}
        }
    }
}

/// Convert a bluer Device into the shared IPC BtDevice type.
async fn device_to_ipc(device: &bluer::Device) -> BtDevice {
    BtDevice {
        address: device.address().to_string(),
        name: device.name().await.ok().flatten(),
        connected: device.is_connected().await.unwrap_or(false),
        paired: device.is_paired().await.unwrap_or(false),
        rssi: device.rssi().await.ok().flatten(),
        battery: device.battery_percentage().await.ok().flatten(),
        icon: device.icon().await.ok().flatten(),
    }
}

// ── Public query API (called by crawl-daemon router) ─────────────────────────

pub async fn get_devices(runtime: &BluetoothRuntime) -> Result<Vec<BtDevice>, BtError> {
    let adapter = runtime.adapter().await?;
    let addrs = adapter.device_addresses().await?;

    let mut devices = Vec::new();
    for addr in addrs {
        let dev_result = adapter.device(addr);
        if let Ok(dev) = dev_result {
            devices.push(device_to_ipc(&dev).await);
        }
    }
    Ok(devices)
}

pub async fn get_status(runtime: &BluetoothRuntime) -> Result<BtStatus, BtError> {
    let adapter = runtime.adapter().await?;
    let powered = adapter.is_powered().await.unwrap_or(false);
    let discovering = adapter.is_discovering().await.unwrap_or(false);
    let devices = get_devices(runtime).await?;
    Ok(BtStatus {
        powered,
        discovering,
        devices,
    })
}

pub async fn scan(runtime: &BluetoothRuntime, timeout_secs: Option<u64>) -> Result<(), BtError> {
    // Stop any existing scan first
    stop_scan(runtime).await;

    let adapter = runtime.adapter().await?;

    let was_powered = adapter.is_powered().await.unwrap_or(false);
    adapter.set_powered(true).await?;

    let mut discovery = adapter.discover_devices().await?;

    let adapter_clone = adapter.clone();
    let (stop_tx, mut stop_rx) = watch::channel(false);

    *runtime.scan_stop.lock().await = Some(stop_tx);

    tokio::spawn(async move {
        let scan_result: Result<(), ()> = match timeout_secs {
            Some(0) | None => {
                // No timeout — run until explicitly stopped
                loop {
                    tokio::select! {
                        _ = discovery.next() => {}
                        _ = stop_rx.changed() => {
                            if *stop_rx.borrow_and_update() {
                                break;
                            }
                        }
                    }
                }
                Ok(())
            }
            Some(secs) => {
                let timeout = std::time::Duration::from_secs(secs);
                let result = tokio::time::timeout(timeout, async {
                    loop {
                        tokio::select! {
                            _ = discovery.next() => {}
                            _ = stop_rx.changed() => {
                                if *stop_rx.borrow_and_update() {
                                    break;
                                }
                            }
                        }
                    }
                })
                .await;
                result.map(|_| ()).map_err(|_| ())
            }
        };

        // Discovery stops when the `discovery` stream is dropped at task exit

        if !was_powered {
            let _ = adapter_clone.set_powered(false).await;
        }

        info!(
            reason = if scan_result.is_ok() {
                "finished"
            } else {
                "timed_out"
            },
            "bluetooth scan finished"
        );
    });

    Ok(())
}

pub async fn stop_scan(runtime: &BluetoothRuntime) {
    if let Some(tx) = runtime.scan_stop.lock().await.take() {
        let _ = tx.send(true);
        info!("bluetooth scan cancelled");
    }
}

pub async fn connect(runtime: &BluetoothRuntime, address: &str) -> Result<(), BtError> {
    let adapter = runtime.adapter().await?;
    let addr: bluer::Address = address
        .parse()
        .map_err(|_| BtError::InvalidAddress(address.to_string()))?;
    let device = adapter.device(addr)?;
    device.connect().await?;
    Ok(())
}

pub async fn disconnect(runtime: &BluetoothRuntime, address: &str) -> Result<(), BtError> {
    let adapter = runtime.adapter().await?;
    let addr: bluer::Address = address
        .parse()
        .map_err(|_| BtError::InvalidAddress(address.to_string()))?;
    let device = adapter.device(addr)?;
    device.disconnect().await?;
    Ok(())
}

pub async fn set_powered(runtime: &BluetoothRuntime, on: bool) -> Result<(), BtError> {
    let adapter = runtime.adapter().await?;
    adapter.set_powered(on).await?;
    Ok(())
}

pub async fn pair(runtime: &BluetoothRuntime, address: &str) -> Result<(), BtError> {
    let adapter = runtime.adapter().await?;
    let addr: bluer::Address = address
        .parse()
        .map_err(|_| BtError::InvalidAddress(address.to_string()))?;
    let device = adapter.device(addr)?;
    device.pair().await?;
    Ok(())
}

pub async fn set_trusted(
    runtime: &BluetoothRuntime,
    address: &str,
    trusted: bool,
) -> Result<(), BtError> {
    let adapter = runtime.adapter().await?;
    let addr: bluer::Address = address
        .parse()
        .map_err(|_| BtError::InvalidAddress(address.to_string()))?;
    let device = adapter.device(addr)?;
    device.set_trusted(trusted).await?;
    Ok(())
}

pub async fn remove_device(runtime: &BluetoothRuntime, address: &str) -> Result<(), BtError> {
    let adapter = runtime.adapter().await?;
    let addr: bluer::Address = address
        .parse()
        .map_err(|_| BtError::InvalidAddress(address.to_string()))?;
    adapter.remove_device(addr).await?;
    Ok(())
}

pub async fn set_alias(
    runtime: &BluetoothRuntime,
    address: &str,
    alias: &str,
) -> Result<(), BtError> {
    let adapter = runtime.adapter().await?;
    let addr: bluer::Address = address
        .parse()
        .map_err(|_| BtError::InvalidAddress(address.to_string()))?;
    let device = adapter.device(addr)?;
    device.set_alias(alias.to_string()).await?;
    Ok(())
}

pub async fn set_discoverable(runtime: &BluetoothRuntime, on: bool) -> Result<(), BtError> {
    let adapter = runtime.adapter().await?;
    adapter.set_discoverable(on).await?;
    Ok(())
}

pub async fn set_pairable(runtime: &BluetoothRuntime, on: bool) -> Result<(), BtError> {
    let adapter = runtime.adapter().await?;
    adapter.set_pairable(on).await?;
    Ok(())
}

async fn register_agent(session: &bluer::Session) -> Result<bluer::agent::AgentHandle, BtError> {
    let agent = Agent {
        request_default: true,
        // With request_confirmation + request_authorization + authorize_service
        // all set, bluer derives capability "DisplayYesNo". All pairing requests
        // are auto-confirmed — appropriate since pairing is user-initiated via CLI/UI.
        request_confirmation: Some(Box::new(|_req| Box::pin(async { Ok(()) }))),
        request_authorization: Some(Box::new(|_req| Box::pin(async { Ok(()) }))),
        authorize_service: Some(Box::new(|_req| Box::pin(async { Ok(()) }))),
        request_pin_code: None,
        display_pin_code: None,
        request_passkey: None,
        display_passkey: None,
        _non_exhaustive: (),
    };
    let handle = session.register_agent(agent).await?;
    Ok(handle)
}
