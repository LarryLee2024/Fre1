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
//! - SettingsScreen: Game settings (theme, gameplay toggles)
//! - SaveLoadScreen: Save / load game slots
//! - ShopScreen: Shop / trading screen (MVP)

pub mod battle;
pub mod inventory;
pub mod main_menu;
pub mod save_load;
pub mod settings;
pub mod shop;

use bevy::prelude::*;

use battle::{spawn_battle_screen, systems::on_battle_button_clicked};
use inventory::{spawn_inventory_screen, systems::on_inventory_button_clicked};
use main_menu::{
    spawn_main_menu,
    systems::{on_main_menu_action, on_main_menu_button_handler},
};
use save_load::{on_close_save_load_screen, on_open_save_load_screen, on_save_load_button_clicked};
use settings::{
    on_close_settings_screen, on_open_settings_screen, on_settings_button_clicked,
    settings_toggle_system,
};
use shop::{on_shop_button_clicked, spawn_shop_screen};

use crate::ui::navigation::ScreenType;

/// ScreenPlugin — registers all screen systems and observers
///
/// Registered after WidgetsPlugin. Currently uses Startup systems for
/// spawning screens; will migrate to `OnEnter(GameState::...)` +
/// `OnExit(...)` in the future.
pub struct ScreenPlugin;

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register types for reflection
            .register_type::<main_menu::MenuAction>()
            .register_type::<main_menu::MainMenuScreen>()
            .register_type::<battle::systems::BattleAction>()
            .register_type::<battle::BattleScreen>()
            .register_type::<inventory::InventoryScreen>()
            .register_type::<settings::SettingsScreen>()
            .register_type::<settings::SettingsAction>()
            .register_type::<settings::SettingsToggle>()
            .register_type::<save_load::SaveLoadScreen>()
            .register_type::<save_load::SaveLoadAction>()
            .register_type::<shop::ShopScreen>()
            .register_type::<ScreenType>()
            // Startup systems: spawn screens on app start
            .add_systems(Startup, spawn_main_menu)
            .add_systems(Startup, spawn_battle_screen)
            .add_systems(Startup, spawn_inventory_screen)
            .add_systems(Startup, spawn_shop_screen)
            // Observers: button click -> UiCommand mapping（方案A）
            // Bevy 0.19: Commands::trigger fires the event, add_observer catches it
            .add_observer(on_main_menu_button_handler)
            .add_observer(on_inventory_button_clicked)
            .add_observer(on_battle_button_clicked)
            // Observers: generic UiAction event handling
            .add_observer(on_main_menu_action)
            // Settings screen: lifecycle observers
            .add_observer(on_open_settings_screen)
            .add_observer(on_close_settings_screen)
            // Settings screen: button click handler
            .add_observer(on_settings_button_clicked)
            // Settings screen: toggle state change handler (Update)
            .add_systems(Update, settings_toggle_system)
            // SaveLoad screen: lifecycle observers
            .add_observer(on_open_save_load_screen)
            .add_observer(on_close_save_load_screen)
            // SaveLoad screen: button click handler
            .add_observer(on_save_load_button_clicked)
            // Shop screen: button click handler
            .add_observer(on_shop_button_clicked);
    }
}
