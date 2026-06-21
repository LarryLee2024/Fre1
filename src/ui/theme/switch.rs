//! ThemeVariant and switch_theme — Runtime theme switching
//!
//! Defines the canonical theme variants (Dark, Light) and provides
//! the `switch_theme` function that updates the global Theme resource.
//!
//! Theme switching only updates the Theme resource; existing spawned
//! UI elements retain their original colors until they are refreshed
//! by their owning system (e.g., on screen re-spawn).
//!
//! See `docs/06-ui/02-design-system/theme-localization.md` §2.4

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::Theme;

/// Theme variant identifier.
///
/// Used by theme switching commands and settings persistence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum ThemeVariant {
    /// Dark theme (default)
    Dark,
    /// Light theme
    Light,
}

impl ThemeVariant {
    /// Returns the string representation of this variant.
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeVariant::Dark => "dark",
            ThemeVariant::Light => "light",
        }
    }
}

impl Default for ThemeVariant {
    fn default() -> Self {
        ThemeVariant::Dark
    }
}

impl std::fmt::Display for ThemeVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Switch the global Theme resource to a different variant.
///
/// This function updates the Theme resource's colors to match the
/// requested variant. It does NOT automatically update already-spawned
/// UI elements — those are refreshed by their owning screens/systems.
///
/// # Parameters
/// - `theme`: The global Theme resource (ResMut)
/// - `variant`: The target theme variant
pub fn switch_theme(mut theme: ResMut<Theme>, variant: ThemeVariant) {
    match variant {
        ThemeVariant::Dark => {
            theme.colors = super::colors::UiColors::dark();
            theme.name = "dark";
        }
        ThemeVariant::Light => {
            theme.colors = super::colors::UiColors::light();
            theme.name = "light";
        }
    }
}
