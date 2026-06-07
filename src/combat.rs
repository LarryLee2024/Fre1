// 战斗模块：伤害计算、技能系统

use crate::map::Terrain;
use crate::unit::{Skill, Unit};
use bevy::prelude::*;

/// 计算曼哈顿距离
pub fn manhattan_distance(a: IVec2, b: IVec2) -> u32 {
    (a.x - b.x).unsigned_abs() as u32 + (a.y - b.y).unsigned_abs() as u32
}

/// 计算攻击伤害（含技能加成与状态效果修正）
pub fn calculate_damage(
    attacker: &Unit,
    attacker_atk_mod: i32,
    defender: &Unit,
    defender_def_mod: i32,
    terrain: Terrain,
) -> i32 {
    let skill_multiplier = match attacker.skill {
        Skill::Charge => 1.5,   // 冲锋：1.5倍伤害
        Skill::Pierce => 1.3,   // 穿透箭：1.3倍伤害，无视部分防御
        Skill::Fireball => 1.8, // 火球：1.8倍伤害
        Skill::None => 1.0,
    };

    let effective_atk = attacker.atk + attacker_atk_mod;
    let effective_def = defender.def + defender_def_mod;

    let def_reduction = match attacker.skill {
        Skill::Pierce => (defender.def as f32 * 0.5) as i32, // 穿透箭无视50%基础防御，且无视状态修正
        _ => effective_def,
    };

    let base_damage = effective_atk - def_reduction;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unit::Faction;

    /// 构建测试用 Unit
    fn make_unit(atk: i32, def: i32, skill: Skill) -> Unit {
        Unit {
            faction: Faction::Player,
            mov: 3,
            hp: 30,
            max_hp: 30,
            atk,
            def,
            attack_range: 1,
            acted: false,
            skill,
        }
    }

    // ---- manhattan_distance ----

    #[test]
    fn 曼哈顿距离_相邻格子() {
        assert_eq!(manhattan_distance(IVec2::new(0, 0), IVec2::new(1, 0)), 1);
        assert_eq!(manhattan_distance(IVec2::new(0, 0), IVec2::new(0, 1)), 1);
    }

    #[test]
    fn 曼哈顿距离_对角线() {
        assert_eq!(manhattan_distance(IVec2::new(0, 0), IVec2::new(3, 4)), 7);
    }

    #[test]
    fn 曼哈顿距离_同一位置() {
        assert_eq!(manhattan_distance(IVec2::new(2, 3), IVec2::new(2, 3)), 0);
    }

    #[test]
    fn 曼哈顿距离_负坐标() {
        assert_eq!(manhattan_distance(IVec2::new(-1, -2), IVec2::new(1, 2)), 6);
    }

    // ---- calculate_damage ----

    #[test]
    fn 普通攻击_基础伤害() {
        let attacker = make_unit(10, 2, Skill::None);
        let defender = make_unit(5, 3, Skill::None);
        // 10 - 3 = 7, * 1.0 = 7
        assert_eq!(calculate_damage(&attacker, 0, &defender, 0, Terrain::Plain), 7);
    }

    #[test]
    fn 普通攻击_森林地形减伤() {
        let attacker = make_unit(10, 2, Skill::None);
        let defender = make_unit(5, 3, Skill::None);
        // 10 - 3 - 2(森林) = 5, * 1.0 = 5
        assert_eq!(calculate_damage(&attacker, 0, &defender, 0, Terrain::Forest), 5);
    }

    #[test]
    fn 最低伤害为1() {
        let attacker = make_unit(1, 2, Skill::None);
        let defender = make_unit(5, 10, Skill::None);
        // 1 - 10 = -9, max(1.0) = 1
        assert_eq!(calculate_damage(&attacker, 0, &defender, 0, Terrain::Plain), 1);
    }

    #[test]
    fn 冲锋_1点5倍伤害() {
        let attacker = make_unit(10, 2, Skill::Charge);
        let defender = make_unit(5, 3, Skill::None);
        // 10 - 3 = 7, * 1.5 = 10.5 → 10
        assert_eq!(calculate_damage(&attacker, 0, &defender, 0, Terrain::Plain), 10);
    }

    #[test]
    fn 穿透箭_无视50防御() {
        let attacker = make_unit(10, 2, Skill::Pierce);
        let defender = make_unit(5, 10, Skill::None);
        // 10 - 5(50% of 10) = 5, * 1.3 = 6.5 → 6
        assert_eq!(calculate_damage(&attacker, 0, &defender, 0, Terrain::Plain), 6);
    }

    #[test]
    fn 火球_1点8倍伤害() {
        let attacker = make_unit(10, 2, Skill::Fireball);
        let defender = make_unit(5, 3, Skill::None);
        // 10 - 3 = 7, * 1.8 = 12.6 → 12
        assert_eq!(calculate_damage(&attacker, 0, &defender, 0, Terrain::Plain), 12);
    }

    // ---- 状态效果修正 ----

    #[test]
    fn 攻击者_加攻提升伤害() {
        let attacker = make_unit(10, 2, Skill::None);
        let defender = make_unit(5, 3, Skill::None);
        // (10 + 5) - 3 = 12
        assert_eq!(calculate_damage(&attacker, 5, &defender, 0, Terrain::Plain), 12);
    }

    #[test]
    fn 攻击者_减攻降低伤害() {
        let attacker = make_unit(10, 2, Skill::None);
        let defender = make_unit(5, 3, Skill::None);
        // (10 - 8) - 3 = -1 → 1
        assert_eq!(calculate_damage(&attacker, -8, &defender, 0, Terrain::Plain), 1);
    }

    #[test]
    fn 防御者_加防减伤() {
        let attacker = make_unit(10, 2, Skill::None);
        let defender = make_unit(5, 3, Skill::None);
        // 10 - (3 + 5) = 2
        assert_eq!(calculate_damage(&attacker, 0, &defender, 5, Terrain::Plain), 2);
    }

    #[test]
    fn 防御者_减防增伤() {
        let attacker = make_unit(10, 2, Skill::None);
        let defender = make_unit(5, 3, Skill::None);
        // 10 - (3 - 5) = 12
        assert_eq!(calculate_damage(&attacker, 0, &defender, -5, Terrain::Plain), 12);
    }

    #[test]
    fn 穿透箭_无视修正后的防御() {
        let attacker = make_unit(10, 2, Skill::Pierce);
        let defender = make_unit(5, 10, Skill::None);
        // 有效防 = 10 + 0 = 10, 穿透后 10 * 0.5 = 5
        // 10 - 5 = 5, * 1.3 = 6.5 → 6
        assert_eq!(calculate_damage(&attacker, 0, &defender, 0, Terrain::Plain), 6);
        // 加防后被穿透
        assert_eq!(calculate_damage(&attacker, 0, &defender, 10, Terrain::Plain), 6);
    }

    // ---- skill_range ----

    #[test]
    fn 技能范围_普通攻击用基础范围() {
        assert_eq!(skill_range(&Skill::None, 1), 1);
        assert_eq!(skill_range(&Skill::None, 3), 3);
    }

    #[test]
    fn 技能范围_冲锋固定为1() {
        assert_eq!(skill_range(&Skill::Charge, 3), 1);
    }

    #[test]
    fn 技能范围_穿透箭至少4() {
        assert_eq!(skill_range(&Skill::Pierce, 1), 4);
        assert_eq!(skill_range(&Skill::Pierce, 3), 4);
        assert_eq!(skill_range(&Skill::Pierce, 5), 6);
    }

    #[test]
    fn 技能范围_火球固定为3() {
        assert_eq!(skill_range(&Skill::Fireball, 1), 3);
        assert_eq!(skill_range(&Skill::Fireball, 5), 3);
    }

    // ---- skill_name ----

    #[test]
    fn 技能名称映射() {
        assert_eq!(skill_name(&Skill::None), "普通攻击");
        assert_eq!(skill_name(&Skill::Charge), "冲锋");
        assert_eq!(skill_name(&Skill::Pierce), "穿透箭");
        assert_eq!(skill_name(&Skill::Fireball), "火球");
    }
}
