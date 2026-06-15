//! 技能领域错误（DomainError）
//!
//! 覆盖技能系统中的预期异常：技能配置缺失、技能未就绪等。
//! 错误码格式：S + 三位序号
//!
//! 🟩 与 SkillUseError（RuleFailure）分离：
//!   - SkillError  → 配置缺失、系统异常（DomainError）
//!   - SkillUseError → 条件不满足、冷却中（RuleFailure）
//!
//! S001-S009: 通用错误
//! S010-S019: 状态错误

use crate::shared::ids::AbilityId;
use thiserror::Error;

/// 技能领域错误枚举
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SkillError {
    /// S004: 技能配置不存在
    #[error("S004: 技能配置不存在: {skill_id}")]
    SkillNotFound { skill_id: AbilityId },

    /// S005: 技能未就绪（未学习或已被禁用）
    #[error("S005: 技能未就绪: {skill_id}, 原因={reason}")]
    SkillNotReady {
        skill_id: AbilityId,
        reason: SkillNotReadyReason,
    },
}

/// 技能未就绪的具体原因
#[derive(Debug, Clone, PartialEq)]
pub enum SkillNotReadyReason {
    /// 未学习
    NotLearned,
    /// 被禁用
    Disabled,
}

impl std::fmt::Display for SkillNotReadyReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotLearned => write!(f, "未学习"),
            Self::Disabled => write!(f, "已被禁用"),
        }
    }
}

/// 技能领域结果类型
pub type SkillResult<T> = Result<T, SkillError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_not_found_包含错误码() {
        let err = SkillError::SkillNotFound {
            skill_id: AbilityId::new("fireball"),
        };
        let msg = err.to_string();
        assert!(msg.contains("S004"));
        assert!(msg.contains("fireball"));
    }

    #[test]
    fn skill_not_ready_包含原因() {
        let err = SkillError::SkillNotReady {
            skill_id: AbilityId::new("ultimate"),
            reason: SkillNotReadyReason::Disabled,
        };
        let msg = err.to_string();
        assert!(msg.contains("S005"));
        assert!(msg.contains("ultimate"));
        assert!(msg.contains("禁用"));
    }

    #[test]
    fn skill_result_类型可用() {
        let ok: SkillResult<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);

        let err: SkillResult<i32> = Err(SkillError::SkillNotFound {
            skill_id: AbilityId::new("test"),
        });
        assert!(err.is_err());
    }
}
