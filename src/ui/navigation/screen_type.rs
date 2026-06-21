//! Module Name: ScreenType — Screen type identifiers for navigation stack
//!
//! Defines the canonical set of screen types managed by ScreenStack.
//! Each variant corresponds to a distinct full-screen view.

use bevy::prelude::*;

/// Identifies a screen type in the navigation stack.
///
/// Used by ScreenStack to track navigation history and by UiScreenState
/// to represent the current screen's lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum ScreenType {
    /// Main menu / title screen
    MainMenu,
    /// Battle / combat screen
    Battle,
    /// Inventory / equipment management screen
    Inventory,
    /// Shop / trading screen
    Shop,
    /// Settings / options screen
    Settings,
    /// Save / load screen
    SaveLoad,
}
