//! Ability Integration — Combat 域与 Ability Capability 的 Anti-Corruption Layer。
//!
//! 封装 Ability Capability 的战斗相关操作（激活技能、冷却管理、实例ID生成）。
//! Systems 通过 CombatAbilityFacade / CombatAbilityParam 交互，不知道 Capability 内部类型。
//!
//! 详见 ADR-024 §2

mod facade;

pub use facade::{CombatAbilityFacade, CombatAbilityParam};

#[cfg(test)]
mod tests;
