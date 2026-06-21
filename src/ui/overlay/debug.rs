//! DebugOverlay — 调试覆盖层（仅 dev feature）

use bevy::prelude::*;

/// 调试覆盖层标记
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
pub struct DebugOverlay;

/// 切换调试覆盖层（骨架）
pub fn toggle_debug_overlay(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    query: Query<Entity, With<DebugOverlay>>,
) {
    if keyboard.just_pressed(KeyCode::F12) {
        if let Ok(entity) = query.single() {
            commands.entity(entity).despawn();
        }
        // 骨架：暂不实现重新生成逻辑
    }
}
