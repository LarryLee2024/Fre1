//! 横切4: Modding — Mod 扩展层（跨层聚合）
//!
//! Mod 加载沙箱 / Mod API 稳定层 / 版本兼容检查。
//!
//! 详见 `docs/01-architecture/README.md` §3.5

mod modding_plugin;

pub use modding_plugin::ModdingPlugin;
