use crate::color::Color;
use crate::dynamic::hct::Hct;
use crate::dynamic::scheme_type::{PaletteParams, SchemeType};
use crate::dynamic::tones;
use crate::palette::Palette;

/// The five tonal palettes derived from a seed color.
/// Each palette is indexed by tone (0–100).
pub struct TonalSet {
    pub primary: Vec<Color>,
    pub secondary: Vec<Color>,
    pub tertiary: Vec<Color>,
    pub neutral: Vec<Color>,
    pub neutral_variant: Vec<Color>,
}

impl TonalSet {
    pub fn from_seed(seed: &Hct, variant: SchemeType) -> Self {
        let params = variant.palette_hue_chroma(seed);
        Self::from_params(&params)
    }

    pub fn from_params(params: &PaletteParams) -> Self {
        Self {
            primary: tonal_range(params.primary.0, params.primary.1),
            secondary: tonal_range(params.secondary.0, params.secondary.1),
            tertiary: tonal_range(params.tertiary.0, params.tertiary.1),
            neutral: tonal_range(params.neutral.0, params.neutral.1),
            neutral_variant: tonal_range(params.neutral_variant.0, params.neutral_variant.1),
        }
    }

    pub fn shade(&self, tone: u8) -> &Color {
        &self.neutral[tone.min(100) as usize]
    }
}

fn tonal_range(hue: f32, chroma: f32) -> Vec<Color> {
    let mut tones = Vec::with_capacity(101);
    for t in 0..=100 {
        let c = if t == 0 || t == 100 { 0.0 } else { chroma };
        let hct = Hct::from(hue, c, t as f32);
        let (r, g, b) = hct.to_rgb();
        tones.push(Color::new(r, g, b));
    }
    tones
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SchemeMode {
    Dark,
    Light,
}

/// Maps tonal palettes to a Catppuccin-shaped Palette using tone constants.
pub struct DynamicScheme {
    pub seed: Color,
    pub mode: SchemeMode,
    pub variant: SchemeType,
    pub tones: TonalSet,
}

impl DynamicScheme {
    pub fn from_seed(seed: Color, mode: SchemeMode, variant: SchemeType) -> Self {
        let hct = Hct::from_rgb(seed.0, seed.1, seed.2);
        Self {
            seed,
            mode,
            variant,
            tones: TonalSet::from_seed(&hct, variant),
        }
    }

    pub fn to_palette(&self) -> Palette {
        let t = match self.mode {
            SchemeMode::Dark => &tones::DARK,
            SchemeMode::Light => &tones::LIGHT,
        };

        let n = |tone: u8| self.tones.neutral[tone.min(100) as usize].clone();
        let nv = |tone: u8| self.tones.neutral_variant[tone.min(100) as usize].clone();
        let p = |tone: u8| self.tones.primary[tone.min(100) as usize].clone();
        let s = |tone: u8| self.tones.secondary[tone.min(100) as usize].clone();
        let t3 = |tone: u8| self.tones.tertiary[tone.min(100) as usize].clone();

        Palette::new(
            n(t.base),
            n(t.mantle),
            n(t.crust),
            n(t.surface0),
            n(t.surface1),
            n(t.surface2),
            nv(t.overlay0),
            nv(t.overlay1),
            nv(t.overlay2),
            n(t.text),
            n(t.subtext0),
            n(t.subtext1),
            t3(t.accent),
            p(t.accent),
            t3(t.accent),
            t3(t.accent),
            p(t.accent),
            p(t.accent),
            s(t.accent),
            p(t.accent),
            s(t.accent),
            t3(t.accent),
            t3(t.accent),
            s(t.accent),
            p(t.accent),
            t3(t.accent),
        )
    }

    pub fn is_dark(&self) -> bool {
        matches!(self.mode, SchemeMode::Dark)
    }
}
