//! 统一可观测性入口——Observer 的唯一埋点接口。
//!
//! # 职责
//!
//! 封装当前日志 + 度量（metrics）调用，未来可扩展至 trace/audit/analytics，
//! Observer 无需关心底层实现。
//!
//! # 当前能力
//!
//! - `emit(LogCode)` — 记录一次编码化日志事件（metrics::record）
//!
//! # 未来扩展（规划中）
//!
//! - 自动生成 tracing span / event
//! - audit::record / analytics::record
//! - 消除 Observer 中 `target` 两处重复（span + info!）
//!
//! # 使用示例
//!
//! ```ignore
//! use crate::infra::logging::telemetry;
//! use crate::shared::diagnostics::LogCode;
//!
//! telemetry::emit(LogCode::RCT001);
//! ```

use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 记录一次可观测性事件。
///
/// 等价于 `metrics::record(code)`，但作为统一入口，未来扩展不改变 Observer 调用点。
pub fn emit(code: LogCode) {
    metrics::record(code);
}
