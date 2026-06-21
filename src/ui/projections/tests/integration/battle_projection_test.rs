//! 战斗投影集成测试 — 领域事件到 ViewModel 的 Observer 接线验证
//!
//! 测试验证各投影 Observer 正确地：
//! - 将领域事件投影到 UiStore
//! - 标记对应的 Dirty<T> 组件为脏
//!
//! 这些是 ECS 集成测试。

use bevy::prelude::*;

use crate::core::capabilities::effect::events::EffectApplied;
use crate::core::events::{BattleStarted, TurnEnded, TurnStarted};
use crate::ui::binding::Dirty;
use crate::ui::projections::battle::{
    on_battle_started_projection, on_effect_applied_projection, on_turn_ended_projection,
    on_turn_started_projection,
};
use crate::ui::view_models::UiStore;
use crate::ui::view_models::battle_hud::BattleHudVm;
use crate::ui::view_models::skill_panel::SkillPanelVm;

// ── 辅助函数 ─────────────────────────────────────────────────────────────

/// 构建注册了投影 Observer 的最小 App，包含 UiStore 和 Dirty 组件。
fn projection_app() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<UiStore>();

    // 注册 Observer
    app.add_observer(on_battle_started_projection);
    app.add_observer(on_turn_started_projection);
    app.add_observer(on_turn_ended_projection);
    app.add_observer(on_effect_applied_projection);

    // 生成携带 Dirty<BattleHudVm> 的实体用于断言
    let battle_hud_entity = app.world_mut().spawn(Dirty::<BattleHudVm>::default()).id();

    // 生成携带 Dirty<SkillPanelVm> 的实体用于断言
    let _skill_panel_entity = app.world_mut().spawn(Dirty::<SkillPanelVm>::default()).id();

    (app, battle_hud_entity)
}

/// 检查指定实体上的 Dirty<T> 组件是否被标记为脏并消费标记。
fn is_dirty<T: Reflect + Default + Clone + Send + Sync + 'static>(
    app: &mut App,
    entity: Entity,
) -> bool {
    app.world_mut()
        .entity_mut(entity)
        .get_mut::<Dirty<T>>()
        .unwrap()
        .consume()
}

// ── BattleStarted 投影 ──────────────────────────────────────────────────

#[test]
fn on_battle_started_projection_sets_turn_number_to_one() {
    let (mut app, _) = projection_app();

    app.world_mut().trigger(BattleStarted);

    let store = app.world().resource::<UiStore>();
    assert_eq!(
        store.battle_hud.turn_number, 1,
        "BattleStarted must set turn_number to 1"
    );
}

#[test]
fn on_battle_started_projection_sets_phase_key() {
    let (mut app, _) = projection_app();

    app.world_mut().trigger(BattleStarted);

    let store = app.world().resource::<UiStore>();
    assert_eq!(
        store.battle_hud.phase_key, "ui.battle.phase.player",
        "BattleStarted must set phase_key to player phase"
    );
}

#[test]
fn on_battle_started_projection_marks_battle_hud_dirty() {
    let (mut app, battle_hud_entity) = projection_app();

    // Consume the initial dirty from Default to get a clean slate
    is_dirty::<BattleHudVm>(&mut app, battle_hud_entity);

    app.world_mut().trigger(BattleStarted);

    assert!(
        is_dirty::<BattleHudVm>(&mut app, battle_hud_entity),
        "BattleStarted must mark Dirty<BattleHudVm> as dirty"
    );
}

// ── TurnStarted 投影 ────────────────────────────────────────────────────

#[test]
fn on_turn_started_projection_increments_turn_number() {
    let (mut app, _) = projection_app();

    // Pre-set turn_number to verify increment
    app.world_mut()
        .resource_mut::<UiStore>()
        .battle_hud
        .turn_number = 5;
    app.world_mut().trigger(TurnStarted {
        unit: Entity::from_bits(1),
    });

    let store = app.world().resource::<UiStore>();
    assert_eq!(
        store.battle_hud.turn_number, 6,
        "TurnStarted must increment turn_number from 5 to 6"
    );
}

