//! 横切2: Content — 内容桥接层（数据驱动核心）
//!
//! 从 assets/config/ 加载配置 → 校验 → 注册到 Registry。
//! 依赖: Core + Infra（只做加载/校验/注册）
//!
//! 详见 ADR-047 和 `docs/01-architecture/README.md` §3.5

mod content_plugin;
pub mod def_impls;
pub mod loading;

pub use content_plugin::{ContentPlugin, ContentState, LoadedSpellDefs};
pub use loading::{ConfigError, ContentFile, DefinitionType, ValidationError};

#[cfg(test)]
mod tests;
