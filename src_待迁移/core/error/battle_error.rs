/// 战斗领域错误
///
/// ADR-004 §决策: 分领域错误枚举
/// 覆盖战斗管线中的预期异常：技能配置缺失、目标无效、伤害计算异常等。
use bevy::prelude::Entity;
use thiserror::Error;

/// 战斗领域错误
///
/// 错误码格式：B + 三位序号
#[derive(Error, Debug, Clone, PartialEq)]
pub enum BattleError {
    /// B001: 技能配置不存在
    #[error("B001: 技能配置不存在: {skill_id}")]
    SkillNotFound { skill_id: String },

    /// B002: 目标实体不存在
    #[error("B002: 目标实体不存在: {target:?}")]
    TargetNotFound { target: Entity },

    /// B003: 伤害计算溢出
    #[error("B003: 伤害计算溢出: {damage}")]
    DamageOverflow { damage: f32 },

    /// B004: 攻击者不存在
    #[error("B004: 攻击者实体不存在: {attacker:?}")]
    AttackerNotFound { attacker: Entity },

    /// B005: 效果队列为空
    #[error("B005: 效果队列为空，无法执行")]
    EmptyEffectQueue,

    /// B006: 无效的攻击意图
    #[error("B006: 无效的攻击意图: 源={from:?} 目标={target:?}")]
    InvalidCombatIntent { from: Entity, target: Entity },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battle_error_错误消息格式() {
        let err = BattleError::SkillNotFound {
            skill_id: "fireball".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("B001"));
        assert!(msg.contains("fireball"));
    }

    #[test]
    fn battle_error_相等性() {
        let a = BattleError::EmptyEffectQueue;
        let b = BattleError::EmptyEffectQueue;
        assert_eq!(a, b);
    }

    #[test]
    fn battle_error_不同变体不等() {
        let a = BattleError::EmptyEffectQueue;
        let b = BattleError::DamageOverflow { damage: 9999.0 };
        assert_ne!(a, b);
    }
}
