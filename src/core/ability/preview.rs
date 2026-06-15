use crate::core::buff::BuffRegistry;
use crate::core::effect::{
    EffectHandlerRegistry, EffectPreview as HandlerEffectPreview, PreviewContext,
};
use bevy::prelude::*;

use super::domain::SkillData;

// ── 技能执行上下文 ──

/// 技能执行上下文：封装一次技能释放的所有信息
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct SkillExecutionContext {
    pub source: Entity,
    pub target: Entity,
    pub skill_id: String,
    pub source_attrs: crate::core::attribute::Attributes,
    pub target_attrs: crate::core::attribute::Attributes,
    pub source_tags: crate::core::tag::GameplayTags,
    pub target_tags: crate::core::tag::GameplayTags,
    pub terrain_defense_bonus: i32,
}

impl SkillExecutionContext {
    /// 从 ECS 查询构建上下文（纯数据快照，避免借用冲突）
    #[allow(dead_code)]
    pub fn from_query(
        source: Entity,
        target: Entity,
        skill_id: &str,
        source_attrs: &crate::core::attribute::Attributes,
        target_attrs: &crate::core::attribute::Attributes,
        source_tags: &crate::core::tag::GameplayTags,
        target_tags: &crate::core::tag::GameplayTags,
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
#[allow(dead_code)]
pub struct SkillPreview {
    pub skill_id: String,
    pub skill_name: String,
    pub predictions: Vec<EffectPreview>,
}

/// 单个效果的预览
#[derive(Clone, Debug)]
#[allow(dead_code)]
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
    // ================================================
    // Bevy SRPG AI宪法 v1.1 自检结果（测试专用）
    // ================================================
    // ✅ 测行为不测实现：是 — 断言验证预览结果，不验证内部计算步骤
    // ✅ 符合领域规则：是 — 覆盖 INV-SKILL-017~021 技能预览不变量
    // ✅ 确定性：是 — 硬编码属性值和技能数据
    // ✅ 使用标准数据：是 — 使用标准 SkillPreview
    // ✅ 无越界测试：是 — 仅测试公共 API
    // ✅ 未测试私有实现：是 — 仅通过 pub 接口测试
    // ================================================
    use super::super::domain::{BASIC_ATTACK_ID, SkillTargeting};
    use super::*;
    use crate::core::attribute::AttributeKind;
    use crate::core::effect::EffectDef;

    fn make_source_attrs(atk: f32) -> crate::core::attribute::Attributes {
        let mut attrs = crate::core::attribute::Attributes::default();
        // Attack = Might * 2, 所以 Might = atk / 2
        attrs.set_base(AttributeKind::Might, atk / 2.0);
        attrs.set_base_attack_range(1);
        attrs.fill_vital_resources();
        attrs
    }

    fn make_target_attrs(def: f32, hp: f32) -> crate::core::attribute::Attributes {
        let mut attrs = crate::core::attribute::Attributes::default();
        // Defense = Vitality, 所以 Vitality = def
        // MaxHp = 5 + Vitality * 5 = 5 + def * 5
        attrs.set_base(AttributeKind::Vitality, def);
        attrs.fill_vital_resources();
        // 覆盖当前 HP 为指定值
        attrs.set_vital(AttributeKind::Hp, hp);
        attrs
    }

    #[test]
    fn 预览_伤害预览() {
        let source_attrs = make_source_attrs(10.0);
        let target_attrs = make_target_attrs(3.0, 20.0);

        let ctx = SkillExecutionContext {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: BASIC_ATTACK_ID.into(),
            source_attrs,
            target_attrs,
            source_tags: crate::core::tag::GameplayTags::default(),
            target_tags: crate::core::tag::GameplayTags::default(),
            terrain_defense_bonus: 0,
        };

        // 使用硬编码构建 SkillData 用于测试（不依赖文件系统）
        let skill = SkillData {
            id: BASIC_ATTACK_ID.into(),
            name: "普通攻击".into(),
            description: String::new(),
            name_key: None,
            desc_key: None,
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
        let buff_reg = crate::core::buff::BuffRegistry::default();
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
        let source_attrs = make_source_attrs(50.0);
        let target_attrs = make_target_attrs(3.0, 5.0);

        let ctx = SkillExecutionContext {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: BASIC_ATTACK_ID.into(),
            source_attrs,
            target_attrs,
            source_tags: crate::core::tag::GameplayTags::default(),
            target_tags: crate::core::tag::GameplayTags::default(),
            terrain_defense_bonus: 0,
        };

        let skill = SkillData {
            id: BASIC_ATTACK_ID.into(),
            name: "普通攻击".into(),
            description: String::new(),
            name_key: None,
            desc_key: None,
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
        let buff_reg = crate::core::buff::BuffRegistry::default();
        let preview = preview_skill_effects(&ctx, &skill, &buff_reg);

        if let EffectPreview::Damage { lethal, .. } = &preview.predictions[0] {
            assert!(lethal);
        }
    }

    #[test]
    fn 预览_治疗预览() {
        let source_attrs = crate::core::attribute::Attributes::default();
        let target_attrs = make_target_attrs(3.0, 12.0);
        // MaxHp = 5 + 3*5 = 20

        let ctx = SkillExecutionContext {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: "heal".into(),
            source_attrs,
            target_attrs,
            source_tags: crate::core::tag::GameplayTags::default(),
            target_tags: crate::core::tag::GameplayTags::default(),
            terrain_defense_bonus: 0,
        };

        let skill = SkillData {
            id: "heal".into(),
            name: "治疗".into(),
            description: String::new(),
            name_key: None,
            desc_key: None,
            cost_mp: 0,
            range: 3,
            targeting: SkillTargeting::SingleAlly,
            effects: vec![EffectDef::Heal { amount: 8 }],
            tags: vec![],
            conditions: vec![],
            cooldown: 2,
            priority: 15,
        };
        let buff_reg = crate::core::buff::BuffRegistry::default();
        let preview = preview_skill_effects(&ctx, &skill, &buff_reg);

        if let EffectPreview::Heal { amount } = &preview.predictions[0] {
            assert_eq!(*amount, 8);
        }
    }

    #[test]
    fn 预览_治疗不超过最大hp() {
        let source_attrs = crate::core::attribute::Attributes::default();
        let target_attrs = make_target_attrs(3.0, 18.0);
        // MaxHp = 5 + 3*5 = 20

        let ctx = SkillExecutionContext {
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: "heal".into(),
            source_attrs,
            target_attrs,
            source_tags: crate::core::tag::GameplayTags::default(),
            target_tags: crate::core::tag::GameplayTags::default(),
            terrain_defense_bonus: 0,
        };

        let skill = SkillData {
            id: "heal".into(),
            name: "治疗".into(),
            description: String::new(),
            name_key: None,
            desc_key: None,
            cost_mp: 0,
            range: 3,
            targeting: SkillTargeting::SingleAlly,
            effects: vec![EffectDef::Heal { amount: 8 }],
            tags: vec![],
            conditions: vec![],
            cooldown: 2,
            priority: 15,
        };
        let buff_reg = crate::core::buff::BuffRegistry::default();
        let preview = preview_skill_effects(&ctx, &skill, &buff_reg);

        if let EffectPreview::Heal { amount } = &preview.predictions[0] {
            assert_eq!(*amount, 2); // min(8, 20-18) = 2
        }
    }
}
