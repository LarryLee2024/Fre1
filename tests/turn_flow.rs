//! P2 集成测试：回合管理流程
//!
//! 跨 turn + character 模块测试回合状态机流转、行动队列构建、阵营切换。

use bevy::prelude::*;
use tactical_rpg::character::{Faction, Unit};
use tactical_rpg::core::attribute::{AttributeKind, Attributes};
use tactical_rpg::turn::{
    ForceEndFaction, NeedsResolve, TurnOrder, TurnPhase, TurnState, turn_end_on_enter,
};

// ── 测试辅助 ──

fn setup_turn_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<TurnPhase>()
        .init_resource::<TurnState>()
        .init_resource::<TurnOrder>()
        .init_resource::<tactical_rpg::turn::AiTimer>()
        .init_resource::<NeedsResolve>()
        .init_resource::<ForceEndFaction>()
        .add_systems(OnEnter(TurnPhase::TurnEnd), turn_end_on_enter);
    app
}

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
// 场景一：行动队列构建
// ══════════════════════════════════════════════════════════════

#[test]
fn 行动队列_按initiative降序排列() {
    let e1 = Entity::from_bits(1);
    let e2 = Entity::from_bits(2);
    let e3 = Entity::from_bits(3);
    let queue = TurnOrder::build(&[(e1, 10.0), (e2, 20.0), (e3, 15.0)]);
    assert_eq!(queue, vec![e2, e3, e1]);
}

#[test]
fn 行动队列_相同initiative稳定排序() {
    let e1 = Entity::from_bits(1);
    let e2 = Entity::from_bits(2);
    let queue = TurnOrder::build(&[(e1, 10.0), (e2, 10.0)]);
    assert_eq!(queue, vec![e1, e2]);
}

#[test]
fn 行动队列_空队列() {
    let queue = TurnOrder::build(&[]);
    assert!(queue.is_empty());
}

#[test]
fn 行动队列_current_unit和advance() {
    let e1 = Entity::from_bits(1);
    let e2 = Entity::from_bits(2);
    let e3 = Entity::from_bits(3);
    let mut order = TurnOrder {
        queue: vec![e1, e2, e3],
        current_index: 0,
        turn_number: 1,
    };
    assert_eq!(order.current_unit(), Some(e1));
    assert_eq!(order.advance(), Some(e2));
    assert_eq!(order.current_unit(), Some(e2));
    assert_eq!(order.advance(), Some(e3));
    assert_eq!(order.advance(), None);
}

// ══════════════════════════════════════════════════════════════
// 场景二：回合结束 → 重建队列 + 回合数+1
// ══════════════════════════════════════════════════════════════

#[test]
fn 回合结束_重建队列并增加回合数() {
    let mut app = setup_turn_test_app();

    spawn_unit(&mut app, Faction::Player, 10.0);
    spawn_unit(&mut app, Faction::Enemy, 8.0);

    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();

    let turn_state = app.world().resource::<TurnState>();
    assert_eq!(turn_state.turn_number, 2);

    let turn_order = app.world().resource::<TurnOrder>();
    assert!(!turn_order.queue.is_empty());
    assert_eq!(turn_order.current_index, 0);
}

#[test]
fn 回合结束_needs_resolve标记设置() {
    let mut app = setup_turn_test_app();

    spawn_unit(&mut app, Faction::Player, 10.0);

    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();

    let needs_resolve = app.world().resource::<NeedsResolve>();
    assert!(needs_resolve.0);
}

#[test]
fn 回合结束_总是切换到_select_unit() {
    let mut app = setup_turn_test_app();

    spawn_unit(&mut app, Faction::Player, 10.0);

    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();

    let phase = app.world().resource::<State<TurnPhase>>();
    assert_eq!(*phase.get(), TurnPhase::SelectUnit);
}

// ══════════════════════════════════════════════════════════════
// 场景三：阵营切换
// ══════════════════════════════════════════════════════════════

#[test]
fn 回合结束_当前阵营为队首单位阵营() {
    let mut app = setup_turn_test_app();

    // 敌方先行动（initiative 高）
    spawn_unit(&mut app, Faction::Enemy, 20.0);
    spawn_unit(&mut app, Faction::Player, 10.0);

    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();

    let turn_state = app.world().resource::<TurnState>();
    assert_eq!(turn_state.current_faction, Faction::Enemy);
}

// ══════════════════════════════════════════════════════════════
// 场景四：强制结束回合
// ══════════════════════════════════════════════════════════════

#[test]
fn 强制结束_重置标记() {
    let mut app = setup_turn_test_app();

    spawn_unit(&mut app, Faction::Player, 10.0);

    // 设置强制结束
    app.world_mut().resource_mut::<ForceEndFaction>().0 = true;

    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();

    let force_end = app.world().resource::<ForceEndFaction>();
    assert!(!force_end.0); // 已重置
}

// ══════════════════════════════════════════════════════════════
// 场景五：AI 计时器重置
// ══════════════════════════════════════════════════════════════

