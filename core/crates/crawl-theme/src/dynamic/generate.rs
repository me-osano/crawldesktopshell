use std::collections::HashMap;

use crate::color::Color;
use crate::contrast::ensure_contrast;
use crate::dynamic::quantizer::strategies::find_error_color;

const MIN_HUE_DISTANCE: f32 = 30.0;

fn pick_secondary(palette: &[Color], primary: &Color) -> Color {
    let primary_h = primary.to_hsl().0;
    if palette.len() > 1 {
        let sec_h = palette[1].to_hsl().0;
        if hue_distance(primary_h, sec_h) > MIN_HUE_DISTANCE {
            return palette[1];
        }
    }
    primary.shift_hue(30.0)
}

fn pick_tertiary(palette: &[Color], primary: &Color, secondary: &Color) -> Color {
    let primary_h = primary.to_hsl().0;
    let sec_h = secondary.to_hsl().0;
    if palette.len() > 2 {
        let ter_h = palette[2].to_hsl().0;
        if hue_distance(primary_h, ter_h) > MIN_HUE_DISTANCE
            && hue_distance(sec_h, ter_h) > MIN_HUE_DISTANCE
        {
            return palette[2];
        }
    }
    primary.shift_hue(60.0)
}

fn hue_distance(h1: f32, h2: f32) -> f32 {
    let diff = (h1 - h2).abs();
    diff.min(360.0 - diff)
}

fn make_container_dark(base: &Color) -> Color {
    let (h, s, l) = base.to_hsl();
    Color::from_hsl(h, (s + 0.15).min(1.0), (l - 0.35).max(0.15))
}

fn make_container_light(base: &Color) -> Color {
    let (h, s, l) = base.to_hsl();
    Color::from_hsl(h, (s - 0.20).max(0.30), (l + 0.35).min(0.85))
}

fn make_fixed_dark(base: &Color) -> (Color, Color) {
    let (h, s, _) = base.to_hsl();
    let fixed = Color::from_hsl(h, s.max(0.70), 0.85);
    let fixed_dim = Color::from_hsl(h, s.max(0.65), 0.75);
    (fixed, fixed_dim)
}

fn make_fixed_light(base: &Color) -> (Color, Color) {
    let (h, s, _) = base.to_hsl();
    let fixed = Color::from_hsl(h, s.max(0.70), 0.40);
    let fixed_dim = Color::from_hsl(h, s.max(0.65), 0.30);
    (fixed, fixed_dim)
}

fn surface_hue_from(palette: &[Color]) -> f32 {
    let (mut hue, _, _) = palette[0].to_hsl();
    if 160.0 <= hue && hue <= 200.0 {
        hue = (hue + 10.0) % 360.0;
    }
    hue
}

fn collect_theme_map(
    primary: &Color, on_primary: &Color, primary_container: &Color, on_primary_container: &Color,
    primary_fixed: &Color, primary_fixed_dim: &Color, on_primary_fixed: &Color, on_primary_fixed_variant: &Color,
    secondary: &Color, on_secondary: &Color, secondary_container: &Color, on_secondary_container: &Color,
    secondary_fixed: &Color, secondary_fixed_dim: &Color, on_secondary_fixed: &Color, on_secondary_fixed_variant: &Color,
    tertiary: &Color, on_tertiary: &Color, tertiary_container: &Color, on_tertiary_container: &Color,
    tertiary_fixed: &Color, tertiary_fixed_dim: &Color, on_tertiary_fixed: &Color, on_tertiary_fixed_variant: &Color,
    error: &Color, on_error: &Color, error_container: &Color, on_error_container: &Color,
    surface: &Color, on_surface: &Color, surface_variant: &Color, on_surface_variant: &Color,
    surface_dim: &Color, surface_bright: &Color,
    surface_container_lowest: &Color, surface_container_low: &Color, surface_container: &Color,
    surface_container_high: &Color, surface_container_highest: &Color,
    outline: &Color, outline_variant: &Color, shadow: &Color, scrim: &Color,
    inverse_surface: &Color, inverse_on_surface: &Color, inverse_primary: &Color,
    background: &Color, on_background: &Color,
    surface_tint: &Color,
) -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("primary".into(), primary.hex());
    m.insert("on_primary".into(), on_primary.hex());
    m.insert("primary_container".into(), primary_container.hex());
    m.insert("on_primary_container".into(), on_primary_container.hex());
    m.insert("primary_fixed".into(), primary_fixed.hex());
    m.insert("primary_fixed_dim".into(), primary_fixed_dim.hex());
    m.insert("on_primary_fixed".into(), on_primary_fixed.hex());
    m.insert("on_primary_fixed_variant".into(), on_primary_fixed_variant.hex());
    m.insert("surface_tint".into(), surface_tint.hex());
    m.insert("secondary".into(), secondary.hex());
    m.insert("on_secondary".into(), on_secondary.hex());
    m.insert("secondary_container".into(), secondary_container.hex());
    m.insert("on_secondary_container".into(), on_secondary_container.hex());
    m.insert("secondary_fixed".into(), secondary_fixed.hex());
    m.insert("secondary_fixed_dim".into(), secondary_fixed_dim.hex());
    m.insert("on_secondary_fixed".into(), on_secondary_fixed.hex());
    m.insert("on_secondary_fixed_variant".into(), on_secondary_fixed_variant.hex());
    m.insert("tertiary".into(), tertiary.hex());
    m.insert("on_tertiary".into(), on_tertiary.hex());
    m.insert("tertiary_container".into(), tertiary_container.hex());
    m.insert("on_tertiary_container".into(), on_tertiary_container.hex());
    m.insert("tertiary_fixed".into(), tertiary_fixed.hex());
    m.insert("tertiary_fixed_dim".into(), tertiary_fixed_dim.hex());
    m.insert("on_tertiary_fixed".into(), on_tertiary_fixed.hex());
    m.insert("on_tertiary_fixed_variant".into(), on_tertiary_fixed_variant.hex());
    m.insert("error".into(), error.hex());
    m.insert("on_error".into(), on_error.hex());
    m.insert("error_container".into(), error_container.hex());
    m.insert("on_error_container".into(), on_error_container.hex());
    m.insert("surface".into(), surface.hex());
    m.insert("on_surface".into(), on_surface.hex());
    m.insert("surface_variant".into(), surface_variant.hex());
    m.insert("on_surface_variant".into(), on_surface_variant.hex());
    m.insert("surface_dim".into(), surface_dim.hex());
    m.insert("surface_bright".into(), surface_bright.hex());
    m.insert("surface_container_lowest".into(), surface_container_lowest.hex());
    m.insert("surface_container_low".into(), surface_container_low.hex());
    m.insert("surface_container".into(), surface_container.hex());
    m.insert("surface_container_high".into(), surface_container_high.hex());
    m.insert("surface_container_highest".into(), surface_container_highest.hex());
    m.insert("outline".into(), outline.hex());
    m.insert("outline_variant".into(), outline_variant.hex());
    m.insert("shadow".into(), shadow.hex());
    m.insert("scrim".into(), scrim.hex());
    m.insert("inverse_surface".into(), inverse_surface.hex());
    m.insert("inverse_on_surface".into(), inverse_on_surface.hex());
    m.insert("inverse_primary".into(), inverse_primary.hex());
    m.insert("background".into(), background.hex());
    m.insert("on_background".into(), on_background.hex());
    m
}

