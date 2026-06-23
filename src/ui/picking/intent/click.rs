//! Click Intent — Pointer<Click> 事件 → PickIntent 转换
//!
//! 全局 `On<Pointer<Click>>` 观察者，将 Bevy 原生点击事件转换为
//! 业务层可消费的 PickIntent 事件。
//!
//! 后续由 selection/bridge.rs 消费 PickIntent 并触发领域事件。
//!
//! 详见 ADR-068 §Module Design。

use bevy::prelude::*;

use crate::core::domains::combat::components::UnitIdComponent;
use crate::core::domains::tactical::components::GridPos;
use crate::ui::picking::pick_target::{InteractionPhase, PickIntent, PickTarget};

/// 全局 Pointer<Click> 观察者
///
/// 检测点击目标：
/// - 如果目标实体有 UnitIdComponent → PickTarget::Unit
/// - 如果目标实体有 GridPos → PickTarget::Tile
/// - 否则 → PickTarget::Empty
///
/// 仅在 Primary（左键）点击时生成 PickIntent。
/// Secondary（右键）由 bridge 层处理。
pub fn on_pointer_click(
    ev: On<Pointer<Click>>,
    mut commands: Commands,
    unit_ids: Query<&UnitIdComponent>,
    grid_positions: Query<&GridPos>,
) {
    // 跳过非左键点击（右键由 bridge 层处理）
    if ev.event().button != PointerButton::Primary {
        info!(
            target: "ui::picking",
            "[Picking] Non-primary click: {:?} — skipping",
            ev.event().button,
        );
        return;
    }

    let target_entity = ev.event_target();

    // 诊断日志：打印目标实体的所有组件
    info!(
        target: "ui::picking",
        "[Picking] Click entity={:?} — UnitId={:?} GridPos={:?}",
        target_entity,
        unit_ids.get(target_entity).ok(),
        grid_positions.get(target_entity).ok(),
    );

    let target = resolve_pick_target(target_entity, &unit_ids, &grid_positions);

    info!(
        target: "ui::picking",
        "[Picking] Click: target={:?} entity={:?}",
        target,
        target_entity,
    );

    commands.trigger(PickIntent {
        target,
        phase: InteractionPhase::Commit,
        button: ev.event().button,
    });
}

/// 将点击目标解析为 PickTarget
fn resolve_pick_target(
    entity: Entity,
    unit_ids: &Query<&UnitIdComponent>,
    grid_positions: &Query<&GridPos>,
) -> PickTarget {
    if let Ok(uid) = unit_ids.get(entity) {
        return PickTarget::Unit(uid.id.clone());
    }
    if let Ok(pos) = grid_positions.get(entity) {
        return PickTarget::Tile(*pos);
    }
    PickTarget::Empty
}
