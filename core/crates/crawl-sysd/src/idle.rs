//! Idle inhibition via logind D-Bus API.
//!
//! Calls `org.freedesktop.login1.Manager.Inhibit` and holds the returned
//! file descriptor. The inhibition is active as long as the fd stays open;
//! dropping it releases the inhibitor lock.

use std::fs::File;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::sync::Arc;

use thiserror::Error;
use tokio::sync::Mutex;
use tracing::info;
use zbus::Connection;

use crawl_ipc::events::{CrawlEvent, IdleEvent};
use crawl_ipc::types::IdleStatus;

/// Shared handle to the current inhibition state.
pub struct IdleState {
    inhibited: bool,
    reason: String,
    /// Holds the dup'd logind inhibitor fd. Dropping it releases the lock.
    _inhibit_fd: Option<File>,
}

/// Controller for idle inhibition, safe to share across tasks.
pub struct IdleController {
    state: Arc<Mutex<IdleState>>,
    event_tx: tokio::sync::broadcast::Sender<CrawlEvent>,
}

impl IdleController {
    pub fn new(event_tx: tokio::sync::broadcast::Sender<CrawlEvent>) -> Self {
        Self {
            state: Arc::new(Mutex::new(IdleState {
                inhibited: false,
                reason: String::new(),
                _inhibit_fd: None,
            })),
            event_tx,
        }
    }

    pub async fn inhibit(&self) -> Result<(), IdleError> {
        let reason = "User requested";
        let mut state = self.state.lock().await;
        if state.inhibited {
            return Ok(());
        }

        let conn = Connection::system()
            .await
            .map_err(|e| IdleError::Connection(e.to_string()))?;

        let reply = conn
            .call_method(
                Some("org.freedesktop.login1"),
                "/org/freedesktop/login1",
                Some("org.freedesktop.login1.Manager"),
                "Inhibit",
                &("idle", "crawl", reason, "block"),
            )
            .await
            .map_err(|e| IdleError::Inhibit(e.to_string()))?;

        let body = reply.body();
        let fd: zbus::zvariant::Fd = body
            .deserialize()
            .map_err(|e| IdleError::Inhibit(e.to_string()))?;

        let duped = unsafe { libc::dup(fd.as_raw_fd()) };
        if duped < 0 {
            return Err(IdleError::Inhibit("failed to dup inhibitor fd".into()));
        }
        let owned_fd = unsafe { File::from_raw_fd(duped) };

        state._inhibit_fd = Some(owned_fd);
        state.inhibited = true;
        state.reason = reason.to_string();

        info!("Idle inhibition started");

        let _ = self
            .event_tx
            .send(CrawlEvent::Idle(IdleEvent::InhibitionStarted));
        let _ = self
            .event_tx
            .send(CrawlEvent::Idle(IdleEvent::StateChanged {
                inhibited: true,
            }));

        Ok(())
    }

    pub async fn uninhibit(&self) {
        let mut state = self.state.lock().await;
        if !state.inhibited {
            return;
        }

        state._inhibit_fd = None;
        state.inhibited = false;
        state.reason.clear();

        info!("Idle inhibition stopped");

        let _ = self
            .event_tx
            .send(CrawlEvent::Idle(IdleEvent::InhibitionStopped));
        let _ = self
            .event_tx
            .send(CrawlEvent::Idle(IdleEvent::StateChanged {
                inhibited: false,
            }));
    }

    pub async fn status(&self) -> IdleStatus {
        let state = self.state.lock().await;
        IdleStatus {
            inhibited: state.inhibited,
            reason: state.reason.clone(),
        }
    }

    pub async fn inhibit_with_timeout(&self, seconds: u64) -> Result<(), IdleError> {
        self.inhibit().await?;
        let state = self.state.clone();
        let event_tx = self.event_tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(seconds)).await;
            let mut s = state.lock().await;
            if s.inhibited {
                s._inhibit_fd = None;
                s.inhibited = false;
                s.reason.clear();
                info!("Idle inhibition timed out after {}s", seconds);
                let _ = event_tx.send(CrawlEvent::Idle(IdleEvent::InhibitionStopped));
                let _ = event_tx.send(CrawlEvent::Idle(IdleEvent::StateChanged {
                    inhibited: false,
                }));
            }
        });
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum IdleError {
    #[error("D-Bus connection failed: {0}")]
    Connection(String),
    #[error("inhibit failed: {0}")]
    Inhibit(String),
}
