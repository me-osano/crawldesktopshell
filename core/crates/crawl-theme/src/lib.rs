pub mod apply;
pub mod color;
pub mod config;
pub mod contrast;
pub mod dynamic;
pub mod error;
pub mod ipc;
pub mod palette;
pub mod variants;

/// Macro to define a palette function from hex color strings.
/// Used by variant modules to define named palette constants.
#[macro_export]
macro_rules! palette_def {
    ($name:ident, $base:expr, $mantle:expr, $crust:expr,
     $s0:expr, $s1:expr, $s2:expr,
     $o0:expr, $o1:expr, $o2:expr,
     $text:expr, $sub0:expr, $sub1:expr,
     $rose:expr, $flam:expr, $pink:expr,
     $mauve:expr, $red:expr, $maroon:expr,
     $peach:expr, $yellow:expr, $green:expr,
     $teal:expr, $sky:expr, $sapph:expr,
     $blue:expr, $lav:expr) => {
        pub fn $name() -> $crate::Palette {
            $crate::Palette::new(
                $crate::Color::from_hex($base).unwrap(),
                $crate::Color::from_hex($mantle).unwrap(),
                $crate::Color::from_hex($crust).unwrap(),
                $crate::Color::from_hex($s0).unwrap(),
                $crate::Color::from_hex($s1).unwrap(),
                $crate::Color::from_hex($s2).unwrap(),
                $crate::Color::from_hex($o0).unwrap(),
                $crate::Color::from_hex($o1).unwrap(),
                $crate::Color::from_hex($o2).unwrap(),
                $crate::Color::from_hex($text).unwrap(),
                $crate::Color::from_hex($sub0).unwrap(),
                $crate::Color::from_hex($sub1).unwrap(),
                $crate::Color::from_hex($rose).unwrap(),
                $crate::Color::from_hex($flam).unwrap(),
                $crate::Color::from_hex($pink).unwrap(),
                $crate::Color::from_hex($mauve).unwrap(),
                $crate::Color::from_hex($red).unwrap(),
                $crate::Color::from_hex($maroon).unwrap(),
                $crate::Color::from_hex($peach).unwrap(),
                $crate::Color::from_hex($yellow).unwrap(),
                $crate::Color::from_hex($green).unwrap(),
                $crate::Color::from_hex($teal).unwrap(),
                $crate::Color::from_hex($sky).unwrap(),
                $crate::Color::from_hex($sapph).unwrap(),
                $crate::Color::from_hex($blue).unwrap(),
                $crate::Color::from_hex($lav).unwrap(),
            )
        }
    };
}

pub use color::Color;
pub use config::{AccentColor, CursorConfig, GtkConfig, ThemeConfig, ThemeVariant};
pub use dynamic::SchemeType;
pub use dynamic::generate::{expand_predefined_scheme, generate_theme};
pub use error::{ThemeError, ThemeResult};
pub use ipc::ThemeChangedEvent;
pub use palette::Palette;
