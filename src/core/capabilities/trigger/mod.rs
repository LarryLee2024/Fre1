//! trigger — Trigger（触发器）能力领域
//!
//! 定义"什么条件下可以激活什么技能"的映射关系。
//! 检测条件触发后通知 Ability 系统创建技能实例。
//!
//! 分层结构：
//! - foundation/: 纯数据类型（TriggerType, TriggerEntry, TriggerFrequency）
//! - mechanism/:  ECS 组件（TriggerContainer）+ 评估器（can_trigger）
//! - events/:     领域事件（TriggerFired, TriggerRegistered, etc.）
//!
//! 详见 docs/02-domain/trigger_domain.md

pub mod events;
pub mod foundation;
pub mod mechanism;

mod plugin;
pub use plugin::*;
