//! Combat test fixtures — 可复用的测试构建器与标准测试场景
//!
//! 遵循 test-guardian 规范，使用标准测试单元构造战斗场景。
//!
//! # 标准测试单元
//!
//! | 编号 | 角色 | 先攻 | 队伍 | 冲刺力 |
//! |------|------|------|------|--------|
//! | e(1) | 战士 | 25 | player | 6.0 |
//! | e(2) | 法师 | 22 | enemy | 5.0 |
//! | e(3) | 游侠 | 18 | player | 7.0 |
//! | e(4) | 坦克 | 14 | enemy | 4.0 |
//! | e(5) | 武僧 | 10 | player | 8.0 |

use bevy::prelude::Entity;

use crate::core::domains::combat::components::{CombatParticipant, TeamId, TurnEntry, TurnQueue};

/// 创建测试用的 Entity（基于原始 u32 ID）。
pub fn e(id: u32) -> Entity {
    Entity::from_raw_u32(id).unwrap()
}

/// 标准玩家队伍标识。
pub fn player_team() -> TeamId {
    TeamId::new("player")
}

/// 标准敌方队伍标识。
pub fn enemy_team() -> TeamId {
    TeamId::new("enemy")
}

/// 创建一个所有参赛者存活的 CombatParticipant。
pub fn alive_participant(team: TeamId) -> CombatParticipant {
    CombatParticipant::alive(team)
}

/// 创建一个已阵亡的 CombatParticipant。
pub fn dead_participant(team: TeamId) -> CombatParticipant {
    CombatParticipant {
        team_id: team,
        is_alive: false,
    }
}

/// 标准混合两队的回合条目（先攻从高到低）。
pub fn interleaved_entries() -> Vec<TurnEntry> {
    vec![
        TurnEntry::new(e(1), player_team(), 25),
        TurnEntry::new(e(2), enemy_team(), 22),
        TurnEntry::new(e(3), player_team(), 18),
        TurnEntry::new(e(4), enemy_team(), 14),
        TurnEntry::new(e(5), player_team(), 10),
    ]
}

/// 单队回合条目。
pub fn single_team_entries() -> Vec<TurnEntry> {
    vec![
        TurnEntry::new(e(1), player_team(), 30),
        TurnEntry::new(e(2), player_team(), 20),
        TurnEntry::new(e(3), player_team(), 10),
    ]
}

/// 从 TurnEntry 列表创建并填充 BattlePhase/CombatParticipant 后的标准化测试场景。
pub struct TestCombatScenario {
    pub queue: TurnQueue,
    pub entries: Vec<TurnEntry>,
}

impl TestCombatScenario {
    /// 创建标准两军对垒场景（5 个单位，先攻交错）。
    pub fn standard_battle() -> Self {
        let entries = interleaved_entries();
        Self {
            queue: TurnQueue::new(entries.clone()),
            entries,
        }
    }

    /// 创建单队场景。
    pub fn single_team_battle() -> Self {
        let entries = single_team_entries();
        Self {
            queue: TurnQueue::new(entries.clone()),
            entries,
        }
    }

    /// 创建空队列场景（边界测试）。
    pub fn empty() -> Self {
        Self {
            queue: TurnQueue::new(vec![]),
            entries: vec![],
        }
    }
}
