//! UiPlugin — UI 表现层主 Plugin
//!
//! 注册顺序（Phase 11，在 Infra Phase 8 和 ScenePlugin Phase 9 之后）：
//! 1. ThemePlugin     — 主题与设计令牌
//! 2. WidgetsPlugin   — Widget 组件
//!
//! 详见 `docs/06-ui/01-architecture/architecture.md` §8

use bevy::prelude::*;

use super::theme::ThemePlugin;
use super::widgets::WidgetsPlugin;

/// UiPlugin — L3 UI 表现层入口
///
/// 注册所有 UI 子系统：Theme → Widgets → Overlay → Screens。
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        // 1. Theme — 必须在 Widget 之前注册
        app.add_plugins(ThemePlugin);
        // 2. Widgets — 可复用组件
        app.add_plugins(WidgetsPlugin);
        // （后续扩展：Focus, Navigation, Overlay, Screens ...）
    }
}
