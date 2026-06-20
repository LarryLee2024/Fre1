//! ReactionPlugin — 反应领域 Plugin
//!
//! 注册反应组件、事件和系统。
//! 管理反应槽位、反应队列、回合重置。
//!
//! 详见 docs/02-domain/domains/reaction_domain.md

use bevy::prelude::*;

use super::components::ReactionState;
use super::resources::{GlobalReactionQueue, ReactionConfig};
use super::systems::{
    cleanup_reaction_queue, on_opportunity_attack_executed, process_reaction_queue,
    reset_reactions_on_turn_start,
};
use crate::app::scenes::GameState;
use crate::register_domain_types;

pub struct ReactionPlugin;

impl Plugin for ReactionPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Component 类型 ──
        register_domain_types!(app, [ReactionState,]);

        // ── 初始化 Resource ──
        app.init_resource::<ReactionConfig>();
        app.init_resource::<GlobalReactionQueue>();

        // ── 注册 Observer System ──
        app.add_observer(on_opportunity_attack_executed);

        // ── 注册 Update System ──
        app.add_systems(
            First,
            reset_reactions_on_turn_start.run_if(in_state(GameState::Combat)),
        );
        app.add_systems(
            Update,
            process_reaction_queue.run_if(in_state(GameState::Combat)),
        );
        app.add_systems(
            Last,
            cleanup_reaction_queue.run_if(in_state(GameState::Combat)),
        );
    }
}