/// Generate wallust-style dark theme from palette.
pub fn generate_normal_dark(palette: &[Color]) -> HashMap<String, String> {
    let primary = palette.first().copied().unwrap_or(Color(255, 245, 155));
    let primary_h = primary.to_hsl().0;

    let secondary = pick_secondary(palette, &primary);
    let tertiary = pick_tertiary(palette, &primary, &secondary);
    let error = find_error_color(palette);

    // Keep colors vibrant - preserve saturation
    let (h, s, l) = primary.to_hsl();
    let primary_adjusted = Color::from_hsl(h, s.max(0.7), l.max(0.65));

    let (h, s, l) = secondary.to_hsl();
    let secondary_adjusted = Color::from_hsl(h, s.max(0.6), l.max(0.60));

    let (h, s, l) = tertiary.to_hsl();
    let tertiary_adjusted = Color::from_hsl(h, s.max(0.5), l.max(0.60));

    let primary_container = make_container_dark(&primary_adjusted);
    let secondary_container = make_container_dark(&secondary_adjusted);
    let tertiary_container = make_container_dark(&tertiary_adjusted);
    let error_container = make_container_dark(&error);

    // Surface
    let surface_hue = surface_hue_from(palette);
    let surface_sat_cap = if surface_hue < 60.0 || surface_hue > 300.0 {
        0.35
    } else if surface_hue < 120.0 {
        0.50
    } else {
        0.90
    };
    let base_surface = Color::from_hsl(surface_hue, palette[0].to_hsl().1.min(surface_sat_cap), 0.5);

    let surface = base_surface.adjust_surface(surface_sat_cap, 0.12);
    let surface_variant = base_surface.adjust_surface(surface_sat_cap.min(0.80), 0.16);

    let surface_container_lowest = base_surface.adjust_surface(0.85, 0.06);
    let surface_container_low = base_surface.adjust_surface(0.85, 0.10);
    let surface_container = base_surface.adjust_surface(0.70, 0.20);
    let surface_container_high = base_surface.adjust_surface(0.75, 0.18);
    let surface_container_highest = base_surface.adjust_surface(0.70, 0.22);

    // Text colors
    let text_h = palette[0].to_hsl().0;
    let base_on_surface = Color::from_hsl(text_h, 0.05, 0.95);
    let on_surface = ensure_contrast(&base_on_surface, &surface, 4.5, None);

    let base_on_surface_variant = Color::from_hsl(text_h, 0.05, 0.70);
    let on_surface_variant = ensure_contrast(&base_on_surface_variant, &surface_variant, 4.5, None);

    let outline = ensure_contrast(&palette[0].adjust_surface(0.10, 0.30), &surface, 3.0, None);
    let outline_variant = ensure_contrast(&palette[0].adjust_surface(0.10, 0.40), &surface, 3.0, None);

    // Contrasting foregrounds
    let dark_fg = Color::from_hsl(palette[0].to_hsl().0, 0.20, 0.12);
    let on_primary = ensure_contrast(&dark_fg, &primary_adjusted, 7.0, None);
    let on_secondary = ensure_contrast(&dark_fg, &secondary_adjusted, 7.0, None);
    let on_tertiary = ensure_contrast(&dark_fg, &tertiary_adjusted, 7.0, None);
    let on_error = ensure_contrast(&dark_fg, &error, 7.0, None);

    // "On" colors for containers
    let on_primary_container = ensure_contrast(&Color::from_hsl(primary_h, 0.0, 0.90), &primary_container, 4.5, Some(true));
    let sec_h = secondary.to_hsl().0;
    let on_secondary_container = ensure_contrast(&Color::from_hsl(sec_h, 0.0, 0.90), &secondary_container, 4.5, Some(true));
    let ter_h = tertiary.to_hsl().0;
    let on_tertiary_container = ensure_contrast(&Color::from_hsl(ter_h, 0.0, 0.90), &tertiary_container, 4.5, Some(true));
    let err_h = error.to_hsl().0;
    let on_error_container = ensure_contrast(&Color::from_hsl(err_h, 0.0, 0.90), &error_container, 4.5, Some(true));

    // Shadow and scrim
    let shadow = surface;
    let scrim = Color(0, 0, 0);

    // Inverse colors
    let inv_h = palette[0].to_hsl().0;
    let inverse_surface = Color::from_hsl(inv_h, 0.08, 0.90);
    let inverse_on_surface = Color::from_hsl(inv_h, 0.05, 0.15);
    let inverse_primary = Color::from_hsl(primary_h, (primary_adjusted.to_hsl().1 * 0.8).max(0.5), 0.40);

    // Background aliases
    let background = surface;
    let on_background = on_surface;

    // Fixed colors
    let (primary_fixed, primary_fixed_dim) = make_fixed_dark(&primary_adjusted);
    let (secondary_fixed, secondary_fixed_dim) = make_fixed_dark(&secondary_adjusted);
    let (tertiary_fixed, tertiary_fixed_dim) = make_fixed_dark(&tertiary_adjusted);

    let on_primary_fixed = ensure_contrast(&Color::from_hsl(primary_h, 0.15, 0.15), &primary_fixed, 4.5, None);
    let on_primary_fixed_variant = ensure_contrast(&Color::from_hsl(primary_h, 0.15, 0.20), &primary_fixed_dim, 4.5, None);
    let on_secondary_fixed = ensure_contrast(&Color::from_hsl(secondary.to_hsl().0, 0.15, 0.15), &secondary_fixed, 4.5, None);
    let on_secondary_fixed_variant = ensure_contrast(&Color::from_hsl(secondary.to_hsl().0, 0.15, 0.20), &secondary_fixed_dim, 4.5, None);
    let on_tertiary_fixed = ensure_contrast(&Color::from_hsl(tertiary.to_hsl().0, 0.15, 0.15), &tertiary_fixed, 4.5, None);
    let on_tertiary_fixed_variant = ensure_contrast(&Color::from_hsl(tertiary.to_hsl().0, 0.15, 0.20), &tertiary_fixed_dim, 4.5, None);

    // Surface dim/bright
    let surface_dim = base_surface.adjust_surface(0.85, 0.08);
    let surface_bright = base_surface.adjust_surface(0.75, 0.24);

    collect_theme_map(
        &primary_adjusted, &on_primary, &primary_container, &on_primary_container,
        &primary_fixed, &primary_fixed_dim, &on_primary_fixed, &on_primary_fixed_variant,
        &secondary_adjusted, &on_secondary, &secondary_container, &on_secondary_container,
        &secondary_fixed, &secondary_fixed_dim, &on_secondary_fixed, &on_secondary_fixed_variant,
        &tertiary_adjusted, &on_tertiary, &tertiary_container, &on_tertiary_container,
        &tertiary_fixed, &tertiary_fixed_dim, &on_tertiary_fixed, &on_tertiary_fixed_variant,
        &error, &on_error, &error_container, &on_error_container,
        &surface, &on_surface, &surface_variant, &on_surface_variant,
        &surface_dim, &surface_bright,
        &surface_container_lowest, &surface_container_low, &surface_container,
        &surface_container_high, &surface_container_highest,
        &outline, &outline_variant, &shadow, &scrim,
        &inverse_surface, &inverse_on_surface, &inverse_primary,
        &background, &on_background,
        &primary_adjusted,
    )
}

