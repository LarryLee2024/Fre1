//! 战斗投影 — 领域事件到 ViewModel 投影单元测试
//!
//! 测试验证 BattleProjection 纯函数正确地将
//! 领域事件（TurnStarted、EffectApplied）转换为 UiStore 更新：
//!
//! - on_turn_started：递增 turn_number，设置 phase_key
//! - on_effect_applied：匹配 effect def_id 到技能并更新冷却状态
//! - on_turn_started_for_skills：递减技能冷却
//!
//! 这些是纯函数测试 — 不需要 ECS 设置。投影函数
//! 接收 &mut UiStore 和事件引用。

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
        store.battle_hud.phase_key, "ui.battle.phase.player",
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

// ── EffectApplied projection tests ──────────────────────────────────────

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
        store.battle_hud.phase_key, "ui.battle.phase.enemy",
        "battle_hud phase_key must not be modified"
    );
}

#[test]
fn on_effect_applied_does_not_modify_unmatched_skill_cooldowns() {
    let mut store = UiStore::default();
    // Record initial skill state before projection
    let initial_cooldowns: Vec<(u32, u32, bool)> = store
        .skill_panel
        .skills
        .iter()
        .map(|(&id, s)| (id, s.cooldown_remaining, s.is_usable))
        .collect();

    let event = EffectApplied {
        instance_id: "inst_003".into(),
        def_id: "def_buff".into(),
        tags: vec!["buff".into()],
        source_entity: "unit_mage".into(),
        target_entity: "unit_tank".into(),
        duration_type: "timed".into(),
    };

    BattleProjection::on_effect_applied(&mut store, &event);

    // Skills must still exist (default has 3 sample skills)
    assert_eq!(
        store.skill_panel.skills.len(),
        3,
        "default skill count must be preserved"
    );

    // Cooldowns must not change for non-matching effect def_ids
    for (_id, slot) in store.skill_panel.skills.iter() {
        assert_eq!(
            slot.cooldown_remaining, 0,
            "unmatched effect must not modify skill cooldowns"
        );
        assert!(slot.is_usable, "unmatched effect must not disable skills");
    }
}

#[test]
fn on_effect_applied_with_matching_effect_sets_cooldown() {
    let mut store = UiStore::default();

    // "ui.skill.fireball" contains "fireball" → should match
    let event = EffectApplied {
        instance_id: "inst_004".into(),
        def_id: "fireball".into(),
        tags: vec!["damage".into(), "fire".into()],
        source_entity: "unit_mage".into(),
        target_entity: "unit_goblin".into(),
        duration_type: "instant".into(),
    };

    BattleProjection::on_effect_applied(&mut store, &event);

    // Fireball skill (id=2) should have cooldown_remaining = max_cooldown = 3
    let fireball = store.skill_panel.skills.get(&2).unwrap();
    assert_eq!(
        fireball.cooldown_remaining, 3,
        "matching effect must set cooldown_remaining to max_cooldown"
    );
    assert!(
        !fireball.is_usable,
        "matching effect must mark skill as unusable"
    );
}

#[test]
fn on_effect_applied_with_matching_effect_does_not_affect_unmatched_skills() {
    let mut store = UiStore::default();

    let event = EffectApplied {
        instance_id: "inst_005".into(),
        def_id: "fireball".into(),
        tags: vec!["damage".into(), "fire".into()],
        source_entity: "unit_mage".into(),
        target_entity: "unit_goblin".into(),
        duration_type: "instant".into(),
    };

    BattleProjection::on_effect_applied(&mut store, &event);

    // Attack (id=1) should be unchanged (max_cooldown=0 → no cooldown)
    let attack = store.skill_panel.skills.get(&1).unwrap();
    assert_eq!(
        attack.cooldown_remaining, 0,
        "attack cooldown must remain 0"
    );
    assert!(attack.is_usable, "attack must remain usable");

    // Heal (id=3, max_cooldown=2) should be unchanged
    let heal = store.skill_panel.skills.get(&3).unwrap();
    assert_eq!(heal.cooldown_remaining, 0, "heal cooldown must remain 0");
    assert!(heal.is_usable, "heal must remain usable");

    // Fireball (id=2, max_cooldown=3) should be on cooldown
    let fireball = store.skill_panel.skills.get(&2).unwrap();
    assert_eq!(
        fireball.cooldown_remaining, 3,
        "fireball must be on cooldown"
    );
    assert!(!fireball.is_usable, "fireball must be unusable");
}

/// TurnStarted 投影到技能冷却的测试
#[test]
fn on_turn_started_for_skills_ticks_down_cooldowns() {
    let mut store = UiStore::default();

    // Manually set fireball on cooldown
    let fireball = store.skill_panel.skills.get_mut(&2).unwrap();
    fireball.cooldown_remaining = 3;
    fireball.is_usable = false;

    let event = TurnStarted {
        unit: Entity::from_bits(1),
    };

    BattleProjection::on_turn_started_for_skills(&mut store, &event);

    // Fireball cooldown should be 2, still not usable
    let fireball = store.skill_panel.skills.get(&2).unwrap();
    assert_eq!(
        fireball.cooldown_remaining, 2,
        "cooldown must tick down from 3 to 2"
    );
    assert!(
        !fireball.is_usable,
        "skill at cooldown 2 must not be usable"
    );
}

#[test]
fn on_turn_started_for_skills_marks_skill_usable_when_cooldown_reaches_zero() {
    let mut store = UiStore::default();

    // Manually set fireball at cooldown 1
    let fireball = store.skill_panel.skills.get_mut(&2).unwrap();
    fireball.cooldown_remaining = 1;
    fireball.is_usable = false;

    let event = TurnStarted {
        unit: Entity::from_bits(1),
    };

    BattleProjection::on_turn_started_for_skills(&mut store, &event);

    let fireball = store.skill_panel.skills.get(&2).unwrap();
    assert_eq!(
        fireball.cooldown_remaining, 0,
        "cooldown must tick down to 0"
    );
    assert!(
        fireball.is_usable,
        "skill must become usable when cooldown reaches 0"
    );
}

#[test]
fn on_turn_started_for_skills_does_not_affect_zero_cooldown_skills() {
    let mut store = UiStore::default();

    let event = TurnStarted {
        unit: Entity::from_bits(1),
    };

    BattleProjection::on_turn_started_for_skills(&mut store, &event);

    // All skills start at cooldown 0, should remain at 0
    for (_id, slot) in store.skill_panel.skills.iter() {
        assert_eq!(
            slot.cooldown_remaining, 0,
            "zero-cooldown skill must remain 0"
        );
        assert!(slot.is_usable, "zero-cooldown skill must remain usable");
    }
}
