//! 战斗领域错误
//!
//! 覆盖战斗管线中的预期异常：技能配置缺失、目标无效、伤害计算异常等。
//! 错误码格式：B + 三位序号
//!
//! B001-B009: 通用错误
//! B010-B019: 单位相关
//! B020-B029: 伤害相关

use crate::shared::ids::{AbilityId, UnitId};
use thiserror::Error;

/// 战斗领域错误枚举
///
/// 🟥 禁止在这里定义 RuleFailure 类型的错误（如"法力不足"）
///     RuleFailure 应使用专门的结果枚举
#[derive(Error, Debug, Clone, PartialEq)]
pub enum BattleError {
    /// B001: 技能配置不存在
    #[error("B001: 技能配置不存在: {skill_id}")]
    SkillNotFound { skill_id: AbilityId },

    /// B002: 目标单位不存在
    #[error("B002: 目标单位不存在: {target}")]
    TargetNotFound { target: UnitId },

    /// B003: 伤害计算溢出
    #[error("B003: 伤害计算溢出: {damage}")]
    DamageOverflow { damage: f32 },

    /// B004: 攻击者不存在
    #[error("B004: 攻击者单位不存在: {attacker}")]
    AttackerNotFound { attacker: UnitId },

    /// B005: 效果队列为空，无法执行
    #[error("B005: 效果队列为空: pipeline={pipeline}, 状态={status:?}")]
    EmptyEffectQueue {
        /// 管线阶段标识
        pipeline: &'static str,
        /// 当前队列状态
        status: QueueStatus,
    },

    /// B006: 无效的攻击意图
    #[error("B006: 无效的攻击意图: 源={from} 目标={target}, 阶段={phase}")]
    InvalidCombatIntent {
        from: UnitId,
        target: UnitId,
        phase: &'static str,
    },
}

/// 队列状态——补充 EmptyEffectQueue 上下文
#[derive(Debug, Clone, PartialEq)]
pub enum QueueStatus {
    /// 队列从未被填充
    NeverPopulated,
    /// 队列已被消费完
    AlreadyConsumed,
    /// 队列被意外清空
    UnexpectedlyCleared,
}

/// 战斗领域结果类型
pub type BattleResult<T> = Result<T, BattleError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 技能未找到_包含错误码和技能id() {
        let err = BattleError::SkillNotFound {
            skill_id: AbilityId::new("fireball"),
        };
        let msg = err.to_string();
        assert!(msg.contains("B001"));
        assert!(msg.contains("fireball"));
    }

    #[test]
    fn 目标未找到_包含unit_id() {
        let err = BattleError::TargetNotFound {
            target: UnitId::new("goblin_01"),
        };
        let msg = err.to_string();
        assert!(msg.contains("B002"));
        assert!(msg.contains("goblin_01"));
    }

    #[test]
    fn 空效果队列_包含上下文() {
        let err = BattleError::EmptyEffectQueue {
            pipeline: "damage",
            status: QueueStatus::NeverPopulated,
        };
        let msg = err.to_string();
        assert!(msg.contains("B005"));
        assert!(msg.contains("damage"));
    }

    #[test]
    fn 无效战斗意图_包含双方单位() {
        let err = BattleError::InvalidCombatIntent {
            from: UnitId::new("warrior"),
            target: UnitId::new("mage"),
            phase: "execution",
        };
        let msg = err.to_string();
        assert!(msg.contains("B006"));
        assert!(msg.contains("warrior"));
        assert!(msg.contains("mage"));
    }

    #[test]
    fn 战斗错误_相等性() {
        let a = BattleError::EmptyEffectQueue {
            pipeline: "test",
            status: QueueStatus::NeverPopulated,
        };
        let b = BattleError::EmptyEffectQueue {
            pipeline: "test",
            status: QueueStatus::NeverPopulated,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn 战斗结果_类型可用() {
        let ok: BattleResult<i32> = Ok(42);
        assert_eq!(ok.unwrap(), 42);

        let err: BattleResult<i32> = Err(BattleError::DamageOverflow { damage: 9999.0 });
        assert!(err.is_err());
    }
}
