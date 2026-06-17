use crate::core::capabilities::runtime::registry::foundation::{
    DefRegistry, IdType, RegistryEntry,
};
use crate::core::capabilities::runtime::registry::mechanism::validator::*;

#[test]
fn validate_valid_format_passes() {
    assert!(validate_id_format("abl_000001").is_ok());
    assert!(validate_id_format("eff_000042").is_ok());
}

#[test]
fn empty_format_validation_fails() {
    assert!(validate_id_format("").is_err());
}

#[test]
fn unknown_prefix_validation_fails() {
    assert!(validate_id_format("xxx_000001").is_err());
}

#[test]
fn cross_reference_discovers_broken() {
    let mut reg = DefRegistry::new();
    reg.register(RegistryEntry::new(
        "abl_000001",
        "Ability",
        "uses=eff_000001",
    ))
    .unwrap();
    // eff_000001 is not registered, so it's broken
    let report = validate_cross_references(&reg);
    assert!(report.has_broken_references());
}

#[test]
fn cross_references_all_valid() {
    let mut reg = DefRegistry::new();
    reg.register(RegistryEntry::new(
        "abl_000001",
        "Ability",
        "uses=eff_000001",
    ))
    .unwrap();
    reg.register(RegistryEntry::new("eff_000001", "Effect", "type=damage"))
        .unwrap();
    let report = validate_cross_references(&reg);
    assert!(!report.has_broken_references());
}

#[test]
fn extract_id_references_test() {
    let data = "effect=eff_000001,trigger=trg_000042";
    let refs = extract_id_references(data);
    assert!(refs.contains(&"eff_000001".to_string()));
    assert!(refs.contains(&"trg_000042".to_string()));
}

#[test]
fn no_id_references_returns_empty() {
    let data = "name=Fireball,damage=50";
    let refs = extract_id_references(data);
    assert!(refs.is_empty());
}

#[test]
fn infer_id_type_from_prefix() {
    assert_eq!(IdType::from_prefix("abl_"), Some(IdType::Ability));
    assert_eq!(IdType::from_prefix("eff_"), Some(IdType::Effect));
    assert_eq!(IdType::from_prefix("sho"), None);
}
