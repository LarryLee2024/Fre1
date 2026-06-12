//! 统一移动系统 Feature Test
//!
//! 验证 MovementIntent 事件驱动的移动执行系统：
//! - MovementIntent 事件可正确构造和传递
//! - 玩家移动和 AI 移动使用相同的执行路径
//! - 非法移动被正确拒绝
//! - 移动速度配置化

// ================================================
// AI Self-Check (test_spec.md §13.1)
// ================================================
// ✅ 测试行为，不是实现
// ✅ 符合领域规则
// ✅ 测试是确定性的
// ✅ 使用标准测试数据
// ✅ 没有测试私有实现
// ✅ 没有生成不在范围内的测试
// ================================================

use bevy::prelude::*;
use tactical_rpg::character::{Faction, GridPosition, MovingUnit, Unit};
use tactical_rpg::core::attribute::{AttributeKind, Attributes};
use tactical_rpg::core::tag::GameplayTags;
use tactical_rpg::map::{
    GameMap, OccupancyGrid, TerrainCostRegistry, TerrainGrid, TerrainRegistry,
};
use tactical_rpg::turn::TurnPhase;
use tactical_rpg::ui::events::{IntentSource, MovementIntent};

use crate::common::fixtures::UnitBuilder;

// ── 测试辅助 ──

/// 设置移动执行测试 App
fn setup_movement_execution_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<TurnPhase>()
        .insert_resource(GameMap {
            width: 10,
            height: 10,
            tile_size: 64.0,
        })
        .insert_resource(TerrainGrid::default_plain(10, 10))
        .insert_resource(TerrainRegistry::default())
        .insert_resource(OccupancyGrid::default())
        .insert_resource(TerrainCostRegistry::default())
        .add_message::<MovementIntent>();

    // 注册移动执行系统
    app.add_systems(
        bevy::prelude::Update,
        tactical_rpg::character::movement_execution_system,
    );

    app
}

/// 在指定坐标生成单位（带 MoveRange=5）
fn spawn_unit_at(app: &mut App, x: i32, y: i32) -> Entity {
    let builder = UnitBuilder::unit_001();
    let mut attrs = builder.attrs().clone();
    attrs.set_base(AttributeKind::MoveRange, 5.0); // 设置移动范围为5格

    app.world_mut()
        .spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            attrs,
            GridPosition {
                coord: IVec2::new(x, y),
            },
            GameplayTags::default(),
        ))
        .id()
}

// ══════════════════════════════════════════════════════════════
// 场景一：MovementIntent 事件可正确构造
// ══════════════════════════════════════════════════════════════

/// FT-UMV-001: MovementIntent 事件可正确构造（玩家来源）
///
/// Given: 一个实体和目标坐标
/// When:  构造 MovementIntent { source: Player }
/// Then:  事件字段正确设置
#[test]
fn movement_intent_player_constructible() {
    // Given
    let entity = Entity::from_bits(42);
    let target = IVec2::new(5, 5);

    // When
    let intent = MovementIntent {
        entity,
        target_coord: target,
        source: IntentSource::Player,
    };

    // Then
    assert_eq!(intent.entity, entity);
    assert_eq!(intent.target_coord, target);
    assert!(matches!(intent.source, IntentSource::Player));
}

/// FT-UMV-002: MovementIntent 事件可正确构造（AI 来源）
///
/// Given: 一个实体和目标坐标
/// When:  构造 MovementIntent { source: Ai }
/// Then:  事件字段正确设置
#[test]
fn movement_intent_ai_constructible() {
    // Given
    let entity = Entity::from_bits(99);
    let target = IVec2::new(3, 7);

    // When
    let intent = MovementIntent {
        entity,
        target_coord: target,
        source: IntentSource::Ai,
    };

    // Then
    assert_eq!(intent.entity, entity);
    assert_eq!(intent.target_coord, target);
    assert!(matches!(intent.source, IntentSource::Ai));
}

// ══════════════════════════════════════════════════════════════
// 场景二：玩家移动执行 → MovingUnit 正确添加
// ══════════════════════════════════════════════════════════════

/// FT-UMV-003: 玩家移动执行 → MovingUnit 正确添加，next_phase = ActionMenu
///
/// Given: 一个玩家单位在 (0,0)，发送 MovementIntent 到 (2,0)
/// When:  运行移动执行系统
/// Then:  单位获得 MovingUnit 组件，next_phase = ActionMenu
#[test]
fn player_movement_execution_adds_moving_unit() {
    let mut app = setup_movement_execution_app();

    // Given: 玩家单位在 (0,0)
    let unit_entity = spawn_unit_at(&mut app, 0, 0);

    // When: 发送 MovementIntent 到 (2,0)
    app.world_mut().write_message(MovementIntent {
        entity: unit_entity,
        target_coord: IVec2::new(2, 0),
        source: IntentSource::Player,
    });
    app.update();

    // Then: 单位应获得 MovingUnit 组件
    let moving = app.world().get::<MovingUnit>(unit_entity);
    assert!(
        moving.is_some(),
        "Player unit should have MovingUnit component"
    );

    let moving = moving.unwrap();
    assert_eq!(
        moving.next_phase,
        TurnPhase::ActionMenu,
        "Player movement should transition to ActionMenu"
    );
    assert!(!moving.path.is_empty(), "Path should not be empty");
}