/// Generate wallust-style light theme from palette.
pub fn generate_normal_light(palette: &[Color]) -> HashMap<String, String> {
    let primary = palette.first().copied().unwrap_or(Color(93, 101, 245));
    let primary_h = primary.to_hsl().0;

    let secondary = pick_secondary(palette, &primary);
    let tertiary = pick_tertiary(palette, &primary, &secondary);
    let error = find_error_color(palette);

    // Keep colors vibrant - darken for visibility on light bg
    let (h, s, l) = primary.to_hsl();
    let primary_adjusted = Color::from_hsl(h, s.max(0.7), l.min(0.45).max(0.25));

    let (h, s, l) = secondary.to_hsl();
    let secondary_adjusted = Color::from_hsl(h, s.max(0.6), l.min(0.40).max(0.22));

    let (h, s, l) = tertiary.to_hsl();
    let tertiary_adjusted = Color::from_hsl(h, s.max(0.5), l.min(0.35).max(0.20));

    // Container colors - lighter, less saturated
    let primary_container = make_container_light(&primary_adjusted);
    let secondary_container = make_container_light(&secondary_adjusted);
    let tertiary_container = make_container_light(&tertiary_adjusted);
    let error_container = make_container_light(&error);

    // Surface: colorful light
    let surface = palette[0].adjust_surface(0.90, 0.90);
    let surface_variant = palette[0].adjust_surface(0.80, 0.78);

    let surface_container_lowest = palette[0].adjust_surface(0.85, 0.96);
    let surface_container_low = palette[0].adjust_surface(0.85, 0.92);
    let surface_container = palette[0].adjust_surface(0.80, 0.86);
    let surface_container_high = palette[0].adjust_surface(0.75, 0.84);
    let surface_container_highest = palette[0].adjust_surface(0.70, 0.80);

    // Text colors
    let text_h = palette[0].to_hsl().0;
    let base_on_surface = Color::from_hsl(text_h, 0.05, 0.10);
    let on_surface = ensure_contrast(&base_on_surface, &surface, 4.5, None);

    let base_on_surface_variant = Color::from_hsl(text_h, 0.05, 0.35);
    let on_surface_variant = ensure_contrast(&base_on_surface_variant, &surface_variant, 4.5, None);

    // Contrasting foregrounds
    let light_fg = Color::from_hsl(text_h, 0.1, 0.98);
    let on_primary = ensure_contrast(&light_fg, &primary_adjusted, 7.0, None);
    let on_secondary = ensure_contrast(&light_fg, &secondary_adjusted, 7.0, None);
    let on_tertiary = ensure_contrast(&light_fg, &tertiary_adjusted, 7.0, None);
    let on_error = ensure_contrast(&light_fg, &error, 7.0, None);

    // "On" colors for containers
    let on_primary_container = ensure_contrast(&Color::from_hsl(primary_h, 0.0, 0.15), &primary_container, 4.5, Some(false));
    let sec_h = secondary.to_hsl().0;
    let on_secondary_container = ensure_contrast(&Color::from_hsl(sec_h, 0.0, 0.15), &secondary_container, 4.5, Some(false));
    let ter_h = tertiary.to_hsl().0;
    let on_tertiary_container = ensure_contrast(&Color::from_hsl(ter_h, 0.0, 0.15), &tertiary_container, 4.5, Some(false));
    let err_h = error.to_hsl().0;
    let on_error_container = ensure_contrast(&Color::from_hsl(err_h, 0.0, 0.15), &error_container, 4.5, Some(false));

    // Outline
    let surface_h = palette[0].to_hsl().0;
    let surface_s = palette[0].to_hsl().1;
    let outline = ensure_contrast(&Color::from_hsl(surface_h, (surface_s * 0.4).max(0.25), 0.65), &surface, 3.0, None);
    let outline_variant = ensure_contrast(&Color::from_hsl(surface_h, (surface_s * 0.3).max(0.20), 0.75), &surface, 3.0, None);
    let shadow = Color::from_hsl(surface_h, (surface_s * 0.3).max(0.15), 0.80);
    let scrim = Color(0, 0, 0);

    // Inverse colors
    let inverse_surface = Color::from_hsl(surface_h, 0.08, 0.15);
    let inverse_on_surface = Color::from_hsl(surface_h, 0.05, 0.90);
    let inverse_primary = Color::from_hsl(primary_h, (primary_adjusted.to_hsl().1 * 0.8).max(0.5), 0.70);

    // Background aliases
    let background = surface;
    let on_background = on_surface;

    // Fixed colors
    let (primary_fixed, primary_fixed_dim) = make_fixed_light(&primary_adjusted);
    let (secondary_fixed, secondary_fixed_dim) = make_fixed_light(&secondary_adjusted);
    let (tertiary_fixed, tertiary_fixed_dim) = make_fixed_light(&tertiary_adjusted);

    let on_primary_fixed = ensure_contrast(&Color::from_hsl(primary_h, 0.15, 0.90), &primary_fixed, 4.5, None);
    let on_primary_fixed_variant = ensure_contrast(&Color::from_hsl(primary_h, 0.15, 0.85), &primary_fixed_dim, 4.5, None);
    let on_secondary_fixed = ensure_contrast(&Color::from_hsl(secondary.to_hsl().0, 0.15, 0.90), &secondary_fixed, 4.5, None);
    let on_secondary_fixed_variant = ensure_contrast(&Color::from_hsl(secondary.to_hsl().0, 0.15, 0.85), &secondary_fixed_dim, 4.5, None);
    let on_tertiary_fixed = ensure_contrast(&Color::from_hsl(tertiary.to_hsl().0, 0.15, 0.90), &tertiary_fixed, 4.5, None);
    let on_tertiary_fixed_variant = ensure_contrast(&Color::from_hsl(tertiary.to_hsl().0, 0.15, 0.85), &tertiary_fixed_dim, 4.5, None);

    // Surface dim/bright
    let surface_dim = palette[0].adjust_surface(0.85, 0.82);
    let surface_bright = palette[0].adjust_surface(0.90, 0.95);

    collect_theme_map(
        &primary_adjusted, &on_primary, &primary_container, &on_primary_container,
        &primary_fixed, &primary_fixed_dim, &on_primary_fixed, &on_primary_fixed_variant,
        &secondary_adjusted, &on_secondary, &secondary_container, &on_secondary_container,
        &secondary_fixed, &secondary_fixed_dim, &on_secondary_fixed, &on_secondary_fixed_variant,
        &tertiary_adjusted, &on_tertiary, &tertiary_container, &on_tertiary_container,
        &tertiary_fixed, &tertiary_fixed_dim, &on_tertiary_fixed, &on_tertiary_fixed_variant,
        &error, &on_error, &error_container, &on_error_container,
        &surface, &on_surface, &surface_variant, &on_surface_variant,
        &surface_dim, &surface_bright,
        &surface_container_lowest, &surface_container_low, &surface_container,
        &surface_container_high, &surface_container_highest,
        &outline, &outline_variant, &shadow, &scrim,
        &inverse_surface, &inverse_on_surface, &inverse_primary,
        &background, &on_background,
        &primary_adjusted,
    )
}

