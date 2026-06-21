//! Spec 领域错误。
//!
//! 定义 Spec 生命周期管理过程中的各类错误。

/// Spec 相关错误。
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SpecError {
    /// Def 未注册
    #[error("def '{0}' 未注册")]
    DefNotRegistered(String),
    /// 等级越界
    #[error("等级 {level} 超出范围 [{min}, {max}]")]
    LevelOutOfRange { level: u8, min: u8, max: u8 },
    /// 重复 Spec（同一实体已有同 Def 的 Spec）
    #[error("def '{def_id}' 重复 spec: 已存在 spec '{spec_id}'")]
    DuplicateSpec { def_id: String, spec_id: String },
    /// Spec 不存在（移除不存在的 Spec）
    #[error("spec '{0}' 未找到")]
    SpecNotFound(String),
    /// Spec 当前有活跃 Instance，不允许操作
    #[error("spec '{0}' 有活跃实例，无法修改")]
    ActiveInstanceExists(String),
}
