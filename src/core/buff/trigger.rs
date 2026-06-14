/// ADR-022: Buff 触发系统 — Phase 1 基础设施
///
/// 仅定义类型、trait 和注册表，不包含任何 Handler 实现（Phase 2+）。
/// 不修改现有行为，纯增量。
use bevy::prelude::*;
use std::collections::HashMap;

use crate::core::effect::EffectDef;

// ---------------------------------------------------------------------------
// Trigger 枚举 — 12 种触发时机
// ---------------------------------------------------------------------------

/// 触发时机枚举 — 与 `skill-buff-abstraction.md` §4.8 对齐
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigger {
    TurnStart,
    TurnEnd,
    BeforeAttack,
    AfterAttack,
    BeforeDamaged,
    AfterDamaged,
    BeforeMove,
    AfterMove,
    KillTarget,
    Death,
    BattleStart,
    BattleEnd,
}

// ---------------------------------------------------------------------------
// TriggerContext — 一次触发的全部上下文数据
// ---------------------------------------------------------------------------

/// 触发上下文 — 封装一次触发所需的全部数据
#[derive(Debug, Clone)]
pub struct TriggerContext {
    /// 触发时机
    pub trigger: Trigger,
    /// 来源实体（谁造成的触发）
    pub source: Entity,
    /// 目标实体（谁被触发影响）
    pub target: Entity,
    /// 技能 ID（如有）
    pub skill_id: Option<String>,
    /// 造成的伤害量（AfterAttack/AfterDamaged 需要）
    pub damage_dealt: Option<i32>,
    /// 是否暴击
    pub is_critical: bool,
    /// 地形 ID
    pub terrain_id: String,
}

// ---------------------------------------------------------------------------
// TriggerHandler trait
// ---------------------------------------------------------------------------

/// Trigger Handler trait — 每种触发器的处理逻辑
///
/// Handler 只能返回 `Vec<EffectDef>`，禁止直接修改 ECS World 状态。
/// 效果必须通过 Effect Pipeline 执行（Generate → Modify → Execute）。
pub trait TriggerHandler: Send + Sync + 'static {
    /// 触发器类型
    fn trigger_type(&self) -> Trigger;

    /// 处理触发事件，返回要执行的 Effect 列表
    fn handle(&self, ctx: &TriggerContext) -> Vec<EffectDef>;

    /// 触发优先级（决定同 Tick 内的执行顺序，值越大越先执行）
    fn priority(&self) -> i32 {
        0
    }
}

// ---------------------------------------------------------------------------
// TriggerRegistry — Resource
// ---------------------------------------------------------------------------

/// 触发器注册条目（handler + priority）
struct TriggerHandlerEntry {
    handler: Box<dyn TriggerHandler>,
    priority: i32,
}

/// 触发器注册表 — 所有 Trigger Handler 统一注册
///
/// Phase 1 仅提供注册和分发 API，不注册任何实际 Handler。
/// Phase 2+ 由 TriggerPlugin 调用 `init_resource::<TriggerRegistry>()`。
#[derive(Resource, Default)]
pub struct TriggerRegistry {
    handlers: HashMap<Trigger, Vec<TriggerHandlerEntry>>,
}

