use crate::skill::{BASIC_ATTACK_ID, SkillCooldowns};

use super::behavior::SkillStrategy;

/// 根据技能策略选择技能
pub(crate) fn select_skill<'a>(
    skill_ids: &'a [String],
    cooldowns: &SkillCooldowns,
    strategy: SkillStrategy,
    priority: &'a [String],
) -> &'a str {
    match strategy {
        SkillStrategy::PreferSpecial => {
            // 优先特殊技能（跳过冷却中的），否则基础攻击
            skill_ids
                .iter()
                .find(|id| *id != BASIC_ATTACK_ID && cooldowns.get(id) == 0)
                .map(|s| s.as_str())
                .unwrap_or(BASIC_ATTACK_ID)
        }
        SkillStrategy::PreferBasic => {
            // 优先基础攻击
            if cooldowns.get(BASIC_ATTACK_ID) == 0 {
                BASIC_ATTACK_ID
            } else {
                skill_ids
                    .iter()
                    .find(|id| cooldowns.get(id) == 0)
                    .map(|s| s.as_str())
                    .unwrap_or(BASIC_ATTACK_ID)
            }
        }
        SkillStrategy::ByPriority => {
            // 按优先级列表选择
            if !priority.is_empty() {
                for preferred in priority {
                    if skill_ids.iter().any(|id| id == preferred) && cooldowns.get(preferred) == 0 {
                        return preferred.as_str();
                    }
                }
            }
            // 回退：优先特殊技能
            skill_ids
                .iter()
                .find(|id| *id != BASIC_ATTACK_ID && cooldowns.get(id) == 0)
                .map(|s| s.as_str())
                .unwrap_or(BASIC_ATTACK_ID)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 技能策略_优先特殊_跳过冷却() {
        let skill_ids = vec![BASIC_ATTACK_ID.into(), "charge".into(), "fireball".into()];
        let mut cooldowns = SkillCooldowns::default();
        cooldowns.set("charge", 2); // charge 在冷却
        let result = select_skill(&skill_ids, &cooldowns, SkillStrategy::PreferSpecial, &[]);
        assert_eq!(result, "fireball");
    }

    #[test]
    fn 技能策略_优先特殊_全冷却回退基础攻击() {
        let skill_ids = vec![BASIC_ATTACK_ID.into(), "charge".into()];
        let mut cooldowns = SkillCooldowns::default();
        cooldowns.set("charge", 3);
        let result = select_skill(&skill_ids, &cooldowns, SkillStrategy::PreferSpecial, &[]);
        assert_eq!(result, BASIC_ATTACK_ID);
    }

    #[test]
    fn 技能策略_优先基础() {
        let skill_ids = vec![BASIC_ATTACK_ID.into(), "fireball".into()];
        let cooldowns = SkillCooldowns::default();
        let result = select_skill(&skill_ids, &cooldowns, SkillStrategy::PreferBasic, &[]);
        assert_eq!(result, BASIC_ATTACK_ID);
    }

    #[test]
    fn 技能策略_按优先级() {
        let skill_ids = vec![BASIC_ATTACK_ID.into(), "heal".into(), "fireball".into()];
        let cooldowns = SkillCooldowns::default();
        let priority = vec!["fireball".into(), "heal".into()];
        let result = select_skill(&skill_ids, &cooldowns, SkillStrategy::ByPriority, &priority);
        assert_eq!(result, "fireball");
    }

    #[test]
    fn 技能策略_按优先级_首选冷却时选次选() {
        let skill_ids = vec![BASIC_ATTACK_ID.into(), "heal".into(), "fireball".into()];
        let mut cooldowns = SkillCooldowns::default();
        cooldowns.set("fireball", 2);
        let priority = vec!["fireball".into(), "heal".into()];
        let result = select_skill(&skill_ids, &cooldowns, SkillStrategy::ByPriority, &priority);
        assert_eq!(result, "heal");
    }

    #[test]
    fn 技能策略_按优先级_空回退特殊() {
        let skill_ids = vec![BASIC_ATTACK_ID.into(), "charge".into()];
        let cooldowns = SkillCooldowns::default();
        let result = select_skill(&skill_ids, &cooldowns, SkillStrategy::ByPriority, &[]);
        assert_eq!(result, "charge");
    }
}
