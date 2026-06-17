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
//! 详见 docs/02-domain/event_domain.md

pub mod events;
pub mod foundation;
pub mod mechanism;

mod plugin;
pub use plugin::*;
