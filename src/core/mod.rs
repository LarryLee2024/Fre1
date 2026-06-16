//! L1: Core — 领域规则层
//!
//! 依赖: Shared (L0)
//!
//! 双轴结构:
//! - `capabilities/` — 15 个核心能力领域（通用机制骨架）
//! - `domains/` — 15 个业务子系统（全部玩法复杂度）
//! - `mod_api/` — Mod 稳定 API 层
//!
//! 详见 `docs/01-architecture/README.md` §3.2–3.3

pub mod capabilities;
pub mod core_plugin;
pub mod domains;
pub mod mod_api;

pub use core_plugin::CorePlugin;
