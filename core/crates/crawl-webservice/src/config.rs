use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,
    #[serde(default = "default_socket_path")]
    pub socket_path: PathBuf,
    #[serde(default)]
    pub rss: RssConfig,
    #[serde(default = "default_max_parallel_downloads")]
    pub max_parallel_downloads: usize,
    #[serde(default)]
    pub wallhaven: WallhavenConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RssConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_max_concurrent_fetches")]
    pub max_concurrent_fetches: usize,
    #[serde(default = "default_fetch_interval_secs")]
    pub default_fetch_interval_secs: u64,
    #[serde(default = "default_rss_timeout_secs")]
    pub timeout_secs: u64,
    #[serde(default = "default_rss_user_agent")]
    pub user_agent: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WallhavenConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default = "default_wallhaven_rate_per_min")]
    pub rate_per_min: u32,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for RssConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_concurrent_fetches: default_max_concurrent_fetches(),
            default_fetch_interval_secs: default_fetch_interval_secs(),
            timeout_secs: default_rss_timeout_secs(),
            user_agent: default_rss_user_agent(),
        }
    }
}

impl Default for WallhavenConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            rate_per_min: default_wallhaven_rate_per_min(),
            enabled: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_path: default_db_path(),
            socket_path: default_socket_path(),
            rss: RssConfig::default(),
            max_parallel_downloads: default_max_parallel_downloads(),
            wallhaven: WallhavenConfig::default(),
        }
    }
}

pub fn load() -> anyhow::Result<Config> {
    let config_path = std::env::var("CRAWL_WEBSERVICE_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_config_path());

    if config_path.exists() {
        let raw = std::fs::read_to_string(&config_path)?;
        let cfg: Config = toml::from_str(&raw)?;
        Ok(cfg)
    } else {
        Ok(Config::default())
    }
}

fn default_config_path() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir).join("crawl/webservice.toml");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config/crawl/webservice.toml");
    }
    PathBuf::from("./webservice.toml")
}

fn default_db_path() -> PathBuf {
    if let Ok(path) = std::env::var("CRAWL_WEBSERVICE_DB") {
        return PathBuf::from(path);
    }
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(dir).join("crawl/webservice.db");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local/share/crawl/webservice.db");
    }
    PathBuf::from("./webservice.db")
}

fn default_socket_path() -> PathBuf {
    if let Ok(path) = std::env::var("CRAWL_WEBSERVICE_SOCKET") {
        return PathBuf::from(path);
    }
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(dir).join("crawl-webservice.sock");
    }
    PathBuf::from("/tmp/crawl-webservice.sock")
}

fn default_max_concurrent_fetches() -> usize {
    8
}

fn default_fetch_interval_secs() -> u64 {
    1800
}

fn default_max_parallel_downloads() -> usize {
    3
}

fn default_true() -> bool {
    true
}

fn default_rss_timeout_secs() -> u64 {
    30
}

fn default_rss_user_agent() -> String {
    "CrawlDS/0.1 RSS Reader".to_string()
}

fn default_wallhaven_rate_per_min() -> u32 {
    45
}
