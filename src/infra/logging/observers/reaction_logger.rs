//! reaction_logger — Reaction 域日志 Observer
//!
//! 监听反应触发、执行、拒绝事件，生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量（但 target 因 tracing 限制需重复指定，见下文）
//! - 不使用 `context_desc` 等高基数字段
//!
//! # target 重复说明
//!
//! `#[instrument(target = "domain.reaction")]` 与 `info!(target = "domain.reaction", ...)`
//! 两处都需要指定 target。这是因为 tracing 的 Event 不会继承父 Span 的 target，
//! 去掉 info! 的 target 会使 event target 退化为模块路径（如 `infra::logging::observers::reaction_logger`），
//! 破坏按 `domain.reaction` 的日志聚合。
//!
//! 这是当前模式的已知冗余，后续通过 `telemetry::emit` 扩展可消除。

use bevy::prelude::*;

use crate::core::domains::reaction::events::{
    CounterspellExecuted, GuardianUsed, OpportunityAttackExecuted, ReactionDeclined,
    ReactionExecuted, ReactionTriggered, ShieldUsed,
};
use crate::infra::logging::telemetry;
use crate::shared::diagnostics::LogCode;

/// 反应触发日志 Observer。
#[tracing::instrument(skip_all, target = "domain.reaction", fields(
    code = ?LogCode::RCT001,
    event = "reaction_triggered",
))]
pub(crate) fn on_reaction_triggered(trigger: On<ReactionTriggered>) {
    telemetry::emit(LogCode::RCT001);
    let event = trigger.event();
    info!(
        target = "domain.reaction",
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
    telemetry::emit(LogCode::RCT002);
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
    telemetry::emit(LogCode::RCT003);
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
    telemetry::emit(LogCode::RCT004);
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
    telemetry::emit(LogCode::RCT005);
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
    code = ?LogCode::RCT006,
    event = "shield_used",
))]
pub(crate) fn on_shield_used(trigger: On<ShieldUsed>) {
    telemetry::emit(LogCode::RCT006);
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
    code = ?LogCode::RCT007,
    event = "guardian_used",
))]
pub(crate) fn on_guardian_used(trigger: On<GuardianUsed>) {
    telemetry::emit(LogCode::RCT007);
    let event = trigger.event();
    info!(
        target = "domain.reaction",
        guardian = ?event.guardian,
        target = ?event.target,
        damage = event.transferred_damage,
        "援护格挡",
    );
}
