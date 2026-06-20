//! Module Name: Screens — Full-screen UI views (page level)
//!
//! Each screen represents a mutually exclusive full-screen view, driven by
//! Startup systems or scene state transitions.
//! Screens sit at the top of the UI architecture, composing Primitives and
//! Widgets into complete pages.
//!
//! Current implementation:
//! - MainMenuScreen: Title / main menu
//! - BattleScreen: Battle / combat screen (MVP)

pub mod battle;
pub mod main_menu;

use bevy::prelude::*;

use battle::{spawn_battle_screen, systems::on_battle_button_clicked};
use main_menu::{spawn_main_menu, systems::on_main_menu_button_clicked};

/// ScreenPlugin — registers all screen systems and observers
///
/// Registered after WidgetsPlugin. Currently uses Startup systems for
/// spawning screens; will migrate to `OnEnter(GameState::...)` +
/// `OnExit(...)` in the future.
pub struct ScreenPlugin;

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register types
            .register_type::<main_menu::MenuAction>()
            .register_type::<main_menu::MainMenuScreen>()
            .register_type::<battle::systems::BattleAction>()
            .register_type::<battle::BattleScreen>()
            // Startup systems: spawn screens on app start
            .add_systems(Startup, spawn_main_menu)
            .add_systems(Startup, spawn_battle_screen)
            // Observers: listen for button clicks
            .add_observer(on_main_menu_button_clicked)
            .add_observer(on_battle_button_clicked);
    }
}
