//! Module Name: UiScreenState — Current screen lifecycle state
//!
//! Tracks the currently active screen and its lifecycle phase.
//! Paired with ScreenStack to provide complete navigation state.

use bevy::prelude::*;

use super::screen_type::ScreenType;

/// Represents the lifecycle phase of the current screen.
///
/// Screens move through these states: Defined → Loading → Active,
/// then on navigation away: Active → Background, or on removal:
/// Active → Destroyed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum ScreenLifecycle {
    /// Screen is registered but not yet loaded.
    Defined,
    /// Screen resources are being loaded (assets, view models, observers).
    Loading,
    /// Screen is fully active and interactive.
    Active,
    /// Screen is in the background (another screen is on top).
    Background,
    /// Screen has been destroyed and its entities should be despawned.
    Destroyed,
}

/// Tracks the current screen's identity and lifecycle phase.
///
/// Stored as a Bevy Resource. Unlike ScreenStack which tracks navigation
/// history, UiScreenState captures only the immediate screen state and
/// serves as a quick-lookup for systems that need the current context.
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct UiScreenState {
    /// The currently active (top-most) screen type.
    pub current_screen: Option<ScreenType>,
    /// Lifecycle phase of the current screen.
    pub lifecycle: ScreenLifecycle,
    /// The previous screen before the current one was pushed.
    pub previous_screen: Option<ScreenType>,
}

impl UiScreenState {
    /// Creates a new UiScreenState with no current screen.
    pub fn new() -> Self {
        Self {
            current_screen: None,
            lifecycle: ScreenLifecycle::Defined,
            previous_screen: None,
        }
    }
}

impl Default for UiScreenState {
    fn default() -> Self {
        Self::new()
    }
}
