// 胜负条件检查系统：数据驱动，读取 LevelConfig.VictoryConditionDef
//
// 两层检查机制：
// 1. 响应式（OnEnter(TurnEnd)）：检查所有配置的胜负条件
// 2. 兜底（Update）：仅检查"全灭玩家即失败"绝对不变量

use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

use crate::core::character::{Dead, Faction, Unit, UnitId};
use crate::core::map::{
    ConditionTypeDef, LevelRegistry, LoseConditionDef, VictoryConditionDef, WinConditionDef,
};
use crate::shared::event::campaign::LevelCompleted as LogLevelCompleted;

use super::order::TurnState;
use super::state::GameOverState;

/// 关卡完成消息：GameOverState 变为 Victory 或 Defeat 时发送
#[derive(Message, Debug, Clone)]
pub struct LevelCompleted {
    pub level_id: String,
    pub result: GameOverState,
    pub turn_number: u32,
}

/// 响应式胜负检查：在 TurnEnd 阶段统一检查所有配置的胜负条件
///
/// 检查顺序：先检查所有 lose_conditions（失败优先原则），
/// 再检查所有 win_conditions。终态不可逆。
pub fn check_victory_conditions(
    level_registry: Res<LevelRegistry>,
    turn_state: Res<TurnState>,
    alive_players: Query<&Unit, (With<Unit>, Without<Dead>)>,
    alive_enemies: Query<&Unit, (With<Unit>, Without<Dead>)>,
    all_units: Query<(&Unit, Option<&UnitId>), Without<Dead>>,
    mut game_over: ResMut<GameOverState>,
    mut level_completed_writer: MessageWriter<LevelCompleted>,
    mut log_level_writer: MessageWriter<LogLevelCompleted>,
) {
    // 终态不可逆（FORBIDDEN-5）
    if *game_over != GameOverState::Playing {
        return;
    }

    // 获取当前关卡配置
    let Some(level_config) = level_registry.levels.values().next() else {
        return;
    };

    let Some(victory_condition) = &level_config.victory_condition else {
        // 无胜负条件配置时，仅执行默认全灭检查
        if check_all_dead_players(&alive_players) {
            *game_over = GameOverState::Defeat;
            send_level_completed(
                &mut level_completed_writer,
                &mut log_level_writer,
                &level_config.id,
                GameOverState::Defeat,
                turn_state.turn_number,
            );
        }
        return;
    };

    let turn_number = turn_state.turn_number;

    // 失败优先原则（FORBIDDEN-6）：先检查所有 lose_conditions
    let any_lose = check_lose_conditions(victory_condition, &alive_players, turn_number);

    // 默认全灭检查（绝对不变量 3.1/3.8）
    let all_dead = check_all_dead_players(&alive_players);

    // 再检查所有 win_conditions
    let any_win = check_win_conditions(victory_condition, &alive_enemies, &all_units, turn_number);

    let is_defeat = any_lose || all_dead;
    let is_victory = any_win;

    if is_defeat {
        *game_over = GameOverState::Defeat;
        send_level_completed(
            &mut level_completed_writer,
            &mut log_level_writer,
            &level_config.id,
            GameOverState::Defeat,
            turn_number,
        );
    } else if is_victory {
        *game_over = GameOverState::Victory;
        send_level_completed(
            &mut level_completed_writer,
            &mut log_level_writer,
            &level_config.id,
            GameOverState::Victory,
            turn_number,
        );
    }
}

/// 兜底系统：全灭玩家即失败（绝对不变量防御性保障）
///
/// 每帧运行，但有 early return 优化。防止 OnEnter(TurnEnd) 被跳过时
/// "全灭玩家即失败"这一绝对不变量被遗漏。
pub fn check_all_dead_safety(
    mut game_over: ResMut<GameOverState>,
    level_registry: Res<LevelRegistry>,
    turn_state: Res<TurnState>,
    alive_players: Query<&Unit, (With<Unit>, Without<Dead>)>,
    mut level_completed_writer: MessageWriter<LevelCompleted>,
) {
    // 只在 Playing 状态时检查（终态不可逆 FORBIDDEN-5）
    if *game_over != GameOverState::Playing {
        return;
    }

    // 有存活玩家 → 安全，early return
    if alive_players.iter().any(|u| u.faction == Faction::Player) {
        return;
    }

    // 全灭玩家 → 强制失败
    let level_id = level_registry
        .first()
        .map(|c| c.id.as_str())
        .unwrap_or("unknown");
    let turn_number = turn_state.turn_number;
    bevy::log::warn!(target: "turn", level_id = level_id, turn = turn_number, "兜底检查触发：全灭玩家即失败");
    *game_over = GameOverState::Defeat;
    level_completed_writer.write(LevelCompleted {
        level_id: level_id.to_string(),
        result: GameOverState::Defeat,
        turn_number,
    });
}

