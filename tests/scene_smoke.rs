//! 跨领域场景冒烟集成测试。
//!
//! 验证 `PartySetup -> Combat` 状态转换后，ECS World 包含领域规则所规定的
//! correct resources and entities。
//!
//! 这是一个根 `tests/` 级别的测试，横跨 `app/scenes` 和多个
//! `core/domains`（combat, tactical）。
//!
//! # Test Matrix
//!
//! | Test ID       | Test Name                              | Assertion Target                               |
//! |---------------|----------------------------------------|------------------------------------------------|
//! | TS-SMOKE-001  | `combat_state_creates_grid_map`        | GridMap resource: width=6, height=6, Square    |
//! | TS-SMOKE-002  | `combat_state_creates_turn_queue`      | TurnQueue resource: 4 entries, round=1         |
//! | TS-SMOKE-003  | `combat_state_spawns_units`            | 4 units with correct HP / GridPos / components |
//! | TS-SMOKE-004  | `combat_state_creates_grid_background` | 36 grid background entities (6x6)              |

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;

use fre::app::scenes::test_battle::TestBattlePlugin;
use fre::app::scenes::{GameState, ScenePlugin, StateTransitionQueue, TransitionRequest};
use fre::core::domains::combat::{CombatParticipant, HitPoints, TurnQueue, UnitIdComponent};
use fre::core::domains::tactical::{GridLayout, GridMap, GridPos};

// ---------------------------------------------------------------------------
// 测试辅助函数
// ---------------------------------------------------------------------------

/// 构建包含所有必需插件的最小测试 App。
fn build_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin, ScenePlugin, TestBattlePlugin))
        .init_resource::<Assets<Image>>();
    app
}

// ---------------------------------------------------------------------------
// TS-SMOKE-001: combat_state_creates_grid_map
//
// Title: Combat 状态创建 GridMap Resource
// Given: 带有 TestBattlePlugin 的最小 App；Startup 已加载场景
// When:  通过 StateTransitionQueue 从 MainMenu 转换到 Combat
// Then:  GridMap Resource 存在，width=6, height=6, layout=Square
// ---------------------------------------------------------------------------
#[test]
fn combat_state_creates_grid_map() {
    let mut app = build_test_app();

    // 触发 Startup 系统：load_test_battle_scenario
    app.update();
    app.world_mut().flush();

    // 入队 Combat 转换请求
    app.world_mut()
        .resource_mut::<StateTransitionQueue>()
        .push(TransitionRequest::Change(GameState::Combat));

    // Update 1: process_transition_queue 在 Last schedule 运行 -> 设置 NextState
    app.update();
    // Update 2: StateTransition 执行 OnExit(MainMenu) + OnEnter(Combat)
    //           -> spawn_test_battle 插入 GridMap Resource
    app.update();
    app.world_mut().flush();

    // ---- 验证 ----
    let grid_map = app.world().resource::<GridMap>();
    assert_eq!(
        grid_map.width, 6,
        "GridMap width should be 6 (from test_battle.ron)"
    );
    assert_eq!(
        grid_map.height, 6,
        "GridMap height should be 6 (from test_battle.ron)"
    );
    assert_eq!(
        grid_map.layout,
        GridLayout::Square,
        "GridMap layout should be Square"
    );
    assert_eq!(grid_map.tiles.len(), 36, "GridMap should contain 36 tiles");
}

// ---------------------------------------------------------------------------
// TS-SMOKE-002: combat_state_creates_turn_queue
//
// Title: Combat 状态创建 TurnQueue Resource
// Given: 带有 TestBattlePlugin 的最小 App
// When:  从 MainMenu 转换到 Combat 状态
// Then:  TurnQueue Resource 存在，4 个条目，round=1, index=0
// ---------------------------------------------------------------------------
#[test]
fn combat_state_creates_turn_queue() {
    let mut app = build_test_app();

    // Startup: 加载 test_battle 场景
    app.update();
    app.world_mut().flush();

    // 转换到 Combat 状态
    app.world_mut()
        .resource_mut::<StateTransitionQueue>()
        .push(TransitionRequest::Change(GameState::Combat));
    app.update();
    app.update();
    app.world_mut().flush();

    // ---- 验证 ----
    let turn_queue = app.world().resource::<TurnQueue>();
    assert_eq!(
        turn_queue.len(),
        4,
        "TurnQueue should have 4 entries (one per unit in test_battle.ron)"
    );
    assert_eq!(
        turn_queue.round_number(),
        1,
        "TurnQueue should start at round 1"
    );
    assert_eq!(
        turn_queue.current_index(),
        0,
        "TurnQueue should start at index 0"
    );
}

