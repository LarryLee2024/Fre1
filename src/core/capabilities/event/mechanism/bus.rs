//! EventBus — 全局事件路由基础设施
//!
//! 负责接收事件并分发给所有匹配的订阅者。
//! 提供双阶段发布-分发模型：先入队，后批量分发。
//!
//! 详见 docs/02-domain/event_domain.md §5.1-5.5。

use std::collections::HashMap;

use bevy::prelude::*;

use crate::core::capabilities::event::foundation::{
    DispatchReport, EVENT_CYCLE_LIMIT, EventHandler, EventPayload, EventPriority, EventTag,
    GameplayEvent, SubscriberEntry,
};

static NEXT_EVENT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// 全局事件总线 Resource。
///
/// 管理订阅者注册、待分发事件队列、循环检测。
/// 提供 publish（入队）和 dispatch_pending（批量分发）双阶段 API。
#[derive(Resource, Debug)]
pub struct EventBus {
    /// 已注册的订阅者
    subscribers: Vec<SubscriberEntry>,
    /// 待分发的事件队列
    pending_events: Vec<GameplayEvent>,
    /// 循环检测计数器（EventTag → 当前处理链中的触发次数）
    cycle_counters: HashMap<EventTag, u32>,
}

impl EventBus {
    /// 创建一个空的 EventBus。
    pub fn new() -> Self {
        Self {
            subscribers: Vec::new(),
            pending_events: Vec::new(),
            cycle_counters: HashMap::new(),
        }
    }

    // ── 订阅管理 ───────────────────────────────────────────

    /// 注册一个订阅者。
    ///
    /// 不变量 §3.5：不会影响当前正在分发的事件。
    /// 如果订阅者 ID 已存在，覆盖旧注册。
    pub fn subscribe(&mut self, entry: SubscriberEntry) {
        // 移除同 ID 的旧注册
        self.subscribers.retain(|s| s.id != entry.id);
        self.subscribers.push(entry);
    }

    /// 注销一个订阅者（按 ID）。幂等。
    pub fn unsubscribe(&mut self, subscriber_id: &str) {
        self.subscribers.retain(|s| s.id != subscriber_id);
    }

    /// 获取指定事件标签的订阅者数量。
    pub fn subscriber_count(&self, tag: &EventTag) -> usize {
        self.subscribers
            .iter()
            .filter(|s| s.tags.contains(tag))
            .count()
    }

    /// 总订阅者数。
    pub fn total_subscribers(&self) -> usize {
        self.subscribers.len()
    }

    // ── 事件发布 ───────────────────────────────────────────

    /// 发布一个事件（入队）。
    ///
    /// 不变量 §3.1：事件必须有明确来源。
    /// 不变量 §3.2：不影响其他事件的分发。
    ///
    /// 事件不会立即分发，而是加入待分发队列。
    /// 调用 `dispatch_pending()` 批量处理。
    pub fn publish(&mut self, tag: EventTag, source: impl Into<String>, payload: EventPayload) {
        let id = self.next_event_id();
        self.pending_events.push(GameplayEvent {
            id,
            tag,
            source: source.into(),
            priority: EventPriority::Normal,
            payload,
        });
    }

    /// 带优先级发布事件。
    pub fn publish_with_priority(
        &mut self,
        tag: EventTag,
        source: impl Into<String>,
        payload: EventPayload,
        priority: EventPriority,
    ) {
        let id = self.next_event_id();
        self.pending_events.push(GameplayEvent {
            id,
            tag,
            source: source.into(),
            priority,
            payload,
        });
    }

    /// 获取待分发事件数量。
    pub fn pending_count(&self) -> usize {
        self.pending_events.len()
    }

    // ── 事件分发 ───────────────────────────────────────────

    /// 批量分发所有待处理事件。
    ///
    /// 流程（§5.2）：
    /// 1. 按优先级排序（高→低），同优先级 FIFO
    /// 2. 查找匹配的订阅者
    /// 3. 逐个投递，记录成功/失败
    /// 4. 循环检测（不变量 §3.4）
    ///
    /// 返回分发报告。
    pub fn dispatch_pending(&mut self) -> DispatchReport {
        if self.pending_events.is_empty() {
            return DispatchReport::default();
        }

        // 取出所有待分发事件并按优先级排序
        let mut events = std::mem::take(&mut self.pending_events);
        events.sort_by(|a, b| a.priority.cmp(&b.priority));

        let mut report = DispatchReport::default();

        for event in &events {
            // 循环检测
            let cycle_count = self.cycle_counters.entry(event.tag.clone()).or_insert(0);
            if *cycle_count >= EVENT_CYCLE_LIMIT {
                report.cycle_interrupted = true;
                report.errors.push((
                    "EventBus".into(),
                    format!(
                        "cycle detected for event '{}': exceeded limit of {}",
                        event.tag.name(),
                        EVENT_CYCLE_LIMIT
                    ),
                ));
                continue;
            }

            // 查找匹配的订阅者
            let matching: Vec<(EventHandler, String)> = self
                .subscribers
                .iter()
                .filter(|s| s.tags.contains(&event.tag))
                .map(|s| (s.handler.clone(), s.id.clone()))
                .collect();

            report.total += matching.len();

            for (handler, subscriber_id) in &matching {
                match handler(&event.payload) {
                    Ok(()) => {
                        report.delivered += 1;
                    }
                    Err(err_msg) => {
                        report.failed += 1;
                        report.errors.push((subscriber_id.clone(), err_msg.clone()));
                    }
                }
            }

            // 递增循环计数器
            *cycle_count += 1;
        }

        report
    }

    /// 重置所有循环检测计数器（应在每帧/每回合开始时调用）。
    pub fn reset_cycle_counters(&mut self) {
        self.cycle_counters.clear();
    }

    // ── 内部辅助 ───────────────────────────────────────────

    fn next_event_id(&mut self) -> String {
        let id = NEXT_EVENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        format!("evt_{:010}", id)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

// ── 纯函数辅助 ──────────────────────────────────────────────
// 当前无纯函数辅助 —— filter_subscribers_by_tag 和 create_event_id
// 在之前的轮次中已删除（无调用者，Dead Code 清理）。
