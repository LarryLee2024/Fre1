//! 关系判定规则 — 纯函数
//!
//! 根据阵营关系 + 声望值综合判定个体之间的关系状态。
//! 详见 docs/02-domain/domains/faction_domain.md §5.2

use crate::core::domains::faction::components::{
    FactionId, FactionMembership, FactionRelationTable, FactionRelationType, RelationshipState,
    Reputation,
};
use crate::core::domains::faction::rules::reputation::evaluate_relationship;

/// 判定两个实体之间的关系状态。
///
/// # 参数
/// - `subject_membership`: 主体实体的阵营归属
/// - `subject_reputation`: 主体实体的声望
/// - `target_membership`: 目标实体的阵营归属
/// - `target_faction`: 目标实体所属的主要阵营（用于声望查询）
/// - `relation_table`: 全局阵营关系表
///
/// # 返回值
/// 主体对目标的综合关系状态。
///
/// # 规则
/// 1. 如果双方共享至少一个阵营 → Allied
/// 2. 否则，取主体对目标所有所属阵营的关系中最高的一个
/// 3. 关系计算 = FactionRelationType + Reputation 修正
pub fn relationship_between_entities(
    subject_membership: &FactionMembership,
    subject_reputation: &Reputation,
    target_membership: &FactionMembership,
    relation_table: &FactionRelationTable,
) -> RelationshipState {
    // 如果双方共享阵营 → 强制 Allied
    for faction in &subject_membership.factions {
        if target_membership.is_member(faction) {
            return RelationshipState::Allied;
        }
    }

    // 取对目标所有阵营中关系最"强"（最敌对）的一个
    // 从 Allied（最弱）开始，让任意实际结果都能覆盖它
    let mut strongest = RelationshipState::Allied;

    for target_faction in &target_membership.factions {
        let state = relationship_with_faction(
            subject_membership,
            subject_reputation,
            target_faction,
            relation_table,
        );
        strongest = stronger_relationship(strongest, state);
    }

    strongest
}

/// 判定实体对指定阵营的关系状态。
///
/// 如果实体属于该阵营 → Allied。
/// 否则，取实体所属阵营与该阵营的关系中最高的一个，再叠加声望修正。
pub fn relationship_with_faction(
    subject_membership: &FactionMembership,
    subject_reputation: &Reputation,
    target_faction: &FactionId,
    relation_table: &FactionRelationTable,
) -> RelationshipState {
    // 如果主体就是该阵营成员 → Allied
    if subject_membership.is_member(target_faction) {
        return RelationshipState::Allied;
    }

    let mut strongest = RelationshipState::Neutral;

    // 遍历主体的所有阵营，取与目标阵营关系中最强的一个
    for subject_faction in &subject_membership.factions {
        let base = relation_table.get_relation(subject_faction, target_faction);
        let rep_level = subject_reputation.level(target_faction);
        let state = evaluate_relationship(base, rep_level);
        strongest = stronger_relationship(strongest, state);
    }

    // 如果主体无阵营归属，用其对该阵营的声望单独判定
    if subject_membership.factions.is_empty() {
        let rep_level = subject_reputation.level(target_faction);
        strongest = reputation_level_to_state(rep_level);
    }

    strongest
}

/// 关系状态优先级排序（从高到低：War > Hostile > Neutral > Allied）。
///
/// 返回两者中"更强"（更敌对）的关系。
fn stronger_relationship(a: RelationshipState, b: RelationshipState) -> RelationshipState {
    use RelationshipState::*;
    match (a, b) {
        (War, _) | (_, War) => War,
        (Hostile, _) | (_, Hostile) => Hostile,
        (Neutral, _) | (_, Neutral) => Neutral,
        _ => Allied,
    }
}

/// 声望等级单独映射到关系状态（无阵营关系时）。
fn reputation_level_to_state(
    level: crate::core::domains::faction::components::ReputationLevel,
) -> RelationshipState {
    use crate::core::domains::faction::components::ReputationLevel::*;
    match level {
        Hated | Hostile => RelationshipState::Hostile,
        Neutral => RelationshipState::Neutral,
        Friendly | Honored | Revered => RelationshipState::Allied,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_membership(factions: &[&str]) -> FactionMembership {
        let mut m = FactionMembership::new();
        for f in factions {
            m.join(FactionId::new(*f));
        }
        m
    }

    fn make_reputation(entries: &[(&str, i32)]) -> Reputation {
        let mut r = Reputation::new();
        for (f, v) in entries {
            r.set(FactionId::new(*f), *v);
        }
        r
    }

    #[test]
    fn shared_faction_is_allied() {
        let table = FactionRelationTable::new();
        let m1 = make_membership(&["faction_a"]);
        let m2 = make_membership(&["faction_a"]);
        let r1 = make_reputation(&[]);
        let r2 = make_reputation(&[]);

        let state = relationship_between_entities(&m1, &r1, &m2, &table);
        assert_eq!(state, RelationshipState::Allied);
    }

    #[test]
    fn hostile_relation_without_reputation() {
        let mut table = FactionRelationTable::new();
        table.set_relation(
            FactionId::new("faction_a"),
            FactionId::new("faction_b"),
            FactionRelationType::Hostile,
        );

        let m1 = make_membership(&["faction_a"]);
        let m2 = make_membership(&["faction_b"]);
        let r1 = make_reputation(&[]);

        let state = relationship_between_entities(&m1, &r1, &m2, &table);
        assert_eq!(state, RelationshipState::Hostile);
    }

    #[test]
    fn revered_reputation_overrides_hostile() {
        let mut table = FactionRelationTable::new();
        table.set_relation(
            FactionId::new("faction_a"),
            FactionId::new("faction_b"),
            FactionRelationType::Hostile,
        );

        let m1 = make_membership(&["faction_a"]);
        let m2 = make_membership(&["faction_b"]);
        let r1 = make_reputation(&[("faction_b", 90)]); // Revered

        let state = relationship_between_entities(&m1, &r1, &m2, &table);
        // Revered 声望使 Hostile 阵营关系缓和为 Neutral
        assert_eq!(state, RelationshipState::Neutral);
    }

    #[test]
    fn war_overrides_everything() {
        let mut table = FactionRelationTable::new();
        table.set_relation(
            FactionId::new("faction_a"),
            FactionId::new("faction_b"),
            FactionRelationType::War,
        );

        let m1 = make_membership(&["faction_a"]);
        let m2 = make_membership(&["faction_b"]);
        let r1 = make_reputation(&[("faction_b", 100)]); // Revered

        let state = relationship_between_entities(&m1, &r1, &m2, &table);
        // War 无视声望
        assert_eq!(state, RelationshipState::War);
    }

    #[test]
    fn no_faction_uses_reputation() {
        let table = FactionRelationTable::new();
        let m1 = make_membership(&[]); // 无阵营
        let m2 = make_membership(&["faction_b"]);
        let r1 = make_reputation(&[("faction_b", 50)]); // Honored

        let state = relationship_between_entities(&m1, &r1, &m2, &table);
        assert_eq!(state, RelationshipState::Allied);
    }

    #[test]
    fn stronger_relationship_logic() {
        use RelationshipState::*;
        assert_eq!(stronger_relationship(War, Allied), War);
        assert_eq!(stronger_relationship(Hostile, Neutral), Hostile);
        assert_eq!(stronger_relationship(Neutral, Allied), Neutral);
        assert_eq!(stronger_relationship(Allied, Allied), Allied);
    }
}
