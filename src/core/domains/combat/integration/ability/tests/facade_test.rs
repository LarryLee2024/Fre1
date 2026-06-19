//! CombatAbilityFacade 测试
//!
//! 验证技能 facade 的容器创建、技能激活、完成冷却、冷却推进主线流程。

use bevy::prelude::*;

use crate::core::capabilities::ability::foundation::{AbilityError, CostEntry};
use crate::core::domains::combat::integration::ability::CombatAbilityFacade;

#[test]
fn empty_container_creates_no_instances_and_no_cooldowns() {
    let container = CombatAbilityFacade::empty_container();
    assert!(container.active_instances.is_empty());
    assert!(container.cooldowns.is_empty());
}

#[test]
fn try_activate_ability_succeeds_when_ready() {
    let mut world = World::new();
    let mut commands = world.commands();
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
        &mut commands,
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
    let mut world = World::new();
    let mut commands = world.commands();
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
        &mut commands,
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
        &mut commands,
    );
    assert!(second.is_err());
    assert!(matches!(
        second,
        Err(AbilityError::AlreadyActive { .. })
    ));
}

#[test]
fn try_activate_ability_fails_when_on_cooldown() {
    let mut world = World::new();
    let mut commands = world.commands();
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
        &mut commands,
    );
    assert!(first.is_ok());

    // Complete and start cooldown (3 turns)
    let instance_id = first.unwrap();
    CombatAbilityFacade::complete_and_cooldown(
        &mut container,
        &instance_id,
        3,
        caster,
        &mut commands,
    )
    .unwrap();
    assert!(container.is_on_cooldown("spec_fireball"));

    // Activation while on cooldown
    let second = CombatAbilityFacade::try_activate_ability(
        &mut container,
        "spec_fireball",
        "def_fireball",
        caster,
        target,
        200,
        vec![],
        &mut commands,
    );
    assert!(second.is_err());
    assert!(matches!(second, Err(AbilityError::OnCooldown { .. })));
}

#[test]
fn cooldown_expires_after_sufficient_ticks() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut container = CombatAbilityFacade::empty_container();
    let caster = Entity::from_raw_u32(1).unwrap();
    let target = Entity::from_raw_u32(2).unwrap();

    let first = CombatAbilityFacade::try_activate_ability(
        &mut container,
        "spec_fireball",
        "def_fireball",
        caster,
        target,
        100,
        vec![],
        &mut commands,
    )
    .unwrap();
    CombatAbilityFacade::complete_and_cooldown(&mut container, &first, 2, caster, &mut commands)
        .unwrap();

    assert!(container.is_on_cooldown("spec_fireball"));

    let expired = CombatAbilityFacade::tick_all_cooldowns(&mut container);
    assert!(expired.is_empty());
    assert!(container.is_on_cooldown("spec_fireball"));

    let expired = CombatAbilityFacade::tick_all_cooldowns(&mut container);
    assert_eq!(expired, vec!["spec_fireball"]);
    assert!(!container.is_on_cooldown("spec_fireball"));
}
