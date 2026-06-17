//! 声望计算规则 — 纯函数
//!
//! 声望值 clamp、等级转换、阈值检查。
//! 详见 docs/02-domain/domains/faction_domain.md §1, §3

use crate::core::domains::faction::components::{
    FactionRelationType, KeyCharacter, RelationshipState, ReputationLevel,
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
    if reputation_level == ReputationLevel::Honored || reputation_level == ReputationLevel::Revered
    {
        if base_relation == FactionRelationType::Hostile {
            return RelationshipState::Neutral;
        }
    }

    // 声望 <= -50 时，即使阵营关系为 Allied，个体也无法交易/对话
    if reputation_level == ReputationLevel::Hated {
        if base_relation == FactionRelationType::Allied {
            return RelationshipState::Hostile;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_reputation_boundaries() {
        assert_eq!(clamp_reputation(-150), -100);
        assert_eq!(clamp_reputation(150), 100);
        assert_eq!(clamp_reputation(0), 0);
        assert_eq!(clamp_reputation(-100), -100);
        assert_eq!(clamp_reputation(100), 100);
    }

    #[test]
    fn apply_change_with_clamp() {
        assert_eq!(apply_reputation_change(95, 10), 100);
        assert_eq!(apply_reputation_change(-95, -10), -100);
        assert_eq!(apply_reputation_change(50, -20), 30);
    }

    #[test]
    fn level_change_detection() {
        // 从中立(-10)到友好(+10)跨越阈值
        assert_eq!(
            check_level_change(-5, 15),
            Some((ReputationLevel::Neutral, ReputationLevel::Friendly))
        );

        // 未跨越阈值
        assert_eq!(check_level_change(0, 5), None);

        // 从敌对(-50)到仇恨(-100)
        assert_eq!(
            check_level_change(-20, -60),
            Some((ReputationLevel::Hostile, ReputationLevel::Hated))
        );
    }

    #[test]
    fn key_character_protection() {
        // 普通角色可以自由降低声望
        assert!(is_reputation_change_allowed(0, -100, false));

        // 关键角色不能降到 -50 以下
        assert!(is_reputation_change_allowed(0, -40, true));
        assert!(!is_reputation_change_allowed(0, -60, true));
        assert!(!is_reputation_change_allowed(-40, -20, true));

        // 关键角色声望提升始终允许
        assert!(is_reputation_change_allowed(-40, 10, true));
    }

    #[test]
    fn safe_change_with_protection() {
        assert_eq!(safe_reputation_change(0, -40, true), Some(-40));
        assert_eq!(safe_reputation_change(0, -60, true), None);
        assert_eq!(safe_reputation_change(0, -60, false), Some(-60));
    }

    #[test]
    fn reputation_level_mapping() {
        assert_eq!(
            reputation_level_to_state(ReputationLevel::Hated),
            RelationshipState::Hostile
        );
        assert_eq!(
            reputation_level_to_state(ReputationLevel::Neutral),
            RelationshipState::Neutral
        );
        assert_eq!(
            reputation_level_to_state(ReputationLevel::Revered),
            RelationshipState::Allied
        );
    }

    #[test]
    fn evaluate_allied_with_hated_reputation() {
        // 阵营关系 Allied + 声望 Hated → Hostile（声望压制）
        let state = evaluate_relationship(FactionRelationType::Allied, ReputationLevel::Hated);
        assert_eq!(state, RelationshipState::Hostile);
    }

    #[test]
    fn evaluate_hostile_with_revered_reputation() {
        // 阵营关系 Hostile + 声望 Revered → Neutral（声望缓和）
        let state = evaluate_relationship(FactionRelationType::Hostile, ReputationLevel::Revered);
        assert_eq!(state, RelationshipState::Neutral);
    }

    #[test]
    fn evaluate_war_overrides_reputation() {
        // 战争状态无视声望
        for level in [
            ReputationLevel::Hated,
            ReputationLevel::Hostile,
            ReputationLevel::Neutral,
            ReputationLevel::Friendly,
            ReputationLevel::Honored,
            ReputationLevel::Revered,
        ] {
            assert_eq!(
                evaluate_relationship(FactionRelationType::War, level),
                RelationshipState::War
            );
        }
    }

    #[test]
    fn evaluate_neutral_faction() {
        // 阵营 Neutral 时，仅两个特殊例外生效：
        // - Hostile 声望使关系 Hostile
        // - 正向声望不会改善 Neutral（domain doc §5.2：正向仅缓和 Hostile）
        assert_eq!(
            evaluate_relationship(FactionRelationType::Neutral, ReputationLevel::Hostile),
            RelationshipState::Hostile
        );
        assert_eq!(
            evaluate_relationship(FactionRelationType::Neutral, ReputationLevel::Neutral),
            RelationshipState::Neutral
        );
        // 阵营 Neutral + Friendly → Neutral（无特殊例外，取更敌对的一方）
        assert_eq!(
            evaluate_relationship(FactionRelationType::Neutral, ReputationLevel::Friendly),
            RelationshipState::Neutral
        );
    }
}
