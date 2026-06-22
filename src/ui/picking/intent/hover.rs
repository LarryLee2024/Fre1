//! Hover Intent — Pointer<Over>/<Out> 事件 → PickIntent 转换
//!
//! 全局 `On<Pointer<Over>>` / `On<Pointer<Out>>` 观察者，
//! 将鼠标悬停事件转换为 PickIntent::Preview / PreviewEnd。
//!
//! 详见 ADR-068 §Module Design。

use bevy::prelude::*;

use crate::core::domains::combat::components::UnitIdComponent;
use crate::core::domains::tactical::components::GridPos;
use crate::ui::picking::pick_target::{PickIntent, PickTarget};

/// 全局 Pointer<Over> 观察者
///
/// 鼠标进入目标时触发 PickIntent::Preview。
pub fn on_pointer_over(
    ev: On<Pointer<Over>>,
    mut commands: Commands,
    unit_ids: Query<&UnitIdComponent>,
    grid_positions: Query<&GridPos>,
) {
    let target = resolve_hover_target(ev.event_target(), &unit_ids, &grid_positions);

    info!(
        target: "ui::picking",
        "[Picking] Hover over: target={:?} entity={:?}",
        target,
        ev.event_target(),
    );

    commands.trigger(PickIntent::preview(target));
}

/// 全局 Pointer<Out> 观察者
///
/// 鼠标离开目标时触发 PickIntent::PreviewEnd。
pub fn on_pointer_out(
    ev: On<Pointer<Out>>,
    mut commands: Commands,
    unit_ids: Query<&UnitIdComponent>,
    grid_positions: Query<&GridPos>,
) {
    let target = resolve_hover_target(ev.event_target(), &unit_ids, &grid_positions);

    info!(
        target: "ui::picking",
        "[Picking] Hover out: target={:?} entity={:?}",
        target,
        ev.event_target(),
    );

    commands.trigger(PickIntent::preview_end(target));
}

/// 解析悬停目标（与 click.rs 共享相同逻辑）
fn resolve_hover_target(
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
