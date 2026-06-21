//! UI 数据绑定基础设施
//!
//! 提供 Dirty<T> 变更跟踪机制和 UiBinding 枚举，
//! 将投影更新（UiStore 写入）连接到 Widget 刷新。
//!
//! Dirty<T> 是挂载到 Widget 实体上的 Component。投影更新
//! 将其标记为脏；Widget 系统调用 consume() 来检查并仅在
//! 数据变更时刷新。
//!
//! UiBinding 是标记枚举，标识 UI Node 绑定到哪个 ViewModel 字段。
//! 使用反标记模式避免 Archetype 爆炸。
//!
//! 参见 `docs/06-ui/02-design-system/focus-binding.md` §3-4

pub mod dirty_flag;
pub mod ui_binding;

pub use dirty_flag::Dirty;
pub use ui_binding::UiBinding;

#[cfg(test)]
mod tests;
