use bevy::prelude::*;
use std::collections::HashMap;

use super::domain::{BASIC_ATTACK_ID, SkillData};

// ── 技能槽组件 ──

/// 单位的技能槽组件
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct SkillSlots {
    pub skill_ids: Vec<String>,
}

impl SkillSlots {
    pub fn new(skill_ids: Vec<String>) -> Self {
        Self { skill_ids }
    }

    /// 获取默认攻击技能 ID
    pub fn default_attack(&self) -> &str {
        self.skill_ids
            .first()
            .map(|s| s.as_str())
            .unwrap_or(BASIC_ATTACK_ID)
    }

    /// 获取特殊技能 ID（第二个技能，如果有）
    pub fn special_skill(&self) -> Option<&str> {
        self.skill_ids.get(1).map(|s| s.as_str())
    }

    /// 获取所有技能 ID（迭代器）
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.skill_ids.iter().map(|s| s.as_str())
    }
}

// ── 运行时冷却追踪 ──

/// 运行时技能冷却追踪组件
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct SkillCooldowns {
    /// skill_id → 剩余冷却回合数
    cooldowns: HashMap<String, u32>,
}

impl SkillCooldowns {
    /// 获取技能当前冷却
    pub fn get(&self, skill_id: &str) -> u32 {
        self.cooldowns.get(skill_id).copied().unwrap_or(0)
    }

    /// 设置技能冷却
    pub fn set(&mut self, skill_id: &str, turns: u32) {
        if turns > 0 {
            self.cooldowns.insert(skill_id.to_string(), turns);
        }
    }

    /// 回合结束：递减所有冷却
    pub fn tick(&mut self) {
        self.cooldowns.retain(|_, cd| {
            *cd = cd.saturating_sub(1);
            *cd > 0
        });
    }

    /// 清除所有冷却
    pub fn clear(&mut self) {
        self.cooldowns.clear();
    }
}

/// 获取技能的有效范围（考虑单位基础攻击范围）
pub fn effective_skill_range(skill_data: &SkillData, base_attack_range: u32) -> u32 {
    if skill_data.range > 0 {
        skill_data.range
    } else {
        base_attack_range
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skill::domain::SkillTargeting;

    // ── SkillSlots ──

    #[test]
    fn 技能槽_默认攻击() {
        let slots = SkillSlots::new(vec![BASIC_ATTACK_ID.into(), "charge".into()]);
        assert_eq!(slots.default_attack(), BASIC_ATTACK_ID);
    }

    #[test]
    fn 技能槽_默认攻击_空列表回退() {
        let slots = SkillSlots::new(vec![]);
        assert_eq!(slots.default_attack(), BASIC_ATTACK_ID);
    }

    #[test]
    fn 技能槽_特殊技能() {
        let slots = SkillSlots::new(vec![BASIC_ATTACK_ID.into(), "charge".into()]);
        assert_eq!(slots.special_skill(), Some("charge"));
    }

    #[test]
    fn 技能槽_特殊技能_只有一个技能() {
        let slots = SkillSlots::new(vec![BASIC_ATTACK_ID.into()]);
        assert_eq!(slots.special_skill(), None);
    }

    #[test]
    fn 技能槽_特殊技能_空列表() {
        let slots = SkillSlots::new(vec![]);
        assert_eq!(slots.special_skill(), None);
    }

    #[test]
    fn 技能槽_迭代器() {
        let slots = SkillSlots::new(vec![BASIC_ATTACK_ID.into(), "charge".into()]);
        let ids: Vec<&str> = slots.iter().collect();
        assert_eq!(ids, vec![BASIC_ATTACK_ID, "charge"]);
    }

    // ── effective_skill_range ──

    #[test]
    fn 技能范围_技能自带范围() {
        let skill = SkillData {
            id: "fireball".into(),
            name: "火球".into(),
            description: String::new(),
            cost_mp: 0,
            range: 3,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![],
            tags: vec![],
            conditions: vec![],
            cooldown: 0,
            priority: 0,
        };
        assert_eq!(effective_skill_range(&skill, 1), 3);
    }

    #[test]
    fn 技能范围_使用单位基础范围() {
        let skill = SkillData {
            id: BASIC_ATTACK_ID.into(),
            name: "普通攻击".into(),
            description: String::new(),
            cost_mp: 0,
            range: 0,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![],
            tags: vec![],
            conditions: vec![],
            cooldown: 0,
            priority: 0,
        };
        assert_eq!(effective_skill_range(&skill, 3), 3);
    }

    // ── SkillCooldowns ──

    #[test]
    fn 冷却_初始为0() {
        let cds = SkillCooldowns::default();
        assert_eq!(cds.get("fireball"), 0);
    }

    #[test]
    fn 冷却_设置和查询() {
        let mut cds = SkillCooldowns::default();
        cds.set("fireball", 3);
        assert_eq!(cds.get("fireball"), 3);
    }

    #[test]
    fn 冷却_tick递减() {
        let mut cds = SkillCooldowns::default();
        cds.set("fireball", 2);
        cds.tick();
        assert_eq!(cds.get("fireball"), 1);
        cds.tick();
        assert_eq!(cds.get("fireball"), 0); // 归零后被移除
    }

    #[test]
    fn 冷却_clear清空() {
        let mut cds = SkillCooldowns::default();
        cds.set("fireball", 3);
        cds.set("pierce", 2);
        cds.clear();
        assert_eq!(cds.get("fireball"), 0);
        assert_eq!(cds.get("pierce"), 0);
    }
}
