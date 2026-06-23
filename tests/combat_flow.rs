//! 战斗循环集成测试。
//!
//! 验证可玩战斗循环的核心链路：
//!
//! | Test ID          | Test Name                        | Assertion Target                              |
//! |------------------|----------------------------------|-----------------------------------------------|
//! | CMB-FLOW-001     | player_attack_damages_enemy      | AttackRequested → HP 减少                     |
//! | CMB-FLOW-002     | victory_state_transition         | BattleEnded → GameState::Result → ResultScreen |
//! | CMB-FLOW-003     | defeat_state_transition          | BattleEnded(false) → GameState::GameOver       |

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use fre::app::scenes::test_battle::TestBattlePlugin;
use fre::app::scenes::{GameState, ScenePlugin, StateTransitionQueue, TransitionRequest};
use fre::core::domains::combat::{
    AttackRequested, BattlePhase, CombatParticipant, HitPoints, TurnQueue, UnitIdComponent,
};
use fre::core::events::BattleEnded;
use fre::infra::pipeline::PipelinePlugin;
use fre::shared::SharedPlugin;

// ---------------------------------------------------------------------------
// Headless 测试 App 构建器
// ---------------------------------------------------------------------------
// 遵循 AppPlugin 的 Phase 顺序，但使用 MinimalPlugins 替代 DefaultPlugins，
// 避免渲染/窗口开销。

fn build_headless_app() -> App {
    let mut app = App::new();

    // Phase 0: Core Bevy + Shared
    app.add_plugins((
        MinimalPlugins,
        StatesPlugin,
        bevy::input::InputPlugin, // ButtonInput<KeyCode> — infra::InputPlugin 依赖
        bevy::asset::AssetPlugin::default(), // AssetServer — UI/Theme 依赖
        bevy::image::ImagePlugin::default(), // Images — 渲染背景需要
        bevy::text::TextPlugin,   // Font asset — UI/Text 需要
    ))
    .add_plugins(SharedPlugin);

    // Phase 1-7: Core (capabilities + domains, including CombatPlugin)
    app.add_plugins(fre::core::CorePlugin);

    // Phase 8: Infrastructure
    app.add_plugins(PipelinePlugin);
    app.add_plugins(fre::infra::input::InputPlugin); // InputState — CombatPlugin 依赖
    app.add_plugins(fre::infra::localization::LocalizationPlugin::new());

    // Phase 9: Scene Management
    app.add_plugins(ScenePlugin);
    app.add_plugins(TestBattlePlugin);

    // Phase 10: Content
    app.add_plugins(fre::content::ContentPlugin);

    // Phase 11: UI (Theme, Primitives, Screens — 用于 Result/GameOver 场景)
    app.add_plugins(fre::ui::UiPlugin);

    // Image asset 支持
    app.init_resource::<Assets<Image>>();

    app
}

/// 推进 N 帧，每帧后 flush
fn advance_frames(app: &mut App, n: usize) {
    for _ in 0..n {
        app.update();
        app.world_mut().flush();
    }
}

/// 过渡到 Combat 状态
fn transition_to_combat(app: &mut App) {
    app.update();
    app.world_mut().flush();

    app.world_mut()
        .resource_mut::<StateTransitionQueue>()
        .push(TransitionRequest::Change(GameState::Combat));

    advance_frames(app, 6);
    app.world_mut().flush();
}

fn current_game_state(app: &App) -> GameState {
    app.world()
        .get_resource::<State<GameState>>()
        .map(|s| s.get().clone())
        .unwrap_or(GameState::MainMenu)
}

fn current_battle_phase(app: &App) -> Option<BattlePhase> {
    app.world()
        .get_resource::<State<BattlePhase>>()
        .map(|s| s.get().clone())
}

