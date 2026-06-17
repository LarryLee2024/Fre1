//! gameplay_context — 游戏上下文能力领域
//!
//! 跨系统传递的统一数据载体，封装一次游戏行为的所有相关数据。
//! 通过 ContextBuilder 构建，构建完成后不可变。
//!
//! 详见 docs/02-domain/gameplay_context_domain.md

pub mod events;
// [ADR-045] pub(crate) — 基础类型，crate 内共享，外部不可访问
pub(crate) mod foundation;
// [ADR-045] pub(crate) — 机制实现，crate 内共享，外部不可访问
pub(crate) mod mechanism;

mod plugin;
pub use plugin::*;

#[cfg(test)]
mod tests;
