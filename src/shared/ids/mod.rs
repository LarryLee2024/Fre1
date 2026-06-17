//! 强类型 ID 模块
//!
//! 提供所有领域 ID 类型。每个 ID 类型由宏生成，确保行为一致。
//!
//! Display 格式: `<prefix>:<value>`（如 `attr:attr_000001`）
//! FromStr/Serde 兼容: 同时接受 `<prefix>:<value>` 和裸 `<value>` 格式。
//!
//! # 使用
//! ```ignore
//! use crate::shared::ids::AbilityId;
//! let id = AbilityId::new("abl_1001");
//! assert_eq!(id.to_string(), "ability:abl_1001");
//! ```

mod types;

/// 所有强类型 ID 必须实现的 trait。
///
/// 提供统一的接口以支持 Registry 约束和跨模块泛型操作。
pub trait StrongId:
    std::fmt::Display + std::str::FromStr + std::ops::Deref<Target = str> + Sized
{
    fn prefix() -> &'static str;
    fn as_str(&self) -> &str;
}

pub use types::*;

#[cfg(test)]
mod tests;
