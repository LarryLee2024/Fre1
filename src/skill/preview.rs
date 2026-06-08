use crate::gameplay::attribute::AttributeKind;
use crate::gameplay::effect::{EffectDef, calculate_damage_from_effect};
use crate::buff::BuffRegistry;
use bevy::prelude::*;

use super::domain::SkillData;

// ── 技能执行上下文 ──

/// 技能执行上下文：封装一次技能释放的所有信息
#[derive(Clone, Debug)]
pub struct SkillExecutionContext {
    pub source: Entity,
    pub target: Entity,
    pub skill_id: String,
    pub source_attrs: crate::gameplay::attribute::Attributes,
    pub target_attrs: crate::gameplay::attribute::Attributes,
    pub source_tags: crate::gameplay::tag::GameplayTags,
    pub target_tags: crate::gameplay::tag::GameplayTags,
    pub terrain_defense_bonus: i32,
}

impl SkillExecutionContext {
    /// 从 ECS 查询构建上下文（纯数据快照，避免借用冲突）
    pub fn from_query(
        source: Entity,
        target: Entity,
        skill_id: &str,
        source_attrs: &crate::gameplay::attribute::Attributes,
        target_attrs: &crate::gameplay::attribute::Attributes,
        source_tags: &crate::gameplay::tag::GameplayTags,
        target_tags: &crate::gameplay::tag::GameplayTags,
        terrain_defense_bonus: i32,
    ) -> Self {
        Self {
            source,
            target,
            skill_id: skill_id.to_string(),
            source_attrs: source_attrs.clone(),
            target_attrs: target_attrs.clone(),
            source_tags: source_tags.clone(),
            target_tags: target_tags.clone(),
            terrain_defense_bonus,
        }
    }
}

// ── 效果预览 ──

/// 技能效果预览结果
#[derive(Clone, Debug)]
pub struct SkillPreview {
    pub skill_id: String,
    pub skill_name: String,
    pub predictions: Vec<EffectPreview>,
}

/// 单个效果的预览
#[derive(Clone, Debug)]
pub enum EffectPreview {
    Damage { amount: i32, lethal: bool },
    Heal { amount: i32 },
    BuffApplied { buff_name: String },
    Cleanse,
}

