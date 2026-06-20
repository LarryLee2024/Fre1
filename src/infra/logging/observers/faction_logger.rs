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
#[tracing::instrument(skip_all, fields(code = ?LogCode::FAC001, event = "声望变化"), target = "faction")]
pub(crate) fn on_reputation_changed(trigger: On<ReputationChanged>) {
    metrics::record(LogCode::FAC001);
    let event = trigger.event();
    info!(
        code = ?LogCode::FAC001,
        event = "声望变化",
        entity = ?event.entity,
        faction = %event.faction_id,
        old = event.old_value,
        new = event.new_value,
        reason = %event.reason,
        "声望变化"
    );
}

/// 阵营关系变化日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::FAC002, event = "阵营关系变化"), target = "faction")]
pub(crate) fn on_faction_relation_changed(trigger: On<FactionRelationChanged>) {
    metrics::record(LogCode::FAC002);
    let event = trigger.event();
    info!(
        code = ?LogCode::FAC002,
        event = "阵营关系变化",
        faction_a = %event.faction_a,
        faction_b = %event.faction_b,
        new_relation = ?event.new_relation,
        "阵营关系变化"
    );
}

/// 声望等级提升日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::FAC003, event = "声望等级提升"), target = "faction")]
pub(crate) fn on_reputation_level_up(trigger: On<ReputationLevelUp>) {
    metrics::record(LogCode::FAC003);
    let event = trigger.event();
    info!(
        code = ?LogCode::FAC003,
        event = "声望等级提升",
        entity = ?event.entity,
        faction = %event.faction_id,
        new_level = ?event.new_level,
        "声望等级提升"
    );
}

/// 关系判定完成日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::FAC004, event = "关系判定完成"), target = "faction")]
pub(crate) fn on_relationship_evaluated(trigger: On<RelationshipEvaluated>) {
    metrics::record(LogCode::FAC004);
    let event = trigger.event();
    info!(
        code = ?LogCode::FAC004,
        event = "关系判定完成",
        entity = ?event.entity,
        final_state = ?event.final_state,
        "关系判定完成"
    );
}