// ══════════════════════════════════════════════════════════════
// 场景三：AI 移动执行 → MovingUnit 正确添加
// ══════════════════════════════════════════════════════════════

/// FT-UMV-004: AI 移动执行 → MovingUnit 正确添加，next_phase = ExecuteAction
///
/// Given: 一个 AI 单位在 (0,0)，发送 MovementIntent 到 (1,1)
/// When:  运行移动执行系统
/// Then:  单位获得 MovingUnit 组件，next_phase = ExecuteAction
#[test]
fn ai_movement_execution_adds_moving_unit() {
    let mut app = setup_movement_execution_app();

    // Given: AI 单位在 (0,0)
    let mut attrs = Attributes::default();
    attrs.set_base(AttributeKind::Might, 15.0);
    attrs.set_base(AttributeKind::Vitality, 20.0);
    attrs.set_base(AttributeKind::Agility, 10.0);
    attrs.set_base(AttributeKind::Dexterity, 5.0);
    attrs.set_base(AttributeKind::Intelligence, 3.0);
    attrs.set_base(AttributeKind::Willpower, 5.0);
    attrs.set_base(AttributeKind::Presence, 3.0);
    attrs.set_base(AttributeKind::Luck, 3.0);
    attrs.set_base_attack_range(1);
    attrs.fill_vital_resources();

    let unit_entity = app
        .world_mut()
        .spawn((
            Unit {
                faction: Faction::Enemy,
                acted: false,
            },
            attrs,
            GridPosition {
                coord: IVec2::new(0, 0),
            },
            GameplayTags::default(),
        ))
        .id();

    // When: 发送 MovementIntent 到 (1,1)
    app.world_mut().write_message(MovementIntent {
        entity: unit_entity,
        target_coord: IVec2::new(1, 1),
        source: IntentSource::Ai,
    });
    app.update();

    // Then: 单位应获得 MovingUnit 组件
    let moving = app.world().get::<MovingUnit>(unit_entity);
    assert!(moving.is_some(), "AI unit should have MovingUnit component");

    let moving = moving.unwrap();
    assert_eq!(
        moving.next_phase,
        TurnPhase::ExecuteAction,
        "AI movement should transition to ExecuteAction"
    );
    assert!(!moving.path.is_empty(), "Path should not be empty");
}

// ══════════════════════════════════════════════════════════════
// 场景四：非法移动被拒绝
// ══════════════════════════════════════════════════════════════

/// FT-UMV-005: 目标超出移动范围 → 不添加 MovingUnit
///
/// Given: 一个单位在 (0,0)，MoveRange=3，尝试移动到 (10,10)
/// When:  发送 MovementIntent
/// Then:  单位没有 MovingUnit 组件（路径为空）
#[test]
fn out_of_range_movement_rejected() {
    let mut app = setup_movement_execution_app();

    // Given: 单位在 (0,0)，MoveRange 默认约 3-5
    let unit_entity = spawn_unit_at(&mut app, 0, 0);

    // When: 尝试移动到远处 (10,10)
    app.world_mut().write_message(MovementIntent {
        entity: unit_entity,
        target_coord: IVec2::new(10, 10),
        source: IntentSource::Player,
    });
    app.update();

    // Then: 单位不应获得 MovingUnit 组件（或路径为空）
    let moving = app.world().get::<MovingUnit>(unit_entity);
    // 如果添加了 MovingUnit，路径应该为空表示无法到达
    if let Some(m) = moving {
        assert!(
            m.path.is_empty(),
            "Out of range movement should have empty path"
        );
    }
    // 或者根本没有 MovingUnit
}

// ══════════════════════════════════════════════════════════════
// 场景五：原地不动不触发移动
// ══════════════════════════════════════════════════════════════

/// FT-UMV-006: 目标与起点相同 → 不添加 MovingUnit
///
/// Given: 一个单位在 (3,3)，发送 MovementIntent 到 (3,3)
/// When:  运行移动执行系统
/// Then:  单位没有 MovingUnit 组件
#[test]
fn same_position_movement_skipped() {
    let mut app = setup_movement_execution_app();

    // Given: 单位在 (3,3)
    let unit_entity = spawn_unit_at(&mut app, 3, 3);

    // When: 发送到相同位置
    app.world_mut().write_message(MovementIntent {
        entity: unit_entity,
        target_coord: IVec2::new(3, 3),
        source: IntentSource::Player,
    });
    app.update();

    // Then: 单位不应获得 MovingUnit 组件
    let moving = app.world().get::<MovingUnit>(unit_entity);
    assert!(
        moving.is_none(),
        "Same position movement should not add MovingUnit"
    );
}

// ══════════════════════════════════════════════════════════════
// 场景六：移动路径正确性
// ══════════════════════════════════════════════════════════════

