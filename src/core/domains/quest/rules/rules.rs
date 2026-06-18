//! 任务业务规则 — 纯函数
//!
//! 包括前置条件检查、进度规则、奖励发放检查等。
//! 详见 docs/02-domain/domains/quest_domain.md §3

use super::super::components::{QuestDefId, QuestEntry, QuestLog, QuestState};

// ─── 前置条件检查 ──────────────────────────────────────────────────

/// 检查任务的前置条件是否全部满足。
///
/// 不变量 3.1：任务的所有前置条件必须全部满足。
///
/// # 参数
/// - `_quest_id`: 要检查的任务
/// - `_quest_log`: 当前任务日志
/// - `_player_level`: 玩家等级
///
/// # 返回值
/// `Ok(())` 或失败原因。
///
/// # 注意
/// 这是一个骨架函数。完整的前置条件检查需要 Condition 领域支持。
pub fn check_prerequisites(
    _quest_id: &QuestDefId,
    _quest_log: &QuestLog,
    _player_level: u32,
) -> Result<(), String> {
    // TODO: 接入 Condition 领域进行完整的前置条件评估
    // 当前简化实现：检查前置任务是否已完成
    Ok(())
}

// ─── 进度规则 ──────────────────────────────────────────────────────

/// 检查进度是否单调递增。
///
/// 不变量 3.2：目标进度只增不减。
///
/// # 参数
/// - `current`: 当前进度
/// - `previous`: 先前进度
///
/// # 返回值
/// `true` 如果进度没有倒退。
pub fn check_progress_monotonic(current: u32, previous: u32) -> bool {
    current >= previous
}

/// 检查是否所有目标已完成。
pub fn are_all_objectives_completed(entry: &QuestEntry) -> bool {
    entry.all_objectives_completed()
}

/// 检查任务是否可交付（Active 状态且所有目标完成）。
pub fn can_turn_in(entry: &QuestEntry) -> bool {
    entry.state == QuestState::Active && entry.all_objectives_completed()
}

// ─── 奖励规则 ──────────────────────────────────────────────────────

/// 检查奖励是否已发放。
///
/// 不变量 3.3：每个任务完成后只发放一次奖励。
///
/// # 参数
/// - `entry`: 任务条目
///
/// # 返回值
/// `true` 如果奖励已经发放。
pub fn is_reward_already_granted(entry: &QuestEntry) -> bool {
    entry.is_reward_granted()
}

/// 检查玩家是否可以接受新任务。
///
/// 不变量 3.4：互斥任务不能同时处于 Active 状态。
///
/// # 参数
/// - `quest_id`: 要接受的任务
/// - `exclusive_with`: 互斥任务列表
/// - `quest_log`: 当前任务日志
///
/// # 返回值
/// `Ok(())` 或互斥任务名。
pub fn check_exclusivity(
    quest_id: &QuestDefId,
    exclusive_with: &[QuestDefId],
    quest_log: &QuestLog,
) -> Result<(), QuestDefId> {
    for exclusive_id in exclusive_with {
        if let Some(entry) = quest_log.get_entry(exclusive_id) {
            if entry.state == QuestState::Active && *exclusive_id != *quest_id {
                return Err(exclusive_id.clone());
            }
        }
    }
    Ok(())
}

/// 检查关键任务是否可以放弃。
///
/// 不变量 3.5：标记为"关键"的任务不可被放弃或失败。
///
/// # 参数
/// - `is_critical`: 任务是否为关键任务。
///
/// # 返回值
/// `true` 如果可以安全放弃。
pub fn can_abandon_quest(is_critical: bool) -> bool {
    !is_critical
}

// ─── 状态转换检查 ────────────────────────────────────────────────

/// 检查任务是否可以从当前状态转换到目标状态。
///
/// 有效的状态转换：
/// - Unavailable -> Available
/// - Available -> Active
/// - Active -> Completed
/// - Active -> Failed
/// - Failed -> Available (重置)
pub fn can_transition(current: &QuestState, target: &QuestState) -> bool {
    match (current, target) {
        (QuestState::Unavailable, QuestState::Available) => true,
        (QuestState::Available, QuestState::Active) => true,
        (QuestState::Active, QuestState::Completed) => true,
        (QuestState::Active, QuestState::Failed) => true,
        (QuestState::Failed, QuestState::Available) => true,
        _ => false,
    }
}

// ─── 进度计算 ────────────────────────────────────────────────────

/// 计算任务的整体完成百分比。
pub fn calc_completion_percentage(entry: &QuestEntry) -> f32 {
    if entry.objective_progress.is_empty() {
        return 0.0;
    }
    let total: u32 = entry
        .objective_progress
        .iter()
        .map(|p| p.current_value.min(p.target_value))
        .sum();
    let max: u32 = entry
        .objective_progress
        .iter()
        .map(|p| p.target_value.max(1))
        .sum();
    if max == 0 {
        return 0.0;
    }
    (total as f32 / max as f32) * 100.0
}
