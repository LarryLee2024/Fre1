//! Shared 层统一预导入。
//!
//! 所有领域模块使用 `use crate::shared::prelude::*;` 引入最常用的共享基础设施。
//!
//! # 包含内容
//!
//! - 核心诊断类型（Domain, LogCode, ObservableEvent, DomainEvent 等）
//! - 游戏时间（GameTime）
//! - 随机数（DeterministicRng, RngStream）
//! - 规则失败（RuleFailure）
//! - 本地化（LocalizationKey）
//! - 全局常量
//!
//! # 不包含内容
//!
//! - 领域特定的 ID 类型（使用 `use crate::shared::ids::types::*;` 显式导入）
//! - 错误类型（每个 domain 有自己的 Error 枚举）
//! - 测试工具（只在测试模块中导入）

pub use crate::shared::constants::*;
pub use crate::shared::diagnostics::{
    AuditEvent, Domain, DomainEvent, FieldCollector, LogCode, ObservableEvent, ReplayEvent,
};
pub use crate::shared::localization_key::LocalizationKey;
pub use crate::shared::random::{DeterministicRng, RngSeeds, RngStream};
pub use crate::shared::time::GameTime;
pub use crate::shared::traits::RuleFailure;
