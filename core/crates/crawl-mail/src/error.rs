use thiserror::Error;

#[derive(Debug, Error)]
pub enum MailError {
    #[error("config error: {0}")]
    Config(String),
    #[error("store error: {0}")]
    Store(String),
    #[error("imap error: {0}")]
    Imap(String),
    #[error("smtp error: {0}")]
    Smtp(String),
    #[error("ipc error: {0}")]
    Ipc(String),
}

pub type MailResult<T> = Result<T, MailError>;
