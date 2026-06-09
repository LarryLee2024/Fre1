// 效果处理器 trait：描述如何生成/预览一种效果
// 新增效果类型只需实现此 trait 并注册，无需修改核心代码
// 遵循"Trait 描述规则，不描述内容"原则

use crate::buff::BuffRegistry;
use crate::gameplay::attribute::{AttributeKind, Attributes};
use crate::gameplay::tag::GameplayTag;
use bevy::prelude::*;

use super::types::{EffectDef, PendingEffectData, calculate_damage_from_effect};

// ── 上下文结构体（纯数据，避免 ECS 借用问题）──

/// 生成效果的上下文
#[derive(Clone, Debug)]
pub struct GenerateContext {
    pub source_entity: Entity,
    pub target_entity: Entity,
    pub source_attrs: Attributes,
    pub target_attrs: Attributes,
    pub defense_bonus: i32,
    pub skill_id: String,
    pub source_tags: Vec<GameplayTag>,
    pub terrain_id: String,
}

/// 预览效果的上下文
#[derive(Clone, Debug)]
pub struct PreviewContext {
    pub source_attrs: Attributes,
    pub target_attrs: Attributes,
    pub terrain_defense_bonus: i32,
    pub buff_registry: BuffRegistry,
}

// ── 效果预览结果 ──

/// 单个效果的预览
#[derive(Clone, Debug)]
pub enum EffectPreview {
    Damage { amount: i32, lethal: bool },
    Heal { amount: i32 },
    BuffApplied { buff_name: String },
    Cleanse,
}

// ── EffectHandler trait ──

/// 效果处理规则 trait：描述如何生成/预览一种效果
/// 新增效果类型只需实现此 trait 并注册到 EffectHandlerRegistry，无需修改核心代码
pub trait EffectHandler: Send + Sync + 'static {
    /// 此处理器负责的效果类型名（与 EffectDef::type_name 对应）
    fn type_name(&self) -> &'static str;

    /// 从效果定义生成待处理效果数据
    fn generate(&self, def: &EffectDef, ctx: &GenerateContext) -> Option<PendingEffectData>;

    /// 预览效果
    fn preview(&self, def: &EffectDef, ctx: &PreviewContext) -> Option<EffectPreview>;
}

// ── 内置处理器 ──

/// 伤害处理器
pub struct DamageHandler;

impl EffectHandler for DamageHandler {
    fn type_name(&self) -> &'static str {
        "Damage"
    }

    fn generate(&self, def: &EffectDef, ctx: &GenerateContext) -> Option<PendingEffectData> {
        let EffectDef::Damage {
            multiplier,
            ignore_def_percent,
        } = def
        else {
            return None;
        };

        let effective_atk = ctx.source_attrs.get(AttributeKind::Attack);
        let effective_def = ctx.target_attrs.get(AttributeKind::Defense);
        let base_def = ctx.target_attrs.core_base(AttributeKind::Vitality);

        let amount = calculate_damage_from_effect(
            effective_atk,
            effective_def,
            base_def,
            *multiplier,
            *ignore_def_percent,
            ctx.defense_bonus,
        );

        Some(PendingEffectData::Damage {
            amount,
            is_skill: ctx.skill_id != crate::skill::BASIC_ATTACK_ID,
        })
    }

    fn preview(&self, def: &EffectDef, ctx: &PreviewContext) -> Option<EffectPreview> {
        let EffectDef::Damage {
            multiplier,
            ignore_def_percent,
        } = def
        else {
            return None;
        };

        let effective_atk = ctx.source_attrs.get(AttributeKind::Attack);
        let effective_def = ctx.target_attrs.get(AttributeKind::Defense);
        let base_def = ctx.target_attrs.core_base(AttributeKind::Vitality);

        let amount = calculate_damage_from_effect(
            effective_atk,
            effective_def,
            base_def,
            *multiplier,
            *ignore_def_percent,
            ctx.terrain_defense_bonus,
        );
        let current_hp = ctx.target_attrs.get(AttributeKind::Hp);
        Some(EffectPreview::Damage {
            amount,
            lethal: current_hp - amount as f32 <= 0.0,
        })
    }
}

