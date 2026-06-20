//! Theme — Global UI theme resource
//!
//! The `Theme` struct is a global ECS Resource holding all style tokens
//! (colors, spacing, typography). It is registered first in the UI plugin
//! chain so that all widgets and screens can access theme values.
//!
//! Dark theme is the default. Light theme is provided as a variant for
//! future theme-switching support.
//!
//! See `docs/06-ui/02-design-system/theme-localization.md` §2

use bevy::prelude::*;

use super::colors::UiColors;
use super::spacing::UiSpacing;
use super::typography::UiTypography;

/// Global UI theme resource.
///
/// Inserted into the ECS by `ThemePlugin`. All UI widgets access style
/// tokens through this resource — never through hardcoded values.
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct Theme {
    /// Theme name identifier (e.g., "dark", "light")
    pub name: &'static str,
    /// Semantic color tokens
    pub colors: UiColors,
    /// Spacing scale tokens
    pub spacing: UiSpacing,
    /// Typography tokens
    pub typography: UiTypography,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Create the dark theme variant.
    pub fn dark() -> Self {
        Self {
            name: "dark",
            colors: UiColors::dark(),
            spacing: UiSpacing::default_scale(),
            typography: UiTypography::default_values(),
        }
    }

    /// Create the light theme variant.
    pub fn light() -> Self {
        Self {
            name: "light",
            colors: UiColors::light(),
            spacing: UiSpacing::default_scale(),
            typography: UiTypography::default_values(),
        }
    }
}
