//! Unix socket transport layer for crawl IPC.
//! No protocol knowledge — only byte-stream transport.

use std::path::{Path, PathBuf};
use tokio::net::{UnixListener, UnixStream};

/// Bind a Unix socket for listening, cleaning up stale sockets.
pub async fn bind_socket(path: impl AsRef<Path>) -> std::io::Result<UnixListener> {
    let path = path.as_ref();
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    UnixListener::bind(path)
}

/// Connect to a Unix socket as a client.
pub async fn connect_socket(path: impl AsRef<Path>) -> std::io::Result<UnixStream> {
    UnixStream::connect(path).await
}

/// Resolve the default socket path for the crawl daemon.
/// Checks CRAWL_SOCKET env var, then XDG_RUNTIME_DIR.
pub fn default_socket_path() -> PathBuf {
    if let Ok(path) = std::env::var("CRAWL_SOCKET") {
        return PathBuf::from(path);
    }
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        return PathBuf::from(dir).join("crawl.sock");
    }
    // Fallback: try /run/user/<uid> using /proc/self/uid if available
    #[cfg(target_os = "linux")]
    {
        if let Ok(s) = std::fs::read_to_string("/proc/self/uid") {
            if let Ok(uid) = s.trim().parse::<u32>() {
                return PathBuf::from(format!("/run/user/{}/crawl.sock", uid));
            }
        }
    }
    PathBuf::from("/tmp/crawl.sock")
}
