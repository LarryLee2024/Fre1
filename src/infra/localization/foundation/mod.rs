//! 基础层 — 纯类型，零 Bevy ECS 依赖。
//!
//! 包含 LocaleId 枚举、LocError 和 Pattern 结构体。
//! 这些类型没有 ECS Resource/Component 或 System 依赖。

pub(crate) mod error;
pub(crate) mod locale_id;
pub(crate) mod pattern;

pub use error::LocError;
pub use locale_id::LocaleId;
pub use pattern::Pattern;
