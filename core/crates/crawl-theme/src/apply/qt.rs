use crate::color::Color;
use crate::config::GtkConfig;
use crate::error::ThemeResult;
use crate::palette::Palette;

/// Write Qt5ct/Qt6ct color scheme config based on palette.
/// Qt5ct and Qt6ct watch these files and apply them live.
pub fn apply(config: &GtkConfig, palette: &Palette, accent: &Color) -> ThemeResult<()> {
    // Determine which Qt config tool is in use
    if let Some(qt5_dir) = qt_config_dir("qt5ct") {
        write_palette(&qt5_dir, config, palette, accent, "qt5ct")?;
    }
    if let Some(qt6_dir) = qt_config_dir("qt6ct") {
        write_palette(&qt6_dir, config, palette, accent, "qt6ct")?;
    }
    Ok(())
}

fn qt_config_dir(version: &str) -> Option<std::path::PathBuf> {
    let base = dirs::config_dir()?;
    Some(base.join(version))
}

fn write_palette(
    dir: &std::path::Path,
    _config: &GtkConfig,
    palette: &Palette,
    accent: &Color,
    version: &str,
) -> ThemeResult<()> {
    std::fs::create_dir_all(dir)?;

    let colors_file = dir.join("colors.conf");
    let accent_hex = accent.hex_stripped();
    let content = format!(
        r#"[ColorScheme]
name=crawlds
#
# Background
base={base}
# Text
text={text}
# Selection
selection_bg={accent_hex}
selection_fg={on_accent}
# Window
window={mantle}
window_text={text}
# Button
button={surface0}
button_text={text}
# Links
link={accent_hex}
link_visit={accent_hex}
"#,
        base = palette.base.hex_stripped(),
        text = palette.text.hex_stripped(),
        mantle = palette.mantle.hex_stripped(),
        surface0 = palette.surface0.hex_stripped(),
        accent_hex = accent_hex,
        on_accent = palette.base.hex_stripped(),
    );

    std::fs::write(&colors_file, content)?;

    // Write qt5ct.conf / qt6ct.conf with the color scheme reference
    let settings_file = dir.join(format!("{}.conf", version));
    let settings = format!(
        r#"[Appearance]
color_scheme_path={}/colors.conf
style=kvantum
"#,
        dir.display(),
    );

    std::fs::write(&settings_file, settings)?;

    Ok(())
}