// ---------------------------------------------------------------------------
// CMB-FLOW-001: player_attack_damages_enemy
//
// 验证攻击命令能通过 AttackRequested → on_attack_requested → DamageDealt
// → on_damage_dealt → HP 减少 的完整链路。
// ---------------------------------------------------------------------------
#[test]
fn player_attack_damages_enemy() {
    let mut app = build_headless_app();
    transition_to_combat(&mut app);

    // 找到所有战斗单位
    let mut unit_query = app
        .world_mut()
        .query::<(&UnitIdComponent, &HitPoints, &CombatParticipant)>();
    let units: Vec<_> = unit_query.iter(&app.world()).collect();

    let goblin = units
        .iter()
        .find(|(uid, _, _)| uid.id == "unit_goblin")
        .expect("goblin 单位应存在于 test_battle 场景中");
    let goblin_hp_before = goblin.1.current;
    assert_eq!(goblin_hp_before, 40, "goblin 初始 HP 应为 40");

    // 触发攻击请求（DamagePolicy.base=10, def=0 → 最终伤害=10）
    app.world_mut().trigger(AttackRequested {
        attacker_id: "unit_hero".to_string(),
        target_id: "unit_goblin".to_string(),
        ability_slot: None,
    });
    app.world_mut().flush();

    // 验证 HP 减少
    let mut hp_query = app.world_mut().query::<(&UnitIdComponent, &HitPoints)>();
    let goblin_hp_after = hp_query
        .iter(&app.world())
        .find(|(uid, _)| uid.id == "unit_goblin")
        .map(|(_, hp)| hp.current)
        .expect("goblin 应仍存在");

    assert!(
        goblin_hp_after < goblin_hp_before,
        "goblin HP 应减少（原始={}, 现在={}）",
        goblin_hp_before,
        goblin_hp_after
    );
}

// ---------------------------------------------------------------------------
// CMB-FLOW-002: victory_state_transition
//
// 验证 BattleEnded { victory: true }
// → on_battle_ended observer
// → GameState::Result
// → ResultScreen 生成
// ---------------------------------------------------------------------------
#[test]
fn victory_state_transition() {
    let mut app = build_headless_app();
    transition_to_combat(&mut app);

    assert_eq!(
        current_game_state(&app),
        GameState::Combat,
        "初始 GameState 应为 Combat"
    );

    // 触发 BattleEnded 事件（模拟战斗胜利）
    app.world_mut().trigger(BattleEnded { victory: true });
    app.world_mut().flush();

    // 推进帧：observer → queue → process_transition_queue → OnEnter(Result)
    advance_frames(&mut app, 8);
    app.world_mut().flush();

    let state = current_game_state(&app);
    assert_eq!(
        state,
        GameState::Result,
        "BattleEnded(victory=true) 后应转换到 GameState::Result，实际={:?}",
        state,
    );

    // 验证 ResultScreen 实体已生成
    let mut query = app
        .world_mut()
        .query::<&fre::app::scenes::result::ResultScreen>();
    let count = query.iter(&app.world()).count();
    assert_eq!(count, 1, "Result 场景应包含 1 个 ResultScreen 实体");
}

// ---------------------------------------------------------------------------
// CMB-FLOW-003: defeat_state_transition
//
// 验证 BattleEnded { victory: false }
// → on_battle_ended observer
// → GameState::GameOver
// → GameOverScreen 生成
// ---------------------------------------------------------------------------
#[test]
fn defeat_state_transition() {
    let mut app = build_headless_app();
    transition_to_combat(&mut app);

    assert_eq!(
        current_game_state(&app),
        GameState::Combat,
        "初始 GameState 应为 Combat"
    );

    // 触发战败事件
    app.world_mut().trigger(BattleEnded { victory: false });
    app.world_mut().flush();

    advance_frames(&mut app, 8);
    app.world_mut().flush();

    let state = current_game_state(&app);
    assert_eq!(
        state,
        GameState::GameOver,
        "BattleEnded(victory=false) 后应转换到 GameState::GameOver，实际={:?}",
        state,
    );

    // 验证 GameOverScreen 实体已生成
    let mut query = app
        .world_mut()
        .query::<&fre::app::scenes::game_over::GameOverScreen>();
    let count = query.iter(&app.world()).count();
    assert_eq!(count, 1, "GameOver 场景应包含 1 个 GameOverScreen 实体");
}