#[test]
fn 回合结束_ai计时器重置() {
    let mut app = setup_turn_test_app();

    spawn_unit(&mut app, Faction::Player, 10.0);

    // 模拟计时器已过期
    {
        let mut timer = app
            .world_mut()
            .resource_mut::<tactical_rpg::turn::AiTimer>();
        timer.timer.tick(std::time::Duration::from_secs(5));
        assert!(timer.timer.just_finished());
    }

    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();

    let timer = app.world().resource::<tactical_rpg::turn::AiTimer>();
    assert!(!timer.timer.just_finished()); // 已重置
}

// ══════════════════════════════════════════════════════════════
// 场景六：多次回合结束
// ══════════════════════════════════════════════════════════════

#[test]
fn 多次回合结束_回合数持续递增() {
    let mut app = setup_turn_test_app();

    spawn_unit(&mut app, Faction::Player, 10.0);

    // 第1次回合结束
    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();
    assert_eq!(app.world().resource::<TurnState>().turn_number, 2);

    // 第2次回合结束
    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();
    assert_eq!(app.world().resource::<TurnState>().turn_number, 3);

    // 第3次回合结束
    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();
    assert_eq!(app.world().resource::<TurnState>().turn_number, 4);
}

// ══════════════════════════════════════════════════════════════
// 场景七：回合结束时重置 acted 状态
// ══════════════════════════════════════════════════════════════

#[test]
fn 回合结束_重置所有单位acted状态() {
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

    // 验证 acted 状态
    assert!(app.world().get::<Unit>(e1).unwrap().acted);
    assert!(app.world().get::<Unit>(e2).unwrap().acted);

    // 触发回合结束
    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();

    // 验证 acted 状态已重置
    assert!(!app.world().get::<Unit>(e1).unwrap().acted);
    assert!(!app.world().get::<Unit>(e2).unwrap().acted);
}

#[test]
fn 回合结束_只有部分单位acted也全部重置() {
    let mut app = setup_turn_test_app();

    let e1 = spawn_unit(&mut app, Faction::Player, 10.0);
    let e2 = spawn_unit(&mut app, Faction::Enemy, 8.0);

    // 只有 e1 已行动
    {
        let mut unit1 = app.world_mut().get_mut::<Unit>(e1).unwrap();
        unit1.acted = true;
    }

    assert!(app.world().get::<Unit>(e1).unwrap().acted);
    assert!(!app.world().get::<Unit>(e2).unwrap().acted);

    // 触发回合结束
    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();

    // 验证所有 acted 状态已重置
    assert!(!app.world().get::<Unit>(e1).unwrap().acted);
    assert!(!app.world().get::<Unit>(e2).unwrap().acted);
}

// ══════════════════════════════════════════════════════════════
// 场景八：单位被击败后重建队列不包含已死亡单位
// ══════════════════════════════════════════════════════════════

#[test]
fn 回合结束_重建队列过滤已死亡单位() {
    let mut app = setup_turn_test_app();

    let e1 = spawn_unit(&mut app, Faction::Player, 10.0);
    let e2 = spawn_unit(&mut app, Faction::Enemy, 8.0);
    let e3 = spawn_unit(&mut app, Faction::Player, 5.0);

    // 初始回合结束，建立队列
    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();

    // 验证队列包含所有3个单位
    let turn_order = app.world().resource::<TurnOrder>();
    assert_eq!(turn_order.queue.len(), 3);
    assert!(turn_order.queue.contains(&e1));
    assert!(turn_order.queue.contains(&e2));
    assert!(turn_order.queue.contains(&e3));

    // 模拟 e2 被击败（销毁实体）
    app.world_mut().despawn(e2);

    // 再次触发回合结束，重建队列
    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();

    // 验证队列只包含存活的单位
    let turn_order = app.world().resource::<TurnOrder>();
    assert_eq!(turn_order.queue.len(), 2);
    assert!(turn_order.queue.contains(&e1));
    assert!(!turn_order.queue.contains(&e2)); // e2 已死亡，不在队列中
    assert!(turn_order.queue.contains(&e3));
}

#[test]
fn 回合结束_所有敌方死亡后队列只有玩家() {
    let mut app = setup_turn_test_app();

    let e1 = spawn_unit(&mut app, Faction::Player, 10.0);
    let e2 = spawn_unit(&mut app, Faction::Enemy, 8.0);

    // 初始回合结束
    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();

    // 敌方被击败
    app.world_mut().despawn(e2);

    // 再次触发回合结束
    app.world_mut()
        .resource_mut::<NextState<TurnPhase>>()
        .set(TurnPhase::TurnEnd);
    app.update();

    // 队列只包含玩家单位
    let turn_order = app.world().resource::<TurnOrder>();
    assert_eq!(turn_order.queue.len(), 1);
    assert!(turn_order.queue.contains(&e1));
    assert!(!turn_order.queue.contains(&e2));
}
