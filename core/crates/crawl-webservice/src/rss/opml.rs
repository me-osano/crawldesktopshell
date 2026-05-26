use std::collections::HashMap;

use anyhow::Result;
use quick_xml::Reader;
use quick_xml::events::Event;
use tracing::warn;

use crate::rss::store::Store;

pub struct OpmlImporter<'a> {
    store: &'a Store,
}

impl<'a> OpmlImporter<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }

    pub async fn import(&self, xml: &str) -> Result<(u32, u32, u32)> {
        let outlines = parse_outlines(xml)?;
        let total = outlines.len() as u32;
        let mut imported = 0u32;
        let mut failed = 0u32;

        for outline in outlines {
            match self.store.get_feed_id_by_url(&outline.url).await {
                Ok(Some(_)) => {
                    // Already exists
                }
                Ok(None) => match self.store.add_feed(&outline.url, &outline.category).await {
                    Ok(_) => imported += 1,
                    Err(e) => {
                        warn!("Failed to import feed {}: {e}", outline.url);
                        failed += 1;
                    }
                },
                Err(e) => {
                    warn!("Error checking feed {}: {e}", outline.url);
                    failed += 1;
                }
            }
        }

        Ok((total, imported, failed))
    }
}

struct Outline {
    url: String,
    category: String,
}

fn parse_outlines(xml: &str) -> Result<Vec<Outline>> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut outlines = Vec::new();
    let mut buf = Vec::new();
    let mut categories: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                if e.name().as_ref() == b"outline" {
                    let mut url = String::new();
                    let mut title = String::new();
                    let mut category = String::new();

                    for attr in e.attributes().flatten() {
                        match attr.key.as_ref() {
                            b"xmlUrl" | b"xmlurl" => {
                                url = String::from_utf8_lossy(&attr.value).to_string();
                            }
                            b"text" | b"title" => {
                                title = String::from_utf8_lossy(&attr.value).to_string();
                            }
                            b"category" => {
                                category = String::from_utf8_lossy(&attr.value).to_string();
                            }
                            _ => {}
                        }
                    }

                    // If no explicit category, use the last category in the stack
                    if category.is_empty() && !categories.is_empty() {
                        category = categories.last().cloned().unwrap_or_default();
                    }

                    // If it has nested outlines, it's a category folder
                    if url.is_empty() {
                        // Use the text/title as the category name
                        let cat_name = if !title.is_empty() {
                            title
                        } else {
                            String::new()
                        };
                        if !cat_name.is_empty() {
                            categories.push(cat_name);
                        }
                    } else {
                        outlines.push(Outline { url, category });
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"outline" {
                    categories.pop();
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                warn!("OPML parse warning: {e}");
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(outlines)
}

pub fn generate_opml(feeds: &[crawl_ipc::types::FeedInfo]) -> String {
    let mut xml = String::new();
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push_str(r#"<opml version="1.0">"#);
    xml.push_str("<head><title>CrawlDS RSS Feeds</title></head>");
    xml.push_str("<body>");

    // Group by category
    let mut cat_map: HashMap<String, Vec<&crawl_ipc::types::FeedInfo>> = HashMap::new();
    for feed in feeds {
        let cat = if feed.category.is_empty() {
            "Uncategorized".to_string()
        } else {
            feed.category.clone()
        };
        cat_map.entry(cat).or_default().push(feed);
    }

    for (category, feeds) in &cat_map {
        if category == "Uncategorized" {
            for feed in feeds {
                xml.push_str(&format!(
                    r#"<outline text="{}" title="{}" type="rss" xmlUrl="{}" htmlUrl="{}"/>"#,
                    feed.title, feed.title, feed.url, feed.site_url
                ));
            }
        } else {
            xml.push_str(&format!(r#"<outline text="{}">"#, category));
            for feed in feeds {
                xml.push_str(&format!(
                    r#"<outline text="{}" title="{}" type="rss" xmlUrl="{}" htmlUrl="{}"/>"#,
                    feed.title, feed.title, feed.url, feed.site_url
                ));
            }
            xml.push_str("</outline>");
        }
    }

    xml.push_str("</body></opml>");
    xml
}
