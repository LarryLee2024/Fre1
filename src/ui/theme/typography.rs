//! UiTypography — Font and text style tokens
//!
//! Widgets must reference typography tokens (e.g., `theme.typography.size_body`)
//! instead of raw font sizes or hardcoded font paths. This enables consistent
//! text styling and global typography changes.
//!
//! See `docs/06-ui/02-design-system/theme-localization.md` §2.5

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Typography tokens for the UI theme.
///
/// Font paths reference assets in the `assets/fonts/` directory.
/// Widgets MUST NOT hardcode font sizes or font paths — always
/// reference a token from this struct.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct UiTypography {
    // ── Font paths ──
    /// Path to the body text font asset
    pub font_body: String,
    /// Path to the heading/title font asset
    pub font_heading: String,
    /// Path to the monospace font asset (for numbers, code)
    pub font_mono: String,

    // ── Font sizes ──
    /// Body text size (14px)
    pub size_body: f32,
    /// Small/caption text size (12px)
    pub size_small: f32,
    /// Heading text size (18px)
    pub size_heading: f32,
    /// Title text size (24px)
    pub size_title: f32,
    /// Display/large title size (36px)
    pub size_display: f32,

    // ── Font weights ──
    /// Normal font weight
    pub weight_normal: f32,
    /// Bold font weight
    pub weight_bold: f32,
}

impl UiTypography {
    /// Default typography values (shared across themes).
    pub fn default_values() -> Self {
        Self {
            font_body: "fonts/FiraSans-Regular.ttf".into(),
            font_heading: "fonts/FiraSans-Bold.ttf".into(),
            font_mono: "fonts/FiraCode-Regular.ttf".into(),
            size_body: 14.0,
            size_small: 12.0,
            size_heading: 18.0,
            size_title: 24.0,
            size_display: 36.0,
            weight_normal: 400.0,
            weight_bold: 700.0,
        }
    }
}

impl Default for UiTypography {
    fn default() -> Self {
        Self::default_values()
    }
}