/// FT-UMV-007: 水平移动路径正确
///
/// Given: 单位在 (0,0)，移动到 (3,0)
/// When:  执行移动
/// Then:  路径包含 [(1,0), (2,0), (3,0)]（不包含起点）
#[test]
fn horizontal_movement_path_correct() {
    let mut app = setup_movement_execution_app();

    // Given: 单位在 (0,0)
    let unit_entity = spawn_unit_at(&mut app, 0, 0);

    // When: 水平移动到 (3,0)
    app.world_mut().write_message(MovementIntent {
        entity: unit_entity,
        target_coord: IVec2::new(3, 0),
        source: IntentSource::Player,
    });
    app.update();

    // Then: 路径应正确
    let moving = app.world().get::<MovingUnit>(unit_entity).unwrap();
    assert!(!moving.path.is_empty(), "Path should not be empty");
    // reconstruct_path 返回的路径不包含起点
    assert_eq!(
        moving.path.first(),
        Some(&IVec2::new(1, 0)),
        "Path should start at (1,0)"
    );
    assert_eq!(
        moving.path.last(),
        Some(&IVec2::new(3, 0)),
        "Path should end at target"
    );
}

/// FT-UMV-008: 对角移动路径为直角折线（非斜线飞行）
///
/// Given: 单位在 (0,0)，移动到 (2,2)
/// When:  执行移动
/// Then:  路径应为直角折线，如 [(1,0), (2,0), (2,1), (2,2)]（不包含起点）
#[test]
fn diagonal_movement_path_is_orthogonal() {
    let mut app = setup_movement_execution_app();

    // Given: 单位在 (0,0)
    let unit_entity = spawn_unit_at(&mut app, 0, 0);

    // When: 对角移动到 (2,2)
    app.world_mut().write_message(MovementIntent {
        entity: unit_entity,
        target_coord: IVec2::new(2, 2),
        source: IntentSource::Player,
    });
    app.update();

    // Then: 路径应为直角折线
    let moving = app.world().get::<MovingUnit>(unit_entity).unwrap();
    assert!(!moving.path.is_empty(), "Path should not be empty");

    // 验证路径中每一步都是正交移动（非斜角）
    for i in 0..moving.path.len().saturating_sub(1) {
        let from = moving.path[i];
        let to = moving.path[i + 1];
        let diff = to - from;
        // 正交移动：只能在一个轴上变化
        assert!(
            (diff.x == 0 || diff.y == 0) && (diff.x.abs() <= 1 && diff.y.abs() <= 1),
            "Step from {:?} to {:?} is not orthogonal",
            from,
            to
        );
    }
}

// ══════════════════════════════════════════════════════════════
// 场景七：多单位移动独立性
// ══════════════════════════════════════════════════════════════

/// FT-UMV-009: 多个单位同时移动互不干扰
///
/// Given: 两个单位分别在 (0,0) 和 (5,5)
/// When:  同时发送两个 MovementIntent
/// Then:  两个单位都获得独立的 MovingUnit 组件
#[test]
fn multiple_units_move_independently() {
    let mut app = setup_movement_execution_app();

    // Given: 两个单位
    let unit1 = spawn_unit_at(&mut app, 0, 0);
    let unit2 = spawn_unit_at(&mut app, 5, 5);

    // When: 同时发送移动意图
    app.world_mut().write_message(MovementIntent {
        entity: unit1,
        target_coord: IVec2::new(2, 0),
        source: IntentSource::Player,
    });
    app.world_mut().write_message(MovementIntent {
        entity: unit2,
        target_coord: IVec2::new(5, 7),
        source: IntentSource::Ai,
    });
    app.update();

    // Then: 两个单位都应获得 MovingUnit
    let moving1 = app.world().get::<MovingUnit>(unit1);
    let moving2 = app.world().get::<MovingUnit>(unit2);

    assert!(moving1.is_some(), "Unit 1 should have MovingUnit");
    assert!(moving2.is_some(), "Unit 2 should have MovingUnit");

    // 验证路径不同
    assert_ne!(
        moving1.unwrap().path,
        moving2.unwrap().path,
        "Paths should be different"
    );
}

// ══════════════════════════════════════════════════════════════
// 场景八：移动速度固定
// ══════════════════════════════════════════════════════════════

/// FT-UMV-010: 移动速度固定为 0.15 秒/格
///
/// Given: 任意单位执行移动
/// When:  检查 MovingUnit.speed
/// Then:  speed = 0.15
#[test]
fn movement_speed_is_fixed() {
    let mut app = setup_movement_execution_app();

    // Given: 单位在 (0,0)
    let unit_entity = spawn_unit_at(&mut app, 0, 0);

    // When: 执行移动
    app.world_mut().write_message(MovementIntent {
        entity: unit_entity,
        target_coord: IVec2::new(1, 0),
        source: IntentSource::Player,
    });
    app.update();

    // Then: speed 应为 0.15
    let moving = app.world().get::<MovingUnit>(unit_entity).unwrap();
    assert_eq!(moving.speed, 0.15, "Movement speed should be fixed at 0.15");
}
