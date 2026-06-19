//! targeting — Targeting（目标选择）能力领域
//!
//! 目标选择机制，定义技能/效果作用于哪些目标及如何筛选。
//! 是技能流程的关键环节——Ability 激活后调用 Targeting 选择目标。
//!
//! 分层结构：
//! - foundation/: 纯数据类型（TargetType, TargetShape, PriorityRule, TargetingDef, TargetData）
//! - mechanism/:  选择器（TargetingSelector: 筛选→校验→排序→返回 TargetData）
//! - events/:     领域事件（TargetSelected, TargetChanged, NoValidTarget, TargetValidated）
//!
//! 职责边界：
//! - 负责：合法目标的筛选规则、目标数据封装
//! - 不负责：目标选择后的效果执行（Execution）、网格地形数据（Tactical）
//!
//! 详见 docs/02-domain/capabilities/targeting_domain.md

pub mod events;
// [ADR-045] pub(crate) — 基础类型，crate 内共享，外部不可访问
pub(crate) mod foundation;
// [ADR-045] pub(crate) — 机制实现，crate 内共享，外部不可访问
pub(crate) mod mechanism;

mod plugin;
pub use plugin::*;

#[cfg(test)]
mod tests;
