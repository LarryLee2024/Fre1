//! PickingPlugin — 配置 Bevy picking 管线
//!
//! DefaultPlugins 已包含 DefaultPickingPlugins（bevy_picking feature），
//! SpritePlugin 已包含 SpritePickingPlugin。
//! 本模块只需覆写配置 + 注册 Selection 资源。

use bevy::picking::PickingSettings;
use bevy::prelude::*;
use bevy::sprite::{SpritePickingMode, SpritePickingSettings};

use super::selection::Selection;

/// Debug: 全局 Pointer<Click> 观察者
fn debug_click_handler(ev: On<Pointer<Click>>) {
    let hit_pos = ev
        .event()
        .hit
        .position
        .map(|p| format!("({:.0},{:.0})", p.x, p.y));
    println!(
        "[DEBUG] Global click: target={:?} button={:?} pos={:?}",
        ev.event_target(),
        ev.event().button,
        hit_pos.unwrap_or_default(),
    );
}

/// Debug: 全局 Pointer<Over> 观察者
fn debug_hover_handler(ev: On<Pointer<Over>>) {
    println!(
        "[DEBUG] Global over observer: target={:?}",
        ev.event_target(),
    );
}

/// Picking 基础设施插件
///
/// - 设置 SpritePickingMode::BoundingBox（纯色方块不需要 Alpha 检测）
/// - 禁用窗口级 picking（减少 UI 树干扰）
/// - 注册 Selection 资源
///
/// 必须在 Phase 8 注册（Camera 之后，UI 之前）。详见 ADR-067。
pub struct PickingPlugin;

impl Plugin for PickingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpritePickingSettings {
            // BoundingBox：纯色方块不需要 Alpha 检测，性能更好
            picking_mode: SpritePickingMode::BoundingBox,
            // 不需要标记摄像头，所有 Camera2d 参与 picking
            require_markers: false,
        })
        .insert_resource(PickingSettings { ..default() })
        .init_resource::<Selection>()
        .add_observer(debug_click_handler)
        .add_observer(debug_hover_handler);
    }
}
