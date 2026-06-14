/// Buff 领域错误
///
/// ADR-004 §决策: 分领域错误枚举
/// 覆盖 Buff 系统中的预期异常：配置缺失、实例不存在等。
use crate::shared::ids::BuffId;
use thiserror::Error;

/// Buff 领域错误
///
/// 错误码格式：BF + 三位序号
#[derive(Error, Debug, Clone, PartialEq)]
pub enum BuffError {
    /// BF001: Buff 配置不存在
    #[error("BF001: Buff 配置不存在: {buff_id}")]
    BuffNotFound { buff_id: BuffId },

    /// BF002: Buff 实例不存在
    #[error("BF002: Buff 实例不存在: {instance_id}")]
    InstanceNotFound { instance_id: u64 },

    /// BF003: Buff 叠加超过上限
    #[error("BF003: Buff 叠加超过上限: {buff_id}, 当前 {current}, 上限 {max}")]
    StackOverflow {
        buff_id: BuffId,
        current: u32,
        max: u32,
    },

    /// BF004: 无效的 Buff 目标
    #[error("BF004: 无效的 Buff 目标: 目标无法接受此 Buff")]
    InvalidTarget,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ids::BuffId;

    #[test]
    fn buff_error_配置不存在() {
        let err = BuffError::BuffNotFound {
            buff_id: BuffId::new("poison"),
        };
        assert!(err.to_string().contains("BF001"));
        assert!(err.to_string().contains("poison"));
    }

    #[test]
    fn buff_error_叠加上限() {
        let err = BuffError::StackOverflow {
            buff_id: BuffId::new("berserk"),
            current: 5,
            max: 3,
        };
        let msg = err.to_string();
        assert!(msg.contains("BF003"));
        assert!(msg.contains("5"));
        assert!(msg.contains("3"));
    }
}
