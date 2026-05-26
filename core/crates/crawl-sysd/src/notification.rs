use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use regex::Regex;
use tokio::sync::Mutex;
use tracing::warn;
use zbus::zvariant::{Str as ZStr, Value as ZValue};

use crawl_ipc::events::{CrawlEvent, NotificationEvent};
use crawl_ipc::types::{
    NotificationAction, NotificationHistoryPolicy, NotificationItem, NotificationPolicy,
    NotificationState,
};

use crate::config::NotificationsConfig;
use crate::state::SharedState;

const NOTIFICATION_PATH: &str = "/org/freedesktop/Notifications";
const NOTIFICATION_INTERFACE: &str = "org.freedesktop.Notifications";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct NotificationRulesFile {
    rules: Vec<NotificationRule>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct NotificationRule {
    pattern: String,
    action: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct NotificationStateFile {
    do_not_disturb: bool,
    last_seen_ts: u64,
    policy: NotificationPolicy,
}

struct NotificationRuntime {
    popups: Vec<NotificationItem>,
    history: Vec<NotificationItem>,
    do_not_disturb: bool,
    last_seen_ts: u64,
    policy: NotificationPolicy,
    rules: Vec<NotificationRule>,
}

pub struct NotificationBackend {
    state: Mutex<NotificationRuntime>,
    event_tx: tokio::sync::broadcast::Sender<CrawlEvent>,
    id_counter: AtomicU32,
    connection: Mutex<Option<zbus::Connection>>,
    history_path: PathBuf,
    state_path: PathBuf,
    rules_path: PathBuf,
    config: NotificationsConfig,
}

impl NotificationBackend {
    pub fn new(state: SharedState) -> Self {
        let cfg = &state.config.notifications;
        let history_path = resolve_history_path(cfg.history_file.as_deref());
        let state_path = resolve_state_path(cfg.state_file.as_deref());
        let rules_path = resolve_rules_path(cfg.rules_file.as_deref());
        Self {
            state: Mutex::new(NotificationRuntime {
                popups: Vec::new(),
                history: Vec::new(),
                do_not_disturb: false,
                last_seen_ts: 0,
                policy: default_policy(),
                rules: Vec::new(),
            }),
            event_tx: state.event_bus.sender(),
            id_counter: AtomicU32::new(1),
            connection: Mutex::new(None),
            history_path,
            state_path,
            rules_path,
            config: cfg.clone(),
        }
    }

    pub async fn init(&self) {
        let mut runtime = self.state.lock().await;

        // Start with defaults, overlay config, then state file (highest priority)
        let mut policy = default_policy();
        policy.enabled = self.config.enabled;
        policy.max_popups = self.config.max_popups;
        policy.max_history = if self.config.max_history < 0 {
            2000
        } else {
            self.config.max_history as usize
        };
        policy.respect_expire_timeout = self.config.respect_expire_timeout;
        policy.low_urgency_duration_ms = self.config.low_urgency_duration_ms;
        policy.normal_urgency_duration_ms = self.config.normal_urgency_duration_ms;
        policy.critical_urgency_duration_ms = self.config.critical_urgency_duration_ms;
        policy.save_to_history.low = self.config.save_low;
        policy.save_to_history.normal = self.config.save_normal;
        policy.save_to_history.critical = self.config.save_critical;
        runtime.policy = policy;

        if let Some(state_file) = load_state_file(&self.state_path) {
            runtime.do_not_disturb = state_file.do_not_disturb;
            runtime.last_seen_ts = state_file.last_seen_ts;
            runtime.policy = state_file.policy;
        }

        runtime.history = load_history_file(&self.history_path);
        runtime.rules = load_rules_file(&self.rules_path);
    }

    pub async fn start_dbus(self: Arc<Self>) -> anyhow::Result<()> {
        let backend = self.clone();
        tokio::spawn(async move {
            let builder = match zbus::ConnectionBuilder::session() {
                Ok(b) => b,
                Err(e) => {
                    warn!(error = %e, "Failed to create notification D-Bus session");
                    return;
                }
            };

            let builder = match builder.name(NOTIFICATION_INTERFACE) {
                Ok(b) => b,
                Err(e) => {
                    warn!(error = %e, "Failed to request notification bus name");
                    return;
                }
            };

            let builder = match builder.serve_at(
                NOTIFICATION_PATH,
                NotificationDbus {
                    backend: backend.clone(),
                },
            ) {
                Ok(b) => b,
                Err(e) => {
                    warn!(error = %e, "Failed to register notification object");
                    return;
                }
            };

            let conn = match builder.build().await {
                Ok(c) => c,
                Err(e) => {
                    warn!(error = %e, "Failed to start notification D-Bus service");
                    return;
                }
            };

            {
                let mut guard = backend.connection.lock().await;
                *guard = Some(conn.clone());
            }

            futures_util::future::pending::<()>().await;
        });

        let backend = self.clone();
        tokio::spawn(async move {
            backend.run_expiration_loop().await;
        });

        Ok(())
    }

    pub async fn get_state(&self) -> NotificationState {
        let runtime = self.state.lock().await;
        NotificationState {
            popups: runtime.popups.clone(),
            history: runtime.history.clone(),
            do_not_disturb: runtime.do_not_disturb,
            last_seen_ts: runtime.last_seen_ts,
        }
    }

    pub async fn set_dnd(&self, enabled: bool) {
        let mut runtime = self.state.lock().await;
        runtime.do_not_disturb = enabled;
        save_state_file(&self.state_path, &runtime);
        let _ = self
            .event_tx
            .send(CrawlEvent::Notification(NotificationEvent::DndChanged {
                enabled,
            }));
    }

    pub async fn set_last_seen(&self, ts: u64) {
        let mut runtime = self.state.lock().await;
        runtime.last_seen_ts = ts;
        save_state_file(&self.state_path, &runtime);
        let _ = self.event_tx.send(CrawlEvent::Notification(
            NotificationEvent::LastSeenChanged { ts },
        ));
    }

    pub async fn set_policy(&self, policy: NotificationPolicy) {
        let mut runtime = self.state.lock().await;
        runtime.policy = policy;
        save_state_file(&self.state_path, &runtime);
    }

    pub async fn get_policy(&self) -> NotificationPolicy {
        let runtime = self.state.lock().await;
        runtime.policy.clone()
    }

    pub async fn set_rules_json(&self, rules_json: &str) -> Result<(), String> {
        let parsed: NotificationRulesFile =
            serde_json::from_str(rules_json).map_err(|e| format!("Invalid rules JSON: {e}"))?;
        let mut runtime = self.state.lock().await;
        runtime.rules = parsed.rules.clone();
        if let Some(parent) = self.rules_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Err(e) = fs::write(&self.rules_path, rules_json) {
            return Err(format!("Failed to write rules file: {e}"));
        }
        Ok(())
    }

    pub async fn get_rules_json(&self) -> String {
        if let Ok(text) = fs::read_to_string(&self.rules_path) {
            return text;
        }
        let runtime = self.state.lock().await;
        let rules = NotificationRulesFile {
            rules: runtime.rules.clone(),
        };
        serde_json::to_string(&rules).unwrap_or_else(|_| "{\"rules\":[]}".to_string())
    }

    pub async fn remove_from_history(&self, id: &str) -> bool {
        let mut runtime = self.state.lock().await;
        if let Some(pos) = runtime.history.iter().position(|item| item.id == id) {
            runtime.history.remove(pos);
            save_history_file(&self.history_path, &runtime.history);
            let _ = self.event_tx.send(CrawlEvent::Notification(
                NotificationEvent::HistoryRemoved { id: id.to_string() },
            ));
            return true;
        }
        false
    }

    pub async fn clear_history(&self) {
        let mut runtime = self.state.lock().await;
        runtime.history.clear();
        save_history_file(&self.history_path, &runtime.history);
        let _ = self
            .event_tx
            .send(CrawlEvent::Notification(NotificationEvent::HistoryCleared));
    }

    pub async fn dismiss_popup(&self, id: &str) -> bool {
        let mut runtime = self.state.lock().await;
        if let Some(pos) = runtime.popups.iter().position(|item| item.id == id) {
            runtime.popups.remove(pos);
            let _ = self
                .event_tx
                .send(CrawlEvent::Notification(NotificationEvent::PopupRemoved {
                    id: id.to_string(),
                }));
            return true;
        }
        false
    }

    pub async fn dismiss_all_popups(&self) {
        let mut runtime = self.state.lock().await;
        let ids: Vec<String> = runtime.popups.iter().map(|item| item.id.clone()).collect();
        runtime.popups.clear();
        for id in ids {
            let _ = self
                .event_tx
                .send(CrawlEvent::Notification(NotificationEvent::PopupRemoved {
                    id,
                }));
        }
    }

    pub async fn invoke_action(&self, id: &str, action_id: &str) -> bool {
        let runtime = self.state.lock().await;
        let item = runtime
            .popups
            .iter()
            .find(|i| i.id == id)
            .or_else(|| runtime.history.iter().find(|i| i.id == id));
        let Some(item) = item else {
            return false;
        };
        let original_id = item.original_id;
        drop(runtime);

        let conn = { self.connection.lock().await.clone() };
        let Some(conn) = conn else {
            return false;
        };
        if emit_action_invoked(&conn, original_id, action_id)
            .await
            .is_err()
        {
            return false;
        }
        true
    }

    pub async fn handle_notify(
        &self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, ZValue<'_>>,
        expire_timeout: i32,
    ) -> u32 {
        let mut runtime = self.state.lock().await;

        if !runtime.policy.enabled {
            return if replaces_id == 0 {
                self.id_counter.fetch_add(1, Ordering::Relaxed)
            } else {
                replaces_id
            };
        }

        let new_id = if replaces_id != 0 {
            replaces_id
        } else {
            self.id_counter.fetch_add(1, Ordering::Relaxed)
        };
        let original_id = new_id;

        let urgency = hints
            .get("urgency")
            .and_then(|v| v.downcast_ref::<u8>().ok())
            .unwrap_or(1);

        let transient = hints
            .get("transient")
            .and_then(|v| v.downcast_ref::<bool>().ok())
            .unwrap_or(false);

        let suppress_sound = hints
            .get("suppress-sound")
            .and_then(|v| v.downcast_ref::<bool>().ok())
            .unwrap_or(false);

        let app_name = if app_name.trim().is_empty() {
            hints
                .get("desktop-entry")
                .and_then(|v| v.downcast_ref::<ZStr>().ok())
                .map(|s| s.to_string())
                .unwrap_or_default()
        } else {
            app_name
        };

        let image_hint = hints
            .get("image-path")
            .or_else(|| hints.get("image_path"))
            .or_else(|| hints.get("image"))
            .and_then(|v| v.downcast_ref::<ZStr>().ok())
            .map(|s| s.to_string())
            .unwrap_or_default();

        let actions = parse_actions(actions);

        let muted = suppress_sound
            || rule_action(&runtime.rules, &app_name, &summary, &body) == Some("mute".to_string());
        let rule = rule_action(&runtime.rules, &app_name, &summary, &body);
        if let Some(action) = rule.as_deref() {
            if action == "block" {
                return new_id;
            }
        }

        let item = NotificationItem {
            id: format!("n-{}", new_id),
            summary: summary.clone(),
            summary_markdown: summary.clone(),
            body: body.clone(),
            body_markdown: body.clone(),
            app_name: app_name.clone(),
            urgency,
            expire_timeout,
            timestamp_ms: now_ms(),
            progress: 1.0,
            original_image: if !image_hint.is_empty() {
                image_hint
            } else {
                app_icon.clone()
            },
            cached_image: String::new(),
            actions: actions.clone(),
            original_id,
            muted,
        };

        if should_save_history(&runtime.policy, urgency) && !transient {
            let max_history = runtime.policy.max_history;
            upsert_history(&mut runtime.history, item.clone(), max_history);
            save_history_file(&self.history_path, &runtime.history);
            let _ = self
                .event_tx
                .send(CrawlEvent::Notification(NotificationEvent::HistoryAdded {
                    item: item.clone(),
                }));
        }

        if runtime.do_not_disturb {
            return new_id;
        }
        if let Some(action) = rule.as_deref() {
            if action == "hide" {
                return new_id;
            }
        }

        if let Some(existing) = runtime
            .popups
            .iter_mut()
            .find(|n| n.original_id == original_id)
        {
            *existing = item.clone();
            let _ = self
                .event_tx
                .send(CrawlEvent::Notification(NotificationEvent::PopupUpdated {
                    item,
                }));
            return new_id;
        }

        if let Some(dup_id) = find_duplicate_popup(&runtime.popups, &item) {
            runtime.popups.retain(|p| p.id != dup_id);
            let _ = self
                .event_tx
                .send(CrawlEvent::Notification(NotificationEvent::PopupRemoved {
                    id: dup_id,
                }));
        }

        runtime.popups.insert(0, item.clone());
        if runtime.popups.len() > runtime.policy.max_popups {
            let max_popups = runtime.policy.max_popups;
            let removed: Vec<String> = runtime.popups.drain(max_popups..).map(|p| p.id).collect();
            for id in removed {
                let _ =
                    self.event_tx
                        .send(CrawlEvent::Notification(NotificationEvent::PopupRemoved {
                            id,
                        }));
            }
        }
        let _ = self
            .event_tx
            .send(CrawlEvent::Notification(NotificationEvent::PopupAdded {
                item,
            }));

        new_id
    }

    pub async fn close_notification(&self, id: u32) {
        let mut runtime = self.state.lock().await;
        let id_string = format!("n-{}", id);
        if let Some(pos) = runtime.popups.iter().position(|item| item.id == id_string) {
            runtime.popups.remove(pos);
            let _ = self
                .event_tx
                .send(CrawlEvent::Notification(NotificationEvent::PopupRemoved {
                    id: id_string.clone(),
                }));
        }

        let conn = { self.connection.lock().await.clone() };
        if let Some(conn) = conn {
            let _ = emit_notification_closed(&conn, id, 3).await;
        }
    }

    async fn run_expiration_loop(&self) {
        let interval = std::time::Duration::from_millis(100);
        loop {
            tokio::time::sleep(interval).await;
            let mut runtime = self.state.lock().await;
            if runtime.popups.is_empty() {
                continue;
            }
            let now = now_ms();
            let policy = runtime.policy.clone();
            let mut to_remove = Vec::new();
            for item in &mut runtime.popups {
                let duration = calculate_duration(&policy, item);
                let progress = match duration {
                    Some(d) if d > 0 => {
                        let elapsed = now.saturating_sub(item.timestamp_ms);
                        if elapsed >= d {
                            to_remove.push(item.id.clone());
                            0.0
                        } else {
                            1.0 - (elapsed as f32 / d as f32)
                        }
                    }
                    Some(_) => 0.0,
                    None => item.progress,
                };
                if (item.progress - progress).abs() > 0.005 {
                    item.progress = progress;
                    let _ = self.event_tx.send(CrawlEvent::Notification(
                        NotificationEvent::PopupUpdated { item: item.clone() },
                    ));
                }
            }
            if !to_remove.is_empty() {
                runtime.popups.retain(|p| !to_remove.contains(&p.id));
                for id in to_remove {
                    let _ = self.event_tx.send(CrawlEvent::Notification(
                        NotificationEvent::PopupRemoved { id },
                    ));
                }
            }
        }
    }
}

#[derive(Clone)]
struct NotificationDbus {
    backend: Arc<NotificationBackend>,
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl NotificationDbus {
    async fn get_capabilities(&self) -> zbus::fdo::Result<Vec<String>> {
        Ok(vec![
            "actions".into(),
            "body".into(),
            "body-markup".into(),
            "persistence".into(),
            "sound".into(),
        ])
    }

    async fn notify(
        &self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, ZValue<'_>>,
        expire_timeout: i32,
    ) -> zbus::fdo::Result<u32> {
        Ok(self
            .backend
            .handle_notify(
                app_name,
                replaces_id,
                app_icon,
                summary,
                body,
                actions,
                hints,
                expire_timeout,
            )
            .await)
    }

    async fn close_notification(&self, id: u32) -> zbus::fdo::Result<()> {
        self.backend.close_notification(id).await;
        Ok(())
    }

    async fn get_server_information(&self) -> zbus::fdo::Result<(String, String, String, String)> {
        Ok((
            "crawl".to_string(),
            "crawl-sysd".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
            "1.2".to_string(),
        ))
    }
}

fn resolve_history_path(override_path: Option<&str>) -> PathBuf {
    if let Some(path) = override_path {
        return PathBuf::from(path);
    }
    if let Ok(path) = std::env::var("CRAWL_NOTIF_HISTORY_FILE") {
        return PathBuf::from(path);
    }
    let cache_dir = std::env::var("XDG_CACHE_HOME")
        .or_else(|_| std::env::var("HOME").map(|home| format!("{home}/.cache")))
        .unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(format!("{cache_dir}/crawl/notifications.json"))
}

fn resolve_state_path(override_path: Option<&str>) -> PathBuf {
    if let Some(path) = override_path {
        return PathBuf::from(path);
    }
    let cache_dir = std::env::var("XDG_CACHE_HOME")
        .or_else(|_| std::env::var("HOME").map(|home| format!("{home}/.cache")))
        .unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(format!("{cache_dir}/crawl/notifications-state.json"))
}

fn resolve_rules_path(override_path: Option<&str>) -> PathBuf {
    if let Some(path) = override_path {
        return PathBuf::from(path);
    }
    if let Ok(path) = std::env::var("CRAWL_NOTIF_RULES_FILE") {
        return PathBuf::from(path);
    }
    let config_dir = std::env::var("CRAWL_CONFIG_DIR")
        .or_else(|_| std::env::var("XDG_CONFIG_HOME").map(|home| format!("{home}/crawl")))
        .or_else(|_| std::env::var("HOME").map(|home| format!("{home}/.config/crawl")))
        .unwrap_or_else(|_| "/tmp/crawl".to_string());
    PathBuf::from(format!("{config_dir}/notification-rules.json"))
}

fn default_policy() -> NotificationPolicy {
    NotificationPolicy {
        enabled: true,
        max_popups: 5,
        max_history: 100,
        respect_expire_timeout: false,
        low_urgency_duration_ms: 3000,
        normal_urgency_duration_ms: 8000,
        critical_urgency_duration_ms: 15000,
        save_to_history: NotificationHistoryPolicy {
            low: true,
            normal: true,
            critical: true,
        },
    }
}

fn load_history_file(path: &Path) -> Vec<NotificationItem> {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let parsed: serde_json::Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    let list = parsed
        .get("notifications")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    list.into_iter().filter_map(parse_history_item).collect()
}

fn save_history_file(path: &Path, items: &[NotificationItem]) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let payload = serde_json::json!({"notifications": items});
    if let Ok(text) = serde_json::to_string(&payload) {
        let _ = fs::write(path, text);
    }
}

fn load_state_file(path: &Path) -> Option<NotificationStateFile> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str::<NotificationStateFile>(&text).ok()
}

fn save_state_file(path: &Path, runtime: &NotificationRuntime) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let file = NotificationStateFile {
        do_not_disturb: runtime.do_not_disturb,
        last_seen_ts: runtime.last_seen_ts,
        policy: runtime.policy.clone(),
    };
    if let Ok(text) = serde_json::to_string(&file) {
        let _ = fs::write(path, text);
    }
}

