//! tag — 能力领域
//!
//! 详见 docs/02-domain/capabilities/tag_domain.md

pub mod events;
// [ADR-045] pub(crate) — 基础类型，crate 内共享，外部不可访问
pub(crate) mod foundation;
// [ADR-045] pub(crate) — 机制实现，crate 内共享，外部不可访问
pub(crate) mod mechanism;
// [ADR-045] mod — 内容桥接，私有模块，仅本 Module 调用
mod content;

mod plugin;
pub use plugin::*;

#[cfg(test)]
mod tests;
