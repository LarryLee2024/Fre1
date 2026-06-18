//! Turn Systems — 战斗生命周期系统
//!
//! 回合内流程（TurnStart → PhaseCheck → UnitAction → TurnSettlement → TurnEnd）
//! 已迁移至 `pipeline::CombatPipelineDriver`。本模块保留：
//!
//! - BattlePhase 生命周期（OnEnter/OnExit）
//! - Observer 响应（OnTurnEnd → 效果计时/冷却，OnTurnStart → 触发器评估）
//! - `initialize_turn_order` 初始化工具

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::core::domains::combat::components::{
    ActionPoints, CombatParticipant, TeamId, TurnEntry, TurnQueue,
};
use crate::core::domains::combat::events::{
    BattleResult, OnBattleEnd, OnBattleStart, OnTurnEnd, OnTurnStart,
};
use crate::core::domains::combat::integration::ability::CombatAbilityFacade;
use crate::core::domains::combat::integration::ability::CombatAbilityParam;
use crate::core::domains::combat::integration::trigger::CombatTriggerFacade;
use crate::core::domains::combat::integration::trigger::CombatTriggerParam;
use crate::core::domains::combat::integration::trigger::CombatTriggerType;

// ═══════════════════════════════════════════════════════════════════════
// BattlePhase 生命周期
// ═══════════════════════════════════════════════════════════════════════

/// 进入 BattlePhase::Battle 时触发 OnBattleStart 并启动回合管线。
pub(crate) fn on_enter_battle(mut commands: Commands) {
    commands.trigger(OnBattleStart);
    info!("[Combat] Battle started");
}

// ═══════════════════════════════════════════════════════════════════════
// Capability Integration — Ability Cooldown Tick
// ═══════════════════════════════════════════════════════════════════════

/// Observer: OnTurnEnd → 推进当前单位的 Ability 冷却计时。
///
/// 与 effect_tick_system 并行，在回合结束时推进：
/// - 技能冷却（tick_all_cooldowns）
/// - 共享冷却（tick_shared_cooldowns）
pub(crate) fn on_turn_end_tick_ability_cooldowns(
    trigger: On<'_, '_, OnTurnEnd>,
    mut ability_param: CombatAbilityParam,
) {
    let unit = trigger.event().unit;
    let expired = ability_param.tick_cooldowns_for_unit(unit);
    if !expired.is_empty() {
        debug!(
            "[Combat-Ability] {} ability cooldown(s) expired for unit {:?}",
            expired.len(),
            unit
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Capability Integration — Trigger Evaluation
// ═══════════════════════════════════════════════════════════════════════

/// Observer: OnTurnStart → 评估当前单位的 OnTurnStart 触发器。
///
/// 查找并记录触发器中 OnTurnStart 类型的就绪条目。
/// 后续将通过 CombatTriggerFacade 分发到目标 Ability 激活。
pub(crate) fn on_turn_start_evaluate_triggers(
    trigger: On<'_, '_, OnTurnStart>,
    mut trigger_param: CombatTriggerParam,
) {
    let unit = trigger.event().unit;
    let ready_ids = trigger_param.evaluate_and_consume(unit, CombatTriggerType::TurnStarted);
    if !ready_ids.is_empty() {
        debug!(
            "[Combat-Trigger] {} OnTurnStart trigger(s) ready for unit {:?}: {:?}",
            ready_ids.len(),
            unit,
            ready_ids
        );
    }
}

/// 进入 BattlePhase::Victory 时触发 OnBattleEnd。
pub(crate) fn on_enter_victory(mut commands: Commands) {
    commands.trigger(OnBattleEnd {
        result: BattleResult::Victory,
    });
    info!("[Combat] Victory!");
}

/// 进入 BattlePhase::Defeat 时触发 OnBattleEnd。
pub(crate) fn on_enter_defeat(mut commands: Commands) {
    commands.trigger(OnBattleEnd {
        result: BattleResult::Defeat,
    });
    info!("[Combat] Defeat!");
}

// ═══════════════════════════════════════════════════════════════════════
// Utility: 根据回合条目初始化 TurnQueue + ActionPoints
// ═══════════════════════════════════════════════════════════════════════

/// 初始化回合队列和所有参与者的行动资源。
///
/// 在战斗开始时调用一次：
/// 1. 为每个参与者添加 `ActionPoints` 和 `CombatParticipant` 组件
/// 2. 将 `TurnQueue` 插入为 Resource
///
/// # Panics
///
/// 如果 `entries` 为空，则 panic（战斗至少需要 2 方）。
pub fn initialize_turn_order(
    commands: &mut Commands,
    entries: Vec<(Entity, TeamId, u32)>,
    default_movement: f32,
) {
    let turn_entries: Vec<_> = entries
        .into_iter()
        .map(|(entity, team_id, initiative)| {
            commands.entity(entity).insert((
                ActionPoints::new(default_movement),
                CombatParticipant::alive(team_id.clone()),
                CombatAbilityFacade::empty_container(),
                CombatTriggerFacade::empty_container(),
            ));
            TurnEntry::new(entity, team_id, initiative)
        })
        .collect();

    let queue = TurnQueue::new(turn_entries);
    commands.insert_resource(queue);
}
