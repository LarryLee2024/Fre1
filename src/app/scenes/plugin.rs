//! ScenePlugin — 场景管理 Plugin
//!
//! 注册 GameState、StateTransitionQueue、以及 Last 调度中的队列处理系统。
//! 各场景的 OnEnter/OnExit 系统在本 Plugin 中集中注册。
//!
//! 详见 ADR-050 §4: 场景生命周期。

use bevy::prelude::*;

use super::components::SceneRoot;
use super::queue::{StateTransitionQueue, cleanup_scene, process_transition_queue};
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

        // ── 场景生命周期桩系统 ──
        // OnEnter: 创建场景根实体（带 SceneRoot 标记），后续由具体场景系统填充子实体。
        // OnExit: cleanup_scene 统一 despawn 所有 SceneRoot 实体。
        //
        // TODO[P2][Scene]: 各场景 OnEnter 填充具体逻辑（UI 生成、资源加载等）
        app.add_systems(OnEnter(GameState::MainMenu), setup_scene_root);
        app.add_systems(OnEnter(GameState::PartySetup), setup_scene_root);
        app.add_systems(OnEnter(GameState::TacticalMap), setup_scene_root);
        app.add_systems(OnEnter(GameState::Combat), setup_scene_root);
        app.add_systems(OnEnter(GameState::Result), setup_scene_root);
        app.add_systems(OnEnter(GameState::CampRest), setup_scene_root);
        app.add_systems(OnEnter(GameState::GameOver), setup_scene_root);

        app.add_systems(OnExit(GameState::MainMenu), cleanup_scene);
        app.add_systems(OnExit(GameState::PartySetup), cleanup_scene);
        app.add_systems(OnExit(GameState::TacticalMap), cleanup_scene);
        app.add_systems(OnExit(GameState::Combat), cleanup_scene);
        app.add_systems(OnExit(GameState::Result), cleanup_scene);
        app.add_systems(OnExit(GameState::CampRest), cleanup_scene);
        app.add_systems(OnExit(GameState::GameOver), cleanup_scene);
    }
}

/// 场景根实体创建桩系统。
///
/// 为当前场景创建一个带 `SceneRoot` 标记的根实体，
/// 后续由具体场景系统在此实体下填充子实体（UI、摄像机等）。
/// OnExit 时 `cleanup_scene` 通过 `SceneRoot` 标记 despawn 整个场景子树。
fn setup_scene_root(mut commands: Commands) {
    commands.spawn(SceneRoot);
}
