//! ECS Components — 战斗领域核心组件
//!
//! 定义宏观战斗阶段、回合队列、行动资源等 ECS 组件与资源。
//! 回合内流程由 `pipeline::CombatPipelineDriver` 驱动，替代原 TurnSubState 状态机。
//! 详见 ADR-021, docs/02-domain/domains/combat_domain.md, docs/04-data/domains/combat_schema.md
//!
//! # 状态层次
//!
//! ```text
//! BattlePhase (States)
//! ──────────────
//! Preparation   — 战前部署
//! Battle        — 战斗中（由 CombatPipelineDriver 驱动回合循环）
//! Victory       — 胜利
//! Defeat        — 失败
//! ```

use bevy::prelude::*;

// ─── 外层宏观状态 ─────────────────────────────────────────────────────

/// 宏观战斗阶段。
///
/// 使用 Bevy States 实现，驱动战斗全生命周期。
/// 回合内流程由 `pipeline::CombatPipelineDriver` 驱动。
#[derive(States, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub enum BattlePhase {
    /// 战前部署（编队、先攻检定）。默认起始状态。
    #[default]
    Preparation,
    /// 战斗中（回合循环）。
    Battle,
    /// 胜利。
    Victory,
    /// 失败。
    Defeat,
}



// ─── 队伍标识 ─────────────────────────────────────────────────────────

/// 简单的队伍/阵营标识符。
///
/// 统一使用 shared::ids::TeamId（前缀: `team_`）。
/// 外部通过 integration layer 映射到 FactionId。
pub use crate::shared::ids::TeamId;

// ─── 回合队列 ─────────────────────────────────────────────────────────

/// 回合队列 — 管理行动顺序。
///
/// Resource，存储排序后的参与者列表和当前索引。
/// `current_index` 指向当前正在行动的单位。
#[derive(Resource, Debug, Clone)]
pub struct TurnQueue {
    /// 按先攻值从高到低排列的参与者列表
    entries: Vec<TurnEntry>,
    /// 当前行动单位在 entries 中的索引
    current_index: usize,
    /// 当前轮数（从 1 开始）
    round_number: u32,
}

impl TurnQueue {
    /// 创建一个新的回合队列。
    ///
    /// `entries` 必须已按先攻值从高到低排序。
    pub fn new(entries: Vec<TurnEntry>) -> Self {
        Self {
            entries,
            current_index: 0,
            round_number: 1,
        }
    }

    /// 当前行动的单位。
    pub fn current(&self) -> Option<&TurnEntry> {
        self.entries.get(self.current_index)
    }

    /// 当前行动单位的可变引用。
    pub fn current_mut(&mut self) -> Option<&mut TurnEntry> {
        self.entries.get_mut(self.current_index)
    }

    /// 前进到下一个单位。
    ///
    /// 如果回到队列开头，轮数 +1。空队列时无操作。
    pub fn advance(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        self.current_index = (self.current_index + 1) % self.entries.len();
        if self.current_index == 0 {
            self.round_number += 1;
        }
    }

    /// 当前索引。
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// 当前轮数。
    pub fn round_number(&self) -> u32 {
        self.round_number
    }

    /// 是否刚切换队伍（当前单位与前一个单位不同队）。
    pub fn just_changed_team(&self) -> bool {
        if self.entries.len() < 2 {
            return false;
        }
        let prev_index = if self.current_index == 0 {
            self.entries.len() - 1
        } else {
            self.current_index - 1
        };
        self.entries[prev_index].team_id != self.entries[self.current_index].team_id
    }

    /// 当前队伍标识。
    pub fn current_team(&self) -> Option<&TeamId> {
        self.current().map(|e| &e.team_id)
    }

    /// 所有条目引用。
    pub fn entries(&self) -> &[TurnEntry] {
        &self.entries
    }

    /// 条目数量（参与者总数）。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 队列是否为空。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for TurnQueue {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            current_index: 0,
            round_number: 1,
        }
    }
}

// ─── 回合条目 ─────────────────────────────────────────────────────────

/// 回合队列中的单个条目。
#[derive(Debug, Clone, PartialEq)]
pub struct TurnEntry {
    /// 参与者实体
    pub entity: Entity,
    /// 所属队伍
    pub team_id: TeamId,
    /// 先攻值（用于排序）
    pub initiative: u32,
}

impl TurnEntry {
    pub fn new(entity: Entity, team_id: TeamId, initiative: u32) -> Self {
        Self {
            entity,
            team_id,
            initiative,
        }
    }
}

// ─── 战斗参与者标记 ────────────────────────────────────────────────────

/// 战斗参与者标记与存活状态。
///
/// 每个参与战斗的单位都会获得此组件。
/// `is_alive` 标记用于胜利条件判定（团队全灭检查）。
///
/// 详见 combat_domain.md §5.4, combat_schema.md §1.3
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct CombatParticipant {
    /// 所属队伍
    pub team_id: TeamId,
    /// 是否存活
    pub is_alive: bool,
}

impl CombatParticipant {
    /// 创建存活状态的参与者。
    pub fn alive(team_id: TeamId) -> Self {
        Self {
            team_id,
            is_alive: true,
        }
    }
}

// ─── 行动资源 ─────────────────────────────────────────────────────────

/// 单位在当前回合的行动资源。每轮重置。
#[derive(Component, Debug, Clone, PartialEq, Reflect)]
#[reflect(Component)]
pub struct ActionPoints {
    /// 标准动作是否可用
    pub standard_action: bool,
    /// 附赠动作是否可用
    pub bonus_action: bool,
    /// 反应是否可用（回合外使用，每轮最多 1 次）
    pub reaction: bool,
    /// 当前剩余移动力
    pub movement: f32,
    /// 最大移动力
    pub max_movement: f32,
}

impl ActionPoints {
    /// 创建新的行动资源，所有动作可用。
    pub fn new(max_movement: f32) -> Self {
        Self {
            standard_action: true,
            bonus_action: true,
            reaction: true,
            movement: max_movement,
            max_movement,
        }
    }

    /// 重置所有行动资源（回合开始调用）。
    pub fn reset(&mut self) {
        self.standard_action = true;
        self.bonus_action = true;
        // 反应不在此重置（每轮自然恢复，当前保留可用状态）
        self.movement = self.max_movement;
    }

    /// 消耗标准动作。返回 false 如果不可用。
    pub fn use_standard_action(&mut self) -> bool {
        if !self.standard_action {
            return false;
        }
        self.standard_action = false;
        true
    }

    /// 消耗附赠动作。返回 false 如果不可用。
    pub fn use_bonus_action(&mut self) -> bool {
        if !self.bonus_action {
            return false;
        }
        self.bonus_action = false;
        true
    }

    /// 消耗反应。返回 false 如果不可用。
    pub fn use_reaction(&mut self) -> bool {
        if !self.reaction {
            return false;
        }
        self.reaction = false;
        true
    }

    /// 消耗移动力。返回 false 如果不足。
    pub fn consume_movement(&mut self, cost: f32) -> bool {
        if cost > self.movement {
            return false;
        }
        self.movement -= cost;
        true
    }

    /// 是否有任何可用动作。
    pub fn has_any_action(&self) -> bool {
        self.standard_action || self.bonus_action
    }

    /// 是否完全没有可做的事（用于 PhaseCheck 跳过的判定）。
    pub fn is_idle(&self) -> bool {
        !self.has_any_action() && self.movement <= 0.0
    }
}