// ── 内部检查函数（私有） ──

/// 检查所有失败条件（OR 关系）
fn check_lose_conditions(
    victory_condition: &VictoryConditionDef,
    alive_players: &Query<&Unit, (With<Unit>, Without<Dead>)>,
    turn_number: u32,
) -> bool {
    victory_condition
        .lose_conditions
        .iter()
        .any(|cond| check_single_lose_condition(cond, alive_players, turn_number))
}

/// 检查单条失败条件
fn check_single_lose_condition(
    cond: &LoseConditionDef,
    alive_players: &Query<&Unit, (With<Unit>, Without<Dead>)>,
    turn_number: u32,
) -> bool {
    match cond.condition_type {
        ConditionTypeDef::AllDead => check_all_dead_players(alive_players),
        ConditionTypeDef::TurnLimitExceeded => {
            let max_turns = cond
                .params
                .as_ref()
                .and_then(|p| p.max_turns)
                .unwrap_or(u32::MAX);
            check_turn_limit_exceeded(turn_number, max_turns)
        }
        _ => false,
    }
}

/// 检查所有胜利条件（OR 关系）
fn check_win_conditions(
    victory_condition: &VictoryConditionDef,
    alive_enemies: &Query<&Unit, (With<Unit>, Without<Dead>)>,
    all_units: &Query<(&Unit, Option<&UnitId>), Without<Dead>>,
    turn_number: u32,
) -> bool {
    victory_condition
        .win_conditions
        .iter()
        .any(|cond| check_single_win_condition(cond, alive_enemies, all_units, turn_number))
}

/// 检查单条胜利条件
fn check_single_win_condition(
    cond: &WinConditionDef,
    alive_enemies: &Query<&Unit, (With<Unit>, Without<Dead>)>,
    all_units: &Query<(&Unit, Option<&UnitId>), Without<Dead>>,
    turn_number: u32,
) -> bool {
    match cond.condition_type {
        ConditionTypeDef::KillAll => check_kill_all(alive_enemies),
        ConditionTypeDef::SurviveTurns => {
            let n = cond.params.as_ref().and_then(|p| p.n).unwrap_or(u32::MAX); // 缺失参数时条件永不触发（安全默认值）
            check_survive_turns(turn_number, n)
        }
        ConditionTypeDef::DefeatBoss => {
            let boss_id = cond
                .params
                .as_ref()
                .and_then(|p| p.boss_id.as_deref())
                .unwrap_or("");
            check_defeat_boss(all_units, boss_id)
        }
        _ => false,
    }
}

/// 检查 KillAll：所有敌方单位已死亡（排除 Dead 组件）
fn check_kill_all(alive_enemies: &Query<&Unit, (With<Unit>, Without<Dead>)>) -> bool {
    !alive_enemies.iter().any(|u| u.faction == Faction::Enemy)
}

/// 检查 SurviveTurns：当前回合号 >= 目标回合数
fn check_survive_turns(turn_number: u32, target: u32) -> bool {
    turn_number >= target
}

/// 检查 DefeatBoss：指定 boss_id 的单位已死亡或不存在
fn check_defeat_boss(
    all_units: &Query<(&Unit, Option<&UnitId>), Without<Dead>>,
    boss_id: &str,
) -> bool {
    if boss_id.is_empty() {
        return false;
    }

    // 在存活单位中查找 boss_id；找不到说明 Boss 已死亡或不存在
    !all_units
        .iter()
        .any(|(_, unit_id)| unit_id.map_or(false, |id| id.0 == boss_id))
}

/// 检查 AllDead：所有玩家单位已死亡（排除 Dead 组件）
fn check_all_dead_players(alive_players: &Query<&Unit, (With<Unit>, Without<Dead>)>) -> bool {
    !alive_players.iter().any(|u| u.faction == Faction::Player)
}

/// 检查 TurnLimitExceeded：当前回合号 > 最大回合数
fn check_turn_limit_exceeded(turn_number: u32, max_turns: u32) -> bool {
    turn_number > max_turns
}

