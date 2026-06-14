/// 技能领域错误
///
/// ADR-004 §决策: 分领域错误枚举
/// 覆盖技能系统中的预期异常：法力不足、冷却未结束、目标超出范围等。
use crate::shared::ids::SkillId;
use thiserror::Error;

/// 技能领域错误
///
/// 错误码格式：S + 三位序号
#[derive(Error, Debug, Clone, PartialEq)]
pub enum SkillError {
    /// S001: 法力不足
    #[error("S001: 法力不足: 需要 {required}, 当前 {current}")]
    InsufficientMp { required: f32, current: f32 },

    /// S002: 冷却未结束
    #[error("S002: 冷却未结束: 剩余 {turns} 回合")]
    CooldownNotReady { turns: u32 },

    /// S003: 目标超出范围
    #[error("S003: 目标超出范围: 距离 {distance}, 范围 {range}")]
    TargetOutOfRange { distance: u32, range: u32 },

    /// S004: 技能配置不存在
    #[error("S004: 技能配置不存在: {skill_id}")]
    SkillNotFound { skill_id: SkillId },

    /// S005: 技能未就绪（未学习或已被禁用）
    #[error("S005: 技能未就绪: {skill_id}")]
    SkillNotReady { skill_id: SkillId },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ids::SkillId;

    #[test]
    fn skill_error_法力不足消息() {
        let err = SkillError::InsufficientMp {
            required: 30.0,
            current: 10.0,
        };
        let msg = err.to_string();
        assert!(msg.contains("S001"));
        assert!(msg.contains("30"));
        assert!(msg.contains("10"));
    }

    #[test]
    fn skill_error_冷却未结束() {
        let err = SkillError::CooldownNotReady { turns: 2 };
        assert!(err.to_string().contains("2"));
    }

    #[test]
    fn skill_error_技能不存在() {
        let err = SkillError::SkillNotFound {
            skill_id: SkillId::new("fireball"),
        };
        assert!(err.to_string().contains("fireball"));
    }
}