#[test]
fn on_turn_started_projection_sets_phase_key() {
    let (mut app, _) = projection_app();

    app.world_mut().trigger(TurnStarted {
        unit: Entity::from_bits(1),
    });

    let store = app.world().resource::<UiStore>();
    assert_eq!(
        store.battle_hud.phase_key, "ui.battle.phase.player",
        "TurnStarted must set phase_key to player phase"
    );
}

#[test]
fn on_turn_started_projection_marks_battle_hud_dirty() {
    let (mut app, battle_hud_entity) = projection_app();

    is_dirty::<BattleHudVm>(&mut app, battle_hud_entity);

    app.world_mut().trigger(TurnStarted {
        unit: Entity::from_bits(1),
    });

    assert!(
        is_dirty::<BattleHudVm>(&mut app, battle_hud_entity),
        "TurnStarted must mark Dirty<BattleHudVm> as dirty"
    );
}

// ── TurnEnded 投影 ──────────────────────────────────────────────────────

#[test]
fn on_turn_ended_projection_sets_phase_key_to_enemy() {
    let (mut app, _) = projection_app();

    app.world_mut().trigger(TurnEnded {
        unit: Entity::from_bits(1),
    });

    let store = app.world().resource::<UiStore>();
    assert_eq!(
        store.battle_hud.phase_key, "ui.battle.phase.enemy",
        "TurnEnded must set phase_key to enemy phase"
    );
}

#[test]
fn on_turn_ended_projection_preserves_turn_number() {
    let (mut app, _) = projection_app();

    app.world_mut()
        .resource_mut::<UiStore>()
        .battle_hud
        .turn_number = 3;
    app.world_mut().trigger(TurnEnded {
        unit: Entity::from_bits(1),
    });

    let store = app.world().resource::<UiStore>();
    assert_eq!(
        store.battle_hud.turn_number, 3,
        "TurnEnded must not modify turn_number"
    );
}

#[test]
fn on_turn_ended_projection_marks_battle_hud_dirty() {
    let (mut app, battle_hud_entity) = projection_app();

    is_dirty::<BattleHudVm>(&mut app, battle_hud_entity);

    app.world_mut().trigger(TurnEnded {
        unit: Entity::from_bits(1),
    });

    assert!(
        is_dirty::<BattleHudVm>(&mut app, battle_hud_entity),
        "TurnEnded must mark Dirty<BattleHudVm> as dirty"
    );
}

// ── EffectApplied 投影 ──────────────────────────────────────────────────

#[test]
fn on_effect_applied_projection_does_not_panic() {
    let (mut app, _) = projection_app();

    // Should complete without panicking
    app.world_mut().trigger(EffectApplied {
        instance_id: "inst_001".into(),
        def_id: "def_fireball".into(),
        tags: vec!["damage".into(), "fire".into()],
        source_entity: "unit_warrior".into(),
        target_entity: "unit_goblin".into(),
        duration_type: "instant".into(),
    });
}

#[test]
fn on_effect_applied_projection_marks_skill_panel_dirty() {
    let (mut app, _) = projection_app();

    // Find the SkillPanelVm entity we spawned
    let skill_panel_entity = app.world_mut().spawn(Dirty::<SkillPanelVm>::default()).id();

    is_dirty::<SkillPanelVm>(&mut app, skill_panel_entity);

    app.world_mut().trigger(EffectApplied {
        instance_id: "inst_002".into(),
        def_id: "def_buff".into(),
        tags: vec!["buff".into()],
        source_entity: "unit_mage".into(),
        target_entity: "unit_tank".into(),
        duration_type: "timed".into(),
    });

    assert!(
        is_dirty::<SkillPanelVm>(&mut app, skill_panel_entity),
        "EffectApplied must mark Dirty<SkillPanelVm> as dirty"
    );
}