/// 治疗处理器
pub struct HealHandler;

impl EffectHandler for HealHandler {
    fn type_name(&self) -> &'static str {
        "Heal"
    }

    fn generate(&self, def: &EffectDef, _ctx: &GenerateContext) -> Option<PendingEffectData> {
        let EffectDef::Heal { amount } = def else {
            return None;
        };
        Some(PendingEffectData::Heal { amount: *amount })
    }

    fn preview(&self, def: &EffectDef, ctx: &PreviewContext) -> Option<EffectPreview> {
        let EffectDef::Heal { amount } = def else {
            return None;
        };
        let max_hp = ctx.target_attrs.get(AttributeKind::MaxHp);
        let current_hp = ctx.target_attrs.get(AttributeKind::Hp);
        let actual = (*amount as f32).min(max_hp - current_hp).max(0.0) as i32;
        Some(EffectPreview::Heal { amount: actual })
    }
}

/// Buff 处理器
pub struct BuffHandler;

impl EffectHandler for BuffHandler {
    fn type_name(&self) -> &'static str {
        "ApplyBuff"
    }

    fn generate(&self, def: &EffectDef, _ctx: &GenerateContext) -> Option<PendingEffectData> {
        let EffectDef::ApplyBuff { buff_id, duration } = def else {
            return None;
        };
        Some(PendingEffectData::ApplyBuff {
            buff_id: buff_id.clone(),
            duration: *duration,
        })
    }

    fn preview(&self, def: &EffectDef, ctx: &PreviewContext) -> Option<EffectPreview> {
        let EffectDef::ApplyBuff { buff_id, .. } = def else {
            return None;
        };
        let buff_name = ctx
            .buff_registry
            .get(buff_id)
            .map(|b| b.name.as_str())
            .unwrap_or(buff_id);
        Some(EffectPreview::BuffApplied {
            buff_name: buff_name.to_string(),
        })
    }
}

/// 净化处理器
pub struct CleanseHandler;

impl EffectHandler for CleanseHandler {
    fn type_name(&self) -> &'static str {
        "Cleanse"
    }

    fn generate(&self, def: &EffectDef, _ctx: &GenerateContext) -> Option<PendingEffectData> {
        let EffectDef::Cleanse = def else {
            return None;
        };
        Some(PendingEffectData::Cleanse)
    }

    fn preview(&self, def: &EffectDef, _ctx: &PreviewContext) -> Option<EffectPreview> {
        let EffectDef::Cleanse = def else {
            return None;
        };
        Some(EffectPreview::Cleanse)
    }
}

// ── 处理器注册表 ──

/// 效果处理器注册表资源
/// 通过 type_name 查找对应的 EffectHandler，实现 trait 分发
#[derive(Resource)]
pub struct EffectHandlerRegistry {
    handlers: Vec<Box<dyn EffectHandler>>,
}

impl Default for EffectHandlerRegistry {
    fn default() -> Self {
        let mut registry = Self {
            handlers: Vec::new(),
        };
        registry.register_defaults();
        registry
    }
}

impl EffectHandlerRegistry {
    /// 根据类型名查找处理器
    pub fn find(&self, type_name: &str) -> Option<&dyn EffectHandler> {
        self.handlers
            .iter()
            .find(|h| h.type_name() == type_name)
            .map(|h| h.as_ref())
    }

    /// 注册一个处理器
    pub fn register(&mut self, handler: Box<dyn EffectHandler>) {
        // 避免重复注册
        let name = handler.type_name();
        if self.find(name).is_some() {
            bevy::log::warn!("效果处理器 {} 已注册，跳过重复注册", name);
            return;
        }
        self.handlers.push(handler);
    }