/// 预览技能效果（纯函数，不修改任何状态）
pub fn preview_skill_effects(
    ctx: &SkillExecutionContext,
    skill_data: &SkillData,
    buff_registry: &BuffRegistry,
) -> SkillPreview {
    let mut predictions = Vec::new();

    for effect_def in &skill_data.effects {
        match effect_def {
            EffectDef::Damage {
                multiplier,
                ignore_def_percent,
            } => {
                let effective_atk = ctx.source_attrs.get(AttributeKind::Atk);
                let effective_def = ctx.target_attrs.get(AttributeKind::Def);
                let base_def = ctx
                    .target_attrs
                    .base
                    .get(&AttributeKind::Def)
                    .copied()
                    .unwrap_or(0.0);

                let amount = calculate_damage_from_effect(
                    effective_atk,
                    effective_def,
                    base_def,
                    *multiplier,
                    *ignore_def_percent,
                    ctx.terrain_defense_bonus,
                );
                let current_hp = ctx.target_attrs.get(AttributeKind::Hp);
                predictions.push(EffectPreview::Damage {
                    amount,
                    lethal: current_hp - amount as f32 <= 0.0,
                });
            }
            EffectDef::Heal { amount } => {
                let max_hp = ctx.target_attrs.get(AttributeKind::MaxHp);
                let current_hp = ctx.target_attrs.get(AttributeKind::Hp);
                let actual = (*amount as f32).min(max_hp - current_hp).max(0.0) as i32;
                predictions.push(EffectPreview::Heal { amount: actual });
            }
            EffectDef::ApplyBuff { buff_id, .. } => {
                let buff_name = buff_registry
                    .get(buff_id)
                    .map(|b| b.name.as_str())
                    .unwrap_or(buff_id);
                predictions.push(EffectPreview::BuffApplied {
                    buff_name: buff_name.to_string(),
                });
            }
            EffectDef::Cleanse => {
                predictions.push(EffectPreview::Cleanse);
            }
        }
    }

    SkillPreview {
        skill_id: skill_data.id.clone(),
        skill_name: skill_data.name.clone(),
        predictions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameplay::attribute::AttributeKind;
    use crate::gameplay::tag::GameplayTags;
    use super::super::domain::{BASIC_ATTACK_ID, SkillTargeting};

    #[test]
    fn 预览_伤害预览() {
        let mut source_attrs = crate::gameplay::attribute::Attributes::default();
        source_attrs.set_base(AttributeKind::Atk, 10.0);
        let mut target_attrs = crate::gameplay::attribute::Attributes::default();
        target_attrs.set_base(AttributeKind::Def, 3.0);
        target_attrs.set_base(AttributeKind::Hp, 20.0);

        let ctx = SkillExecutionContext {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: BASIC_ATTACK_ID.into(),
            source_attrs,
            target_attrs,
            source_tags: crate::gameplay::tag::GameplayTags::default(),
            target_tags: crate::gameplay::tag::GameplayTags::default(),
            terrain_defense_bonus: 0,
        };

        // 使用硬编码构建 SkillData 用于测试（不依赖文件系统）
        let skill = SkillData {
            id: BASIC_ATTACK_ID.into(),
            name: "普通攻击".into(),
            description: String::new(),
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
        };
        let buff_reg = crate::buff::BuffRegistry::default();
        let preview = preview_skill_effects(&ctx, &skill, &buff_reg);

        assert_eq!(preview.skill_id, BASIC_ATTACK_ID);
        assert_eq!(preview.predictions.len(), 1);
        if let EffectPreview::Damage { amount, lethal } = &preview.predictions[0] {
            assert_eq!(*amount, 7); // 10 - 3 = 7
            assert!(!lethal);
        } else {
            panic!("应该是伤害预览");
        }
    }

    #[test]
    fn 预览_致死伤害标记() {
        let mut source_attrs = crate::gameplay::attribute::Attributes::default();
        source_attrs.set_base(AttributeKind::Atk, 50.0);
        let mut target_attrs = crate::gameplay::attribute::Attributes::default();
        target_attrs.set_base(AttributeKind::Def, 3.0);
        target_attrs.set_base(AttributeKind::Hp, 5.0);

        let ctx = SkillExecutionContext {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: BASIC_ATTACK_ID.into(),
            source_attrs,
            target_attrs,
            source_tags: crate::gameplay::tag::GameplayTags::default(),
            target_tags: crate::gameplay::tag::GameplayTags::default(),
            terrain_defense_bonus: 0,
        };

        let skill = SkillData {
            id: BASIC_ATTACK_ID.into(),
            name: "普通攻击".into(),
            description: String::new(),
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
        };
        let buff_reg = crate::buff::BuffRegistry::default();
        let preview = preview_skill_effects(&ctx, &skill, &buff_reg);

        if let EffectPreview::Damage { lethal, .. } = &preview.predictions[0] {
            assert!(lethal);
        }
    }

    #[test]
    fn 预览_治疗预览() {
        let mut source_attrs = crate::gameplay::attribute::Attributes::default();
        let mut target_attrs = crate::gameplay::attribute::Attributes::default();
        target_attrs.set_base(AttributeKind::Hp, 12.0);
        target_attrs.set_base(AttributeKind::MaxHp, 20.0);

        let ctx = SkillExecutionContext {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: "heal".into(),
            source_attrs,
            target_attrs,
            source_tags: crate::gameplay::tag::GameplayTags::default(),
            target_tags: crate::gameplay::tag::GameplayTags::default(),
            terrain_defense_bonus: 0,
        };

        let skill = SkillData {
            id: "heal".into(),
            name: "治疗".into(),
            description: String::new(),
            cost_mp: 0,
            range: 3,
            targeting: SkillTargeting::SingleAlly,
            effects: vec![EffectDef::Heal { amount: 8 }],
            tags: vec![],
            conditions: vec![],
            cooldown: 2,
            priority: 15,
        };
        let buff_reg = crate::buff::BuffRegistry::default();
        let preview = preview_skill_effects(&ctx, &skill, &buff_reg);

        if let EffectPreview::Heal { amount } = &preview.predictions[0] {
            assert_eq!(*amount, 8);
        }
    }

    #[test]
    fn 预览_治疗不超过最大hp() {
        let mut source_attrs = crate::gameplay::attribute::Attributes::default();
        let mut target_attrs = crate::gameplay::attribute::Attributes::default();
        target_attrs.set_base(AttributeKind::Hp, 18.0);
        target_attrs.set_base(AttributeKind::MaxHp, 20.0);

        let ctx = SkillExecutionContext {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: "heal".into(),
            source_attrs,
            target_attrs,
            source_tags: crate::gameplay::tag::GameplayTags::default(),
            target_tags: crate::gameplay::tag::GameplayTags::default(),
            terrain_defense_bonus: 0,
        };

        let skill = SkillData {
            id: "heal".into(),
            name: "治疗".into(),
            description: String::new(),
            cost_mp: 0,
            range: 3,
            targeting: SkillTargeting::SingleAlly,
            effects: vec![EffectDef::Heal { amount: 8 }],
            tags: vec![],
            conditions: vec![],
            cooldown: 2,
            priority: 15,
        };
        let buff_reg = crate::buff::BuffRegistry::default();
        let preview = preview_skill_effects(&ctx, &skill, &buff_reg);

        if let EffectPreview::Heal { amount } = &preview.predictions[0] {
            assert_eq!(*amount, 2); // min(8, 20-18) = 2
        }
    }
}
