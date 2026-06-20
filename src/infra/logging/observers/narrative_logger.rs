//! narrative_logger — Narrative 域日志 Observer
//!
//! 监听对话、选择、过场事件，生成 INFO 日志。
//!
//! # 规范
//! - `#[instrument(fields(...))]` 声明不变量（code、event）
//! - `info!()` 只放变量字段，不重复不变量

use bevy::prelude::*;

use crate::core::domains::narrative::events::{
    ChoiceMade, CutsceneEnded, CutsceneStarted, DialogueStarted, StoryFlagSet,
};
use crate::infra::logging::telemetry;
use crate::shared::diagnostics::LogCode;

/// 对话开始日志 Observer。
#[tracing::instrument(skip_all, target = "domain.narrative", fields(
    code = ?LogCode::NAR001,
    event = "dialogue_started",
))]
pub(crate) fn on_dialogue_started(trigger: On<DialogueStarted>) {
    telemetry::emit(LogCode::NAR001);
    let event = trigger.event();
    info!(
        target = "domain.narrative",
        entity = ?event.entity,
        npc = ?event.npc,
        tree_id = %event.tree_id,
        "对话开始",
    );
}

/// 玩家选择分支日志 Observer。
#[tracing::instrument(skip_all, target = "domain.narrative", fields(
    code = ?LogCode::NAR002,
    event = "choice_made",
))]
pub(crate) fn on_choice_made(trigger: On<ChoiceMade>) {
    telemetry::emit(LogCode::NAR002);
    let event = trigger.event();
    info!(
        target = "domain.narrative",
        entity = ?event.entity,
        choice_id = %event.choice_id,
        flags_set = event.story_flags_set.len(),
        "选择分支",
    );
}

/// 故事标记设置日志 Observer。
#[tracing::instrument(skip_all, target = "domain.narrative", fields(
    code = ?LogCode::NAR003,
    event = "story_flag_set",
))]
pub(crate) fn on_story_flag_set(trigger: On<StoryFlagSet>) {
    telemetry::emit(LogCode::NAR003);
    let event = trigger.event();
    info!(
        target = "domain.narrative",
        flag_id = %event.flag_id,
        value = %event.value,
        source = %event.source,
        "故事标记设置",
    );
}

/// 过场动画开始日志 Observer。
#[tracing::instrument(skip_all, target = "domain.narrative", fields(
    code = ?LogCode::NAR004,
    event = "cutscene_started",
))]
pub(crate) fn on_cutscene_started(trigger: On<CutsceneStarted>) {
    telemetry::emit(LogCode::NAR004);
    let event = trigger.event();
    info!(
        target = "domain.narrative",
        cutscene_id = %event.cutscene_id,
        duration = event.duration,
        "过场动画开始",
    );
}

/// 过场动画结束日志 Observer。
#[tracing::instrument(skip_all, target = "domain.narrative", fields(
    code = ?LogCode::NAR005,
    event = "cutscene_ended",
))]
pub(crate) fn on_cutscene_ended(trigger: On<CutsceneEnded>) {
    telemetry::emit(LogCode::NAR005);
    let event = trigger.event();
    info!(
        target = "domain.narrative",
        cutscene_id = %event.cutscene_id,
        "过场动画结束",
    );
}
