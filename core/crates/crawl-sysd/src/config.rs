use crawl_ipc::types::HotspotBackend;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Toml},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
// Sysd Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub config_path: PathBuf,
    pub daemon: DaemonConfig,
    pub audio: AudioConfig,
    pub bluetooth: BluetoothConfig,
    pub brightness: BrightnessConfig,
    pub network: NetworkConfig,
    pub processes: ProcConfig,
    pub sysmon: SysmonConfig,
    pub theme: ThemeConfig,
    pub notifications: NotificationsConfig,
}

impl Config {
    /// Validate the configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if !(0.0..=100.0).contains(&self.sysmon.cpu_spike_threshold) {
            anyhow::bail!("sysmon.cpu_spike_threshold must be 0.0-100.0");
        }
        if !(0.0..=100.0).contains(&self.sysmon.mem_pressure_threshold) {
            anyhow::bail!("sysmon.mem_pressure_threshold must be 0.0-100.0");
        }
        if self.sysmon.poll_interval_ms == 0 {
            anyhow::bail!("sysmon.poll_interval_ms must be > 0");
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            config_path: PathBuf::new(),
            daemon: DaemonConfig::default(),
            audio: AudioConfig::default(),
            bluetooth: BluetoothConfig::default(),
            brightness: BrightnessConfig::default(),
            network: NetworkConfig::default(),
            processes: ProcConfig::default(),
            sysmon: SysmonConfig::default(),
            theme: ThemeConfig::default(),
            notifications: NotificationsConfig::default(),
        }
    }
}

// Load config from /etc/crawl/config.toml (optional), env vars, or defaults
pub fn load() -> anyhow::Result<Config> {
    let config_path = PathBuf::from("/etc/crawl/config.toml");

    let mut figment = Figment::from(Serialized::defaults(Config::default()));
    if config_path.exists() {
        figment = figment.merge(Toml::file(&config_path));
    }
    let config: Config = figment
        .merge(Env::prefixed("CRAWL_").split("__"))
        .extract()?;

    // Validate config
    config.validate()?;

    Ok(Config {
        config_path,
        ..config
    })
}

// === Daemon configuration types ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub log_level: String,
    pub socket_path: String,
    /// Child daemon routes: method prefixes → socket paths.
    /// Commands not handled locally are forwarded to the matching child.
    #[serde(default = "default_child_daemons")]
    pub child_daemons: Vec<ChildDaemonConfig>,
}

fn default_child_daemons() -> Vec<ChildDaemonConfig> {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));
    vec![
        ChildDaemonConfig {
            name: "webservice".into(),
            socket_path: format!("{runtime_dir}/crawl-webservice.sock"),
            method_prefixes: vec!["Rss".into(), "Wallhaven".into()],
        },
        ChildDaemonConfig {
            name: "mail".into(),
            socket_path: format!("{runtime_dir}/crawl-mail.sock"),
            method_prefixes: vec![
                "ListAccounts".into(),
                "AddAccount".into(),
                "RemoveAccount".into(),
                "ListFolders".into(),
                "SelectFolder".into(),
                "ListMessages".into(),
                "GetMessage".into(),
                "SearchMessages".into(),
                "SendMessage".into(),
                "MoveMessage".into(),
                "CopyMessage".into(),
                "DeleteMessage".into(),
                "SetFlags".into(),
                "SyncNow".into(),
                "FetchBody".into(),
                "SaveAttachment".into(),
            ],
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChildDaemonConfig {
    pub name: String,
    pub socket_path: String,
    pub method_prefixes: Vec<String>,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));
        Self {
            log_level: "info".into(),
            socket_path: format!("{runtime_dir}/crawl.sock"),
            child_daemons: default_child_daemons(),
        }
    }
}

// === Audio configuration types ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// PulseAudio server address. Empty = default (respects PULSE_SERVER env)
    pub server: String,
    /// Application name reported to PulseAudio
    pub app_name: String,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            server: String::new(),
            app_name: "crawl".into(),
        }
    }
}

// === Bluetooth configuration types ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothConfig {
    /// Power the adapter on startup if true
    pub auto_enable: bool,
    /// Scan timeout in seconds (0 = no timeout)
    #[serde(alias = "scan_timeout_sec")]
    pub scan_timeout_secs: u64,
}

impl Default for BluetoothConfig {
    fn default() -> Self {
        Self {
            auto_enable: false,
            scan_timeout_secs: 30,
        }
    }
}

// === Brightness Configuration types ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrightnessConfig {
    #[serde(default)]
    pub min_percent: f32,
    #[serde(default)]
    pub max_percent: f32,
    #[serde(default)]
    pub device: String,
}

impl Default for BrightnessConfig {
    fn default() -> Self {
        Self {
            min_percent: 1.0,
            max_percent: 100.0,
            device: String::new(),
        }
    }
}

