use bevy::prelude::{Commands, Entity, World};

use crate::core::capabilities::gameplay_context::foundation::error::ContextBuildError;
use crate::core::capabilities::gameplay_context::foundation::{
    ContextOrigin, ElementType, SourceInfo, TargetInfo,
};
use crate::core::capabilities::gameplay_context::mechanism::ContextBuilder;

fn test_entity(index: u32) -> Entity {
    Entity::from_bits((index as u64) << 32 | 0x10000)
}

fn valid_source() -> SourceInfo {
    SourceInfo {
        entity: test_entity(1),
        faction: "fct_000001".to_string(),
        position: Some((0, 0)),
    }
}

fn valid_target() -> TargetInfo {
    TargetInfo {
        entity: test_entity(2),
        faction: "fct_000002".to_string(),
        position: Some((5, 5)),
        is_valid: true,
    }
}

#[test]
fn debug_entity_from_bits() {
    let from_bits_1 = Entity::from_bits(1);
    let placeholder = Entity::PLACEHOLDER;
    println!(
        "from_bits(1) = {:?} (bits=0x{:x})",
        from_bits_1,
        from_bits_1.to_bits()
    );
    println!(
        "PLACEHOLDER  = {:?} (bits=0x{:x})",
        placeholder,
        placeholder.to_bits()
    );
    println!("equal: {}", from_bits_1 == placeholder);
}

#[test]
fn builder_with_source_and_target_succeeds() {
    let mut world = World::new();
    let mut commands = world.commands();
    let ctx = ContextBuilder::new(ContextOrigin::Direct, 1)
        .source(valid_source())
        .target(valid_target())
        .build(&mut commands)
        .expect("build should succeed");
    assert_eq!(ctx.origin, ContextOrigin::Direct);
    assert!(ctx.context_id.starts_with("ctx_"));
    assert_eq!(ctx.created_at_frame, 1);
}

#[test]
fn builder_missing_source_fails() {
    let mut world = World::new();
    let mut commands = world.commands();
    let ctx = ContextBuilder::new(ContextOrigin::Direct, 1)
        .target(valid_target())
        .build(&mut commands);
    assert!(
        matches!(ctx, Err(ContextBuildError::MissingFields(fields)) if fields.contains(&"source".to_string()))
    );
}

#[test]
fn builder_missing_target_fails() {
    let mut world = World::new();
    let mut commands = world.commands();
    let ctx = ContextBuilder::new(ContextOrigin::Direct, 1)
        .source(valid_source())
        .build(&mut commands);
    assert!(
        matches!(ctx, Err(ContextBuildError::MissingFields(fields)) if fields.contains(&"target".to_string()))
    );
}

#[test]
fn builder_all_optional_fields_succeeds() {
    let mut world = World::new();
    let mut commands = world.commands();
    let ctx = ContextBuilder::new(ContextOrigin::Triggered, 42)
        .source(valid_source())
        .target(valid_target())
        .ability("abl_000001")
        .equipment("equip_000001")
        .element(ElementType::Fire)
        .critical()
        .build(&mut commands)
        .unwrap();
    assert_eq!(ctx.origin, ContextOrigin::Triggered);
    assert_eq!(ctx.ability_id, Some("abl_000001".to_string()));
    assert_eq!(ctx.equipment_id, Some("equip_000001".to_string()));
    assert_eq!(ctx.element_type, Some(ElementType::Fire));
    assert!(ctx.is_critical);
    assert_eq!(ctx.created_at_frame, 42);
    assert_eq!(ctx.chain.len(), 1);
}

#[test]
fn chain_first_node_at_head() {
    let mut world = World::new();
    let mut commands = world.commands();
    let ctx = ContextBuilder::new(ContextOrigin::Direct, 10)
        .source(valid_source())
        .target(valid_target())
        .build(&mut commands)
        .unwrap();
    let last = ctx.chain.last().unwrap();
    assert_eq!(last.frame, 10);
    assert_eq!(last.source.entity, test_entity(1));
}

#[test]
fn context_id_unique_per_build() {
    let mut world = World::new();
    let mut commands = world.commands();
    let ctx1 = ContextBuilder::new(ContextOrigin::Direct, 1)
        .source(valid_source())
        .target(valid_target())
        .build(&mut commands)
        .unwrap();
    let ctx2 = ContextBuilder::new(ContextOrigin::Direct, 1)
        .source(valid_source())
        .target(valid_target())
        .build(&mut commands)
        .unwrap();
    assert_ne!(ctx1.context_id, ctx2.context_id);
}
