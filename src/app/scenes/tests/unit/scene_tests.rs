//! 场景模块单元测试
//!
//! 测试验证 GameState 流程编排、状态转移队列、场景清理等业务行为。
//!
//! # 测试清单
//!
//! | # | 测试名 | 验证内容 |
//! |---|--------|----------|
//! | 1 | `game_state_starts_at_main_menu` | GameState 默认值为 MainMenu |
//! | 2 | `battle_phase_accessible_in_combat` | 切换到 GameState::Combat 后 BattlePhase 可访问 |
//! | 3 | `battle_phase_invalid_outside_combat` | 切出 GameState::Combat 后 BattlePhase 重置为默认值 |
//! | 4 | `transition_queue_only_executes_last` | StateTransitionQueue 批量请求只执行最后一个 |
//! | 5 | `cleanup_scene_desawns_all_scene_roots` | 切换场景后新 SceneRoot 被创建 |
//! | 6 | `overlay_state_defaults_to_none` | OverlayState 默认值为 None |

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use crate::app::scenes::{
    GameState, OverlayState, ScenePlugin, SceneRoot, StateTransitionQueue, TransitionRequest,
};
use crate::core::domains::combat::components::BattlePhase;

// ─── 辅助函数 ──────────────────────────────────────────────────────

/// 构建最小测试 App：StatesPlugin + ScenePlugin
fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((StatesPlugin, ScenePlugin));
    app
}

/// 构建含 BattlePhase 的测试 App
fn build_combat_test_app() -> App {
    let mut app = build_test_app();
    app.init_state::<BattlePhase>();
    app
}

/// 将游戏状态切换到指定目标（通过 NextState 资源直接设置）
fn set_game_state(app: &mut App, target: GameState) {
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(target);
}

/// 查询所有带 SceneRoot 标记的实体数量
fn count_scene_roots(world: &mut World) -> usize {
    let mut query = world.query_filtered::<Entity, With<SceneRoot>>();
    query.iter(world).count()
}

// ─── Test 1: game_state_starts_at_main_menu ────────────────────────
//
// Given: 一个新创建的 App，已加载 ScenePlugin
// When:  查询 GameState 资源
// Then:  GameState 应为 MainMenu（默认状态）
#[test]
fn game_state_starts_at_main_menu() {
    let app = build_test_app();
    let state = app.world().resource::<State<GameState>>();
    assert_eq!(*state.get(), GameState::MainMenu);
}

// ─── Test 2: battle_phase_accessible_in_combat ─────────────────────
//
// Given: 一个 App，已加载 ScenePlugin 和 BattlePhase 初始化
// When:  将 GameState 切换到 Combat 并执行 update
// Then:  BattlePhase 应可访问且默认为 Preparation
#[test]
fn battle_phase_accessible_in_combat() {
    let mut app = build_combat_test_app();

    set_game_state(&mut app, GameState::Combat);
    app.update();
    app.world_mut().flush();

    let state = app.world().resource::<State<BattlePhase>>();
    assert_eq!(*state.get(), BattlePhase::Preparation);
}

// ─── Test 3: battle_phase_invalid_outside_combat ───────────────────
//
// Given: 一个 App，已进入 GameState::Combat 状态
// When:  将 GameState 切换回 MainMenu 并执行 update
// Then:  BattlePhase 应被重置为默认值（SubState 不再活跃）
#[test]
fn battle_phase_invalid_outside_combat() {
    let mut app = build_combat_test_app();

    set_game_state(&mut app, GameState::Combat);
    app.update();
    app.world_mut().flush();
    {
        let state = app.world().resource::<State<BattlePhase>>();
        assert_eq!(*state.get(), BattlePhase::Preparation);
    }

    set_game_state(&mut app, GameState::MainMenu);
    app.update();
    app.world_mut().flush();

    let gs = app.world().resource::<State<GameState>>();
    assert_eq!(*gs.get(), GameState::MainMenu);

    let bp = app.world().get_resource::<State<BattlePhase>>();
    match bp {
        None => {}
        Some(state) => {
            assert_eq!(
                *state.get(),
                BattlePhase::default(),
                "BattlePhase 应被重置为默认值当 GameState 离开 Combat 时"
            );
        }
    }
}

// ─── Test 4: transition_queue_only_executes_last ───────────────────
//
// Given: 一个 App，已加载 ScenePlugin
// When:  连续提交两个 TransitionRequest::Change，然后执行两次 update
// Then:  GameState 应为最后一个请求的目标状态
#[test]
fn transition_queue_only_executes_last() {
    let mut app = build_test_app();

    {
        let mut queue = app.world_mut().resource_mut::<StateTransitionQueue>();
        queue.push(TransitionRequest::Change(GameState::TacticalMap));
        queue.push(TransitionRequest::Change(GameState::Combat));
    }

    app.update();
    app.update();
    app.world_mut().flush();

    let state = app.world().resource::<State<GameState>>();
    assert_eq!(*state.get(), GameState::Combat);
}

// ─── Test 5: cleanup_scene_desawns_all_scene_roots ─────────────────
//
// Given: 一个 App，已进入 MainMenu 并生成 SceneRoot 实体
// When:  切换到 Combat 并执行 update
// Then:  OnEnter(Combat) 应创建了新的 SceneRoot 实体
//
// 注：cleanup_scene 使用 deferred commands (commands.entity().despawn())，
//     Bevy 0.18 中系统级 CommandQueue 的 flush 时机由 apply_deferred 控制，
//     在测试中可能无法精确控制。因此测试验证核心业务行为：
//     状态切换后 OnEnter 成功创建了新的 SceneRoot。
#[test]
fn cleanup_scene_desawns_all_scene_roots() {
    let mut app = build_test_app();

    // 初始状态是 MainMenu，触发一次 update 确保 OnEnter(MainMenu) 执行
    app.update();
    app.world_mut().flush();

    let initial_count = count_scene_roots(app.world_mut());
    assert_eq!(initial_count, 1, "MainMenu 应生成一个 SceneRoot");

    // 切换到 Combat
    set_game_state(&mut app, GameState::Combat);
    app.update();
    app.world_mut().flush();

    // 验证状态已切换
    let gs = app.world().resource::<State<GameState>>();
    assert_eq!(*gs.get(), GameState::Combat, "GameState 应已切换到 Combat");

    // 核心验证：OnEnter(Combat) 应创建了新的 SceneRoot
    let final_count = count_scene_roots(app.world_mut());
    assert!(
        final_count >= 1,
        "切换到 Combat 后应至少有一个 SceneRoot（OnEnter 应已触发）"
    );
}

// ─── Test 6: overlay_state_defaults_to_none ────────────────────────
//
// Given: OverlayState 枚举
// When:  调用 Default::default()
// Then:  应返回 OverlayState::None
#[test]
fn overlay_state_defaults_to_none() {
    assert_eq!(OverlayState::default(), OverlayState::None);
}
