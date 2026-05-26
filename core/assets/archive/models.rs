use std::sync::Arc;
use std::time::Duration;
use anyhow::Result;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info};
use crawl_ipc::protocol::{CrawlResponse, Error};
use crawl_ipc::{error_code, ErrorEnvelope};

type MakeFuture = Arc<
    dyn Fn() -> tokio::task::JoinHandle<Result<()>> + Send + Sync,
>;

#[async_trait::async_trait]
pub trait Service: Send + Sync {
    fn name(&self) -> &'static str;
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()> {
        Ok(())
    }
    async fn handle(&self, _method: &str, _params: &Value, _id: Option<Value>) -> Option<CrawlResponse> {
        None
    }
    fn is_healthy(&self) -> bool {
        true
    }
}
pub struct DomainService {
    name: &'static str,
    make_fut: MakeFuture,
    handle: Mutex<Option<JoinHandle<()>>>,
}
impl DomainService {
    pub fn new<F, Fut>(name: &'static str, make_fut: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let make_fut: MakeFuture = Arc::new(move || tokio::spawn(make_fut()));
        Self {
            name,
            make_fut,
            handle: Mutex::new(None),
        }
    }
}
#[async_trait::async_trait]
impl Service for DomainService {
    fn name(&self) -> &'static str {
        self.name
    }
    async fn start(&self) -> Result<()> {
        let name = self.name;
        let make_fut = self.make_fut.clone();
        let handle = tokio::spawn(async move {
            let mut backoff = 1u64;
            loop {
                let result = make_fut().await;
                match result {
                    Ok(Ok(())) => break,
                    Ok(Err(err)) => {
                        error!(domain = name, error = %err, "domain failed");
                        tokio::time::sleep(Duration::from_secs(backoff)).await;
                        backoff = (backoff * 2).min(60);
                    }
                    Err(err) => {
                        error!(domain = name, error = %err, "domain task failed");
                        tokio::time::sleep(Duration::from_secs(backoff)).await;
                        backoff = (backoff * 2).min(60);
                    }
                }
            }
        });
        let mut guard = self.handle.lock().await;
        if let Some(existing) = guard.take() {
            existing.abort();
        }
        *guard = Some(handle);
        Ok(())
    }
    async fn stop(&self) -> Result<()> {
        let mut guard = self.handle.lock().await;
        if let Some(handle) = guard.take() {
            handle.abort();
        }
        Ok(())
    }
}
pub struct ServiceRegistry {
    services: Vec<Arc<dyn Service>>,
}
impl ServiceRegistry {
    pub fn new() -> Self {
        Self { services: Vec::new() }
    }
    pub fn register(&mut self, service: Arc<dyn Service>) {
        info!(service = service.name(), "registering service");
        self.services.push(service);
    }
    pub async fn start_all(&self) -> Result<()> {
        for service in &self.services {
            info!(service = service.name(), "starting service");
            service.start().await?;
        }
        Ok(())
    }
    pub async fn stop_all(&self) -> Result<()> {
        for service in self.services.iter().rev() {
            info!(service = service.name(), "stopping service");
            if let Err(err) = service.stop().await {
                error!(service = service.name(), error = %err, "failed to stop service");
            }
        }
        Ok(())
    }
    pub fn health_check(&self) -> std::collections::HashMap<String, bool> {
        self.services
            .iter()
            .map(|svc| (svc.name().to_string(), svc.is_healthy()))
            .collect()
    }
}
impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
impl Clone for ServiceRegistry {
    fn clone(&self) -> Self {
        Self {
            services: self.services.clone(),
        }
    }
}

pub fn error_response(id: Option<serde_json::Value>, message: String) -> CrawlResponse {
    let envelope = ErrorEnvelope::new("crawl", "internal_error", message);
    CrawlResponse {
        jsonrpc: crawl_ipc::protocol::JSONRPC_VERSION.to_string(),
        result: None,
        error: Some(Error {
            code: error_code::INTERNAL_ERROR,
            message: "internal error".to_string(),
            data: Some(serde_json::to_value(envelope).unwrap_or_default()),
        }),
        id,
    }
}
