//! 强类型 ID 模块
//!
//! 提供所有领域 ID 类型。每个 ID 类型由宏生成，确保行为一致。
//!
//! Display 格式: `<prefix>:<value>`（如 `attr:attr_000001`）
//! FromStr/Serde 兼容: 同时接受 `<prefix>:<value>` 和裸 `<value>` 格式。
//!
//! # 使用
//!
//! ## String ID（配置表标识）
//! ```ignore
//! use crate::shared::ids::AbilityId;
//! let id = AbilityId::new("abl_1001");
//! assert_eq!(id.to_string(), "abl:abl_1001");
//! ```
//!
//! ## Runtime ID（运行时实例标识，带 Generation 保护）
//! ```ignore
//! use crate::shared::ids::runtime_id::{RuntimeId, RuntimeIdAllocator};
//!
//! let mut allocator = RuntimeIdAllocator::new();
//! let id1 = allocator.alloc();  // RuntimeId { index: 0, generation: 0 }
//! allocator.free(id1);
//! let id2 = allocator.alloc();  // RuntimeId { index: 0, generation: 1 }
//!
//! // 旧引用 id1 的 generation 不匹配，可以安全检测
//! assert!(id1.is_stale(&id2));
//! ```

pub mod runtime_id;
// [ADR-045] pub(crate) — EntityMapper 用于 Domain 层隔离 Entity，crate 内共享
pub(crate) mod entity_mapper;
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
