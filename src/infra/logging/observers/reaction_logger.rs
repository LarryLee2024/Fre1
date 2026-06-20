//! reaction_logger — Reaction 域日志 Observer
//!
//! 监听反应触发、执行、拒绝事件，生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::domains::reaction::events::{
    CounterspellExecuted, GuardianUsed, OpportunityAttackExecuted, ReactionDeclined,
    ReactionExecuted, ReactionTriggered, ShieldUsed,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 反应触发日志 Observer。
#[tracing::instrument(skip_all, target = "domain.reaction", fields(
    code = ?LogCode::RCT001,
    event = "reaction_triggered",
))]
pub(crate) fn on_reaction_triggered(trigger: On<ReactionTriggered>) {
    metrics::record(LogCode::RCT001);
    let event = trigger.event();
    info!(
        target = "domain.reaction",
        reactor = ?event.reactor,
        reaction_type = ?event.reaction_type,
        "反应触发",
    );
}

/// 反应执行日志 Observer。
#[tracing::instrument(skip_all, target = "domain.reaction", fields(
    code = ?LogCode::RCT002,
    event = "reaction_executed",
))]
pub(crate) fn on_reaction_executed(trigger: On<ReactionExecuted>) {
    metrics::record(LogCode::RCT002);
    let event = trigger.event();
    info!(
        target = "domain.reaction",
        reactor = ?event.reactor,
        result = %event.result,
        "反应执行",
    );
}

/// 反应拒绝日志 Observer。
#[tracing::instrument(skip_all, target = "domain.reaction", fields(
    code = ?LogCode::RCT003,
    event = "reaction_declined",
))]
pub(crate) fn on_reaction_declined(trigger: On<ReactionDeclined>) {
    metrics::record(LogCode::RCT003);
    let event = trigger.event();
    info!(
        target = "domain.reaction",
        reactor = ?event.reactor,
        reason = %event.reason,
        "反应拒绝",
    );
}

/// 机会攻击日志 Observer。
#[tracing::instrument(skip_all, target = "domain.reaction", fields(
    code = ?LogCode::RCT004,
    event = "opportunity_attack_executed",
))]
pub(crate) fn on_opportunity_attack(trigger: On<OpportunityAttackExecuted>) {
    metrics::record(LogCode::RCT004);
    let event = trigger.event();
    info!(
        target = "domain.reaction",
        attacker = ?event.attacker,
        target = ?event.target,
        hit = event.hit,
        damage = event.damage,
        critical = event.critical,
        "机会攻击",
    );
}

/// 法术反制日志 Observer。
#[tracing::instrument(skip_all, target = "domain.reaction", fields(
    code = ?LogCode::RCT005,
    event = "counterspell_executed",
))]
pub(crate) fn on_counterspell(trigger: On<CounterspellExecuted>) {
    metrics::record(LogCode::RCT005);
    let event = trigger.event();
    info!(
        target = "domain.reaction",
        counterer = ?event.counterer,
        target_spell = %event.target_spell,
        success = event.success,
        "法术反制",
    );
}

/// 护盾术日志 Observer。
#[tracing::instrument(skip_all, target = "domain.reaction", fields(
    code = ?LogCode::RCT005,
    event = "shield_used",
))]
pub(crate) fn on_shield_used(trigger: On<ShieldUsed>) {
    metrics::record(LogCode::RCT005);
    let event = trigger.event();
    info!(
        target = "domain.reaction",
        caster = ?event.caster,
        ac_bonus = event.ac_bonus,
        still_hit = event.still_hit,
        "护盾术",
    );
}

/// 援护格挡日志 Observer。
#[tracing::instrument(skip_all, target = "domain.reaction", fields(
    code = ?LogCode::RCT005,
    event = "guardian_used",
))]
pub(crate) fn on_guardian_used(trigger: On<GuardianUsed>) {
    metrics::record(LogCode::RCT005);
    let event = trigger.event();
    info!(
        target = "domain.reaction",
        guardian = ?event.guardian,
        target = ?event.target,
        damage = event.transferred_damage,
        "援护格挡",
    );
}
