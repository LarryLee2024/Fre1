//! 强类型 ID 模块
//!
//! 提供 22 个领域 ID 类型，全部由 `define_id!` 宏生成。
//! 遵循 ADR-030 设计：Display 格式为 `prefix:id_value`，Serde 序列化为完整字符串。
//!
//! # 使用
//! ```ignore
//! use tactical_rpg::shared::ids::AbilityId;
//! let id = AbilityId::new("s_1001");
//! assert_eq!(id.to_string(), "ability:s_1001");
//! ```

mod types;

/// 所有强类型 ID 必须实现的 trait。
///
/// 提供统一的接口以支持 Registry 约束和跨模块泛型操作。
pub trait StrongId:
    std::fmt::Display + std::str::FromStr + std::ops::Deref<Target = str> + Sized
{
    /// 返回 ID 前缀（如 `"ability"`）
    fn prefix() -> &'static str;
    /// 返回内部字符串引用
    fn as_str(&self) -> &str;
}

// Re-export all 22 ID types from types module
pub use types::*;
