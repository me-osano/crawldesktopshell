use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_db_path")]
    pub db_path: PathBuf,
    #[serde(default = "default_socket_path")]
    pub socket_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            db_path: default_db_path(),
            socket_path: default_socket_path(),
        }
    }
}

pub fn load() -> anyhow::Result<Config> {
    let config_path = std::env::var("CRAWL_MAIL_CONFIG")
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
        return PathBuf::from(dir).join("crawl/config.toml");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".config/crawl/config.toml");
    }
    PathBuf::from("./config.toml")
}

fn default_db_path() -> PathBuf {
    if let Ok(path) = std::env::var("CRAWL_MAIL_DB") {
        return PathBuf::from(path);
    }
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        return PathBuf::from(dir).join("crawl/mail.db");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local/share/crawl/mail.db");
    }
    PathBuf::from("./mail.db")
}

fn default_socket_path() -> PathBuf {
    if let Ok(path) = std::env::var("CRAWL_MAIL_SOCKET") {
        return PathBuf::from(path);
    }
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(dir).join("crawl-mail.sock");
    }
    PathBuf::from("/tmp/crawl-mail.sock")
}
