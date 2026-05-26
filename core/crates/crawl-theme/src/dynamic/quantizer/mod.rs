//! Color Quantizer Module
//!
//! Provides Wu and WSMeans color quantization algorithms for extracting
//! color palettes from images.

pub mod kmeans;
pub mod score;
pub mod strategies;
pub mod wsmeans;
pub mod wu;

pub use kmeans::kmeans_cluster;
pub use score::{extract_source_color, score_colors};
pub use strategies::{extract_palette, find_error_color};
pub use wsmeans::quantize_wsmeans;
pub use wu::{QuantizerWu, quantize_wu};
