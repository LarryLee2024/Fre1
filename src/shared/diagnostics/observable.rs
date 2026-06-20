//! 可观测事件契约——领域事件与可观测系统之间的正式接口。
//!
//! 任何需要被日志/指标/追踪系统监听的领域事件都应实现此 trait。
//! 这保证了 Observer 可以通过统一方式提取事件的结构化字段。
//!
//! # 设计要点
//!
//! - `const DOMAIN` — 路由归属，决定 tracing target（`domain.xxx`）
//! - `const CODE` — 事件编码，关联 LogCode 枚举
//! - `fn record_fields()` — 动态字段收集（runtime，可被 Observer 调用）
//!
//! 详见 ADR-052 ObservableEvent。

use std::fmt;

use super::{Domain, LogCode};

/// 可观测事件——领域事件实现此 trait 后，Observability Facade
/// 可以自动将事件分发到日志、指标、追踪等所有 sink。
///
/// # 实现示例
///
/// ```ignore
/// impl ObservableEvent for LevelUp {
///     const DOMAIN: Domain = Domain::Progression;
///     const CODE: LogCode = LogCode::PRG002;
///
///     fn record_fields(&self, collector: &mut FieldCollector) {
///         collector.add_field("entity", format_args!("{:?}", self.entity));
///         collector.add_field("old", self.old_level);
///         collector.add_field("new", self.new_level);
///     }
/// }
/// ```
pub trait ObservableEvent: fmt::Debug + Send + Sync + 'static {
    /// 路由域——决定 tracing target，与 LogCode 的编码职责分离。
    ///
    /// LogCode 只回答"这是什么事件"，Domain 只回答"路由到哪里"。
    const DOMAIN: Domain;

    /// 事件编码——该事件类型对应的 LogCode 枚举变体。
    const CODE: LogCode;

    /// 返回该事件对应的 LogCode。
    ///
    /// 默认实现返回 `Self::CODE`。当需要基于运行时状态选择不同 LogCode 时
    /// 可覆盖此方法（极少数情况）。
    fn log_code(&self) -> LogCode {
        Self::CODE
    }

    /// 将事件的结构化字段写入 FieldCollector。
    /// Observer 可以通过此方法获取事件的动态字段，无需反射。
    ///
    /// 默认实现不收集任何字段，事件类型可覆盖此方法提供字段。
    fn record_fields(&self, _collector: &mut FieldCollector) {}
}

// ─── Marker Trait 事件分类 ────────────────────────────────────
//
// Marker Trait 不携带行为、不创建层级，仅作为注册标签。
// 驱动自动注册系统：事件类型只需 impl 对应 Marker Trait 即可被自动发现。

/// Marker trait for domain events.
///
/// All domain events represent meaningful business occurrences within a domain.
/// Implementing this marker trait enables automatic discovery, logging, replay,
/// and audit capabilities without behavioral inheritance.
///
/// # Marker Trait vs Classification Trait
///
/// This is a pure marker trait: it carries no behavior, creates no hierarchy,
/// and serves only as a registration tag for auto-registration systems.
pub trait DomainEvent {}

/// Marker trait for replay events.
///
/// Events implementing this trait are recorded during gameplay and replayed
/// during verification. Replay events are a subset of system events related
/// to the replay infrastructure itself (e.g., ReplayStarted, RecordingCompleted).
///
/// # Marker Trait vs Classification Trait
///
/// This is a pure marker trait: it carries no behavior, creates no hierarchy,
/// and serves only as a registration tag for auto-registration systems.
pub trait ReplayEvent {}

/// Marker trait for audit events.
///
/// Events implementing this trait are recorded in the audit trail for
/// compliance, debugging, and post-mortem analysis.
///
/// # Marker Trait vs Classification Trait
///
/// This is a pure marker trait: it carries no behavior, creates no hierarchy,
/// and serves only as a registration tag for auto-registration systems.
pub trait AuditEvent {}

/// 结构化字段收集器——Observer 通过此结构收集事件字段。
#[derive(Debug, Default)]
pub struct FieldCollector {
    fields: Vec<(&'static str, String)>,
}

impl FieldCollector {
    /// 添加一个结构化字段（用于 record_fields 实现内部）。
    pub fn add_field(&mut self, key: &'static str, value: impl fmt::Display) {
        self.fields.push((key, value.to_string()));
    }

    /// 获取所有收集的字段。
    pub fn fields(&self) -> &[(&'static str, String)] {
        &self.fields
    }
}