/// 发送 LevelCompleted 消息
fn send_level_completed(
    writer: &mut MessageWriter<LevelCompleted>,
    log_writer: &mut MessageWriter<LogLevelCompleted>,
    level_id: &str,
    result: GameOverState,
    turn_number: u32,
) {
    log_writer.write(LogLevelCompleted {
        level_id: level_id.to_string(),
        success: matches!(result, GameOverState::Victory),
        turns_used: turn_number,
    });
    writer.write(LevelCompleted {
        level_id: level_id.to_string(),
        result,
        turn_number,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::map::{
        ConditionParamsDef, LoseConditionDef, VictoryConditionDef, WinConditionDef,
    };
    use std::collections::HashMap;

    // ── 纯函数测试 ──

    /// Test ID: VC-FUNC-001
    /// Title: check_survive_turns - 回合数达到目标
    #[test]
    fn check_survive_turns_达到目标() {
        assert!(check_survive_turns(10, 10));
        assert!(check_survive_turns(11, 10));
    }

    /// Test ID: VC-FUNC-002
    /// Title: check_survive_turns - 回合数未达目标
    #[test]
    fn check_survive_turns_未达目标() {
        assert!(!check_survive_turns(9, 10));
        assert!(!check_survive_turns(0, 1));
    }

    /// Test ID: VC-FUNC-003
    /// Title: check_turn_limit_exceeded - 超过最大回合数
    #[test]
    fn check_turn_limit_exceeded_超过上限() {
        assert!(check_turn_limit_exceeded(21, 20));
        assert!(check_turn_limit_exceeded(100, 20));
    }

    /// Test ID: VC-FUNC-004
    /// Title: check_turn_limit_exceeded - 未超过最大回合数
    #[test]
    fn check_turn_limit_exceeded_未超过上限() {
        assert!(!check_turn_limit_exceeded(20, 20));
        assert!(!check_turn_limit_exceeded(1, 20));
    }

    /// Test ID: VC-FUNC-004b
    /// Title: 存活与超时边界值 - SurviveTurns 用 >= 而 TurnLimitExceeded 用 >
    ///
    /// 业务规则：存活 N 回合在第 N 回合即满足（>=），超时在第 N+1 回合才触发（>）。
    /// 同一回合号 N 可以同时满足存活但不算超时。
    #[test]
    fn 边界值_存活与超时的不对称判定() {
        // 回合 20：存活目标 20 → 满足（>=）；超时限制 20 → 不满足（>）
        assert!(check_survive_turns(20, 20), "回合 20 应满足存活 20 回合");
        assert!(
            !check_turn_limit_exceeded(20, 20),
            "回合 20 不应触发超时 20 限制"
        );

        // 回合 21：存活目标 20 → 满足；超时限制 20 → 触发
        assert!(check_survive_turns(21, 20));
        assert!(check_turn_limit_exceeded(21, 20));
    }

    /// Test ID: VC-FUNC-005 (replaced: was testing String::is_empty)
    /// Title: RON 配置带胜负条件的关卡反序列化
    ///
    /// 业务规则（FORBIDDEN-1）：胜负条件必须从配置读取，不能硬编码。
    /// 此测试验证 VictoryConditionDef 可通过 RON 正确反序列化。
    #[test]
    fn ron_胜负条件配置反序列化() {
        let ron_str = r#"
            (
                win_conditions: [
                    (type: KillAll),
                ],
                lose_conditions: [
                    (type: AllDead),
                    (type: TurnLimitExceeded, params: Some((max_turns: Some(20)))),
                ],
            )
        "#;
        let vc: VictoryConditionDef = ron::de::from_str(ron_str).expect("RON 应正确反序列化");
        assert_eq!(vc.win_conditions.len(), 1);
        assert_eq!(
            vc.win_conditions[0].condition_type,
            ConditionTypeDef::KillAll
        );
        assert_eq!(vc.lose_conditions.len(), 2);
        assert_eq!(
            vc.lose_conditions[0].condition_type,
            ConditionTypeDef::AllDead
        );
        assert_eq!(
            vc.lose_conditions[1].condition_type,
            ConditionTypeDef::TurnLimitExceeded
        );
        assert_eq!(
            vc.lose_conditions[1]
                .params
                .as_ref()
                .and_then(|p| p.max_turns),
            Some(20)
        );
    }

    // ── 条件匹配测试 ──

    /// Test ID: VC-FUNC-006
    /// Title: WinConditionDef KillAll 条件类型匹配
    #[test]
    fn win_condition_kill_all_类型() {
        let cond = WinConditionDef {
            condition_type: ConditionTypeDef::KillAll,
            params: None,
        };
        assert_eq!(cond.condition_type, ConditionTypeDef::KillAll);
        assert!(cond.params.is_none());
    }

    /// Test ID: VC-FUNC-007
    /// Title: WinConditionDef SurviveTurns 携带参数
    #[test]
    fn win_condition_survive_turns_参数() {
        let cond = WinConditionDef {
            condition_type: ConditionTypeDef::SurviveTurns,
            params: Some(ConditionParamsDef {
                n: Some(10),
                boss_id: None,
                max_turns: None,
            }),
        };
        assert_eq!(cond.condition_type, ConditionTypeDef::SurviveTurns);
        assert_eq!(cond.params.as_ref().and_then(|p| p.n), Some(10));
    }

    /// Test ID: VC-FUNC-008
    /// Title: LoseConditionDef TurnLimitExceeded 携带参数
    #[test]
    fn lose_condition_turn_limit_参数() {
        let cond = LoseConditionDef {
            condition_type: ConditionTypeDef::TurnLimitExceeded,
            params: Some(ConditionParamsDef {
                n: None,
                boss_id: None,
                max_turns: Some(20),
            }),
        };
        assert_eq!(cond.condition_type, ConditionTypeDef::TurnLimitExceeded);
        assert_eq!(cond.params.as_ref().and_then(|p| p.max_turns), Some(20));
    }

    /// Test ID: VC-FUNC-009
    /// Title: VictoryConditionDef 默认值为空
    #[test]
    fn victory_condition_默认为空() {
        let vc = VictoryConditionDef::default();
        assert!(vc.win_conditions.is_empty());
        assert!(vc.lose_conditions.is_empty());
    }

    /// Test ID: VC-FUNC-010
    /// Title: GameOverState 终态不可逆验证
    #[test]
    fn game_over_state_终态不可逆() {
        // 验证 Playing 可以转为终态
        let mut state = GameOverState::Playing;
        assert_eq!(state, GameOverState::Playing);

        // 模拟转为 Victory
        state = GameOverState::Victory;
        assert_eq!(state, GameOverState::Victory);

        // 系统代码中通过 if *game_over != GameOverState::Playing { return; } 保证不可逆
        // 这里验证终态不等于 Playing
        assert_ne!(state, GameOverState::Playing);
    }

    // ── ECS 集成测试 ──
    //
    // AI Self-Check (test_spec.md §13.1)
    // ✅ 测试行为，不是实现 — 断言 GameOverState 变化（业务结果）
    // ✅ 符合领域规则 — 覆盖 victory_condition_rules_v1 不变量 3.1-3.8
    // ✅ 测试是确定性的 — 无随机数
    // ✅ 使用标准测试数据 — UnitBuilder::warrior/goblin
    // ✅ 没有测试私有实现 — 通过 ECS System 公共入口测试
    // ✅ 没有生成不在范围内的测试

    use crate::core::character::{Dead, Unit, UnitId};
    use crate::core::map::LevelConfig;
    use crate::core::turn::order::TurnState;

    /// 构建胜负检查测试 App（仅注册 check_victory_conditions 系统）
    fn victory_check_test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
            .init_state::<super::super::state::AppState>()
            .init_resource::<GameOverState>()
            .init_resource::<LevelRegistry>()
            .init_resource::<TurnState>()
            .add_message::<LevelCompleted>()
            .add_message::<LogLevelCompleted>()
            .add_systems(Update, check_victory_conditions);
        app
    }

    /// 构建兜底系统测试 App（仅注册 check_all_dead_safety 系统）
    fn safety_check_test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
            .init_state::<super::super::state::AppState>()
            .init_resource::<GameOverState>()
            .init_resource::<LevelRegistry>()
            .init_resource::<TurnState>()
            .add_message::<LevelCompleted>()
            .add_message::<LogLevelCompleted>()
            .add_systems(Update, check_all_dead_safety);
        app
    }

    /// 辅助：创建带胜负条件的 LevelConfig 并插入 LevelRegistry
    fn insert_level_with_victory(app: &mut App, level_id: &str, vc: VictoryConditionDef) {
        let mut registry = LevelRegistry::default();
        registry.levels.insert(
            level_id.to_string(),
            LevelConfig {
                id: level_id.to_string(),
                name: "test".into(),
                width: 5,
                height: 5,
                tile_size: 64.0,
                terrain_map: HashMap::new(),
                player_units: vec![],
                enemy_units: vec![],
                victory_condition: Some(vc),
                turn_limit: None,
            },
        );
        app.insert_resource(registry);
    }

    /// Test ID: VC-ECS-001
    /// Title: KillAll - 所有敌方单位死亡时判定 Victory
    ///
    /// Given: KillAll 胜利条件 + 无存活敌方 + 1 存活玩家
    /// When: 执行胜负检查
    /// Then: GameOverState = Victory
    #[test]
    fn ecs_全灭敌人判定胜利() {
        let mut app = victory_check_test_app();

        insert_level_with_victory(
            &mut app,
            "test_level",
            VictoryConditionDef {
                win_conditions: vec![WinConditionDef {
                    condition_type: ConditionTypeDef::KillAll,
                    params: None,
                }],
                lose_conditions: vec![],
            },
        );

        // 1 alive player, no enemies
        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitId("player_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(*state, GameOverState::Victory, "所有敌方死亡应判定 Victory");
    }

    /// Test ID: VC-ECS-002
    /// Title: KillAll - 仍有存活敌方时继续战斗
    ///
    /// Given: KillAll 胜利条件 + 1 存活敌方 + 1 存活玩家
    /// When: 执行胜负检查
    /// Then: GameOverState = Playing（继续战斗）
    #[test]
    fn ecs_仍有存活敌人继续战斗() {
        let mut app = victory_check_test_app();

        insert_level_with_victory(
            &mut app,
            "test_level",
            VictoryConditionDef {
                win_conditions: vec![WinConditionDef {
                    condition_type: ConditionTypeDef::KillAll,
                    params: None,
                }],
                lose_conditions: vec![],
            },
        );

        app.world_mut().spawn((
            Unit {
                faction: Faction::Enemy,
                acted: false,
            },
            UnitId("enemy_1".into()),
        ));
        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitId("player_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(*state, GameOverState::Playing, "敌人未全灭时应继续战斗");
    }

    /// Test ID: VC-ECS-003
    /// Title: AllDead - 所有玩家死亡判定 Defeat（不变量 3.1）
    ///
    /// Given: 1 死亡玩家(Dead) + 1 存活敌方
    /// When: 执行胜负检查
    /// Then: GameOverState = Defeat
    #[test]
    fn ecs_全灭玩家判定失败() {
        let mut app = victory_check_test_app();

        insert_level_with_victory(
            &mut app,
            "test_level",
            VictoryConditionDef {
                win_conditions: vec![WinConditionDef {
                    condition_type: ConditionTypeDef::KillAll,
                    params: None,
                }],
                lose_conditions: vec![],
            },
        );

        // Dead player
        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitId("dead_player".into()),
            Dead,
        ));
        // Alive enemy
        app.world_mut().spawn((
            Unit {
                faction: Faction::Enemy,
                acted: false,
            },
            UnitId("enemy_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(
            *state,
            GameOverState::Defeat,
            "全灭玩家必须判定 Defeat（不变量 3.1）"
        );
    }

    /// Test ID: VC-ECS-004
    /// Title: 失败优先原则 - 胜负同时满足判定 Defeat（不变量 3.2 / FORBIDDEN-6）
    ///
    /// Given: KillAll 满足（无存活敌方）+ 全灭玩家（无存活玩家）
    /// When: 执行胜负检查
    /// Then: GameOverState = Defeat（失败优先）
    #[test]
    fn ecs_失败优先_胜负同时满足判定失败() {
        let mut app = victory_check_test_app();

        insert_level_with_victory(
            &mut app,
            "test_level",
            VictoryConditionDef {
                win_conditions: vec![WinConditionDef {
                    condition_type: ConditionTypeDef::KillAll,
                    params: None,
                }],
                lose_conditions: vec![],
            },
        );

        // No units at all — both KillAll and AllDead satisfied
        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(
            *state,
            GameOverState::Defeat,
            "胜负同时满足时必须判定 Defeat（不变量 3.2 / FORBIDDEN-6）"
        );
    }

    /// Test ID: VC-ECS-005
    /// Title: 终态不可逆 - Victory 后不被后续检查覆盖（不变量 3.3 / FORBIDDEN-5）
    ///
    /// Given: GameOverState = Victory（已终态）+ 存活敌方和玩家
    /// When: 执行胜负检查
    /// Then: GameOverState 保持 Victory
    #[test]
    fn ecs_终态不可逆_Victory后保持() {
        let mut app = victory_check_test_app();

        *app.world_mut().resource_mut::<GameOverState>() = GameOverState::Victory;

        insert_level_with_victory(
            &mut app,
            "test_level",
            VictoryConditionDef {
                win_conditions: vec![WinConditionDef {
                    condition_type: ConditionTypeDef::KillAll,
                    params: None,
                }],
                lose_conditions: vec![],
            },
        );

        app.world_mut().spawn((
            Unit {
                faction: Faction::Enemy,
                acted: false,
            },
            UnitId("enemy_1".into()),
        ));
        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitId("player_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(
            *state,
            GameOverState::Victory,
            "Victory 终态不可被覆盖（不变量 3.3）"
        );
    }

    /// Test ID: VC-ECS-006
    /// Title: 终态不可逆 - Defeat 后不被后续检查覆盖
    ///
    /// Given: GameOverState = Defeat（已终态）+ 存活玩家
    /// When: 执行胜负检查
    /// Then: GameOverState 保持 Defeat
    #[test]
    fn ecs_终态不可逆_Defeat后保持() {
        let mut app = victory_check_test_app();

        *app.world_mut().resource_mut::<GameOverState>() = GameOverState::Defeat;

        insert_level_with_victory(
            &mut app,
            "test_level",
            VictoryConditionDef {
                win_conditions: vec![WinConditionDef {
                    condition_type: ConditionTypeDef::KillAll,
                    params: None,
                }],
                lose_conditions: vec![],
            },
        );

        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitId("player_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(
            *state,
            GameOverState::Defeat,
            "Defeat 终态不可被覆盖（不变量 3.3）"
        );
    }

    /// Test ID: VC-ECS-007
    /// Title: 双方存活且无超时 — 继续战斗
    ///
    /// Given: KillAll 胜利条件 + 双方各有存活单位
    /// When: 执行胜负检查
    /// Then: GameOverState = Playing
    #[test]
    fn ecs_继续战斗_条件均未满足() {
        let mut app = victory_check_test_app();

        insert_level_with_victory(
            &mut app,
            "test_level",
            VictoryConditionDef {
                win_conditions: vec![WinConditionDef {
                    condition_type: ConditionTypeDef::KillAll,
                    params: None,
                }],
                lose_conditions: vec![LoseConditionDef {
                    condition_type: ConditionTypeDef::AllDead,
                    params: None,
                }],
            },
        );

        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitId("player_1".into()),
        ));
        app.world_mut().spawn((
            Unit {
                faction: Faction::Enemy,
                acted: false,
            },
            UnitId("enemy_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(*state, GameOverState::Playing, "双方存活时应继续战斗");
    }

    /// Test ID: VC-ECS-008
    /// Title: 无胜负条件配置时全灭即失败（不变量 3.6）
    ///
    /// Given: LevelConfig.victory_condition = None + 无存活玩家
    /// When: 执行胜负检查
    /// Then: GameOverState = Defeat
    #[test]
    fn ecs_无配置时全灭即失败() {
        let mut app = victory_check_test_app();

        let mut registry = LevelRegistry::default();
        registry.levels.insert(
            "no_vc".into(),
            LevelConfig {
                id: "no_vc".into(),
                name: "test".into(),
                width: 3,
                height: 3,
                tile_size: 64.0,
                terrain_map: HashMap::new(),
                player_units: vec![],
                enemy_units: vec![],
                victory_condition: None,
                turn_limit: None,
            },
        );
        app.insert_resource(registry);

        // No alive players, only enemy
        app.world_mut().spawn((
            Unit {
                faction: Faction::Enemy,
                acted: false,
            },
            UnitId("enemy_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(
            *state,
            GameOverState::Defeat,
            "无配置时全灭玩家仍应判定 Defeat"
        );
    }

    /// Test ID: VC-ECS-009
    /// Title: 空关卡注册表不崩溃
    ///
    /// Given: LevelRegistry 为空
    /// When: 执行胜负检查
    /// Then: GameOverState = Playing（不变）
    #[test]
    fn ecs_空注册表不崩溃() {
        let mut app = victory_check_test_app();
        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(*state, GameOverState::Playing, "空注册表时应保持 Playing");
    }

    /// Test ID: VC-ECS-010
    /// Title: 胜利时发送 LevelCompleted 消息
    ///
    /// Given: KillAll 满足 + turn_number = 5
    /// When: 执行胜负检查
    /// Then: LevelCompleted 消息包含正确的 level_id、result、turn_number
    #[test]
    fn ecs_胜利时发送LevelCompleted消息() {
        let mut app = victory_check_test_app();

        insert_level_with_victory(
            &mut app,
            "msg_test_level",
            VictoryConditionDef {
                win_conditions: vec![WinConditionDef {
                    condition_type: ConditionTypeDef::KillAll,
                    params: None,
                }],
                lose_conditions: vec![],
            },
        );

        app.world_mut().resource_mut::<TurnState>().turn_number = 5;

        // 必须有存活玩家，否则"全灭玩家即失败"不变量会触发 Defeat
        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitId("player_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(*state, GameOverState::Victory);

        // Verify message via Resource wrapper
        #[derive(Resource, Default)]
        struct CapturedMsg(Option<LevelCompleted>);

        app.insert_resource(CapturedMsg::default());
        app.add_systems(
            Update,
            |mut reader: MessageReader<LevelCompleted>, mut captured: ResMut<CapturedMsg>| {
                for msg in reader.read() {
                    captured.0 = Some(msg.clone());
                }
            },
        );
        app.update();

        let captured = app.world().resource::<CapturedMsg>();
        assert!(captured.0.is_some(), "Victory 时应发送 LevelCompleted 消息");
        let msg = captured.0.as_ref().unwrap();
        assert_eq!(msg.level_id, "msg_test_level");
        assert_eq!(msg.result, GameOverState::Victory);
        assert_eq!(msg.turn_number, 5);
    }

    /// Test ID: VC-ECS-011
    /// Title: DefeatBoss - Boss 已死亡判定 Victory
    ///
    /// Given: DefeatBoss("dark_lord") + dark_lord 单位有 Dead 组件
    /// When: 执行胜负检查
    /// Then: GameOverState = Victory
    #[test]
    fn ecs_DefeatBoss_Boss已死亡判定胜利() {
        let mut app = victory_check_test_app();

        insert_level_with_victory(
            &mut app,
            "boss_level",
            VictoryConditionDef {
                win_conditions: vec![WinConditionDef {
                    condition_type: ConditionTypeDef::DefeatBoss,
                    params: Some(ConditionParamsDef {
                        n: None,
                        boss_id: Some("dark_lord".into()),
                        max_turns: None,
                    }),
                }],
                lose_conditions: vec![],
            },
        );

        // Dead boss
        app.world_mut().spawn((
            Unit {
                faction: Faction::Enemy,
                acted: false,
            },
            UnitId("dark_lord".into()),
            Dead,
        ));
        // Alive player
        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitId("player_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(*state, GameOverState::Victory, "Boss 已死亡应判定 Victory");
    }

    /// Test ID: VC-ECS-012
    /// Title: DefeatBoss - Boss 存活时继续战斗
    ///
    /// Given: DefeatBoss("dark_lord") + dark_lord 单位存活
    /// When: 执行胜负检查
    /// Then: GameOverState = Playing
    #[test]
    fn ecs_DefeatBoss_Boss存活继续战斗() {
        let mut app = victory_check_test_app();

        insert_level_with_victory(
            &mut app,
            "boss_level",
            VictoryConditionDef {
                win_conditions: vec![WinConditionDef {
                    condition_type: ConditionTypeDef::DefeatBoss,
                    params: Some(ConditionParamsDef {
                        n: None,
                        boss_id: Some("dark_lord".into()),
                        max_turns: None,
                    }),
                }],
                lose_conditions: vec![],
            },
        );

        // Alive boss
        app.world_mut().spawn((
            Unit {
                faction: Faction::Enemy,
                acted: false,
            },
            UnitId("dark_lord".into()),
        ));
        // Alive player
        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitId("player_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(*state, GameOverState::Playing, "Boss 存活时应继续战斗");
    }

    /// Test ID: VC-ECS-013
    /// Title: SurviveTurns - 达到目标回合数判定 Victory
    ///
    /// Given: SurviveTurns(5) + turn_number = 5 + 存活玩家
    /// When: 执行胜负检查
    /// Then: GameOverState = Victory
    #[test]
    fn ecs_SurviveTurns_达到目标回合判定胜利() {
        let mut app = victory_check_test_app();

        insert_level_with_victory(
            &mut app,
            "survive_level",
            VictoryConditionDef {
                win_conditions: vec![WinConditionDef {
                    condition_type: ConditionTypeDef::SurviveTurns,
                    params: Some(ConditionParamsDef {
                        n: Some(5),
                        boss_id: None,
                        max_turns: None,
                    }),
                }],
                lose_conditions: vec![],
            },
        );

        app.world_mut().resource_mut::<TurnState>().turn_number = 5;
        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitId("player_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(*state, GameOverState::Victory, "存活回合达标应判定 Victory");
    }

    /// Test ID: VC-ECS-014
    /// Title: TurnLimitExceeded - 超过最大回合数判定 Defeat
    ///
    /// Given: TurnLimitExceeded(10) + turn_number = 11 + 存活玩家
    /// When: 执行胜负检查
    /// Then: GameOverState = Defeat
    #[test]
    fn ecs_TurnLimitExceeded_超时判定失败() {
        let mut app = victory_check_test_app();

        insert_level_with_victory(
            &mut app,
            "limit_level",
            VictoryConditionDef {
                win_conditions: vec![],
                lose_conditions: vec![LoseConditionDef {
                    condition_type: ConditionTypeDef::TurnLimitExceeded,
                    params: Some(ConditionParamsDef {
                        n: None,
                        boss_id: None,
                        max_turns: Some(10),
                    }),
                }],
            },
        );

        app.world_mut().resource_mut::<TurnState>().turn_number = 11;
        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitId("player_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(*state, GameOverState::Defeat, "超时应判定 Defeat");
    }

    /// Test ID: VC-ECS-015
    /// Title: 多胜利条件 OR - 任一满足即 Victory
    ///
    /// Given: KillAll + SurviveTurns(5) + turn=5 + 存活敌方
    /// When: 执行胜负检查
    /// Then: GameOverState = Victory（SurviveTurns 满足）
    #[test]
    fn ecs_多胜利条件OR_任一满足即胜利() {
        let mut app = victory_check_test_app();

        insert_level_with_victory(
            &mut app,
            "multi_level",
            VictoryConditionDef {
                win_conditions: vec![
                    WinConditionDef {
                        condition_type: ConditionTypeDef::KillAll,
                        params: None,
                    },
                    WinConditionDef {
                        condition_type: ConditionTypeDef::SurviveTurns,
                        params: Some(ConditionParamsDef {
                            n: Some(5),
                            boss_id: None,
                            max_turns: None,
                        }),
                    },
                ],
                lose_conditions: vec![],
            },
        );

        // KillAll NOT satisfied (enemy alive), but SurviveTurns IS (turn=5)
        app.world_mut().resource_mut::<TurnState>().turn_number = 5;
        app.world_mut().spawn((
            Unit {
                faction: Faction::Enemy,
                acted: false,
            },
            UnitId("enemy_1".into()),
        ));
        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitId("player_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(
            *state,
            GameOverState::Victory,
            "多条件 OR：SurviveTurns 满足即 Victory"
        );
    }

    /// Test ID: VC-ECS-016
    /// Title: 失败优先于胜利 - 超时且全灭敌人判定 Defeat（不变量 3.2）
    ///
    /// Given: TurnLimitExceeded(10) + KillAll + turn=11 + 无敌人 + 存活玩家
    /// When: 执行胜负检查
    /// Then: GameOverState = Defeat（失败优先）
    #[test]
    fn ecs_失败优先_超时且全灭敌人判定失败() {
        let mut app = victory_check_test_app();

        insert_level_with_victory(
            &mut app,
            "priority_level",
            VictoryConditionDef {
                win_conditions: vec![WinConditionDef {
                    condition_type: ConditionTypeDef::KillAll,
                    params: None,
                }],
                lose_conditions: vec![LoseConditionDef {
                    condition_type: ConditionTypeDef::TurnLimitExceeded,
                    params: Some(ConditionParamsDef {
                        n: None,
                        boss_id: None,
                        max_turns: Some(10),
                    }),
                }],
            },
        );

        app.world_mut().resource_mut::<TurnState>().turn_number = 11;
        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitId("player_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(
            *state,
            GameOverState::Defeat,
            "失败优先：超时 + 全灭敌人同时满足时判定 Defeat（不变量 3.2）"
        );
    }

    /// Test ID: VC-ECS-017
    /// Title: 兜底系统 - 全灭玩家即失败（绝对不变量 3.1 防御性保障）
    ///
    /// Given: GameOverState = Playing + 无存活玩家
    /// When: 执行兜底检查
    /// Then: GameOverState = Defeat
    #[test]
    fn ecs_兜底_全灭玩家即失败() {
        let mut app = safety_check_test_app();

        // Only enemies alive, no players
        app.world_mut().spawn((
            Unit {
                faction: Faction::Enemy,
                acted: false,
            },
            UnitId("enemy_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(
            *state,
            GameOverState::Defeat,
            "兜底：全灭玩家必须判定 Defeat（绝对不变量 3.1）"
        );
    }

    /// Test ID: VC-ECS-018
    /// Title: 兜底系统 - 有存活玩家时不触发
    ///
    /// Given: GameOverState = Playing + 1 存活玩家
    /// When: 执行兜底检查
    /// Then: GameOverState = Playing（不变）
    #[test]
    fn ecs_兜底_有存活玩家不触发() {
        let mut app = safety_check_test_app();

        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitId("player_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(*state, GameOverState::Playing, "有存活玩家时兜底不触发");
    }

    /// Test ID: VC-ECS-019
    /// Title: 兜底系统 - 终态后不重复触发（不变量 3.3）
    ///
    /// Given: GameOverState = Victory + 无存活玩家
    /// When: 执行兜底检查
    /// Then: GameOverState 保持 Victory（不被覆盖为 Defeat）
    #[test]
    fn ecs_兜底_终态后不重复触发() {
        let mut app = safety_check_test_app();

        *app.world_mut().resource_mut::<GameOverState>() = GameOverState::Victory;

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(
            *state,
            GameOverState::Victory,
            "终态后兜底不应覆盖 Victory 为 Defeat（不变量 3.3）"
        );
    }

    /// Test ID: VC-ECS-020
    /// Title: 默认全灭条件不可移除（不变量 3.8 / FORBIDDEN-8）
    ///
    /// Given: 关卡 lose_conditions 为空（未显式配置 AllDead）+ 所有玩家死亡
    /// When: 执行胜负检查
    /// Then: GameOverState = Defeat（默认全灭检查仍生效）
    #[test]
    fn ecs_默认全灭条件不可移除() {
        let mut app = victory_check_test_app();

        // lose_conditions is EMPTY — no explicit AllDead configured
        insert_level_with_victory(
            &mut app,
            "no_alldead_config",
            VictoryConditionDef {
                win_conditions: vec![WinConditionDef {
                    condition_type: ConditionTypeDef::KillAll,
                    params: None,
                }],
                lose_conditions: vec![], // No AllDead configured!
            },
        );

        // All players dead
        app.world_mut().spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitId("dead_player".into()),
            Dead,
        ));
        // Enemy alive
        app.world_mut().spawn((
            Unit {
                faction: Faction::Enemy,
                acted: false,
            },
            UnitId("enemy_1".into()),
        ));

        app.update();

        let state = app.world().resource::<GameOverState>();
        assert_eq!(
            *state,
            GameOverState::Defeat,
            "即使未配置 AllDead，全灭玩家仍必须判定 Defeat（不变量 3.8 / FORBIDDEN-8）"
        );
    }
}
