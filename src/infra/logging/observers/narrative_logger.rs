//! narrative_logger — Narrative 域日志 Observer
//!
//! 监听对话、选择、过场事件，生成 INFO 日志。

use bevy::prelude::*;

use crate::core::domains::narrative::events::{
    ChoiceMade, CutsceneEnded, CutsceneStarted, DialogueStarted, StoryFlagSet,
};
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

/// 对话开始日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::NAR001, event = "dialogue_started"))]
pub(crate) fn on_dialogue_started(trigger: On<DialogueStarted>) {
    metrics::record(LogCode::NAR001);
    let event = trigger.event();
    info!(
        code = ?LogCode::NAR001,
        event = "dialogue_started",
        entity = ?event.entity,
        npc = ?event.npc,
        tree_id = %event.tree_id,
        "dialogue_started"
    );
}

/// 玩家选择分支日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::NAR002, event = "choice_made"))]
pub(crate) fn on_choice_made(trigger: On<ChoiceMade>) {
    metrics::record(LogCode::NAR002);
    let event = trigger.event();
    info!(
        code = ?LogCode::NAR002,
        event = "choice_made",
        entity = ?event.entity,
        choice_id = %event.choice_id,
        flags_set = event.story_flags_set.len(),
        "choice_made"
    );
}

/// 故事标记设置日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::NAR003, event = "story_flag_set"))]
pub(crate) fn on_story_flag_set(trigger: On<StoryFlagSet>) {
    metrics::record(LogCode::NAR003);
    let event = trigger.event();
    info!(
        code = ?LogCode::NAR003,
        event = "story_flag_set",
        flag_id = %event.flag_id,
        value = %event.value,
        source = %event.source,
        "story_flag_set"
    );
}

/// 过场动画开始日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::NAR004, event = "cutscene_started"))]
pub(crate) fn on_cutscene_started(trigger: On<CutsceneStarted>) {
    metrics::record(LogCode::NAR004);
    let event = trigger.event();
    info!(
        code = ?LogCode::NAR004,
        event = "cutscene_started",
        cutscene_id = %event.cutscene_id,
        duration = event.duration,
        "cutscene_started"
    );
}

/// 过场动画结束日志 Observer。
#[tracing::instrument(skip_all, fields(code = ?LogCode::NAR005, event = "cutscene_ended"))]
pub(crate) fn on_cutscene_ended(trigger: On<CutsceneEnded>) {
    metrics::record(LogCode::NAR005);
    let event = trigger.event();
    info!(
        code = ?LogCode::NAR005,
        event = "cutscene_ended",
        cutscene_id = %event.cutscene_id,
        "cutscene_ended"
    );
}
