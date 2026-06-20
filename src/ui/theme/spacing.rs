//! UiSpacing — Design spacing scale tokens
//!
//! Widgets must reference semantic spacing tokens (e.g., `theme.spacing.md`)
//! instead of raw `Val::Px(16.0)` values. This ensures consistent spacing
//! across the entire UI and enables global spacing adjustments.
//!
//! See `docs/06-ui/02-design-system/theme-localization.md` §2.4

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Design spacing scale for the UI theme.
///
/// All values are in logical pixels. Widgets MUST NOT hardcode
/// `Val::Px(...)` — always reference a token from this struct.
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct UiSpacing {
    // ── Spacing scale ──
    /// Extra small spacing (4px)
    pub xs: f32,
    /// Small spacing (8px)
    pub sm: f32,
    /// Medium spacing (16px)
    pub md: f32,
    /// Large spacing (24px)
    pub lg: f32,
    /// Extra large spacing (32px)
    pub xl: f32,
    /// Double extra large spacing (48px)
    pub xxl: f32,

    // ── Specific sizes ──
    /// Small border radius (4px)
    pub border_radius_sm: f32,
    /// Medium border radius (8px)
    pub border_radius_md: f32,
    /// Large border radius (12px)
    pub border_radius_lg: f32,
    /// Standard icon size (32px)
    pub icon_size: f32,
    /// Standard button height (40px)
    pub button_height: f32,
    /// Minimum touch target size (44px, accessibility)
    pub min_touch_target: f32,
}

impl UiSpacing {
    /// Default spacing values (dark and light themes share the same scale).
    pub fn default_scale() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 16.0,
            lg: 24.0,
            xl: 32.0,
            xxl: 48.0,
            border_radius_sm: 4.0,
            border_radius_md: 8.0,
            border_radius_lg: 12.0,
            icon_size: 32.0,
            button_height: 40.0,
            min_touch_target: 44.0,
        }
    }
}

impl Default for UiSpacing {
    fn default() -> Self {
        Self::default_scale()
    }
}
