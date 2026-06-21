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

/// Sealed trait — 防止外部实现破坏 ObservableEvent 的不变量。
pub(crate) mod sealed {
    pub trait Sealed {}
}

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
pub trait ObservableEvent: sealed::Sealed + fmt::Debug + Send + Sync + 'static {
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

/// 领域事件标记 trait。
///
/// 所有领域事件代表某个领域内的有意义的业务事件。
/// 实现此标记 trait 可启用自动发现、日志、回放、审计能力，
/// 无需行为继承。
///
/// 任何派生 `Event + Debug + Clone + Send + Sync + 'static` 的 Bevy 事件
/// 都会通过 blanket impl 自动实现此 trait — 无需宏或手动 impl。
/// 领域事件标记 trait。
///
/// 存在原因：所有 DomainEvent 自动获得日志、回放、审计能力，
/// 通过 blanket impl 零样板代码实现，替代 `impl_domain_event!()` 宏。
pub trait DomainEvent: bevy::prelude::Event + fmt::Debug + Clone + Send + Sync + 'static {}

/// Blanket Impl：所有满足 supertrait 约束的 Bevy 事件
/// 自动成为 DomainEvent。这消除了对
/// `impl_domain_event!()` 或 `#[derive(DomainEvent)]` 的需求。
impl<T> DomainEvent for T where T: bevy::prelude::Event + fmt::Debug + Clone + Send + Sync + 'static {}

/// 回放事件标记 trait。
///
/// 实现此 trait 的事件在游戏过程中被记录，在验证时被回放。
/// 回放事件是与回放基础设施本身相关的系统事件的子集
/// （如 ReplayStarted、RecordingCompleted）。
///
/// # 标记 Trait 与分类 Trait
///
/// 这是纯标记 trait：不携带行为、不创建层级，
/// 仅作为自动注册系统的注册标签。
/// 回放事件标记 trait。
///
/// 存在原因：回放基础设施自身的事件（ReplayStarted、RecordingCompleted 等）需要与业务领域事件区分，
/// 避免回放系统误处理业务事件。
pub trait ReplayEvent {}

/// 审计事件标记 trait。
///
/// 实现此 trait 的事件被记录到审计日志中，
/// 用于合规、调试和事后分析。
///
/// # 标记 Trait 与分类 Trait
///
/// 这是纯标记 trait：不携带行为、不创建层级，
/// 仅作为自动注册系统的注册标签。
/// 审计事件标记 trait。
///
/// 存在原因：合规/调试/事后分析需要记录关键操作轨迹，
/// 审计事件与业务事件分离，确保审计日志独立于业务日志。
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
