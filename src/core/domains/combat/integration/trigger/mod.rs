//! Trigger Integration — Combat 域与 Trigger Capability 的 Anti-Corruption Layer。
//!
//! 封装 Trigger Capability 的战斗相关操作（触发器注册、触发、频率限制检查）。
//! Systems 通过 CombatTriggerFacade / CombatTriggerParam 交互，不直接访问 TriggerContainer。
//!
//! 详见 ADR-024 §2

mod facade;
mod system_param;

pub use facade::{CombatTriggerFacade, CombatTriggerType};
pub use system_param::CombatTriggerParam;

#[cfg(test)]
mod tests;
