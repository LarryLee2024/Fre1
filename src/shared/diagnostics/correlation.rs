//! 关联标识类型，用于串联一次完整战斗行为中的所有日志。
//!
//! 层级关系: `BattleId` → `TurnId` → `ActionId`

/// 战斗唯一标识。
pub type BattleId = u64;

/// 回合标识：`(BattleId, RoundNumber, TurnIndex)`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TurnId {
    /// 所属战斗 ID
    pub battle_id: BattleId,
    /// 轮次（Round）
    pub round: u32,
    /// 该轮中的回合索引
    pub turn_index: u32,
}

impl TurnId {
    /// 创建新的 TurnId。
    pub fn new(battle_id: BattleId, round: u32, turn_index: u32) -> Self {
        Self {
            battle_id,
            round,
            turn_index,
        }
    }
}

impl std::fmt::Display for TurnId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Turn({},R{})", self.turn_index, self.round)
    }
}

/// 行动标识：`(TurnId, ActionSequence)`。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActionId {
    /// 所属回合标识
    pub turn_id: TurnId,
    /// 行动序号（同一回合内的第几次行动）
    pub sequence: u32,
}

impl ActionId {
    /// 创建新的 ActionId。
    pub fn new(turn_id: TurnId, sequence: u32) -> Self {
        Self { turn_id, sequence }
    }
}

impl std::fmt::Display for ActionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Action({},{}R{})",
            self.sequence, self.turn_id.turn_index, self.turn_id.round
        )
    }
}

/// 关联标识，用于串联一次完整战斗行为中的所有日志。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CorrelationId {
    /// 战斗级关联：同一场战斗的所有日志
    Battle(BattleId),

    /// 回合级关联：同一轮/回合的所有日志
    Turn(TurnId),

    /// 行动级关联：同一次行动（技能/攻击/物品使用）的所有日志
    Action(ActionId),
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Battle(id) => write!(f, "Battle({})", id),
            Self::Turn(tid) => write!(f, "{}", tid),
            Self::Action(aid) => write!(f, "{}", aid),
        }
    }
}
