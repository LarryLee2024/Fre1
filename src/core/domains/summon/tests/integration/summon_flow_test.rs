//! Summon Domain — 召唤流程集成测试
//!
//! 验证 SummonPlugin 注册后的 Observer 链路：
//! - 召唤创建：on_summon_created → SummonSlotManager 新增
//! - 召唤消失：on_summon_expired → SummonSlotManager 移除

use bevy::prelude::*;

use crate::core::domains::summon::components::SummonSlotManager;
use crate::core::domains::summon::events::{SummonCreated, SummonExpireReason, SummonExpired};
use crate::core::domains::summon::plugin::SummonPlugin;

// ─── 辅助函数 ──────────────────────────────────────────────────────

fn spawn_summoner(world: &mut World, max_slots: u32) -> Entity {
    world.spawn(SummonSlotManager::new(max_slots)).id()
}

fn active_summon_count(world: &World, entity: Entity) -> u32 {
    world
        .get::<SummonSlotManager>(entity)
        .map(|m| m.active_summons.len() as u32)
        .unwrap_or(0)
}

// ─── 召唤创建 ──────────────────────────────────────────────────────

#[test]
fn create_summon_adds_to_manager() {
    let mut app = App::new();
    app.add_plugins(SummonPlugin);

    let caster = spawn_summoner(app.world_mut(), 3);
    let summon = app.world_mut().spawn_empty().id();
    app.world_mut().flush();

    app.world_mut().trigger(SummonCreated {
        caster,
        summon_entity: summon,
        template_id: "wolf".into(),
        position: (3, 5),
        duration_type: "temporary".into(),
    });
    app.world_mut().flush();

    assert_eq!(
        active_summon_count(app.world_mut(), caster),
        1,
        "召唤后槽位管理器应有一个活跃召唤物"
    );

    let manager = app.world_mut().get::<SummonSlotManager>(caster).unwrap();
    assert!(
        manager.active_summons.contains(&summon),
        "召唤物实体应在活跃列表中"
    );
}

#[test]
fn multiple_summons_added_sequentially() {
    let mut app = App::new();
    app.add_plugins(SummonPlugin);

    let caster = spawn_summoner(app.world_mut(), 3);
    app.world_mut().flush();

    for i in 0..3 {
        let summon = app.world_mut().spawn_empty().id();
        app.world_mut().flush();

        app.world_mut().trigger(SummonCreated {
            caster,
            summon_entity: summon,
            template_id: format!("wolf_{}", i),
            position: (0, 0),
            duration_type: "temporary".into(),
        });
    }
    app.world_mut().flush();

    assert_eq!(
        active_summon_count(app.world_mut(), caster),
        3,
        "应能创建最多 3 个召唤物"
    );
}

#[test]
fn create_summon_when_full_is_rejected() {
    let mut app = App::new();
    app.add_plugins(SummonPlugin);

    let caster = spawn_summoner(app.world_mut(), 1); // 只有 1 个槽位
    app.world_mut().flush();

    // 填满
    let summon1 = app.world_mut().spawn_empty().id();
    app.world_mut().flush();
    app.world_mut().trigger(SummonCreated {
        caster,
        summon_entity: summon1,
        template_id: "wolf".into(),
        position: (0, 0),
        duration_type: "temporary".into(),
    });
    app.world_mut().flush();

    assert_eq!(active_summon_count(app.world_mut(), caster), 1);

    // 尝试再召唤一个
    let summon2 = app.world_mut().spawn_empty().id();
    app.world_mut().flush();
    app.world_mut().trigger(SummonCreated {
        caster,
        summon_entity: summon2,
        template_id: "bear".into(),
        position: (1, 1),
        duration_type: "temporary".into(),
    });
    app.world_mut().flush();

    assert_eq!(
        active_summon_count(app.world_mut(), caster),
        1,
        "槽位满时应拒绝新召唤物"
    );
}

// ─── 召唤消失 ──────────────────────────────────────────────────────

