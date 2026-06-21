//! Fre SRPG — 库根
//!
//! 架构：DDD 纵向三层 (shared/core/infra) + 横切四层 (app/content/tools/modding)
//! 详见 `docs/01-architecture/README.md`

// 压制预留 dead code：大量 Capability/Domain 的 foundation 和 rules 模块定义了完整的
// 领域概念但尚未接入运行时系统。按 ADR-045 §6.2 标记为"预留非债务"。
#![allow(dead_code)]
// 压制 unused imports：同上原因，各模块的公共 API 导出和 domain rules/ 通配符重导入
// 为预留设计，待接入运行时后自然消除。
#![allow(unused_imports)]

pub mod app;
pub mod content;
pub mod core;
pub mod infra;
pub mod modding;
pub mod shared;
pub mod ui;

#[cfg(feature = "dev")]
pub mod tools;
