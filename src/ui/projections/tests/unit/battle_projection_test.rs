//! BattleProjection — Domain Event to ViewModel projection unit tests
//!
//! Tests verify that BattleProjection pure functions correctly transform
//! domain events (TurnStarted, EffectApplied) into UiStore updates:
//!
//! - on_turn_started: increments turn_number, sets phase_key
//! - on_effect_applied: placeholder no-op (does not panic)
//!
//! These are pure function tests — no ECS setup needed. The projection
//! functions take &mut UiStore and an event reference.

use bevy::prelude::Entity;

use crate::core::capabilities::effect::events::EffectApplied;
use crate::core::events::TurnStarted;
use crate::ui::projections::BattleProjection;
use crate::ui::view_models::UiStore;

// ── TurnStarted projection tests ─────────────────────────────────────

#[test]
fn on_turn_started_increments_turn_number() {
    let mut store = UiStore::default();
    let event = TurnStarted {
        unit: Entity::from_bits(1),
    };

    BattleProjection::on_turn_started(&mut store, &event);

    assert_eq!(
        store.battle_hud.turn_number, 1,
        "turn_number must increment from 0 to 1"
    );
}

#[test]
fn on_turn_started_sets_phase_key() {
    let mut store = UiStore::default();
    let event = TurnStarted {
        unit: Entity::from_bits(1),
    };

    BattleProjection::on_turn_started(&mut store, &event);

    assert_eq!(
        store.battle_hud.phase_key,
        "ui.battle.phase.player",
        "phase_key must be set to player phase key"
    );
}

#[test]
fn on_turn_started_increments_from_existing_value() {
    let mut store = UiStore::default();
    store.battle_hud.turn_number = 5;
    let event = TurnStarted {
        unit: Entity::from_bits(1),
    };

    BattleProjection::on_turn_started(&mut store, &event);

    assert_eq!(
        store.battle_hud.turn_number, 6,
        "turn_number must increment from 5 to 6"
    );
}

#[test]
fn on_turn_started_preserves_other_fields() {
    let mut store = UiStore::default();
    store.battle_hud.hp = 80.0;
    store.battle_hud.max_hp = 100.0;
    store.battle_hud.ap = 3.0;
    let event = TurnStarted {
        unit: Entity::from_bits(1),
    };

    BattleProjection::on_turn_started(&mut store, &event);

    assert_eq!(store.battle_hud.hp, 80.0, "hp must be preserved");
    assert_eq!(store.battle_hud.max_hp, 100.0, "max_hp must be preserved");
    assert_eq!(store.battle_hud.ap, 3.0, "ap must be preserved");
    assert_eq!(store.battle_hud.turn_number, 1, "turn_number must be 1");
}

#[test]
fn on_turn_started_multiple_calls_accumulate() {
    let mut store = UiStore::default();
    let event = TurnStarted {
        unit: Entity::from_bits(1),
    };

    BattleProjection::on_turn_started(&mut store, &event);
    BattleProjection::on_turn_started(&mut store, &event);
    BattleProjection::on_turn_started(&mut store, &event);

    assert_eq!(
        store.battle_hud.turn_number, 3,
        "multiple calls must accumulate turn_number"
    );
}

// ── EffectApplied projection tests (placeholder) ──────────────────────

#[test]
fn on_effect_applied_does_not_panic() {
    let mut store = UiStore::default();
    let event = EffectApplied {
        instance_id: "inst_001".into(),
        def_id: "def_fireball".into(),
        tags: vec!["damage".into(), "fire".into()],
        source_entity: "unit_warrior".into(),
        target_entity: "unit_goblin".into(),
        duration_type: "instant".into(),
    };

    // Should complete without panicking
    BattleProjection::on_effect_applied(&mut store, &event);
}

#[test]
fn on_effect_applied_does_not_modify_battle_hud() {
    let mut store = UiStore::default();
    store.battle_hud.turn_number = 42;
    store.battle_hud.phase_key = "ui.battle.phase.enemy";
    let event = EffectApplied {
        instance_id: "inst_002".into(),
        def_id: "def_heal".into(),
        tags: vec!["heal".into()],
        source_entity: "unit_cleric".into(),
        target_entity: "unit_warrior".into(),
        duration_type: "instant".into(),
    };

    BattleProjection::on_effect_applied(&mut store, &event);

    assert_eq!(
        store.battle_hud.turn_number, 42,
        "battle_hud must not be modified by on_effect_applied"
    );
    assert_eq!(
        store.battle_hud.phase_key,
        "ui.battle.phase.enemy",
        "battle_hud phase_key must not be modified"
    );
}

#[test]
fn on_effect_applied_does_not_modify_skill_panel() {
    let mut store = UiStore::default();
    let event = EffectApplied {
        instance_id: "inst_003".into(),
        def_id: "def_buff".into(),
        tags: vec!["buff".into()],
        source_entity: "unit_mage".into(),
        target_entity: "unit_tank".into(),
        duration_type: "timed".into(),
    };

    BattleProjection::on_effect_applied(&mut store, &event);

    assert!(
        store.skill_panel.skills.is_empty(),
        "skill_panel must not be modified by placeholder on_effect_applied"
    );
}