// ── Muted theme generators ──

const MUTED_SAT_PRIMARY: f32 = 0.15;
const MUTED_SAT_SECONDARY: f32 = 0.12;
const MUTED_SAT_TERTIARY: f32 = 0.10;
const MUTED_SAT_SURFACE: f32 = 0.08;

/// Generate muted dark theme from palette (low saturation).
pub fn generate_muted_dark(palette: &[Color]) -> HashMap<String, String> {
    let primary = palette.first().copied().unwrap_or(Color(128, 128, 128));
    let primary_h = primary.to_hsl().0;

    // Derive secondary/tertiary with subtle hue shifts
    let secondary = primary.shift_hue(15.0);
    let tertiary = primary.shift_hue(30.0);
    let error = find_error_color(palette);

    // Cap saturation low
    let (h, s, l) = primary.to_hsl();
    let primary_adjusted = Color::from_hsl(h, s.min(MUTED_SAT_PRIMARY), l.max(0.65));

    let (h, s, l) = secondary.to_hsl();
    let secondary_adjusted = Color::from_hsl(h, s.min(MUTED_SAT_SECONDARY), l.max(0.60));

    let (h, s, l) = tertiary.to_hsl();
    let tertiary_adjusted = Color::from_hsl(h, s.min(MUTED_SAT_TERTIARY), l.max(0.60));

    // Container colors
    let make_container = |base: &Color| -> Color {
        let (h, s, l) = base.to_hsl();
        Color::from_hsl(h, (s + 0.05).min(MUTED_SAT_PRIMARY), (l - 0.35).max(0.15))
    };

    let primary_container = make_container(&primary_adjusted);
    let secondary_container = make_container(&secondary_adjusted);
    let tertiary_container = make_container(&tertiary_adjusted);
    let error_container = make_container(&error);

    // Surface: very low saturation
    let base_surface = Color::from_hsl(primary_h, MUTED_SAT_SURFACE, 0.5);
    let surface = base_surface.adjust_surface(MUTED_SAT_SURFACE, 0.12);
    let surface_variant = base_surface.adjust_surface(MUTED_SAT_SURFACE, 0.16);
    let surface_container_lowest = base_surface.adjust_surface(MUTED_SAT_SURFACE, 0.06);
    let surface_container_low = base_surface.adjust_surface(MUTED_SAT_SURFACE, 0.10);
    let surface_container = base_surface.adjust_surface(MUTED_SAT_SURFACE, 0.20);
    let surface_container_high = base_surface.adjust_surface(MUTED_SAT_SURFACE, 0.18);
    let surface_container_highest = base_surface.adjust_surface(MUTED_SAT_SURFACE, 0.22);

    // Text colors
    let base_on_surface = Color::from_hsl(primary_h, 0.03, 0.95);
    let on_surface = ensure_contrast(&base_on_surface, &surface, 4.5, None);
    let base_on_surface_variant = Color::from_hsl(primary_h, 0.03, 0.70);
    let on_surface_variant = ensure_contrast(&base_on_surface_variant, &surface_variant, 4.5, None);
    let outline = ensure_contrast(&Color::from_hsl(primary_h, 0.05, 0.30), &surface, 3.0, None);
    let outline_variant = ensure_contrast(&Color::from_hsl(primary_h, 0.05, 0.40), &surface, 3.0, None);

    // Contrasting foregrounds
    let dark_fg = Color::from_hsl(primary_h, 0.10, 0.12);
    let on_primary = ensure_contrast(&dark_fg, &primary_adjusted, 7.0, None);
    let on_secondary = ensure_contrast(&dark_fg, &secondary_adjusted, 7.0, None);
    let on_tertiary = ensure_contrast(&dark_fg, &tertiary_adjusted, 7.0, None);
    let on_error = ensure_contrast(&dark_fg, &error, 7.0, None);

    // "On" container colors
    let on_primary_container = ensure_contrast(&Color::from_hsl(primary_h, 0.05, 0.90), &primary_container, 4.5, Some(true));
    let sec_h = secondary.to_hsl().0;
    let on_secondary_container = ensure_contrast(&Color::from_hsl(sec_h, 0.05, 0.90), &secondary_container, 4.5, Some(true));
    let ter_h = tertiary.to_hsl().0;
    let on_tertiary_container = ensure_contrast(&Color::from_hsl(ter_h, 0.05, 0.90), &tertiary_container, 4.5, Some(true));
    let err_h = error.to_hsl().0;
    let on_error_container = ensure_contrast(&Color::from_hsl(err_h, 0.05, 0.90), &error_container, 4.5, Some(true));

    let shadow = surface;
    let scrim = Color(0, 0, 0);

    // Inverse colors
    let inverse_surface = Color::from_hsl(primary_h, 0.05, 0.90);
    let inverse_on_surface = Color::from_hsl(primary_h, 0.03, 0.15);
    let inverse_primary = Color::from_hsl(primary_h, (primary.to_hsl().1 * 0.5).min(MUTED_SAT_PRIMARY), 0.40);

    let background = surface;
    let on_background = on_surface;

    // Fixed colors
    let make_fixed = |base: &Color| -> (Color, Color) {
        let (h, s, _) = base.to_hsl();
        let fixed = Color::from_hsl(h, s.min(MUTED_SAT_PRIMARY), 0.85);
        let fixed_dim = Color::from_hsl(h, s.min(MUTED_SAT_PRIMARY), 0.75);
        (fixed, fixed_dim)
    };

    let (primary_fixed, primary_fixed_dim) = make_fixed(&primary_adjusted);
    let (secondary_fixed, secondary_fixed_dim) = make_fixed(&secondary_adjusted);
    let (tertiary_fixed, tertiary_fixed_dim) = make_fixed(&tertiary_adjusted);

    let on_primary_fixed = ensure_contrast(&Color::from_hsl(primary_h, 0.05, 0.15), &primary_fixed, 4.5, None);
    let on_primary_fixed_variant = ensure_contrast(&Color::from_hsl(primary_h, 0.05, 0.20), &primary_fixed_dim, 4.5, None);
    let on_secondary_fixed = ensure_contrast(&Color::from_hsl(secondary.to_hsl().0, 0.05, 0.15), &secondary_fixed, 4.5, None);
    let on_secondary_fixed_variant = ensure_contrast(&Color::from_hsl(secondary.to_hsl().0, 0.05, 0.20), &secondary_fixed_dim, 4.5, None);
    let on_tertiary_fixed = ensure_contrast(&Color::from_hsl(tertiary.to_hsl().0, 0.05, 0.15), &tertiary_fixed, 4.5, None);
    let on_tertiary_fixed_variant = ensure_contrast(&Color::from_hsl(tertiary.to_hsl().0, 0.05, 0.20), &tertiary_fixed_dim, 4.5, None);

    let surface_dim = base_surface.adjust_surface(MUTED_SAT_SURFACE, 0.08);
    let surface_bright = base_surface.adjust_surface(MUTED_SAT_SURFACE, 0.24);

    collect_theme_map(
        &primary_adjusted, &on_primary, &primary_container, &on_primary_container,
        &primary_fixed, &primary_fixed_dim, &on_primary_fixed, &on_primary_fixed_variant,
        &secondary_adjusted, &on_secondary, &secondary_container, &on_secondary_container,
        &secondary_fixed, &secondary_fixed_dim, &on_secondary_fixed, &on_secondary_fixed_variant,
        &tertiary_adjusted, &on_tertiary, &tertiary_container, &on_tertiary_container,
        &tertiary_fixed, &tertiary_fixed_dim, &on_tertiary_fixed, &on_tertiary_fixed_variant,
        &error, &on_error, &error_container, &on_error_container,
        &surface, &on_surface, &surface_variant, &on_surface_variant,
        &surface_dim, &surface_bright,
        &surface_container_lowest, &surface_container_low, &surface_container,
        &surface_container_high, &surface_container_highest,
        &outline, &outline_variant, &shadow, &scrim,
        &inverse_surface, &inverse_on_surface, &inverse_primary,
        &background, &on_background,
        &primary_adjusted,
    )
}

