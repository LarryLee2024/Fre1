//! Module Name: Widgets — 可复用 UI 组件集合
//!
//! 每个 Widget 有独立的 Plugin、Factory、Component 和 System。
//! Widget 只通过 Factory 创建，禁止直接 spawn Node。
//! 详见 `docs/06-ui/01-architecture/architecture.md` §3 目录结构

pub mod button;
pub mod progress_bar;

use bevy::prelude::*;

use self::button::ButtonPlugin;
use self::progress_bar::ProgressBarPlugin;

/// WidgetsPlugin — 注册所有 Widget Plugin
///
/// 在 ThemePlugin 之后、Overlay/Screen Plugin 之前注册。
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ButtonPlugin, ProgressBarPlugin));
    }
}
