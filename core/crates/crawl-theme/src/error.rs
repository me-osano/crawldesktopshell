use thiserror::Error;

#[derive(Debug, Error)]
pub enum ThemeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("Invalid color: {0}")]
    InvalidColor(String),

    #[error("Unknown variant: {0}")]
    UnknownVariant(String),

    #[error("Config not loaded")]
    ConfigNotLoaded,

    #[error("{0}")]
    Other(String),
}

pub type ThemeResult<T> = Result<T, ThemeError>;
