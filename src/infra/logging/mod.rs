//! 日志基础设施 — 领域事件驱动的日志系统
//!
//! 核心原则：领域层不写日志，通过 Observer 监听 Domain Event 生成日志。
//! 详见 ADR-052。
//!
//! 模块结构：
//! - `plugin.rs` — LoggingPlugin 入口
//! - `observers/` — 各领域事件的日志 Observer
//! - `rate_limit/` — 日志风暴保护（OnceGuard + 宏）
//! - `sinks/` — 日志输出后端（文件/控制台）
//! - `metrics/` — 事件度量统计（MetricsCollector）
//! - `telemetry.rs` — 统一可观测性入口（Observer 的唯一埋点接口）

mod plugin;

pub use plugin::*;

pub mod metrics;
pub mod observers;
pub mod rate_limit;
pub mod sinks;
pub mod telemetry;