fn load_rules_file(path: &Path) -> Vec<NotificationRule> {
    let text = match fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let parsed: NotificationRulesFile = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    parsed
        .rules
        .into_iter()
        .filter(|r| !r.pattern.trim().is_empty())
        .collect()
}

fn rule_action(
    rules: &[NotificationRule],
    app_name: &str,
    summary: &str,
    body: &str,
) -> Option<String> {
    let haystack = format!("{app_name} {summary} {body}");
    for rule in rules {
        let pattern = rule.pattern.trim();
        if pattern.is_empty() {
            continue;
        }
        let mut matched = false;
        if pattern.len() >= 3 && pattern.starts_with('/') && pattern.ends_with('/') {
            let raw = &pattern[1..pattern.len() - 1];
            if let Ok(re) = Regex::new(raw) {
                matched = re.is_match(&haystack);
            }
        } else if pattern.contains('*') {
            let esc = regex::escape(pattern).replace("\\*", ".*");
            if let Ok(re) = Regex::new(&esc) {
                matched = re.is_match(&haystack);
            } else {
                matched = haystack.to_lowercase().contains(&pattern.to_lowercase());
            }
        } else {
            matched = haystack.to_lowercase().contains(&pattern.to_lowercase());
        }
        if matched {
            let action = rule.action.to_lowercase();
            if action == "mute" || action == "hide" {
                return Some(action);
            }
            if action == "silence" {
                return Some("hide".to_string());
            }
            return Some("block".to_string());
        }
    }
    None
}

