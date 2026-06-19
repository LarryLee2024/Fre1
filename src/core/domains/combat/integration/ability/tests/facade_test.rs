//! CombatAbilityFacade 测试
//!
//! 验证技能 facade 的容器创建、技能激活、完成冷却、冷却推进主线流程。

use bevy::prelude::Entity;

use crate::core::capabilities::ability::foundation::{AbilityError, CostEntry};
use crate::core::capabilities::ability::mechanism::ActiveAbilityContainer;
use crate::core::domains::combat::integration::ability::CombatAbilityFacade;

#[test]
fn empty_container_creates_no_instances_and_no_cooldowns() {
    let container = CombatAbilityFacade::empty_container();
    assert!(container.active_instances.is_empty());
    assert!(container.cooldowns.is_empty());
}

#[test]
fn try_activate_ability_succeeds_when_ready() {
    let mut container = CombatAbilityFacade::empty_container();
    let caster = Entity::from_raw_u32(1).unwrap();
    let target = Entity::from_raw_u32(2).unwrap();
    let costs = vec![CostEntry::new("mana", 10.0)];

    let result = CombatAbilityFacade::try_activate_ability(
        &mut container,
        "spec_fireball",
        "def_fireball",
        caster,
        target,
        100,
        costs,
    );

    assert!(
        result.is_ok(),
        "activation should succeed when no cooldown or active instance"
    );
    let instance_id = result.unwrap();
    assert!(container.get_instance(&instance_id).is_some());
}

#[test]
fn try_activate_ability_fails_when_already_active() {
    let mut container = CombatAbilityFacade::empty_container();
    let caster = Entity::from_raw_u32(1).unwrap();
    let target = Entity::from_raw_u32(2).unwrap();

    // First activation
    let first = CombatAbilityFacade::try_activate_ability(
        &mut container,
        "spec_fireball",
        "def_fireball",
        caster,
        target,
        100,
        vec![],
    );
    assert!(first.is_ok());

    // Second activation of same spec should fail
    let second = CombatAbilityFacade::try_activate_ability(
        &mut container,
        "spec_fireball",
        "def_fireball",
        caster,
        target,
        101,
        vec![],
    );

    assert!(
        matches!(second, Err(AbilityError::AlreadyActive { .. })),
        "should fail with AlreadyActive when same spec is active"
    );
}

#[test]
fn try_activate_ability_fails_when_on_cooldown() {
    let mut container = CombatAbilityFacade::empty_container();
    let caster = Entity::from_raw_u32(1).unwrap();
    let target = Entity::from_raw_u32(2).unwrap();

    // Activate, complete with cooldown
    let instance_id = CombatAbilityFacade::try_activate_ability(
        &mut container,
        "spec_fireball",
        "def_fireball",
        caster,
        target,
        100,
        vec![],
    )
    .unwrap();

    CombatAbilityFacade::complete_and_cooldown(&mut container, &instance_id, 3).unwrap();

    // Try to activate again while on cooldown
    let result = CombatAbilityFacade::try_activate_ability(
        &mut container,
        "spec_fireball",
        "def_fireball",
        caster,
        target,
        101,
        vec![],
    );

    assert!(
        matches!(result, Err(AbilityError::OnCooldown { .. })),
        "should fail with OnCooldown when spec is cooling down"
    );
}

#[test]
fn complete_and_cooldown_transitions_to_cooldown() {
    let mut container = CombatAbilityFacade::empty_container();
    let caster = Entity::from_raw_u32(1).unwrap();
    let target = Entity::from_raw_u32(2).unwrap();

    let instance_id = CombatAbilityFacade::try_activate_ability(
        &mut container,
        "spec_fireball",
        "def_fireball",
        caster,
        target,
        100,
        vec![],
    )
    .unwrap();

    let result = CombatAbilityFacade::complete_and_cooldown(&mut container, &instance_id, 2);
    assert!(result.is_ok());
    assert!(container.cooldowns.contains_key("spec_fireball"));
}

#[test]
fn complete_and_cooldown_fails_for_nonexistent_instance() {
    let mut container = CombatAbilityFacade::empty_container();
    let fake_id = crate::core::capabilities::ability::foundation::AbilityInstanceId::from(9999u64);

    let result = CombatAbilityFacade::complete_and_cooldown(&mut container, &fake_id, 2);
    assert!(
        matches!(result, Err(AbilityError::InstanceNotFound(_))),
        "should fail when instance does not exist"
    );
}

#[test]
fn tick_all_cooldowns_reduces_remaining_turns() {
    let mut container = CombatAbilityFacade::empty_container();
    let caster = Entity::from_raw_u32(1).unwrap();
    let target = Entity::from_raw_u32(2).unwrap();

    let instance_id = CombatAbilityFacade::try_activate_ability(
        &mut container,
        "spec_fireball",
        "def_fireball",
        caster,
        target,
        100,
        vec![],
    )
    .unwrap();

    CombatAbilityFacade::complete_and_cooldown(&mut container, &instance_id, 3).unwrap();
    assert_eq!(container.cooldowns["spec_fireball"].remaining_turns, 3);

    let expired = CombatAbilityFacade::tick_all_cooldowns(&mut container);
    assert!(
        expired.is_empty(),
        "cooldown should not expire after 1 tick"
    );
    assert_eq!(container.cooldowns["spec_fireball"].remaining_turns, 2);
}

#[test]
fn tick_all_cooldowns_returns_expired_spec_ids() {
    let mut container = CombatAbilityFacade::empty_container();
    let caster = Entity::from_raw_u32(1).unwrap();
    let target = Entity::from_raw_u32(2).unwrap();

    let instance_id = CombatAbilityFacade::try_activate_ability(
        &mut container,
        "spec_fireball",
        "def_fireball",
        caster,
        target,
        100,
        vec![],
    )
    .unwrap();

    CombatAbilityFacade::complete_and_cooldown(&mut container, &instance_id, 1).unwrap();

    let expired = CombatAbilityFacade::tick_all_cooldowns(&mut container);
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0], "spec_fireball");
}

#[test]
fn activate_multiple_abilities_independently() {
    let mut container = CombatAbilityFacade::empty_container();
    let caster = Entity::from_raw_u32(1).unwrap();
    let target = Entity::from_raw_u32(2).unwrap();

    let id1 = CombatAbilityFacade::try_activate_ability(
        &mut container,
        "spec_heal",
        "def_heal",
        caster,
        target,
        100,
        vec![],
    )
    .unwrap();

    let id2 = CombatAbilityFacade::try_activate_ability(
        &mut container,
        "spec_shield",
        "def_shield",
        caster,
        target,
        100,
        vec![],
    )
    .unwrap();

    assert_ne!(
        id1, id2,
        "different specs should produce different instance ids"
    );
    assert_eq!(container.active_instances.len(), 2);
}
