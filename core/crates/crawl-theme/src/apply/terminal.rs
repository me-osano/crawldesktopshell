use std::collections::HashMap;

const COLOR_ORDER: [&str; 8] = [
    "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
];

fn strip_hash(s: &str) -> String {
    s.trim_start_matches('#').to_string()
}

fn ensure_hash(s: &str) -> String {
    if s.starts_with('#') {
        s.to_string()
    } else {
        format!("#{}", s)
    }
}

fn darken_hex(color: &str, percent: f32) -> String {
    let hex = color.trim_start_matches('#');
    if hex.len() < 6 {
        return color.to_string();
    }
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
    let factor = 1.0 - (percent / 100.0);
    let nr = (r as f32 * factor).max(0.0) as u8;
    let ng = (g as f32 * factor).max(0.0) as u8;
    let nb = (b as f32 * factor).max(0.0) as u8;
    format!("#{:02x}{:02x}{:02x}", nr, ng, nb)
}

#[derive(Debug, Clone)]
pub struct TabBarColors {
    pub background: String,
    pub inactive_tab_edge: String,
    pub active_tab: (String, String),
    pub inactive_tab: (String, String),
    pub inactive_tab_hover: (String, String),
    pub new_tab: (String, String),
    pub new_tab_hover: (String, String),
}

#[derive(Debug, Clone)]
pub struct TerminalColors {
    pub foreground: String,
    pub background: String,
    pub cursor: String,
    pub cursor_text: String,
    pub selection_fg: String,
    pub selection_bg: String,
    pub normal: HashMap<String, String>,
    pub bright: HashMap<String, String>,
    pub compose_cursor: String,
    pub scrollbar_thumb: String,
    pub split: String,
    pub visual_bell: String,
    pub indexed: HashMap<u32, String>,
    pub tab_bar: TabBarColors,
    pub active_border: String,
    pub inactive_border: String,
}

fn get_str<'a>(val: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    val.get(key).and_then(|v| v.as_str())
}

fn get_map(val: &serde_json::Value, key: &str) -> HashMap<String, String> {
    val.get(key)
        .and_then(|v| {
            v.as_object().map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("#000000").to_string()))
                    .collect()
            })
            .or_else(|| v.as_str().and_then(|s| serde_json::from_str(s).ok()))
        })
        .unwrap_or_default()
}

impl TerminalColors {
    pub fn from_dict(data: &serde_json::Value, scheme: &HashMap<String, String>) -> Self {
        let foreground = get_str(data, "foreground").unwrap_or("").to_string();
        let background = get_str(data, "background").unwrap_or("").to_string();
        let cursor = get_str(data, "cursor").unwrap_or(&foreground).to_string();
        let cursor_text = get_str(data, "cursorText").unwrap_or(&background).to_string();
        let selection_fg = get_str(data, "selectionFg").unwrap_or(&foreground).to_string();
        let selection_bg = get_str(data, "selectionBg")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "#585b70".into());

        let normal = get_map(data, "normal");
        let bright = get_map(data, "bright");

        let m_primary = scheme.get("cPrimary").unwrap_or(&cursor).clone();
        let m_on_primary = scheme.get("cOnPrimary").unwrap_or(&cursor_text).clone();
        let m_secondary = scheme
            .get("cSecondary")
            .unwrap_or(normal.get("yellow").unwrap_or(&cursor))
            .clone();
        let _m_surface_variant = scheme
            .get("cSurfaceVariant")
            .unwrap_or(&selection_bg)
            .clone();

        let bright_black = bright.get("black").cloned().unwrap_or_default();
        let _yellow = normal.get("yellow").cloned().unwrap_or_default();
        let black = normal.get("black").cloned().unwrap_or_default();

