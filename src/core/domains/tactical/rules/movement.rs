//! 移动消耗计算 — 纯函数
//!
//! 根据地形倍率和基础消耗计算单位移动一个格子的实际消耗。
//!
//! 来源：
//! - ADR-022 §3：地形类型与移动类型分类
//! - docs/02-domain/domains/tactical_domain.md §5：移动力消耗规则
//! - 地形倍率值（0.5-999.0）为设计初版配置，后续应从 TerrainDef 配置加载

use super::super::components::{GridPos, MovementType};

/// 地形移动倍率表。
///
/// 每个条目对应一个 TerrainDefId（u16）到倍率的映射。
/// 从配置文件加载，运行时通过 Terrain 域提供。
pub fn movement_cost(
    terrain_def_id: u16,
    movement_type: MovementType,
    _from: GridPos,
    _to: GridPos,
) -> f32 {
    match movement_type {
        MovementType::Walk => match terrain_def_id {
            0 => 1.0,   // 平地
            1 => 0.5,   // 道路
            2 => 2.0,   // 森林
            3 => 3.0,   // 沼泽
            4 => 999.0, // 深水（步行不可通行）
            5 => 2.5,   // 碎石
            6 => 1.5,   // 浅水
            7 => 4.0,   // 陡坡
            _ => 1.0,
        },
        MovementType::Fly => 1.0, // 飞行无视地形
        MovementType::Swim => match terrain_def_id {
            3 => 1.0,  // 沼泽
            4 => 1.0,  // 深水
            6 => 0.75, // 浅水
            _ => 2.0,  // 非水域效率低
        },
        MovementType::Climb => match terrain_def_id {
            5 => 1.0, // 碎石
            7 => 1.0, // 陡坡
            _ => 2.0,
        },
        MovementType::Teleport => 0.0, // 瞬移无视消耗
    }
}

/// 计算路径总消耗。
pub fn path_total_cost(path: &[GridPos], terrain_ids: &[u16], movement_type: MovementType) -> f32 {
    if path.len() < 2 {
        return 0.0;
    }

    path.windows(2)
        .enumerate()
        .map(|(i, w)| {
            let terrain_id = terrain_ids.get(i).copied().unwrap_or(0);
            movement_cost(terrain_id, movement_type, w[0], w[1])
        })
        .sum()
}
