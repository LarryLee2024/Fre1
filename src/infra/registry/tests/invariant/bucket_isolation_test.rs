use crate::infra::registry::{DefinitionId, DefinitionRegistry, RegistryEntry};

/// 不变量：不同桶的插入互不影响。
#[test]
fn buckets_insert_does_not_affect_others() {
    let mut registry = DefinitionRegistry::new();

    registry
        .abilities
        .insert("abl_000001", RegistryEntry::new("abl_000001"));
    registry
        .effects
        .insert("eff_000001", RegistryEntry::new("eff_000001"));

    assert_eq!(registry.abilities.len(), 1);
    assert_eq!(registry.effects.len(), 1);
    assert_eq!(registry.terrains.len(), 0);
    assert_eq!(registry.total_count(), 2);
}

/// 不变量：从一个桶移除不影响其他桶。
#[test]
fn buckets_remove_does_not_affect_others() {
    let mut registry = DefinitionRegistry::new();

    registry
        .abilities
        .insert("abl_000001", RegistryEntry::new("abl_000001"));
    registry
        .effects
        .insert("eff_000001", RegistryEntry::new("eff_000001"));

    registry.abilities.remove(&DefinitionId::new("abl_000001"));

    assert!(registry.abilities.is_empty());
    assert_eq!(registry.effects.len(), 1);
    assert_eq!(registry.total_count(), 1);
}

/// 不变量：桶的版本号各自独立递增。
#[test]
fn buckets_version_tracking_is_independent() {
    let mut registry = DefinitionRegistry::new();

    assert_eq!(registry.abilities.version(), 0);
    assert_eq!(registry.terrains.version(), 0);

    registry
        .abilities
        .insert("abl_000001", RegistryEntry::new("abl_000001"));
    assert_eq!(registry.abilities.version(), 1);
    assert_eq!(registry.terrains.version(), 0);

    registry
        .terrains
        .insert("ter_000001", RegistryEntry::new("ter_000001"));
    assert_eq!(registry.abilities.version(), 1);
    assert_eq!(registry.terrains.version(), 1);

    registry.terrains.clear();
    assert_eq!(registry.abilities.version(), 1);
    assert_eq!(registry.terrains.version(), 2);
}

/// 不变量：相同 ID 字符串在不同桶中不冲突。
#[test]
fn buckets_same_id_different_buckets_no_conflict() {
    let mut registry = DefinitionRegistry::new();

    // "abl_000001" in abilities
    registry
        .abilities
        .insert("abl_000001", RegistryEntry::new("abl_000001"));
    // Same string "abl_000001" in effects — should not conflict per V2 check
    registry
        .effects
        .insert("abl_000001", RegistryEntry::new("abl_000001"));

    assert_eq!(registry.abilities.len(), 1);
    assert_eq!(registry.effects.len(), 1);
    // V2 validation catches duplicates across buckets, but storage is independent
    assert_eq!(registry.total_count(), 2);
}

/// 不变量：清空一个桶不影响其他桶。
#[test]
fn buckets_clear_does_not_affect_others() {
    let mut registry = DefinitionRegistry::new();

    registry
        .abilities
        .insert("abl_000001", RegistryEntry::new("abl_000001"));
    registry
        .effects
        .insert("eff_000001", RegistryEntry::new("eff_000001"));
    registry
        .terrains
        .insert("ter_000001", RegistryEntry::new("ter_000001"));

    registry.abilities.clear();

    assert!(registry.abilities.is_empty());
    assert_eq!(registry.effects.len(), 1);
    assert_eq!(registry.terrains.len(), 1);
    assert_eq!(registry.total_count(), 2);
}

/// 不变量：通过 bucket_mut 操作后，其他桶不受影响。
#[test]
fn buckets_dynamic_mut_access_is_isolated() {
    let mut registry = DefinitionRegistry::new();

    registry
        .abilities
        .insert("abl_000001", RegistryEntry::new("abl_000001"));

    let terrain_bucket = registry.bucket_mut("terrains").unwrap();
    terrain_bucket.insert("ter_000001", RegistryEntry::new("ter_000001"));

    assert_eq!(registry.abilities.len(), 1);
    assert_eq!(registry.terrains.len(), 1);
}
