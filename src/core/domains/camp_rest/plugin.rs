//! CampRestPlugin — 营地/休息领域 Plugin
//!
//! 注册休息组件、事件和系统。
//! 处理短休、长休、生命骰管理、营地事件。
//!
//! 详见 ADR-031

use bevy::prelude::*;

use super::components::{CampRestMarker, HitDicePool, RestState};
use super::systems::{
    CampEventRegistry, handle_long_rest_complete, handle_long_rest_interrupted,
    handle_short_rest_complete, process_camp_events,
};
use crate::register_domain_types;
use crate::shared::game_state::GameState;

/// 营地/休息业务领域 Plugin。
///
/// 注册休息组件、事件和系统。
/// 处理短休、长休、生命骰管理、营地事件。
pub struct CampRestPlugin;

impl Plugin for CampRestPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Component 类型 ──
        register_domain_types!(app, [RestState, HitDicePool, CampRestMarker,]);

        // ── 初始化 Resource ──
        app.init_resource::<CampEventRegistry>();

        // ── 注册 Observer System ──
        app.add_observer(handle_short_rest_complete);
        app.add_observer(handle_long_rest_complete);
        app.add_observer(handle_long_rest_interrupted);

        // ── 注册普通 System ──
        app.add_systems(
            Update,
            process_camp_events.run_if(in_state(GameState::CampRest)),
        );
    }
}
