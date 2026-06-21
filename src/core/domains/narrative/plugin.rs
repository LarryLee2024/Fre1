//! NarrativePlugin — 叙事/对话领域 Plugin
//!
//! 注册对话组件、事件和系统。
//! 处理对话流程、故事标记、演出管理。

use bevy::prelude::*;

use super::components::{
    CutsceneState, DialogueHistory, DialogueState, DialogueTreeRegistry, StoryFlags,
};
use super::systems::cutscene_system::{on_cutscene_ended, on_cutscene_start};
use super::systems::dialogue_system::{on_choice_select, on_dialogue_start};
use super::systems::story_flag_system::on_story_flag_set;
use crate::register_domain_types;
use crate::shared::game_state::GameState;

/// 叙事/对话领域 Plugin——注册对话树、StoryFlag 组件和对话流程系统。
pub struct NarrativePlugin;

impl Plugin for NarrativePlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Component 类型 ──
        register_domain_types!(app, [DialogueState, StoryFlags, CutsceneState,]);

        // ── 初始化 Resource ──
        app.init_resource::<DialogueTreeRegistry>();
        app.init_resource::<DialogueHistory>();

        // ── 注册 Observer System ──
        app.add_observer(on_dialogue_start);
        app.add_observer(on_choice_select);
        app.add_observer(on_story_flag_set);
        app.add_observer(on_cutscene_start);
        app.add_observer(on_cutscene_ended);

        // ── 注册常规 System ──
        // cutscene_progress_system 已替换为 on_cutscene_ended (Delayed Commands)
    }
}
