use serde_json::Value;
use std::sync::Arc;

use anyhow::Result;
use libsystemd::daemon::{NotifyState, notify};
use tokio::sync::broadcast;
use tracing::{info, warn};

use crawl_ipc::CrawlEvent;
use crawl_ipc::IpcRouter;
use crawl_ipc::error_code;
use crawl_ipc::protocol::CrawlResponse;

use crate::bluetooth::BluetoothRuntime;
use crate::config::Config;
use crate::state::{AppState, SharedState};
use crate::services::audio::AudioService;
use crate::services::bluetooth::BluetoothService;
use crate::services::brightness::BrightnessService;
use crate::services::idle::IdleService;
use crate::services::models::ServiceRegistry;
use crate::services::network::NetworkService;
use crate::services::notification::NotificationService;
use crate::services::proc::ProcService;
use crate::services::sysmon::SysmonService;
use crate::services::theme::ThemeService;
use crate::services::{daemon, diagnostics};

pub struct Daemon {
    config: Config,
    state: SharedState,
    event_bus: EventBus,
    dispatcher: Dispatcher,
}

impl Daemon {
    pub async fn new(config: Config) -> Result<Self> {
        let event_bus = EventBus::new(100);
        let bluetooth_rt = match BluetoothRuntime::new().await {
            Ok(rt) => {
                info!("Bluetooth runtime initialized");
                Some(Arc::new(rt))
            }
            Err(e) => {
                warn!(error = %e, "Bluetooth runtime unavailable, bluetooth service disabled");
                None
            }
        };
        let state = Arc::new(AppState::new(
            config.clone(),
            event_bus.clone(),
            bluetooth_rt,
        ));
        let dispatcher = Dispatcher::new(state.clone());

        Ok(Self {
            config,
            state,
            event_bus,
            dispatcher,
        })
    }

    pub async fn register_services(&mut self) {
        let registry = self.dispatcher.registry();
        let state = self.state.clone();
        registry
            .register(Arc::new(AudioService::new(state.clone())))
            .await;
        registry
            .register(Arc::new(BluetoothService::new(state.clone())))
            .await;
        registry
            .register(Arc::new(BrightnessService::new(state.clone())))
            .await;
        registry
            .register(Arc::new(IdleService::new(state.clone())))
            .await;
        registry
            .register(Arc::new(NetworkService::new(state.clone())))
            .await;
        registry
            .register(Arc::new(ProcService::new(state.clone())))
            .await;
        registry
            .register(Arc::new(SysmonService::new(state.clone())))
            .await;
        registry
            .register(Arc::new(ThemeService::new(state.clone())))
            .await;
        registry
            .register(Arc::new(NotificationService::new(state.clone())))
            .await;
    }

    /// Configure the IPC router to forward domain-specific commands
    /// (RSS, Wallhaven, Mail) to their respective child daemon sockets.
    /// Also spawns event bridge tasks to forward child events to the local bus.
    pub fn configure_router(&mut self) {
        let mut router = IpcRouter::new();
        for child in &self.config.daemon.child_daemons {
            let path: std::path::PathBuf = child.socket_path.clone().into();
            for prefix in &child.method_prefixes {
                router.register(prefix, path.clone());
            }
            info!(
                name = %child.name,
                socket = %child.socket_path,
                "Registered child daemon route"
            );
        }
        // Bridge child daemon events to our local event bus so clients
        // subscribed to sysd see all events transparently.
        router.bridge_events(self.event_bus.sender());
        self.dispatcher.set_router(router);
    }

    pub async fn run(&self) -> Result<()> {
        info!(
            socket_path = %self.config.daemon.socket_path,
            "crawl-sysd starting"
        );

        let dispatcher = {
            let dispatcher = Arc::new(self.dispatcher.clone());
            let handler: Arc<
                dyn Fn(
                        String,
                        serde_json::Value,
                        Option<serde_json::Value>,
                    ) -> std::pin::Pin<
                        Box<dyn futures_util::Future<Output = CrawlResponse> + Send>,
                    > + Send
                    + Sync,
            > = Arc::new(move |method, params, id| {
                let dispatcher = dispatcher.clone();
                Box::pin(async move { dispatcher.dispatch(method, params, id).await })
            });
            handler
        };
        let mut ipc_server = crawl_ipc::IpcServer::new(
            self.config.daemon.socket_path.clone().into(),
            self.event_bus.sender(),
        );
        ipc_server.set_dispatcher(dispatcher);
        let server = Arc::new(ipc_server);
        let mut server_task = {
            let server = server.clone();
            tokio::spawn(async move { server.run().await })
        };

        self.dispatcher.services.start_all().await?;
        let health_task = spawn_health_monitor(self.dispatcher.services.clone());
        let watchdog_task = spawn_watchdog_ticker();

        if let Err(e) = notify(false, &[NotifyState::Ready]) {
            warn!(error = %e, "Failed to send READY=1 to systemd");
        } else {
            info!("Sent READY=1 to systemd");
        }

        tokio::select! {
            res = &mut server_task => {
                match res {
                    Ok(Ok(())) => {}
                    Ok(Err(err)) => warn!(error = %err, "IPC server error"),
                    Err(err) => warn!(error = %err, "IPC server task failed"),
                }
            }
            _ = shutdown_signal() => {
                info!("Shutting down crawl-sysd");
            }
        }

        if let Err(e) = notify(false, &[NotifyState::Stopping]) {
            warn!(error = %e, "Failed to send STOPPING=1 to systemd");
        }

        watchdog_task.abort();
        health_task.abort();
        server_task.abort();
        if let Err(err) = remove_socket(&self.config.daemon.socket_path) {
            warn!(error = %err, "Failed to remove socket file");
        }
        let _ = self.dispatcher.services.stop_all().await;
        Ok(())
    }
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut term = signal(SignalKind::terminate()).ok();
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = async {
                if let Some(ref mut term) = term {
                    term.recv().await;
                }
            } => {}
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

fn spawn_health_monitor(services: Arc<ServiceRegistry>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
            let health = services.health_check().await;
            if !health.values().all(|&v| v) {
                warn!(?health, "Unhealthy services detected");
            }
        }
    })
}

