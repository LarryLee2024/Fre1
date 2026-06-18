//! 召唤领域 — 错误类型

use bevy::prelude::*;

/// 召唤领域错误。
#[derive(Debug, Clone, Event)]
pub enum SummonError {
    /// 召唤位置不可用
    InvalidPosition { reason: String },
    /// 专注冲突
    ConcentrationConflict,
    /// 召唤数量已达上限
    SlotLimitReached { current: u32, max: u32 },
    /// 模板不存在
    TemplateNotFound(String),
    /// 嵌套召唤被禁止
    NestedSummonForbidden,
    /// 召唤者已死亡
    CasterDead,
}
