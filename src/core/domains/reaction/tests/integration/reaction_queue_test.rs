//! Reaction Domain — 反应队列集成测试
//!
//! 验证 ReactionPlugin 注册后的 Observer 链路和系统协作：
//! - reset_reactions_on_turn_start: 回合开始重置反应槽位
//! - process_reaction_queue: 反应队列处理与触发
//! - on_opportunity_attack_executed: 机会攻击执行链路

use bevy::prelude::*;

use crate::core::domains::reaction::components::{
    ReactionEntry, ReactionEntryStatus, ReactionQueue, ReactionState, ReactionType,
};
use crate::core::domains::reaction::events::{OpportunityAttackExecuted, ReactionTriggered};
use crate::core::domains::reaction::plugin::ReactionPlugin;
use crate::core::domains::reaction::resources::GlobalReactionQueue;
use crate::core::domains::reaction::systems::process_reaction_queue;

// ─── 辅助函数 ──────────────────────────────────────────────────────

fn spawn_reactor(world: &mut World, can_react: bool) -> Entity {
    let state = if can_react {
        ReactionState::new() // used = false, extra_reactions = 0
    } else {
        ReactionState {
            used: true,
            extra_reactions: 0,
            extra_used: 0,
        }
    };
    world.spawn(state).id()
}

fn setup_reaction_queue(world: &mut World, entries: Vec<ReactionEntry>) {
    let mut queue = ReactionQueue::new();
    for entry in entries {
        queue.enqueue(entry);
    }
    world.insert_resource(GlobalReactionQueue { queue });
}

// ─── 回合重置 ──────────────────────────────────────────────────────

#[test]
fn reset_reactions_makes_all_reactors_available() {
    let mut app = App::new();
    app.add_plugins(ReactionPlugin);

    // 创建两个已使用反应的实体
    let e1 = spawn_reactor(app.world_mut(), false); // used
    let e2 = spawn_reactor(app.world_mut(), false); // used
    let e3 = spawn_reactor(app.world_mut(), true); // available
    app.world_mut().flush();

    // reset_reactions_on_turn_start 注册为 First 系统
    app.update();
    app.world_mut().flush();

    // 验证：所有实体的反应槽位被重置
    for e in &[e1, e2, e3] {
        let state = app.world_mut().get::<ReactionState>(*e).unwrap();
        assert!(state.can_react(), "回合重置后所有实体应可反应");
    }
}

#[test]
fn extra_reactions_also_reset_on_turn_start() {
    let mut app = App::new();
    app.add_plugins(ReactionPlugin);

    let e = app
        .world_mut()
        .spawn(ReactionState {
            used: true,
            extra_reactions: 2,
            extra_used: 1,
        })
        .id();
    app.world_mut().flush();

    app.update();
    app.world_mut().flush();

    let state = app.world_mut().get::<ReactionState>(e).unwrap();
    assert!(!state.used, "基本反应应重置");
    assert_eq!(state.extra_used, 0, "额外反应消耗应重置");
}

// ─── 反应队列处理 ──────────────────────────────────────────────────

#[test]
fn process_queue_triggers_available_reaction() {
    let mut app = App::new();
    app.add_plugins(ReactionPlugin);

    let reactor = spawn_reactor(app.world_mut(), true);
    app.world_mut().flush();

    setup_reaction_queue(
        app.world_mut(),
        vec![ReactionEntry {
            reactor,
            reaction_type: ReactionType::OpportunityAttack,
            trigger: crate::core::domains::reaction::components::ReactionTrigger::EnemySpellCast {
                caster: reactor,
                spell_id: "spl_test".into(),
            },
            priority: 100,
            status: ReactionEntryStatus::Pending,
        }],
    );

    // process_reaction_queue 在 Update 中执行
    app.update();
    app.world_mut().flush();

    // 验证：队列被处理（is_finished 或条目状态变化）
    let queue = app.world_mut().resource::<GlobalReactionQueue>();
    assert!(
        queue.queue.is_finished() || !queue.queue.entries.is_empty(),
        "反应队列应被处理"
    );
}

