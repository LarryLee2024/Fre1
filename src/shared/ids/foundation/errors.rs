//! ID 系统专用错误类型。
//!
//! 提供 `IdFormatError`、`IdAllocationError` 等错误类型，用于 ID 格式校验和分配管理。
//! 所有错误类型使用 `thiserror` 派生。

/// ID 格式校验错误。
///
/// 在 `define_string_id!` 的 `checked_new()` 方法中使用（待实现）。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdFormatError {
    /// ID 为空。
    #[error("ID 为空")]
    Empty,

    /// 前缀不匹配。
    #[error("前缀无效: 期望 '{expected}', 实际 '{actual}'")]
    PrefixMismatch {
        /// 期望的前缀。
        expected: &'static str,
        /// 实际值。
        actual: String,
    },

    /// ID 包含非法字符。
    #[error("ID 包含非法字符: {0}")]
    InvalidCharacters(String),

    /// ID 超过最大长度。
    #[error("ID 超过最大长度 {max}: 实际 {len}")]
    TooLong {
        /// 最大长度。
        max: usize,
        /// 实际长度。
        len: usize,
    },
}

/// ID 分配错误。
///
/// 在 `IdAllocator` 中使用（待实现）。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum IdAllocationError {
    /// 该前缀的 ID 空间已耗尽。
    #[error("前缀 '{0}' 的 ID 范围已耗尽")]
    RangeExhausted(&'static str),

    /// 尝试重新使用已废弃的 ID。
    #[error("ID '{0}' 已废弃，不可重用")]
    Deprecated(String),

    /// 预留范围内无可用 ID。
    #[error("前缀 '{0}' 的预留范围内无可用 ID")]
    ReservedRangeExhausted(&'static str),
}

/// ID 创建审计信息（Debug 模式）。
///
/// 记录 ID 的创建来源，帮助追踪"幽灵对象"的来源。
/// 仅在 `debug_assertions` 启用时收集。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdCreationInfo {
    /// 创建者（系统名或模块路径）。
    pub created_by: &'static str,
    /// 创建时的游戏帧号（0 表示不可用）。
    pub frame: u64,
    /// 来源描述（如 "SummonAbility"、"BattleSpawnSystem"）。
    pub source: &'static str,
}

impl IdCreationInfo {
    /// 创建审计信息。
    pub fn new(created_by: &'static str, source: &'static str, frame: u64) -> Self {
        Self {
            created_by,
            source,
            frame,
        }
    }
}
