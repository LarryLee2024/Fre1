// 回合管理模块：状态机、回合切换

use crate::unit::{Faction, Unit};
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
    /// 行动菜单（右键弹出）
    ActionMenu,
    /// 选择攻击目标
    SelectTarget,
    /// 执行攻击
    ExecuteAction,
    /// 待机
    WaitAction,
    /// 回合结束
    TurnEnd,
}

/// 当前回合阵营与回合数
#[derive(Resource)]
pub struct TurnState {
    pub current_faction: Faction,
    pub turn_number: u32,
}

impl Default for TurnState {
    fn default() -> Self {
        Self {
            current_faction: Faction::Player,
            turn_number: 1,
        }
    }
}

/// AI 行动延迟计时器
#[derive(Resource)]
pub struct AiTimer {
    pub timer: Timer,
}

impl Default for AiTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(0.8, TimerMode::Once),
        }
    }
}

/// 回合结束（OnEnter）
pub fn turn_end_on_enter(
    mut turn_state: ResMut<TurnState>,
    mut units: Query<&mut Unit>,
    mut next_phase: ResMut<NextState<TurnPhase>>,
    mut ai_timer: ResMut<AiTimer>,
) {
    let current_faction = turn_state.current_faction;

    // 检查当前阵营是否所有单位都已行动
    let all_acted = units
        .iter_mut()
        .filter(|u| u.faction == current_faction)
        .all(|u| u.acted);

    if all_acted {
        let next_faction = match current_faction {
            Faction::Player => Faction::Enemy,
            Faction::Enemy => {
                turn_state.turn_number += 1;
                Faction::Player
            }
        };
        turn_state.current_faction = next_faction;

        // 重置新阵营单位的行动状态
        for mut unit in units.iter_mut() {
            if unit.faction == next_faction {
                unit.acted = false;
            }
        }

        // 切换到敌方时重置 AI 计时器
        if next_faction == Faction::Enemy {
            ai_timer.timer.reset();
        }
    }

    next_phase.set(TurnPhase::SelectUnit);
}
