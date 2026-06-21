//! Relationship System — 关系判定系统
//!
//! 监听关系评估请求，计算个体对阵营/其他实体的综合关系状态。
//! 详见 docs/02-domain/domains/faction_domain.md §5.2

use bevy::prelude::*;

use crate::core::domains::faction::components::{
    FactionId, FactionMembership, FactionRelationTable, FactionRelationType, RelationshipState,
    Reputation,
};
use crate::core::domains::faction::events::RelationshipEvaluated;
use crate::core::domains::faction::rules::relationship::relationship_between_entities;

/// 关系评估请求事件。
///
/// 外部系统通过触发此事件来请求关系判定，本 Observer 评估并发出 RelationshipEvaluated 事件。
#[derive(Event, Debug, Clone)]
pub struct RelationshipEvalRequest {
    /// 评估主体实体
    pub entity: Entity,
    /// 目标实体
    pub target: Entity,
}

/// 取两个 FactionRelationType 中更敌对（优先级更高）的一个。
///
/// 优先级：War > Hostile > Neutral > Allied
fn stronger_faction_relation(
    a: FactionRelationType,
    b: FactionRelationType,
) -> FactionRelationType {
    use FactionRelationType::*;
    match (a, b) {
        (War, _) | (_, War) => War,
        (Hostile, _) | (_, Hostile) => Hostile,
        (Neutral, _) | (_, Neutral) => Neutral,
        _ => Allied,
    }
}

/// 响应关系评估请求，计算综合关系状态。
pub(crate) fn on_relationship_eval_request(
    trigger: On<RelationshipEvalRequest>,
    query: Query<(&FactionMembership, &Reputation)>,
    relation_table: Res<FactionRelationTable>,
    mut commands: Commands,
) {
    let req = trigger.event();
    let entity = req.entity;
    let target = req.target;

    let Ok((subj_membership, subj_reputation)) = query.get(entity) else {
        tracing::warn!(target: "faction",
            event = "faction.relationship_eval.missing_subject",
            entity = ?entity,
            "RelationshipEvalRequest: 主体 {:?} 没有 FactionMembership",
            entity
        );
        return;
    };

    let Ok((target_membership, _target_reputation)) = query.get(target) else {
        tracing::warn!(target: "faction",
            event = "faction.relationship_eval.missing_target",
            entity = ?entity,
            target = ?target,
            "RelationshipEvalRequest: 目标 {:?} 没有 FactionMembership",
            target
        );
        return;
    };

    // 多阵营组合取最强关系：一个单位可能属于多个阵营，与目标的关系取最敌对的那个
    let mut base_relation = FactionRelationType::default();
    for sf in &subj_membership.factions {
        for tf in &target_membership.factions {
            let rel = relation_table.get_relation(sf, tf);
            base_relation = stronger_faction_relation(base_relation, rel);
        }
    }

    let final_state = relationship_between_entities(
        subj_membership,
        subj_reputation,
        target_membership,
        &relation_table,
    );

    // 取目标的主要阵营用于事件报告
    let faction_id = target_membership
        .factions
        .first()
        .cloned()
        .unwrap_or_else(|| FactionId::new("unknown"));

    let rep_level = subj_reputation.level(&faction_id);

    commands.trigger(RelationshipEvaluated {
        entity,
        target_entity: Some(target),
        faction_id,
        base_relation,
        reputation_level: rep_level,
        final_state,
    });
}

/// 直接关系查询函数（非 Observer，供其他 System 内联调用）。
///
/// 比触发事件更高效，适用于高频查询场景。
pub fn evaluate_relationship_direct(
    subject: &FactionMembership,
    subject_reputation: &Reputation,
    target: &FactionMembership,
    relation_table: &FactionRelationTable,
) -> RelationshipState {
    relationship_between_entities(subject, subject_reputation, target, relation_table)
}
