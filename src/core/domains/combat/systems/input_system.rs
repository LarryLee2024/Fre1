//! Combat Input System — 战斗回合输入处理
//!
//! 读取 InputState，将技能槽、回合结束等输入映射为战斗命令。
//! 仅在玩家回合期间响应输入。

use bevy::prelude::*;

use crate::infra::input::action::InputAction;
use crate::infra::input::resources::InputState;

use super::super::components::{ActionPoints, BattlePhase, CombatParticipant, TurnQueue};

/// 战斗域玩家回合标记（Resource）。
///
/// 由回合系统设置，标识当前是否为玩家回合。
#[derive(Resource, Debug, Default)]
pub struct PlayerTurnState {
    pub is_player_turn: bool,
}

/// 战斗域输入系统 — 处理技能选择、回合结束等。
pub(crate) fn combat_input_system(
    input_state: Res<InputState>,
    battle_phase: Res<State<BattlePhase>>,
    turn_queue: Res<TurnQueue>,
    mut action_points_q: Query<&mut ActionPoints>,
) {
    if *battle_phase.get() != BattlePhase::Battle {
        return;
    }

    let Some(current_entry) = turn_queue.current() else {
        return;
    };

    let Ok(mut ap) = action_points_q.get_mut(current_entry.entity) else {
        return;
    };

    if input_state.just_pressed(InputAction::SkillSlot1) {
        info!("[CombatInput] Skill slot 1 selected");
    }

    if input_state.just_pressed(InputAction::SkillSlot2) {
        info!("[CombatInput] Skill slot 2 selected");
    }

    if input_state.just_pressed(InputAction::SkillSlot3) {
        info!("[CombatInput] Skill slot 3 selected");
    }

    if input_state.just_pressed(InputAction::SkillSlot4) {
        info!("[CombatInput] Skill slot 4 selected");
    }

    if input_state.just_pressed(InputAction::EndTurn) {
        info!("[CombatInput] End turn requested");
    }

    if input_state.just_pressed(InputAction::Select) {
        info!("[CombatInput] Confirm action");
    }

    if input_state.just_pressed(InputAction::Cancel) {
        info!("[CombatInput] Cancel action");
    }
}
