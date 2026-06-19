//! Playback Systems — Combat 域回放命令分发
//!
//! 在回放模式下，当 CombatPipelineDriver 暂停等待动作时，
//! 从 PlaybackSession 读取已录制的命令并触发 UnitActionComplete 恢复管线。
//!
//! # 回放流程
//!
//! ```text
//! 管线运行到 unit_action → paused = true
//!   │
//!   ├── dispatch_combat_replay_commands (Update 系统)
//!   │     ├── 检查: ReplayModeGuard.is_replay && 管线 paused
//!   │     ├── 读取 PlaybackSession.current_commands()
//!   │     ├── 匹配当前 TurnQueue 单位的命令
//!   │     └── commands.trigger(UnitActionComplete { unit })
//!   │
//!   ├── on_unit_action_complete (原有 observer)
//!   │     └── pipeline.resume()
//!   │
//!   └── 继续管线到下一单位
//! ```
//!
//! 详见 ADR-048 §Communication Design

use bevy::prelude::*;

use super::registry::BattleUnitRegistry;
use crate::core::capabilities::runtime::replay::foundation::ReplayCommand;
use crate::core::domains::combat::components::TurnQueue;
use crate::core::domains::combat::events::UnitActionComplete;
use crate::core::domains::combat::pipeline::driver::CombatPipelineDriver;
use crate::infra::replay::resources::{PlaybackSession, ReplayModeGuard};

/// Update 系统：回放模式下，当管线暂停时从 PlaybackSession 读取命令并恢复。
///
/// 执行条件：
/// - `ReplayModeGuard.is_replay == true`（回放模式）
/// - `CombatPipelineDriver.is_paused() == true`（管线暂停在 unit_action 阶段）
/// - `PlaybackSession.0.is_some()`（有回放会话）
///
/// 匹配策略：
/// 1. 读取 PlaybackSession 当前帧的所有命令
/// 2. 用 BattleUnitRegistry 将 String ID 转回 Entity
/// 3. 匹配当前 TurnQueue.current() 的单位
/// 4. 触发 UnitActionComplete 恢复管线
pub(crate) fn dispatch_combat_replay_commands(
    mode: Res<ReplayModeGuard>,
    pipeline: Res<CombatPipelineDriver>,
    turn_queue: Res<TurnQueue>,
    registry: Res<BattleUnitRegistry>,
    mut playback: ResMut<PlaybackSession>,
    mut commands: Commands,
) {
    // Guard 1: 仅在回放模式下执行
    if !mode.0.is_replay {
        return;
    }

    // Guard 2: 仅在管线暂停时执行
    if !pipeline.is_paused() {
        return;
    }

    // Guard 3: 需要有回放会话
    let Some(ref session) = playback.0 else {
        return;
    };

    // Guard 4: 需要有当前单位
    let Some(current) = turn_queue.current() else {
        return;
    };
    let current_unit = current.entity;

    // 读取当前帧的所有命令
    let frame_commands: Vec<&ReplayCommand> = session.current_commands();

    // 在当前帧中查找匹配当前单位的命令
    for replay_cmd in &frame_commands {
        let cmd_unit_id = match replay_cmd {
            ReplayCommand::UnitMove { unit, .. } => unit,
            ReplayCommand::UseAbility { caster, .. } => caster,
            ReplayCommand::SkipTurn { unit, .. } => unit,
            ReplayCommand::UseItem { user, .. } => user,
            ReplayCommand::ReactionConfirm { reactor, .. } => reactor,
            ReplayCommand::ConfirmTargets { caster, .. } => caster,
            ReplayCommand::DialogueChoice { speaker, .. } => speaker,
            ReplayCommand::Custom { params, .. } => {
                // Custom 命令：尝试查找 "unit" 参数
                params
                    .iter()
                    .find(|(k, _)| k == "unit")
                    .map(|(_, v)| v.as_str())
                    .unwrap_or("") // No match for empty string
            }
        };

        // Try to match this command's unit to the current TurnQueue unit
        if let Some(mapped_entity) = registry.get_entity_by_str(cmd_unit_id)
            && *mapped_entity == current_unit
        {
            // 找到匹配命令！推进到下一帧 & 触发恢复
            let _ = &mut playback.0.as_mut().unwrap().advance_frame();
            commands.trigger(UnitActionComplete { unit: current_unit });
            return;
        }
    }
}

/// PreUpdate 系统：回放模式下阻止真实玩家输入。
///
/// 通过清空 InputState 的 just_pressed 状态，防止回放时玩家操作影响结果。
pub(crate) fn block_player_input_during_replay(
    mode: Res<ReplayModeGuard>,
    input_state: Option<ResMut<crate::infra::input::resources::InputState>>,
) {
    if !mode.0.is_replay {
        return;
    }

    if let Some(mut input) = input_state {
        input.just_pressed_actions.clear();
        input.just_released_actions.clear();
    }
}
