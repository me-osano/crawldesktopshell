use crate::{imap::ImapSession, parser::parse_message, store::Store};
use tracing::{info, warn};

pub struct SyncEngine {
    pub account_id: String,
    pub store: Store,
}

impl SyncEngine {
    pub async fn full_sync(&self, session: &mut ImapSession) -> anyhow::Result<()> {
        info!(account = %self.account_id, "Starting full sync");
        let folders = session.list("", "*").await?;

        for folder in &folders {
            self.store.upsert_folder(&self.account_id, folder).await?;
            if let Err(e) = self.sync_folder(session, &folder.name).await {
                warn!(folder = %folder.name, err = ?e, "Folder sync failed, skipping");
            }
        }
        Ok(())
    }

    pub async fn sync_folder(&self, session: &mut ImapSession, folder: &str) -> anyhow::Result<()> {
        session.select(folder).await?;

        let (uid_validity, uid_next) = session.uid_validity_and_next().await?;
        let stored = self
            .store
            .get_folder_state(&self.account_id, folder)
            .await?;

        if stored.map(|s| s.uidvalidity) != Some(uid_validity) {
            self.store
                .clear_folder_messages(&self.account_id, folder)
                .await?;
            self.fetch_uid_range(session, folder, 1, uid_next).await?;
            return Ok(());
        }

        let last_uid = self
            .store
            .max_uid(&self.account_id, folder)
            .await?
            .unwrap_or(0);
        if uid_next > last_uid + 1 {
            self.fetch_uid_range(session, folder, last_uid + 1, uid_next)
                .await?;
        }

        self.sync_flags(session, folder).await?;

        self.store
            .update_folder_state(&self.account_id, folder, uid_validity, uid_next)
            .await?;
        Ok(())
    }

    async fn fetch_uid_range(
        &self,
        session: &mut ImapSession,
        folder: &str,
        from: u32,
        to: u32,
    ) -> anyhow::Result<()> {
        let range = format!("{}:{}", from, to);
        let messages = session
            .uid_fetch(&range, "(UID FLAGS RFC822.SIZE ENVELOPE BODY.PEEK[HEADER])")
            .await?;

        for raw in messages {
            match parse_message(&raw) {
                Ok(msg) => {
                    self.store
                        .upsert_message(&self.account_id, folder, &msg)
                        .await?;
                }
                Err(e) => warn!(uid = raw.uid, err = ?e, "Parse error"),
            }
        }
        Ok(())
    }

    async fn sync_flags(&self, session: &mut ImapSession, folder: &str) -> anyhow::Result<()> {
        let flag_map = session.uid_fetch("1:*", "FLAGS").await?;
        for item in flag_map {
            if let (Some(uid), Some(flags)) = (item.uid, item.flags()) {
                self.store
                    .update_flags(&self.account_id, folder, uid, flags)
                    .await?;
            }
        }
        Ok(())
    }
}