/// Generate muted light theme from palette (low saturation).
pub fn generate_muted_light(palette: &[Color]) -> HashMap<String, String> {
    let primary = palette.first().copied().unwrap_or(Color(128, 128, 128));
    let primary_h = primary.to_hsl().0;

    let secondary = primary.shift_hue(15.0);
    let tertiary = primary.shift_hue(30.0);
    let error = find_error_color(palette);

    // Cap saturation low
    let (h, s, l) = primary.to_hsl();
    let primary_adjusted = Color::from_hsl(h, s.min(MUTED_SAT_PRIMARY), l.min(0.45));

    let (h, s, l) = secondary.to_hsl();
    let secondary_adjusted = Color::from_hsl(h, s.min(MUTED_SAT_SECONDARY), l.min(0.40));

    let (h, s, l) = tertiary.to_hsl();
    let tertiary_adjusted = Color::from_hsl(h, s.min(MUTED_SAT_TERTIARY), l.min(0.35));

    // Container colors
    let make_container = |base: &Color| -> Color {
        let (h, s, l) = base.to_hsl();
        Color::from_hsl(h, (s - 0.05).max(0.05), (l + 0.35).min(0.85))
    };

    let primary_container = make_container(&primary_adjusted);
    let secondary_container = make_container(&secondary_adjusted);
    let tertiary_container = make_container(&tertiary_adjusted);
    let error_container = make_container(&error);

    // Surface: very low saturation
    let surface = primary.adjust_surface(MUTED_SAT_SURFACE, 0.90);
    let surface_variant = primary.adjust_surface(MUTED_SAT_SURFACE, 0.78);
    let surface_container_lowest = primary.adjust_surface(MUTED_SAT_SURFACE, 0.96);
    let surface_container_low = primary.adjust_surface(MUTED_SAT_SURFACE, 0.92);
    let surface_container = primary.adjust_surface(MUTED_SAT_SURFACE, 0.86);
    let surface_container_high = primary.adjust_surface(MUTED_SAT_SURFACE, 0.84);
    let surface_container_highest = primary.adjust_surface(MUTED_SAT_SURFACE, 0.80);

    // Text colors
    let base_on_surface = Color::from_hsl(primary_h, 0.03, 0.10);
    let on_surface = ensure_contrast(&base_on_surface, &surface, 4.5, None);
    let base_on_surface_variant = Color::from_hsl(primary_h, 0.03, 0.35);
    let on_surface_variant = ensure_contrast(&base_on_surface_variant, &surface_variant, 4.5, None);

    let light_fg = Color::from_hsl(primary_h, 0.05, 0.98);
    let on_primary = ensure_contrast(&light_fg, &primary_adjusted, 7.0, None);
    let on_secondary = ensure_contrast(&light_fg, &secondary_adjusted, 7.0, None);
    let on_tertiary = ensure_contrast(&light_fg, &tertiary_adjusted, 7.0, None);
    let on_error = ensure_contrast(&light_fg, &error, 7.0, None);

    // "On" container colors
    let on_primary_container = ensure_contrast(&Color::from_hsl(primary_h, 0.05, 0.15), &primary_container, 4.5, Some(false));
    let sec_h = secondary.to_hsl().0;
    let on_secondary_container = ensure_contrast(&Color::from_hsl(sec_h, 0.05, 0.15), &secondary_container, 4.5, Some(false));
    let ter_h = tertiary.to_hsl().0;
    let on_tertiary_container = ensure_contrast(&Color::from_hsl(ter_h, 0.05, 0.15), &tertiary_container, 4.5, Some(false));
    let err_h = error.to_hsl().0;
    let on_error_container = ensure_contrast(&Color::from_hsl(err_h, 0.05, 0.15), &error_container, 4.5, Some(false));

    let outline = ensure_contrast(&Color::from_hsl(primary_h, 0.05, 0.65), &surface, 3.0, None);
    let outline_variant = ensure_contrast(&Color::from_hsl(primary_h, 0.05, 0.75), &surface, 3.0, None);
    let shadow = Color::from_hsl(primary_h, 0.05, 0.80);
    let scrim = Color(0, 0, 0);

    let inverse_surface = Color::from_hsl(primary_h, 0.05, 0.15);
    let inverse_on_surface = Color::from_hsl(primary_h, 0.03, 0.90);
    let inverse_primary = Color::from_hsl(primary_h, (primary.to_hsl().1 * 0.5).min(MUTED_SAT_PRIMARY), 0.70);

    let background = surface;
    let on_background = on_surface;

    // Fixed colors
    let make_fixed = |base: &Color| -> (Color, Color) {
        let (h, s, _) = base.to_hsl();
        let fixed = Color::from_hsl(h, s.min(MUTED_SAT_PRIMARY), 0.40);
        let fixed_dim = Color::from_hsl(h, s.min(MUTED_SAT_PRIMARY), 0.30);
        (fixed, fixed_dim)
    };

    let (primary_fixed, primary_fixed_dim) = make_fixed(&primary_adjusted);
    let (secondary_fixed, secondary_fixed_dim) = make_fixed(&secondary_adjusted);
    let (tertiary_fixed, tertiary_fixed_dim) = make_fixed(&tertiary_adjusted);

    let on_primary_fixed = ensure_contrast(&Color::from_hsl(primary_h, 0.05, 0.90), &primary_fixed, 4.5, None);
    let on_primary_fixed_variant = ensure_contrast(&Color::from_hsl(primary_h, 0.05, 0.85), &primary_fixed_dim, 4.5, None);
    let on_secondary_fixed = ensure_contrast(&Color::from_hsl(secondary.to_hsl().0, 0.05, 0.90), &secondary_fixed, 4.5, None);
    let on_secondary_fixed_variant = ensure_contrast(&Color::from_hsl(secondary.to_hsl().0, 0.05, 0.85), &secondary_fixed_dim, 4.5, None);
    let on_tertiary_fixed = ensure_contrast(&Color::from_hsl(tertiary.to_hsl().0, 0.05, 0.90), &tertiary_fixed, 4.5, None);
    let on_tertiary_fixed_variant = ensure_contrast(&Color::from_hsl(tertiary.to_hsl().0, 0.05, 0.85), &tertiary_fixed_dim, 4.5, None);

    let surface_dim = primary.adjust_surface(MUTED_SAT_SURFACE, 0.82);
    let surface_bright = primary.adjust_surface(MUTED_SAT_SURFACE, 0.95);

    collect_theme_map(
        &primary_adjusted, &on_primary, &primary_container, &on_primary_container,
        &primary_fixed, &primary_fixed_dim, &on_primary_fixed, &on_primary_fixed_variant,
        &secondary_adjusted, &on_secondary, &secondary_container, &on_secondary_container,
        &secondary_fixed, &secondary_fixed_dim, &on_secondary_fixed, &on_secondary_fixed_variant,
        &tertiary_adjusted, &on_tertiary, &tertiary_container, &on_tertiary_container,
        &tertiary_fixed, &tertiary_fixed_dim, &on_tertiary_fixed, &on_tertiary_fixed_variant,
        &error, &on_error, &error_container, &on_error_container,
        &surface, &on_surface, &surface_variant, &on_surface_variant,
        &surface_dim, &surface_bright,
        &surface_container_lowest, &surface_container_low, &surface_container,
        &surface_container_high, &surface_container_highest,
        &outline, &outline_variant, &shadow, &scrim,
        &inverse_surface, &inverse_on_surface, &inverse_primary,
        &background, &on_background,
        &primary_adjusted,
    )
}

