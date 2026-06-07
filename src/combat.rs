// 战斗模块：伤害计算、技能系统

use crate::map::Terrain;
use crate::unit::{Skill, Unit};
use bevy::prelude::*;

/// 计算曼哈顿距离
pub fn manhattan_distance(a: IVec2, b: IVec2) -> u32 {
    (a.x - b.x).unsigned_abs() as u32 + (a.y - b.y).unsigned_abs() as u32
}

/// 计算攻击伤害（含技能加成）
pub fn calculate_damage(attacker: &Unit, defender: &Unit, terrain: Terrain) -> i32 {
    let skill_multiplier = match attacker.skill {
        Skill::Charge => 1.5,   // 冲锋：1.5倍伤害
        Skill::Pierce => 1.3,   // 穿透箭：1.3倍伤害，无视部分防御
        Skill::Fireball => 1.8, // 火球：1.8倍伤害
        Skill::None => 1.0,
    };

    let def_reduction = match attacker.skill {
        Skill::Pierce => (defender.def as f32 * 0.5) as i32, // 穿透箭无视50%防御
        _ => defender.def,
    };

    let base_damage = attacker.atk - def_reduction;
    let terrain_bonus = terrain.defense_bonus();
    ((base_damage - terrain_bonus) as f32 * skill_multiplier).max(1.0) as i32
}

/// 获取技能名称
pub fn skill_name(skill: &Skill) -> &'static str {
    match skill {
        Skill::None => "普通攻击",
        Skill::Charge => "冲锋",
        Skill::Pierce => "穿透箭",
        Skill::Fireball => "火球",
    }
}

/// 获取技能范围（覆盖单位默认攻击范围）
pub fn skill_range(skill: &Skill, base_range: u32) -> u32 {
    match skill {
        Skill::Charge => 1,                       // 冲锋：近战
        Skill::Pierce => (base_range + 1).max(4), // 穿透箭：远程+1
        Skill::Fireball => 3,                     // 火球：中程
        Skill::None => base_range,
    }
}
