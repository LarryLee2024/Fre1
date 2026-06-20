//! UiColors — Semantic color token definitions
//!
//! Widgets must reference semantic tokens (e.g., `theme.colors.text_primary`)
//! instead of raw RGB values. This enables theme switching without touching
//! widget code.
//!
//! See `docs/06-ui/02-design-system/theme-localization.md` §2.3

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Semantic color tokens for the UI theme.
///
/// All values are defined in sRGB color space. Widgets MUST NOT use
/// raw `Color::srgb(...)` or `Color::WHITE` / `Color::BLACK` — always
/// reference a token from this struct.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct UiColors {
    // ── UI surface colors ──
    /// Primary surface background (panels, menus)
    pub surface_primary: Color,
    /// Secondary surface background (sub-panels, tooltips)
    pub surface_secondary: Color,
    /// Surface for danger/destructive actions
    pub surface_danger: Color,
    /// Disabled surface (non-interactive elements)
    pub surface_disabled: Color,
    /// Surface secondary hover state
    pub surface_secondary_hover: Color,
    /// Surface secondary pressed state
    pub surface_secondary_pressed: Color,
    /// Surface danger hover state
    pub surface_danger_hover: Color,
    /// Surface danger pressed state
    pub surface_danger_pressed: Color,

    // ── Text colors ──
    /// Primary text color (body content)
    pub text_primary: Color,
    /// Secondary text color (labels, captions)
    pub text_secondary: Color,
    /// Disabled text color
    pub text_disabled: Color,
    /// Accent text color (highlights, key values)
    pub text_accent: Color,

    // ── Accent / interaction colors ──
    /// Primary accent color (buttons, interactive elements)
    pub accent_primary: Color,
    /// Hover state accent
    pub accent_hover: Color,
    /// Pressed state accent
    pub accent_pressed: Color,

    // ── Ghost button colors (transparent background variants) ──
    /// Ghost button default background (transparent)
    pub ghost: Color,
    /// Ghost button hover background
    pub ghost_hover: Color,
    /// Ghost button pressed background
    pub ghost_pressed: Color,

    // ── Feedback colors ──
    /// Positive feedback (healing, success)
    pub feedback_positive: Color,
    /// Negative feedback (damage, error)
    pub feedback_negative: Color,
    /// Warning feedback (caution, alert)
    pub feedback_warning: Color,

    // ── Border colors ──
    /// Default border color
    pub border_default: Color,
    /// Focus/highlight border color
    pub border_focus: Color,
}

impl UiColors {
    /// Dark theme color palette.
    pub fn dark() -> Self {
        Self {
            surface_primary: Color::srgb(0.11, 0.11, 0.14),
            surface_secondary: Color::srgb(0.16, 0.16, 0.20),
            surface_danger: Color::srgb(0.24, 0.08, 0.08),
            surface_disabled: Color::srgb(0.18, 0.18, 0.20),
            surface_secondary_hover: Color::srgb(0.20, 0.20, 0.25),
            surface_secondary_pressed: Color::srgb(0.13, 0.13, 0.17),
            surface_danger_hover: Color::srgb(0.30, 0.10, 0.10),
            surface_danger_pressed: Color::srgb(0.18, 0.06, 0.06),
            text_primary: Color::srgb(0.90, 0.90, 0.92),
            text_secondary: Color::srgb(0.65, 0.65, 0.70),
            text_disabled: Color::srgb(0.40, 0.40, 0.45),
            text_accent: Color::srgb(0.94, 0.77, 0.25),
            accent_primary: Color::srgb(0.29, 0.56, 0.85),
            accent_hover: Color::srgb(0.36, 0.63, 0.91),
            accent_pressed: Color::srgb(0.23, 0.47, 0.76),
            ghost: Color::NONE,
            ghost_hover: Color::srgba(1.0, 1.0, 1.0, 0.10),
            ghost_pressed: Color::srgba(1.0, 1.0, 1.0, 0.18),
            feedback_positive: Color::srgb(0.30, 0.69, 0.31),
            feedback_negative: Color::srgb(0.82, 0.18, 0.18),
            feedback_warning: Color::srgb(0.95, 0.61, 0.07),
            border_default: Color::srgb(0.33, 0.33, 0.36),
            border_focus: Color::srgb(0.29, 0.56, 0.85),
        }
    }

    /// Light theme color palette.
    pub fn light() -> Self {
        Self {
            surface_primary: Color::srgb(0.97, 0.97, 0.98),
            surface_secondary: Color::srgb(0.92, 0.92, 0.94),
            surface_danger: Color::srgb(0.98, 0.85, 0.85),
            surface_disabled: Color::srgb(0.88, 0.88, 0.90),
            surface_secondary_hover: Color::srgb(0.88, 0.88, 0.91),
            surface_secondary_pressed: Color::srgb(0.95, 0.95, 0.96),
            surface_danger_hover: Color::srgb(0.97, 0.78, 0.78),
            surface_danger_pressed: Color::srgb(0.99, 0.90, 0.90),
            text_primary: Color::srgb(0.13, 0.13, 0.16),
            text_secondary: Color::srgb(0.45, 0.45, 0.50),
            text_disabled: Color::srgb(0.65, 0.65, 0.70),
            text_accent: Color::srgb(0.71, 0.53, 0.04),
            accent_primary: Color::srgb(0.20, 0.46, 0.78),
            accent_hover: Color::srgb(0.27, 0.53, 0.85),
            accent_pressed: Color::srgb(0.14, 0.38, 0.69),
            ghost: Color::NONE,
            ghost_hover: Color::srgba(0.0, 0.0, 0.0, 0.06),
            ghost_pressed: Color::srgba(0.0, 0.0, 0.0, 0.12),
            feedback_positive: Color::srgb(0.18, 0.55, 0.20),
            feedback_negative: Color::srgb(0.72, 0.08, 0.08),
            feedback_warning: Color::srgb(0.85, 0.51, 0.02),
            border_default: Color::srgb(0.70, 0.70, 0.73),
            border_focus: Color::srgb(0.20, 0.46, 0.78),
        }
    }
}

impl Default for UiColors {
    fn default() -> Self {
        Self::dark()
    }
}
