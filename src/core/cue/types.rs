//! Cue 模块类型定义
//!
//! 定义 CueEvent 枚举 — 统一表现事件总线（ADR-026 §四）
//! - CueDamage：伤害表现事件
//! - CueDeath：死亡表现事件
//! - CueHeal：治疗表现事件
//! - CueBuffApply：Buff 施加表现事件
//! - CueBuffRemove：Buff 移除表现事件
//! - CueShield：护盾表现事件
//! - CueSkillCast：技能释放表现事件
//! - CueMovement：移动表现事件

use bevy::ecs::message::Message;
use bevy::prelude::*;

/// 伤害表现事件
#[derive(Message, Debug, Clone, Reflect)]
pub struct CueDamage {
    /// 伤害目标
    pub target: Entity,
    /// 伤害量
    pub amount: i32,
    /// 是否暴击
    pub is_critical: bool,
    /// 攻击者
    pub attacker: Option<Entity>,
}

/// 死亡表现事件
#[derive(Message, Debug, Clone, Reflect)]
pub struct CueDeath {
    /// 死亡实体
    pub entity: Entity,
    /// 击杀者
    pub killer: Option<Entity>,
}

/// 治疗表现事件
#[derive(Message, Debug, Clone, Reflect)]
pub struct CueHeal {
    /// 治疗目标
    pub target: Entity,
    /// 治疗量
    pub amount: i32,
    /// 治疗来源
    pub source: Option<Entity>,
}

/// Buff 施加表现事件
#[derive(Message, Debug, Clone, Reflect)]
pub struct CueBuffApply {
    /// 目标实体
    pub target: Entity,
    /// Buff ID
    pub buff_id: String,
    /// 当前层数
    pub stacks: u32,
}

/// Buff 移除表现事件
#[derive(Message, Debug, Clone, Reflect)]
pub struct CueBuffRemove {
    /// 目标实体
    pub target: Entity,
    /// Buff ID
    pub buff_id: String,
}

/// 护盾表现事件
#[derive(Message, Debug, Clone, Reflect)]
pub struct CueShield {
    /// 目标实体
    pub target: Entity,
    /// 护盾值
    pub amount: i32,
}

/// 技能释放表现事件
#[derive(Message, Debug, Clone, Reflect)]
pub struct CueSkillCast {
    /// 施法者
    pub caster: Entity,
    /// 技能 ID
    pub skill_id: String,
    /// 目标位置（可选）
    pub target_pos: Option<IVec2>,
}

/// 移动表现事件
#[derive(Message, Debug, Clone, Reflect)]
pub struct CueMovement {
    /// 移动实体
    pub entity: Entity,
    /// 起始位置
    pub from: IVec2,
    /// 目标位置
    pub to: IVec2,
}

/// 状态变化表现事件
#[derive(Message, Debug, Clone, Reflect)]
pub struct CueStatusChange {
    /// 目标实体
    pub entity: Entity,
    /// 状态类型
    pub status: CueStatusType,
    /// 是否激活
    pub active: bool,
}

/// 状态类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum CueStatusType {
    Stun,
    Burn,
    Regen,
    Poison,
    Frozen,
    Silence,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cue_damage_construction() {
        let cue = CueDamage {
            target: Entity::from_bits(1),
            amount: 50,
            is_critical: true,
            attacker: Some(Entity::from_bits(2)),
        };
        assert_eq!(cue.amount, 50);
        assert!(cue.is_critical);
    }

    #[test]
    fn cue_death_construction() {
        let cue = CueDeath {
            entity: Entity::from_bits(1),
            killer: Some(Entity::from_bits(2)),
        };
        assert!(cue.killer.is_some());
    }

    #[test]
    fn cue_heal_construction() {
        let cue = CueHeal {
            target: Entity::from_bits(1),
            amount: 30,
            source: Some(Entity::from_bits(2)),
        };
        assert_eq!(cue.amount, 30);
    }
}
