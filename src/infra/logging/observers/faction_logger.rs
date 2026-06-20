//! faction_logger — Faction 域日志 Observer
//!
//! 监听声望变化、阵营关系事件，生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::domains::faction::events::{
    FactionRelationChanged, RelationshipEvaluated, ReputationChanged, ReputationLevelUp,
};
use crate::infra::logging::telemetry;
use crate::shared::diagnostics::LogCode;

/// 声望变化日志 Observer。
#[tracing::instrument(skip_all, target = "domain.faction", fields(
    code = ?LogCode::FAC001,
    event = "reputation_changed",
))]
pub(crate) fn on_reputation_changed(trigger: On<ReputationChanged>) {
    telemetry::emit(LogCode::FAC001);
    let event = trigger.event();
    info!(
        target = "domain.faction",
        entity = ?event.entity,
        faction = %event.faction_id,
        old = event.old_value,
        new = event.new_value,
        reason = %event.reason,
        "声望变化",
    );
}

/// 阵营关系变化日志 Observer。
#[tracing::instrument(skip_all, target = "domain.faction", fields(
    code = ?LogCode::FAC002,
    event = "faction_relation_changed",
))]
pub(crate) fn on_faction_relation_changed(trigger: On<FactionRelationChanged>) {
    telemetry::emit(LogCode::FAC002);
    let event = trigger.event();
    info!(
        target = "domain.faction",
        faction_a = %event.faction_a,
        faction_b = %event.faction_b,
        new_relation = ?event.new_relation,
        "阵营关系变化",
    );
}

/// 声望等级提升日志 Observer。
#[tracing::instrument(skip_all, target = "domain.faction", fields(
    code = ?LogCode::FAC003,
    event = "reputation_tier_raised",
))]
pub(crate) fn on_reputation_level_up(trigger: On<ReputationLevelUp>) {
    telemetry::emit(LogCode::FAC003);
    let event = trigger.event();
    info!(
        target = "domain.faction",
        entity = ?event.entity,
        faction = %event.faction_id,
        new_level = ?event.new_level,
        "声望等级提升",
    );
}

/// 关系判定完成日志 Observer。
#[tracing::instrument(skip_all, target = "domain.faction", fields(
    code = ?LogCode::FAC004,
    event = "relation_assessed",
))]
pub(crate) fn on_relationship_evaluated(trigger: On<RelationshipEvaluated>) {
    telemetry::emit(LogCode::FAC004);
    let event = trigger.event();
    info!(
        target = "domain.faction",
        entity = ?event.entity,
        final_state = ?event.final_state,
        "关系判定",
    );
}
