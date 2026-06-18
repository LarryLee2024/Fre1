//! 领域错误 — Party 域错误枚举
//!
//! 涵盖成员管理、战斗换人、羁绊激活等操作的错误。
//! 详见 docs/02-domain/domains/party_domain.md §4

use bevy::prelude::*;

/// 队伍系统错误。
#[derive(Debug, Clone, PartialEq, Event)]
pub enum PartyError {
    /// 队伍已满，无法添加新成员。
    Full { current: usize, max: u32 },
    /// 该成员已在队伍中。
    AlreadyInParty { entity: Entity },
    /// 未找到指定成员。
    MemberNotFound { entity: Entity },
    /// 活跃成员已满，无法激活更多。
    ActiveFull { current: usize, max: u32 },
    /// 本回合已进行过换人操作。
    SwapAlreadyPerformedThisTurn,
    /// 行动力不足，无法换人。
    InsufficientActionPoints { required: u32, available: u32 },
    /// 该队员不在预备队伍中。
    NotInReserve { entity: Entity },
    /// 羁绊模板未注册。
    BondDefNotFound { bond_id: String },
    /// 羁绊已激活。
    BondAlreadyActive { bond_id: String },
}
