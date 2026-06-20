//! progression_logger — Progression 域日志 Observer
//!
//! 监听经验、升级、天赋事件，生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::domains::progression::events::{
    ASICompleted, ClassGained, ExperienceGained, LevelUp, SubclassChosen, TalentUnlocked,
};
use crate::infra::logging::telemetry;
use crate::shared::diagnostics::LogCode;

/// 经验获得日志 Observer。
#[tracing::instrument(skip_all, target = "domain.progression", fields(
    code = ?LogCode::PRG001,
    event = "experience_gained",
))]
pub(crate) fn on_experience_gained(trigger: On<ExperienceGained>) {
    telemetry::emit(LogCode::PRG001);
    let event = trigger.event();
    info!(
        target = "domain.progression",
        entity = ?event.entity,
        amount = event.amount,
        source = %event.source,
        level = event.current_level,
        "经验获得",
    );
}

/// 升级日志 Observer。
#[tracing::instrument(skip_all, target = "domain.progression", fields(
    code = ?LogCode::PRG002,
    event = "level_up",
))]
pub(crate) fn on_level_up(trigger: On<LevelUp>) {
    telemetry::emit(LogCode::PRG002);
    let event = trigger.event();
    info!(
        target = "domain.progression",
        entity = ?event.entity,
        old = event.old_level,
        new = event.new_level,
        asi = event.is_asi_level,
        "角色升级",
    );
}

/// 天赋解锁日志 Observer。
#[tracing::instrument(skip_all, target = "domain.progression", fields(
    code = ?LogCode::PRG003,
    event = "talent_unlocked",
))]
pub(crate) fn on_talent_unlocked(trigger: On<TalentUnlocked>) {
    telemetry::emit(LogCode::PRG003);
    let event = trigger.event();
    info!(
        target = "domain.progression",
        entity = ?event.entity,
        talent_id = %event.talent_id,
        "天赋解锁",
    );
}

/// 子职选择日志 Observer。
#[tracing::instrument(skip_all, target = "domain.progression", fields(
    code = ?LogCode::PRG004,
    event = "subclass_chosen",
))]
pub(crate) fn on_subclass_chosen(trigger: On<SubclassChosen>) {
    telemetry::emit(LogCode::PRG004);
    let event = trigger.event();
    info!(
        target = "domain.progression",
        entity = ?event.entity,
        subclass_id = %event.subclass_id,
        "子职选择",
    );
}

/// ASI 完成日志 Observer。
#[tracing::instrument(skip_all, target = "domain.progression", fields(
    code = ?LogCode::PRG005,
    event = "asi_completed",
))]
pub(crate) fn on_asi_completed(trigger: On<ASICompleted>) {
    telemetry::emit(LogCode::PRG005);
    let event = trigger.event();
    info!(
        target = "domain.progression",
        entity = ?event.entity,
        level = event.level,
        choices = event.choices.len(),
        "属性提升完成",
    );
}

/// 职业等级获得日志 Observer。
#[tracing::instrument(skip_all, target = "domain.progression", fields(
    code = ?LogCode::PRG006,
    event = "class_gained",
))]
pub(crate) fn on_class_gained(trigger: On<ClassGained>) {
    telemetry::emit(LogCode::PRG006);
    let event = trigger.event();
    info!(
        target = "domain.progression",
        entity = ?event.entity,
        class_id = %event.class_id,
        level = event.new_level,
        "职业等级获得",
    );
}
