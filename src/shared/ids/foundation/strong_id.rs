//! StrongId trait — 所有强类型 ID 的统一接口。
//!
//! 提供统一的 `prefix()` 和 `as_str()` 接口，支持 Registry 泛型约束和跨模块操作。
//!
//! # 实现者
//!
//! - `define_string_id!` 宏为生成的每个 String ID 类型自动实现 `StrongId`
//! - `DefinitionId` 手动实现 `StrongId`
//!
//! # 示例
//!
//! ```ignore
//! use crate::shared::ids::StrongId;
//!
//! assert_eq!(AttributeId::prefix(), "attr");
//! assert_eq!(StrongId::as_str(&AttributeId::new("hp")), "hp");
//! ```

/// Sealed trait — 防止外部实现破坏 StrongId 的不变量。
pub(crate) mod sealed {
    pub trait Sealed {}
}

/// 所有强类型 ID 必须实现的 trait。
///
/// 提供统一的接口以支持 Registry 约束和跨模块泛型操作。
pub trait StrongId:
    sealed::Sealed + std::fmt::Display + std::str::FromStr + std::ops::Deref<Target = str> + Sized
{
    /// 返回类型前缀（如 `"attr"`、`"tag"`、`"abl"`）。
    fn prefix() -> &'static str;

    /// 返回内部值（如 `"hp_max"`、`"abl_000042"`）。
    fn as_str(&self) -> &str;
}
