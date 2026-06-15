//! Buff 领域错误
//!
//! 覆盖 Buff 系统中的预期异常：配置缺失、实例不存在等。
//! 错误码格式：BF + 三位序号
//!
//! BF001-BF009: 通用错误
//! BF010-BF019: 叠加相关
//! BF020-BF029: 目标相关

use crate::shared::ids::EffectId;
use thiserror::Error;

/// Buff 领域错误枚举
#[derive(Error, Debug, Clone, PartialEq)]
pub enum BuffError {
    /// BF001: Buff 配置不存在
    #[error("BF001: Buff 配置不存在: {buff_id}")]
    BuffNotFound { buff_id: EffectId },

    /// BF002: Buff 实例不存在
    #[error("BF002: Buff 实例不存在: instance_id={instance_id}")]
    InstanceNotFound { instance_id: u64 },

    /// BF003: Buff 叠加超过上限
    #[error("BF003: Buff 叠加超过上限: {buff_id}, 当前 {current}, 上限 {max}")]
    StackOverflow {
        buff_id: EffectId,
        current: u32,
        max: u32,
    },

    /// BF020: 无效的 Buff 目标
    #[error("BF020: 无效的 Buff 目标: target={target}, buff={buff_id}, 原因={reason}")]
    InvalidTarget {
        /// 目标 ID
        target: String,
        /// Buff ID
        buff_id: EffectId,
        /// 失败原因
        reason: InvalidTargetReason,
    },
}

/// 无效目标的具体原因
#[derive(Debug, Clone, PartialEq)]
pub enum InvalidTargetReason {
    /// 目标免疫此 Buff
    Immune,
    /// 目标已有同类 Buff 且不可叠加
    AlreadyPresent,
    /// 目标已死亡
    Dead,
    /// 阵营不匹配
    FactionMismatch,
    /// 其他原因
    Other(String),
}

impl std::fmt::Display for InvalidTargetReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Immune => write!(f, "免疫"),
            Self::AlreadyPresent => write!(f, "已存在同类Buff"),
            Self::Dead => write!(f, "目标已死亡"),
            Self::FactionMismatch => write!(f, "阵营不匹配"),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

/// Buff 领域结果类型
pub type BuffResult<T> = Result<T, BuffError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buff_not_found_包含错误码() {
        let err = BuffError::BuffNotFound {
            buff_id: EffectId::new("poison"),
        };
        assert!(err.to_string().contains("BF001"));
        assert!(err.to_string().contains("poison"));
    }

    #[test]
    fn stack_overflow_包含详情() {
        let err = BuffError::StackOverflow {
            buff_id: EffectId::new("berserk"),
            current: 5,
            max: 3,
        };
        let msg = err.to_string();
        assert!(msg.contains("BF003"));
        assert!(msg.contains("5"));
        assert!(msg.contains("3"));
    }

    #[test]
    fn invalid_target_包含完整上下文() {
        let err = BuffError::InvalidTarget {
            target: "enemy_01".into(),
            buff_id: EffectId::new("stun"),
            reason: InvalidTargetReason::Immune,
        };
        let msg = err.to_string();
        assert!(msg.contains("BF020"));
        assert!(msg.contains("stun"));
        assert!(msg.contains("免疫"));
    }

    #[test]
    fn buff_result_类型可用() {
        let ok: BuffResult<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);

        let err: BuffResult<i32> = Err(BuffError::BuffNotFound {
            buff_id: EffectId::new("test"),
        });
        assert!(err.is_err());
    }
}
