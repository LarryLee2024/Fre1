use crate::infra::registry::resolver::{AllocatorState, IdAllocator, IdType, ValidationRunner};
use crate::infra::registry::{DefinitionId, DefinitionRegistry, RegistryEntry};

#[test]
fn test_id_type_prefix_roundtrip() {
    let types = vec![
        IdType::Ability,
        IdType::Effect,
        IdType::Trigger,
        IdType::Tag,
        IdType::Attribute,
        IdType::Cue,
        IdType::Terrain,
        IdType::Buff,
    ];
    for t in &types {
        let prefix = t.prefix();
        let parsed = IdType::from_prefix(prefix);
        assert_eq!(parsed.as_ref(), Some(t));
    }
}

#[test]
fn test_allocator_sequential_ids() {
    let mut state = AllocatorState::new("abl_", 6);
    let id1 = state.allocate();
    let id2 = state.allocate();
    let id3 = state.allocate();

    assert_eq!(id1.as_str(), "abl_000001");
    assert_eq!(id2.as_str(), "abl_000002");
    assert_eq!(id3.as_str(), "abl_000003");
}

#[test]
fn test_allocator_recycle() {
    let mut state = AllocatorState::new("eff_", 6);
    let id1 = state.allocate();
    assert_eq!(id1.as_str(), "eff_000001");

    state.recycle(1);
    let id2 = state.allocate();
    assert_eq!(id2.as_str(), "eff_000001");

    let id3 = state.allocate();
    assert_eq!(id3.as_str(), "eff_000002");
}

#[test]
fn test_id_allocator_full() {
    let mut allocator = IdAllocator::new_full();
    let ability_id = allocator.allocate(&IdType::Ability);
    assert!(ability_id.is_some());
    assert_eq!(ability_id.unwrap().as_str(), "abl_000001");

    let terrain_id = allocator.allocate(&IdType::Terrain);
    assert!(terrain_id.is_some());
    assert_eq!(terrain_id.unwrap().as_str(), "ter_000001");
}

#[test]
fn test_id_allocator_unregistered_type() {
    let mut allocator = IdAllocator::new();
    let result = allocator.allocate(&IdType::Ability);
    assert!(result.is_none());
}

#[test]
fn test_validate_id() {
    assert!(IdAllocator::validate_id(&DefinitionId::new("abl_000001")));
    assert!(IdAllocator::validate_id(&DefinitionId::new("ter_999999")));
    assert!(!IdAllocator::validate_id(&DefinitionId::new("xyz_000001")));
    assert!(!IdAllocator::validate_id(&DefinitionId::new("ab")));
}

#[test]
fn test_validate_id_type() {
    let id = DefinitionId::new("abl_000001");
    assert!(IdAllocator::validate_id_type(&id, &IdType::Ability));
    assert!(!IdAllocator::validate_id_type(&id, &IdType::Effect));
}

#[test]
fn test_validation_runner_clean() {
    let mut registry = DefinitionRegistry::new();
    registry.abilities.insert(
        "abl_000001",
        RegistryEntry::new("abl_000001").with_data(serde_json::json!({"name": "Test"})),
    );
    registry.effects.insert(
        "eff_000001",
        RegistryEntry::new("eff_000001").with_data(serde_json::json!({"duration": 3})),
    );

    let result = ValidationRunner::validate(&registry);
    assert!(result.is_clean());
}

#[test]
fn test_validation_runner_invalid_id() {
    let mut registry = DefinitionRegistry::new();
    registry
        .abilities
        .insert("bad_id", RegistryEntry::new("bad_id"));

    let result = ValidationRunner::validate(&registry);
    assert!(result.has_errors);
    assert!(!result.errors.is_empty());
}

#[test]
fn test_validation_runner_duplicate_across_buckets() {
    let mut registry = DefinitionRegistry::new();
    registry.abilities.insert(
        "abl_000001",
        RegistryEntry::new("abl_000001").with_data(serde_json::json!({})),
    );
    registry.effects.insert(
        "abl_000001",
        RegistryEntry::new("abl_000001").with_data(serde_json::json!({})),
    );

    let result = ValidationRunner::validate(&registry);
    assert!(result.has_errors);
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("duplicate"))
    );
}

#[test]
fn test_validation_runner_warning_on_empty_data() {
    let mut registry = DefinitionRegistry::new();
    registry
        .terrains
        .insert("ter_000001", RegistryEntry::new("ter_000001"));

    let result = ValidationRunner::validate(&registry);
    assert!(result.is_clean());
    assert!(!result.warnings.is_empty());
}
