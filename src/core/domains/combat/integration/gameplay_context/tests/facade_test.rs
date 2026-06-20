//! CombatContextFacade 测试

use bevy::prelude::*;

use crate::core::capabilities::gameplay_context::foundation::ContextOrigin;
use crate::core::domains::combat::integration::gameplay_context::CombatContextFacade;

#[test]
fn build_attack_context_succeeds() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut counter = 1u64;
    let result = CombatContextFacade::build_attack_context(
        Entity::from_raw_u32(1).unwrap(),
        "faction_a",
        Some((5, 3)),
        Entity::from_raw_u32(2).unwrap(),
        "faction_b",
        Some((6, 3)),
        Some("fireball"),
        42,
        &mut commands,
        &mut counter,
    );
    assert!(result.is_ok());
    let ctx = result.unwrap();
    assert_eq!(ctx.source.entity, Entity::from_raw_u32(1).unwrap());
    assert_eq!(ctx.target.entity, Entity::from_raw_u32(2).unwrap());
    assert_eq!(ctx.ability_id, Some("fireball".to_string()));
    assert_eq!(ctx.created_at_frame, 42);
}

#[test]
fn build_reaction_context_creates_chain_reaction() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut counter = 1u64;
    let result = CombatContextFacade::build_reaction_context(
        Entity::from_raw_u32(2).unwrap(),
        "faction_b",
        Entity::from_raw_u32(1).unwrap(),
        "faction_a",
        ContextOrigin::ChainReaction,
        43,
        &mut commands,
        &mut counter,
    );
    assert!(result.is_ok());
    let ctx = result.unwrap();
    assert_eq!(ctx.origin, ContextOrigin::ChainReaction);
}

#[test]
fn build_periodic_context_has_periodic_origin() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut counter = 1u64;
    let result = CombatContextFacade::build_periodic_context(
        Entity::from_raw_u32(1).unwrap(),
        Entity::from_raw_u32(2).unwrap(),
        Some("poison"),
        100,
        &mut commands,
        &mut counter,
    );
    assert!(result.is_ok());
    let ctx = result.unwrap();
    assert_eq!(ctx.origin, ContextOrigin::Periodic);
    assert_eq!(ctx.ability_id, Some("poison".to_string()));
}
