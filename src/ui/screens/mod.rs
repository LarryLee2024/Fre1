//! 模块名：Screens — 全屏 UI 视图（页面级）
//!
//! 每个 Screen 代表一个互斥的全屏视图，由 Startup 系统
//! 或场景状态转换驱动。
//! Screen 位于 UI 架构的顶层，将 Primitives 和 Widgets
//! 组合成完整页面。
//!
//! 当前实现：
//! - MainMenuScreen：标题 / 主菜单
//! - BattleScreen：战斗界面（MVP）
//! - SettingsScreen：游戏设置（主题、玩法开关）
//! - SaveLoadScreen：存档 / 读档槽位
//! - ShopScreen：商店界面（MVP）

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

/// ScreenPlugin — 注册所有 Screen 系统和 Observer
///
/// 在 WidgetsPlugin 之后注册。当前使用 Startup 系统生成 Screen；
/// 未来将迁移到 `OnEnter(GameState::...)` + `OnExit(...)`。
pub struct ScreenPlugin;

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册反射类型
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
            // Startup 系统：应用启动时生成 Screen
            .add_systems(Startup, spawn_main_menu)
            .add_systems(Startup, spawn_battle_screen)
            .add_systems(Startup, spawn_inventory_screen)
            .add_systems(Startup, spawn_shop_screen)
            // Observer：按钮点击 → UiCommand 映射（方案A）
            // Bevy 0.19：Commands::trigger 触发事件，add_observer 捕获
            .add_observer(on_main_menu_button_handler)
            .add_observer(on_inventory_button_clicked)
            .add_observer(on_battle_button_clicked)
            // Observer：通用 UiAction 事件处理
            .add_observer(on_main_menu_action)
            // Settings 界面：生命周期 Observer
            .add_observer(on_open_settings_screen)
            .add_observer(on_close_settings_screen)
            // Settings 界面：按钮点击处理器
            .add_observer(on_settings_button_clicked)
            // Settings 界面：开关状态变更处理器（Update）
            .add_systems(Update, settings_toggle_system)
            // SaveLoad 界面：生命周期 Observer
            .add_observer(on_open_save_load_screen)
            .add_observer(on_close_save_load_screen)
            // SaveLoad 界面：按钮点击处理器
            .add_observer(on_save_load_button_clicked)
            // Shop 界面：按钮点击处理器
            .add_observer(on_shop_button_clicked);
    }
}
