//! Event Integration — Combat 域与 Event Capability (EventBus) 的桥接层。
//!
//! 封装 Combat 域的事件发布（回合开始/结束、伤害造成/承受、击杀等）。
//! 替代域自定义 EventWriter，通过 EventBus 统一事件分发与订阅。
//!
//! 详见 ADR-024 §2

mod facade;

pub use crate::core::capabilities::event::mechanism::EventBus;
pub use facade::{CombatEventFacade, CombatEventParam, CombatEventTag};

#[cfg(test)]
mod tests;
