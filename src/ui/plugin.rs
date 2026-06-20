//! UiPlugin — UI 表现层主 Plugin
//!
//! 注册顺序（Phase 11，在 Infra Phase 8 和 ScenePlugin Phase 9 之后）：
//! 1. ThemePlugin       — 主题与设计令牌
//! 2. PrimitivesPlugin  — UI 原语（Button/Panel/Text/List/etc.）
//! 3. WidgetsPlugin     — 游戏业务控件（当前为骨架）
//! 4. ScreenPlugin      — 全屏页面（主菜单等）
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §8

use bevy::prelude::*;

use super::primitives::PrimitivesPlugin;
use super::screens::ScreenPlugin;
use super::theme::ThemePlugin;
use super::widgets::WidgetsPlugin;

/// UiPlugin — L3 UI 表现层入口
///
/// 注册顺序不可变：Theme → Primitives → Widgets → Screens。
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // 1. Theme — 必须在所有 UI 组件之前注册
        app.add_plugins(ThemePlugin);
        // 2. Primitives — 基础 UI 原语
        app.add_plugins(PrimitivesPlugin);
        // 3. Widgets — 游戏业务控件（骨架阶段）
        app.add_plugins(WidgetsPlugin);
        // 4. Screens — 全屏页面（主菜单等）
        app.add_plugins(ScreenPlugin);
    }
}
