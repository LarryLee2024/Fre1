//! registry — 注册中心
//!
//! C3 Runtime 的子模块：全局 DefRegistry、IdAllocator、注册校验。
//! 所有 Def 在内容加载时通过 Registry 注册，运行时只读。
//!
//! 详见 docs/04-data/infrastructure/registry_schema.md

pub mod events;
// [ADR-045] pub(crate) — 基础类型，crate 内共享，外部不可访问
pub(crate) mod foundation;
// [ADR-045] pub(crate) — 机制实现，crate 内共享，外部不可访问
pub(crate) mod mechanism;

#[cfg(test)]
mod tests;