// === Network configuration types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Power the adapter on startup if true
    pub auto_enable: bool,
    ///
    pub wifi_scan_on_start: bool,
    ///
    pub wifi_scan_finish_delay_ms: u64,
    ///
    #[serde(default)]
    pub hotspot_backend: Option<HotspotBackend>,
    #[serde(default = "default_true")]
    pub hotspot_virtual_iface: bool,
}

fn default_true() -> bool {
    true
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            auto_enable: false,
            wifi_scan_on_start: false,
            wifi_scan_finish_delay_ms: 30000,
            hotspot_backend: Some(HotspotBackend::default()),
            hotspot_virtual_iface: true,
        }
    }
}

// === Processes configuration types ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcConfig {
    /// Default sort field: cpu | mem | pid | name
    #[serde(default = "default_sort")]
    pub sort_by: String,
    /// Default number of top processes to return
    pub top: usize,
    /// Include command line in process info (expensive)
    pub include_cmd: bool,
    /// Interval for top-N tracking in ms
    pub top_interval_ms: u64,
    /// Interval for full scan in ms
    pub full_interval_ms: u64,
}

fn default_sort() -> String {
    "cpu".to_string()
}

impl Default for ProcConfig {
    fn default() -> Self {
        Self {
            sort_by: "cpu".into(),
            top: 20,
            include_cmd: false,
            top_interval_ms: 1000,
            full_interval_ms: 5000,
        }
    }
}

// === Sysmon configuration types ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysmonConfig {
    /// Polling interval in milliseconds
    pub poll_interval_ms: u64,
    /// Publish a CpuSpike event when aggregate exceeds this percent
    pub cpu_spike_threshold: f32,
    /// Publish a MemPressure event when usage exceeds this percent
    pub mem_pressure_threshold: f32,
    /// Minimum change in CPU % to trigger update
    pub cpu_change_threshold: f32,
    /// Minimum change in memory % to trigger update
    pub mem_change_threshold: f32,
    /// Minimum change in network bytes to trigger update
    pub net_change_threshold: u64,
}

// === Theme configuration types ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub cache_dir: String,
    /// Path to the colors.json file that QML Theme.qml watches.
    /// Defaults to $XDG_CONFIG_HOME/crawlds/colors.json.
    #[serde(default = "default_qml_colors_path")]
    pub qml_colors_path: String,
    #[serde(default)]
    pub theme: crawl_theme::ThemeConfig,
}

fn default_qml_colors_path() -> String {
    let config_dir = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
        format!("{}/.config", home)
    });
    format!("{}/crawlds/colors.json", config_dir)
}

impl Default for ThemeConfig {
    fn default() -> Self {
        let cache_dir = std::env::var("XDG_CACHE_HOME").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
            format!("{}/.cache", home)
        });
        Self {
            cache_dir: format!("{}/crawl", cache_dir),
            qml_colors_path: default_qml_colors_path(),
            theme: crawl_theme::ThemeConfig::default(),
        }
    }
}

impl Default for SysmonConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 1000,
            cpu_spike_threshold: 90.0,
            mem_pressure_threshold: 85.0,
            cpu_change_threshold: 2.0,
            mem_change_threshold: 1.0,
            net_change_threshold: 1024,
        }
    }
}

// === Notification configuration types ===
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationsConfig {
    pub enabled: bool,
    /// Max concurrent popups shown
    pub max_popups: usize,
    /// Max notifications retained in history (-1 for no limit)
    pub max_history: isize,
    /// Whether to honor notification's expire_timeout
    pub respect_expire_timeout: bool,
    /// Popup duration for LOW urgency (ms)
    pub low_urgency_duration_ms: u64,
    /// Popup duration for NORMAL urgency (ms)
    pub normal_urgency_duration_ms: u64,
    /// Popup duration for CRITICAL urgency (ms)
    pub critical_urgency_duration_ms: u64,
    /// Save LOW urgency to history
    pub save_low: bool,
    /// Save NORMAL urgency to history
    pub save_normal: bool,
    /// Save CRITICAL urgency to history
    pub save_critical: bool,
    /// Override path for history file (optional)
    pub history_file: Option<String>,
    /// Override path for state file (optional)
    pub state_file: Option<String>,
    /// Override path for rules file (optional)
    pub rules_file: Option<String>,
}

impl Default for NotificationsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_popups: 5,
            max_history: 100,
            respect_expire_timeout: false,
            low_urgency_duration_ms: 3000,
            normal_urgency_duration_ms: 8000,
            critical_urgency_duration_ms: 15000,
            save_low: true,
            save_normal: true,
            save_critical: true,
            history_file: None,
            state_file: None,
            rules_file: None,
        }
    }
}
