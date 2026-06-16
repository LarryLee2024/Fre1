//! gameplay_context — 游戏上下文能力领域
//!
//! 跨系统传递的统一数据载体，封装一次游戏行为的所有相关数据。
//! 通过 ContextBuilder 构建，构建完成后不可变。
//!
//! 详见 docs/02-domain/gameplay_context_domain.md

pub mod events;
pub mod foundation;
pub mod mechanism;

mod plugin;
pub use plugin::*;
