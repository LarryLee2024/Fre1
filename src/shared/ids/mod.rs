//! 强类型 ID 模块
//!
//! 提供 UnitId, SkillId, BuffId, ItemId 等强类型 ID。
//! 从 src/core/id/ 迁移至此（Phase 1.1）。

mod buff_id;
mod item_id;
mod skill_id;
mod unit_id;

pub use buff_id::BuffId;
pub use item_id::ItemId;
pub use skill_id::SkillId;
pub use unit_id::UnitId;
