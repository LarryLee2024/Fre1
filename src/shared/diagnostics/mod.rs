//! 日志诊断基础设施
//!
//! 提供日志类型编码、分类、关联标识和诊断上下文等纯类型定义。
//! 不包含任何业务逻辑，仅定义数据结构。
//!
//! 详见 `docs/04-data/infrastructure/logging_schema.md`

mod context;
mod correlation;
mod log_category;
mod log_code;
mod observable;

pub use context::DiagnosticContext;
pub use correlation::{ActionId, BattleId, CorrelationId, TurnId};
pub use log_category::LogCategory;
pub use log_code::LogCode;
pub use observable::{FieldCollector, ObservableEvent};
