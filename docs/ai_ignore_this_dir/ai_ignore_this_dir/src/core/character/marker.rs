// 标记组件：可移动范围、攻击范围、选中高亮
// 这些是表现层标记，由逻辑层添加、表现层读取

use bevy::prelude::*;

use super::components::GridPosition;

/// 可移动范围标记
#[derive(Component)]
pub struct MovableRange;

/// 可攻击范围标记
#[derive(Component)]
pub struct AttackRange;

/// 选中高亮（独立实体）
#[derive(Component)]
pub struct SelectionHighlight;

/// 清除范围标记和高亮（不含 Selected 移除）
pub fn clear_markers(
    commands: &mut Commands,
    range_entities: &Query<
        (Entity, Option<&GridPosition>),
        Or<(With<MovableRange>, With<AttackRange>)>,
    >,
    highlights: &Query<Entity, With<SelectionHighlight>>,
) {
    for (marker, _) in range_entities {
        commands.entity(marker).try_despawn();
    }
    for h in highlights {
        commands.entity(h).try_despawn();
    }
}
