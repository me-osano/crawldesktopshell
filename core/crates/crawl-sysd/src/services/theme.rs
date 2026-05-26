use std::path::{Path, PathBuf};

use async_trait::async_trait;
use tokio::sync::Mutex;

use crawl_ipc::protocol::CrawlResponse;
use crawl_theme::apply::terminal::write_terminal_themes;
use crawl_theme::dynamic::generate::expand_predefined_scheme;
use crawl_theme::{AccentColor, Color, ThemeChangedEvent, ThemeConfig, ThemeVariant};

use crate::services::models::{Service, error_response};
use crate::state::SharedState;

pub struct ThemeService {
    state: SharedState,
    config: Mutex<ThemeConfig>,
    config_path: PathBuf,
    colors_json_path: PathBuf,
}

impl ThemeService {
    pub fn new(state: SharedState) -> Self {
        let cache_dir = state.config.theme.cache_dir.clone();
        Self {
            colors_json_path: PathBuf::from(&state.config.theme.qml_colors_path),
            state,
            config: Mutex::new(ThemeConfig::default()),
            config_path: PathBuf::from(&cache_dir).join("theme.toml"),
        }
    }

    fn resolve_and_apply(
        config: &ThemeConfig,
        colors_json_path: &Path,
    ) -> crawl_theme::ThemeResult<ThemeChangedEvent> {
        let palette = config.resolve_palette();
        let accent = config.accent_color();

        // Write colors.json for QML Theme.qml to pick up
        if let Err(e) = crawl_theme::apply::colors_json::write(colors_json_path, &palette, &accent) {
            tracing::warn!("Failed to write colors.json: {}", e);
        }

        // Apply system settings
        let res_gtk = crawl_theme::apply::gtk::apply(&config.gtk, &config.cursor, &palette);
        let res_qt = crawl_theme::apply::qt::apply(&config.gtk, &palette, &accent);
        let res_cursor = crawl_theme::apply::cursor::apply(&config.cursor);
        if let Err(e) = res_gtk {
            tracing::warn!("Failed to apply GTK theme: {}", e);
        }
        if let Err(e) = res_qt {
            tracing::warn!("Failed to apply Qt theme: {}", e);
        }
        if let Err(e) = res_cursor {
            tracing::warn!("Failed to apply cursor theme: {}", e);
        }
        #[cfg(target_os = "linux")]
        if let Err(e) = crawl_theme::apply::niri::apply(&palette, &accent) {
            tracing::warn!("Failed to apply niri theme: {}", e);
        }

        Ok(ThemeChangedEvent::from_config(config, &palette, &accent))
    }
}

