//! 强类型 ID 模块
//!
//! 提供所有领域 ID 类型。按职责分层组织：
//!
//! - `foundation/` — 核心抽象（StrongId trait、宏、错误类型）
//! - `types/` — 具体 ID 类型定义（string_ids、definition_id、runtime_id）
//! - `mapping/` — Entity ↔ ID 运行时映射（EntityMapper）
//!
//! Display 格式: `<prefix>:<value>`（如 `attr:attr_000001`）
//! FromStr/Serde 兼容: 同时接受 `<prefix>:<value>` 和裸 `<value>` 格式。
//!
//! # 模块可见性
//!
//! - `pub`: StrongId trait, 所有 ID 类型, 宏, types 子模块
//! - `pub(crate)`: mapping（EntityMapper，Domain 层隔离，仅 crate 内可用）
//! - 遵循 [ADR-045] 可见性策略

pub mod foundation;
pub(crate) mod mapping;
pub mod prelude;
pub mod types;

// Re-export 所有 ID 类型
pub use foundation::StrongId;
pub use types::*;

// Note: 宏 `define_string_id!` 和 `define_numeric_id!` 使用 `#[macro_export]`
// 所以它们自动在 crate 根可用，无需额外 re-export。

#[cfg(test)]
mod tests;
