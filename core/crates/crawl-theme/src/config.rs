use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::color::Color;
use crate::dynamic::scheme::SchemeMode;
use crate::dynamic::{DynamicScheme, SchemeType};
use crate::error::ThemeResult;
use crate::palette::Palette;
use crate::variants;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemeVariant {
    Mocha,
    Latte,
    Frappe,
    Macchiato,
    Nord,
    TokyoNight,
    GruvboxDark,
    GruvboxLight,
    KanagawaDark,
    KanagawaLight,
    RosePineDark,
    RosePineLight,
    /// Theme generated dynamically from a seed color (e.g. from wallpaper).
    /// Stores the seed hex so it survives daemon restart.
    /// variant is the dynamic scheme type (tonal_spot, content, etc.).
    Dynamic {
        seed: String,
        mode: String,
        variant: Option<String>,
    },
    #[serde(untagged)]
    Custom(String),
}

impl ThemeVariant {
    pub fn as_str(&self) -> &str {
        match self {
            ThemeVariant::Mocha => "mocha",
            ThemeVariant::Latte => "latte",
            ThemeVariant::Frappe => "frappe",
            ThemeVariant::Macchiato => "macchiato",
            ThemeVariant::Nord => "nord",
            ThemeVariant::TokyoNight => "tokyo-night",
            ThemeVariant::GruvboxDark => "gruvbox-dark",
            ThemeVariant::GruvboxLight => "gruvbox-light",
            ThemeVariant::KanagawaDark => "kanagawa-dark",
            ThemeVariant::KanagawaLight => "kanagawa-light",
            ThemeVariant::RosePineDark => "rose-pine-dark",
            ThemeVariant::RosePineLight => "rose-pine-light",
            ThemeVariant::Dynamic { .. } => "dynamic",
            ThemeVariant::Custom(name) => name.as_str(),
        }
    }

    pub fn is_dark(&self) -> bool {
        match self {
            ThemeVariant::Latte => false,
            ThemeVariant::GruvboxLight => false,
            ThemeVariant::KanagawaLight => false,
            ThemeVariant::RosePineLight => false,
            ThemeVariant::Dynamic { mode, .. } if mode == "light" => false,
            _ => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccentColor {
    Rosewater,
    Flamingo,
    Pink,
    Mauve,
    Red,
    Maroon,
    Peach,
    Yellow,
    Green,
    Teal,
    Sky,
    Sapphire,
    Blue,
    Lavender,
    #[serde(untagged)]
    Custom(String),
}

impl AccentColor {
    pub fn as_str(&self) -> &str {
        match self {
            AccentColor::Rosewater => "rosewater",
            AccentColor::Flamingo => "flamingo",
            AccentColor::Pink => "pink",
            AccentColor::Mauve => "mauve",
            AccentColor::Red => "red",
            AccentColor::Maroon => "maroon",
            AccentColor::Peach => "peach",
            AccentColor::Yellow => "yellow",
            AccentColor::Green => "green",
            AccentColor::Teal => "teal",
            AccentColor::Sky => "sky",
            AccentColor::Sapphire => "sapphire",
            AccentColor::Blue => "blue",
            AccentColor::Lavender => "lavender",
            AccentColor::Custom(s) => s.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorConfig {
    pub theme: String,
    pub size: u32,
}

impl Default for CursorConfig {
    fn default() -> Self {
        Self {
            theme: "catppuccin-mocha-dark-cursors".into(),
            size: 24,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GtkConfig {
    pub prefer_dark: bool,
    pub theme: Option<String>,
    pub icon_theme: Option<String>,
    pub font_name: Option<String>,
    pub font_size: Option<u32>,
}

impl Default for GtkConfig {
    fn default() -> Self {
        Self {
            prefer_dark: true,
            theme: None,
            icon_theme: None,
            font_name: None,
            font_size: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub variant: ThemeVariant,
    pub accent: AccentColor,
    #[serde(default)]
    pub cursor: CursorConfig,
    #[serde(default)]
    pub gtk: GtkConfig,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            variant: ThemeVariant::Mocha,
            accent: AccentColor::Mauve,
            cursor: CursorConfig::default(),
            gtk: GtkConfig::default(),
        }
    }
}

impl ThemeConfig {
    pub fn load(path: &Path) -> ThemeResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> ThemeResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn resolve_palette(&self) -> Palette {
        match &self.variant {
            ThemeVariant::Mocha => variants::catppuccin_mocha(),
            ThemeVariant::Latte => variants::catppuccin_latte(),
            ThemeVariant::Frappe => variants::catppuccin_frappe(),
            ThemeVariant::Macchiato => variants::catppuccin_macchiato(),
            ThemeVariant::Nord => variants::nord(),
            ThemeVariant::TokyoNight => variants::tokyo_night(),
            ThemeVariant::GruvboxDark => variants::gruvbox_dark(),
            ThemeVariant::GruvboxLight => variants::gruvbox_light(),
            ThemeVariant::KanagawaDark => variants::kanagawa_dark(),
            ThemeVariant::KanagawaLight => variants::kanagawa_light(),
            ThemeVariant::RosePineDark => variants::rose_pine_dark(),
            ThemeVariant::RosePineLight => variants::rose_pine_light(),
            ThemeVariant::Dynamic {
                seed,
                mode,
                variant,
            } => {
                let color = Color::from_hex(seed).unwrap_or(Color::new(0x42, 0x85, 0xF4));
                let m = if mode == "light" {
                    SchemeMode::Light
                } else {
                    SchemeMode::Dark
                };
                let v = variant
                    .as_deref()
                    .and_then(SchemeType::from_str)
                    .unwrap_or(SchemeType::TonalSpot);
                DynamicScheme::from_seed(color, m, v).to_palette()
            }
            ThemeVariant::Custom(path) => {
                let p = Path::new(path);
                if p.exists() {
                    variants::custom::load_from_toml(p)
                        .unwrap_or_else(|_| variants::catppuccin_mocha())
                } else {
                    variants::catppuccin_mocha()
                }
            }
        }
    }

    pub fn accent_color(&self) -> Color {
        let palette = self.resolve_palette();
        match &self.accent {
            AccentColor::Custom(hex) => {
                Color::from_hex(hex).unwrap_or_else(|| palette.mauve.clone())
            }
            _ => palette
                .accent(self.accent.as_str())
                .cloned()
                .unwrap_or_else(|| palette.mauve.clone()),
        }
    }

    pub fn is_dark(&self) -> bool {
        self.variant.is_dark()
    }
}
