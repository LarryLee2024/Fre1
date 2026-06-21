//! Bevy ECS 类型的扩展 trait（EntityCommands、Query）。
//!
//! 这些 trait 在 Bevy 的 ECS 原语之上提供方法语法糖，
//! 内部委托给集成层 Facade 函数。
//!
//! # 设计
//!
//! - 每个 trait 遵循 `*Ext` 命名约定（与 `ContextExt` 一致）。
//! - 方法是 Facade — 调用集成 Facade 函数，而非直接调用 Capabilities。
//! - 扩展方法内部不实现业务逻辑。
//!
//! # 用法
//!
//! ```ignore
//! use crate::core::domains::combat::integration::ext::EntityCommandsExt;
//!
//! fn my_system(mut commands: Commands) {
//!     let mut entity = commands.spawn_empty();
//!     entity.add_buff(my_buff_id);
//!     entity.heal(50);
//! }
//! ```
//!
//! 参见 ADR-060 了解完整理由。

pub mod entity_commands_ext;
pub mod query_ext;

pub use entity_commands_ext::EntityCommandsExt;
pub use query_ext::QueryExt;

#[cfg(test)]
mod tests;
