use serde::{Deserialize, Serialize};

// === System Information ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub compositor: CompositorInfo,
    pub os: OsInfo,
    pub session: SessionInfo,
    pub hardware: HardwareInfo,
    pub display: DisplayInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositorInfo {
    #[serde(rename = "type")]
    pub compositor_type: String,
    pub name: String,
    pub capabilities: CompositorCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositorCapabilities {
    pub layer_shell: bool,
    pub blur: bool,
    pub screencopy: bool,
    pub wallpaper_control: bool,
    pub dpms: bool,
    pub socket_ipc: bool,
    pub http_ipc: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub kernel: String,
    pub pretty_name: String,
    pub hostname: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    #[serde(rename = "type")]
    pub session_type: String,
    pub user: String,
    pub seat: Option<String>,
    pub home: String,
    pub shell: Option<String>,
    pub terminal: Option<String>,
    pub uptime_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu_model: String,
    pub cpu_cores: usize,
    pub memory_total: u64,
    pub gpu: Option<String>,
    pub disk_total: Option<u64>,
    pub disk_used: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayInfo {
    pub monitors: Vec<MonitorInfo>,
    pub scales: std::collections::HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub name: String,
    pub scale: f32,
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    pub refresh_rate: f32,
    pub focused: bool,
    pub active: bool,
}

// === Audio (PipeWire/PulseAudio) ===

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AudioDeviceKind {
    Sink,
    Source,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDevice {
    pub id: u32,
    pub name: String,
    pub description: Option<String>,
    pub kind: AudioDeviceKind,
    pub volume_percent: u32,
    pub muted: bool,
    pub is_default: bool,
}

// === Bluetooth ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtDevice {
    pub address: String,
    pub name: Option<String>,
    pub connected: bool,
    pub paired: bool,
    pub rssi: Option<i16>,
    pub battery: Option<u8>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtStatus {
    pub powered: bool,
    pub discovering: bool,
    pub devices: Vec<BtDevice>,
}

// === Brightness ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrightnessStatus {
    pub device: String,
    pub current: u64,
    pub max: u64,
    pub percent: f32,
}

// === Network ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetInterface {
    pub name: String,
    pub state: String,
    pub ip4: Option<String>,
    pub ip6: Vec<String>,
    pub mac: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: u8,
    pub secured: bool,
    pub connected: bool,
    pub existing: bool,
    pub cached: bool,
    pub password_required: bool,
    pub security: String,
    pub frequency_mhz: Option<u32>,
    pub bssid: Option<String>,
    pub last_seen_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveWifiDetails {
    pub ifname: Option<String>,
    pub ssid: Option<String>,
    pub signal: Option<u8>,
    pub frequency_mhz: Option<u32>,
    pub band: Option<String>,
    pub channel: Option<u32>,
    pub rate_mbps: Option<u32>,
    pub ip4: Option<String>,
    pub ip6: Vec<String>,
    pub gateway4: Option<String>,
    pub gateway6: Vec<String>,
    pub dns4: Vec<String>,
    pub dns6: Vec<String>,
    pub security: Option<String>,
    pub bssid: Option<String>,
    pub mac: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EthernetInterface {
    pub ifname: String,
    pub connected: bool,
    pub mac: Option<String>,
    pub ip4: Option<String>,
    pub ip6: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveEthernetDetails {
    pub ifname: String,
    pub speed: Option<String>,
    pub ipv4: Option<String>,
    pub ipv6: Vec<String>,
    pub gateway4: Option<String>,
    pub gateway6: Vec<String>,
    pub dns4: Vec<String>,
    pub dns6: Vec<String>,
    pub mac: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum NetMode {
    Station,
    Ap,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetStatus {
    pub connectivity: String,
    pub wifi_enabled: bool,
    pub network_enabled: bool,
    pub wifi_available: bool,
    pub ethernet_available: bool,
    pub mode: NetMode,
    pub active_ssid: Option<String>,
    pub interfaces: Vec<NetInterface>,
}

// ── Hotspot ─────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HotspotBackend {
    NetworkManager,
    Hostapd,
}

impl Default for HotspotBackend {
    fn default() -> Self {
        Self::NetworkManager
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotConfig {
    pub ssid: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub iface: Option<String>,
    #[serde(default)]
    pub band: Option<String>,
    #[serde(default)]
    pub channel: Option<u32>,
    #[serde(default)]
    pub backend: Option<HotspotBackend>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotspotClient {
    pub mac: String,
    pub ip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HotspotStatus {
    pub active: bool,
    pub ssid: Option<String>,
    pub iface: Option<String>,
    pub band: Option<String>,
    pub channel: Option<u32>,
    pub clients: Vec<HotspotClient>,
    #[serde(default)]
    pub backend: HotspotBackend,
    pub supports_virtual_ap: bool,
}

// === Sysmon ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuStatus {
    pub aggregate: f32,
    pub cores: Vec<f32>,
    pub frequency_mhz: Vec<u64>,
    pub load_avg: LoadAvg,
    pub temperature_c: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadAvg {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemStatus {
    pub total_kb: u64,
    pub used_kb: u64,
    pub available_kb: u64,
    pub swap_total_kb: u64,
    pub swap_used_kb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetTraffic {
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_bps: u64,
    pub tx_bps: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuStatus {
    pub name: Option<String>,
    pub temperature_c: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskStatus {
    pub mount: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub filesystem: Option<String>,
}

// === Processes ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub ppid: Option<u32>,
    pub name: String,
    pub exe_path: Option<String>,
    pub cpu_percent: f32,
    pub cpu_ticks: Option<f64>,
    pub mem_rss_kb: u64,
    pub status: String,
    pub user: Option<String>,
    pub cmd: Vec<String>,
}

// === Notifications ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    pub identifier: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationItem {
    pub id: String,
    pub summary: String,
    pub summary_markdown: String,
    pub body: String,
    pub body_markdown: String,
    pub app_name: String,
    pub urgency: u8,
    pub expire_timeout: i32,
    pub timestamp_ms: u64,
    pub progress: f32,
    pub original_image: String,
    pub cached_image: String,
    pub actions: Vec<NotificationAction>,
    pub original_id: u32,
    pub muted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationState {
    pub popups: Vec<NotificationItem>,
    pub history: Vec<NotificationItem>,
    pub do_not_disturb: bool,
    pub last_seen_ts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationHistoryPolicy {
    pub low: bool,
    pub normal: bool,
    pub critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPolicy {
    pub enabled: bool,
    pub max_popups: usize,
    pub max_history: usize,
    pub respect_expire_timeout: bool,
    pub low_urgency_duration_ms: u64,
    pub normal_urgency_duration_ms: u64,
    pub critical_urgency_duration_ms: u64,
    pub save_to_history: NotificationHistoryPolicy,
}

// === Mail ===
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountInfo {
    pub id: String,
    pub display_name: String,
    pub email: String,
    pub unread_count: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddAccount {
    pub display_name: String,
    pub email: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListMessages {
    pub account_id: String,
    pub folder: String,
    pub offset: u32,
    pub limit: u32,
    pub sort: MailSortOrder,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetMessage {
    pub account_id: String,
    pub uid: u32,
    pub fetch_remote: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Search {
    pub account_id: String,
    pub query: String,
    pub folder: Option<String>,
    pub limit: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttachmentRef {
    pub path: String,
    pub mime_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SendMessage {
    pub account_id: String,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub attachments: Vec<AttachmentRef>,
    pub in_reply_to: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveMessage {
    pub account_id: String,
    pub uid: u32,
    pub from_folder: String,
    pub to_folder: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CopyMessage {
    pub account_id: String,
    pub uid: u32,
    pub to_folder: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SetFlags {
    pub account_id: String,
    pub folder: String,
    pub uid: u32,
    pub add: Vec<MailFlag>,
    pub remove: Vec<MailFlag>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SaveAttachment {
    pub account_id: String,
    pub uid: u32,
    pub attachment_id: String,
    pub dest_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttachmentInfo {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: u64,
    pub cached: bool,
}

#[derive(Debug, Serialize)]
pub struct FolderInfo {
    pub name: String,
    pub display_name: String,
    pub unread: u32,
    pub total: u32,
    pub kind: FolderKind,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FolderKind {
    Inbox,
    Sent,
    Drafts,
    Trash,
    Spam,
    Archive,
    Custom,
}

#[derive(Debug, Serialize)]
pub struct MessageSummary {
    pub uid: u32,
    pub account_id: String,
    pub folder: String,
    pub from: String,
    pub subject: String,
    pub date: String, // ISO 8601
    pub flags: Vec<MailFlag>,
    pub has_attachments: bool,
    pub snippet: String, // first ~120 chars of body
}

#[derive(Debug, Serialize)]
pub struct MessageFull {
    pub uid: u32,
    pub account_id: String,
    pub folder: String,
    pub message_id: String,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub subject: String,
    pub date: String,
    pub flags: Vec<MailFlag>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub attachments: Vec<AttachmentInfo>,
    pub thread_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MailSortOrder {
    DateDesc,
    DateAsc,
    SenderAsc,
    SubjectAsc,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MailFlag {
    Seen,
    Answered,
    Flagged,
    Deleted,
    Draft,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatusKind {
    Running,
    Idle,
    Error,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MailErrorCode {
    AuthFailed,
    NetworkError,
    NotFound,
    InvalidParams,
    ImapError,
    SmtpError,
    DbError,
    Unknown,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum MailResponse {
    Ok,
    Error {
        code: MailErrorCode,
        message: String,
    },

    AccountList {
        accounts: Vec<AccountInfo>,
    },
    FolderList {
        folders: Vec<FolderInfo>,
    },
    MessageList {
        messages: Vec<MessageSummary>,
        total: u32,
    },
    Message {
        message: MessageFull,
    },
    SearchResults {
        messages: Vec<MessageSummary>,
    },
    SyncStatus {
        account_id: String,
        status: SyncStatusKind,
    },
    SendQueued {
        queue_id: String,
    },

    // Push events (unsolicited — daemon → frontend)
    NewMessages {
        account_id: String,
        folder: String,
        count: u32,
    },
    SyncComplete {
        account_id: String,
    },
    FlagChanged {
        account_id: String,
        uid: u32,
        flags: Vec<MailFlag>,
    },
}

// (Theme types removed — all theme data now lives in crawl-theme crate)

// === RSS ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RssListEntriesParams {
    pub feed_id: Option<String>,
    pub category: Option<String>,
    pub offset: u32,
    pub limit: u32,
    pub only_unread: bool,
    pub only_starred: bool,
    pub sort: RssSortOrder,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RssSortOrder {
    NewestFirst,
    OldestFirst,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedInfo {
    pub id: String,
    pub url: String,
    pub title: String,
    pub description: String,
    pub site_url: String,
    pub icon_url: String,
    pub category: String,
    pub unread_count: u32,
    pub error_count: u32,
    pub last_error: String,
    pub last_fetched: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryInfo {
    pub id: String,
    pub feed_id: String,
    pub feed_title: String,
    pub title: String,
    pub author: String,
    pub summary: String,
    pub published: String,
    pub is_read: bool,
    pub is_starred: bool,
    pub image_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryFull {
    pub id: String,
    pub feed_id: String,
    pub feed_title: String,
    pub url: String,
    pub title: String,
    pub author: String,
    pub summary: String,
    pub content: String,
    pub content_type: String,
    pub published: String,
    pub is_read: bool,
    pub is_starred: bool,
    pub image_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RssErrorCode {
    FetchFailed,
    ParseFailed,
    NotFound,
    InvalidUrl,
    Duplicate,
    DbError,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum RssResponse {
    Ok,
    Error {
        code: RssErrorCode,
        message: String,
    },
    FeedList {
        feeds: Vec<FeedInfo>,
    },
    EntryList {
        entries: Vec<EntryInfo>,
        total: u32,
    },
    Entry {
        entry: EntryFull,
    },
    Categories {
        categories: Vec<String>,
    },
    ImportResult {
        imported: u32,
        failed: u32,
        total: u32,
    },
    ExportData {
        opml: String,
    },
}

// === Wallhaven ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallhavenSearchParams {
    pub query: String,
    pub categories: String,
    pub purity: String,
    pub sorting: String,
    pub order: String,
    pub page: u32,
    pub seed: Option<String>,
    pub top_range: Option<String>,
    pub atleast: Option<String>,
    pub resolutions: Option<String>,
    pub ratios: Option<String>,
    pub colors: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallhavenThumbs {
    pub small: Option<String>,
    pub large: Option<String>,
    pub original: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallhavenWallpaper {
    pub id: String,
    pub url: String,
    pub path: Option<String>,
    pub thumbs: Option<WallhavenThumbs>,
    pub resolution: Option<String>,
    pub tags: Vec<WallhavenTag>,
    pub purity: Option<String>,
    pub created_at: Option<String>,
    pub category: Option<String>,
    pub colors: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallhavenTag {
    pub id: Option<i64>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallhavenMeta {
    pub current_page: u32,
    pub last_page: u32,
    pub per_page: u32,
    pub total: u32,
    pub seed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum WallhavenResponse {
    Ok,
    Error {
        code: String,
        message: String,
    },
    SearchResults {
        results: Vec<WallhavenWallpaper>,
        meta: WallhavenMeta,
    },
    DownloadStarted {
        wallpaper_id: String,
        local_path: String,
    },
    DownloadComplete {
        wallpaper_id: String,
        local_path: String,
    },
}

// === Idle ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdleStatus {
    pub inhibited: bool,
    pub reason: String,
}

// ── Wallpaper ─────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WallpaperBackend {
    Swaybg,
    Hyprpaper,
    Wbg,
    #[serde(other)]
    Other,
}

impl Default for WallpaperBackend {
    fn default() -> Self {
        Self::Swaybg
    }
}
