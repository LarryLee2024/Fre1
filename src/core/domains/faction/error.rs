//! 领域错误 — Faction 域错误枚举。
//!
//! 涵盖声望管理、阵营关系判定等操作的错误。
//! 详见 docs/02-domain/domains/faction_domain.md §4

use bevy::prelude::*;
use thiserror::Error;

use super::components::FactionId;

/// 阵营系统错误。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum FactionError {
    /// 声望值超出有效范围 [-100, +100]。
    #[error("reputation value out of range: {value}")]
    ReputationOutOfRange { value: i32 },
    /// 实体不属于指定阵营。
    #[error("entity is not a member of faction: {faction_id}")]
    NotMemberOfFaction { faction_id: FactionId },
    /// 阵营 ID 未注册。
    #[error("faction not found: {faction_id}")]
    FactionNotFound { faction_id: FactionId },
    /// 关键角色声望保护触发，不允许降到该阈值以下。
    #[error("critical character reputation protection triggered for faction: {faction_id}")]
    CriticalCharacterProtection { faction_id: FactionId },
    /// 阵营间关系不对称，违反对称性不变量。
    #[error("faction relation asymmetry detected between {a} and {b}")]
    RelationAsymmetry { a: FactionId, b: FactionId },
}