        TerminalColors {
            compose_cursor: cursor.clone(),
            scrollbar_thumb: selection_bg.clone(),
            split: bright_black.clone(),
            visual_bell: black,
            indexed: [(16, m_secondary.clone()), (17, cursor.clone())]
                .iter()
                .map(|(k, v)| (*k, v.clone()))
                .collect(),
            tab_bar: TabBarColors {
                background: darken_hex(&background, 10.0),
                inactive_tab_edge: selection_bg.clone(),
                active_tab: (m_primary.clone(), m_on_primary.clone()),
                inactive_tab: (darken_hex(&background, 5.0), foreground.clone()),
                inactive_tab_hover: (background.clone(), foreground.clone()),
                new_tab: (selection_bg.clone(), foreground.clone()),
                new_tab_hover: (bright_black, foreground.clone()),
            },
            active_border: m_primary,
            inactive_border: m_secondary,
            foreground,
            background,
            cursor,
            cursor_text,
            selection_fg,
            selection_bg,
            normal,
            bright,
        }
    }
}

pub struct TerminalGenerator {
    colors: TerminalColors,
}

impl TerminalGenerator {
    pub fn new(colors: TerminalColors) -> Self {
        Self { colors }
    }

    fn color(&self, name: &str, is_normal: bool) -> &str {
        if is_normal {
            self.colors.normal.get(name).map(|s| s.as_str()).unwrap_or("#000000")
        } else {
            self.colors.bright.get(name).map(|s| s.as_str()).unwrap_or("#000000")
        }
    }

    pub fn generate_foot(&self) -> String {
        let c = &self.colors;
        let mut lines = vec!["[colors]".to_string()];
        lines.push(format!("foreground={}", strip_hash(&c.foreground)));
        lines.push(format!("background={}", strip_hash(&c.background)));

        for (i, name) in COLOR_ORDER.iter().enumerate() {
            let val = c.normal.get(*name).map(|s| s.as_str()).unwrap_or("#000000");
            lines.push(format!("regular{i}={}", strip_hash(val)));
        }
        for (i, name) in COLOR_ORDER.iter().enumerate() {
            let val = c.bright.get(*name).map(|s| s.as_str()).unwrap_or("#000000");
            lines.push(format!("bright{i}={}", strip_hash(val)));
        }

        lines.push(format!("selection-foreground={}", strip_hash(&c.selection_fg)));
        lines.push(format!("selection-background={}", strip_hash(&c.selection_bg)));
        lines.push(format!(
            "cursor={} {}",
            strip_hash(&c.cursor_text),
            strip_hash(&c.cursor)
        ));

        lines.join("\n") + "\n"
    }

    pub fn generate_ghostty(&self) -> String {
        let c = &self.colors;
        let mut lines = Vec::new();

        for (i, name) in COLOR_ORDER.iter().enumerate() {
            let val = c.normal.get(*name).map(|s| s.as_str()).unwrap_or("#000000");
            lines.push(format!("palette = {i}={}", ensure_hash(val)));
        }
        for (i, name) in COLOR_ORDER.iter().enumerate() {
            let val = c.bright.get(*name).map(|s| s.as_str()).unwrap_or("#000000");
            lines.push(format!("palette = {}= {}", i + 8, ensure_hash(val)));
        }

        lines.push(format!("background = {}", ensure_hash(&c.background)));
        lines.push(format!("foreground = {}", ensure_hash(&c.foreground)));
        lines.push(format!("cursor-color = {}", ensure_hash(&c.cursor)));
        lines.push(format!("cursor-text = {}", ensure_hash(&c.cursor_text)));
        lines.push(format!("selection-background = {}", ensure_hash(&c.selection_bg)));
        lines.push(format!("selection-foreground = {}", ensure_hash(&c.selection_fg)));

        lines.join("\n") + "\n"
    }

    pub fn generate_kitty(&self) -> String {
        let c = &self.colors;
        let mut lines = Vec::new();

        for (i, name) in COLOR_ORDER.iter().enumerate() {
            let val = c.normal.get(*name).map(|s| s.as_str()).unwrap_or("#000000");
            lines.push(format!("color{i} {}", ensure_hash(val)));
        }
        for (i, name) in COLOR_ORDER.iter().enumerate() {
            let val = c.bright.get(*name).map(|s| s.as_str()).unwrap_or("#000000");
            lines.push(format!("color{} {}", i + 8, ensure_hash(val)));
        }

        lines.push(format!("background {}", ensure_hash(&c.background)));
        lines.push(format!("selection_foreground {}", ensure_hash(&c.cursor_text)));
        lines.push(format!("cursor {}", ensure_hash(&c.cursor)));
        lines.push(format!("cursor_text_color {}", ensure_hash(&c.cursor_text)));
        lines.push(format!("foreground {}", ensure_hash(&c.foreground)));
        lines.push(format!("selection_background {}", ensure_hash(&c.foreground)));
        lines.push(format!("active_border_color {}", ensure_hash(&c.active_border)));
        lines.push(format!("inactive_border_color {}", ensure_hash(&c.inactive_border)));

        lines.join("\n") + "\n"
    }

