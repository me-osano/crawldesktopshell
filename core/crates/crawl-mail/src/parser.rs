use mail_parser::MessageParser;

use crate::imap::ImapMessage;

#[derive(Debug, Clone, Default)]
pub struct ParsedMessage {
    pub uid: u32,
    pub message_id: Option<String>,
    pub thread_id: Option<String>,
    pub from_addr: String,
    pub from_name: Option<String>,
    pub to_addrs: Vec<String>,
    pub cc_addrs: Vec<String>,
    pub subject: Option<String>,
    pub date: String,
    pub flags: Vec<String>,
    pub snippet: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub body_fetched: bool,
    pub has_attachments: bool,
    pub raw_size: Option<i64>,
}

pub fn parse_message(raw: &ImapMessage) -> anyhow::Result<ParsedMessage> {
    let headers = match &raw.raw_headers {
        Some(h) => h,
        None => return Ok(ParsedMessage::default()),
    };

    let msg = match MessageParser::new().parse(headers) {
        Some(m) => m,
        None => return Ok(ParsedMessage::default()),
    };

    let date = msg
        .date()
        .map(|d| d.to_rfc3339())
        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

    let (from_addr, from_name) = extract_from(&msg);
    let to_addrs = extract_addrs(msg.to());
    let cc_addrs = extract_addrs(msg.cc());
    let snippet = msg.body_preview(150).map(|s| s.into_owned());

    Ok(ParsedMessage {
        uid: raw.uid.unwrap_or(0),
        message_id: msg.message_id().map(|s| s.to_string()),
        thread_id: None,
        from_addr,
        from_name,
        to_addrs,
        cc_addrs,
        subject: msg.subject().map(|s| s.to_string()),
        date,
        flags: raw.flags().unwrap_or_default(),
        snippet,
        body_text: msg.body_text(0).map(|s| s.into_owned()),
        body_html: msg.body_html(0).map(|s| s.into_owned()),
        body_fetched: false,
        has_attachments: msg.attachment_count() > 0,
        raw_size: raw.raw_size.map(|s| s as i64),
    })
}

fn extract_from(msg: &mail_parser::Message<'_>) -> (String, Option<String>) {
    match msg.from() {
        Some(addr) => {
            if let Some(first) = addr.first() {
                (
                    first.address().unwrap_or("").to_string(),
                    first
                        .name()
                        .map(|n| n.to_string())
                        .filter(|n| !n.is_empty()),
                )
            } else {
                (String::new(), None)
            }
        }
        None => (String::new(), None),
    }
}

fn extract_addrs(addr: Option<&mail_parser::Address<'_>>) -> Vec<String> {
    match addr {
        Some(a) => a
            .iter()
            .filter_map(|a| a.address().map(|a| a.to_string()))
            .collect(),
        None => vec![],
    }
}