#[async_trait]
impl Service for ThemeService {
    fn name(&self) -> &'static str {
        "theme"
    }

    async fn start(&self) -> anyhow::Result<()> {
        // Try loading saved config, fall back to defaults
        let config = if self.config_path.exists() {
            ThemeConfig::load(&self.config_path).unwrap_or_default()
        } else {
            ThemeConfig::default()
        };

        let event = Self::resolve_and_apply(&config, &self.colors_json_path)
            .map_err(|e| anyhow::anyhow!("Failed to apply initial theme: {}", e))?;

        *self.config.lock().await = config;

        // Persist and broadcast
        let _ = self.config.lock().await.save(&self.config_path);
        let _ = self.state.event_bus.publish(crawl_ipc::CrawlEvent::Theme(
            crawl_ipc::ThemeEvent::Changed {
                name: event.variant.clone(),
                source: "config".into(),
                scheme: event.accent.clone(),
                palette: event.palette.clone(),
            },
        ));

        Ok(())
    }

    async fn handle(
        &self,
        method: &str,
        params: &serde_json::Value,
        id: Option<serde_json::Value>,
    ) -> Option<CrawlResponse> {
        let response = match method {
            "ThemeGet" => {
                let guard = self.config.lock().await;
                let palette = guard.resolve_palette();
                let accent = guard.accent_color();
                let event = ThemeChangedEvent::from_config(&guard, &palette, &accent);
                CrawlResponse::success(id, serde_json::to_value(&event).unwrap_or_default())
            }
            "ThemeList" => {
                let variants = vec![
                    "mocha",
                    "latte",
                    "frappe",
                    "macchiato",
                    "nord",
                    "tokyo-night",
                    "gruvbox-dark",
                    "gruvbox-light",
                    "kanagawa-dark",
                    "kanagawa-light",
                    "rose-pine-dark",
                    "rose-pine-light",
                ];
                CrawlResponse::success(id, serde_json::to_value(&variants).unwrap_or_default())
            }
            "ThemeSet" => {
                let name = params
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let accent = params
                    .get("accent")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                let mut guard = self.config.lock().await;

                guard.variant = match name.to_lowercase().as_str() {
                    "mocha" => ThemeVariant::Mocha,
                    "latte" => ThemeVariant::Latte,
                    "frappe" => ThemeVariant::Frappe,
                    "macchiato" => ThemeVariant::Macchiato,
                    "nord" | "nordic" => ThemeVariant::Nord,
                    "tokyo-night" | "tokyonight" => ThemeVariant::TokyoNight,
                    "gruvbox-dark" | "gruvbox_dark" => ThemeVariant::GruvboxDark,
                    "gruvbox-light" | "gruvbox_light" | "gruvbox" => ThemeVariant::GruvboxLight,
                    "kanagawa-dark" | "kanagawa_dark" => ThemeVariant::KanagawaDark,
                    "kanagawa-light" | "kanagawa_light" | "kanagawa" => ThemeVariant::KanagawaLight,
                    "rose-pine-dark" | "rose_pine_dark" => ThemeVariant::RosePineDark,
                    "rose-pine-light" | "rose_pine_light" | "rose-pine" | "rosepine" => {
                        ThemeVariant::RosePineLight
                    }
                    other => ThemeVariant::Custom(other.to_string()),
                };

                if let Some(a) = accent {
                    guard.accent = match a.to_lowercase().as_str() {
                        "rosewater" => AccentColor::Rosewater,
                        "flamingo" => AccentColor::Flamingo,
                        "pink" => AccentColor::Pink,
                        "mauve" => AccentColor::Mauve,
                        "red" => AccentColor::Red,
                        "maroon" => AccentColor::Maroon,
                        "peach" => AccentColor::Peach,
                        "yellow" => AccentColor::Yellow,
                        "green" => AccentColor::Green,
                        "teal" => AccentColor::Teal,
                        "sky" => AccentColor::Sky,
                        "sapphire" => AccentColor::Sapphire,
                        "blue" => AccentColor::Blue,
                        "lavender" => AccentColor::Lavender,
                        hex => AccentColor::Custom(hex.to_string()),
                    };
                }

                let palette = guard.resolve_palette();
                let accent_color = guard.accent_color();
                let event = ThemeChangedEvent::from_config(&guard, &palette, &accent_color);

                // Persist, apply, broadcast
                let _ = guard.save(&self.config_path);
                let _ = Self::resolve_and_apply(&guard, &self.colors_json_path);
                let _ = self.state.event_bus.publish(crawl_ipc::CrawlEvent::Theme(
                    crawl_ipc::ThemeEvent::Changed {
                        name: event.variant.clone(),
                        source: "config".into(),
                        scheme: event.accent.clone(),
                        palette: event.palette.clone(),
                    },
                ));

                CrawlResponse::success(id, serde_json::to_value(&event).unwrap_or_default())
            }
            "ThemeGenerate" => {
                let color_hex = params.get("color").and_then(|v| v.as_str()).unwrap_or("");
                let scheme_type_str = params.get("scheme").and_then(|v| v.as_str());
                let mode_str = params.get("mode").and_then(|v| v.as_str());
                let seed = Color::from_hex(color_hex).unwrap_or(Color::new(0x42, 0x85, 0xF4));
                let mode = match mode_str {
                    Some("light") => "light",
                    _ => "dark",
                };
                let scheme_variant = scheme_type_str
                    .and_then(crawl_theme::SchemeType::from_str)
                    .unwrap_or(crawl_theme::SchemeType::TonalSpot);

                let mut guard = self.config.lock().await;
                guard.variant = ThemeVariant::Dynamic {
                    seed: seed.hex(),
                    mode: mode.into(),
                    variant: Some(scheme_variant.as_str().into()),
                };
                guard.accent = AccentColor::Custom(seed.hex());

                let palette = guard.resolve_palette();
                let accent_color = guard.accent_color();
                let event = ThemeChangedEvent::from_config(&guard, &palette, &accent_color);

                let _ = guard.save(&self.config_path);
                let _ = Self::resolve_and_apply(&guard, &self.colors_json_path);
                let _ = self.state.event_bus.publish(crawl_ipc::CrawlEvent::Theme(
                    crawl_ipc::ThemeEvent::Changed {
                        name: event.variant.clone(),
                        source: "generate".into(),
                        scheme: event.accent.clone(),
                        palette: event.palette.clone(),
                    },
                ));

                CrawlResponse::success(id, serde_json::to_value(&event).unwrap_or_default())
            }
            "ThemeGenerateFromImage" => {
                let image_path = params.get("path").and_then(|v| v.as_str()).unwrap_or("");
                let scheme_type_str = params.get("scheme").and_then(|v| v.as_str());
                let mode_str = params.get("mode").and_then(|v| v.as_str());
                let path = Path::new(image_path);

                let seed = match crawl_theme::dynamic::extract::dominant_color(path) {
                    Ok(c) => c,
                    Err(e) => {
                        return Some(error_response(
                            id,
                            format!("Image extraction failed: {}", e),
                        ));
                    }
                };

                let mode = match mode_str {
                    Some("light") => "light",
                    _ => "dark",
                };
                let scheme_variant = scheme_type_str
                    .and_then(crawl_theme::SchemeType::from_str)
                    .unwrap_or(crawl_theme::SchemeType::TonalSpot);

                let mut guard = self.config.lock().await;
                guard.variant = ThemeVariant::Dynamic {
                    seed: seed.hex(),
                    mode: mode.into(),
                    variant: Some(scheme_variant.as_str().into()),
                };
                guard.accent = AccentColor::Custom(seed.hex());

                let palette = guard.resolve_palette();
                let accent_color = guard.accent_color();
                let event = ThemeChangedEvent::from_config(&guard, &palette, &accent_color);

                let _ = guard.save(&self.config_path);
                let _ = Self::resolve_and_apply(&guard, &self.colors_json_path);
                let _ = self.state.event_bus.publish(crawl_ipc::CrawlEvent::Theme(
                    crawl_ipc::ThemeEvent::Changed {
                        name: event.variant.clone(),
                        source: "generate-from-image".into(),
                        scheme: event.accent.clone(),
                        palette: event.palette.clone(),
                    },
                ));

                CrawlResponse::success(id, serde_json::to_value(&event).unwrap_or_default())
            }
            "ThemeGenerateFromPredefined" => {
                let scheme_json = params
                    .get("scheme_json")
                    .and_then(|v| v.as_str())
                    .unwrap_or("{}");
                let mode = params
                    .get("mode")
                    .and_then(|v| v.as_str())
                    .unwrap_or("dark");

                let scheme_data: std::collections::HashMap<String, String> =
                    serde_json::from_str(scheme_json).unwrap_or_default();

                let palette = expand_predefined_scheme(&scheme_data, mode);

                CrawlResponse::success(id, serde_json::to_value(&palette).unwrap_or_default())
            }
            "ThemeGenerateTerminal" => {
                let scheme_json = params
                    .get("scheme_json")
                    .and_then(|v| v.as_str())
                    .unwrap_or("{}");
                let outputs_json = params
                    .get("outputs_json")
                    .and_then(|v| v.as_str())
                    .unwrap_or("{}");

                let scheme_data: serde_json::Value =
                    serde_json::from_str(scheme_json).unwrap_or_default();
                let outputs: std::collections::HashMap<String, String> =
                    serde_json::from_str(outputs_json).unwrap_or_default();

                match write_terminal_themes(&scheme_data, &outputs) {
                    Ok(()) => CrawlResponse::success(id, serde_json::json!({"ok": true})),
                    Err(e) => {
                        return Some(crate::services::models::error_response(
                            id,
                            format!("Terminal generation failed: {}", e),
                        ));
                    }
                }
            }
            _ => return None,
        };

        Some(response)
    }
}
