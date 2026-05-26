use async_imap::types::NameAttribute;
use async_native_tls::TlsConnector;
use tokio_stream::StreamExt;

type TlsTcpStream = async_native_tls::TlsStream<async_std::net::TcpStream>;

#[derive(Debug, Clone)]
pub struct ImapFolder {
    pub name: String,
    pub display_name: String,
    pub kind: String,
}

#[derive(Debug, Clone)]
pub struct ImapMessage {
    pub uid: Option<u32>,
    pub raw_size: Option<u32>,
    pub raw_headers: Option<Vec<u8>>,
    flags: Vec<String>,
}

impl ImapMessage {
    pub fn flags(&self) -> Option<Vec<String>> {
        if self.flags.is_empty() {
            None
        } else {
            Some(self.flags.clone())
        }
    }
}

pub struct ImapSession {
    session: Option<async_imap::Session<TlsTcpStream>>,
    selected: Option<String>,
    uidvalidity: u32,
    uidnext: u32,
}

impl ImapSession {
    fn s_mut(&mut self) -> &mut async_imap::Session<TlsTcpStream> {
        self.session.as_mut().expect("IMAP session not connected")
    }

    pub async fn connect(
        host: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> anyhow::Result<Self> {
        let tcp = async_std::net::TcpStream::connect((host, port))
            .await
            .map_err(|e| anyhow::anyhow!("TCP connect failed: {}", e))?;
        let tls = TlsConnector::new();
        let tls_stream = tls
            .connect(host, tcp)
            .await
            .map_err(|e| anyhow::anyhow!("TLS handshake failed: {}", e))?;
        let mut client = async_imap::Client::new(tls_stream);
        let _greeting = client
            .read_response()
            .await
            .ok_or_else(|| anyhow::anyhow!("Connection closed before greeting"))?
            .map_err(|e| anyhow::anyhow!("Greeting error: {}", e))?;
        let session = client
            .login(username, password)
            .await
            .map_err(|(e, _)| anyhow::anyhow!("IMAP login failed: {}", e))?;
        Ok(Self {
            session: Some(session),
            selected: None,
            uidvalidity: 0,
            uidnext: 0,
        })
    }

    pub async fn list(
        &mut self,
        reference: &str,
        pattern: &str,
    ) -> anyhow::Result<Vec<ImapFolder>> {
        let mut stream = self.s_mut().list(Some(reference), Some(pattern)).await?;
        let mut mailboxes = Vec::new();
        while let Some(mb) = stream.next().await {
            let mb = mb?;
            mailboxes.push(mb);
        }
        Ok(mailboxes
            .into_iter()
            .map(|mb| {
                let name = mb.name().to_string();
                let display_name = name
                    .rsplit_once(mb.delimiter().unwrap_or("/"))
                    .map(|(_, last)| last.to_string())
                    .unwrap_or_else(|| name.clone());
                let kind = classify_folder(&name, mb.attributes());
                ImapFolder {
                    name,
                    display_name,
                    kind,
                }
            })
            .collect())
    }

    pub async fn select(&mut self, folder: &str) -> anyhow::Result<()> {
        let mailbox = self.s_mut().select(folder).await?;
        self.selected = Some(folder.to_string());
        self.uidvalidity = mailbox.uid_validity.unwrap_or(0);
        self.uidnext = mailbox.uid_next.unwrap_or(0);
        Ok(())
    }

    pub async fn uid_validity_and_next(&mut self) -> anyhow::Result<(u32, u32)> {
        Ok((self.uidvalidity, self.uidnext))
    }

    pub async fn uid_fetch(
        &mut self,
        range: &str,
        query: &str,
    ) -> anyhow::Result<Vec<ImapMessage>> {
        let mut stream = self.s_mut().uid_fetch(range, query).await?;
        let mut messages = Vec::new();
        while let Some(result) = stream.next().await {
            let fetch = result?;

            let flags: Vec<String> = fetch.flags().map(flag_to_string).collect();

            messages.push(ImapMessage {
                uid: fetch.uid,
                raw_size: fetch.size,
                raw_headers: fetch.header().map(|h| h.to_vec()),
                flags,
            });
        }
        Ok(messages)
    }

    pub async fn fetch_raw_body(&mut self, uid: u32) -> anyhow::Result<Option<Vec<u8>>> {
        let range = uid.to_string();
        let mut stream = self.s_mut().uid_fetch(range, "(BODY[])").await?;
        while let Some(result) = stream.next().await {
            let fetch = result?;
            return Ok(fetch.body().map(|b| b.to_vec()));
        }
        Ok(None)
    }

    pub async fn idle_once(mut self) -> (bool, Option<Self>) {
        let session = self.session.take().expect("IMAP session not connected");
        let mut handle = session.idle();
        let _ = handle.init().await;
        let (wait_fut, _interrupt) = handle.wait();
        let _ = wait_fut.await;
        match handle.done().await {
            Ok(session) => {
                self.session = Some(session);
                (true, Some(self))
            }
            Err(e) => {
                tracing::warn!("IDLE done failed: {}", e);
                (false, None)
            }
        }
    }
}

fn classify_folder<'a>(name: &str, attributes: &[NameAttribute<'a>]) -> String {
    let name_upper = name.to_uppercase();
    if name_upper == "INBOX" {
        return "inbox".to_string();
    }
    for attr in attributes {
        match attr {
            async_imap::types::NameAttribute::Sent => return "sent".to_string(),
            async_imap::types::NameAttribute::Drafts => return "drafts".to_string(),
            async_imap::types::NameAttribute::Trash => return "trash".to_string(),
            async_imap::types::NameAttribute::Junk => return "spam".to_string(),
            async_imap::types::NameAttribute::Archive => return "archive".to_string(),
            _ => {}
        }
    }
    "custom".to_string()
}

fn flag_to_string(flag: async_imap::types::Flag<'_>) -> String {
    match flag {
        async_imap::types::Flag::Seen => "\\Seen".to_string(),
        async_imap::types::Flag::Answered => "\\Answered".to_string(),
        async_imap::types::Flag::Flagged => "\\Flagged".to_string(),
        async_imap::types::Flag::Deleted => "\\Deleted".to_string(),
        async_imap::types::Flag::Draft => "\\Draft".to_string(),
        async_imap::types::Flag::Recent => "\\Recent".to_string(),
        async_imap::types::Flag::MayCreate => "\\*".to_string(),
        async_imap::types::Flag::Custom(s) => s.to_string(),
    }
}
