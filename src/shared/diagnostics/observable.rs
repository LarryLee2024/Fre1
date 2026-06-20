//! 可观测事件契约——领域事件与可观测系统之间的正式接口。
//!
//! 任何需要被日志/指标/追踪系统监听的领域事件都应实现此 trait。
//! 这保证了 Observer 可以通过统一方式提取事件的结构化字段。
//!
//! 详见 ADR-052 ObservableEvent。

use std::fmt;

use super::LogCode;

/// 可观测事件——领域事件实现此 trait 后，Observability Facade
/// 可以自动将事件分发到日志、指标、追踪等所有 sink。
///
/// # 实现示例
///
/// ```ignore
/// impl ObservableEvent for LevelUp {
///     fn log_code(&self) -> LogCode {
///         LogCode::PRG002
///     }
/// }
/// ```
pub trait ObservableEvent: fmt::Debug + Send + Sync + 'static {
    /// 返回该事件对应的 LogCode。
    fn log_code(&self) -> LogCode;

    /// 将事件的结构化字段写入 FieldCollector。
    /// Observer 可以通过此方法获取事件的动态字段，无需反射。
    ///
    /// 默认实现不收集任何字段，事件类型可覆盖此方法提供字段。
    fn record_fields(&self, _collector: &mut FieldCollector) {}
}

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
