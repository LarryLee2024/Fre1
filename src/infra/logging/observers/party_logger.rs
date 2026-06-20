//! party_logger — Party 域日志 Observer
//!
//! 监听队伍成员变更、羁绊事件，生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::domains::party::events::{
    BondActivated, BondDeactivated, MemberJoined, MemberRemoved, MemberSwapped,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 成员加入日志 Observer。
#[tracing::instrument(skip_all, target = "domain.party", fields(
    code = ?LogCode::PRY001,
    event = "member_joined",
))]
pub(crate) fn on_member_joined(trigger: On<MemberJoined>) {
    metrics::record(LogCode::PRY001);
    let event = trigger.event();
    info!(
        target = "domain.party",
        entity = ?event.entity,
        role = %event.role,
        "成员加入",
    );
}

/// 成员离开日志 Observer。
#[tracing::instrument(skip_all, target = "domain.party", fields(
    code = ?LogCode::PRY002,
    event = "member_left",
))]
pub(crate) fn on_member_removed(trigger: On<MemberRemoved>) {
    metrics::record(LogCode::PRY002);
    let event = trigger.event();
    info!(
        target = "domain.party",
        entity = ?event.entity,
        reason = %event.reason,
        "成员离开",
    );
}

/// 换人日志 Observer。
#[tracing::instrument(skip_all, target = "domain.party", fields(
    code = ?LogCode::PRY003,
    event = "battle_swap",
))]
pub(crate) fn on_member_swapped(trigger: On<MemberSwapped>) {
    metrics::record(LogCode::PRY003);
    let event = trigger.event();
    info!(
        target = "domain.party",
        outgoing = ?event.outgoing,
        incoming = ?event.incoming,
        "战斗换人",
    );
}

/// 羁绊激活日志 Observer。
#[tracing::instrument(skip_all, target = "domain.party", fields(
    code = ?LogCode::PRY004,
    event = "bond_activated",
))]
pub(crate) fn on_bond_activated(trigger: On<BondActivated>) {
    metrics::record(LogCode::PRY004);
    let event = trigger.event();
    info!(
        target = "domain.party",
        bond_id = %event.bond_id,
        members = ?event.members,
        "羁绊激活",
    );
}

/// 羁绊解除日志 Observer。
#[tracing::instrument(skip_all, target = "domain.party", fields(
    code = ?LogCode::PRY005,
    event = "bond_dissolved",
))]
pub(crate) fn on_bond_deactivated(trigger: On<BondDeactivated>) {
    metrics::record(LogCode::PRY005);
    let event = trigger.event();
    info!(
        target = "domain.party",
        bond_id = %event.bond_id,
        reason = %event.reason,
        "羁绊解除",
    );
}
