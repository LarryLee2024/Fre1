//! Turn Systems — 回合状态机各阶段 System
//!
//! 实现 ADR-021 定义的五阶段回合流程：
//! TurnStart → PhaseCheck → UnitAction → TurnSettlement → TurnEnd
//!
//! # 约束
//!
//! - 🟥 禁止在 OnEnter/OnExit 中执行重型操作（重型逻辑放 Update System）
//! - 🟥 禁止跳过 PhaseCheck 阶段
//! - 🟥 同一 System 不得跨多个阶段
//! - 🟥 外部禁止直接调用 `next_state.set(TurnSubState::...)`
//!
//! # 状态流转
//!
//! ```text
//! BattlePhase::Battle (OnEnter)
//!   │
//!   ▼
//! TurnSubState::TurnStart (OnEnter)
//!   │  ├── ActionPoint 重置
//!   │  ├── commands.trigger(OnTurnStart { unit })
//!   │  └── → TurnSubState::PhaseCheck
//!   │
//!   ▼
//! TurnSubState::PhaseCheck (Update)
//!   │  ├── 有 AP/MP → TurnSubState::UnitAction
//!   │  └── 无行动 → TurnSubState::TurnSettlement
//!   │
//!   ▼
//! TurnSubState::UnitAction (Update)
//!   │  └── UnitActionComplete Event → TurnSubState::TurnSettlement
//!   │
//!   ▼
//! TurnSubState::TurnSettlement (OnEnter)
//!   │  ├── commands.trigger(OnTurnEnd { unit })
//!   │  └── → TurnSubState::TurnEnd
//!   │
//!   ▼
//! TurnSubState::TurnEnd (OnEnter)
//!      ├── turn_queue.advance()
//!      ├── 战斗结束？→ BattlePhase::Victory/Defeat
//!      └── 继续 → TurnSubState::TurnStart
//! ```

use bevy::ecs::observer::On;
use bevy::prelude::*;

use crate::core::domains::combat::components::{
    ActionPoints, BattlePhase, CombatParticipant, TeamId, TurnEntry, TurnQueue, TurnSubState,
};
use crate::core::domains::combat::events::{
    BattleResult, BetweenTurns, OnBattleEnd, OnBattleStart, OnRoundEnd, OnTurnEnd, OnTurnStart,
    UnitActionComplete,
};

// ═══════════════════════════════════════════════════════════════════════
// Phase 1: TurnStart
// ═══════════════════════════════════════════════════════════════════════

/// 进入 TurnStart 阶段：重置当前单位的行动资源，发射领域事件。
///
/// 职责（轻量 OnEnter）：
/// 1. 获取当前单位
/// 2. 重置 ActionPoints（标准动作/附赠动作/移动力）
/// 3. 发射 OnTurnStart Trigger
/// 4. 立即切换到 PhaseCheck
pub(crate) fn on_enter_turn_start(
    mut commands: Commands,
    turn_queue: Res<TurnQueue>,
    mut ap_query: Query<&mut ActionPoints>,
    mut next_state: ResMut<NextState<TurnSubState>>,
) {
    let Some(current) = turn_queue.current() else {
        warn!("[Combat] TurnStart: empty turn queue, skipping");
        return;
    };

    // 重置行动资源
    if let Ok(mut ap) = ap_query.get_mut(current.entity) {
        ap.reset();
    }

    // 发射 OnTurnStart 领域事件
    commands.trigger(OnTurnStart {
        unit: current.entity,
    });

    debug!(
        "[Combat] TurnStart: unit={:?}, team={}, round={}",
        current.entity,
        current.team_id,
        turn_queue.round_number()
    );

    // 进入下一阶段
    next_state.set(TurnSubState::PhaseCheck);
}

// ═══════════════════════════════════════════════════════════════════════
// Phase 2: PhaseCheck
// ═══════════════════════════════════════════════════════════════════════

