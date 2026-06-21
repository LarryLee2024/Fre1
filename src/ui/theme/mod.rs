//! Module Name: Theme — UI theme system (ThemePlugin + style tokens)
//!
//! Registers the global `Theme` resource and all style token types
//! for reflection support. Must be added first in the UI plugin chain
//! so that widgets and screens can access theme values during construction.
//!
//! See `docs/06-ui/01-architecture/architecture.md` §8.2

pub mod colors;
pub mod resource;
pub mod spacing;
pub mod switch;
pub mod typography;

pub use colors::UiColors;
pub use resource::Theme;
pub use spacing::UiSpacing;
pub use switch::ThemeVariant;
pub use typography::UiTypography;

use bevy::prelude::*;

/// Plugin that registers the global Theme resource and reflect types.
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        // Insert the default (dark) theme as a global resource.
        app.init_resource::<Theme>();

        // Register all theme types for reflection support.
        app.register_type::<UiColors>();
        app.register_type::<UiSpacing>();
        app.register_type::<UiTypography>();
    }
}
