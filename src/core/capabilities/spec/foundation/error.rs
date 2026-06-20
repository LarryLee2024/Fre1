//! Spec 领域错误。
//!
//! 定义 Spec 生命周期管理过程中的各类错误。

/// Spec 相关错误。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SpecError {
    /// Def 未注册
    #[error("def '{0}' is not registered")]
    DefNotRegistered(String),
    /// 等级越界
    #[error("level {level} out of range [{min}, {max}]")]
    LevelOutOfRange { level: u8, min: u8, max: u8 },
    /// 重复 Spec（同一实体已有同 Def 的 Spec）
    #[error("duplicate spec for def '{def_id}': existing spec '{spec_id}'")]
    DuplicateSpec { def_id: String, spec_id: String },
    /// Spec 不存在（移除不存在的 Spec）
    #[error("spec '{0}' not found")]
    SpecNotFound(String),
    /// Spec 当前有活跃 Instance，不允许操作
    #[error("spec '{0}' has active instances, cannot modify")]
    ActiveInstanceExists(String),
}