fn spawn_watchdog_ticker() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let interval = std::time::Duration::from_secs(20);
        loop {
            tokio::time::sleep(interval).await;
            if let Err(e) = notify(false, &[NotifyState::Watchdog]) {
                warn!(error = %e, "Failed to send WATCHDOG=1 to systemd");
            }
        }
    })
}

fn remove_socket(socket_path: &str) -> std::io::Result<()> {
    let path = std::path::Path::new(socket_path);
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// === Event Bus ===
#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<CrawlEvent>,
}

#[expect(dead_code)]
impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn publish(&self, event: CrawlEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<CrawlEvent> {
        self.tx.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<CrawlEvent> {
        self.tx.clone()
    }

    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

/// === Dispatcher ===
#[derive(Clone)]
pub struct Dispatcher {
    pub(crate) services: Arc<ServiceRegistry>,
    state: SharedState,
    router: Option<IpcRouter>,
}

impl Dispatcher {
    pub fn new(state: SharedState) -> Self {
        Self {
            services: Arc::new(ServiceRegistry::new()),
            state,
            router: None,
        }
    }

    /// Get a mutable reference to the service registry (for registering services).
    pub fn registry(&self) -> Arc<ServiceRegistry> {
        self.services.clone()
    }

    /// Set the IPC router for forwarding commands to child daemons.
    pub fn set_router(&mut self, router: IpcRouter) {
        self.router = Some(router);
    }

    pub async fn dispatch(
        &self,
        method: String,
        params: Value,
        id: Option<Value>,
    ) -> CrawlResponse {
        // Try services first
        if let Some(response) = self.services.dispatch(&method, &params, id.clone()).await {
            return response;
        }

        // Fallback to built-in commands
        match method.as_str() {
            "DaemonPing" | "ServiceList" | "ServiceRegister" | "ServiceUnregister" => {
                daemon::handle(
                    self.state.clone(),
                    self.services.clone(),
                    method,
                    params,
                    id,
                )
                .await
            }
            "Status" => diagnostics::handle_status(self.state.clone(), id).await,
            "Health" => diagnostics::handle_health(self.state.clone(), id).await,
            "Sysinfo" => {
                let info = crate::sysinfo::get_info();
                CrawlResponse::success(id, serde_json::to_value(info).unwrap_or_default())
            }
            "Ping" => CrawlResponse::success(
                id,
                serde_json::json!({"time_ms": crawl_ipc::protocol::now_ms()}),
            ),
            "Hello" => CrawlResponse::success(
                id,
                serde_json::json!({"version": env!("CARGO_PKG_VERSION"), "time_ms": crawl_ipc::protocol::now_ms()}),
            ),
            "Subscribe" => CrawlResponse::success(id, serde_json::json!({ "subscribed": true })),
            _ => {
                // Try forwarding to a child daemon via the IPC router
                if let Some(ref router) = self.router {
                    match router.route(&method, params, id.clone()).await {
                        Ok(response) => response,
                        Err(crawl_ipc::RouteError::NoRoute) => CrawlResponse::error(
                            id,
                            error_code::METHOD_NOT_FOUND,
                            &format!("Unknown method: {}", method),
                        ),
                        Err(e) => CrawlResponse::error(
                            id,
                            error_code::INTERNAL_ERROR,
                            &format!("Child daemon error for method '{}': {e}", method),
                        ),
                    }
                } else {
                    CrawlResponse::error(
                        id,
                        error_code::METHOD_NOT_FOUND,
                        &format!("Unknown method: {}", method),
                    )
                }
            }
        }
    }
}