    pub fn generate_alacritty(&self) -> String {
        let c = &self.colors;
        let mut lines = vec!["# Colors (CrawlDS)".to_string(), String::new()];

        lines.push("[colors.bright]".to_string());
        let mut sorted_names: Vec<&str> = COLOR_ORDER.to_vec();
        sorted_names.sort();
        for name in sorted_names {
            let val = c.bright.get(name).map(|s| s.as_str()).unwrap_or("#000000");
            lines.push(format!("{} = '{}'", name, ensure_hash(val)));
        }
        lines.push(String::new());

        lines.push("[colors.cursor]".to_string());
        lines.push(format!("cursor = '{}'", ensure_hash(&c.cursor)));
        lines.push(format!("text = '{}'", ensure_hash(&c.cursor_text)));
        lines.push(String::new());

        lines.push("[colors.normal]".to_string());
        let mut sorted_names: Vec<&str> = COLOR_ORDER.to_vec();
        sorted_names.sort();
        for name in sorted_names {
            let val = c.normal.get(name).map(|s| s.as_str()).unwrap_or("#000000");
            lines.push(format!("{} = '{}'", name, ensure_hash(val)));
        }
        lines.push(String::new());

        lines.push("[colors.primary]".to_string());
        lines.push(format!("background = '{}'", ensure_hash(&c.background)));
        lines.push(format!("foreground = '{}'", ensure_hash(&c.foreground)));
        lines.push(String::new());

        lines.push("[colors.selection]".to_string());
        lines.push(format!("background = '{}'", ensure_hash(&c.selection_bg)));
        lines.push(format!("text = '{}'", ensure_hash(&c.selection_fg)));

        lines.join("\n") + "\n"
    }

    pub fn generate_wezterm(&self) -> String {
        let c = &self.colors;
        let mut lines = vec!["[colors]".to_string()];

        lines.push("ansi = [".to_string());
        for name in &COLOR_ORDER {
            let val = c.normal.get(*name).map(|s| s.as_str()).unwrap_or("#000000");
            lines.push(format!("    \"{}\",", ensure_hash(val)));
        }
        lines.push("]".to_string());

        lines.push(format!("background = \"{}\"", ensure_hash(&c.background)));

        lines.push("brights = [".to_string());
        for name in &COLOR_ORDER {
            let val = c.bright.get(*name).map(|s| s.as_str()).unwrap_or("#000000");
            lines.push(format!("    \"{}\",", ensure_hash(val)));
        }
        lines.push("]".to_string());

        lines.push(format!("compose_cursor = \"{}\"", ensure_hash(&c.compose_cursor)));
        lines.push(format!("cursor_bg = \"{}\"", ensure_hash(&c.cursor)));
        lines.push(format!("cursor_border = \"{}\"", ensure_hash(&c.cursor)));
        lines.push(format!("cursor_fg = \"{}\"", ensure_hash(&c.cursor_text)));
        lines.push(format!("foreground = \"{}\"", ensure_hash(&c.foreground)));
        lines.push(format!("scrollbar_thumb = \"{}\"", ensure_hash(&c.scrollbar_thumb)));
        lines.push(format!("selection_bg = \"{}\"", ensure_hash(&c.selection_bg)));
        lines.push(format!("selection_fg = \"{}\"", ensure_hash(&c.selection_fg)));
        lines.push(format!("split = \"{}\"", ensure_hash(&c.split)));
        lines.push(format!("visual_bell = \"{}\"", ensure_hash(&c.visual_bell)));

        lines.push(String::new());
        lines.push("[colors.indexed]".to_string());
        let mut sorted_indices: Vec<u32> = c.indexed.keys().copied().collect();
        sorted_indices.sort();
        for idx in sorted_indices {
            if let Some(val) = c.indexed.get(&idx) {
                lines.push(format!("{idx} = \"{}\"", ensure_hash(val)));
            }
        }

        lines.push(String::new());
        lines.push("[colors.tab_bar]".to_string());
        lines.push(format!("background = \"{}\"", ensure_hash(&c.tab_bar.background)));
        lines.push(format!(
            "inactive_tab_edge = \"{}\"",
            ensure_hash(&c.tab_bar.inactive_tab_edge)
        ));

        let sections: Vec<(&str, &str, &(String, String))> = vec![
            ("active_tab", "activeTab", &c.tab_bar.active_tab),
            ("inactive_tab", "inactiveTab", &c.tab_bar.inactive_tab),
            ("inactive_tab_hover", "inactiveTabHover", &c.tab_bar.inactive_tab_hover),
            ("new_tab", "newTab", &c.tab_bar.new_tab),
            ("new_tab_hover", "newTabHover", &c.tab_bar.new_tab_hover),
        ];

        for (key, _py_key, colors) in &sections {
            lines.push(String::new());
            lines.push(format!("[colors.tab_bar.{}]", key));
            lines.push(format!("bg_color = \"{}\"", ensure_hash(&colors.0)));
            lines.push(format!("fg_color = \"{}\"", ensure_hash(&colors.1)));
            lines.push("intensity = \"Normal\"".to_string());
            lines.push("italic = false".to_string());
            lines.push("strikethrough = false".to_string());
            lines.push("underline = \"None\"".to_string());
        }

        lines.push(String::new());
        lines.push("[metadata]".to_string());
        lines.push("author = \"CrawlDS\"".to_string());
        lines.push("name = \"CrawlDS\"".to_string());

        lines.join("\n") + "\n"
    }

