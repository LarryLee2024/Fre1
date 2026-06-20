//! metrics — 日志度量统计
//!
//! 提供全局度量计数器，Observer 通过 `metrics::record(LogCode::BAT001)` 调用。
//! `MetricsCollector` Resource + `metrics_flush_system` 定期输出摘要。
//!
//! 对应 ADR-052 §"领域事件 → 日志的四路消费"中的 Metrics 消费端。

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use bevy::prelude::*;

use crate::infra::replay::resources::FrameCounter;
use crate::shared::diagnostics::LogCode;

// ════════════════════════════════════════════
// 全局计数器（Observer 中调用，无需 World 访问）
// ════════════════════════════════════════════

static GLOBAL_COUNTERS: LazyLock<Mutex<HashMap<LogCode, u64>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 记录一次事件触发。
pub fn record(code: LogCode) {
    if let Ok(mut map) = GLOBAL_COUNTERS.lock() {
        *map.entry(code).or_insert(0) += 1;
    }
}

/// 读取 LogCode 的当前计数值。
pub fn count(code: LogCode) -> u64 {
    GLOBAL_COUNTERS
        .lock()
        .ok()
        .and_then(|m| m.get(&code).copied())
        .unwrap_or(0)
}

/// 读取并清除计数器（供 MetricsCollector 快照用）。
fn drain() -> HashMap<LogCode, u64> {
    GLOBAL_COUNTERS
        .lock()
        .ok()
        .map(|mut m| std::mem::take(&mut *m))
        .unwrap_or_default()
}

// ════════════════════════════════════════════
// Resource：周期快照 + 增量摘要
// ════════════════════════════════════════════

/// 度量收集器 Bevy Resource。
///
/// 定期从全局计数器取增量，输出 DEBUG 摘要日志。
#[derive(Resource, Reflect)]
#[reflect(Resource)]
#[derive(Default)]
pub struct MetricsCollector {
    last_summary_frame: u64,
    lifetime_total: u64,
}

impl MetricsCollector {
    /// 取增量并输出摘要 DEBUG 日志。
    pub fn flush_summary(&mut self, frame: u64) {
        let delta = drain();

        if delta.is_empty() {
            self.last_summary_frame = frame;
            return;
        }

        let delta_total: u64 = delta.values().sum();
        self.lifetime_total += delta_total;

        let mut by_prefix: HashMap<&str, (u32, u64)> = HashMap::new();
        for (code, count) in &delta {
            let prefix = &code.code()[..3];
            let entry = by_prefix.entry(prefix).or_insert((0, 0));
            entry.0 += 1;
            entry.1 += count;
        }

        let detail: String = by_prefix
            .into_iter()
            .map(|(prefix, (types, count))| format!("{}:{}x{}", prefix, types, count))
            .collect::<Vec<_>>()
            .join(" ");

        tracing::debug!(target: "logging", 
            frame = frame,
            delta_events = delta_total,
            total_events = self.lifetime_total,
            detail = detail,
            "[Metrics] 汇总"
        );

        self.last_summary_frame = frame;
    }
}

// ════════════════════════════════════════════
// System
// ════════════════════════════════════════════

/// 每 60 帧输出一次度量摘要。
pub fn metrics_flush_system(mut collector: ResMut<MetricsCollector>, frame: Res<FrameCounter>) {
    if frame.0.is_multiple_of(60) {
        collector.flush_summary(frame.0);
    }
}
