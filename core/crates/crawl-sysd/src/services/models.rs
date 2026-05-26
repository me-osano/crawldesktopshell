use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::RwLock;

use crawl_ipc::protocol::{CrawlResponse, Error};
use crawl_ipc::{ErrorEnvelope, error_code};

#[async_trait]
pub trait Service: Send + Sync {
    fn name(&self) -> &'static str;
    async fn start(&self) -> Result<()>;
    async fn stop(&self) -> Result<()> {
        Ok(())
    }
    async fn handle(
        &self,
        _method: &str,
        _params: &Value,
        _id: Option<Value>,
    ) -> Option<CrawlResponse> {
        None
    }
    fn is_healthy(&self) -> bool {
        true
    }
}

pub struct ServiceRegistry {
    pub(crate) services: RwLock<HashMap<String, Arc<dyn Service>>>,
}
impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
        }
    }
    pub async fn register(&self, service: Arc<dyn Service>) {
        let name = service.name().to_string();
        tracing::info!("Registering service: {}", name);
        let mut services = self.services.write().await;
        services.insert(name, service);
    }
    pub async fn unregister(&self, name: &str) -> Option<Arc<dyn Service>> {
        tracing::info!("Unregistering service: {}", name);
        let mut services = self.services.write().await;
        services.remove(name)
    }
    #[expect(dead_code)]
    pub async fn get(&self, name: &str) -> Option<Arc<dyn Service>> {
        let services = self.services.read().await;
        services.get(name).cloned()
    }
    pub async fn start_all(&self) -> Result<()> {
        let services: Vec<(String, Arc<dyn Service>)> = {
            let services = self.services.read().await;
            services
                .iter()
                .map(|(name, svc)| (name.clone(), svc.clone()))
                .collect()
        };
        for (name, service) in services {
            tracing::info!("Starting service: {}", name);
            service.start().await?;
        }
        Ok(())
    }
    pub async fn stop_all(&self) -> Result<()> {
        let mut services: Vec<(String, Arc<dyn Service>)> = {
            let services = self.services.read().await;
            services
                .iter()
                .map(|(name, svc)| (name.clone(), svc.clone()))
                .collect()
        };
        services.sort_by(|a, b| b.0.cmp(&a.0));
        for (name, service) in services {
            tracing::info!("Stopping service: {}", name);
            if let Err(e) = service.stop().await {
                tracing::error!("Failed to stop service {}: {}", name, e);
            }
        }
        Ok(())
    }
    pub async fn health_check(&self) -> HashMap<String, bool> {
        let services = self.services.read().await;
        services
            .iter()
            .map(|(name, svc)| (name.clone(), svc.is_healthy()))
            .collect()
    }

    /// Dispatch a command to the appropriate service.
    /// Returns Some(Response) if handled, None otherwise.
    pub async fn dispatch(
        &self,
        method: &str,
        params: &Value,
        id: Option<Value>,
    ) -> Option<CrawlResponse> {
        let services: Vec<Arc<dyn Service>> = {
            let services = self.services.read().await;
            services.values().cloned().collect()
        };
        for service in services {
            if let Some(response) = service.handle(method, params, id.clone()).await {
                return Some(response);
            }
        }
        None
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
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
