//! execution — Execution（执行计算）能力领域
//!
//! 执行计算引擎，负责按 ExecutionType 分发到对应计算公式，
//! 计算伤害/治疗/直接属性修改等数值结果，输出 ExecutionResult。
//! 是技能流程的关键环节——Ability → Targeting 后调用 Execution 进行数值结算。
//!
//! 分层结构：
//! - foundation/: 纯数据类型（ExecutionType, ExecutionContext, DamageParams, HealParams, ExecutionResult）
//! - mechanism/:  计算调度器（Calculator: 按类型分发 → 调用公式 → 返回结果）
//! - events/:     领域事件（ExecutionCompleted, ExecutionFailed, CustomExecutionRegistered）
//!
//! 职责边界：
//! - 负责：计算调度与分发、上下文准备、结果封装
//! - 不负责：业务公式实现（归 Domains/rules/）、技能生命周期（Ability）、效果管理（Effect）
//!
//! 详见 docs/02-domain/execution_domain.md
//! 详见 docs/04-data/capabilities/execution_schema.md

pub mod events;
// [ADR-045] pub(crate) — 基础类型，crate 内共享，外部不可访问
pub(crate) mod foundation;
// [ADR-045] pub(crate) — 机制实现，crate 内共享，外部不可访问
pub(crate) mod mechanism;

mod plugin;
pub use plugin::*;

#[cfg(test)]
mod tests;
