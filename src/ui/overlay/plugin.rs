//! OverlayPlugin — UI 浮层系统 Plugin

use bevy::prelude::*;

use super::damage_text::tick_damage_numbers;
use super::debug::DebugOverlay;
use super::layers::create_ui_roots;
use super::notification::{NotificationService, process_notification_queue, tick_notifications};
use super::tooltip::TooltipService;

/// UI 浮层系统 Plugin
///
/// 注册顺序（在 ThemePlugin 和 ScreenPlugin 之后）：
/// 1. Startup: create_ui_roots — 5 层 UI 根节点
/// 2. 注册 Service Resource（NotificationService, TooltipService）
/// 3. Update: 通知队列处理 + 生命周期管理
pub struct OverlayPlugin;

impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_ui_roots)
            .init_resource::<NotificationService>()
            .init_resource::<TooltipService>()
            .register_type::<DebugOverlay>()
            .add_systems(
                Update,
                (
                    process_notification_queue,
                    tick_notifications,
                    tick_damage_numbers,
                ),
            );
    }
}
