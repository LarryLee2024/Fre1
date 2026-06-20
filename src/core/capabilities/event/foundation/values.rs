//! Event 值对象——订阅者条目

use std::sync::Arc;

use crate::core::capabilities::event::foundation::types::{EventPayload, EventTag};

/// 事件处理回调类型。
///
/// 接收 GameplayEvent 引用，返回 DeliveryStatus。
pub type EventHandler = Arc<dyn Fn(&EventPayload) -> Result<(), String> + Send + Sync>;

/// 订阅者条目。
#[derive(Clone)]
pub struct SubscriberEntry {
    /// 订阅者唯一标识
    pub id: String,
    /// 订阅的事件标签列表
    pub tags: Vec<EventTag>,
    /// 处理回调
    pub handler: EventHandler,
}

impl std::fmt::Debug for SubscriberEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubscriberEntry")
            .field("id", &self.id)
            .field("tags", &self.tags)
            .field("handler", &"<fn>")
            .finish()
    }
}

/// 单次事件分发的报告。
#[derive(Debug, Clone, Default)]
pub struct DispatchReport {
    /// 尝试投递数
    pub total: usize,
    /// 成功投递数
    pub delivered: usize,
    /// 失败投递数
    pub failed: usize,
    /// 失败详情
    pub errors: Vec<(String, String)>, // (subscriber_id, error_message)
    /// 是否因循环保护被中断
    pub cycle_interrupted: bool,
}

impl DispatchReport {
    /// 无失败且无循环中断才是完全成功。failed>0 或 cycle_interrupted=true 均返回 false。
    pub fn all_succeeded(&self) -> bool {
        self.failed == 0 && !self.cycle_interrupted
    }
}
