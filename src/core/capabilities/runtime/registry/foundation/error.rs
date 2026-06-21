//! Registry 领域错误枚举。
//!
//! 定义 Def 注册中心操作过程中可能出现的错误类型。

/// Registry 领域错误。
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum RegistryError {
    /// ID 已存在
    #[error("重复的 registry ID: {id}")]
    DuplicateId { id: String },
    /// ID 不存在
    #[error("registry ID 未找到: {id}")]
    IdNotFound { id: String },
    /// ID 格式无效
    #[error("无效的 ID 格式: {id}")]
    InvalidIdFormat { id: String },
    /// 跨 Def 引用断裂
    #[error("引用断裂: {source_id}.{field} → {target}（未找到）")]
    BrokenReference {
        /// 源 Def ID
        source_id: String,
        /// 字段名称
        field: String,
        /// 目标 Def ID
        target: String,
    },
    /// 分配器未注册
    #[error("allocator 未找到: {allocator}")]
    AllocatorNotFound { allocator: String },
}
