// 技能领域模块：数据驱动的技能定义与注册表

mod defaults;
mod types;

pub use types::*;

use crate::core::registry_loader::RegistryLoader;
use bevy::prelude::*;
use std::collections::HashMap;

// ── 技能注册表 ──

/// 技能注册表资源
#[derive(Resource, Default)]
pub struct SkillRegistry {
    pub skills: HashMap<String, SkillData>,
}

impl SkillRegistry {
    pub fn get(&self, id: &str) -> Option<&SkillData> {
        self.skills.get(id)
    }

    /// 注册一个技能
    pub fn register(&mut self, skill: SkillData) {
        self.skills.insert(skill.id.clone(), skill);
    }

    /// 注册内置默认技能（委托给 defaults 模块）
    fn register_defaults(&mut self) {
        defaults::register_defaults(self);
    }
}

impl RegistryLoader for SkillRegistry {
    type Item = SkillDef;

    fn register_item(&mut self, item: SkillDef) {
        let id = item.id.clone();
        self.register(item.into());
        bevy::log::info!(target: "skill", id = %id, "技能已加载");
    }

    fn register_defaults(&mut self) {
        // 委托给 SkillRegistry 自身的 register_defaults（已保证幂等）
        SkillRegistry::register_defaults(self);
    }

    fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    fn registry_name() -> &'static str {
        "技能"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::attribute::AttributeKind;
    use crate::core::effect::EffectDef;
    use crate::core::tag::{GameplayTag, GameplayTags, TagName};
    use ron::de::from_bytes;

    // ── SkillTargeting ──

    #[test]
    fn 目标类型_需要选择目标() {
        assert!(SkillTargeting::SingleEnemy.requires_target_selection());
        assert!(SkillTargeting::SingleAlly.requires_target_selection());
        assert!(!SkillTargeting::SelfOnly.requires_target_selection());
        assert!(!SkillTargeting::NoTarget.requires_target_selection());
    }

    // ── SkillData::can_use ──

    fn make_attrs(hp: f32, max_hp: f32, mp: f32) -> crate::core::attribute::Attributes {
        let mut attrs = crate::core::attribute::Attributes::default();
        // 通过核心属性 Vitality 推导 MaxHp，通过 Intelligence 推导 MaxMp
        // MaxHp = 5 + Vitality * 5, 所以 Vitality = (max_hp - 5) / 5
        // MaxMp = Intelligence * 5, 所以 Intelligence = mp / 5
        let vit = if max_hp > 5.0 {
            (max_hp - 5.0) / 5.0
        } else {
            0.0
        };
        let int = if mp > 0.0 { mp / 5.0 } else { 0.0 };
        attrs.set_base(AttributeKind::Vitality, vit);
        attrs.set_base(AttributeKind::Intelligence, int);
        attrs.fill_vital_resources();
        // 覆盖当前 HP 为指定值
        attrs.set_base(AttributeKind::Hp, hp);
        attrs
    }

