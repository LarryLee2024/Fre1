//! ScenePlugin — 场景管理 Plugin
//!
//! 注册 GameState、StateTransitionQueue、以及 Last 调度中的队列处理系统。
//! 各场景的 OnEnter/OnExit 系统通过 `SceneRegister` trait 集中注册。
//!
//! 详见 ADR-050 §4: 场景生命周期。

use bevy::prelude::*;

use super::game_over::{GameOverScreen, GameOverScreenAction};
use super::queue::{StateTransitionQueue, process_transition_queue};
use super::register::{SceneRegister, empty};
use super::result::{ResultScreen, ResultScreenAction};
use super::state::GameState;
use crate::core::domains::combat::components::BattlePhase;

/// 场景管理 Plugin——注册 GameState、StateTransitionQueue 和场景切换系统。
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_sub_state::<BattlePhase>()
            .insert_resource(StateTransitionQueue::default())
            .add_systems(Last, process_transition_queue);

        // ── 场景生命周期注册 ──
        // 每个场景自动获得：
        //   OnEnter: setup_scene_root + 场景自定义系统
        //   OnExit:  cleanup_scene + 场景自定义系统
        app.register_scene(GameState::MainMenu, empty, empty);
        app.register_scene(
            GameState::PartySetup,
            super::party_setup::spawn_party_setup,
            super::party_setup::despawn_party_setup,
        );
        app.register_scene(GameState::TacticalMap, empty, empty);
        app.register_scene(GameState::Combat, empty, empty);
        app.register_scene(
            GameState::Result,
            super::result::spawn_result_screen,
            super::result::despawn_result_screen,
        );
        app.register_scene(GameState::CampRest, empty, empty);
        app.register_scene(
            GameState::GameOver,
            super::game_over::spawn_game_over_screen,
            super::game_over::despawn_game_over_screen,
        );

        // ── Reflect types ──
        app.register_type::<ResultScreen>();
        app.register_type::<ResultScreenAction>();
        app.register_type::<GameOverScreen>();
        app.register_type::<GameOverScreenAction>();

        // ── Game command observers ──
        app.add_observer(super::game_setup::on_new_game_command);
        app.add_observer(super::open_menu::on_open_menu_command);

        // ── PartySetup button handlers ──
        app.add_observer(super::party_setup::on_party_setup_button);

        // ── Battle lifecycle observers ──
        app.add_observer(super::battle_end::on_battle_ended);

        // ── Result / GameOver button handlers ──
        app.add_observer(super::result::on_result_screen_button);
        app.add_observer(super::game_over::on_game_over_screen_button);
    }
}
