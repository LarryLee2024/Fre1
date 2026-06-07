// 战斗模块：伤害计算

use crate::map::Terrain;
use crate::unit::Unit;
use bevy::prelude::*;

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
