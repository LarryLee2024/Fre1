//! registry — Definition 注册中心
//!
//! 提供全局统一的 Definition 存储和查询基础设施。
//! 管理所有 Def 类型的注册、ID 分配、变更追踪和一致性校验。
//!
//! 详见 docs/04-data/infrastructure/registry_schema.md

mod plugin;
pub mod registry;
pub mod resolver;

pub use plugin::*;
pub use registry::*;
pub use resolver::*;

#[cfg(test)]
mod tests;
