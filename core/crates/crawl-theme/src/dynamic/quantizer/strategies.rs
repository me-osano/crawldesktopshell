use std::collections::HashMap;

use crate::color::Color;
use crate::dynamic::hct::cam16::Cam16;
use crate::dynamic::hct::Hct;
use crate::dynamic::quantizer::kmeans::kmeans_cluster;

const MIN_CHROMA: f32 = 10.0;

fn hue_to_family(hue: f32) -> usize {
    if hue >= 330.0 || hue < 30.0 {
        0 // RED
    } else if hue < 60.0 {
        1 // ORANGE
    } else if hue < 105.0 {
        2 // YELLOW
    } else if hue < 190.0 {
        3 // GREEN
    } else if hue < 270.0 {
        4 // BLUE
    } else {
        5 // PURPLE
    }
}

fn family_center_hue(family: usize) -> f32 {
    match family {
        0 => 0.0,
        1 => 45.0,
        2 => 82.5,
        3 => 147.5,
        4 => 230.0,
        5 => 300.0,
        _ => 0.0,
    }
}

fn circular_hue_diff(h1: f32, h2: f32) -> f32 {
    let diff = (h1 - h2).abs();
    diff.min(360.0 - diff)
}

fn downsample(pixels: &[(u8, u8, u8)], factor: usize) -> Vec<(u8, u8, u8)> {
    if factor <= 1 {
        return pixels.to_vec();
    }
    let step = factor * factor;
    pixels.iter().step_by(step).copied().collect()
}

/// Score colors prioritizing chroma (vibrant mode).
fn score_colors_chroma(colors_with_counts: &[((u8, u8, u8), usize)]) -> Vec<(Color, f32)> {
    let mut results: Vec<(Color, f32)> = Vec::new();
    for &(rgb, count) in colors_with_counts {
        let color = Color(rgb.0, rgb.1, rgb.2);
        match Cam16::from_rgb(rgb.0, rgb.1, rgb.2) {
            Ok(cam) => {
                let chroma_score = cam.chroma;

                let tone_penalty = if cam.hue < 20.0 {
                    (20.0 - cam.hue) * 2.0
                } else if cam.hue > 80.0 {
                    (cam.hue - 80.0) * 1.5
                } else if cam.hue < 40.0 {
                    (40.0 - cam.hue) * 0.5
                } else if cam.hue > 60.0 {
                    (cam.hue - 60.0) * 0.3
                } else {
                    0.0
                };

                let hue_penalty = if cam.hue > 80.0 && cam.hue < 110.0 {
                    5.0
                } else {
                    0.0
                };

                let score = (chroma_score - tone_penalty - hue_penalty) * (count as f32).powf(0.3);
                results.push((color, score));
            }
            Err(_) => {
                results.push((color, 0.0));
            }
        }
    }
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results
}

/// Score colors by area/pixel count grouped by hue family (faithful mode).
fn score_colors_count(colors_with_counts: &[((u8, u8, u8), usize)]) -> Vec<(Color, f32)> {
    let mut hue_families: HashMap<usize, Vec<(Color, f32, f32, usize)>> = HashMap::new();

    for &(rgb, count) in colors_with_counts {
        let color = Color(rgb.0, rgb.1, rgb.2);
        if let Ok(cam) = Cam16::from_rgb(rgb.0, rgb.1, rgb.2) {
            if cam.chroma >= MIN_CHROMA {
                let family = hue_to_family(cam.hue);
                hue_families
                    .entry(family)
                    .or_default()
                    .push((color, cam.hue, cam.chroma, count));
            }
        }
    }

    if hue_families.is_empty() {
        let mut result: Vec<(Color, f32)> = colors_with_counts
            .iter()
            .map(|&(rgb, count)| (Color(rgb.0, rgb.1, rgb.2), count as f32))
            .collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        return result;
    }

    let mut family_totals: Vec<(usize, usize)> = hue_families
        .iter()
        .map(|(&family, colors)| (family, colors.iter().map(|c| c.3).sum()))
        .collect();
    family_totals.sort_by(|a, b| b.1.cmp(&a.1));

    let mut result_colors: Vec<(Color, f32)> = Vec::new();
    for (family_rank, &(family, _)) in family_totals.iter().enumerate() {
        if let Some(colors) = hue_families.get(&family) {
            let mut family_colors = colors.clone();
            family_colors.sort_by(|a, b| b.3.cmp(&a.3).then(b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal)));
            for (color, _, chroma, count) in family_colors {
                let score = (family_totals.len() - family_rank) as f32 * 1_000_000.0
                    + count as f32 * 1000.0
                    + chroma;
                result_colors.push((color, score));
            }
        }
    }

    result_colors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result_colors
}

