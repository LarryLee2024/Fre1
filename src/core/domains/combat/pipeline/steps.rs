//! Pipeline Steps — 回合管线各步骤执行函数
//!
//! 将原有的 OnEnter/Observer System 逻辑提取为纯数据操作函数，
//! 由 CombatPipelineDriver 调度，不直接与 ECS 调度器连接。

use bevy::prelude::*;

use crate::core::domains::combat::components::{ActionPoints, CombatParticipant, Dead, TurnQueue};
use crate::core::domains::combat::events::{
    BattleResult, BetweenTurns, OnBattleEnd, OnRoundEnd, OnTurnEnd, OnTurnStart,
};
use crate::core::events::TurnEnded;

// ═══════════════════════════════════════════════════════════════════════
// Step 1: TurnStart
// ═══════════════════════════════════════════════════════════════════════

/// TurnStart 步骤：重置当前单位的行动资源，发射 OnTurnStart 领域事件。
pub(crate) fn step_turn_start(
    commands: &mut Commands,
    turn_queue: &TurnQueue,
    ap_query: &mut Query<&mut ActionPoints>,
) {
    let Some(current) = turn_queue.current() else {
        debug!("[Combat] TurnStart: 回合队列为空，跳过");
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
        "[Combat] TurnStart: 单位={:?}, 队伍={}, 回合={}",
        current.entity,
        current.team_id,
        turn_queue.round_number()
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Step 2: PhaseCheck
// ═══════════════════════════════════════════════════════════════════════

/// PhaseCheck 判定结果。
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum PhaseCheckResult {
    /// 单位有可用行动，应进入 UnitAction 等待输入
    HasActions,
    /// 单位无可用行动，应跳过至 TurnSettlement
    Idle,
}

/// PhaseCheck 步骤：判定当前单位可以执行什么行动。
///
/// 返回判定结果供驾驶员决定下一阶段。
pub(crate) fn step_phase_check(
    turn_queue: &TurnQueue,
    ap_query: &Query<&mut ActionPoints>,
) -> PhaseCheckResult {
    let Some(current) = turn_queue.current() else {
        debug!("[Combat] PhaseCheck: 回合队列为空，跳过");
        return PhaseCheckResult::Idle;
    };

    match ap_query.get(current.entity) {
        Ok(ap) => {
            if ap.is_idle() {
                debug!(
                    "[Combat] PhaseCheck: 单位 {:?} 空闲，跳过到结算阶段",
                    current.entity
                );
                PhaseCheckResult::Idle
            } else {
                debug!(
                    "[Combat] PhaseCheck: 单位 {:?} 有可用行动，等待输入",
                    current.entity
                );
                PhaseCheckResult::HasActions
            }
        }
        Err(_) => {
            debug!(
                "[Combat] PhaseCheck: 单位 {:?} 没有 ActionPoints，跳过",
                current.entity
            );
            PhaseCheckResult::Idle
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Step 3: UnitAction (pause point)
// ═══════════════════════════════════════════════════════════════════════

/// UnitAction 步骤：此步骤由驾驶员处理（暂停等待），此处仅做日志记录。
pub(crate) fn step_unit_action(_commands: &mut Commands, turn_queue: &TurnQueue) {
    let Some(current) = turn_queue.current() else {
        return;
    };
    debug!(
        "[Combat] UnitAction: 等待输入，单位={:?}",
        current.entity
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Step 4: TurnSettlement
// ═══════════════════════════════════════════════════════════════════════

/// TurnSettlement 步骤：触发 OnTurnEnd 领域事件。
pub(crate) fn step_turn_settlement(commands: &mut Commands, turn_queue: &TurnQueue) {
    let Some(current) = turn_queue.current() else {
        debug!("[Combat] TurnSettlement: 回合队列为空，跳过");
        return;
    };

    // 发射 OnTurnEnd 领域事件（同步触发 Observers: effects tick, ability cooldowns）
    commands.trigger(OnTurnEnd {
        unit: current.entity,
    });

    // 发射全局 TurnEnded 事件（供其他 Domain 订阅，避免跨域直接依赖）
    commands.trigger(TurnEnded {
        unit: current.entity,
    });

    debug!(
        "[Combat] TurnSettlement: 单位 {:?} 结算完成",
        current.entity
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Step 5: TurnEnd
// ═══════════════════════════════════════════════════════════════════════

/// TurnEnd 步骤执行结果。
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TurnEndResult {
    /// 战斗结束（一方全灭）
    BattleOver,
    /// 继续下一单位回合
    Continue,
}

/// TurnEnd 步骤：切换到下一个单位/队伍。
///
/// 返回值指示驾驶员是否应结束循环。
pub(crate) fn step_turn_end(
    commands: &mut Commands,
    turn_queue: &mut TurnQueue,
    combatant_query: &Query<&CombatParticipant>,
    dead_query: &Query<&CombatParticipant, With<Dead>>,
) -> TurnEndResult {
    if turn_queue.is_empty() {
        debug!("[Combat] TurnEnd: 回合队列为空，战斗结束");
        return TurnEndResult::BattleOver;
    }

    // 记录当前信息（在 advance 之前）
    let changed_team = turn_queue.just_changed_team();
    let was_last_in_round = turn_queue.current_index() == turn_queue.len() - 1;
    let prev_team = turn_queue.current_team().cloned();

    // 前进到下一个单位
    turn_queue.advance();

    let round = turn_queue.round_number();

    debug!(
        "[Combat] TurnEnd: 前进到 index={}, 回合={}",
        turn_queue.current_index(),
        round
    );

    // 如果切换队伍 → 发射 BetweenTurns
    if changed_team && let Some(team) = prev_team {
        commands.trigger(BetweenTurns { team });
    }

    // 如果所有队伍完成一轮 → 发射 OnRoundEnd
    if was_last_in_round {
        commands.trigger(OnRoundEnd { round });
    }

    // 胜负判定 — 检查是否仅剩 ≤1 个队伍存活
    if check_team_elimination(combatant_query, dead_query) {
        debug!("[Combat] 胜负判定：战斗结束（活跃队伍 ≤1）");
        commands.trigger(OnBattleEnd {
            result: BattleResult::Victory,
        });
        TurnEndResult::BattleOver
    } else {
        TurnEndResult::Continue
    }
}

/// 检查是否满足队伍全灭胜利条件。
///
/// 遍历所有 CombatParticipant，统计各队伍的总数和存活数。
/// 如果活跃队伍数 ≤1，返回 true（战斗结束）。
/// 检查是否有一方队伍被全灭。
///
/// 通过 `Without<Dead>` 过滤器判定存活单位，而非检查 `is_alive` 字段。
pub(crate) fn check_team_elimination(
    all_query: &Query<&CombatParticipant>,
    dead_query: &Query<&CombatParticipant, With<Dead>>,
) -> bool {
    let mut team_total = std::collections::HashMap::<String, usize>::new();
    let mut team_alive = std::collections::HashMap::<String, usize>::new();

    for participant in all_query.iter() {
        let team = participant.team_id.0.clone();
        *team_total.entry(team.clone()).or_insert(0) += 1;
        *team_alive.entry(team).or_insert(0) += 1;
    }

    for participant in dead_query.iter() {
        let team = participant.team_id.0.clone();
        if let Some(alive) = team_alive.get_mut(&team) {
            *alive = alive.saturating_sub(1);
        }
    }

    let alive_teams = team_alive.values().filter(|&&count| count > 0).count();
    alive_teams <= 1
}
