//! condition — Condition（条件/限制/免疫）能力领域
//!
//! 统一的条件检查引擎，支持标签、属性、资源三类内置条件检查
//! 和 AND/OR/NOT 逻辑组合，提供免疫检查特殊流程。
//!
//! 分层结构：
//! - foundation/: 纯数据类型（Condition 枚举树, ConditionResult, etc.）
//! - mechanism/:  ECS 组件（ConditionContainer）+ 评估器（evaluate）
//! - events/:     领域事件（ConditionPassed, ConditionFailed, ImmunityTriggered）
//!
//! 详见 docs/02-domain/condition_domain.md

pub mod events;
pub mod foundation;
pub mod mechanism;

mod plugin;
pub use plugin::*;
