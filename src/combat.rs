// 战斗模块：伤害计算、攻击范围

use bevy::prelude::*;
use crate::unit::Unit;
use crate::map::Terrain;

/// 计算曼哈顿距离
pub fn manhattan_distance(a: IVec2, b: IVec2) -> u32 {
    (a.x - b.x).unsigned_abs() as u32 + (a.y - b.y).unsigned_abs() as u32
}

/// 计算攻击伤害
pub fn calculate_damage(attacker: &Unit, defender: &Unit, terrain: Terrain) -> i32 {
    let base_damage = attacker.atk - defender.def;
    let terrain_bonus = terrain.defense_bonus();
    (base_damage - terrain_bonus).max(1)
}

/// 获取攻击范围内的目标
pub fn get_targets_in_range(
    attacker_pos: IVec2,
    attacker: &Unit,
    units: &[(Entity, IVec2, &Unit)],
) -> Vec<Entity> {
    units
        .iter()
        .filter(|(_, pos, unit)| {
            unit.faction != attacker.faction
                && manhattan_distance(attacker_pos, *pos) <= attacker.attack_range
        })
        .map(|(entity, _, _)| *entity)
        .collect()
}
