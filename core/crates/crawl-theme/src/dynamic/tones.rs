/// Tone values for mapping Material You tonal palettes → Catppuccin Palette.
///
/// These are opinionated but follow the Material You convention:
/// - Dark surfaces use very low tones (4–30)
/// - Light surfaces use very high tones (90–99)
/// - Text uses the opposite extreme
/// - Accents use mid-high tones for dark (80), mid-low for light (40)

pub struct Tones {
    pub base: u8,
    pub mantle: u8,
    pub crust: u8,
    pub surface0: u8,
    pub surface1: u8,
    pub surface2: u8,
    pub overlay0: u8,
    pub overlay1: u8,
    pub overlay2: u8,
    pub text: u8,
    pub subtext0: u8,
    pub subtext1: u8,
    pub accent: u8,
}

pub const DARK: Tones = Tones {
    base: 10,
    mantle: 6,
    crust: 4,
    surface0: 12,
    surface1: 17,
    surface2: 22,
    overlay0: 30,
    overlay1: 42,
    overlay2: 55,
    text: 92,
    subtext0: 78,
    subtext1: 65,
    accent: 80,
};

pub const LIGHT: Tones = Tones {
    base: 99,
    mantle: 97,
    crust: 95,
    surface0: 94,
    surface1: 90,
    surface2: 85,
    overlay0: 75,
    overlay1: 65,
    overlay2: 55,
    text: 8,
    subtext0: 20,
    subtext1: 35,
    accent: 40,
};
