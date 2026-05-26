use std::sync::Arc;
use std::time::Duration;

use tokio::sync::broadcast;
use tracing::{error, info, warn};

use crawl_ipc::CrawlEvent;
use crawl_ipc::events::MailEvent;
use crawl_ipc::types::SyncStatusKind;

use crate::imap::ImapSession;
use crate::imap::sync::SyncEngine;
use crate::store::Store;
use crate::store::accounts::AccountImap;

pub struct AccountManager {
    store: Arc<Store>,
    events: broadcast::Sender<CrawlEvent>,
}

impl AccountManager {
    pub async fn sync_now(&self, account_id: &str) -> anyhow::Result<()> {
        let imap = self
            .store
            .accounts()
            .get_imap(account_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No IMAP config for {}", account_id))?;

        let _ = self.events.send(CrawlEvent::Mail(MailEvent::SyncStatus {
            account_id: account_id.to_string(),
            status: SyncStatusKind::Running,
        }));

        match try_sync(account_id, &self.store, &imap).await {
            Ok(()) => {
                let _ = self.events.send(CrawlEvent::Mail(MailEvent::SyncComplete {
                    account_id: account_id.to_string(),
                }));
                let _ = self.events.send(CrawlEvent::Mail(MailEvent::SyncStatus {
                    account_id: account_id.to_string(),
                    status: SyncStatusKind::Idle,
                }));
                Ok(())
            }
            Err(e) => {
                let _ = self.events.send(CrawlEvent::Mail(MailEvent::SyncStatus {
                    account_id: account_id.to_string(),
                    status: SyncStatusKind::Error,
                }));
                Err(e)
            }
        }
    }

    pub fn new(store: Arc<Store>, events: broadcast::Sender<CrawlEvent>) -> Self {
        Self { store, events }
    }

    pub async fn start_all(&self) -> anyhow::Result<()> {
        let accounts = self.store.accounts().list_accounts().await?;
        if accounts.is_empty() {
            info!("No mail accounts configured");
            return Ok(());
        }

        for acct in accounts {
            let imap = match self.store.accounts().get_imap(&acct.id).await? {
                Some(imap) => imap,
                None => {
                    warn!(account_id = %acct.id, "IMAP config not found, skipping");
                    continue;
                }
            };

            if imap.password.is_empty() {
                warn!(account_id = %acct.id, "No password set, skipping sync");
                continue;
            }

            info!(account_id = %acct.id, email = %acct.email, "Starting sync loop");

            let store = Arc::clone(&self.store);
            let events = self.events.clone();
            let aid = acct.id.clone();

            tokio::spawn(async move {
                run_sync_loop(aid, store, events, imap).await;
            });
        }

        Ok(())
    }
}

async fn run_sync_loop(
    account_id: String,
    store: Arc<Store>,
    events: broadcast::Sender<CrawlEvent>,
    imap: AccountImap,
) {
    let sync_interval = Duration::from_secs(300);

    loop {
        let _ = events.send(CrawlEvent::Mail(MailEvent::SyncStatus {
            account_id: account_id.clone(),
            status: SyncStatusKind::Running,
        }));

        match try_sync(&account_id, &store, &imap).await {
            Ok(()) => {
                info!(account_id = %account_id, "Sync complete");
                let _ = events.send(CrawlEvent::Mail(MailEvent::SyncComplete {
                    account_id: account_id.clone(),
                }));
            }
            Err(e) => {
                error!(account_id = %account_id, err = ?e, "Sync failed");
                let _ = events.send(CrawlEvent::Mail(MailEvent::SyncStatus {
                    account_id: account_id.clone(),
                    status: SyncStatusKind::Error,
                }));
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue;
            }
        }

        let _ = events.send(CrawlEvent::Mail(MailEvent::SyncStatus {
            account_id: account_id.clone(),
            status: SyncStatusKind::Idle,
        }));

        tokio::time::sleep(sync_interval).await;
    }
}

async fn try_sync(account_id: &str, store: &Store, imap: &AccountImap) -> anyhow::Result<()> {
    let mut session =
        ImapSession::connect(&imap.host, imap.port, &imap.username, &imap.password).await?;

    let engine = SyncEngine {
        account_id: account_id.to_string(),
        store: store.clone(),
    };

    engine.full_sync(&mut session).await?;
    Ok(())
}
