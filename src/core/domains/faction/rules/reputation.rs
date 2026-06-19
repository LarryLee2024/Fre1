//! 声望计算规则 — 纯函数
//!
//! 声望值 clamp、等级转换、阈值检查。
//! 详见 docs/02-domain/domains/faction_domain.md §1, §3

use crate::core::domains::faction::components::{
    FactionRelationType, RelationshipState, ReputationLevel,
};

/// 声望值边界。
pub const REPUTATION_MIN: i32 = -100;
pub const REPUTATION_MAX: i32 = 100;

/// 关键角色最低声望保护阈值（不变量 3.5）。
pub const KEY_CHARACTER_MIN_REPUTATION: i32 = -50;

/// Clamp 声望值到有效范围。
pub fn clamp_reputation(value: i32) -> i32 {
    value.clamp(REPUTATION_MIN, REPUTATION_MAX)
}

/// 计算声望变更后的新值（自动 clamp）。
///
/// # 参数
/// - `current`: 当前声望值
/// - `delta`: 变化量（可为负）
///
/// # 返回值
/// 变更后的声望值（已 clamp）
pub fn apply_reputation_change(current: i32, delta: i32) -> i32 {
    clamp_reputation(current + delta)
}

/// 检查声望变更是否会导致等级跨越。
///
/// # 返回值
/// - `Some((old_level, new_level))` — 如果等级发生变化
/// - `None` — 如果等级未变
pub fn check_level_change(
    old_value: i32,
    new_value: i32,
) -> Option<(ReputationLevel, ReputationLevel)> {
    let old_level = ReputationLevel::from_value(old_value);
    let new_level = ReputationLevel::from_value(new_value);
    if old_level != new_level {
        Some((old_level, new_level))
    } else {
        None
    }
}

/// 检查声望变更是否被允许（关键角色保护，不变量 3.5）。
///
/// 关键角色的声望不能降到 `KEY_CHARACTER_MIN_REPUTATION` 以下。
pub fn is_reputation_change_allowed(current: i32, delta: i32, is_key_character: bool) -> bool {
    if !is_key_character {
        return true;
    }
    let new_value = current + delta;
    // 如果变更后声望低于保护阈值，拒绝
    if new_value < KEY_CHARACTER_MIN_REPUTATION {
        return false;
    }
    true
}

/// 计算带保护的声望变更。
///
/// 如果实体是关键角色且变更会触发保护，则返回 `None`。
/// 否则返回变更后的新值。
pub fn safe_reputation_change(current: i32, delta: i32, is_key_character: bool) -> Option<i32> {
    if !is_reputation_change_allowed(current, delta, is_key_character) {
        return None;
    }
    Some(apply_reputation_change(current, delta))
}

/// 声望等级到关系状态的映射（仅考虑声望维度）。
///
/// 当 FactionRelation 为 Neutral 时，Reputation 单独决定关系：
/// - Hated/Hostile → Hostile
/// - Neutral → Neutral
/// - Friendly/Honored/Revered → Allied
pub fn reputation_level_to_state(level: ReputationLevel) -> RelationshipState {
    match level {
        ReputationLevel::Hated | ReputationLevel::Hostile => RelationshipState::Hostile,
        ReputationLevel::Neutral => RelationshipState::Neutral,
        ReputationLevel::Friendly | ReputationLevel::Honored | ReputationLevel::Revered => {
            RelationshipState::Allied
        }
    }
}

/// 返回更敌对的关系（用于比较 War > Hostile > Neutral > Allied）。
fn stronger_relationship(a: RelationshipState, b: RelationshipState) -> RelationshipState {
    use RelationshipState::*;
    match (a, b) {
        (War, _) | (_, War) => War,
        (Hostile, _) | (_, Hostile) => Hostile,
        (Neutral, _) | (_, Neutral) => Neutral,
        _ => Allied,
    }
}

/// FactionRelationType → RelationshipState 映射。
fn relation_type_to_state(rel: FactionRelationType) -> RelationshipState {
    match rel {
        FactionRelationType::Allied => RelationshipState::Allied,
        FactionRelationType::Neutral => RelationshipState::Neutral,
        FactionRelationType::Hostile => RelationshipState::Hostile,
        FactionRelationType::War => RelationshipState::War,
    }
}

/// 综合判定最终关系状态。
///
/// 规则：
/// 1. 声望 >= +50 (Honored/Revered) 可缓和 Hostile 阵营关系到 Neutral
/// 2. 声望 <= -50 (Hated) 可破坏 Allied 关系到 Hostile
/// 3. War 关系无视声望
/// 4. 其他情况取"更敌对"的一方（War > Hostile > Neutral > Allied）
pub fn evaluate_relationship(
    base_relation: FactionRelationType,
    reputation_level: ReputationLevel,
) -> RelationshipState {
    // 声望 >= +50 时，即使阵营关系为 Hostile，个体也不被主动攻击
    if (reputation_level == ReputationLevel::Honored
        || reputation_level == ReputationLevel::Revered)
        && base_relation == FactionRelationType::Hostile
    {
        return RelationshipState::Neutral;
    }

    // 声望 <= -50 时，即使阵营关系为 Allied，个体也无法交易/对话
    if reputation_level == ReputationLevel::Hated && base_relation == FactionRelationType::Allied {
        return RelationshipState::Hostile;
    }

    // 阵营关系为 War 时，无视声望
    if base_relation == FactionRelationType::War {
        return RelationshipState::War;
    }

    // 其他情况：取更敌对的一方
    let rep_state = reputation_level_to_state(reputation_level);
    let base_state = relation_type_to_state(base_relation);
    stronger_relationship(base_state, rep_state)
}