fn interpolate_color(c1: &Color, c2: &Color, t: f32) -> Color {
    let r = (c1.0 as f32 + (c2.0 as f32 - c1.0 as f32) * t).clamp(0.0, 255.0).round() as u8;
    let g = (c1.1 as f32 + (c2.1 as f32 - c1.1 as f32) * t).clamp(0.0, 255.0).round() as u8;
    let b = (c1.2 as f32 + (c2.2 as f32 - c1.2 as f32) * t).clamp(0.0, 255.0).round() as u8;
    Color(r, g, b)
}

/// Expand a 14-color predefined scheme to the full 48-color palette.
///
/// Input keys: cPrimary, cOnPrimary, cSecondary, cOnSecondary, cTertiary,
/// cOnTertiary, cError, cOnError, cSurface, cOnSurface, cSurfaceVariant,
/// cOnSurfaceVariant, cOutline, cShadow (optional, falls back to cSurface).
///
/// Output: Same 48-color dict as `generate_theme()`.
pub fn expand_predefined_scheme(scheme_data: &HashMap<String, String>, mode: &str) -> HashMap<String, String> {
    let is_dark = mode == "dark";

    let parse = |key: &str| -> Color {
        scheme_data
            .get(key)
            .and_then(|s| Color::from_hex(s))
            .unwrap_or(Color(0, 0, 0))
    };

    let primary = parse("cPrimary");
    let on_primary = parse("cOnPrimary");
    let secondary = parse("cSecondary");
    let on_secondary = parse("cOnSecondary");
    let tertiary = parse("cTertiary");
    let on_tertiary = parse("cOnTertiary");
    let error = parse("cError");
    let on_error = parse("cOnError");
    let surface = parse("cSurface");
    let on_surface = parse("cOnSurface");
    let surface_variant = parse("cSurfaceVariant");
    let on_surface_variant = parse("cOnSurfaceVariant");
    let outline_raw = parse("cOutline");
    let shadow = scheme_data
        .get("cShadow")
        .and_then(|s| Color::from_hex(s))
        .unwrap_or(surface);

    // Container colors
    let primary_container = if is_dark { make_container_dark(&primary) } else { make_container_light(&primary) };
    let secondary_container = if is_dark { make_container_dark(&secondary) } else { make_container_light(&secondary) };
    let tertiary_container = if is_dark { make_container_dark(&tertiary) } else { make_container_light(&tertiary) };
    let error_container = if is_dark { make_container_dark(&error) } else { make_container_light(&error) };

    let (ph, ps, _) = primary.to_hsl();
    let (sh, ss, _) = secondary.to_hsl();
    let (th, ts, _) = tertiary.to_hsl();
    let (eh, es, _) = error.to_hsl();

    // "On container" colors
    let (on_primary_container, on_secondary_container, on_tertiary_container, on_error_container) = if is_dark {
        (
            ensure_contrast(&Color::from_hsl(ph, ps, 0.90), &primary_container, 4.5, None),
            ensure_contrast(&Color::from_hsl(sh, ss, 0.90), &secondary_container, 4.5, None),
            ensure_contrast(&Color::from_hsl(th, ts, 0.90), &tertiary_container, 4.5, None),
            ensure_contrast(&Color::from_hsl(eh, es, 0.90), &error_container, 4.5, None),
        )
    } else {
        (
            ensure_contrast(&Color::from_hsl(ph, ps, 0.15), &primary_container, 4.5, None),
            ensure_contrast(&Color::from_hsl(sh, ss, 0.15), &secondary_container, 4.5, None),
            ensure_contrast(&Color::from_hsl(th, ts, 0.15), &tertiary_container, 4.5, None),
            ensure_contrast(&Color::from_hsl(eh, es, 0.15), &error_container, 4.5, None),
        )
    };

    // Fixed colors
    let (primary_fixed, primary_fixed_dim) = if is_dark { make_fixed_dark(&primary) } else { make_fixed_light(&primary) };
    let (secondary_fixed, secondary_fixed_dim) = if is_dark { make_fixed_dark(&secondary) } else { make_fixed_light(&secondary) };
    let (tertiary_fixed, tertiary_fixed_dim) = if is_dark { make_fixed_dark(&tertiary) } else { make_fixed_light(&tertiary) };

    // "On fixed" colors
    let (on_primary_fixed, on_primary_fixed_variant) = if is_dark {
        (
            ensure_contrast(&Color::from_hsl(ph, 0.15, 0.15), &primary_fixed, 4.5, None),
            ensure_contrast(&Color::from_hsl(ph, 0.15, 0.20), &primary_fixed_dim, 4.5, None),
        )
    } else {
        (
            ensure_contrast(&Color::from_hsl(ph, 0.15, 0.90), &primary_fixed, 4.5, None),
            ensure_contrast(&Color::from_hsl(ph, 0.15, 0.85), &primary_fixed_dim, 4.5, None),
        )
    };
    let (on_secondary_fixed, on_secondary_fixed_variant) = if is_dark {
        (
            ensure_contrast(&Color::from_hsl(sh, 0.15, 0.15), &secondary_fixed, 4.5, None),
            ensure_contrast(&Color::from_hsl(sh, 0.15, 0.20), &secondary_fixed_dim, 4.5, None),
        )
    } else {
        (
            ensure_contrast(&Color::from_hsl(sh, 0.15, 0.90), &secondary_fixed, 4.5, None),
            ensure_contrast(&Color::from_hsl(sh, 0.15, 0.85), &secondary_fixed_dim, 4.5, None),
        )
    };
    let (on_tertiary_fixed, on_tertiary_fixed_variant) = if is_dark {
        (
            ensure_contrast(&Color::from_hsl(th, 0.15, 0.15), &tertiary_fixed, 4.5, None),
            ensure_contrast(&Color::from_hsl(th, 0.15, 0.20), &tertiary_fixed_dim, 4.5, None),
        )
    } else {
        (
            ensure_contrast(&Color::from_hsl(th, 0.15, 0.90), &tertiary_fixed, 4.5, None),
            ensure_contrast(&Color::from_hsl(th, 0.15, 0.85), &tertiary_fixed_dim, 4.5, None),
        )
    };

    // Surface containers
    let (surface_h, surface_s, surface_l) = surface.to_hsl();
    let (sv_h, sv_s, sv_l) = surface_variant.to_hsl();

    let surface_container = surface_variant;

    let (surface_container_lowest, surface_container_low,
         surface_container_high, surface_container_highest,
         surface_dim, surface_bright) = if is_dark {
        (
            interpolate_color(&surface, &surface_variant, 0.2),
            interpolate_color(&surface, &surface_variant, 0.5),
            Color::from_hsl(sv_h, sv_s, (sv_l + 0.04).min(0.40)),
            Color::from_hsl(sv_h, sv_s, (sv_l + 0.08).min(0.45)),
            Color::from_hsl(surface_h, surface_s, (surface_l - 0.04).max(0.02)),
            Color::from_hsl(sv_h, sv_s, (sv_l + 0.12).min(0.50)),
        )
    } else {
        (
            interpolate_color(&surface, &surface_variant, 0.2),
            interpolate_color(&surface, &surface_variant, 0.5),
            Color::from_hsl(sv_h, sv_s, (sv_l - 0.04).max(0.60)),
            Color::from_hsl(sv_h, sv_s, (sv_l - 0.08).max(0.55)),
            Color::from_hsl(sv_h, sv_s, (sv_l - 0.12).max(0.50)),
            Color::from_hsl(surface_h, surface_s, (surface_l + 0.03).min(0.98)),
        )
    };

    // Outline
    let outline = ensure_contrast(&outline_raw, &surface, 3.0, None);
    let (outline_h, outline_s, outline_l) = outline.to_hsl();
    let outline_variant = if is_dark {
        Color::from_hsl(outline_h, outline_s, (outline_l - 0.15).max(0.1))
    } else {
        Color::from_hsl(outline_h, outline_s, (outline_l + 0.15).min(0.9))
    };

    let scrim = Color(0, 0, 0);

    // Inverse colors
    let (inverse_surface, inverse_on_surface, inverse_primary) = if is_dark {
        (
            Color::from_hsl(surface_h, 0.08, 0.90),
            Color::from_hsl(surface_h, 0.05, 0.15),
            Color::from_hsl(ph, (ps * 0.8).max(0.5), 0.40),
        )
    } else {
        (
            Color::from_hsl(surface_h, 0.08, 0.15),
            Color::from_hsl(surface_h, 0.05, 0.90),
            Color::from_hsl(ph, (ps * 0.8).max(0.5), 0.70),
        )
    };

    let background = surface;
    let on_background = on_surface;

    collect_theme_map(
        &primary, &on_primary, &primary_container, &on_primary_container,
        &primary_fixed, &primary_fixed_dim, &on_primary_fixed, &on_primary_fixed_variant,
        &secondary, &on_secondary, &secondary_container, &on_secondary_container,
        &secondary_fixed, &secondary_fixed_dim, &on_secondary_fixed, &on_secondary_fixed_variant,
        &tertiary, &on_tertiary, &tertiary_container, &on_tertiary_container,
        &tertiary_fixed, &tertiary_fixed_dim, &on_tertiary_fixed, &on_tertiary_fixed_variant,
        &error, &on_error, &error_container, &on_error_container,
        &surface, &on_surface, &surface_variant, &on_surface_variant,
        &surface_dim, &surface_bright,
        &surface_container_lowest, &surface_container_low, &surface_container,
        &surface_container_high, &surface_container_highest,
        &outline, &outline_variant, &shadow, &scrim,
        &inverse_surface, &inverse_on_surface, &inverse_primary,
        &background, &on_background,
        &primary,
    )
}