/// Score colors prioritizing 2nd most dominant hue family.
fn score_colors_dysfunctional(colors_with_counts: &[((u8, u8, u8), usize)]) -> Vec<(Color, f32)> {
    const MIN_CHROMA: f32 = 10.0;
    const MIN_HUE_DISTANCE: f32 = 45.0;
    const MIN_COUNT_RATIO: f32 = 0.02;

    let mut hue_families: HashMap<usize, Vec<(Color, f32, f32, usize)>> = HashMap::new();

    for &(rgb, count) in colors_with_counts {
        let color = Color(rgb.0, rgb.1, rgb.2);
        if let Ok(cam) = Cam16::from_rgb(rgb.0, rgb.1, rgb.2) {
            if cam.chroma >= MIN_CHROMA {
                let family = hue_to_family(cam.hue);
                hue_families
                    .entry(family)
                    .or_default()
                    .push((color, cam.hue, cam.chroma, count));
            }
        }
    }

    if hue_families.is_empty() {
        let mut result: Vec<(Color, f32)> = colors_with_counts
            .iter()
            .map(|&(rgb, count)| (Color(rgb.0, rgb.1, rgb.2), count as f32))
            .collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        return result;
    }

    let mut family_totals: Vec<(usize, usize)> = hue_families
        .iter()
        .map(|(&family, colors)| (family, colors.iter().map(|c| c.3).sum()))
        .collect();
    family_totals.sort_by(|a, b| b.1.cmp(&a.1));

    let dominant_family = family_totals[0].0;
    let dominant_center = family_center_hue(dominant_family);
    let total_colorful: usize = family_totals.iter().map(|(_, c)| c).sum();
    let min_count = (total_colorful as f32 * MIN_COUNT_RATIO) as usize;

    let mut distant_families: Vec<(usize, usize, f32, f32)> = Vec::new();
    let mut close_families: Vec<usize> = vec![dominant_family];

    for &(family, count) in &family_totals[1..] {
        let fc = family_center_hue(family);
        let hd = circular_hue_diff(dominant_center, fc);
        if hd >= MIN_HUE_DISTANCE && count >= min_count {
            if let Some(colors) = hue_families.get(&family) {
                let max_chroma = colors.iter().map(|c| c.2).fold(0.0f32, f32::max);
                distant_families.push((family, count, hd, max_chroma));
            }
        } else {
            close_families.push(family);
        }
    }

    distant_families.sort_by(|a, b| {
        (b.2 * b.3)
            .partial_cmp(&(a.2 * a.3))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut result_colors: Vec<(Color, f32)> = Vec::new();

    for (d_rank, &(family, _, _, _)) in distant_families.iter().enumerate() {
        if let Some(colors) = hue_families.get(&family) {
            let mut fc = colors.clone();
            fc.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal).then(b.3.cmp(&a.3)));
            for (color, _, chroma, count) in fc {
                let score = (distant_families.len() - d_rank) as f32 * 1_000_000.0
                    + chroma * 1000.0
                    + count as f32;
                result_colors.push((color, score));
            }
        }
    }

    for &family in &close_families {
        if let Some(colors) = hue_families.get(&family) {
            let mut fc = colors.clone();
            fc.sort_by(|a, b| b.3.cmp(&a.3).then(b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal)));
            for (color, _, chroma, count) in fc {
                let score = count as f32 * 1000.0 + chroma;
                result_colors.push((color, score));
            }
        }
    }

    result_colors.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result_colors
}

/// Score colors by pure pixel count without chroma filtering (muted mode).
fn score_colors_muted(colors_with_counts: &[((u8, u8, u8), usize)]) -> Vec<(Color, f32)> {
    let mut result: Vec<(Color, f32)> = colors_with_counts
        .iter()
        .map(|&(rgb, count)| (Color(rgb.0, rgb.1, rgb.2), count as f32))
        .collect();
    result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    result
}

