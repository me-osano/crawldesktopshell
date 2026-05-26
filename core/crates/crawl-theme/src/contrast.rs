use crate::color::Color;

fn linearize(c: u8) -> f32 {
    let c_norm = c as f32 / 255.0;
    if c_norm <= 0.03928 {
        c_norm / 12.92
    } else {
        ((c_norm + 0.055) / 1.055).powf(2.4)
    }
}

pub fn relative_luminance(c: &Color) -> f32 {
    let r = linearize(c.0);
    let g = linearize(c.1);
    let b = linearize(c.2);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

pub fn contrast_ratio(a: &Color, b: &Color) -> f32 {
    let l1 = relative_luminance(a);
    let l2 = relative_luminance(b);
    let lighter = l1.max(l2);
    let darker = l1.min(l2);
    (lighter + 0.05) / (darker + 0.05)
}

pub fn is_dark(c: &Color) -> bool {
    relative_luminance(c) < 0.179
}

pub fn ensure_contrast(
    foreground: &Color,
    background: &Color,
    min_ratio: f32,
    prefer_light: Option<bool>,
) -> Color {
    if contrast_ratio(foreground, background) >= min_ratio {
        return *foreground;
    }

    let (h, s, l) = foreground.to_hsl();
    let bg_dark = is_dark(background);
    let prefer_light = prefer_light.unwrap_or(bg_dark);

    let (mut low, mut high) = if prefer_light {
        (l, 1.0)
    } else {
        (0.0, l)
    };

    let mut best = *foreground;
    for _ in 0..20 {
        let mid = (low + high) / 2.0;
        let test = Color::from_hsl(h, s, mid);
        let ratio = contrast_ratio(&test, background);

        if ratio >= min_ratio {
            best = test;
            if prefer_light { high = mid; } else { low = mid; }
        } else {
            if prefer_light { low = mid; } else { high = mid; }
        }
    }

    best
}



pub fn get_contrasting_color(background: &Color, min_ratio: f32) -> Color {
    let fg = if is_dark(background) {
        Color(243, 237, 247)
    } else {
        Color(14, 14, 67)
    };
    ensure_contrast(&fg, background, min_ratio, None)
}
