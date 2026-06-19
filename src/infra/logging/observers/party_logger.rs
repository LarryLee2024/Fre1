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
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRY001, event = "member_joined"))]
pub(crate) fn on_member_joined(trigger: On<MemberJoined>) {
    metrics::record(LogCode::PRY001);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRY001,
        event = "member_joined",
        entity = ?event.entity,
        role = %event.role,
        "member_joined"
    );
}

/// 成员离开日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRY002, event = "member_removed"))]
pub(crate) fn on_member_removed(trigger: On<MemberRemoved>) {
    metrics::record(LogCode::PRY002);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRY002,
        event = "member_removed",
        entity = ?event.entity,
        reason = %event.reason,
        "member_removed"
    );
}

/// 换人日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRY003, event = "member_swapped"))]
pub(crate) fn on_member_swapped(trigger: On<MemberSwapped>) {
    metrics::record(LogCode::PRY003);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRY003,
        event = "member_swapped",
        outgoing = ?event.outgoing,
        incoming = ?event.incoming,
        "member_swapped"
    );
}

/// 羁绊激活日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRY004, event = "bond_activated"))]
pub(crate) fn on_bond_activated(trigger: On<BondActivated>) {
    metrics::record(LogCode::PRY004);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRY004,
        event = "bond_activated",
        bond_id = %event.bond_id,
        members = ?event.members,
        "bond_activated"
    );
}

/// 羁绊解除日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRY005, event = "bond_deactivated"))]
pub(crate) fn on_bond_deactivated(trigger: On<BondDeactivated>) {
    metrics::record(LogCode::PRY005);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRY005,
        event = "bond_deactivated",
        bond_id = %event.bond_id,
        reason = %event.reason,
        "bond_deactivated"
    );
}
