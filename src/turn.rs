// 回合管理模块：状态机、回合切换

use bevy::prelude::*;

/// 游戏主状态
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, States)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    GameOver,
}

/// 回合阶段（SubState，仅在 InGame 时激活）
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, SubStates)]
#[source(AppState = AppState::InGame)]
pub enum TurnPhase {
    #[default]
    /// 选择单位
    SelectUnit,
    /// 移动阶段
    MoveUnit,
    /// 选择行动
    SelectAction,
    /// 执行行动
    ExecuteAction,
    /// 回合结束
    TurnEnd,
}

/// 当前回合阵营
#[derive(Resource)]
pub struct TurnState {
    pub current_faction: crate::unit::Faction,
    pub turn_number: u32,
}

impl Default for TurnState {
    fn default() -> Self {
        Self {
            current_faction: crate::unit::Faction::Player,
            turn_number: 1,
        }
    }
}
