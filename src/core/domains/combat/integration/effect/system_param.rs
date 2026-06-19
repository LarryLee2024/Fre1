//! EffectTickParam — Bevy SystemParam，封装所有 Effect Capabilities 查询。
//!
//! Systems 通过此 param 驱动效果 Tick，完全不知道 `ActiveEffectContainer` /
//! `tick_durations` / `expire_effects` 的存在。
//!
//! # 用法
//!
//! ```rust,ignore
//! fn my_system(
//!     mut commands: Commands,
//!     mut effect_tick: EffectTickParam,
//! ) {
//!     let outcomes = effect_tick.tick_all(&mut commands);
//!     for outcome in outcomes {
//!         if outcome.has_activity() {
//!             info!("Tick activity: {} ticked, {} expired",
//!                 outcome.ticked.len(), outcome.expired.len());
//!         }
//!     }
//! }
//! ```
//!
//! # 设计决策
//!
//! - 不自动发射 `EffectTicked` 事件——调用方（Observer）决定何时/如何发射
//! - 不包装 `Commands` —— 调用方传入以保持 Observer 语义清晰

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::core::capabilities::effect::foundation::ActiveEffectContainer;
use crate::core::domains::combat::components::TurnQueue;

use super::facade;
use super::types::EffectTickOutcome;

/// 效果 Tick 查询参数 — 封装所有 Effect Capabilities 依赖。
///
/// System 签名中使用此类型替代裸 `Query<&mut ActiveEffectContainer>` + `Res<TurnQueue>`。
#[derive(SystemParam)]
pub struct EffectTickParam<'w, 's> {
    pub turn_queue: Res<'w, TurnQueue>,
    pub container_query: Query<'w, 's, &'static mut ActiveEffectContainer>,
}

impl<'w, 's> EffectTickParam<'w, 's> {
    /// 对所有实体执行 tick + expire 一步到位。
    ///
    /// 遍历所有持有 `ActiveEffectContainer` 的实体，对每个容器执行：
    /// 1. `tick_durations`（推进 1 回合计时）
    /// 2. `expire_effects`（清理到期效果）
    ///
    /// 返回每个实体的处理结果列表。
    pub fn tick_all(&mut self, commands: &mut Commands) -> Vec<EffectTickOutcome> {
        let current_turn = self.turn_queue.round_number() as u64;
        let mut outcomes = Vec::new();

        for mut container in self.container_query.iter_mut() {
            let outcome = facade::tick_and_expire(&mut container, current_turn, commands);
            outcomes.push(outcome);
        }

        outcomes
    }

    /// 仅推进计时，不清理到期效果。
    pub fn tick_only(&mut self, commands: &mut Commands) -> Vec<EffectTickOutcome> {
        let current_turn = self.turn_queue.round_number() as u64;
        let mut outcomes = Vec::new();

        for mut container in self.container_query.iter_mut() {
            let outcome = facade::tick_all_effects(&mut container, current_turn, commands);
            outcomes.push(outcome);
        }

        outcomes
    }

    /// 仅清理到期效果（假设已调用过 tick_only）。
    pub fn expire_only(&mut self) -> Vec<Vec<String>> {
        let mut all_expired = Vec::new();

        for mut container in self.container_query.iter_mut() {
            let expired = facade::expire_all_effects(&mut container);
            all_expired.push(expired);
        }

        all_expired
    }
}
