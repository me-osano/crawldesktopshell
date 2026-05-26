use crate::dynamic::hct::Hct;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemeType {
    TonalSpot,
    Content,
    Monochrome,
    Vibrant,
    Faithful,
    FruitSalad,
    Rainbow,
    Muted,
}

impl SchemeType {
    pub fn all() -> &'static [SchemeType] {
        &[
            SchemeType::TonalSpot,
            SchemeType::Content,
            SchemeType::Monochrome,
            SchemeType::Vibrant,
            SchemeType::Faithful,
            SchemeType::FruitSalad,
            SchemeType::Rainbow,
            SchemeType::Muted,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SchemeType::TonalSpot => "tonal_spot",
            SchemeType::Content => "content",
            SchemeType::Monochrome => "monochrome",
            SchemeType::Vibrant => "vibrant",
            SchemeType::Faithful => "faithful",
            SchemeType::FruitSalad => "fruit_salad",
            SchemeType::Rainbow => "rainbow",
            SchemeType::Muted => "muted",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "tonal_spot" | "tonal-spot" => Some(SchemeType::TonalSpot),
            "content" => Some(SchemeType::Content),
            "monochrome" => Some(SchemeType::Monochrome),
            "vibrant" => Some(SchemeType::Vibrant),
            "faithful" => Some(SchemeType::Faithful),
            "fruit_salad" | "fruit-salad" => Some(SchemeType::FruitSalad),
            "rainbow" => Some(SchemeType::Rainbow),
            "muted" => Some(SchemeType::Muted),
            _ => None,
        }
    }

    pub fn palette_hue_chroma(&self, source: &Hct) -> PaletteParams {
        let h = source.get_hue();
        let c = source.get_chroma();
        match self {
            SchemeType::TonalSpot => PaletteParams {
                primary: (h, 48.0),
                secondary: (h, 16.0),
                tertiary: ((h + 60.0) % 360.0, 24.0),
                neutral: (h, 4.0),
                neutral_variant: (h, 8.0),
            },
            SchemeType::Content => PaletteParams {
                primary: (h, c.max(8.0)),
                secondary: (h, (c - 32.0).max(c * 0.5).max(4.0)),
                tertiary: ((h + 60.0) % 360.0, (c * 0.5).max(4.0)),
                neutral: (h, (c / 8.0).max(1.0)),
                neutral_variant: (h, (c / 8.0 + 4.0).max(2.0)),
            },
            SchemeType::Monochrome => PaletteParams {
                primary: (h, 0.0),
                secondary: (h, 0.0),
                tertiary: (h, 0.0),
                neutral: (h, 0.0),
                neutral_variant: (h, 0.0),
            },
            SchemeType::Vibrant => PaletteParams {
                primary: (h, c.max(48.0)),
                secondary: ((h + 30.0).rem_euclid(360.0), c.max(36.0)),
                tertiary: ((h + 60.0).rem_euclid(360.0), c.max(24.0)),
                neutral: (h, 8.0),
                neutral_variant: (h, 12.0),
            },
            SchemeType::Faithful => PaletteParams {
                primary: (h, c.max(48.0)),
                secondary: ((h + 30.0).rem_euclid(360.0), c.max(24.0)),
                tertiary: ((h + 60.0).rem_euclid(360.0), c.max(24.0)),
                neutral: (h, 8.0),
                neutral_variant: (h, 12.0),
            },
            SchemeType::FruitSalad => PaletteParams {
                primary: ((h - 50.0).rem_euclid(360.0), 48.0),
                secondary: ((h - 50.0).rem_euclid(360.0), 36.0),
                tertiary: (h, 36.0),
                neutral: (h, 10.0),
                neutral_variant: (h, 16.0),
            },
            SchemeType::Rainbow => PaletteParams {
                primary: (h, 48.0),
                secondary: (h, 16.0),
                tertiary: ((h + 60.0) % 360.0, 24.0),
                neutral: (h, 4.0),
                neutral_variant: (h, 8.0),
            },
            SchemeType::Muted => PaletteParams {
                primary: (h, 35.0),
                secondary: (h, 24.0),
                tertiary: (h, 20.0),
                neutral: (h, 6.0),
                neutral_variant: (h, 10.0),
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PaletteParams {
    pub primary: (f32, f32),
    pub secondary: (f32, f32),
    pub tertiary: (f32, f32),
    pub neutral: (f32, f32),
    pub neutral_variant: (f32, f32),
}
