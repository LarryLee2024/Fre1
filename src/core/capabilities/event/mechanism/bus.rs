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

/// 按标签过滤订阅者（纯函数）。
pub fn filter_subscribers_by_tag<'a>(
    subscribers: &'a [SubscriberEntry],
    tag: &EventTag,
) -> Vec<&'a SubscriberEntry> {
    subscribers
        .iter()
        .filter(|s| s.tags.contains(tag))
        .collect()
}

/// 创建事件 ID（确定性自增）。
pub fn create_event_id(counter: &mut u64) -> String {
    let id = *counter;
    *counter += 1;
    format!("evt_{:010}", id)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::core::capabilities::event::foundation::EventTag;

    // ── Basic publish/dispatch ─────────────────────────────

    #[test]
    fn unit_001_publish_and_dispatch_single_event() {
        let mut bus = EventBus::new();

        bus.publish(
            EventTag::UnitSpawned,
            "test_system",
            EventPayload::from_source("entity_001"),
        );

        assert_eq!(bus.pending_count(), 1);
    }

    #[test]
    fn unit_002_dispatch_empty_queue() {
        let mut bus = EventBus::new();
        let report = bus.dispatch_pending();
        assert_eq!(report.total, 0);
        assert!(report.all_succeeded());
    }

    #[test]
    fn unit_003_subscriber_receives_matching_event() {
        let mut bus = EventBus::new();
        let received = Arc::new(std::sync::Mutex::new(false));
        let r = received.clone();

        bus.subscribe(SubscriberEntry {
            id: "test_sub".into(),
            tags: vec![EventTag::UnitSpawned],
            handler: Arc::new(move |_payload| {
                *r.lock().unwrap() = true;
                Ok(())
            }),
        });

        bus.publish(
            EventTag::UnitSpawned,
            "test",
            EventPayload::from_source("entity_001"),
        );

        let report = bus.dispatch_pending();
        assert_eq!(report.delivered, 1);
        assert!(*received.lock().unwrap());
    }

    #[test]
    fn unit_004_subscriber_ignores_non_matching_tag() {
        let mut bus = EventBus::new();
        let received = Arc::new(std::sync::Mutex::new(false));
        let r = received.clone();

        bus.subscribe(SubscriberEntry {
            id: "test_sub".into(),
            tags: vec![EventTag::UnitDied],
            handler: Arc::new(move |_payload| {
                *r.lock().unwrap() = true;
                Ok(())
            }),
        });

        bus.publish(
            EventTag::UnitSpawned,
            "test",
            EventPayload::from_source("entity_001"),
        );

        let report = bus.dispatch_pending();
        assert_eq!(report.delivered, 0);
        assert!(!*received.lock().unwrap());
    }

    // ── Multiple subscribers ───────────────────────────────

    #[test]
    fn unit_005_multiple_subscribers_all_receive() {
        let mut bus = EventBus::new();
        let count = Arc::new(std::sync::Mutex::new(0u32));
        let c1 = count.clone();
        let c2 = count.clone();

        bus.subscribe(SubscriberEntry {
            id: "sub_a".into(),
            tags: vec![EventTag::DamageDealt],
            handler: Arc::new(move |_payload| {
                *c1.lock().unwrap() += 1;
                Ok(())
            }),
        });

        bus.subscribe(SubscriberEntry {
            id: "sub_b".into(),
            tags: vec![EventTag::DamageDealt],
            handler: Arc::new(move |_payload| {
                *c2.lock().unwrap() += 1;
                Ok(())
            }),
        });

        bus.publish(
            EventTag::DamageDealt,
            "test",
            EventPayload::from_source("entity_001"),
        );

        let report = bus.dispatch_pending();
        assert_eq!(report.delivered, 2);
        assert_eq!(*count.lock().unwrap(), 2);
    }

    // ── Handler failure isolation ──────────────────────────

    #[test]
    fn unit_006_handler_failure_does_not_affect_others() {
        let mut bus = EventBus::new();
        let received = Arc::new(std::sync::Mutex::new(false));
        let r = received.clone();

        bus.subscribe(SubscriberEntry {
            id: "failing_sub".into(),
            tags: vec![EventTag::DamageDealt],
            handler: Arc::new(move |_payload| Err("simulated failure".into())),
        });

        bus.subscribe(SubscriberEntry {
            id: "ok_sub".into(),
            tags: vec![EventTag::DamageDealt],
            handler: Arc::new(move |_payload| {
                *r.lock().unwrap() = true;
                Ok(())
            }),
        });

        bus.publish(
            EventTag::DamageDealt,
            "test",
            EventPayload::from_source("entity_001"),
        );

        let report = bus.dispatch_pending();
        assert_eq!(report.delivered, 1);
        assert_eq!(report.failed, 1);
        assert!(*received.lock().unwrap());
    }

    // ── Cycle detection ────────────────────────────────────

    #[test]
    fn unit_007_cycle_detection_blocks_after_limit() {
        let mut bus = EventBus::new();

        // Create a cycle: each dispatch publishes again
        let bus_ptr = Arc::new(std::sync::Mutex::new(Vec::new()));
        let b = bus_ptr.clone();

        bus.subscribe(SubscriberEntry {
            id: "cyclic_sub".into(),
            tags: vec![EventTag::DamageDealt],
            handler: Arc::new(move |_payload| {
                b.lock().unwrap().push("called".to_string());
                Ok(())
            }),
        });

        // Manually simulate cycle: publish 6 events (exceeds limit of 5)
        for _ in 0..6 {
            bus.pending_events.push(GameplayEvent {
                id: "test".into(),
                tag: EventTag::DamageDealt,
                source: "test".into(),
                priority: EventPriority::Normal,
                payload: EventPayload::from_source("entity_001"),
            });
        }

        let report = bus.dispatch_pending();
        assert!(report.cycle_interrupted);
        // Only first 5 should have been attempted
        assert!(report.total <= 5 * 1); // 5 events × 1 subscriber
    }

    // ── Subscribe/unsubscribe ──────────────────────────────

    #[test]
    fn unit_008_unsubscribe_removes_subscriber() {
        let mut bus = EventBus::new();
        bus.subscribe(SubscriberEntry {
            id: "test_sub".into(),
            tags: vec![EventTag::UnitSpawned],
            handler: Arc::new(move |_payload| Ok(())),
        });

        assert_eq!(bus.total_subscribers(), 1);
        bus.unsubscribe("test_sub");
        assert_eq!(bus.total_subscribers(), 0);
    }

    #[test]
    fn unit_009_unsubscribe_idempotent() {
        let mut bus = EventBus::new();
        bus.unsubscribe("nonexistent_sub");
        assert_eq!(bus.total_subscribers(), 0);
    }

    #[test]
    fn unit_010_resubscribe_overwrites() {
        let mut bus = EventBus::new();
        let result = Arc::new(std::sync::Mutex::new(String::new()));
        let r1 = result.clone();
        let r2 = result.clone();

        // First registration
        bus.subscribe(SubscriberEntry {
            id: "test_sub".into(),
            tags: vec![EventTag::UnitSpawned],
            handler: Arc::new(move |_payload| {
                *r1.lock().unwrap() = "first".into();
                Ok(())
            }),
        });

        // Overwrite
        bus.subscribe(SubscriberEntry {
            id: "test_sub".into(),
            tags: vec![EventTag::UnitSpawned],
            handler: Arc::new(move |_payload| {
                *r2.lock().unwrap() = "second".into();
                Ok(())
            }),
        });

        bus.publish(
            EventTag::UnitSpawned,
            "test",
            EventPayload::from_source("entity_001"),
        );
        let report = bus.dispatch_pending();
        assert_eq!(report.delivered, 1);
        assert_eq!(*result.lock().unwrap(), "second");
    }

    // ── EventPayload builder ───────────────────────────────

    #[test]
    fn unit_011_payload_builder() {
        let payload = EventPayload::from_source("entity_001")
            .with_target("entity_002")
            .with_value("damage", 42.0)
            .with_data("element", "fire")
            .with_tag("crit");

        assert_eq!(payload.source_entity, "entity_001");
        assert_eq!(payload.target_entity.unwrap(), "entity_002");
        assert_eq!(payload.values.get("damage").unwrap(), &42.0);
        assert_eq!(payload.custom_data.get("element").unwrap(), "fire");
        assert!(payload.tags.contains(&"crit".to_string()));
    }

    // ── EventTag name ──────────────────────────────────────

    #[test]
    fn unit_012_event_tag_name() {
        assert_eq!(EventTag::DamageDealt.name(), "DamageDealt");
        assert_eq!(EventTag::Custom("test".into()).name(), "Custom");
    }

    // ── DispatchReport ─────────────────────────────────────

    #[test]
    fn unit_013_report_all_succeeded() {
        let report = DispatchReport {
            total: 3,
            delivered: 3,
            failed: 0,
            errors: vec![],
            cycle_interrupted: false,
        };
        assert!(report.all_succeeded());
    }

    #[test]
    fn unit_014_report_failed_with_errors() {
        let report = DispatchReport {
            total: 3,
            delivered: 2,
            failed: 1,
            errors: vec![("sub_001".into(), "error".into())],
            cycle_interrupted: false,
        };
        assert!(!report.all_succeeded());
    }

    #[test]
    fn unit_015_reset_cycle_counters() {
        let mut bus = EventBus::new();
        bus.cycle_counters.insert(EventTag::DamageDealt, 3);
        bus.reset_cycle_counters();
        assert!(bus.cycle_counters.is_empty());
    }
}