    #[test]
    fn 条件_冷却中不可使用() {
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
            cooldown: 3,
            priority: 0,
        };
        let attrs = make_attrs(20.0, 20.0, 10.0);
        let tags = GameplayTags::default();
        let result = skill.can_use(&attrs, &tags, None, 2);
        assert_eq!(result, Err(SkillUseError::OnCooldown { remaining: 2 }));
    }

    #[test]
    fn 条件_mp不足不可使用() {
        let skill = SkillData {
            id: "fireball".into(),
            name: "火球".into(),
            description: String::new(),
            cost_mp: 10,
            range: 3,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![],
            tags: vec![],
            conditions: vec![SkillCondition::MpCost(10)],
            cooldown: 0,
            priority: 0,
        };
        let attrs = make_attrs(20.0, 20.0, 5.0);
        let tags = GameplayTags::default();
        let result = skill.can_use(&attrs, &tags, None, 0);
        assert_eq!(
            result,
            Err(SkillUseError::InsufficientMp {
                required: 10,
                current: 5
            })
        );
    }

    #[test]
    fn 条件_缺少标签不可使用() {
        let skill = SkillData {
            id: "fireball".into(),
            name: "火球".into(),
            description: String::new(),
            cost_mp: 0,
            range: 3,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![],
            tags: vec![],
            conditions: vec![SkillCondition::RequireTag(GameplayTag::MAGE)],
            cooldown: 0,
            priority: 0,
        };
        let attrs = make_attrs(20.0, 20.0, 10.0);
        let tags = GameplayTags::default();
        let result = skill.can_use(&attrs, &tags, None, 0);
        assert_eq!(
            result,
            Err(SkillUseError::MissingTag {
                tag: GameplayTag::MAGE
            })
        );
    }

    #[test]
    fn 条件_满足条件可使用() {
        let skill = SkillData {
            id: "fireball".into(),
            name: "火球".into(),
            description: String::new(),
            cost_mp: 5,
            range: 3,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![],
            tags: vec![],
            conditions: vec![SkillCondition::MpCost(5)],
            cooldown: 0,
            priority: 0,
        };
        let attrs = make_attrs(20.0, 20.0, 10.0);
        let tags = GameplayTags::default();
        assert!(skill.can_use(&attrs, &tags, None, 0).is_ok());
    }

    #[test]
    fn 条件_hp低于阈值() {
        let skill = SkillData {
            id: "desperate".into(),
            name: "背水一战".into(),
            description: String::new(),
            cost_mp: 0,
            range: 1,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![],
            tags: vec![],
            conditions: vec![SkillCondition::HpBelow(0.5)],
            cooldown: 0,
            priority: 0,
        };
        let attrs_low = make_attrs(5.0, 20.0, 10.0);
        let attrs_ok = make_attrs(15.0, 20.0, 10.0);
        let tags = GameplayTags::default();
        assert!(skill.can_use(&attrs_low, &tags, None, 0).is_ok());
        assert_eq!(
            skill.can_use(&attrs_ok, &tags, None, 0),
            Err(SkillUseError::HpNotBelow { threshold: 0.5 })
        );
    }

    #[test]
    fn skill_def_转换为_skill_data() {
        let def = SkillDef {
            version: 0,
            id: "test".into(),
            name: "测试".into(),
            description: "测试技能".into(),
            cost_mp: 5,
            range: 3,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![EffectDef::Damage {
                multiplier: 1.5,
                ignore_def_percent: 0.0,
            }],
            tags: vec![TagName::Fire, TagName::SkillActive],
            conditions: vec![SkillConditionDef::RequireTag(TagName::Mage)],
            cooldown: 2,
            priority: 10,
        };
        let data: SkillData = def.into();
        assert_eq!(data.id, "test");
        assert_eq!(
            data.tags,
            vec![GameplayTag::FIRE, GameplayTag::SKILL_ACTIVE]
        );
        assert_eq!(data.conditions.len(), 1);
        assert!(matches!(
            data.conditions[0],
            SkillCondition::RequireTag(GameplayTag::MAGE)
        ));
    }

    #[test]
    fn ron_反序列化_技能定义() {
        let ron_str = r#"
            (
                id: "test_skill",
                name: "测试技能",
                description: "一个测试技能",
                cost_mp: 10,
                range: 3,
                targeting: SingleEnemy,
                effects: [
                    Damage(multiplier: 2.0, ignore_def_percent: 50.0),
                    ApplyBuff(buff_id: "burn", duration: 2),
                ],
                tags: [FIRE, SKILL_ACTIVE],
                conditions: [
                    MpCost(10),
                    RequireTag(MAGE),
                ],
                cooldown: 3,
                priority: 20,
            )
        "#;
        let def: SkillDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "test_skill");
        assert_eq!(def.tags, vec![TagName::Fire, TagName::SkillActive]);
        assert_eq!(def.effects.len(), 2);
        assert_eq!(def.conditions.len(), 2);
    }

    #[test]
    fn 条件_hp高于阈值() {
        let skill = SkillData {
            id: "heal_self".into(),
            name: "自愈".into(),
            description: String::new(),
            cost_mp: 0,
            range: 1,
            targeting: SkillTargeting::SelfOnly,
            effects: vec![],
            tags: vec![],
            conditions: vec![SkillCondition::HpAbove(0.5)],
            cooldown: 0,
            priority: 0,
        };
        let attrs_high = make_attrs(15.0, 20.0, 10.0);
        let attrs_low = make_attrs(5.0, 20.0, 10.0);
        let tags = GameplayTags::default();
        assert!(skill.can_use(&attrs_high, &tags, None, 0).is_ok());
        assert_eq!(
            skill.can_use(&attrs_low, &tags, None, 0),
            Err(SkillUseError::HpNotAbove { threshold: 0.5 })
        );
    }

    #[test]
    fn 条件_目标缺少标签() {
        let skill = SkillData {
            id: "purify".into(),
            name: "净化".into(),
            description: String::new(),
            cost_mp: 0,
            range: 2,
            targeting: SkillTargeting::SingleAlly,
            effects: vec![],
            tags: vec![],
            conditions: vec![SkillCondition::TargetRequireTag(GameplayTag::BUFF)],
            cooldown: 0,
            priority: 0,
        };
        let attrs = make_attrs(20.0, 20.0, 10.0);
        let source_tags = GameplayTags::default();
        let mut target_tags_with = GameplayTags::default();
        target_tags_with.add(GameplayTag::BUFF);
        let target_tags_without = GameplayTags::default();
        assert!(
            skill
                .can_use(&attrs, &source_tags, Some(&target_tags_with), 0)
                .is_ok()
        );
        assert_eq!(
            skill.can_use(&attrs, &source_tags, Some(&target_tags_without), 0),
            Err(SkillUseError::TargetMissingTag {
                tag: GameplayTag::BUFF
            })
        );
    }

    #[test]
    fn 目标类型_label() {
        assert_eq!(SkillTargeting::SingleEnemy.label(), "单体敌方");
        assert_eq!(SkillTargeting::SingleAlly.label(), "单体友方");
        assert_eq!(SkillTargeting::SelfOnly.label(), "自身");
        assert_eq!(SkillTargeting::AoeEnemies.label(), "范围敌方");
        assert_eq!(SkillTargeting::AoeAllies.label(), "范围友方");
        assert_eq!(SkillTargeting::NoTarget.label(), "无目标");
    }

    #[test]
    fn 条件_空条件列表可使用() {
        let skill = SkillData {
            id: "simple".into(),
            name: "简单".into(),
            description: String::new(),
            cost_mp: 0,
            range: 1,
            targeting: SkillTargeting::SelfOnly,
            effects: vec![],
            tags: vec![],
            conditions: vec![],
            cooldown: 0,
            priority: 0,
        };
        let attrs = make_attrs(10.0, 20.0, 5.0);
        let tags = GameplayTags::default();
        assert!(skill.can_use(&attrs, &tags, None, 0).is_ok());
    }

    #[test]
    fn 条件_多个条件全满足() {
        let skill = SkillData {
            id: "elite".into(),
            name: "精英技能".into(),
            description: String::new(),
            cost_mp: 5,
            range: 1,
            targeting: SkillTargeting::SingleEnemy,
            effects: vec![],
            tags: vec![],
            conditions: vec![
                SkillCondition::MpCost(5),
                SkillCondition::RequireTag(GameplayTag::MAGE),
            ],
            cooldown: 0,
            priority: 0,
        };
        let mut attrs = make_attrs(20.0, 20.0, 10.0);
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::MAGE);
        assert!(skill.can_use(&attrs, &tags, None, 0).is_ok());

        attrs.set_base(AttributeKind::Mp, 2.0);
        assert_eq!(
            skill.can_use(&attrs, &tags, None, 0),
            Err(SkillUseError::InsufficientMp {
                required: 5,
                current: 2
            })
        );
    }
}
