pub mod colors_json;
pub mod cursor;
pub mod gtk;
pub mod qt;
pub mod terminal;

#[cfg(target_os = "linux")]
pub mod niri;
