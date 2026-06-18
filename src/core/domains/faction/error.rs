//! 领域错误 — Faction 域错误枚举。
//!
//! 涵盖声望管理、阵营关系判定等操作的错误。
//! 详见 docs/02-domain/domains/faction_domain.md §4

use bevy::prelude::*;

use super::components::FactionId;

/// 阵营系统错误。
#[derive(Debug, Clone, PartialEq, Event)]
pub enum FactionError {
    /// 声望值超出有效范围 [-100, +100]。
    ReputationOutOfRange { value: i32 },
    /// 实体不属于指定阵营。
    NotMemberOfFaction { faction_id: FactionId },
    /// 阵营 ID 未注册。
    FactionNotFound { faction_id: FactionId },
    /// 关键角色声望保护触发，不允许降到该阈值以下。
    CriticalCharacterProtection { faction_id: FactionId },
    /// 阵营间关系不对称，违反对称性不变量。
    RelationAsymmetry { a: FactionId, b: FactionId },
}

impl std::fmt::Display for FactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReputationOutOfRange { value } => {
                write!(f, "reputation value out of range: {}", value)
            }
            Self::NotMemberOfFaction { faction_id } => {
                write!(f, "entity is not a member of faction: {}", faction_id)
            }
            Self::FactionNotFound { faction_id } => {
                write!(f, "faction not found: {}", faction_id)
            }
            Self::CriticalCharacterProtection { faction_id } => {
                write!(
                    f,
                    "critical character reputation protection triggered for faction: {}",
                    faction_id
                )
            }
            Self::RelationAsymmetry { a, b } => {
                write!(
                    f,
                    "faction relation asymmetry detected between {} and {}",
                    a, b
                )
            }
        }
    }
}

impl std::error::Error for FactionError {}
