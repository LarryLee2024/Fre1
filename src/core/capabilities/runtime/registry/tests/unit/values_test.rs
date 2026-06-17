use crate::core::capabilities::runtime::registry::foundation::{
    CrossReferenceReport, DefRegistry, RegistryEntry,
};

#[test]
fn registry_initially_empty() {
    let reg = DefRegistry::new();
    assert_eq!(reg.count(), 0);
}

#[test]
fn registry_registers_items() {
    let mut reg = DefRegistry::new();
    let entry = RegistryEntry::new("abl_000001", "Ability", "name=Fireball");
    assert!(reg.register(entry).is_ok());
    assert_eq!(reg.count(), 1);
}

#[test]
fn duplicate_id_registration_rejected() {
    let mut reg = DefRegistry::new();
    reg.register(RegistryEntry::new("abl_000001", "Ability", ""))
        .unwrap();
    let result = reg.register(RegistryEntry::new("abl_000001", "Ability", ""));
    assert!(result.is_err());
}

#[test]
fn registry_retrieves_item() {
    let mut reg = DefRegistry::new();
    reg.register(RegistryEntry::new("abl_000001", "Ability", "name=Fireball"))
        .unwrap();
    let entry = reg.get("abl_000001");
    assert!(entry.is_some());
    assert_eq!(entry.unwrap().data, "name=Fireball");
}

#[test]
fn registry_retrieves_nonexistent_item() {
    let reg = DefRegistry::new();
    assert!(reg.get("nonexistent").is_none());
}

#[test]
fn registry_retrieve_by_type() {
    let mut reg = DefRegistry::new();
    reg.register(RegistryEntry::new("abl_000001", "Ability", ""))
        .unwrap();
    reg.register(RegistryEntry::new("abl_000002", "Ability", ""))
        .unwrap();
    reg.register(RegistryEntry::new("eff_000001", "Effect", ""))
        .unwrap();

    let abilities = reg.get_by_type("Ability");
    assert_eq!(abilities.len(), 2);

    let effects = reg.get_by_type("Effect");
    assert_eq!(effects.len(), 1);
}

#[test]
fn registry_contains_check() {
    let mut reg = DefRegistry::new();
    reg.register(RegistryEntry::new("abl_000001", "Ability", ""))
        .unwrap();
    assert!(reg.contains("abl_000001"));
    assert!(!reg.contains("abl_999999"));
}

#[test]
fn registry_mark_deprecated() {
    let mut reg = DefRegistry::new();
    reg.register(RegistryEntry::new("abl_000001", "Ability", ""))
        .unwrap();
    assert!(
        reg.mark_deprecated("abl_000001", Some("abl_000042".into()))
            .is_ok()
    );

    let entry = reg.get("abl_000001").unwrap();
    assert!(entry.deprecated);
    assert_eq!(entry.superseded_by, Some("abl_000042".into()));
}

#[test]
fn registry_get_all_ids() {
    let mut reg = DefRegistry::new();
    reg.register(RegistryEntry::new("abl_000001", "Ability", ""))
        .unwrap();
    reg.register(RegistryEntry::new("eff_000001", "Effect", ""))
        .unwrap();
    assert_eq!(reg.all_ids().len(), 2);
}

#[test]
fn cross_reference_report_initial() {
    let report = CrossReferenceReport::new();
    assert!(!report.has_broken_references());
}

#[test]
fn empty_id_registration_rejected() {
    let mut reg = DefRegistry::new();
    let result = reg.register(RegistryEntry::new("", "Ability", ""));
    assert!(result.is_err());
}
