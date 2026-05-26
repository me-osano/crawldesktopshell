use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::color::Color;
use crate::config::{ThemeConfig, ThemeVariant};
use crate::palette::Palette;

/// Event payload pushed over the IPC socket when the theme changes.
/// QML's Theme singleton receives this and updates all color properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeChangedEvent {
    /// Human-readable variant label, e.g. "mocha", "custom:gruvbox"
    pub variant: String,
    /// Hex accent color, e.g. "#cba6f7"
    pub accent: String,
    /// Flat string-keyed palette map. QML does `palette["base"]` etc.
    pub palette: HashMap<String, String>,
}

impl ThemeChangedEvent {
    pub fn from_config(config: &ThemeConfig, palette: &Palette, accent: &Color) -> Self {
        let var = match &config.variant {
            ThemeVariant::Custom(name) => format!("custom:{}", name),
            v => v.as_str().to_string(),
        };

        let mut p = palette.to_map();
        p.insert("accent".into(), accent.hex());

        Self {
            variant: var,
            accent: accent.hex(),
            palette: p,
        }
    }
}

impl From<(&ThemeConfig, &Palette)> for ThemeChangedEvent {
    fn from((config, palette): (&ThemeConfig, &Palette)) -> Self {
        let accent = config.accent_color();
        Self::from_config(config, palette, &accent)
    }
}
