//! Event Integration — combat 域接入 event capability
//!
//! 封装 EventBus 的战斗相关操作，
//! 替代域自定义 EventWriter，统一事件分发。

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::capabilities::event::foundation::{EventPayload, EventPriority, EventTag};
use crate::core::capabilities::event::mechanism::EventBus;

// ─── 战斗事件标签 ──────────────────────────────────────────────────

/// 战斗中常用的事件标签。
pub enum CombatEventTag {
    /// 回合开始
    TurnStarted,
    /// 回合结束
    TurnEnded,
    /// 伤害造成
    DamageDealt,
    /// 伤害承受
    DamageTaken,
    /// 治疗造成
    HealDealt,
    /// 击杀
    Kill,
    /// 效果应用
    EffectApplied,
    /// 技能激活
    AbilityActivated,
}

impl CombatEventTag {
    /// 转换为 EventTag。
    pub fn to_event_tag(&self) -> EventTag {
        match self {
            CombatEventTag::TurnStarted => EventTag::TurnStarted,
            CombatEventTag::TurnEnded => EventTag::TurnEnded,
            CombatEventTag::DamageDealt => EventTag::DamageTaken, // 复用
            CombatEventTag::DamageTaken => EventTag::DamageTaken,
            CombatEventTag::HealDealt => EventTag::Healed,
            CombatEventTag::Kill => EventTag::Custom("Kill".to_string()),
            CombatEventTag::EffectApplied => EventTag::BuffApplied,
            CombatEventTag::AbilityActivated => EventTag::AbilityUsed,
        }
    }
}

// ─── Facade ────────────────────────────────────────────────────────

/// 战斗事件 Facade — 封装 EventBus 的战斗相关操作。
pub struct CombatEventFacade;

impl CombatEventFacade {
    /// 发布一个战斗事件。
    pub fn publish(
        bus: &mut EventBus,
        tag: CombatEventTag,
        source: impl Into<String>,
        payload: EventPayload,
        commands: &mut Commands,
    ) {
        bus.publish(tag.to_event_tag(), source, payload, commands);
    }

    /// 发布一个高优先级战斗事件。
    pub fn publish_priority(
        bus: &mut EventBus,
        tag: CombatEventTag,
        source: impl Into<String>,
        payload: EventPayload,
        commands: &mut Commands,
    ) {
        bus.publish_with_priority(
            tag.to_event_tag(),
            source,
            payload,
            EventPriority::High,
            commands,
        );
    }
}

// ─── SystemParam ───────────────────────────────────────────────────

/// 战斗事件 SystemParam — 在 System 中便捷访问 EventBus。
#[derive(SystemParam)]
pub struct CombatEventParam<'w> {
    pub bus: ResMut<'w, EventBus>,
}

impl<'w> CombatEventParam<'w> {
    /// 发布一个战斗事件。
    pub fn publish(
        &mut self,
        tag: CombatEventTag,
        source: impl Into<String>,
        payload: EventPayload,
        commands: &mut Commands,
    ) {
        CombatEventFacade::publish(&mut self.bus, tag, source, payload, commands);
    }
}
