//! scheduler — 帧/阶段调度器
//!
//! C3 Runtime 的子模块：定义游戏内时间（GameTime）、帧阶段（TickPhase）、
//! 调度器状态管理和帧推进逻辑。
//!
//! 详见 docs/01-architecture/20-tactical-combat/ADR-021-turn-state-machine.md

pub mod events;
// [ADR-045] pub(crate) — 基础类型，crate 内共享，外部不可访问
pub(crate) mod foundation;
// [ADR-045] pub(crate) — 机制实现，crate 内共享，外部不可访问
pub(crate) mod mechanism;

#[cfg(test)]
mod tests;
