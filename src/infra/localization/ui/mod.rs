//! ui — Presentation layer for localized text.
//!
//! Contains the LocalizedText component and the render system.

pub(crate) mod components;
pub(crate) mod render;

pub use components::LocalizedText;
pub use render::render_localized_text;
