//! Spawn System — 根据 RON 配置文件生成战斗实体
//!
//! 从 `TestBattleScenario` Resource 读取配置数据，生成：
//! - 带战斗组件的单位实体（HitPoints, ActionPoints, CombatParticipant, GridPos等）
//! - TurnQueue Resource 初始化
//!
//! 视觉组件由 `render.rs` 独立添加 —— 逻辑与表现分离原则。

use bevy::prelude::*;

use crate::app::scenes::components::SceneRoot;
use crate::core::domains::combat::components::{
    ActionPoints, BattlePhase, CombatParticipant, HitPoints, TeamId, TurnEntry, TurnQueue,
    UnitIdComponent,
};
use crate::core::domains::tactical::components::GridPos;

use super::def::TestBattleDef;

/// Resource 持有已加载的测试战斗场景配置
#[derive(Resource)]
pub struct TestBattleScenario {
    pub def: TestBattleDef,
}

/// 系统：加载 RON 配置文件到 Resource（Startup）。
///
/// 从 `assets/configs/scenarios/test_battle.ron` 加载并解析配置，
/// 存入 `TestBattleScenario` Resource 供 `spawn_test_battle` 使用。
///
/// 使用 CARGO_MANIFEST_DIR 确保路径不依赖运行时工作目录。
pub fn load_test_battle_scenario(mut commands: Commands) {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/configs/scenarios/test_battle.ron"
    );
    let data = match std::fs::read_to_string(path) {
        Ok(d) => d,
        Err(e) => {
            tracing::error!(target: "app", "Failed to read test battle config: {}", e);
            return;
        }
    };

    let def: TestBattleDef = match ron::from_str(&data) {
        Ok(d) => d,
        Err(e) => {
            tracing::error!(target: "app", "Failed to parse test battle config: {}", e);
            return;
        }
    };

    commands.insert_resource(TestBattleScenario { def });
    tracing::info!(target: "app", "TestBattle scenario loaded");
}

/// 系统：根据配置生成所有战斗实体（OnEnter GameState::Combat）。
///
/// 为每个单位生成：
/// - UnitIdComponent（字符串标识）
/// - HitPoints（当前/最大生命值）
/// - CombatParticipant（队伍归属）
/// - ActionPoints（行动资源）
/// - GridPos（网格坐标）
/// - Name（调试用）
///
/// 初始化 TurnQueue Resource 并设置 BattlePhase → Battle。
pub fn spawn_test_battle(
    mut commands: Commands,
    scenario: Option<Res<TestBattleScenario>>,
    _scene_root: Query<Entity, With<SceneRoot>>,
    battle_phase: Option<ResMut<NextState<BattlePhase>>>,
) {
    let Some(scenario) = scenario else {
        tracing::warn!(target: "app", "No TestBattleScenario loaded — skipping spawn");
        return;
    };

    let def = &scenario.def;
    let mut turn_entries: Vec<TurnEntry> = Vec::new();

    // 每个单位生成一个 ECS 实体
    for unit in &def.units {
        // 构建 TeamId（从 RON 字符串）
        let team_id = TeamId::new(&unit.team);

        // 计算 HP（确保不超上限）
        let current_hp = unit.hp.min(unit.max_hp);

        let entity = commands
            .spawn((
                UnitIdComponent::new(&unit.id),
                HitPoints {
                    current: current_hp,
                    maximum: unit.max_hp,
                },
                CombatParticipant::alive(team_id.clone()),
                ActionPoints::new(5.0), // 默认 5 格移动力
                GridPos::new(unit.coord.0, unit.coord.1),
                Name::new(format!("Unit_{}", unit.id)),
            ))
            .id();

        // 先攻值统一设为 100，TurnQueue 按插入顺序排列
        turn_entries.push(TurnEntry::new(entity, team_id, 100));

        tracing::debug!(target: "app",
            "Spawned unit {} at ({}, {}) team={:?} hp={}/{}",
            unit.id, unit.coord.0, unit.coord.1, unit.team, current_hp, unit.max_hp,
        );
    }

    // 初始化回合队列
    let turn_queue = TurnQueue::new(turn_entries);
    commands.insert_resource(turn_queue);

    // 切换到战斗阶段
    if let Some(mut bp) = battle_phase {
        bp.set(BattlePhase::Battle);
    }

    tracing::info!(target: "app",
        "TestBattle spawned: {} units on {}x{} grid",
        def.units.len(),
        def.grid.width,
        def.grid.height,
    );
}
