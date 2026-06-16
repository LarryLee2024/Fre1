//! Fre SRPG — 库根
//!
//! 架构：DDD 纵向三层 (shared/core/infra) + 横切四层 (app/content/tools/modding)
//! 详见 `docs/01-architecture/README.md`

pub mod app;
pub mod content;
pub mod core;
pub mod infra;
pub mod modding;
pub mod shared;

#[cfg(feature = "dev")]
pub mod tools;