// ---------------------------------------------------------------------------
// TS-SMOKE-003: combat_state_spawns_units
//
// Title: Combat 状态生成 4 个单位，带有正确的 Component
// Given: 带有 TestBattlePlugin 的最小 App
// When:  从 MainMenu 转换到 Combat 状态
// Then:  4 个单位实体存在，每个都有 HitPoints / GridPos / CombatParticipant
//        / UnitIdComponent，且与 RON 配置数据一致
// ---------------------------------------------------------------------------
#[test]
fn combat_state_spawns_units() {
    let mut app = build_test_app();

    // Startup: 加载 test_battle 场景
    app.update();
    app.world_mut().flush();

    // 转换到 Combat 状态
    app.world_mut()
        .resource_mut::<StateTransitionQueue>()
        .push(TransitionRequest::Change(GameState::Combat));
    app.update();
    app.update();
    app.world_mut().flush();

    // ---- 验证 ----
    let mut query = app
        .world_mut()
        .query::<(&UnitIdComponent, &HitPoints, &CombatParticipant, &GridPos)>();
    let units: Vec<_> = query.iter(&app.world()).collect();

    assert_eq!(
        units.len(),
        4,
        "Should spawn exactly 4 units (from test_battle.ron)"
    );

    // -- unit_hero: (1,1) hp=100/100 所属队伍=Player --
    let hero = units.iter().find(|(uid, _, _, _)| uid.id == "unit_hero");
    assert!(hero.is_some(), "unit_hero should exist");
    if let Some((_uid, hp, _participant, pos)) = hero {
        assert_eq!(hp.current, 100, "unit_hero current HP should be 100");
        assert_eq!(hp.maximum, 100, "unit_hero max HP should be 100");
        assert_eq!(**pos, GridPos::new(1, 1), "unit_hero should be at (1,1)");
    }

    // -- unit_ally: (1,2) hp=80/80 所属队伍=Player --
    let ally = units.iter().find(|(uid, _, _, _)| uid.id == "unit_ally");
    assert!(ally.is_some(), "unit_ally should exist");
    if let Some((_uid, hp, _participant, pos)) = ally {
        assert_eq!(hp.current, 80, "unit_ally current HP should be 80");
        assert_eq!(hp.maximum, 80, "unit_ally max HP should be 80");
        assert_eq!(**pos, GridPos::new(1, 2), "unit_ally should be at (1,2)");
    }

    // -- unit_goblin: (5,4) hp=40/40 所属队伍=Enemy --
    let goblin = units.iter().find(|(uid, _, _, _)| uid.id == "unit_goblin");
    assert!(goblin.is_some(), "unit_goblin should exist");
    if let Some((_uid, hp, _participant, pos)) = goblin {
        assert_eq!(hp.current, 40, "unit_goblin current HP should be 40");
        assert_eq!(hp.maximum, 40, "unit_goblin max HP should be 40");
        assert_eq!(**pos, GridPos::new(5, 4), "unit_goblin should be at (5,4)");
    }

    // -- unit_orc: (5,5) hp=60/60 所属队伍=Enemy --
    let orc = units.iter().find(|(uid, _, _, _)| uid.id == "unit_orc");
    assert!(orc.is_some(), "unit_orc should exist");
    if let Some((_uid, hp, _participant, pos)) = orc {
        assert_eq!(hp.current, 60, "unit_orc current HP should be 60");
        assert_eq!(hp.maximum, 60, "unit_orc max HP should be 60");
        assert_eq!(**pos, GridPos::new(5, 5), "unit_orc should be at (5,5)");
    }
}

// ---------------------------------------------------------------------------
// TS-SMOKE-004: combat_state_creates_grid_background
//
// Title: Combat 状态创建网格背景实体
// Given: 带有 TestBattlePlugin 的最小 App
// When:  从 MainMenu 转换到 Combat 状态
// Then:  36 个网格背景实体存在（grid.width x grid.height = 6x6）
//        通过拥有 Sprite + Visibility 但没有 CombatParticipant 来识别
// ---------------------------------------------------------------------------
#[test]
fn combat_state_creates_grid_background() {
    let mut app = build_test_app();

    // Startup: 加载 test_battle 场景
    app.update();
    app.world_mut().flush();

    // 转换到 Combat 状态
    app.world_mut()
        .resource_mut::<StateTransitionQueue>()
        .push(TransitionRequest::Change(GameState::Combat));
    app.update();
    app.update();
    app.world_mut().flush();

    // ---- 验证 ----
    // 网格背景实体有 Sprite + Visibility，但没有 CombatParticipant
    //（单位视觉实体有 Sprite + Visibility + CombatParticipant）
    let mut query = app
        .world_mut()
        .query_filtered::<Entity, (With<Sprite>, With<Visibility>, Without<CombatParticipant>)>();
    let grid_entities: Vec<_> = query.iter(&app.world()).collect();

    assert_eq!(
        grid_entities.len(),
        36,
        "Should spawn exactly 36 grid background entities (6x6 grid)"
    );
}
