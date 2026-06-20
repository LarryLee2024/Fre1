//! Condition Integration — Combat 域与 Condition Capability 的 Anti-Corruption Layer。
//!
//! 封装 Condition Capability 的战斗条件评估（如技能释放前提、效果触发条件）。
//! Systems 通过 CombatConditionFacade 查询条件结果，不直接访问 ConditionContainer。
//!
//! 详见 ADR-024 §2

mod facade;

pub use facade::CombatConditionFacade;

#[cfg(test)]
mod tests;
