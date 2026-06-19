//! ScenePlugin — 场景管理 Plugin
//!
//! 注册 GameState、StateTransitionQueue、以及 Last 调度中的队列处理系统。
//! 各场景的 OnEnter/OnExit 系统在本 Plugin 中集中注册。
//!
//! 详见 ADR-050 §4: 场景生命周期。

use bevy::prelude::*;

use super::queue::{StateTransitionQueue, cleanup_scene, process_transition_queue};
use super::state::GameState;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(StateTransitionQueue::default())
            .add_systems(Last, process_transition_queue);

        // ── 场景生命周期桩系统 ──
        // 各场景 OnEnter/OnExit 将在 E-3 阶段填充具体逻辑。
        // 先注册空的 OnExit cleanup，保证场景切换不会泄漏实体。
        app.add_systems(OnExit(GameState::MainMenu), cleanup_scene);
        app.add_systems(OnExit(GameState::PartySetup), cleanup_scene);
        app.add_systems(OnExit(GameState::TacticalMap), cleanup_scene);
        app.add_systems(OnExit(GameState::Combat), cleanup_scene);
        app.add_systems(OnExit(GameState::Result), cleanup_scene);
        app.add_systems(OnExit(GameState::CampRest), cleanup_scene);
        app.add_systems(OnExit(GameState::GameOver), cleanup_scene);
    }
}
