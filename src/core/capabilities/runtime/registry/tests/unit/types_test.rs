use crate::core::capabilities::runtime::registry::foundation::{
    AllocatorState, IdAllocator, IdType, RegistryEntry, RegistryError,
};

#[test]
fn id_type_prefix_correct() {
    assert_eq!(IdType::Ability.prefix(), "abl_");
    assert_eq!(IdType::Effect.prefix(), "eff_");
    assert_eq!(IdType::Cue.prefix(), "cue_");
}

#[test]
fn parse_id_type_from_prefix() {
    assert_eq!(IdType::from_prefix("abl_"), Some(IdType::Ability));
    assert_eq!(IdType::from_prefix("eff_"), Some(IdType::Effect));
    assert_eq!(IdType::from_prefix("xxx_"), None);
}

#[test]
fn id_type_name_correct() {
    assert_eq!(IdType::Ability.name(), "Ability");
    assert_eq!(IdType::Custom("Test".into()).name(), "Test");
}

#[test]
fn allocator_assigns_id() {
    let mut state = AllocatorState::new("abl_", 6);
    assert_eq!(state.allocate(), "abl_000001");
    assert_eq!(state.allocate(), "abl_000002");
    assert_eq!(state.next_id, 3);
}

#[test]
fn allocator_different_digit_width() {
    let mut state = AllocatorState::new("eff_", 4);
    assert_eq!(state.allocate(), "eff_0001");
}

#[test]
fn id_allocator_multi_type() {
    let mut alloc = IdAllocator::new();
    alloc.register_type(IdType::Ability, AllocatorState::new("abl_", 6));
    alloc.register_type(IdType::Effect, AllocatorState::new("eff_", 6));

    assert_eq!(alloc.allocate(&IdType::Ability), Some("abl_000001".into()));
    assert_eq!(alloc.allocate(&IdType::Effect), Some("eff_000001".into()));
    assert_eq!(alloc.allocate(&IdType::Ability), Some("abl_000002".into()));
}

#[test]
fn unregistered_type_assignment_returns_none() {
    let mut alloc = IdAllocator::new();
    assert_eq!(alloc.allocate(&IdType::Ability), None);
}

#[test]
fn registry_entry_creation() {
    let entry = RegistryEntry::new("abl_000001", "Ability", "name=Fireball,damage=50");
    assert_eq!(entry.def_id, "abl_000001");
    assert_eq!(entry.def_type, "Ability");
    assert!(!entry.deprecated);
}

#[test]
fn registry_entry_mark_deprecated() {
    let entry = RegistryEntry::new("abl_000001", "Ability", "")
        .deprecated()
        .superseded_by("abl_000042");
    assert!(entry.deprecated);
    assert_eq!(entry.superseded_by, Some("abl_000042".into()));
}

#[test]
fn error_message_display() {
    let err = RegistryError::DuplicateId("abl_000001".into());
    let msg = format!("{}", err);
    assert!(msg.contains("abl_000001"));
}

#[test]
fn broken_reference_error() {
    let err = RegistryError::BrokenReference {
        source_id: "abl_000001".into(),
        field: "effect_id".into(),
        target: "eff_999999".into(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("eff_999999"));
}
