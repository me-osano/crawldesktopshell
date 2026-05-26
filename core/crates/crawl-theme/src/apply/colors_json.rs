use std::collections::HashMap;
use std::path::Path;

use crate::color::Color;
use crate::error::ThemeResult;
use crate::palette::Palette;

/// Writes the 16 QML color properties to `colors.json`.
///
/// This file is watched by `Theme.qml` (FileView) on the Quickshell frontend.
/// The mapping from Catppuccin-style Palette to M3-style QML color keys
/// matches `ColorSchemeService.writeColorsToDisk()` in the QML codebase.
pub fn write(path: &Path, palette: &Palette, accent: &Color) -> ThemeResult<()> {
    let is_dark = palette.base.is_dark();

    let surface_c = if is_dark { &palette.base } else { &palette.surface0 };
    let surface_variant_c = if is_dark { &palette.surface0 } else { &palette.base };

    let mut colors: HashMap<String, String> = HashMap::new();
    colors.insert("cPrimary".into(), accent.hex());
    colors.insert("cOnPrimary".into(), palette.crust.hex());
    colors.insert("cSecondary".into(), palette.lavender.hex());
    colors.insert("cOnSecondary".into(), palette.crust.hex());
    colors.insert("cTertiary".into(), palette.green.hex());
    colors.insert("cOnTertiary".into(), palette.crust.hex());
    colors.insert("cError".into(), palette.red.hex());
    colors.insert("cOnError".into(), palette.crust.hex());
    colors.insert("cSurface".into(), surface_c.hex());
    colors.insert("cOnSurface".into(), palette.text.hex());
    colors.insert("cSurfaceVariant".into(), surface_variant_c.hex());
    colors.insert("cOnSurfaceVariant".into(), palette.subtext0.hex());
    colors.insert("cOutline".into(), palette.overlay0.hex());
    colors.insert("cShadow".into(), palette.base.hex());
    colors.insert("cHover".into(), palette.green.hex());
    colors.insert("cOnHover".into(), palette.crust.hex());

    let json = serde_json::to_string_pretty(&colors)?;
    std::fs::write(path, json)?;
    Ok(())
}
