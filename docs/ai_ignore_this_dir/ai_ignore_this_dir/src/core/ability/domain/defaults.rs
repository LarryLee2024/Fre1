// 技能默认注册数据
//
// ADR-013: 内置默认技能作为 RON 加载的后备（当 content/skills/ 为空时使用）
// ADR-015: 所有技能必须携带分类标签(SKILL_ACTIVE/SKILL_PASSIVE)和元素/武器标签

use super::SkillRegistry;
use super::types::*;
use crate::core::effect::{DurationDef, EffectDef, StackingDef};
use crate::core::tag::GameplayTag;
use crate::core::targeting::SkillTargeting;

/// 注册内置默认技能（确保基础功能可用）
pub fn register_defaults(registry: &mut SkillRegistry) {
    if !registry.skills.is_empty() {
        return;
    }
    // 普通攻击
    registry.register(SkillData {
        id: BASIC_ATTACK_ID.into(),
        name: "普通攻击".into(),
        description: "基础物理攻击".into(),
        name_key: Some("skill.s_1001.name".into()),
        desc_key: Some("skill.s_1001.desc".into()),
        cost_mp: 0,
        range: 0,
        targeting: SkillTargeting::SingleEnemy,
        effects: vec![EffectDef::Damage {
            multiplier: 1.0,
            ignore_def_percent: 0.0,
        }],
        tags: vec![GameplayTag::WEAPON_SWORD, GameplayTag::SPECIAL_STATE],
        conditions: vec![],
        cooldown: 0,
        priority: 0,
    });

    // 冲锋
    registry.register(SkillData {
        id: "charge".into(),
        name: "冲锋".into(),
        description: "1.5倍伤害，需要近战".into(),
        name_key: Some("skill.s_1002.name".into()),
        desc_key: Some("skill.s_1002.desc".into()),
        cost_mp: 5,
        range: 0,
        targeting: SkillTargeting::SingleEnemy,
        effects: vec![EffectDef::Damage {
            multiplier: 1.5,
            ignore_def_percent: 0.0,
        }],
        tags: vec![GameplayTag::WEAPON_SWORD, GameplayTag::SPECIAL_STATE],
        conditions: vec![SkillCondition::MpCost(5)],
        cooldown: 2,
        priority: 5,
    });

    // 穿刺
    registry.register(SkillData {
        id: "pierce".into(),
        name: "穿刺".into(),
        description: "无视50%防御".into(),
        name_key: Some("skill.s_1003.name".into()),
        desc_key: Some("skill.s_1003.desc".into()),
        cost_mp: 8,
        range: 0,
        targeting: SkillTargeting::SingleEnemy,
        effects: vec![EffectDef::Damage {
            multiplier: 1.2,
            ignore_def_percent: 50.0,
        }],
        tags: vec![GameplayTag::WEAPON_SWORD, GameplayTag::SPECIAL_STATE],
        conditions: vec![SkillCondition::MpCost(8)],
        cooldown: 3,
        priority: 8,
    });

    // 火球
    registry.register(SkillData {
        id: "fireball".into(),
        name: "火球".into(),
        description: "远程火属性攻击".into(),
        name_key: Some("skill.s_1004.name".into()),
        desc_key: Some("skill.s_1004.desc".into()),
        cost_mp: 10,
        range: 3,
        targeting: SkillTargeting::SingleEnemy,
        effects: vec![
            EffectDef::Damage {
                multiplier: 1.5,
                ignore_def_percent: 0.0,
            },
            EffectDef::ApplyModifier {
                modifier_id: "burn".into(),
                duration: DurationDef::TurnLimited(2),
                stacking: StackingDef::Replace,
            },
        ],
        tags: vec![
            GameplayTag::DMG_FIRE,
            GameplayTag::WEAPON_BOW,
            GameplayTag::SPECIAL_STATE,
        ],
        conditions: vec![SkillCondition::MpCost(10)],
        cooldown: 2,
        priority: 10,
    });

    // 治疗
    registry.register(SkillData {
        id: "heal".into(),
        name: "治疗".into(),
        description: "恢复友方生命值".into(),
        name_key: Some("skill.s_1005.name".into()),
        desc_key: Some("skill.s_1005.desc".into()),
        cost_mp: 6,
        range: 2,
        targeting: SkillTargeting::SingleAlly,
        effects: vec![EffectDef::Heal { amount: 8 }],
        tags: vec![GameplayTag::SPECIAL_STATE],
        conditions: vec![SkillCondition::MpCost(6)],
        cooldown: 2,
        priority: 15,
    });

    // 净化
    registry.register(SkillData {
        id: "cleanse_skill".into(),
        name: "净化".into(),
        description: "驱散友方所有负面效果".into(),
        name_key: Some("skill.s_1006.name".into()),
        desc_key: Some("skill.s_1006.desc".into()),
        cost_mp: 8,
        range: 2,
        targeting: SkillTargeting::SingleAlly,
        effects: vec![EffectDef::Cleanse],
        tags: vec![GameplayTag::SPECIAL_STATE],
        conditions: vec![SkillCondition::MpCost(8)],
        cooldown: 3,
        priority: 12,
    });
}
