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
//! | 3 | `battle_phase_invalid_outside_combat` | 切出 GameState::Combat 后 BattlePhase 不可访问 |
//! | 4 | `transition_queue_only_executes_last` | StateTransitionQueue 批量请求只执行最后一个 |
//! | 5 | `cleanup_scene_desawns_all_scene_roots` | cleanup_scene despawn 所有 SceneRoot 实体 |
//! | 6 | `overlay_state_defaults_to_none` | OverlayState 默认值为 None |

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use crate::app::scenes::{GameState, OverlayState, SceneRoot, ScenePlugin, StateTransitionQueue, TransitionRequest};
use crate::core::domains::combat::components::BattlePhase;

// ─── 辅助函数 ──────────────────────────────────────────────────────

/// 构建最小测试 App：MinimalPlugins + StatesPlugin + ScenePlugin
fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin, ScenePlugin));
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
// When:  将 GameState 切换到 Combat 并执行一次 update
// Then:  BattlePhase 应可访问且默认为 Preparation
#[test]
fn battle_phase_accessible_in_combat() {
    let mut app = build_combat_test_app();
    
    // 切换到 Combat（直接设置 NextState，不通过队列）
    set_game_state(&mut app, GameState::Combat);
    // update 触发 StateTransition → OnEnter(GameState::Combat)
    app.update();
    app.world_mut().flush();
    
    let state = app.world().resource::<State<BattlePhase>>();
    assert_eq!(*state.get(), BattlePhase::Preparation);
}

// ─── Test 3: battle_phase_invalid_outside_combat ───────────────────
//
// Given: 一个 App，已进入 GameState::Combat 状态
// When:  将 GameState 切换回 MainMenu 并执行一次 update
// Then:  BattlePhase 资源应不存在（SubState 被清理）
#[test]
fn battle_phase_invalid_outside_combat() {
    let mut app = build_combat_test_app();
    
    // 进入 Combat
    set_game_state(&mut app, GameState::Combat);
    app.update();
    app.world_mut().flush();
    // 确认 BattlePhase 存在
    assert!(app.world().get_resource::<State<BattlePhase>>().is_some());
    
    // 切出 Combat → MainMenu
    set_game_state(&mut app, GameState::MainMenu);
    app.update();
    app.world_mut().flush();
    
    // BattlePhase 应不存在（SubState 随父状态退出而清理）
    assert!(
        app.world().get_resource::<State<BattlePhase>>().is_none(),
        "BattlePhase SubState 应在 GameState 离开 Combat 时被清理"
    );
}

// ─── Test 4: transition_queue_only_executes_last ───────────────────
//
// Given: 一个 App，已加载 ScenePlugin
// When:  连续提交两个 TransitionRequest::Change，然后执行两次 update
// Then:  GameState 应为最后一个请求的目标状态
//
// 注：process_transition_queue 在 Last 调度中执行，设置 NextState。
//     实际的状态转移在下一帧的 StateTransition 中完成，因此需要两次 update。
#[test]
fn transition_queue_only_executes_last() {
    let mut app = build_test_app();
    
    // 提交多个转移请求
    {
        let mut queue = app.world_mut().resource_mut::<StateTransitionQueue>();
        queue.push(TransitionRequest::Change(GameState::TacticalMap));
        queue.push(TransitionRequest::Change(GameState::Combat));
    }
    
    // 第一次 update：process_transition_queue 在 Last 中执行，
    // 取出最后一个请求并调用 next_state.set(Combat)
    app.update();
    // 第二次 update：StateTransition 处理 NextState，实际切换到 Combat
    app.update();
    app.world_mut().flush();
    
    let state = app.world().resource::<State<GameState>>();
    // 只执行最后一个请求，应为 Combat
    assert_eq!(*state.get(), GameState::Combat);
}

// ─── Test 5: cleanup_scene_desawns_all_scene_roots ─────────────────
//
// Given: 一个 App，已进入 MainMenu 并生成 SceneRoot 实体
// When:  切换到 Combat 并执行 update（触发 OnExit(MainMenu) → cleanup_scene）
// Then:  旧的 SceneRoot 实体应被移除，新的 SceneRoot 实体应被创建
#[test]
fn cleanup_scene_desawns_all_scene_roots() {
    let mut app = build_test_app();
    
    // 初始状态是 MainMenu，触发一次 update 确保 OnEnter 执行
    app.update();
    app.world_mut().flush();
    
    // 记录 MainMenu 的 SceneRoot
    let initial_count = count_scene_roots(app.world_mut());
    assert_eq!(initial_count, 1, "MainMenu 应生成一个 SceneRoot");
    
    // 切换到 Combat
    set_game_state(&mut app, GameState::Combat);
    // update 触发 OnExit(MainMenu) → cleanup_scene，然后 OnEnter(Combat) → setup_scene_root
    app.update();
    app.world_mut().flush();
    
    // 检查 SceneRoot 数量（应为 1，旧的已清理，新的已生成）
    let final_count = count_scene_roots(app.world_mut());
    assert_eq!(final_count, 1, "场景切换后应只有一个 SceneRoot");
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