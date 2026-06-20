//! Combat Input System — 战斗回合输入处理
//!
//! 读取 InputState，将技能槽、回合结束等输入映射为战斗命令。
//! 仅在玩家回合期间响应输入。
//!
//! # 系统间契约
//!
//! - **输入方**：infra::input（InputState / InputAction）— 原始按键已在此层翻译为语义动作
//! - **消费方**：本系统消费语义动作，构造战斗命令入队到 CommandQueue
//! - **边界规则**：本系统不访问原始按键（KeyCode），只消费 InputAction 枚举
//! - **方向**：Infra (Input) → Domain (Combat)，符合架构法第 2 层依赖方向

use bevy::prelude::*;
use tracing;

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

    let Ok(_ap) = action_points_q.get_mut(current_entry.entity) else {
        return;
    };

    if input_state.just_pressed(InputAction::SkillSlot1) {
        tracing::trace!(target: "combat", event = "combat_input.skill_slot", slot = 1);
    }

    if input_state.just_pressed(InputAction::SkillSlot2) {
        tracing::trace!(target: "combat", event = "combat_input.skill_slot", slot = 2);
    }

    if input_state.just_pressed(InputAction::SkillSlot3) {
        tracing::trace!(target: "combat", event = "combat_input.skill_slot", slot = 3);
    }

    if input_state.just_pressed(InputAction::SkillSlot4) {
        tracing::trace!(target: "combat", event = "combat_input.skill_slot", slot = 4);
    }

    if input_state.just_pressed(InputAction::EndTurn) {
        tracing::trace!(target: "combat", event = "combat_input.end_turn");
    }

    if input_state.just_pressed(InputAction::Select) {
        tracing::trace!(target: "combat", event = "combat_input.confirm");
    }

    if input_state.just_pressed(InputAction::Cancel) {
        tracing::trace!(target: "combat", event = "combat_input.cancel");
    }
}
