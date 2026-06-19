//! Movement System — 移动验证与执行系统
//!
//! 处理单位移动请求：验证路径 → 消耗 MP → 更新位置 → 发布事件。
//!
//! 移动消耗计算通过 Capabilities 管线（Tag → Attribute → Modifier）完成，
//! 通过 `integration::movement::facade` 桥接，严禁直接访问 Capabilities 字段。
//!
//! ECS Query 签名中不可避免需要 Capabilities Component 类型（Bevy 机制限制），
//! 但函数体内所有字段访问都委托给 facade 函数。

use bevy::prelude::*;

use crate::core::domains::tactical::components::{GridPos, MovementPoints};
use crate::core::domains::tactical::error::TacticalError;
use crate::core::domains::tactical::events::ComputeMoveRequest;
use crate::core::domains::tactical::integration::movement::{MP, MovementCapabilityParam};
use crate::core::domains::tactical::resources::GridMap;
use crate::core::domains::tactical::rules;

/// 响应 `ComputeMoveRequest` 事件，通过 Capabilities 管线验证并执行移动。
///
/// 验证路径：
/// 1. Tag 管线 — 通过 facade 检查实体是否持有 MovementType Tag
/// 2. Attribute 管线 — 通过 facade 读取移动点数
/// 3. Modifier 管线 — 通过 facade 收集移动成本 Modifier
/// 4. 执行移动 — 消耗 MP、更新 GridPos、发出 UnitMoved 事件
pub(crate) fn on_compute_move(
    trigger: On<ComputeMoveRequest>,
    mut commands: Commands,
    mut tac_query: Query<(&mut MovementPoints, &mut GridPos)>,
    mov: MovementCapabilityParam,
    grid_map: Res<GridMap>,
) {
    let entity = trigger.event().entity;
    let path = &trigger.event().path;
    let emit_event = trigger.event().emit_moved_event;

    if path.len() < 2 {
        warn!(
            "ComputeMoveRequest path too short: {} positions",
            path.len()
        );
        commands.trigger(TacticalError::InvalidGridPosition);
        return;
    }

    let Ok((mut mp, mut pos)) = tac_query.get_mut(entity) else {
        warn!(
            "ComputeMoveRequest entity {} missing tactical components",
            entity
        );
        commands.trigger(TacticalError::InvalidGridPosition);
        return;
    };
    let mov_type = mp.movement_type;

    // ── 步骤 1+2: 通过 MovementCapabilityParam 构建移动能力视图 ──
    let view = match mov.build_view(entity, mov_type) {
        Ok(v) => v,
        Err(_) => {
            warn!("Entity {} missing movement capabilities", entity);
            commands.trigger(TacticalError::InvalidGridPosition);
            return;
        }
    };

    if !view.can_move {
        warn!("Entity {} has no movement tag for {:?}", entity, mov_type);
        commands.trigger(TacticalError::InvalidGridPosition);
        return;
    }
    info!(
        "[Capabilities Integration] ✅ Movement capability view: can_move={}, effective={}, max={}",
        view.can_move, view.effective_points, view.max_points
    );

    // ── 步骤 3: 计算路径成本 ──
    let target = path[path.len() - 1];

    if !grid_map.in_bounds(target) {
        commands.trigger(TacticalError::OutOfBounds);
        return;
    }

    let target_tile = match grid_map.get_tile(target) {
        Some(tile) => tile,
        None => {
            commands.trigger(TacticalError::InvalidGridPosition);
            return;
        }
    };
    if !target_tile.is_passable() {
        commands.trigger(TacticalError::TileNotPassable);
        return;
    }

    let mut total_cost = 0.0f32;
    for window in path.windows(2) {
        let from = window[0];
        let to = window[1];
        let tile = match grid_map.get_tile(to) {
            Some(t) => t,
            None => {
                commands.trigger(TacticalError::InvalidGridPosition);
                return;
            }
        };
        // TODO[P2][Integration]: terrain_def_id() 返回 u16 是 Tactical 域独立编号,
        //   与 D-2 Terrain 的 TerrainType 枚举不对齐。待后续统一时替换为 TerrainType
        //   或通过 Registry 桶查询实际地形定义。
        let base = rules::movement::movement_cost(tile.terrain_def_id(), mov_type, from, to);
        total_cost += base;
    }
    // 应用 modifier 影响
    total_cost += view.modifier_summary.total_effect.0;

    // ── 步骤 4: 执行移动 ──
    if total_cost > mp.current {
        commands.trigger(TacticalError::InsufficientMovementPoints {
            required: total_cost,
            available: mp.current,
        });
        return;
    }

    let old_pos = *pos;
    mp.consume(total_cost);
    *pos = target;

    info!(
        "[Capabilities Integration] ✅ Move executed: {:?} → {:?}, cost={}, remaining_mp={}",
        old_pos, target, total_cost, mp.current
    );

    // ── 步骤 5: 发出事件 ──
    if emit_event {
        commands.trigger(crate::core::domains::tactical::events::UnitMoved {
            entity,
            from: old_pos,
            to: target,
            remaining_mp: mp.current,
        });
    }
}

/// 验证移动请求并执行（纯函数路径，不走 Capabilities 管线）。
pub fn validate_and_execute_move(
    _entity: Entity,
    target: GridPos,
    grid: &GridMap,
    mp: &mut MovementPoints,
    pos: &mut GridPos,
) -> Result<MoveResult, TacticalError> {
    if !grid.in_bounds(target) {
        return Err(TacticalError::OutOfBounds);
    }

    let tile = grid
        .get_tile(target)
        .ok_or(TacticalError::InvalidGridPosition)?;
    if !tile.is_passable() {
        return Err(TacticalError::TileNotPassable);
    }

    let distance = pos.manhattan_distance(target) as f32;
    let cost = distance;

    if cost > mp.current {
        return Err(TacticalError::InsufficientMovementPoints {
            required: cost,
            available: mp.current,
        });
    }

    let old_pos = *pos;
    mp.consume(cost);
    *pos = target;

    Ok(MoveResult {
        old_pos,
        new_pos: target,
        cost: MP(cost),
        remaining_mp: MP(mp.current),
    })
}

/// 移动执行结果。
#[derive(Debug, Clone)]
pub struct MoveResult {
    pub old_pos: GridPos,
    pub new_pos: GridPos,
    pub cost: MP,
    pub remaining_mp: MP,
}
