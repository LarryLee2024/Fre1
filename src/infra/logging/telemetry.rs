//! 可观测性门面（Observability Facade）——Observer 的唯一埋点接口。
//!
//! # 职责
//!
//! 封装日志 + 度量调用，自动从 LogCode 派生 target，消除 info!() 中的 target 重复。
//! Observer 调用 `emit_info!`/`emit_warn!`/`emit_debug!` 后不需要再手动调用 `metrics::record` 或写 `target`。
//!
//! # 演进路线
//!
//! - **L1（当前）**：Observer 调用 `emit_info!(LogCode, fields..., "msg")`，宏内部
//!   调用 `telemetry::record(code)` 记录度量 + `tracing::info!()` 输出日志。
//! - **L2（未来）**：Observer 调用 `telemetry::record_event(&event)`，内部统一分发到
//!   LoggerSink / MetricSink / ReplaySink / AuditSink。
//!
//! # L1 使用示例
//!
//! ```ignore
//! use crate::shared::diagnostics::LogCode;
//!
//! #[tracing::instrument(skip_all, target = "domain.progression", fields(
//!     code = ?LogCode::PRG002,
//!     event = "level_up",
//! ))]
//! fn on_level_up(trigger: On<LevelUp>) {
//!     let e = trigger.event();
//!     emit_info!(
//!         LogCode::PRG002,
//!         entity = ?e.entity,
//!         old = e.old_level,
//!         new = e.new_level,
//!         "角色升级",
//!     );
//! }
//! ```
//!
//! # L2 使用示例（未来）
//!
//! ```ignore
//! #[tracing::instrument(skip_all, target = "domain.progression")]
//! fn on_level_up(trigger: On<LevelUp>) {
//!     telemetry::record_event(trigger.event());
//! }
//! ```

use crate::shared::diagnostics::{LogCode, ObservableEvent};

/// 记录一次可观测性事件（仅 metrics，无日志输出）。
/// 在 emit_info!/emit_warn!/emit_debug! 宏内部自动调用，通常不需要直接使用。
pub fn record(code: LogCode) {
    crate::infra::logging::metrics::record(code);
}

/// 通过 ObservableEvent 记录一次可观测性事件（L2 入口，当前仅 metrics）。
///
/// 这是未来统一分发入口的雏形——
/// 最终 `record_event()` 会将事件同时发送到 LoggerSink / MetricSink / ReplaySink / AuditSink。
///
/// 当前（L1）Observer 仍使用 emit_info!/emit_warn!，record_event 仅记录 metrics。
/// 当所有 sink 就绪后，Observer 可以从此函数作为唯一入口。
pub fn record_event(event: &impl ObservableEvent) {
    record(event.log_code());
}

// ════════════════════════════════════════════
// L1 宏：emit_info! / emit_warn! / emit_debug!
// ════════════════════════════════════════════
//
// target 由 `#[instrument]` span 传递，宏内不再指定 target。
// 这是有意为之：LogCode 只负责事件编码，不负责路由（见 Domain::target）。

/// 统一 INFO 日志 + 度量入口。
/// Observer 的唯一 INFO 级别埋点方式。
///
/// # 用法
///
/// ```ignore
/// emit_info!(LogCode::PRG002, entity = ?e.entity, old = e.old_level, "角色升级");
/// ```
///
/// 内部自动完成：
/// 1. `telemetry::record(LogCode)` — 度量计数
/// 2. `tracing::info!(fields...)` — 结构化日志
///
/// target 继承自外层 `#[instrument]` span，无需重复指定。
#[macro_export]
macro_rules! emit_info {
    ($code:expr, $($arg:tt)*) => {
        {
            $crate::infra::logging::telemetry::record($code);
            ::tracing::info!($($arg)*);
        }
    };
}

/// 统一 WARN 日志 + 度量入口。
/// Observer 的唯一 WARN 级别埋点方式。
///
/// 用法同 emit_info!：
/// ```ignore
/// emit_warn!(LogCode::EFF004, def_id = ?e.def_id, immune_tag = ?e.immune_tag, "效果被免疫");
/// ```
#[macro_export]
macro_rules! emit_warn {
    ($code:expr, $($arg:tt)*) => {
        {
            $crate::infra::logging::telemetry::record($code);
            ::tracing::warn!($($arg)*);
        }
    };
}

/// 统一 DEBUG 日志 + 度量入口。
/// Observer 的唯一 DEBUG 级别埋点方式。
///
/// 用法同 emit_info!：
/// ```ignore
/// emit_debug!(LogCode::EFF003, instance_id = ?e.instance_id, tick = e.tick_number, "效果 Tick");
/// ```
#[macro_export]
macro_rules! emit_debug {
    ($code:expr, $($arg:tt)*) => {
        {
            $crate::infra::logging::telemetry::record($code);
            ::tracing::debug!($($arg)*);
        }
    };
}
