//! registry — 注册中心
//!
//! C3 Runtime 的子模块：全局 DefRegistry、IdAllocator、注册校验。
//! 所有 Def 在内容加载时通过 Registry 注册，运行时只读。
//!
//! 详见 docs/04-data/infrastructure/registry_schema.md

pub mod events;
pub mod foundation;
pub mod mechanism;
