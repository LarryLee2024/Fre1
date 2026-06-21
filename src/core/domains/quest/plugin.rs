//! QuestPlugin — 任务领域 Plugin
//!
//! 注册任务组件、事件和系统。
//! 管理任务日志、目标追踪、奖励发放。
//!
//! 详见 docs/02-domain/domains/quest_domain.md

use bevy::prelude::*;

use super::components::QuestLog;
use super::integration::on_quest_command;
use super::resources::QuestConfig;
use super::systems::{
    on_accept_quest_request, on_advance_objective, on_quest_failed, on_turn_in_quest,
};
use crate::register_domain_types;

/// 任务领域 Plugin——注册任务状态、目标追踪组件和任务流程系统。
pub struct QuestPlugin;

impl Plugin for QuestPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Component 类型 ──
        register_domain_types!(app, [QuestLog,]);

        // ── 初始化 Resource ──
        app.init_resource::<QuestConfig>();

        // ── 注册 Observer System ──
        app.add_observer(on_quest_command);
        app.add_observer(on_accept_quest_request);
        app.add_observer(on_advance_objective);
        app.add_observer(on_turn_in_quest);
        app.add_observer(on_quest_failed);
    }
}
