//! Module Name: Screens — 全屏 UI 视图（页面级）
//!
//! 每个屏幕对应一个独立、排他的全屏 UI 视图，通过 Startup 系统或 Scene 状态切换触发。
//! 屏幕层是 UI 架构的最高层，组合 Primitives 和 Widgets 构建完整页面。
//!
//! 当前实现：
//! - MainMenuScreen: 主菜单（标题画面）

pub mod main_menu;

use bevy::prelude::*;

use main_menu::{spawn_main_menu, systems::on_main_menu_button_clicked};

/// ScreenPlugin — 注册所有屏幕的 System 和 Observer
///
/// 在 WidgetsPlugin 之后注册。当前使用 Startup 系统启动主菜单，
/// 未来将迁移到 `OnEnter(GameState::MainMenu)` + `OnExit(...)` 模式。
pub struct ScreenPlugin;

impl Plugin for ScreenPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册类型
            .register_type::<main_menu::MenuAction>()
            .register_type::<main_menu::MainMenuScreen>()
            // Startup 系统：应用启动时生成主菜单
            .add_systems(Startup, spawn_main_menu)
            // Observer：监听主菜单按钮点击
            .add_observer(on_main_menu_button_clicked);
    }
}
