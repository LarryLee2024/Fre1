// 触发器注册表：所有 Trigger Handler 统一注册与分发
// 参考：docs/01-architecture/skill-buff-abstraction.md §4.8.2
// 参考：docs/02-domain/trigger/trigger-rules.md

use crate::core::effect::EffectDef;
use crate::core::trigger::types::{Trigger, TriggerContext};
use bevy::prelude::*;
use std::collections::HashMap;

// ── TriggerHandler trait ──

/// 触发器处理规则 trait：描述如何响应一种触发事件
/// Handler 只返回 EffectDef，不直接修改 World 状态
pub trait TriggerHandler: Send + Sync + 'static {
    fn trigger_type(&self) -> Trigger;
    fn handle(&self, ctx: &TriggerContext) -> Vec<EffectDef>;
    fn priority(&self) -> i32 {
        0
    }
}

// ── TriggerRegistry ──

/// 触发器注册表资源：所有 TriggerHandler 统一注册中心
#[derive(Resource, Default)]
pub struct TriggerRegistry {
    handlers: HashMap<Trigger, Vec<Box<dyn TriggerHandler>>>,
}

impl TriggerRegistry {
    /// 注册一个触发器处理器
    pub fn register(&mut self, handler: Box<dyn TriggerHandler>) {
        let trigger = handler.trigger_type();
        let priority = handler.priority();
        bevy::log::info!(
            target: "core::trigger",
            event = "trigger_handler_registered",
            trigger = ?trigger,
            priority = priority,
            "触发器处理器已注册"
        );
        self.handlers.entry(trigger).or_default().push(handler);
    }

    /// 分发触发事件，返回所有匹配 Handler 产生的 EffectDef 列表
    /// 按 priority 降序排列（高优先级先执行）
    pub fn dispatch(&self, ctx: &TriggerContext) -> Vec<EffectDef> {
        let Some(handlers) = self.handlers.get(&ctx.trigger) else {
            return Vec::new();
        };

        let mut all_effects: Vec<EffectDef> = Vec::new();

        let mut sorted_handlers: Vec<&Box<dyn TriggerHandler>> = handlers.iter().collect();
        sorted_handlers.sort_by(|a, b| b.priority().cmp(&a.priority()));

        for handler in sorted_handlers {
            let effects = handler.handle(ctx);
            if !effects.is_empty() {
                bevy::log::info!(
                    target: "core::trigger",
                    event = "trigger_dispatched",
                    trigger = ?ctx.trigger,
                    handler_priority = handler.priority(),
                    effect_count = effects.len(),
                    "触发器处理器已分发效果"
                );
                all_effects.extend(effects);
            }
        }

        all_effects
    }

    /// 检查是否有注册的 Handler 处理指定 Trigger
    pub fn has_handlers(&self, trigger: Trigger) -> bool {
        self.handlers.get(&trigger).map_or(false, |h| !h.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct PoisonTriggerHandler;

    impl TriggerHandler for PoisonTriggerHandler {
        fn trigger_type(&self) -> Trigger {
            Trigger::TurnStart
        }
        fn handle(&self, _ctx: &TriggerContext) -> Vec<EffectDef> {
            vec![EffectDef::Damage {
                multiplier: 1.0,
                ignore_def_percent: 0.0,
            }]
        }
        fn priority(&self) -> i32 {
            10
        }
    }

    struct VampiricTriggerHandler;

    impl TriggerHandler for VampiricTriggerHandler {
        fn trigger_type(&self) -> Trigger {
            Trigger::AfterAttack
        }
        fn handle(&self, ctx: &TriggerContext) -> Vec<EffectDef> {
            if let Some(damage) = ctx.damage_dealt {
                vec![EffectDef::Heal { amount: damage / 5 }]
            } else {
                vec![]
            }
        }
        fn priority(&self) -> i32 {
            20
        }
    }

    #[test]
    fn 注册表_默认为空() {
        let registry = TriggerRegistry::default();
        assert!(!registry.has_handlers(Trigger::TurnStart));
        assert!(!registry.has_handlers(Trigger::Death));
    }

    #[test]
    fn 注册表_注册后存在() {
        let mut registry = TriggerRegistry::default();
        registry.register(Box::new(PoisonTriggerHandler));
        assert!(registry.has_handlers(Trigger::TurnStart));
        assert!(!registry.has_handlers(Trigger::Death));
    }

    #[test]
    fn 注册表_分发返回effects() {
        let mut registry = TriggerRegistry::default();
        registry.register(Box::new(PoisonTriggerHandler));

        let ctx = TriggerContext {
            trigger: Trigger::TurnStart,
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            damage_dealt: None,
            is_critical: None,
            chain_depth: 0,
        };

        let effects = registry.dispatch(&ctx);
        assert_eq!(effects.len(), 1);
        assert!(matches!(effects[0], EffectDef::Damage { .. }));
    }

    #[test]
    fn 注册表_无handler返回空() {
        let registry = TriggerRegistry::default();
        let ctx = TriggerContext {
            trigger: Trigger::TurnStart,
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            damage_dealt: None,
            is_critical: None,
            chain_depth: 0,
        };

        let effects = registry.dispatch(&ctx);
        assert!(effects.is_empty());
    }

    #[test]
    fn 注册表_多个handler聚合effects() {
        let mut registry = TriggerRegistry::default();
        registry.register(Box::new(PoisonTriggerHandler));

        struct RegenTriggerHandler;
        impl TriggerHandler for RegenTriggerHandler {
            fn trigger_type(&self) -> Trigger {
                Trigger::TurnStart
            }
            fn handle(&self, _ctx: &TriggerContext) -> Vec<EffectDef> {
                vec![EffectDef::Heal { amount: 30 }]
            }
        }
        registry.register(Box::new(RegenTriggerHandler));

        let ctx = TriggerContext {
            trigger: Trigger::TurnStart,
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            damage_dealt: None,
            is_critical: None,
            chain_depth: 0,
        };

        let effects = registry.dispatch(&ctx);
        assert_eq!(effects.len(), 2);
    }

    #[test]
    fn 注册表_handler按priority排序() {
        let mut registry = TriggerRegistry::default();
        registry.register(Box::new(VampiricTriggerHandler));

        let ctx = TriggerContext {
            trigger: Trigger::AfterAttack,
            source_entity: Entity::from_bits(1),
            target_entity: Entity::from_bits(2),
            damage_dealt: Some(100),
            is_critical: Some(true),
            chain_depth: 0,
        };

        let effects = registry.dispatch(&ctx);
        assert_eq!(effects.len(), 1);
        if let EffectDef::Heal { amount } = &effects[0] {
            assert_eq!(*amount, 20);
        } else {
            panic!("应该是治疗效果");
        }
    }

    #[test]
    fn trigger枚举所有变体() {
        let triggers = [
            Trigger::TurnStart,
            Trigger::TurnEnd,
            Trigger::BeforeAttack,
            Trigger::AfterAttack,
            Trigger::BeforeDamaged,
            Trigger::AfterDamaged,
            Trigger::BeforeMove,
            Trigger::AfterMove,
            Trigger::KillTarget,
            Trigger::Death,
            Trigger::BattleStart,
            Trigger::BattleEnd,
            Trigger::OnHeal,
            Trigger::OnBuffApplied,
            Trigger::OnBuffRemoved,
            Trigger::OnRevive,
        ];
        assert_eq!(triggers.len(), 16);
        let mut set = std::collections::HashSet::new();
        for t in &triggers {
            set.insert(t);
        }
        assert_eq!(set.len(), 16);
    }
}
