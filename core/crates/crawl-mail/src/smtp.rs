use std::time::Duration;

use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    message::{Mailbox, MultiPart, SinglePart},
    transport::smtp::authentication::Credentials,
};
use tracing::{error, info, warn};

use crate::store::Store;

pub struct SmtpQueue {
    store: Store,
    poll_interval: Duration,
}

impl SmtpQueue {
    pub fn new(store: Store) -> Self {
        Self {
            store,
            poll_interval: Duration::from_secs(10),
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        loop {
            if let Err(e) = self.process_queue().await {
                warn!(err = ?e, "SMTP queue processing error");
            }
            tokio::time::sleep(self.poll_interval).await;
        }
    }

    async fn process_queue(&self) -> anyhow::Result<()> {
        let items = self.store.outbox().list_queued(10).await?;
        if items.is_empty() {
            return Ok(());
        }

        for item in &items {
            if item.smtp.password.is_empty() {
                warn!(
                    outbox_id = %item.id,
                    account_id = %item.payload.account_id,
                    "Skipping outbox entry: no SMTP password configured"
                );
                self.store
                    .outbox()
                    .mark_status(&item.id, "failed", Some("No SMTP password"))
                    .await?;
                continue;
            }

            self.store
                .outbox()
                .mark_status(&item.id, "sending", None)
                .await?;

            match send_single(&item.payload, &item.smtp).await {
                Ok(_) => {
                    info!(outbox_id = %item.id, subject = %item.payload.subject, "Message sent");
                    self.store
                        .outbox()
                        .mark_status(&item.id, "sent", None)
                        .await?;
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    error!(outbox_id = %item.id, err = %err_msg, "Failed to send message");
                    self.store
                        .outbox()
                        .mark_status(&item.id, "failed", Some(&err_msg))
                        .await?;
                }
            }
        }

        Ok(())
    }
}

async fn send_single(
    msg: &crawl_ipc::types::SendMessage,
    smtp: &crate::store::accounts::AccountSmtp,
) -> anyhow::Result<()> {
    let from: Mailbox = msg
        .from
        .parse()
        .map_err(|e: lettre::address::AddressError| {
            anyhow::anyhow!("Invalid from address '{}': {}", msg.from, e)
        })?;

    let mut email = Message::builder().from(from).subject(&msg.subject);

    for addr in &msg.to {
        let m: Mailbox = addr.parse()?;
        email = email.to(m);
    }
    for addr in &msg.cc {
        let m: Mailbox = addr.parse()?;
        email = email.cc(m);
    }
    for addr in &msg.bcc {
        let m: Mailbox = addr.parse()?;
        email = email.bcc(m);
    }

    if !msg.attachments.is_empty() {
        warn!(
            count = msg.attachments.len(),
            "SMTP attachments not yet implemented"
        );
    }

    let email = if let Some(html) = &msg.body_html {
        email.multipart(
            MultiPart::alternative()
                .singlepart(SinglePart::plain(msg.body_text.clone()))
                .singlepart(SinglePart::html(html.clone())),
        )?
    } else {
        email.body(msg.body_text.clone())?
    };

    let creds = Credentials::new(smtp.username.clone(), smtp.password.clone());
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&smtp.host)?
        .port(smtp.port)
        .credentials(creds)
        .build();

    mailer.send(email).await?;
    Ok(())
}