/// Generate a full theme dict from palette for the given mode and scheme type.
///
/// `scheme_type` accepts: "tonal-spot", "fruit-salad", "rainbow", "content",
/// "monochrome", "vibrant", "faithful", "muted".
///
/// For "vibrant", "faithful", and "dysfunctional" the normal theme engine is used.
/// For "muted" the muted engine is used.
/// All others delegate to the Material Design 3 DynamicScheme.
pub fn generate_theme(palette: &[Color], mode: &str, scheme_type: &str) -> HashMap<String, String> {
    let palette_colors: Vec<Color> = palette.to_vec();

    match scheme_type {
        "vibrant" | "faithful" | "dysfunctional" => {
            if mode == "dark" {
                generate_normal_dark(&palette_colors)
            } else {
                generate_normal_light(&palette_colors)
            }
        }
        "muted" => {
            if mode == "dark" {
                generate_muted_dark(&palette_colors)
            } else {
                generate_muted_light(&palette_colors)
            }
        }
        _ => {
            // Material schemes: delegate to DynamicScheme
            let seed = palette.first().copied().unwrap_or(Color(103, 80, 164));
            let mode_enum = if mode == "dark" {
                crate::dynamic::SchemeMode::Dark
            } else {
                crate::dynamic::SchemeMode::Light
            };
            let variant = crate::dynamic::SchemeType::from_str(scheme_type).unwrap_or(crate::dynamic::SchemeType::TonalSpot);
            let scheme = crate::dynamic::DynamicScheme::from_seed(seed, mode_enum, variant);
            let palette_obj = scheme.to_palette();
            palette_obj.to_map()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_palette() -> Vec<Color> {
        vec![
            Color(255, 200, 100), // primary - warm gold
            Color(100, 200, 255), // secondary - sky blue
            Color(200, 100, 255), // tertiary - purple
        ]
    }

    fn mock_scheme_data() -> HashMap<String, String> {
        let mut m = HashMap::new();
        m.insert("cPrimary".into(), "#FFC864".into());
        m.insert("cOnPrimary".into(), "#1A1A1A".into());
        m.insert("cSecondary".into(), "#64C8FF".into());
        m.insert("cOnSecondary".into(), "#1A1A1A".into());
        m.insert("cTertiary".into(), "#C864FF".into());
        m.insert("cOnTertiary".into(), "#1A1A1A".into());
        m.insert("cError".into(), "#FF4444".into());
        m.insert("cOnError".into(), "#FFFFFF".into());
        m.insert("cSurface".into(), "#1E1E2E".into());
        m.insert("cOnSurface".into(), "#CDD6F4".into());
        m.insert("cSurfaceVariant".into(), "#45475A".into());
        m.insert("cOnSurfaceVariant".into(), "#BAC2DE".into());
        m.insert("cOutline".into(), "#585B70".into());
        m.insert("cShadow".into(), "#000000".into());
        m
    }

    #[test]
    fn test_interpolate_color_midpoint() {
        let c1 = Color(0, 0, 0);
        let c2 = Color(255, 255, 255);
        let mid = interpolate_color(&c1, &c2, 0.5);
        assert_eq!(mid, Color(128, 128, 128));
    }

    #[test]
    fn test_interpolate_color_start() {
        let c1 = Color(10, 20, 30);
        let c2 = Color(100, 200, 255);
        let start = interpolate_color(&c1, &c2, 0.0);
        assert_eq!(start, c1);
    }

    #[test]
    fn test_interpolate_color_end() {
        let c1 = Color(10, 20, 30);
        let c2 = Color(100, 200, 255);
        let end = interpolate_color(&c1, &c2, 1.0);
        assert_eq!(end, c2);
    }

    #[test]
    fn test_expand_predefined_scheme_dark() {
        let result = expand_predefined_scheme(&mock_scheme_data(), "dark");
        let expected_keys = [
            "primary", "on_primary", "primary_container", "on_primary_container",
            "primary_fixed", "primary_fixed_dim", "on_primary_fixed", "on_primary_fixed_variant",
            "secondary", "on_secondary", "secondary_container", "on_secondary_container",
            "secondary_fixed", "secondary_fixed_dim", "on_secondary_fixed", "on_secondary_fixed_variant",
            "tertiary", "on_tertiary", "tertiary_container", "on_tertiary_container",
            "tertiary_fixed", "tertiary_fixed_dim", "on_tertiary_fixed", "on_tertiary_fixed_variant",
            "error", "on_error", "error_container", "on_error_container",
            "surface", "on_surface", "surface_variant", "on_surface_variant",
            "surface_dim", "surface_bright",
            "surface_container_lowest", "surface_container_low", "surface_container",
            "surface_container_high", "surface_container_highest",
            "outline", "outline_variant", "shadow", "scrim",
            "inverse_surface", "inverse_on_surface", "inverse_primary",
            "background", "on_background",
            "surface_tint",
        ];
        for key in &expected_keys {
            assert!(result.contains_key(*key), "Missing key: {}", key);
        }
        assert_eq!(result.len(), 49);
        assert_eq!(result.get("primary").unwrap(), "#ffc864");
        assert_eq!(result.get("background").unwrap(), result.get("surface").unwrap());
        assert_eq!(result.get("shadow").unwrap(), "#000000");
    }

    #[test]
    fn test_expand_predefined_scheme_light() {
        let result = expand_predefined_scheme(&mock_scheme_data(), "light");
        assert!(result.contains_key("primary"));
        assert!(result.contains_key("on_primary"));
        assert!(result.contains_key("surface"));
        assert!(result.contains_key("on_surface"));
        assert_eq!(result.len(), 49);
        // Container colors differ between dark/light even if input surface is same
        let dark_result = expand_predefined_scheme(&mock_scheme_data(), "dark");
        assert_ne!(
            result.get("primary_container"),
            dark_result.get("primary_container"),
            "Containers should differ between modes"
        );
    }

    #[test]
    fn test_expand_predefined_scheme_missing_cshadow() {
        let mut data = mock_scheme_data();
        data.remove("cShadow");
        let result = expand_predefined_scheme(&data, "dark");
        // Shadow should fall back to surface
        assert_eq!(result.get("shadow").unwrap(), result.get("surface").unwrap());
    }

    #[test]
    fn test_generate_theme_normal_dark() {
        let palette = mock_palette();
        let result = generate_theme(&palette, "dark", "vibrant");
        assert_eq!(result.len(), 49);
        assert!(result.contains_key("primary"));
        assert!(result.contains_key("on_surface"));
    }

    #[test]
    fn test_generate_theme_normal_light() {
        let palette = mock_palette();
        let result = generate_theme(&palette, "light", "vibrant");
        assert_eq!(result.len(), 49);
        assert!(result.contains_key("primary"));
        assert!(result.contains_key("on_surface"));
    }

    #[test]
    fn test_generate_theme_muted_dark() {
        let palette = mock_palette();
        let result = generate_theme(&palette, "dark", "muted");
        assert_eq!(result.len(), 49);
        assert!(result.contains_key("primary"));
        assert!(result.contains_key("on_surface"));
    }

    #[test]
    fn test_generate_theme_muted_light() {
        let palette = mock_palette();
        let result = generate_theme(&palette, "light", "muted");
        assert_eq!(result.len(), 49);
        assert!(result.contains_key("primary"));
        assert!(result.contains_key("on_surface"));
    }

    #[test]
    fn test_generate_theme_faithful() {
        let palette = mock_palette();
        let result = generate_theme(&palette, "dark", "faithful");
        assert_eq!(result.len(), 49);
        assert!(result.contains_key("primary"));
    }

    #[test]
    fn test_generate_theme_dysfunctional() {
        let palette = mock_palette();
        let result = generate_theme(&palette, "dark", "dysfunctional");
        assert_eq!(result.len(), 49);
        assert!(result.contains_key("primary"));
    }

    #[test]
    fn test_generate_theme_m3() {
        let palette = mock_palette();
        for scheme in &["tonal-spot", "content", "fruit-salad", "rainbow", "monochrome"] {
            let result = generate_theme(&palette, "dark", scheme);
            // M3 schemes return a Catppuccin-style Palette via to_map()
            assert!(result.contains_key("base"), "Missing base for {}", scheme);
            assert!(result.contains_key("text"), "Missing text for {}", scheme);
            assert!(result.contains_key("mauve"), "Missing mauve for {}", scheme);
        }
    }
}