fn parse_actions(actions: Vec<String>) -> Vec<NotificationAction> {
    let mut out = Vec::new();
    let mut iter = actions.into_iter();
    while let Some(id) = iter.next() {
        let text = iter.next().unwrap_or_else(|| "Action".to_string());
        out.push(NotificationAction {
            identifier: id,
            text,
        });
    }
    out
}

fn should_save_history(policy: &NotificationPolicy, urgency: u8) -> bool {
    if !policy.enabled {
        return false;
    }
    match urgency {
        0 => policy.save_to_history.low,
        2 => policy.save_to_history.critical,
        _ => policy.save_to_history.normal,
    }
}

fn upsert_history(history: &mut Vec<NotificationItem>, item: NotificationItem, max_history: usize) {
    if let Some(pos) = history
        .iter()
        .position(|n| n.original_id == item.original_id && item.original_id != 0)
    {
        history[pos] = item;
    } else {
        history.insert(0, item);
    }
    let limit = if max_history == 0 { 2000 } else { max_history };
    if history.len() > limit {
        history.truncate(limit);
    }
}

fn find_duplicate_popup(popups: &[NotificationItem], item: &NotificationItem) -> Option<String> {
    for existing in popups {
        if existing.summary == item.summary
            && existing.body == item.body
            && existing.app_name == item.app_name
        {
            return Some(existing.id.clone());
        }
    }
    None
}

fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn calculate_duration(policy: &NotificationPolicy, item: &NotificationItem) -> Option<u64> {
    if policy.respect_expire_timeout {
        if item.expire_timeout == 0 {
            return None;
        }
        if item.expire_timeout > 0 {
            return Some(item.expire_timeout as u64);
        }
    }
    let duration = match item.urgency {
        0 => policy.low_urgency_duration_ms,
        2 => policy.critical_urgency_duration_ms,
        _ => policy.normal_urgency_duration_ms,
    };
    Some(duration)
}

fn parse_history_item(value: serde_json::Value) -> Option<NotificationItem> {
    if let Ok(item) = serde_json::from_value::<NotificationItem>(value.clone()) {
        return Some(item);
    }
    let obj = value.as_object()?;
    let get_str = |key: &str| {
        obj.get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let get_u8 = |key: &str| obj.get(key).and_then(|v| v.as_u64()).unwrap_or(1) as u8;
    let get_u32 = |key: &str| obj.get(key).and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let timestamp_ms = obj
        .get("timestamp")
        .and_then(|v| v.as_u64())
        .or_else(|| obj.get("timestamp_ms").and_then(|v| v.as_u64()))
        .unwrap_or_else(now_ms);
    let actions_json = obj
        .get("actionsJson")
        .and_then(|v| v.as_str())
        .unwrap_or("[]");
    let actions = serde_json::from_str::<Vec<NotificationAction>>(actions_json).unwrap_or_default();
    Some(NotificationItem {
        id: get_str("id"),
        summary: get_str("summary"),
        summary_markdown: get_str("summaryMarkdown"),
        body: get_str("body"),
        body_markdown: get_str("bodyMarkdown"),
        app_name: get_str("appName"),
        urgency: get_u8("urgency"),
        expire_timeout: obj
            .get("expireTimeout")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32,
        timestamp_ms,
        progress: obj.get("progress").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
        original_image: get_str("originalImage"),
        cached_image: get_str("cachedImage"),
        actions,
        original_id: get_u32("originalId"),
        muted: obj.get("muted").and_then(|v| v.as_bool()).unwrap_or(false),
    })
}

async fn emit_notification_closed(
    conn: &zbus::Connection,
    id: u32,
    reason: u32,
) -> zbus::Result<()> {
    conn.emit_signal(
        Option::<&str>::None,
        NOTIFICATION_PATH,
        NOTIFICATION_INTERFACE,
        "NotificationClosed",
        &(id, reason),
    )
    .await
}

async fn emit_action_invoked(
    conn: &zbus::Connection,
    id: u32,
    action_key: &str,
) -> zbus::Result<()> {
    conn.emit_signal(
        Option::<&str>::None,
        NOTIFICATION_PATH,
        NOTIFICATION_INTERFACE,
        "ActionInvoked",
        &(id, action_key),
    )
    .await
}
