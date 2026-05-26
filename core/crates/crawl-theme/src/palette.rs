use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::color::Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Palette {
    // Base surfaces
    pub base: Color,
    pub mantle: Color,
    pub crust: Color,

    // Surface layers
    pub surface0: Color,
    pub surface1: Color,
    pub surface2: Color,

    // Overlay layers
    pub overlay0: Color,
    pub overlay1: Color,
    pub overlay2: Color,

    // Text
    pub text: Color,
    pub subtext0: Color,
    pub subtext1: Color,

    // Accent colors
    pub rosewater: Color,
    pub flamingo: Color,
    pub pink: Color,
    pub mauve: Color,
    pub red: Color,
    pub maroon: Color,
    pub peach: Color,
    pub yellow: Color,
    pub green: Color,
    pub teal: Color,
    pub sky: Color,
    pub sapphire: Color,
    pub blue: Color,
    pub lavender: Color,

    // Extension point for user-defined colors
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra: HashMap<String, Color>,
}

impl Palette {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        base: Color,
        mantle: Color,
        crust: Color,
        surface0: Color,
        surface1: Color,
        surface2: Color,
        overlay0: Color,
        overlay1: Color,
        overlay2: Color,
        text: Color,
        subtext0: Color,
        subtext1: Color,
        rosewater: Color,
        flamingo: Color,
        pink: Color,
        mauve: Color,
        red: Color,
        maroon: Color,
        peach: Color,
        yellow: Color,
        green: Color,
        teal: Color,
        sky: Color,
        sapphire: Color,
        blue: Color,
        lavender: Color,
    ) -> Self {
        Self {
            base,
            mantle,
            crust,
            surface0,
            surface1,
            surface2,
            overlay0,
            overlay1,
            overlay2,
            text,
            subtext0,
            subtext1,
            rosewater,
            flamingo,
            pink,
            mauve,
            red,
            maroon,
            peach,
            yellow,
            green,
            teal,
            sky,
            sapphire,
            blue,
            lavender,
            extra: HashMap::new(),
        }
    }

    pub fn accent(&self, name: &str) -> Option<&Color> {
        match name {
            "rosewater" => Some(&self.rosewater),
            "flamingo" => Some(&self.flamingo),
            "pink" => Some(&self.pink),
            "mauve" => Some(&self.mauve),
            "red" => Some(&self.red),
            "maroon" => Some(&self.maroon),
            "peach" => Some(&self.peach),
            "yellow" => Some(&self.yellow),
            "green" => Some(&self.green),
            "teal" => Some(&self.teal),
            "sky" => Some(&self.sky),
            "sapphire" => Some(&self.sapphire),
            "blue" => Some(&self.blue),
            "lavender" => Some(&self.lavender),
            _ => self.extra.get(name),
        }
    }

    pub fn to_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("base".into(), self.base.hex());
        map.insert("mantle".into(), self.mantle.hex());
        map.insert("crust".into(), self.crust.hex());
        map.insert("surface0".into(), self.surface0.hex());
        map.insert("surface1".into(), self.surface1.hex());
        map.insert("surface2".into(), self.surface2.hex());
        map.insert("overlay0".into(), self.overlay0.hex());
        map.insert("overlay1".into(), self.overlay1.hex());
        map.insert("overlay2".into(), self.overlay2.hex());
        map.insert("text".into(), self.text.hex());
        map.insert("subtext0".into(), self.subtext0.hex());
        map.insert("subtext1".into(), self.subtext1.hex());
        map.insert("rosewater".into(), self.rosewater.hex());
        map.insert("flamingo".into(), self.flamingo.hex());
        map.insert("pink".into(), self.pink.hex());
        map.insert("mauve".into(), self.mauve.hex());
        map.insert("red".into(), self.red.hex());
        map.insert("maroon".into(), self.maroon.hex());
        map.insert("peach".into(), self.peach.hex());
        map.insert("yellow".into(), self.yellow.hex());
        map.insert("green".into(), self.green.hex());
        map.insert("teal".into(), self.teal.hex());
        map.insert("sky".into(), self.sky.hex());
        map.insert("sapphire".into(), self.sapphire.hex());
        map.insert("blue".into(), self.blue.hex());
        map.insert("lavender".into(), self.lavender.hex());
        for (k, v) in &self.extra {
            map.insert(k.clone(), v.hex());
        }
        map
    }
}