#[test]
fn expire_summon_removes_from_manager() {
    let mut app = App::new();
    app.add_plugins(SummonPlugin);

    let caster = spawn_summoner(app.world_mut(), 3);
    let summon = app.world_mut().spawn_empty().id();
    app.world_mut().flush();

    // 先创建
    app.world_mut().trigger(SummonCreated {
        caster,
        summon_entity: summon,
        template_id: "wolf".into(),
        position: (0, 0),
        duration_type: "temporary".into(),
    });
    app.world_mut().flush();
    assert_eq!(active_summon_count(app.world_mut(), caster), 1);

    // 再消失
    app.world_mut().trigger(SummonExpired {
        caster,
        summon_entity: summon,
        reason: SummonExpireReason::DurationExpired,
    });
    app.world_mut().flush();

    assert_eq!(
        active_summon_count(app.world_mut(), caster),
        0,
        "召唤物消失后应从管理器中移除"
    );

    let manager = app.world_mut().get::<SummonSlotManager>(caster).unwrap();
    assert!(
        !manager.active_summons.contains(&summon),
        "消失的召唤物不应在活跃列表中"
    );
}

#[test]
fn expire_only_removes_target_summon() {
    let mut app = App::new();
    app.add_plugins(SummonPlugin);

    let caster = spawn_summoner(app.world_mut(), 5);
    app.world_mut().flush();

    let summons: Vec<Entity> = (0..3)
        .map(|i| {
            let s = app.world_mut().spawn_empty().id();
            app.world_mut().flush();
            app.world_mut().trigger(SummonCreated {
                caster,
                summon_entity: s,
                template_id: format!("creature_{}", i),
                position: (0, 0),
                duration_type: "temporary".into(),
            });
            s
        })
        .collect();
    app.world_mut().flush();

    assert_eq!(active_summon_count(app.world_mut(), caster), 3);

    // 消失第二个召唤物
    app.world_mut().trigger(SummonExpired {
        caster,
        summon_entity: summons[1],
        reason: SummonExpireReason::DurationExpired,
    });
    app.world_mut().flush();

    assert_eq!(
        active_summon_count(app.world_mut(), caster),
        2,
        "消失一个召唤物后应剩余 2 个"
    );

    let manager = app.world_mut().get::<SummonSlotManager>(caster).unwrap();
    assert!(manager.active_summons.contains(&summons[0]), "第一个应保留");
    assert!(
        !manager.active_summons.contains(&summons[1]),
        "第二个应移除"
    );
    assert!(manager.active_summons.contains(&summons[2]), "第三个应保留");
}

#[test]
fn expire_nonexistent_summon_does_nothing() {
    let mut app = App::new();
    app.add_plugins(SummonPlugin);

    let caster = spawn_summoner(app.world_mut(), 3);
    let summon = app.world_mut().spawn_empty().id();
    let ghost = app.world_mut().spawn_empty().id();
    app.world_mut().flush();

    // 创建一个召唤物
    app.world_mut().trigger(SummonCreated {
        caster,
        summon_entity: summon,
        template_id: "wolf".into(),
        position: (0, 0),
        duration_type: "temporary".into(),
    });
    app.world_mut().flush();

    // 尝试消失一个不存在的召唤物
    app.world_mut().trigger(SummonExpired {
        caster,
        summon_entity: ghost,
        reason: SummonExpireReason::DurationExpired,
    });
    app.world_mut().flush();

    // 不 panic，且活跃列表不变
    assert_eq!(
        active_summon_count(app.world_mut(), caster),
        1,
        "消失不存在的召唤物不应影响活跃列表"
    );
}

// ─── 槽位状态 ──────────────────────────────────────────────────────

#[test]
fn slot_changed_event_reflects_usage() {
    let mut app = App::new();
    app.add_plugins(SummonPlugin);

    let caster = spawn_summoner(app.world_mut(), 2);
    app.world_mut().flush();

    let s1 = app.world_mut().spawn_empty().id();
    app.world_mut().flush();
    app.world_mut().trigger(SummonCreated {
        caster,
        summon_entity: s1,
        template_id: "wolf".into(),
        position: (0, 0),
        duration_type: "temporary".into(),
    });
    app.world_mut().flush();

    let manager = app.world_mut().get::<SummonSlotManager>(caster).unwrap();
    assert_eq!(manager.active_summons.len() as u32, 1, "使用 1 个槽位");
    assert_eq!(manager.max_slots, 2, "总槽位 2");
}