/// Extract K dominant colors from pixel data using the specified scoring method.
///
/// - `"population"`: Material Design Score algorithm (delegates to `score_colors`)
/// - `"chroma"`: vibrant, chroma-prioritized
/// - `"count"`: area-dominant by hue family (faithful mode)
/// - `"dysfunctional"`: 2nd most dominant hue family
/// - `"muted"`: like count but without chroma filtering (monochrome wallpapers)
pub fn extract_palette(
    pixels: &[(u8, u8, u8)],
    k: usize,
    scoring: &str,
) -> Vec<Color> {
    let sampled = downsample(pixels, 4);
    let sampled_len = sampled.len();
    let sampled_empty = sampled.is_empty();
    let first_sample = sampled.first().copied();

    let (cluster_count, filtered): (usize, Vec<(u8, u8, u8)>) = match scoring {
        "population" => {
            let cc = 128.min(sampled_len.max(k * 10));
            let mut dedup = sampled.clone();
            dedup.sort();
            dedup.dedup();
            (cc.min(dedup.len().max(k)), sampled)
        }
        "count" | "dysfunctional" => (48, sampled),
        "muted" => (24, sampled),
        _ => {
            // vibrant/chroma
            let cc = 20;
            let filtered: Vec<(u8, u8, u8)> = sampled
                .iter()
                .copied()
                .filter(|&(r, g, b)| {
                    Cam16::from_rgb(r, g, b)
                        .map(|cam| cam.chroma >= 5.0)
                        .unwrap_or(false)
                })
                .collect();
            if filtered.len() < cc * 2 {
                (cc, sampled)
            } else {
                (cc, filtered)
            }
        }
    };

    let effective_k = cluster_count.max(k).min(sampled_len);

    if effective_k < 2 || sampled_empty {
        // Fallback: return a single color or empty
        return if let Some(c) = first_sample {
            vec![Color(c.0, c.1, c.2)]
        } else {
            vec![Color::from_hex("#6750A4").unwrap_or(Color(103, 80, 164))]
        };
    }

    let clusters = kmeans_cluster(&filtered, effective_k, 10);

    // Prepare (rgb, count) pairs for scoring
    let colors_for_scoring: Vec<((u8, u8, u8), usize)> = match scoring {
        "chroma" => clusters.iter().map(|c| (c.0, c.2)).collect(),
        _ => clusters.iter().map(|c| (c.1, c.2)).collect(),
    };

    let scored = match scoring {
        "chroma" => score_colors_chroma(&colors_for_scoring),
        "count" => score_colors_count(&colors_for_scoring),
        "dysfunctional" => score_colors_dysfunctional(&colors_for_scoring),
        "muted" => score_colors_muted(&colors_for_scoring),
        _ => {
            // population: delegate to the existing Material score algorithm
            let mut color_to_population: std::collections::HashMap<u32, usize> =
                std::collections::HashMap::new();
            for &(rgb, count) in &colors_for_scoring {
                let argb = 0xFF000000
                    | ((rgb.0 as u32) << 16)
                    | ((rgb.1 as u32) << 8)
                    | (rgb.2 as u32);
                *color_to_population.entry(argb).or_insert(0) += count;
            }
            let scored_argbs = super::score::score_colors(
                &color_to_population,
                k,
                0xFF4285F4,
                true,
            );
            scored_argbs
                .into_iter()
                .map(|argb| {
                    let r = ((argb >> 16) & 0xFF) as u8;
                    let g = ((argb >> 8) & 0xFF) as u8;
                    let b = (argb & 0xFF) as u8;
                    Color(r, g, b)
                })
                .map(|c| (c, 0.0f32))
                .collect()
        }
    };

    let mut final_colors: Vec<Color> = scored.into_iter().map(|(c, _)| c).collect();

    while final_colors.len() < k {
        if final_colors.is_empty() {
            final_colors.push(Color::from_hex("#6750A4").unwrap_or(Color(103, 80, 164)));
            continue;
        }
        let primary = final_colors[0];
        let primary_hct = Hct::from_rgb(primary.0, primary.1, primary.2);
        let offset = final_colors.len() as f32 * 60.0;
        let new_hue = (primary_hct.get_hue() + offset) % 360.0;
        let new_hct = Hct::from(new_hue, primary_hct.get_chroma(), primary_hct.get_tone());
        let (r, g, b) = new_hct.to_rgb();
        final_colors.push(Color(r, g, b));
    }

    final_colors.truncate(k);
    final_colors
}

/// Find or generate an error color (red-biased).
pub fn find_error_color(palette: &[Color]) -> Color {
    for &color in palette {
        let (h, s, l) = color.to_hsl();
        if (h <= 30.0 || h >= 330.0) && s > 0.4 && l > 0.3 && l < 0.7 {
            return color;
        }
    }
    Color::from_hex("#FD4663").unwrap_or(Color(253, 70, 99))
}
