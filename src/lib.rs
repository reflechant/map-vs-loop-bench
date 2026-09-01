//! Lookup helpers and console plots for the linear-vs-map benchmarks.
//!
//! `cargo bench` measures lookups once, draws a log–log PNG (inlined on
//! Kitty/Ghostty/iTerm), and falls back to a ranked text summary otherwise.

pub mod chart;
pub mod data;
pub mod measure;
pub mod plot;
pub mod term_image;

pub use data::SIZES;
