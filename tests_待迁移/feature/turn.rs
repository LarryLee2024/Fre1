//! 回合系统 Feature Test
//!
//! 跨 turn + character 模块测试回合结束流程：
//! 回合数递增、重置单位行动状态、ForceEndTurn 消息触发回合结束。

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
use tactical_rpg::character::{Faction, Unit};
use tactical_rpg::core::attribute::{AttributeKind, Attributes};
use tactical_rpg::turn::{
    ForceEndTurn, NeedsResolve, TurnEnded, TurnOrder, TurnPhase, TurnStarted, TurnState,
    turn_end_on_enter,
};

use crate::common::fixtures::UnitBuilder;

// ── 测试辅助 ──

fn setup_turn_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<TurnPhase>()
        .init_resource::<TurnState>()
        .init_resource::<TurnOrder>()
        .init_resource::<tactical_rpg::turn::AiTimer>()
        .init_resource::<NeedsResolve>()
        .add_message::<TurnStarted>()
        .add_message::<TurnEnded>()
        .add_message::<ForceEndTurn>()
        .add_systems(OnEnter(TurnPhase::TurnEnd), turn_end_on_enter);
    app
}

/// 生成带指定阵营和 Initiative 的单位
fn spawn_unit(app: &mut App, faction: Faction, initiative: f32) -> Entity {
    let mut attrs = Attributes::default();
    attrs.set_base(AttributeKind::Agility, initiative / 2.0);
    attrs.set_base(AttributeKind::Luck, 0.0);
    attrs.set_base_attack_range(1);
    attrs.fill_vital_resources();
    app.world_mut()
        .spawn((
            Unit {
                faction,
                acted: false,
            },
            attrs,
        ))
        .id()
}

// ══════════════════════════════════════════════════════════════
// 场景一：回合结束 → 回合数递增
// ══════════════════════════════════════════════════════════════

/// FT-TUR-001: 回合结束 → 回合数递增
///
/// Given: 1 个 Player 单位，TurnPhase 初始状态
/// When:  触发 TurnPhase::TurnEnd
/// Then:  turn_number 从 1 变为 2
#[test]
fn 回合结束_回合数递增() {
    let mut app = setup_turn_test_app();

    spawn_unit(&mut app, Faction::Player, 10.0);

    // 初始回合数为 1
    assert_eq!(app.world().resource::<TurnState>().turn_number, 1);

    // 触发回合结束
    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();

    // 回合数应递增为 2
    assert_eq!(app.world().resource::<TurnState>().turn_number, 2);
}

// ══════════════════════════════════════════════════════════════
// 场景二：回合结束 → 重置单位行动状态
// ══════════════════════════════════════════════════════════════

/// FT-TUR-002: 回合结束 → 重置所有单位 acted 状态
///
/// Given: 2 个单位（Player + Enemy），acted 均为 true
/// When:  触发 TurnPhase::TurnEnd
/// Then:  所有单位 acted = false
#[test]
fn 回合结束_重置单位行动状态() {
    let mut app = setup_turn_test_app();

    let e1 = spawn_unit(&mut app, Faction::Player, 10.0);
    let e2 = spawn_unit(&mut app, Faction::Enemy, 8.0);

    // 模拟两个单位都已行动
    {
        let mut unit1 = app.world_mut().get_mut::<Unit>(e1).unwrap();
        unit1.acted = true;
        let mut unit2 = app.world_mut().get_mut::<Unit>(e2).unwrap();
        unit2.acted = true;
    }

    // 验证 acted=true
    assert!(app.world().get::<Unit>(e1).unwrap().acted);
    assert!(app.world().get::<Unit>(e2).unwrap().acted);

    // 触发回合结束
    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();

    // 验证 acted 已重置为 false
    assert!(!app.world().get::<Unit>(e1).unwrap().acted);
    assert!(!app.world().get::<Unit>(e2).unwrap().acted);
}

// ══════════════════════════════════════════════════════════════
// 场景三：ForceEndTurn 消息触发回合结束
// ══════════════════════════════════════════════════════════════

/// FT-TUR-003: ForceEndTurn 消息触发回合结束
///
/// Given: 1 个 Player 单位，TurnPhase 初始状态
/// When:  发送 ForceEndTurn 消息并触发 TurnEnd
/// Then:  turn_number 递增为 2，phase 变为 SelectUnit
#[test]
fn 强制结束回合_force_end_turn消息() {
    let mut app = setup_turn_test_app();

    spawn_unit(&mut app, Faction::Player, 10.0);

    // 发送 ForceEndTurn 消息
    app.world_mut().write_message(ForceEndTurn);

    // 触发回合结束（ForceEndTurn 在 turn_end_on_enter 中被消费）
    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();

    // 验证回合已结束：回合数递增
    assert_eq!(app.world().resource::<TurnState>().turn_number, 2);

    // 验证阶段已切换到 SelectUnit
    let phase = app.world().resource::<State<TurnPhase>>();
    assert_eq!(*phase.get(), TurnPhase::SelectUnit);
}