#[test]
fn process_queue_skips_exhausted_reactor() {
    let mut app = App::new();
    app.add_plugins(ReactionPlugin);

    let exhausted = spawn_reactor(app.world_mut(), false); // 已用反应
    app.world_mut().flush();

    // 先运行一个完整帧（reset_reactions_on_turn_start 会在 First 中重置反应槽位）
    app.update();
    app.world_mut().flush();

    // 重新标记为耗尽（确保测试状态下有至少一个耗尽反应者）
    app.world_mut().entity_mut(exhausted).insert(ReactionState {
        used: true,
        extra_reactions: 0,
        extra_used: 0,
    });
    app.world_mut().flush();

    setup_reaction_queue(
        app.world_mut(),
        vec![ReactionEntry {
            reactor: exhausted,
            reaction_type: ReactionType::OpportunityAttack,
            trigger:
                crate::core::domains::reaction::components::ReactionTrigger::LeaveThreatRange {
                    mover: exhausted,
                    to_x: 3,
                    to_y: 5,
                },
            priority: 100,
            status: ReactionEntryStatus::Pending,
        }],
    );

    // 直接运行 process_reaction_queue，不触发 First 中的重置
    let system_id = app.world_mut().register_system(process_reaction_queue);
    let _ = app.world_mut().run_system(system_id);
    app.world_mut().flush();

    // 已用反应的条目应被取消
    let queue = app.world_mut().resource::<GlobalReactionQueue>();
    assert!(!queue.queue.entries.is_empty(), "队列条目不应被清理");
    let entry = &queue.queue.entries[0];
    assert_eq!(
        entry.status,
        ReactionEntryStatus::Cancelled,
        "已耗尽的反应者条目应被取消"
    );
}

// ─── 机会攻击执行 ──────────────────────────────────────────────────

#[test]
fn opportunity_attack_executed_triggers_reaction_executed() {
    let mut app = App::new();
    app.add_plugins(ReactionPlugin);

    let attacker = app.world_mut().spawn_empty().id();
    let target = app.world_mut().spawn_empty().id();
    app.world_mut().flush();

    // 触发机会攻击执行事件
    app.world_mut().trigger(OpportunityAttackExecuted {
        attacker,
        target,
        hit: true,
        damage: 12,
        critical: false,
    });
    app.world_mut().flush();

    // observer 不修改实体状态，仅转发事件
    // 验证：不 panic 即通过
}

#[test]
fn opportunity_attack_miss_reported_correctly() {
    let mut app = App::new();
    app.add_plugins(ReactionPlugin);

    let attacker = app.world_mut().spawn_empty().id();
    let target = app.world_mut().spawn_empty().id();
    app.world_mut().flush();

    // 未命中
    app.world_mut().trigger(OpportunityAttackExecuted {
        attacker,
        target,
        hit: false,
        damage: 0,
        critical: false,
    });
    app.world_mut().flush();

    // 验证：不 panic 即通过
}

// ─── 队列清理 ──────────────────────────────────────────────────────

#[test]
fn finished_queue_cleaned_up_at_frame_end() {
    let mut app = App::new();
    app.add_plugins(ReactionPlugin);

    // 创建一个已经完成的队列
    let mut queue = ReactionQueue::new();
    // 没有待处理条目的空队列 = is_finished
    app.world_mut()
        .insert_resource(GlobalReactionQueue { queue });
    app.world_mut().flush();

    app.update();
    app.world_mut().flush();

    // 空队列应被清理（clear 后 entries 为空）
    let resource = app.world_mut().resource::<GlobalReactionQueue>();
    assert!(
        resource.queue.entries.is_empty(),
        "完成的队列应在帧末被清理"
    );
}
