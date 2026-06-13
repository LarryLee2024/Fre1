// 技能默认注册数据

use super::SkillRegistry;
use super::types::*;
use crate::core::effect::EffectDef;

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
        cost_mp: 0,
        range: 0,
        targeting: SkillTargeting::SingleEnemy,
        effects: vec![EffectDef::Damage {
            multiplier: 1.0,
            ignore_def_percent: 0.0,
        }],
        tags: vec![],
        conditions: vec![],
        cooldown: 0,
        priority: 0,
    });

    // 冲锋
    registry.register(SkillData {
        id: "charge".into(),
        name: "冲锋".into(),
        description: "1.5倍伤害，需要近战".into(),
        cost_mp: 0,
        range: 0,
        targeting: SkillTargeting::SingleEnemy,
        effects: vec![EffectDef::Damage {
            multiplier: 1.5,
            ignore_def_percent: 0.0,
        }],
        tags: vec![],
        conditions: vec![],
        cooldown: 2,
        priority: 5,
    });

    // 穿刺
    registry.register(SkillData {
        id: "pierce".into(),
        name: "穿刺".into(),
        description: "无视50%防御".into(),
        cost_mp: 0,
        range: 0,
        targeting: SkillTargeting::SingleEnemy,
        effects: vec![EffectDef::Damage {
            multiplier: 1.2,
            ignore_def_percent: 50.0,
        }],
        tags: vec![],
        conditions: vec![],
        cooldown: 3,
        priority: 8,
    });

    // 火球
    registry.register(SkillData {
        id: "fireball".into(),
        name: "火球".into(),
        description: "远程火属性攻击".into(),
        cost_mp: 0,
        range: 3,
        targeting: SkillTargeting::SingleEnemy,
        effects: vec![
            EffectDef::Damage {
                multiplier: 1.5,
                ignore_def_percent: 0.0,
            },
            EffectDef::ApplyBuff {
                buff_id: "burn".into(),
                duration: 2,
            },
        ],
        tags: vec![],
        conditions: vec![],
        cooldown: 2,
        priority: 10,
    });

    // 治疗
    registry.register(SkillData {
        id: "heal".into(),
        name: "治疗".into(),
        description: "恢复友方生命值".into(),
        cost_mp: 0,
        range: 2,
        targeting: SkillTargeting::SingleAlly,
        effects: vec![EffectDef::Heal { amount: 8 }],
        tags: vec![],
        conditions: vec![],
        cooldown: 2,
        priority: 15,
    });

    // 净化
    registry.register(SkillData {
        id: "cleanse_skill".into(),
        name: "净化".into(),
        description: "驱散友方所有负面效果".into(),
        cost_mp: 0,
        range: 2,
        targeting: SkillTargeting::SingleAlly,
        effects: vec![EffectDef::Cleanse],
        tags: vec![],
        conditions: vec![],
        cooldown: 3,
        priority: 12,
    });
}
