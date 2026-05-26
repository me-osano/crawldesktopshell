use anyhow::Result;
use std::time::Instant;

use crawl_ipc::types::{WallhavenMeta, WallhavenSearchParams, WallhavenWallpaper};

const API_BASE: &str = "https://wallhaven.cc/api/v1";

pub struct WallhavenClient {
    client: reqwest::Client,
    api_key: String,
    /// Rate limiting: track requests per minute
    rate_per_min: u32,
    request_times: Vec<Instant>,
}

impl WallhavenClient {
    pub fn new(api_key: String, rate_per_min: u32) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("CrawlDS/0.1 Wallhaven")
            .timeout(std::time::Duration::from_secs(15))
            .gzip(true)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            api_key,
            rate_per_min,
            request_times: Vec::new(),
        }
    }

    pub fn set_api_key(&mut self, key: String) {
        self.api_key = key;
    }

    pub fn has_api_key(&self) -> bool {
        !self.api_key.is_empty()
    }

    async fn enforce_rate_limit(&mut self) {
        let now = Instant::now();
        let window = std::time::Duration::from_secs(60);

        // Remove requests older than 1 minute
        self.request_times
            .retain(|t| now.duration_since(*t) < window);

        if self.request_times.len() as u32 >= self.rate_per_min {
            // Sleep until oldest request falls out of the window
            if let Some(oldest) = self.request_times.first() {
                let elapsed = now.duration_since(*oldest);
                if elapsed < window {
                    tokio::time::sleep(window - elapsed).await;
                }
            }
        }

        self.request_times.push(Instant::now());
    }

    pub async fn search(&mut self, params: &WallhavenSearchParams) -> Result<WhApiResponse> {
        let mut retry = true;

        while retry {
            retry = false;
            self.enforce_rate_limit().await;

            let mut query_params = Vec::new();

            if !params.query.is_empty() {
                query_params.push(("q", params.query.as_str()));
            }
            query_params.push(("categories", params.categories.as_str()));
            query_params.push(("purity", params.purity.as_str()));
            query_params.push(("sorting", params.sorting.as_str()));
            query_params.push(("order", params.order.as_str()));

            if params.sorting == "toplist" {
                if let Some(ref range) = params.top_range {
                    query_params.push(("topRange", range.as_str()));
                }
            }

            if params.sorting == "random" {
                if let Some(ref seed) = params.seed {
                    if !seed.is_empty() {
                        query_params.push(("seed", seed.as_str()));
                    }
                }
            }

            if let Some(ref atleast) = params.atleast {
                if !atleast.is_empty() {
                    query_params.push(("atleast", atleast.as_str()));
                }
            }

            if let Some(ref resolutions) = params.resolutions {
                if !resolutions.is_empty() {
                    query_params.push(("resolutions", resolutions.as_str()));
                }
            }

            if let Some(ref ratios) = params.ratios {
                if !ratios.is_empty() {
                    query_params.push(("ratios", ratios.as_str()));
                }
            }

            if let Some(ref colors) = params.colors {
                if !colors.is_empty() {
                    query_params.push(("colors", colors.as_str()));
                }
            }

            if !self.api_key.is_empty() {
                query_params.push(("apikey", self.api_key.as_str()));
            }

            let page_str = params.page.to_string();
            query_params.push(("page", &page_str));

            let url = format!("{API_BASE}/search");
            let response = self.client.get(&url).query(&query_params).send().await?;

            let status = response.status();
            match status.as_u16() {
                200 => {
                    let raw: serde_json::Value = response.json().await?;
                    let data: Vec<WallhavenWallpaper> =
                        serde_json::from_value(raw.get("data").cloned().unwrap_or_default())
                            .unwrap_or_default();
                    let meta: WallhavenMeta =
                        serde_json::from_value(raw.get("meta").cloned().unwrap_or_default())
                            .unwrap_or(WallhavenMeta {
                                current_page: 1,
                                last_page: 1,
                                per_page: 24,
                                total: 0,
                                seed: None,
                            });

                    return Ok(WhApiResponse::Success {
                        results: data,
                        meta,
                    });
                }
                429 => {
                    tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                    retry = true;
                }
                401 => return Ok(WhApiResponse::AuthError("Invalid API Key".to_string())),
                code => return Ok(WhApiResponse::HttpError(code, response.text().await?)),
            }
        }

        Ok(WhApiResponse::HttpError(429, "Rate limited".to_string()))
    }
}

#[derive(Debug)]
pub enum WhApiResponse {
    Success {
        results: Vec<WallhavenWallpaper>,
        meta: WallhavenMeta,
    },
    AuthError(String),
    HttpError(u16, String),
}
