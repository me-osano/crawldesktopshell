use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Color(r, g, b)
    }

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some(Color(r, g, b))
    }

    pub fn hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }

    pub fn hex_stripped(&self) -> String {
        format!("{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }

    pub fn rgb_u8(&self) -> (u8, u8, u8) {
        (self.0, self.1, self.2)
    }

    pub fn rgba_f32(&self, alpha: f32) -> (f32, f32, f32, f32) {
        (
            self.0 as f32 / 255.0,
            self.1 as f32 / 255.0,
            self.2 as f32 / 255.0,
            alpha,
        )
    }

    pub fn is_dark(&self) -> bool {
        // Relative luminance approximation (ITU-R BT.709)
        let lum = 0.2126 * self.0 as f32 + 0.7152 * self.1 as f32 + 0.0722 * self.2 as f32;
        lum < 128.0
    }

    // ── HSL conversions ──

    pub fn to_hsl(&self) -> (f32, f32, f32) {
        rgb_to_hsl(self.0, self.1, self.2)
    }

    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let (r, g, b) = hsl_to_rgb(h, s, l);
        Color(r, g, b)
    }

    // ── Color helpers ──

    /// Adjust lightness to a target value (0-1).
    pub fn adjust_lightness(&self, target_l: f32) -> Self {
        let (h, s, _) = self.to_hsl();
        Color::from_hsl(h, s, target_l)
    }

    /// Shift hue by specified degrees.
    pub fn shift_hue(&self, degrees: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        let new_h = (h + degrees) % 360.0;
        Color::from_hsl(new_h, s, l)
    }

    /// Minimum angular distance between two hues (0-180).
    pub fn hue_distance(&self, other: &Color) -> f32 {
        let (h1, _, _) = self.to_hsl();
        let (h2, _, _) = other.to_hsl();
        let diff = (h1 - h2).abs();
        diff.min(360.0 - diff)
    }

    /// Derive a surface color with saturation limit and target lightness.
    pub fn adjust_surface(&self, s_max: f32, l_target: f32) -> Self {
        let (h, s, _) = self.to_hsl();
        Color::from_hsl(h, s.min(s_max), l_target)
    }

    /// Adjust saturation by amount (-1 to 1).
    pub fn saturate(&self, amount: f32) -> Self {
        let (h, s, l) = self.to_hsl();
        let new_s = (s + amount).clamp(0.0, 1.0);
        Color::from_hsl(h, new_s, l)
    }
}

// ── Standalone HSL conversion functions ──

/// Convert RGB (0-255) to HSL (0-360, 0-1, 0-1).
pub fn rgb_to_hsl(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let rn = r as f32 / 255.0;
    let gn = g as f32 / 255.0;
    let bn = b as f32 / 255.0;

    let max = rn.max(gn).max(bn);
    let min = rn.min(gn).min(bn);
    let delta = max - min;

    let l = (max + min) / 2.0;

    if delta == 0.0 {
        return (0.0, 0.0, l);
    }

    let s = if l != 0.0 && l != 1.0 {
        delta / (1.0 - (2.0 * l - 1.0).abs())
    } else {
        0.0
    };

    let h = if max == rn {
        60.0 * (((gn - bn) / delta) % 6.0)
    } else if max == gn {
        60.0 * (((bn - rn) / delta) + 2.0)
    } else {
        60.0 * (((rn - gn) / delta) + 4.0)
    };

    (h, s, l)
}

/// Convert HSL (0-360, 0-1, 0-1) to RGB (0-255).
pub fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s == 0.0 {
        let v = (l * 255.0).round() as u8;
        return (v, v, v);
    }

    let hue_to_rgb = |p: f32, q: f32, mut t: f32| -> f32 {
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
        if t < 1.0 / 2.0 { return q; }
        if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
        p
    };

    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let hn = h / 360.0;

    let r = hue_to_rgb(p, q, hn + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, hn);
    let b = hue_to_rgb(p, q, hn - 1.0 / 3.0);

    (
        (r * 255.0).round() as u8,
        (g * 255.0).round() as u8,
        (b * 255.0).round() as u8,
    )
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
    }
}

impl Serialize for Color {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.hex())
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ColorVisitor;
        impl Visitor<'_> for ColorVisitor {
            type Value = Color;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a hex color like #1e1e2e")
            }

            fn visit_str<E: de::Error>(self, v: &str) -> Result<Color, E> {
                Color::from_hex(v)
                    .ok_or_else(|| de::Error::custom(format!("invalid hex color: {}", v)))
            }
        }
        deserializer.deserialize_str(ColorVisitor)
    }
}
