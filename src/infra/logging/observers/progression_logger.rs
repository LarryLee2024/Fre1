//! progression_logger — Progression 域日志 Observer
//!
//! 监听经验、升级、天赋事件，生成 INFO 日志。

use bevy::prelude::*;

use crate::core::domains::progression::events::{
    ASICompleted, ClassGained, ExperienceGained, LevelUp, SubclassChosen, TalentUnlocked,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 经验获得日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRG001, event = "获得经验"))]
pub(crate) fn on_experience_gained(trigger: On<ExperienceGained>) {
    metrics::record(LogCode::PRG001);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRG001,
        event = "获得经验",
        entity = ?event.entity,
        amount = event.amount,
        source = %event.source,
        level = event.current_level,
        "获得经验"
    );
}

/// 升级日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRG002, event = "升级"))]
pub(crate) fn on_level_up(trigger: On<LevelUp>) {
    metrics::record(LogCode::PRG002);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRG002,
        event = "升级",
        entity = ?event.entity,
        old = event.old_level,
        new = event.new_level,
        asi = event.is_asi_level,
        "升级"
    );
}

/// 天赋解锁日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRG003, event = "天赋解锁"))]
pub(crate) fn on_talent_unlocked(trigger: On<TalentUnlocked>) {
    metrics::record(LogCode::PRG003);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRG003,
        event = "天赋解锁",
        entity = ?event.entity,
        talent_id = %event.talent_id,
        "天赋解锁"
    );
}

/// 子职选择日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRG004, event = "子职选择"))]
pub(crate) fn on_subclass_chosen(trigger: On<SubclassChosen>) {
    metrics::record(LogCode::PRG004);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRG004,
        event = "子职选择",
        entity = ?event.entity,
        subclass_id = %event.subclass_id,
        "子职选择"
    );
}

/// ASI 完成日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRG005, event = "属性提升完成"))]
pub(crate) fn on_asi_completed(trigger: On<ASICompleted>) {
    metrics::record(LogCode::PRG005);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRG005,
        event = "属性提升完成",
        entity = ?event.entity,
        level = event.level,
        choices = event.choices.len(),
        "属性提升完成"
    );
}

/// 职业等级获得日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::PRG006, event = "获得职业等级"))]
pub(crate) fn on_class_gained(trigger: On<ClassGained>) {
    metrics::record(LogCode::PRG006);
    let event = trigger.event();
    info!(
        code = ?LogCode::PRG006,
        event = "获得职业等级",
        entity = ?event.entity,
        class_id = %event.class_id,
        level = event.new_level,
        "获得职业等级"
    );
}
