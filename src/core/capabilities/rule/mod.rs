//! rule — Rule（数据驱动规则）能力领域
//!
//! 统一的"条件→效果"规则引擎。
//! 所有领域的条件触发逻辑都可以通过 RuleDef 表达。
//!
//! 分层结构：
//! - foundation/: 纯数据类型（RuleDef, RuleEffect, RuleModifierOp）
//! - mechanism/:  规则评估引擎（evaluate_rules, evaluate_single_rule）
//!
//! 职责边界：
//! - 负责：规则的定义、评估、效果输出
//! - 不负责：条件评估（委托给 Condition 领域）、效果执行（委托给 Effect/Modifier 领域）

pub(crate) mod foundation;
pub(crate) mod mechanism;

pub use foundation::*;
pub use mechanism::*;

#[cfg(test)]
mod tests;
