//! 强类型 ID — 向后兼容重导出
//!
//! 注意：ID 类型已迁移到 `src/shared/ids/`（Phase 1.1）。
//! 新代码应直接使用 `crate::shared::ids::*`。
//! 此模块将在 Phase 2 清理中移除。

pub use crate::shared::ids::{BuffId, ItemId, SkillId, UnitId};
