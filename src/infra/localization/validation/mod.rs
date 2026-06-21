//! 本地化的启动验证和运行时审计。
//!
//! 包含启动时的键完整性验证器和周期性覆盖率审计器。

pub(crate) mod audit;
pub(crate) mod validator;

pub use audit::{AuditTimer, audit_system};
pub use validator::validation_system;
