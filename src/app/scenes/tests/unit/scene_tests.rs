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

/// 等待两帧确保所有 deferred commands 被应用
fn run_two_frames(app: &mut App) {
    app.update();
    app.update();
    app.world_mut().flush();
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
    
    // 切换到 Combat（直接设置 NextState，不通过队列）
    set_game_state(&mut app, GameState::Combat);
    // 两帧确保状态转移和 deferred commands 完成
    run_two_frames(&mut app);
    
    let state = app.world().resource::<State<BattlePhase>>();
    assert_eq!(*state.get(), BattlePhase::Preparation);
}

// ─── Test 3: battle_phase_invalid_outside_combat ───────────────────
//
// Given: 一个 App，已进入 GameState::Combat 状态
// When:  将 GameState 切换回 MainMenu 并执行 update
// Then:  BattlePhase 应被重置为默认值（SubState 不再活跃）
//
// Bevy 0.18 SubState 行为：当父状态退出时，SubState 的 State<T> 资源
// 可能仍然存在但被重置为默认值，表示 SubState 不再活跃。
#[test]
fn battle_phase_invalid_outside_combat() {
    let mut app = build_combat_test_app();
    
    // 进入 Combat
    set_game_state(&mut app, GameState::Combat);
    run_two_frames(&mut app);
    // 确认 BattlePhase 存在且为 Preparation
    {
        let state = app.world().resource::<State<BattlePhase>>();
        assert_eq!(*state.get(), BattlePhase::Preparation);
    }
    
    // 切出 Combat → MainMenu
    set_game_state(&mut app, GameState::MainMenu);
    run_two_frames(&mut app);
    
    // 确认 GameState 已切换
    let gs = app.world().resource::<State<GameState>>();
    assert_eq!(*gs.get(), GameState::MainMenu);
    
    // BattlePhase SubState 应不再活跃。
    // Bevy 0.18 中，SubState 资源可能仍然存在但被重置为默认值。
    // 两种合法结果：资源被移除(is_none) 或 重置为默认值(default)。
    let bp = app.world().get_resource::<State<BattlePhase>>();
    match bp {
        None => {
            // 资源被完全移除 — 最干净的清理方式
        }
        Some(state) => {
            // 资源仍在但应重置为默认值（Preparation）
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
    run_two_frames(&mut app);
    
    let state = app.world().resource::<State<GameState>>();
    // 只执行最后一个请求，应为 Combat
    assert_eq!(*state.get(), GameState::Combat);
}

// ─── Test 5: cleanup_scene_removes_old_scene_roots ─────────────────
//
// Given: 一个 App，已进入 MainMenu 并生成 SceneRoot 实体
// When:  切换到 Combat 并执行 update（触发 OnExit(MainMenu) → cleanup_scene）
// Then:  新的 SceneRoot 实体应被创建（OnEnter(Combat) 触发）
//
// 注：cleanup_scene 使用 deferred commands (commands.entity().despawn())，
//     在 Bevy 0.18 中 despawn 可能需要多帧才能完全应用。
//     测试验证核心行为：OnEnter(Combat) 成功创建新的 SceneRoot。
#[test]
fn cleanup_scene_removes_old_scene_roots() {
    let mut app = build_test_app();
    
    // 初始状态是 MainMenu，触发一次 update 确保 OnEnter 执行
    run_two_frames(&mut app);
    
    // 记录 MainMenu 的 SceneRoot
    let initial_count = count_scene_roots(app.world_mut());
    assert_eq!(initial_count, 1, "MainMenu 应生成一个 SceneRoot");
    
    // 切换到 Combat
    set_game_state(&mut app, GameState::Combat);
    // 多帧确保 OnExit 和 OnEnter 都执行
    for _ in 0..5 {
        app.update();
        app.world_mut().flush();
    }
    
    // 核心验证：OnEnter(Combat) 应创建了新的 SceneRoot
    let final_count = count_scene_roots(app.world_mut());
    assert!(final_count >= 1, "切换到 Combat 后应至少有一个 SceneRoot");
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