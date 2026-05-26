pub mod client;
pub mod commands;
/// crawl-ipc: Shared types, event models, and error envelope.
/// No system dependencies — safe to use in any crate including future QML bridges.
pub mod error;
pub mod events;
pub mod protocol;
pub mod router;
pub mod server;
pub mod socket;
pub mod subscription;
pub mod types;

pub use client::CrawlClient;
pub use error::{CrawlError, CrawlResult, ErrorEnvelope};
pub use events::{CrawlEvent, RssEvent, ThemeEvent, WallhavenEvent};
pub use protocol::{CrawlRequest, CrawlResponse, Error, EventMessage, error_code, now_ms};
pub use router::{IpcRouter, RouteError};
pub use server::{EventReceiver, EventSender, IpcServer, RequestDispatcher};
pub use socket::{bind_socket, connect_socket, default_socket_path};
pub use subscription::EventSubscription;
pub use types::WallpaperBackend;
