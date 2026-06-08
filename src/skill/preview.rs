use crate::buff::BuffRegistry;
use crate::gameplay::effect::{
    EffectHandlerRegistry, EffectPreview as HandlerEffectPreview, PreviewContext,
};
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

/// 将 handler 层的 EffectPreview 转换为 preview 模块的 EffectPreview
/// 两者结构相同但属于不同模块，保持 preview 模块的公共 API 不变
impl From<HandlerEffectPreview> for EffectPreview {
    fn from(p: HandlerEffectPreview) -> Self {
        match p {
            HandlerEffectPreview::Damage { amount, lethal } => {
                EffectPreview::Damage { amount, lethal }
            }
            HandlerEffectPreview::Heal { amount } => EffectPreview::Heal { amount },
            HandlerEffectPreview::BuffApplied { buff_name } => {
                EffectPreview::BuffApplied { buff_name }
            }
            HandlerEffectPreview::Cleanse => EffectPreview::Cleanse,
        }
    }
}

/// 预览技能效果（纯函数，不修改任何状态）
/// 使用 EffectHandlerRegistry trait 分发，新增效果类型无需修改此处
pub fn preview_skill_effects(
    ctx: &SkillExecutionContext,
    skill_data: &SkillData,
    buff_registry: &BuffRegistry,
) -> SkillPreview {
    preview_skill_effects_with_registry(
        ctx,
        skill_data,
        buff_registry,
        &EffectHandlerRegistry::default(),
    )
}

/// 使用指定 Registry 预览技能效果（供测试和自定义注册表使用）
pub fn preview_skill_effects_with_registry(
    ctx: &SkillExecutionContext,
    skill_data: &SkillData,
    buff_registry: &BuffRegistry,
    handler_registry: &EffectHandlerRegistry,
) -> SkillPreview {
    let mut predictions = Vec::new();

    let preview_ctx = PreviewContext {
        source_attrs: ctx.source_attrs.clone(),
        target_attrs: ctx.target_attrs.clone(),
        terrain_defense_bonus: ctx.terrain_defense_bonus,
        buff_registry: buff_registry.clone(),
    };

    for effect_def in &skill_data.effects {
        // 通过 EffectHandlerRegistry trait 分发，新增效果类型无需修改此处
        if let Some(handler) = handler_registry.find(effect_def.type_name()) {
            if let Some(preview) = handler.preview(effect_def, &preview_ctx) {
                predictions.push(preview.into());
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
    use super::super::domain::{BASIC_ATTACK_ID, SkillTargeting};
    use super::*;
    use crate::gameplay::attribute::AttributeKind;
    use crate::gameplay::effect::EffectDef;
    use crate::gameplay::tag::GameplayTags;

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
