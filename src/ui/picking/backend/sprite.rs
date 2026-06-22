//! Sprite Picking Backend — 配置 Bevy SpritePickingPlugin
//!
//! 集中管理 SpritePickingSettings 配置。
//! 使用 BoundingBox 模式（纯色方块不需要 Alpha 检测）。
//! 不标记摄像头（所有 Camera2d 参与 picking）。
//!
//! 从 `infra/picking/plugin.rs` 迁移。
//!
//! 详见 ADR-068 §Module Design。

use bevy::prelude::*;
use bevy::sprite::{SpritePickingMode, SpritePickingSettings};

/// 注册 Sprite Picking 后端配置
///
/// 必须在 PickingUiPlugin 中调用。
pub fn configure_sprite_picking(app: &mut App) {
    app.insert_resource(SpritePickingSettings {
        // BoundingBox：纯色方块不需要 Alpha 检测，性能更好
        picking_mode: SpritePickingMode::BoundingBox,
        // 不需要标记摄像头，所有 Camera2d 参与 picking
        require_markers: false,
    });
}
