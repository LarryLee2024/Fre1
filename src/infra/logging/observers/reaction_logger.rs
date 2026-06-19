//! reaction_logger — Reaction 域日志 Observer
//!
//! 监听反应触发、执行、拒绝事件，生成 INFO 日志。

use bevy::prelude::*;

use crate::core::domains::reaction::events::{
    CounterspellExecuted, GuardianUsed, OpportunityAttackExecuted, ReactionDeclined,
    ReactionExecuted, ReactionTriggered, ShieldUsed,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 反应触发日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::RCT001, event = "reaction_triggered"))]
pub(crate) fn on_reaction_triggered(trigger: On<ReactionTriggered>) {
    metrics::record(LogCode::RCT001);
    let event = trigger.event();
    info!(
        code = ?LogCode::RCT001,
        event = "reaction_triggered",
        reactor = ?event.reactor,
        reaction_type = ?event.reaction_type,
        "reaction_triggered"
    );
}

/// 反应执行日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::RCT002, event = "reaction_executed"))]
pub(crate) fn on_reaction_executed(trigger: On<ReactionExecuted>) {
    metrics::record(LogCode::RCT002);
    let event = trigger.event();
    info!(
        code = ?LogCode::RCT002,
        event = "reaction_executed",
        reactor = ?event.reactor,
        result = %event.result,
        "reaction_executed"
    );
}

/// 反应拒绝日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::RCT003, event = "reaction_declined"))]
pub(crate) fn on_reaction_declined(trigger: On<ReactionDeclined>) {
    metrics::record(LogCode::RCT003);
    let event = trigger.event();
    info!(
        code = ?LogCode::RCT003,
        event = "reaction_declined",
        reactor = ?event.reactor,
        reason = %event.reason,
        "reaction_declined"
    );
}

/// 机会攻击日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::RCT004, event = "opportunity_attack"))]
pub(crate) fn on_opportunity_attack(trigger: On<OpportunityAttackExecuted>) {
    metrics::record(LogCode::RCT004);
    let event = trigger.event();
    info!(
        code = ?LogCode::RCT004,
        event = "opportunity_attack",
        attacker = ?event.attacker,
        target = ?event.target,
        hit = event.hit,
        damage = event.damage,
        critical = event.critical,
        "opportunity_attack"
    );
}

/// 法术反制日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::RCT005, event = "counterspell"))]
pub(crate) fn on_counterspell(trigger: On<CounterspellExecuted>) {
    metrics::record(LogCode::RCT005);
    let event = trigger.event();
    info!(
        code = ?LogCode::RCT005,
        event = "counterspell",
        counterer = ?event.counterer,
        target_spell = %event.target_spell,
        success = event.success,
        "counterspell"
    );
}

/// 护盾术日志 Observer。
pub(crate) fn on_shield_used(trigger: On<ShieldUsed>) {
    let event = trigger.event();
    info!(
        code = ?LogCode::RCT005,
        event = "shield_used",
        caster = ?event.caster,
        ac_bonus = event.ac_bonus,
        still_hit = event.still_hit,
        "shield_used"
    );
}

/// 援护格挡日志 Observer。
pub(crate) fn on_guardian_used(trigger: On<GuardianUsed>) {
    let event = trigger.event();
    info!(
        code = ?LogCode::RCT005,
        event = "guardian_used",
        guardian = ?event.guardian,
        target = ?event.target,
        damage = event.transferred_damage,
        "guardian_used"
    );
}
