//! 核心抽象层 — StrongId trait、宏定义、错误类型
//!
//! `foundation/` 是 `shared/ids/` 模块的基石，提供 ID 系统所需的所有抽象：
//! - `strong_id.rs`: StrongId trait（所有 String ID 的统一接口）
//! - `macros.rs`: `define_string_id!` 和 `define_numeric_id!` 宏
//! - `errors.rs`: `IdFormatError`、`IdAllocationError` 等错误类型
//!
//! 本模块零依赖，仅使用 Rust 标准库。

mod errors;
mod macros;
pub(crate) mod strong_id;

pub use errors::*;
pub use macros::*;
pub use strong_id::*;