    /// 注册4个内置处理器
    pub fn register_defaults(&mut self) {
        self.register(Box::new(DamageHandler));
        self.register(Box::new(HealHandler));
        self.register(Box::new(BuffHandler));
        self.register(Box::new(CleanseHandler));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameplay::attribute::Attributes;

    /// 构建测试用 GenerateContext
    fn make_generate_ctx() -> GenerateContext {
        let mut source_attrs = Attributes::default();
        source_attrs.set_base(AttributeKind::Might, 5.0);
        source_attrs.set_base(AttributeKind::Vitality, 5.0);
        source_attrs.set_base(AttributeKind::Agility, 6.0);
        source_attrs.set_base(AttributeKind::Dexterity, 3.0);
        source_attrs.set_base(AttributeKind::Intelligence, 2.0);
        source_attrs.set_base(AttributeKind::Willpower, 3.0);
        source_attrs.set_base(AttributeKind::Presence, 2.0);
        source_attrs.set_base(AttributeKind::Luck, 2.0);
        source_attrs.set_base_attack_range(1);
        source_attrs.fill_vital_resources();

        let mut target_attrs = Attributes::default();
        target_attrs.set_base(AttributeKind::Might, 2.0);
        target_attrs.set_base(AttributeKind::Vitality, 3.0);
        target_attrs.set_base(AttributeKind::Agility, 4.0);
        target_attrs.set_base(AttributeKind::Dexterity, 2.0);
        target_attrs.set_base(AttributeKind::Intelligence, 1.0);
        target_attrs.set_base(AttributeKind::Willpower, 2.0);
        target_attrs.set_base(AttributeKind::Presence, 1.0);
        target_attrs.set_base(AttributeKind::Luck, 2.0);
        target_attrs.set_base_attack_range(1);
        target_attrs.fill_vital_resources();

        GenerateContext {
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            source_attrs,
            target_attrs,
            defense_bonus: 0,
            skill_id: "basic_attack".into(),
            source_tags: vec![],
            terrain_id: "plain".to_string(),
        }
    }

    /// 构建测试用 PreviewContext
    fn make_preview_ctx() -> PreviewContext {
        let mut source_attrs = Attributes::default();
        source_attrs.set_base(AttributeKind::Might, 5.0);
        source_attrs.set_base(AttributeKind::Vitality, 5.0);
        source_attrs.set_base(AttributeKind::Agility, 6.0);
        source_attrs.set_base(AttributeKind::Dexterity, 3.0);
        source_attrs.set_base(AttributeKind::Intelligence, 2.0);
        source_attrs.set_base(AttributeKind::Willpower, 3.0);
        source_attrs.set_base(AttributeKind::Presence, 2.0);
        source_attrs.set_base(AttributeKind::Luck, 2.0);
        source_attrs.set_base_attack_range(1);
        source_attrs.fill_vital_resources();

        let mut target_attrs = Attributes::default();
        target_attrs.set_base(AttributeKind::Might, 2.0);
        target_attrs.set_base(AttributeKind::Vitality, 3.0);
        target_attrs.set_base(AttributeKind::Agility, 4.0);
        target_attrs.set_base(AttributeKind::Dexterity, 2.0);
        target_attrs.set_base(AttributeKind::Intelligence, 1.0);
        target_attrs.set_base(AttributeKind::Willpower, 2.0);
        target_attrs.set_base(AttributeKind::Presence, 1.0);
        target_attrs.set_base(AttributeKind::Luck, 2.0);
        target_attrs.set_base_attack_range(1);
        target_attrs.fill_vital_resources();
        // HP 有缺口，用于测试治疗预览
        target_attrs.set_base(AttributeKind::Hp, 12.0);

        PreviewContext {
            source_attrs,
            target_attrs,
            terrain_defense_bonus: 0,
            buff_registry: BuffRegistry::default(),
        }
    }

    #[test]
    fn 注册表_默认注册4个处理器() {
        let registry = EffectHandlerRegistry::default();
        assert!(registry.find("Damage").is_some());
        assert!(registry.find("Heal").is_some());
        assert!(registry.find("ApplyBuff").is_some());
        assert!(registry.find("Cleanse").is_some());
        assert!(registry.find("Unknown").is_none());
    }

    #[test]
    fn 注册表_不重复注册() {
        let mut registry = EffectHandlerRegistry::default();
        let count_before = registry.handlers.len();
        registry.register(Box::new(DamageHandler));
        assert_eq!(registry.handlers.len(), count_before);
    }

    #[test]
    fn 伤害处理器_生成() {
        let registry = EffectHandlerRegistry::default();
        let handler = registry.find("Damage").unwrap();
        let ctx = make_generate_ctx();
        let def = EffectDef::Damage {
            multiplier: 1.0,
            ignore_def_percent: 0.0,
        };
        let result = handler.generate(&def, &ctx);
        assert!(result.is_some());
        if let PendingEffectData::Damage { amount, is_skill } = result.unwrap() {
            assert_eq!(amount, 7); // 10 - 3 = 7
            assert!(!is_skill);
        } else {
            panic!("应该是伤害数据");
        }
    }

    #[test]
    fn 伤害处理器_预览() {
        let registry = EffectHandlerRegistry::default();
        let handler = registry.find("Damage").unwrap();
        let ctx = make_preview_ctx();
        let def = EffectDef::Damage {
            multiplier: 1.0,
            ignore_def_percent: 0.0,
        };
        let result = handler.preview(&def, &ctx);
        assert!(result.is_some());
        if let EffectPreview::Damage { amount, lethal } = result.unwrap() {
            assert_eq!(amount, 7);
            assert!(!lethal);
        } else {
            panic!("应该是伤害预览");
        }
    }

    #[test]
    fn 治疗处理器_生成() {
        let registry = EffectHandlerRegistry::default();
        let handler = registry.find("Heal").unwrap();
        let ctx = make_generate_ctx();
        let def = EffectDef::Heal { amount: 8 };
        let result = handler.generate(&def, &ctx);
        assert!(result.is_some());
        if let PendingEffectData::Heal { amount } = result.unwrap() {
            assert_eq!(amount, 8);
        } else {
            panic!("应该是治疗数据");
        }
    }

    #[test]
    fn 治疗处理器_预览() {
        let registry = EffectHandlerRegistry::default();
        let handler = registry.find("Heal").unwrap();
        let ctx = make_preview_ctx();
        let def = EffectDef::Heal { amount: 8 };
        let result = handler.preview(&def, &ctx);
        assert!(result.is_some());
        if let EffectPreview::Heal { amount } = result.unwrap() {
            assert_eq!(amount, 8);
        } else {
            panic!("应该是治疗预览");
        }
    }

    #[test]
    fn buff处理器_生成() {
        let registry = EffectHandlerRegistry::default();
        let handler = registry.find("ApplyBuff").unwrap();
        let ctx = make_generate_ctx();
        let def = EffectDef::ApplyBuff {
            buff_id: "burn".into(),
            duration: 2,
        };
        let result = handler.generate(&def, &ctx);
        assert!(result.is_some());
        if let PendingEffectData::ApplyBuff { buff_id, duration } = result.unwrap() {
            assert_eq!(buff_id, "burn");
            assert_eq!(duration, 2);
        } else {
            panic!("应该是 Buff 数据");
        }
    }

    #[test]
    fn 净化处理器_生成() {
        let registry = EffectHandlerRegistry::default();
        let handler = registry.find("Cleanse").unwrap();
        let ctx = make_generate_ctx();
        let def = EffectDef::Cleanse;
        let result = handler.generate(&def, &ctx);
        assert!(result.is_some());
        assert!(matches!(result.unwrap(), PendingEffectData::Cleanse));
    }

    #[test]
    fn 类型不匹配返回none() {
        let registry = EffectHandlerRegistry::default();
        let handler = registry.find("Damage").unwrap();
        let ctx = make_generate_ctx();
        // 传入 Heal 定义给 Damage 处理器
        let def = EffectDef::Heal { amount: 5 };
        assert!(handler.generate(&def, &ctx).is_none());
    }
}
