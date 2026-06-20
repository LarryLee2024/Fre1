//! party_logger — Party 域日志 Observer
//!
//! 监听队伍成员变更、羁绊事件，生成 INFO 日志。

use bevy::prelude::*;

use crate::core::domains::party::events::{
    BondActivated, BondDeactivated, MemberJoined, MemberRemoved, MemberSwapped,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 成员加入日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRY001, event = "成员加入"), target = "party")]
pub(crate) fn on_member_joined(trigger: On<MemberJoined>) {
    metrics::record(LogCode::PRY001);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRY001,
        event = "成员加入",
        entity = ?event.entity,
        role = %event.role,
        "成员加入"
    );
}

/// 成员离开日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRY002, event = "成员离开"), target = "party")]
pub(crate) fn on_member_removed(trigger: On<MemberRemoved>) {
    metrics::record(LogCode::PRY002);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRY002,
        event = "成员离开",
        entity = ?event.entity,
        reason = %event.reason,
        "成员离开"
    );
}

/// 换人日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRY003, event = "换人"), target = "party")]
pub(crate) fn on_member_swapped(trigger: On<MemberSwapped>) {
    metrics::record(LogCode::PRY003);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRY003,
        event = "换人",
        outgoing = ?event.outgoing,
        incoming = ?event.incoming,
        "换人"
    );
}

/// 羁绊激活日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRY004, event = "羁绊激活"), target = "party")]
pub(crate) fn on_bond_activated(trigger: On<BondActivated>) {
    metrics::record(LogCode::PRY004);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRY004,
        event = "羁绊激活",
        bond_id = %event.bond_id,
        members = ?event.members,
        "羁绊激活"
    );
}

/// 羁绊解除日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRY005, event = "羁绊解除"), target = "party")]
pub(crate) fn on_bond_deactivated(trigger: On<BondDeactivated>) {
    metrics::record(LogCode::PRY005);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRY005,
        event = "羁绊解除",
        bond_id = %event.bond_id,
        reason = %event.reason,
        "羁绊解除"
    );
}
