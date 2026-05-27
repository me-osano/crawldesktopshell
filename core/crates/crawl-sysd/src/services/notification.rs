use async_trait::async_trait;
use tracing::info;

use crawl_ipc::protocol::CrawlResponse;
use crawl_ipc::types::NotificationPolicy;

use crate::notification::NotificationBackend;
use crate::services::models::{Service, error_response};
use crate::state::SharedState;

pub struct NotificationService {
    backend: std::sync::Arc<NotificationBackend>,
}

impl NotificationService {
    pub fn new(state: SharedState) -> Self {
        Self {
            backend: std::sync::Arc::new(NotificationBackend::new(state)),
        }
    }
}

#[async_trait]
impl Service for NotificationService {
    fn name(&self) -> &'static str {
        "notification"
    }

    async fn start(&self) -> anyhow::Result<()> {
        info!("Starting notification service");
        self.backend.init().await;
        self.backend.clone().start_dbus().await?;
        Ok(())
    }

    async fn handle(
        &self,
        method: &str,
        params: &serde_json::Value,
        id: Option<serde_json::Value>,
    ) -> Option<CrawlResponse> {
        let response = match method {
            "NotificationGetState" => {
                let state = self.backend.get_state().await;
                CrawlResponse::success(id, serde_json::to_value(state).unwrap_or_default())
            }
            "NotificationDismiss" => {
                let notif_id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let ok = if !notif_id.is_empty() {
                    self.backend.dismiss_popup(notif_id).await
                } else {
                    false
                };
                CrawlResponse::success(id, serde_json::json!({"ok": ok}))
            }
            "NotificationDismissAll" => {
                self.backend.dismiss_all_popups().await;
                CrawlResponse::success(id, serde_json::json!({"ok": true}))
            }
            "NotificationRemoveHistory" => {
                let notif_id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let ok = if !notif_id.is_empty() {
                    self.backend.remove_from_history(notif_id).await
                } else {
                    false
                };
                CrawlResponse::success(id, serde_json::json!({"ok": ok}))
            }
            "NotificationClearHistory" => {
                self.backend.clear_history().await;
                CrawlResponse::success(id, serde_json::json!({"ok": true}))
            }
            "NotificationInvokeAction" => {
                let notif_id = params.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let action_id = params
                    .get("action_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if notif_id.is_empty() || action_id.is_empty() {
                    error_response(id, "missing 'id' or 'action_id'".to_string())
                } else {
                    let ok = self.backend.invoke_action(notif_id, action_id).await;
                    CrawlResponse::success(id, serde_json::json!({"ok": ok}))
                }
            }
            "NotificationSetDnd" => {
                let enabled = params
                    .get("enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                self.backend.set_dnd(enabled).await;
                CrawlResponse::success(id, serde_json::json!({"enabled": enabled}))
            }
            "NotificationSetLastSeen" => {
                let ts = params.get("ts").and_then(|v| v.as_u64()).unwrap_or(0);
                self.backend.set_last_seen(ts).await;
                CrawlResponse::success(id, serde_json::json!({"ts": ts}))
            }
            "NotificationGetPolicy" => {
                let policy = self.backend.get_policy().await;
                CrawlResponse::success(id, serde_json::to_value(policy).unwrap_or_default())
            }
            "NotificationSetPolicy" => {
                let policy_value = params.get("policy").cloned().unwrap_or_default();
                let policy: NotificationPolicy = match serde_json::from_value(policy_value) {
                    Ok(p) => p,
                    Err(e) => return Some(error_response(id, format!("invalid policy: {e}"))),
                };
                self.backend.set_policy(policy).await;
                CrawlResponse::success(id, serde_json::json!({"ok": true}))
            }
            "NotificationGetRules" => {
                let rules_json = self.backend.get_rules_json().await;
                CrawlResponse::success(id, serde_json::json!({"rules_json": rules_json}))
            }
            "NotificationSetRules" | "NotificationSaveRules" => {
                let rules_json = params
                    .get("rules_json")
                    .and_then(|v| v.as_str())
                    .unwrap_or("{");
                match self.backend.set_rules_json(rules_json).await {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => error_response(id, e),
                }
            }
            _ => return None,
        };

        Some(response)
    }
}
