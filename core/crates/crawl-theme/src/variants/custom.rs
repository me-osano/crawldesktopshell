use std::collections::HashMap;
use std::path::Path;

use crate::color::Color;
use crate::error::ThemeResult;
use crate::palette::Palette;

/// Load a custom palette from a TOML file.
///
/// Expected format:
/// ```toml
/// base = "#1e1e2e"
/// mantle = "#181825"
/// crust = "#11111b"
/// surface0 = "#313244"
/// ...
/// extra = { brand = "#ff6600" }
/// ```
pub fn load_from_toml(path: &Path) -> ThemeResult<Palette> {
    let content = std::fs::read_to_string(path)?;
    let palette: Palette = toml::from_str(&content)?;
    Ok(palette)
}

/// Load palette from a flat string-keyed map.
/// Useful when deserializing from IPC or other sources.
pub fn from_map(map: &HashMap<String, String>) -> Option<Palette> {
    let mut extra = HashMap::new();
    let mut fields = HashMap::new();

    for (k, v) in map {
        let color = Color::from_hex(v)?;
        // Known palette fields are parsed by name; unknown go to extra
        match k.as_str() {
            "base" | "mantle" | "crust" | "surface0" | "surface1" | "surface2" | "overlay0"
            | "overlay1" | "overlay2" | "text" | "subtext0" | "subtext1" | "rosewater"
            | "flamingo" | "pink" | "mauve" | "red" | "maroon" | "peach" | "yellow" | "green"
            | "teal" | "sky" | "sapphire" | "blue" | "lavender" => {
                fields.insert(k.clone(), color);
            }
            _ => {
                extra.insert(k.clone(), color);
            }
        }
    }

    macro_rules! req {
        ($name:expr) => {
            fields.remove($name)?
        };
    }

    Some(Palette {
        base: req!("base"),
        mantle: req!("mantle"),
        crust: req!("crust"),
        surface0: req!("surface0"),
        surface1: req!("surface1"),
        surface2: req!("surface2"),
        overlay0: req!("overlay0"),
        overlay1: req!("overlay1"),
        overlay2: req!("overlay2"),
        text: req!("text"),
        subtext0: req!("subtext0"),
        subtext1: req!("subtext1"),
        rosewater: req!("rosewater"),
        flamingo: req!("flamingo"),
        pink: req!("pink"),
        mauve: req!("mauve"),
        red: req!("red"),
        maroon: req!("maroon"),
        peach: req!("peach"),
        yellow: req!("yellow"),
        green: req!("green"),
        teal: req!("teal"),
        sky: req!("sky"),
        sapphire: req!("sapphire"),
        blue: req!("blue"),
        lavender: req!("lavender"),
        extra,
    })
}