    pub fn generate(&self, terminal_id: &str) -> Result<String, String> {
        match terminal_id {
            "foot" => Ok(self.generate_foot()),
            "ghostty" => Ok(self.generate_ghostty()),
            "kitty" => Ok(self.generate_kitty()),
            "alacritty" => Ok(self.generate_alacritty()),
            "wezterm" => Ok(self.generate_wezterm()),
            _ => Err(format!("Unknown terminal: {}", terminal_id)),
        }
    }
}

/// Write terminal theme files from a predefined scheme's terminal section.
///
/// `scheme_data` should contain the 14-color scheme keys (cPrimary, etc.)
/// plus a `terminal` key whose value is a JSON object with foreground,
/// background, normal, bright and other terminal color fields.
/// `outputs` maps terminal IDs ("foot", "kitty", etc.) to output file paths.
fn extract_scheme_colors(scheme_data: &serde_json::Value) -> HashMap<String, String> {
    let mut colors = HashMap::new();
    for key in &["cPrimary", "cOnPrimary", "cSecondary", "cSurfaceVariant"] {
        if let Some(v) = scheme_data.get(*key).and_then(|v| v.as_str()) {
            colors.insert(key.to_string(), v.to_string());
        }
    }
    colors
}

/// Write terminal theme files from a predefined scheme's terminal section.
///
/// `scheme_data` should contain the 14-color scheme keys (cPrimary, etc.)
/// plus a `terminal` key whose value is a JSON object with foreground,
/// background, normal, bright and other terminal color fields.
/// `outputs` maps terminal IDs ("foot", "kitty", etc.) to output file paths.
pub fn write_terminal_themes(
    scheme_data: &serde_json::Value,
    outputs: &HashMap<String, String>,
) -> Result<(), String> {
    let terminal_data = scheme_data
        .get("terminal")
        .ok_or_else(|| "No terminal section in scheme data".to_string())?;

    let scheme_colors = extract_scheme_colors(scheme_data);
    let terminal_colors = TerminalColors::from_dict(terminal_data, &scheme_colors);
    let generator = TerminalGenerator::new(terminal_colors);

    for (terminal_id, output_path) in outputs {
        let content = generator.generate(terminal_id)?;
        let path = std::path::Path::new(output_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir for {}: {}", terminal_id, e))?;
        }
        std::fs::write(path, &content)
            .map_err(|e| format!("Failed to write {}: {}", terminal_id, e))?;
    }

    Ok(())
}
