use std::path::PathBuf;
use std::time::Duration;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::broadcast;
use tokio::time::timeout;
use tracing::{error, warn};

use crate::events::CrawlEvent;
use crate::protocol::{CrawlRequest, CrawlResponse};
use crate::subscription::EventSubscription;

/// Error returned when routing a command to a child daemon fails.
#[derive(Debug)]
pub enum RouteError {
    NoRoute,
    Unavailable(String),
    Io(std::io::Error),
}

impl std::fmt::Display for RouteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoRoute => write!(f, "no route for method"),
            Self::Unavailable(s) => write!(f, "child daemon unavailable: {s}"),
            Self::Io(e) => write!(f, "routing I/O error: {e}"),
        }
    }
}

impl std::error::Error for RouteError {}

/// A route entry mapping a method prefix to a child daemon socket.
#[derive(Debug, Clone)]
struct Route {
    prefix: String,
    socket_path: PathBuf,
}

/// Routes IPC commands to child daemons and bridges events back.
///
/// This is used by `crawl-sysd` to forward domain-specific commands
/// (RSS, Wallhaven, Mail) to their respective daemon processes while
/// presenting a single IPC socket to all clients (CLI, QML).
#[derive(Default, Clone)]
pub struct IpcRouter {
    routes: Vec<Route>,
}

impl IpcRouter {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Register that any method starting with `prefix` should be forwarded
    /// to the daemon listening at `socket_path`.
    ///
    /// Longer prefixes take priority over shorter ones when multiple match.
    pub fn register(&mut self, prefix: &str, socket_path: PathBuf) {
        self.routes.retain(|r| r.prefix != prefix);
        self.routes.push(Route {
            prefix: prefix.to_string(),
            socket_path,
        });
    }

    /// Find the best-matching route for a method name (longest prefix wins).
    fn find_route(&self, method: &str) -> Option<&Route> {
        self.routes
            .iter()
            .filter(|r| method.starts_with(&r.prefix))
            .max_by_key(|r| r.prefix.len())
    }

    /// Forward a JSON-RPC request to the appropriate child daemon.
    ///
    /// The original request `id` is preserved in the response so the caller
    /// can match it back to the awaiting client.
    pub async fn route(
        &self,
        method: &str,
        params: Value,
        id: Option<Value>,
    ) -> Result<CrawlResponse, RouteError> {
        let route = self.find_route(method).ok_or(RouteError::NoRoute)?;

        let request =
            CrawlRequest::with_id(method, params, id.clone().unwrap_or(serde_json::json!(0)));

        let stream = timeout(
            Duration::from_secs(5),
            UnixStream::connect(&route.socket_path),
        )
        .await
        .map_err(|_| {
            RouteError::Unavailable(format!("connection to {:?} timed out", route.socket_path))
        })?
        .map_err(|e| {
            RouteError::Unavailable(format!("cannot connect to {:?}: {e}", route.socket_path))
        })?;

        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);

        let req_str = serde_json::to_string(&request)
            .map_err(|e| RouteError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        writer
            .write_all(req_str.as_bytes())
            .await
            .map_err(RouteError::Io)?;
        writer.write_all(b"\n").await.map_err(RouteError::Io)?;
        writer.flush().await.map_err(RouteError::Io)?;

        let mut line = String::new();
        timeout(Duration::from_secs(10), reader.read_line(&mut line))
            .await
            .map_err(|_| RouteError::Unavailable("response from child daemon timed out".into()))?
            .map_err(RouteError::Io)?;

        let response: CrawlResponse = serde_json::from_str(&line)
            .map_err(|e| RouteError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

        Ok(response)
    }

    /// Spawn background tasks that subscribe to events from each child daemon
    /// and forward them to the given local event bus sender.
    ///
    /// Each child gets its own tokio task with a persistent connection.
    pub fn bridge_events(&self, tx: broadcast::Sender<CrawlEvent>) {
        for route in &self.routes {
            let socket_path = route.socket_path.clone();
            let tx = tx.clone();
            let prefix = route.prefix.clone();
            tokio::spawn(async move {
                loop {
                    match bridge_one(&socket_path, &tx).await {
                        Ok(()) => {
                            warn!(
                                "Event bridge for '{prefix}' disconnected (clean exit), reconnecting..."
                            );
                        }
                        Err(e) => {
                            warn!("Event bridge for '{prefix}' error: {e}, reconnecting in 5s...");
                        }
                    }
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            });
        }
    }
}

/// Connects to a child daemon socket, subscribes to events, and forwards
/// them to the local event bus. Returns on disconnection.
async fn bridge_one(
    socket_path: &std::path::Path,
    tx: &broadcast::Sender<CrawlEvent>,
) -> anyhow::Result<()> {
    let sub = EventSubscription::new(socket_path);
    sub.subscribe::<_, CrawlEvent>(move |event| {
        if tx.receiver_count() > 0 {
            if let Err(e) = tx.send(event) {
                error!("Event bridge drop (no receivers): {e}");
            }
        }
    })
    .await
}