/// PhaseCheck: 判定当前单位可以执行什么行动。
///
/// - 有可用动作 → 进入 UnitAction 等待用户/AI 输入
/// - 无可用动作 → 跳过至 TurnSettlement
///
/// 在 Update 中运行，由 `in_state(TurnSubState::PhaseCheck)` 条件守卫。
pub(crate) fn phase_check(
    turn_queue: Res<TurnQueue>,
    ap_query: Query<&ActionPoints>,
    mut next_state: ResMut<NextState<TurnSubState>>,
) {
    let Some(current) = turn_queue.current() else {
        warn!("[Combat] PhaseCheck: empty turn queue, skipping");
        next_state.set(TurnSubState::TurnSettlement);
        return;
    };

    match ap_query.get(current.entity) {
        Ok(ap) => {
            if ap.is_idle() {
                debug!(
                    "[Combat] PhaseCheck: unit={:?} idle, skipping to settlement",
                    current.entity
                );
                next_state.set(TurnSubState::TurnSettlement);
            } else {
                debug!(
                    "[Combat] PhaseCheck: unit={:?} has actions, waiting for input",
                    current.entity
                );
                next_state.set(TurnSubState::UnitAction);
            }
        }
        Err(_) => {
            // 没有 ActionPoints 组件 → 跳过
            debug!(
                "[Combat] PhaseCheck: unit={:?} has no ActionPoints, skipping",
                current.entity
            );
            next_state.set(TurnSubState::TurnSettlement);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Phase 3: UnitAction
// ═══════════════════════════════════════════════════════════════════════

/// Observer: 监听 `UnitActionComplete` Trigger，回合内单位行动完成后进入 TurnSettlement。
///
/// 外部系统（Ability/Execution）在行动完成后简单地 `commands.trigger(UnitActionComplete { unit })`。
/// 本 Observer 仅在 `TurnSubState::UnitAction` 阶段处理通知，忽略非当前单位的完成事件。
pub(crate) fn on_unit_action_complete(
    trigger: On<'_, '_, UnitActionComplete>,
    turn_queue: Res<TurnQueue>,
    mut next_state: ResMut<NextState<TurnSubState>>,
) {
    let Some(current) = turn_queue.current() else {
        return;
    };

    let event = trigger.event();
    if event.unit == current.entity {
        debug!(
            "[Combat] UnitAction: unit={:?} action complete, transitioning to settlement",
            event.unit
        );
        next_state.set(TurnSubState::TurnSettlement);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Phase 4: TurnSettlement
// ═══════════════════════════════════════════════════════════════════════

/// 进入 TurnSettlement 阶段：回合结算。
///
/// 职责（轻量 OnEnter）：
/// 1. 发射 OnTurnEnd Trigger（触发"回合结束时"类效果 / Buff Tick / DOT）
/// 2. 切换到 TurnEnd
pub(crate) fn on_enter_turn_settlement(
    mut commands: Commands,
    turn_queue: Res<TurnQueue>,
    mut next_state: ResMut<NextState<TurnSubState>>,
) {
    let Some(current) = turn_queue.current() else {
        warn!("[Combat] TurnSettlement: empty turn queue, skipping");
        return;
    };

    // 发射 OnTurnEnd 领域事件
    commands.trigger(OnTurnEnd {
        unit: current.entity,
    });

    debug!(
        "[Combat] TurnSettlement: unit={:?} settlement complete",
        current.entity
    );

    next_state.set(TurnSubState::TurnEnd);
}

// ═══════════════════════════════════════════════════════════════════════
// Phase 5: TurnEnd
// ═══════════════════════════════════════════════════════════════════════

/// 进入 TurnEnd 阶段：切换到下一个单位/队伍。
///
/// 职责（轻量 OnEnter）：
/// 1. 前进到队列下一个单位
/// 2. 如果切换队伍 → 发射 BetweenTurns
/// 3. 如果所有队伍完成一轮 → 发射 OnRoundEnd
/// 4. 检查战斗结束条件 → Victory/Defeat 或回到 TurnStart
pub(crate) fn on_enter_turn_end(
    mut commands: Commands,
    mut turn_queue: ResMut<TurnQueue>,
    mut battle_phase: ResMut<NextState<BattlePhase>>,
    mut turn_sub_state: ResMut<NextState<TurnSubState>>,
    combatant_query: Query<&CombatParticipant>,
) {
    // 检查战斗结束条件
    if turn_queue.is_empty() {
        warn!("[Combat] TurnEnd: empty turn queue, ending battle");
        battle_phase.set(BattlePhase::Victory);
        return;
    }

    // 记录当前信息（在 advance 之前）
    let changed_team = turn_queue.just_changed_team();
    let was_last_in_round = turn_queue.current_index() == turn_queue.len() - 1;
    let prev_team = turn_queue.current_team().cloned();

    // 前进到下一个单位
    turn_queue.advance();

    let round = turn_queue.round_number();

    debug!(
        "[Combat] TurnEnd: advanced to index={}, round={}",
        turn_queue.current_index(),
        round
    );

    // 如果切换队伍 → 发射 BetweenTurns
    if changed_team {
        if let Some(team) = prev_team {
            commands.trigger(BetweenTurns { team });
        }
    }

    // 如果所有队伍完成一轮 → 发射 OnRoundEnd
    if was_last_in_round {
        commands.trigger(OnRoundEnd { round });
    }

    // Team elimination victory check — any team with all participants dead loses.
    let team_status = combatant_query.iter().fold(
        std::collections::HashMap::<String, (usize, usize)>::new(),
        |mut acc, participant| {
            let team = participant.team_id.0.clone();
            let entry = acc.entry(team).or_insert((0, 0));
            entry.0 += 1;
            if participant.is_alive {
                entry.1 += 1;
            }
            acc
        },
    );

    let mut alive_teams = Vec::new();
    let mut dead_teams = Vec::new();
    for (team, (_total, alive)) in &team_status {
        if *alive == 0 {
            dead_teams.push(team.clone());
        } else {
            alive_teams.push(team.clone());
        }
    }

    if !dead_teams.is_empty() && !alive_teams.is_empty() {
        info!(
            "[Combat] Victory check: dead teams={:?}, alive teams={:?}",
            dead_teams, alive_teams
        );
        commands.trigger(OnBattleEnd {
            result: BattleResult::Victory,
        });
        battle_phase.set(BattlePhase::Victory);
    } else {
        turn_sub_state.set(TurnSubState::TurnStart);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// BattlePhase 生命周期
// ═══════════════════════════════════════════════════════════════════════

/// 进入 BattlePhase::Battle 时触发 OnBattleStart。
pub(crate) fn on_enter_battle(mut commands: Commands) {
    commands.trigger(OnBattleStart);
    info!("[Combat] Battle started");
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
            ));
            TurnEntry::new(entity, team_id, initiative)
        })
        .collect();

    let queue = TurnQueue::new(turn_entries);
    commands.insert_resource(queue);
}
