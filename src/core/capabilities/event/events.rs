//! Event 领域事件（调试/监控）
//!
//! Event 领域自身的事件仅用于调试和监控（避免业务循环），
//! 仅在 dev-tools 模式下生效。
//!
//! 详见 docs/02-domain/event_domain.md §6。

use bevy::prelude::*;

/// 事件被发布到 EventBus 时触发（调试用）。
#[derive(Event, Debug, Clone)]
pub struct EventPublished {
    /// 事件标签
    pub event_tag: String,
    /// 来源标识
    pub source: String,
    /// 分发优先级
    pub priority: String,
    /// 时间戳
    pub timestamp: u64,
}

/// 事件成功投递到订阅者时触发（调试用）。
#[derive(Event, Debug, Clone)]
pub struct EventDelivered {
    /// 事件标签
    pub event_tag: String,
    /// 订阅者标识
    pub subscriber_id: String,
    /// 延迟（处理时长）
    pub latency_ms: u64,
}

/// 事件投递到订阅者失败时触发（用于监控告警）。
#[derive(Event, Debug, Clone)]
pub struct EventDeliveryFailed {
    /// 事件标签
    pub event_tag: String,
    /// 订阅者标识
    pub subscriber_id: String,
    /// 错误信息
    pub error_message: String,
}

/// 检测到事件循环触发时触发（严重告警）。
#[derive(Event, Debug, Clone)]
pub struct EventCycleDetected {
    /// 循环的事件标签
    pub event_tag: String,
    /// 循环深度
    pub cycle_depth: u32,
    /// 事件链追踪
    pub chain_trace: String,
}
