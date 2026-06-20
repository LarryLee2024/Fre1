//! SpellPlugin — 法术领域 Plugin
//!
//! 注册法术组件、事件和系统。
//! 处理法术位管理、施法流程、专注维护。
//!
//! 详见 ADR-023

use bevy::prelude::*;

use super::components::{Concentration, SpellConfig, SpellSlotPool, Spellbook};
use super::systems::{on_spell_cast_request, tick_concentration_duration};
use crate::app::scenes::GameState;
use crate::register_domain_types;

pub struct SpellPlugin;

impl Plugin for SpellPlugin {
    fn build(&self, app: &mut App) {
        // ── 注册 Component 类型 ──
        register_domain_types!(app, [SpellSlotPool, Spellbook, Concentration,]);

        // ── 初始化 Resource ──
        app.init_resource::<SpellConfig>();

        // ── 注册 Observer System ──
        app.add_observer(on_spell_cast_request);

        // ── 注册 Update System ──
        app.add_systems(
            Update,
            tick_concentration_duration.run_if(in_state(GameState::Combat)),
        );
    }
}
