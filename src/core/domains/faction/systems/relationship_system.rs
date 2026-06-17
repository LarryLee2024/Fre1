//! Relationship System — 关系判定系统
//!
//! 监听关系评估请求，计算个体对阵营/其他实体的综合关系状态。
//! 详见 docs/02-domain/domains/faction_domain.md §5.2

use bevy::prelude::*;

use crate::core::domains::faction::components::{
    FactionMembership, FactionRelationTable, RelationshipState, Reputation,
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

/// 响应关系评估请求，计算综合关系状态。
pub(crate) fn on_relationship_eval_request(
    trigger: On<RelationshipEvalRequest>,
    query: Query<(&FactionMembership, &Reputation)>,
    relation_table: Option<Res<FactionRelationTable>>,
    mut commands: Commands,
) {
    let req = trigger.event();
    let entity = req.entity;
    let target = req.target;

    let Ok((subj_membership, subj_reputation)) = query.get(entity) else {
        warn!(
            "[Faction] RelationshipEvalRequest: entity {:?} has no FactionMembership",
            entity
        );
        return;
    };

    let Ok((target_membership, _target_reputation)) = query.get(target) else {
        warn!(
            "[Faction] RelationshipEvalRequest: target {:?} has no FactionMembership",
            target
        );
        return;
    };

    let default_table = FactionRelationTable::new();
    let table = relation_table.as_deref().unwrap_or(&default_table);

    let base_relation = subj_membership
        .factions
        .iter()
        .find_map(|sf| {
            target_membership
                .factions
                .iter()
                .map(|tf| table.get_relation(sf, tf))
                .next()
        })
        .unwrap_or_default();

    let final_state =
        relationship_between_entities(subj_membership, subj_reputation, target_membership, table);

    // 取目标的主要阵营用于事件报告
    let faction_id = target_membership
        .factions
        .first()
        .cloned()
        .unwrap_or_else(|| crate::core::domains::faction::components::FactionId::new("unknown"));

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
