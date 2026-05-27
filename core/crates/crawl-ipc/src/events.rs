use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// All events broadcast over the Unix socket JSON-RPC event stream.
/// Quickshell and CLI --watch consumers filter by domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "domain", content = "data", rename_all = "snake_case")]
pub enum CrawlEvent {
    Audio(AudioEvent),
    Bluetooth(BtEvent),
    Brightness(BrightnessEvent),
    Daemon(DaemonEvent),
    Idle(IdleEvent),
    Mail(MailEvent),
    Network(NetEvent),
    Notification(NotificationEvent),
    Proc(ProcEvent),
    Rss(RssEvent),
    Sysmon(SysmonEvent),
    Sysinfo(SysinfoEvent),
    Wallhaven(WallhavenEvent),
    Wallpaper(WallpaperEvent),
    Theme(ThemeEvent),
    Clipboard(ClipboardEvent),
}

// ---- Audio ---------
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum AudioEvent {
    VolumeChanged { device: AudioDevice },
    MuteToggled { device: AudioDevice },
    DefaultSinkChanged { device: AudioDevice },
    DefaultSourceChanged { device: AudioDevice },
    DeviceAdded { device: AudioDevice },
    DeviceRemoved { id: u32 },
}

// ---- Bluetooth ---------
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum BtEvent {
    DeviceDiscovered { device: BtDevice },
    DeviceConnected { device: BtDevice },
    DeviceDisconnected { device: BtDevice },
    DeviceRemoved { address: String },
    DeviceUpdated { device: BtDevice },
    AdapterPowered { on: bool },
    AdapterDiscoverable { on: bool },
    AdapterPairable { on: bool },
    ScanStarted,
    ScanStopped,
}

// ---- Network ---------
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum NetEvent {
    Connected { ssid: Option<String>, iface: String },
    Disconnected { iface: String },
    IpChanged { iface: String, ip: String },
    WifiEnabled,
    WifiDisabled,
    WifiScanStarted,
    WifiScanFinished,
    WifiListUpdated { networks: Vec<WifiNetwork> },
    ActiveWifiDetailsChanged { details: ActiveWifiDetails },
    EthernetInterfacesChanged { interfaces: Vec<EthernetInterface> },
    ActiveEthernetDetailsChanged { details: ActiveEthernetDetails },
    ModeChanged { mode: NetMode },
    ConnectivityChanged { state: String },
    HotspotStarted { status: HotspotStatus },
    HotspotStopped,
    HotspotStatusChanged { status: HotspotStatus },
    HotspotClientJoined { client: HotspotClient },
    HotspotClientLeft { mac: String },
}

// ---- Sysinfo -----
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum SysinfoEvent {
    Changed,
}

// ---- Daemon -------
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum DaemonEvent {
    Ready,
    Started,
    Stopping,
    DomainError { domain: String, message: String },
}

// ---- Idle ----
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum IdleEvent {
    InhibitionStarted,
    InhibitionStopped,
    StateChanged { inhibited: bool },
}

// ---- Brightness -----
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum BrightnessEvent {
    Changed { status: BrightnessStatus },
}

// ── Wallpaper ───────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum WallpaperEvent {
    Changed { screen: String, path: String },
    BackendChanged { backend: WallpaperBackend },
    BackendNotAvailable { backend: WallpaperBackend },
    Error { message: String },
}

// --- RSS ----
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum RssEvent {
    FeedAdded {
        feed_id: String,
        title: String,
        category: String,
    },
    FeedRemoved {
        feed_id: String,
    },
    EntryUpdated {
        feed_id: String,
        entry_id: String,
    },
    NewEntries {
        feed_id: String,
        count: u32,
    },
    SyncStarted {
        feed_id: Option<String>,
    },
    SyncComplete {
        feed_id: Option<String>,
    },
    SyncError {
        feed_id: String,
        error: String,
    },
    StateChanged {
        enabled: bool,
    },
}

// --- Wallhaven ----
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum WallhavenEvent {
    DownloadStarted {
        wallpaper_id: String,
        local_path: String,
    },
    DownloadProgress {
        wallpaper_id: String,
        bytes_downloaded: u64,
        total_bytes: u64,
    },
    DownloadComplete {
        wallpaper_id: String,
        local_path: String,
    },
    DownloadFailed {
        wallpaper_id: String,
        error: String,
    },
    StateChanged {
        enabled: bool,
    },
}

// ---- Theme ----
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ThemeEvent {
    Changed {
        name: String,
        source: String,
        scheme: String,
        /// Full palette map (26 Catppuccin-style named colors + "accent").
        /// Same format as ThemeChangedEvent.palette from crawl-theme crate.
        palette: HashMap<String, String>,
    },
    Applied {
        app: String,
        path: String,
    },
    Generated {
        name: String,
        color: String,
        path: Option<String>,
    },
}

// --- Sysmon ---
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum SysmonEvent {
    CpuUpdate { cpu: CpuStatus },
    MemUpdate { mem: MemStatus },
    NetUpdate { traffic: NetTraffic },
    GpuUpdate { gpu: GpuStatus },
    CpuSpike { usage: f32, threshold: f32 },
    MemPressure { used_percent: f32 },
}

// --- Processes ---
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ProcEvent {
    Spawned {
        pid: u32,
        name: String,
    },
    Exited {
        pid: u32,
        name: String,
        exit_code: Option<i32>,
    },
    TopUpdate {
        top_by_cpu: Vec<ProcessInfo>,
        top_by_mem: Vec<ProcessInfo>,
    },
}

// --- Mail ----
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum MailEvent {
    AccountAdded {
        account_id: String,
        display_name: String,
        email: String,
    },
    NewMessages {
        account_id: String,
        folder: String,
        count: u32,
    },
    FlagsUpdated {
        account_id: String,
        uid: u32,
        flags: Vec<MailFlag>,
    },
    SyncComplete {
        account_id: String,
    },
    SyncStatus {
        account_id: String,
        status: SyncStatusKind,
    },
    AttachmentSaved {
        account_id: String,
        uid: u32,
        attachment_id: String,
        dest_path: String,
    },
}

// ---- Clipboard ----
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum ClipboardEvent {
    Changed { entry: crate::types::ClipEntry },
    Deleted { id: u64 },
    Cleared,
    Pinned { id: u64, pinned: bool },
}

// ---- Notifications ----
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum NotificationEvent {
    PopupAdded { item: NotificationItem },
    PopupUpdated { item: NotificationItem },
    PopupRemoved { id: String },
    HistoryAdded { item: NotificationItem },
    HistoryRemoved { id: String },
    HistoryCleared,
    DndChanged { enabled: bool },
    LastSeenChanged { ts: u64 },
}