impl TriggerRegistry {
    /// 创建空注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个 TriggerHandler
    ///
    /// 按 priority 降序排列（值越大越先执行）。
    pub fn register(&mut self, handler: Box<dyn TriggerHandler>) {
        let trigger = handler.trigger_type();
        let priority = handler.priority();
        self.handlers
            .entry(trigger)
            .or_default()
            .push(TriggerHandlerEntry { handler, priority });
        // Sort by priority descending (higher = first)
        self.handlers
            .get_mut(&trigger)
            .unwrap()
            .sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// 分发触发事件，返回所有匹配 Handler 产出的 EffectDef
    pub fn dispatch(&self, ctx: &TriggerContext) -> Vec<EffectDef> {
        let mut all_effects = Vec::new();
        if let Some(entries) = self.handlers.get(&ctx.trigger) {
            for entry in entries {
                all_effects.extend(entry.handler.handle(ctx));
            }
        }
        all_effects
    }

    /// 检查指定触发器是否已注册 Handler
    pub fn has_handlers(&self, trigger: Trigger) -> bool {
        self.handlers.get(&trigger).map_or(false, |h| !h.is_empty())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// 空注册表测试
    #[test]
    fn registry_default_is_empty() {
        let registry = TriggerRegistry::new();
        assert!(!registry.has_handlers(Trigger::TurnStart));
        assert!(!registry.has_handlers(Trigger::AfterDamaged));
    }

    /// dispatch 空注册表返回空 Vec
    #[test]
    fn dispatch_empty_registry() {
        let registry = TriggerRegistry::new();
        let ctx = TriggerContext {
            trigger: Trigger::TurnStart,
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: None,
            damage_dealt: None,
            is_critical: false,
            terrain_id: "plain".to_string(),
        };
        let effects = registry.dispatch(&ctx);
        assert!(effects.is_empty());
    }

    /// Mock handler 用于测试注册和分发
    struct MockHealHandler {
        amount: i32,
    }

    impl TriggerHandler for MockHealHandler {
        fn trigger_type(&self) -> Trigger {
            Trigger::AfterDamaged
        }

        fn handle(&self, _ctx: &TriggerContext) -> Vec<EffectDef> {
            vec![EffectDef::Heal {
                amount: self.amount,
            }]
        }

        fn priority(&self) -> i32 {
            10
        }
    }

    #[test]
    fn register_and_dispatch_single_handler() {
        let mut registry = TriggerRegistry::new();
        registry.register(Box::new(MockHealHandler { amount: 5 }));

        assert!(registry.has_handlers(Trigger::AfterDamaged));
        assert!(!registry.has_handlers(Trigger::TurnStart));

        let ctx = TriggerContext {
            trigger: Trigger::AfterDamaged,
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: None,
            damage_dealt: Some(10),
            is_critical: false,
            terrain_id: "plain".to_string(),
        };
        let effects = registry.dispatch(&ctx);
        assert_eq!(effects.len(), 1);
        assert_eq!(effects[0].type_name(), "Heal");
    }

    /// 多 handler 按 priority 降序排列
    struct LowPriorityHandler;
    impl TriggerHandler for LowPriorityHandler {
        fn trigger_type(&self) -> Trigger {
            Trigger::AfterDamaged
        }
        fn handle(&self, _ctx: &TriggerContext) -> Vec<EffectDef> {
            vec![EffectDef::Cleanse]
        }
        fn priority(&self) -> i32 {
            1
        }
    }

    struct HighPriorityHandler;
    impl TriggerHandler for HighPriorityHandler {
        fn trigger_type(&self) -> Trigger {
            Trigger::AfterDamaged
        }
        fn handle(&self, _ctx: &TriggerContext) -> Vec<EffectDef> {
            vec![EffectDef::Heal { amount: 10 }]
        }
        fn priority(&self) -> i32 {
            100
        }
    }

    #[test]
    fn handlers_sorted_by_priority_descending() {
        let mut registry = TriggerRegistry::new();
        registry.register(Box::new(LowPriorityHandler));
        registry.register(Box::new(HighPriorityHandler));

        let ctx = TriggerContext {
            trigger: Trigger::AfterDamaged,
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: None,
            damage_dealt: Some(10),
            is_critical: false,
            terrain_id: "plain".to_string(),
        };
        let effects = registry.dispatch(&ctx);
        assert_eq!(effects.len(), 2);
        // High priority handler runs first → Heal first, then Cleanse
        assert_eq!(effects[0].type_name(), "Heal");
        assert_eq!(effects[1].type_name(), "Cleanse");
    }

    /// dispatch 对不匹配的 trigger 类型返回空
    #[test]
    fn dispatch_wrong_trigger_type() {
        let mut registry = TriggerRegistry::new();
        registry.register(Box::new(MockHealHandler { amount: 5 }));

        let ctx = TriggerContext {
            trigger: Trigger::TurnStart,
            source: Entity::from_bits(1),
            target: Entity::from_bits(2),
            skill_id: None,
            damage_dealt: None,
            is_critical: false,
            terrain_id: "plain".to_string(),
        };
        let effects = registry.dispatch(&ctx);
        assert!(effects.is_empty());
    }
}
