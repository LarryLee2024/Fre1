//! reaction_logger — Reaction 域日志 Observer
//!
//! 监听反应触发、执行、拒绝事件，生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量
//! - 不使用 `context_desc` 等高基数字段
//!
//! # 说明
//!
//! emit_info! 宏已自动从 LogCode 派生 target，消除 info!() 中的 target 字面量重复。

use bevy::prelude::*;

use crate::core::domains::reaction::events::{
    CounterspellExecuted, GuardianUsed, OpportunityAttackExecuted, ReactionDeclined,
    ReactionExecuted, ReactionTriggered, ShieldUsed,
};
use crate::emit_info;
use crate::shared::diagnostics::LogCode;

/// 反应触发日志 Observer。
#[tracing::instrument(skip_all, target = "domain.reaction", fields(
    code = ?LogCode::RCT001,
    event = "reaction_triggered",
))]
pub(crate) fn on_reaction_triggered(trigger: On<ReactionTriggered>) {
    let event = trigger.event();
    emit_info!(
        LogCode::RCT001,
        reactor = ?event.reactor,
        reaction_type = %event.reaction_type.log_name(),
        "反应触发",
    );
}

/// 反应执行日志 Observer。
#[tracing::instrument(skip_all, target = "domain.reaction", fields(
    code = ?LogCode::RCT002,
    event = "reaction_executed",
))]
pub(crate) fn on_reaction_executed(trigger: On<ReactionExecuted>) {
    let event = trigger.event();
    emit_info!(
        LogCode::RCT002,
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
    let event = trigger.event();
    emit_info!(
        LogCode::RCT003,
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
    let event = trigger.event();
    emit_info!(
        LogCode::RCT004,
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
    let event = trigger.event();
    emit_info!(
        LogCode::RCT005,
        counterer = ?event.counterer,
        target_spell = %event.target_spell,
        success = event.success,
        "法术反制",
    );
}

/// 护盾术日志 Observer。
#[tracing::instrument(skip_all, target = "domain.reaction", fields(
    code = ?LogCode::RCT006,
    event = "shield_used",
))]
pub(crate) fn on_shield_used(trigger: On<ShieldUsed>) {
    let event = trigger.event();
    emit_info!(
        LogCode::RCT006,
        caster = ?event.caster,
        ac_bonus = event.ac_bonus,
        still_hit = event.still_hit,
        "护盾术",
    );
}

/// 援护格挡日志 Observer。
#[tracing::instrument(skip_all, target = "domain.reaction", fields(
    code = ?LogCode::RCT007,
    event = "guardian_used",
))]
pub(crate) fn on_guardian_used(trigger: On<GuardianUsed>) {
    let event = trigger.event();
    emit_info!(
        LogCode::RCT007,
        guardian = ?event.guardian,
        target = ?event.target,
        damage = event.transferred_damage,
        "援护格挡",
    );
}
