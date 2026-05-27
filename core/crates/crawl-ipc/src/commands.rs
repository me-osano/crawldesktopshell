use serde::{Deserialize, Serialize};

use crate::types::{
    AddAccount, CopyMessage, GetMessage, ListMessages, MoveMessage, RssListEntriesParams,
    SaveAttachment, Search, SendMessage, SetFlags, WallhavenSearchParams,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method", content = "params", rename_all = "PascalCase")]
pub enum CrawlCommand {
    // ── Meta ────
    /// Open a persistent event stream. Server pushes events for requested topics.
    /// Empty topics = subscribe to all.
    Subscribe {
        topics: Vec<String>,
    },

    Status,
    Health,
    DaemonPing,
    ServiceList,
    ServiceRegister {
        name: String,
    },
    ServiceUnregister {
        name: String,
    },

    // --- Audio ------
    AudioSinks,
    AudioSources,
    AudioVolume {
        percent: u32,
    },
    AudioInputVolume {
        percent: u32,
    },
    AudioMute,
    AudioUnmute,
    AudioMuteInput,
    AudioUnmuteInput,
    AudioSetDefaultSink {
        name: String,
    },
    AudioSetDefaultSource {
        name: String,
    },

    // --- Brightness ---
    BrightnessGet,
    BrightnessSet {
        value: f32,
    },
    BrightnessInc {
        value: f32,
    },
    BrightnessDec {
        value: f32,
    },

    // --- Bluetooth -----
    BluetoothStatus,
    BluetoothDevices,
    BluetoothScan {
        timeout: Option<u64>,
    },
    BluetoothScanStop,
    BluetoothConnect {
        address: String,
    },
    BluetoothDisconnect {
        address: String,
    },
    BluetoothPower {
        enabled: bool,
    },
    BluetoothPair {
        address: String,
    },
    BluetoothRemove {
        address: String,
    },
    BluetoothDiscoverable {
        enabled: bool,
    },
    BluetoothTrust {
        address: String,
        trusted: bool,
    },
    BluetoothAlias {
        address: String,
        alias: String,
    },
    BluetoothPairable {
        enabled: bool,
    },

    // --- Network -----
    NetworkStatus,
    NetworkEnable {
        enabled: bool,
    },
    WifiList,
    WifiScan,
    WifiDetails,
    WifiConnect {
        ssid: String,
        password: Option<String>,
    },
    WifiDisconnect,
    WifiForget {
        ssid: String,
    },
    EthernetList,
    EthernetDetails {
        iface: Option<String>,
    },
    EthernetConnect {
        iface: Option<String>,
    },
    EthernetDisconnect {
        iface: Option<String>,
    },
    HotspotStatus,
    HotspotStart {
        config: crate::types::HotspotConfig,
    },
    HotspotStop,

    // --- Sysmon -----
    SysmonCpu,
    SysmonMem,
    SysmonDisks,
    SysmonNet,
    SysmonGpu,

    // --- Process ---
    ProcList {
        sort_by: Option<String>,
        top: Option<u32>,
    },
    ProcFind {
        name: String,
    },
    ProcKill {
        pid: u32,
        force: Option<bool>,
    },
    ProcWatch {
        pid: u32,
    },

    // --- Theme ---
    ThemeGet,
    ThemeList,
    ThemeSet {
        name: String,
        accent: Option<String>,
    },
    ThemeGenerate {
        color: String,
        scheme: Option<String>,
        mode: Option<String>,
    },
    ThemeGenerateFromImage {
        path: String,
        scheme: Option<String>,
        mode: Option<String>,
    },
    ThemeGenerateFromPredefined {
        /// JSON string of the scheme data (cPrimary, cOnPrimary, etc.)
        scheme_json: String,
        mode: String,
    },
    ThemeGenerateTerminal {
        /// JSON string of the mode-specific scheme data (cPrimary + terminal section)
        scheme_json: String,
        /// JSON dict mapping terminal_id to output path
        outputs_json: String,
    },

    // --- Idle ---
    IdleInhibit,
    IdleUninhibit,
    IdleStatus,
    IdleInhibitWithTimeout {
        seconds: u64,
    },

    // --- Sysinfo ---
    Sysinfo,

    // --- Notifications ---
    NotificationGetState,
    NotificationDismiss {
        id: String,
    },
    NotificationDismissAll,
    NotificationRemoveHistory {
        id: String,
    },
    NotificationClearHistory,
    NotificationInvokeAction {
        id: String,
        action_id: String,
    },
    NotificationSetDnd {
        enabled: bool,
    },
    NotificationSetLastSeen {
        ts: u64,
    },
    NotificationGetPolicy,
    NotificationSetPolicy {
        policy: crate::types::NotificationPolicy,
    },
    NotificationSaveRules {
        rules_json: String,
    },
    NotificationGetRules,
    NotificationSetRules {
        rules_json: String,
    },
    // --- Mail -----
    ListAccounts,
    AddAccount(AddAccount),
    RemoveAccount {
        account_id: String,
    },
    ListFolders {
        account_id: String,
    },
    SelectFolder {
        account_id: String,
        folder: String,
    },
    ListMessages(ListMessages),
    GetMessage(GetMessage),
    SearchMessages(Search),
    SendMessage(SendMessage),
    MoveMessage(MoveMessage),
    CopyMessage(CopyMessage),
    DeleteMessage {
        account_id: String,
        folder: String,
        uid: u32,
    },
    SetFlags(SetFlags),
    SyncNow {
        account_id: String,
    },
    FetchBody {
        account_id: String,
        uid: u32,
    },
    SaveAttachment(SaveAttachment),

    // --- RSS -----
    RssListFeeds,
    RssAddFeed {
        url: String,
        category: Option<String>,
    },
    RssRemoveFeed {
        feed_id: String,
    },
    RssUpdateFeed {
        feed_id: String,
        category: Option<String>,
    },
    RssListEntries(RssListEntriesParams),
    RssGetEntry {
        entry_id: String,
    },
    RssSetEntryRead {
        entry_id: String,
        is_read: bool,
    },
    RssSetEntryStarred {
        entry_id: String,
        is_starred: bool,
    },
    RssMarkAllRead {
        feed_id: String,
    },
    RssRefreshFeed {
        feed_id: String,
    },
    RssRefreshAll,
    RssListCategories,
    RssImportOpml {
        path: String,
    },
    RssExportOpml,
    RssSetEnabled {
        enabled: bool,
    },

    // --- Clipboard -----
    ClipboardList,
    ClipboardGetContent {
        id: u64,
    },
    ClipboardCopy {
        id: u64,
    },
    ClipboardDelete {
        id: u64,
    },
    ClipboardWipe,
    ClipboardPin {
        id: u64,
        pinned: bool,
    },
    ClipboardPasteText {
        text: String,
    },
    ClipboardSet {
        text: String,
        mime: String,
    },

    // --- Wallhaven -----
    WallhavenSearch(WallhavenSearchParams),
    WallhavenDownload {
        wallpaper_id: String,
        url: String,
        dest_dir: String,
        filename: Option<String>,
    },
    WallhavenSetEnabled {
        enabled: bool,
    },
}
