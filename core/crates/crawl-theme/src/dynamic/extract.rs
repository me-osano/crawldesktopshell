use std::path::Path;

use crate::color::Color;
use crate::dynamic::quantizer::extract_source_color;
use crate::error::{ThemeError, ThemeResult};

/// Extract the dominant seed color from an image file.
///
/// Decodes the image down to 112×112, then runs Wu + WSMeans quantization
/// and the Material Score algorithm to pick the best UI-suited color.
pub fn dominant_color(path: &Path) -> ThemeResult<Color> {
    let img =
        image::open(path).map_err(|e| ThemeError::Other(format!("failed to open image: {}", e)))?;

    let small = img.resize_exact(112, 112, image::imageops::Triangle);
    let rgb = small.to_rgb8();

    let pixels: Vec<(u8, u8, u8)> = rgb.pixels().map(|p| (p[0], p[1], p[2])).collect();

    if pixels.is_empty() {
        return Err(ThemeError::Other("no pixels extracted from image".into()));
    }

    let argb = extract_source_color(&pixels, 0xFF_4285_F4);
    let r = ((argb >> 16) & 0xFF) as u8;
    let g = ((argb >> 8) & 0xFF) as u8;
    let b = (argb & 0xFF) as u8;

    Ok(Color(r, g, b))
}
