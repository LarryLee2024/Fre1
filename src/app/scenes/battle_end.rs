//! Module Name: BattleEnd Observer — 战斗结束时切换到结算画面
//!
//! 监听 BattleEnded 事件，根据 victory 字段决定切换到 Result 或 GameOver 状态。
//!
//! 遵循 Observer 通信模式（四级通信第四级），
//! 通过 StateTransitionQueue 提交 GameState 切换请求。

use bevy::prelude::*;

use crate::core::events::BattleEnded;
use crate::shared::game_state::{GameState, TransitionRequest};

use super::queue::StateTransitionQueue;

/// Observer: 战斗结束 — 切换到结算画面
///
/// 当 BattleEnded 事件触发时，检查 victory 标志：
/// - victory: true  → 切换到 GameState::Result
/// - victory: false → 切换到 GameState::GameOver
pub(crate) fn on_battle_ended(
    trigger: On<BattleEnded>,
    mut queue: ResMut<StateTransitionQueue>,
) {
    if trigger.event().victory {
        info!(target: "app", "Battle ended in victory — transitioning to Result");
        queue.push(TransitionRequest::Change(GameState::Result));
    } else {
        info!(target: "app", "Battle ended in defeat — transitioning to GameOver");
        queue.push(TransitionRequest::Change(GameState::GameOver));
    }
}
