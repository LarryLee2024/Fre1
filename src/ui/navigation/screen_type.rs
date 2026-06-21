//! 导航栈的屏幕类型标识符
//!
//! 定义 ScreenStack 管理的规范屏幕类型集合。
//! 每个变体对应一个不同的全屏视图。

use bevy::prelude::*;

/// 标识导航栈中的屏幕类型。
///
/// 被 ScreenStack 用于跟踪导航历史，被 UiScreenState
/// 用于表示当前屏幕的生命周期状态。
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
