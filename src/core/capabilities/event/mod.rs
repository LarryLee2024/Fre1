//! event — Event（事件）能力领域
//!
//! 系统间传递结构化数据的统一事件总线。Domain 间通信的唯一通道。
//! 提供发布-订阅模式的事件路由、循环检测、投递隔离。
//!
//! 分层结构：
//! - foundation/: 纯数据类型（EventTag, GameplayEvent, EventPayload）
//! - mechanism/:  EventBus 全局 Resource（订阅管理、事件分发）
//! - events/:     监控调试事件（EventPublished, EventDelivered, etc.）
//!
//! 详见 docs/02-domain/capabilities/event_domain.md

pub mod events;
// [ADR-045] pub(crate) — 基础类型，crate 内共享，外部不可访问
pub(crate) mod foundation;
// [ADR-045] pub(crate) — 机制实现，crate 内共享，外部不可访问
pub(crate) mod mechanism;

mod plugin;
pub use plugin::*;

#[cfg(test)]
mod tests;
