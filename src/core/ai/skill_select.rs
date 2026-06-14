use crate::core::ability::SkillCooldowns;

use super::strategy::SkillSelector;

/// 根据技能策略选择技能
/// 通过 trait 对象分发，替代原来的 enum+match 模式
pub(crate) fn select_skill<'a>(
    skill_ids: &'a [String],
    cooldowns: &SkillCooldowns,
    selector: &dyn SkillSelector,
    priority: &'a [String],
) -> &'a str {
    selector.select(skill_ids, cooldowns, priority)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::ability::BASIC_ATTACK_ID;
    use crate::core::ai::strategy::{
        AiStrategyRegistry, ByPrioritySkill, PreferBasicSkill, PreferSpecialSkill,
    };

    #[test]
    fn 技能策略_优先特殊_跳过冷却() {
        let skill_ids = vec![BASIC_ATTACK_ID.into(), "charge".into(), "fireball".into()];
        let mut cooldowns = SkillCooldowns::default();
        cooldowns.set("charge", 2); // charge 在冷却
        let selector = PreferSpecialSkill;
        let result = select_skill(&skill_ids, &cooldowns, &selector, &[]);
        assert_eq!(result, "fireball");
    }

    #[test]
    fn 技能策略_优先特殊_全冷却回退基础攻击() {
        let skill_ids = vec![BASIC_ATTACK_ID.into(), "charge".into()];
        let mut cooldowns = SkillCooldowns::default();
        cooldowns.set("charge", 3);
        let selector = PreferSpecialSkill;
        let result = select_skill(&skill_ids, &cooldowns, &selector, &[]);
        assert_eq!(result, BASIC_ATTACK_ID);
    }

    #[test]
    fn 技能策略_优先基础() {
        let skill_ids = vec![BASIC_ATTACK_ID.into(), "fireball".into()];
        let cooldowns = SkillCooldowns::default();
        let selector = PreferBasicSkill;
        let result = select_skill(&skill_ids, &cooldowns, &selector, &[]);
        assert_eq!(result, BASIC_ATTACK_ID);
    }

    #[test]
    fn 技能策略_按优先级() {
        let skill_ids = vec![BASIC_ATTACK_ID.into(), "heal".into(), "fireball".into()];
        let cooldowns = SkillCooldowns::default();
        let priority = vec!["fireball".into(), "heal".into()];
        let selector = ByPrioritySkill;
        let result = select_skill(&skill_ids, &cooldowns, &selector, &priority);
        assert_eq!(result, "fireball");
    }

    #[test]
    fn 技能策略_按优先级_首选冷却时选次选() {
        let skill_ids = vec![BASIC_ATTACK_ID.into(), "heal".into(), "fireball".into()];
        let mut cooldowns = SkillCooldowns::default();
        cooldowns.set("fireball", 2);
        let priority = vec!["fireball".into(), "heal".into()];
        let selector = ByPrioritySkill;
        let result = select_skill(&skill_ids, &cooldowns, &selector, &priority);
        assert_eq!(result, "heal");
    }

    #[test]
    fn 技能策略_按优先级_空回退特殊() {
        let skill_ids = vec![BASIC_ATTACK_ID.into(), "charge".into()];
        let cooldowns = SkillCooldowns::default();
        let selector = ByPrioritySkill;
        let result = select_skill(&skill_ids, &cooldowns, &selector, &[]);
        assert_eq!(result, "charge");
    }

    #[test]
    fn 技能策略_通过注册表分发() {
        let registry = AiStrategyRegistry::default();
        let skill_ids = vec![BASIC_ATTACK_ID.into(), "fireball".into()];
        let cooldowns = SkillCooldowns::default();

        // 通过注册表查找 PreferBasic 策略
        let selector = registry.skill_selector("PreferBasic");
        let result = select_skill(&skill_ids, &cooldowns, selector, &[]);
        assert_eq!(result, BASIC_ATTACK_ID);
    }
}
