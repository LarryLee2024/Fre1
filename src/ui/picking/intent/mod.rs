//! PickIntent — Pointer 事件 → 业务意图转换
//!
//! 将 Bevy 原生 Pointer<Click>/<Over>/<Out> 事件转换为 PickIntent，
//! 供 selection/bridge.rs 消费并触发领域事件。

pub mod click;
pub mod hover;
