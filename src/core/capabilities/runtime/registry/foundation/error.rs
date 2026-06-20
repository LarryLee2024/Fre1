//! Registry 领域错误枚举。
//!
//! 定义 Def 注册中心操作过程中可能出现的错误类型。

/// Registry 领域错误。
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RegistryError {
    /// ID 已存在
    #[error("duplicate registry ID: {0}")]
    DuplicateId(String),
    /// ID 不存在
    #[error("registry ID not found: {0}")]
    IdNotFound(String),
    /// ID 格式无效
    #[error("invalid ID format: {0}")]
    InvalidIdFormat(String),
    /// 跨 Def 引用断裂
    #[error("broken reference: {source_id}.{field} \u{2192} {target} (not found)")]
    BrokenReference {
        /// 源 Def ID
        source_id: String,
        /// 字段名称
        field: String,
        /// 目标 Def ID
        target: String,
    },
    /// 分配器未注册
    #[error("allocator not found: {0}")]
    AllocatorNotFound(String),
}
