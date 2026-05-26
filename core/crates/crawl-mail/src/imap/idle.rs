use std::time::Duration;

use tokio::sync::broadcast;
use tracing::{info, warn};

use crawl_ipc::CrawlEvent;
use crawl_ipc::events::MailEvent;

use crate::imap::ImapSession;
use crate::notify::notify_new_mail;

pub async fn idle_loop(
    mut session: ImapSession,
    account_id: String,
    folder: String,
    tx: broadcast::Sender<CrawlEvent>,
) {
    loop {
        let (success, maybe_session) = session.idle_once().await;
        match maybe_session {
            Some(new_session) => {
                session = new_session;
                if success {
                    info!(%account_id, %folder, "New mail detected via IDLE");
                    let _ = notify_new_mail(&format!("New mail in {}", folder)).await;
                    let _ = tx.send(CrawlEvent::Mail(MailEvent::NewMessages {
                        account_id: account_id.clone(),
                        folder: folder.clone(),
                        count: 0,
                    }));
                } else {
                    warn!(%account_id, %folder, "IDLE error, will retry");
                    tokio::time::sleep(Duration::from_secs(15)).await;
                }
            }
            None => {
                warn!(%account_id, %folder, "IDLE session lost, exiting loop");
                return;
            }
        }
    }
}
