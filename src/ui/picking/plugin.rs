//! PickingUiPlugin — Picking 表现层插件
//!
//! 注册 Picking 后端配置（Sprite）和全局 Observer（Click / Hover）。
//! 不包含任何业务逻辑。Selection 状态管理由 selection/plugin.rs 处理。
//!
//! 从 `infra/picking/plugin.rs` 迁移。
//!
//! 详见 ADR-068 §Module Design。

use bevy::picking::PickingSettings;
use bevy::prelude::*;

use super::backend::sprite::configure_sprite_picking;
use super::intent::click::on_pointer_click;
use super::intent::hover::{on_pointer_out, on_pointer_over};

/// Debug: 全局 Pointer<Click> 观察者
fn debug_click_handler(ev: On<Pointer<Click>>) {
    let hit_pos = ev
        .event()
        .hit
        .position
        .map(|p| format!("({:.0},{:.0})", p.x, p.y));
    info!(
        target: "ui::picking",
        "[DEBUG] Global click: target={:?} button={:?} pos={:?}",
        ev.event_target(),
        ev.event().button,
        hit_pos.unwrap_or_default(),
    );
}

/// Debug: 全局 Pointer<Over> 观察者
fn debug_hover_handler(ev: On<Pointer<Over>>) {
    info!(
        target: "ui::picking",
        "[DEBUG] Global over observer: target={:?}",
        ev.event_target(),
    );
}

/// Picking UI 表现层插件
///
/// - 设置 SpritePickingMode::BoundingBox（纯色方块不需要 Alpha 检测）
/// - 注册全局 PickIntent Observer（Click / Hover）
/// - Debug observer 保留（Phase 4 清理）
///
/// 必须在 UiPlugin 之前注册。详见 ADR-068。
pub struct PickingUiPlugin;

impl Plugin for PickingUiPlugin {
    fn build(&self, app: &mut App) {
        // 1. 配置 Sprite Picking 后端
        configure_sprite_picking(app);

        // 2. PickingSettings（默认配置）
        app.insert_resource(PickingSettings { ..default() })
            // 3. Debug observer（Phase 4 清理）
            .add_observer(debug_click_handler)
            .add_observer(debug_hover_handler)
            // 4. 全局 PickIntent observer
            .add_observer(on_pointer_click)
            .add_observer(on_pointer_over)
            .add_observer(on_pointer_out);
    }
}
