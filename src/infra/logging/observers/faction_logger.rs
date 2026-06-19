//! faction_logger — Faction 域日志 Observer
//!
//! 监听声望变化、阵营关系事件，生成 INFO 日志。

use bevy::prelude::*;

use crate::core::domains::faction::events::{
    FactionRelationChanged, RelationshipEvaluated, ReputationChanged, ReputationLevelUp,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 声望变化日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::FAC001, event = "reputation_changed"))]
pub(crate) fn on_reputation_changed(trigger: On<ReputationChanged>) {
    metrics::record(LogCode::FAC001);
    let event = trigger.event();
    info!(
        code = ?LogCode::FAC001,
        event = "reputation_changed",
        entity = ?event.entity,
        faction = %event.faction_id,
        old = event.old_value,
        new = event.new_value,
        reason = %event.reason,
        "reputation_changed"
    );
}

/// 阵营关系变化日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::FAC002, event = "faction_relation_changed"))]
pub(crate) fn on_faction_relation_changed(trigger: On<FactionRelationChanged>) {
    metrics::record(LogCode::FAC002);
    let event = trigger.event();
    info!(
        code = ?LogCode::FAC002,
        event = "faction_relation_changed",
        faction_a = %event.faction_a,
        faction_b = %event.faction_b,
        new_relation = ?event.new_relation,
        "faction_relation_changed"
    );
}

/// 声望等级提升日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::FAC003, event = "reputation_level_up"))]
pub(crate) fn on_reputation_level_up(trigger: On<ReputationLevelUp>) {
    metrics::record(LogCode::FAC003);
    let event = trigger.event();
    info!(
        code = ?LogCode::FAC003,
        event = "reputation_level_up",
        entity = ?event.entity,
        faction = %event.faction_id,
        new_level = ?event.new_level,
        "reputation_level_up"
    );
}

/// 关系判定完成日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::FAC004, event = "relationship_evaluated"))]
pub(crate) fn on_relationship_evaluated(trigger: On<RelationshipEvaluated>) {
    metrics::record(LogCode::FAC004);
    let event = trigger.event();
    info!(
        code = ?LogCode::FAC004,
        event = "relationship_evaluated",
        entity = ?event.entity,
        final_state = ?event.final_state,
        "relationship_evaluated"
    );
}
