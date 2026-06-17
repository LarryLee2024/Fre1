use crate::infra::registry::{
    DefinitionId, DefinitionRegistry, IndexKey, RegistryBucket, RegistryEntry,
};

#[test]
fn test_bucket_insert_and_get() {
    let mut bucket: RegistryBucket<RegistryEntry> = RegistryBucket::new();
    let id = DefinitionId::new("abl_000001");
    let entry = RegistryEntry::new(id.clone());

    assert!(bucket.insert(id.clone(), entry).is_none());
    assert!(bucket.contains(&id));
    assert_eq!(bucket.len(), 1);
    assert_eq!(bucket.version(), 1);
}

#[test]
fn test_bucket_replace() {
    let mut bucket: RegistryBucket<RegistryEntry> = RegistryBucket::new();
    let id = DefinitionId::new("abl_000001");
    let old_entry = RegistryEntry::new(id.clone()).with_data(serde_json::json!({"name": "old"}));
    let new_entry = RegistryEntry::new(id.clone()).with_data(serde_json::json!({"name": "new"}));

    bucket.insert(id.clone(), old_entry);
    let replaced = bucket.insert(id.clone(), new_entry);
    assert!(replaced.is_some());
    assert_eq!(bucket.len(), 1);
}

#[test]
fn test_bucket_remove() {
    let mut bucket: RegistryBucket<RegistryEntry> = RegistryBucket::new();
    let id = DefinitionId::new("eff_000001");
    bucket.insert(id.clone(), RegistryEntry::new(id.clone()));

    assert_eq!(bucket.version(), 1);
    let removed = bucket.remove(&id);
    assert!(removed.is_some());
    assert!(bucket.is_empty());
    assert_eq!(bucket.version(), 2);
}

#[test]
fn test_bucket_iter_and_ids() {
    let mut bucket: RegistryBucket<RegistryEntry> = RegistryBucket::new();
    let id1 = DefinitionId::new("abl_000001");
    let id2 = DefinitionId::new("abl_000002");
    bucket.insert(id1.clone(), RegistryEntry::new(id1.clone()));
    bucket.insert(id2.clone(), RegistryEntry::new(id2.clone()));

    let ids = bucket.ids();
    assert_eq!(ids.len(), 2);
    assert!(ids.contains(&id1));
    assert!(ids.contains(&id2));

    let count = bucket.iter().count();
    assert_eq!(count, 2);
}

#[test]
fn test_bucket_index() {
    let mut bucket: RegistryBucket<RegistryEntry> = RegistryBucket::new();
    let id = DefinitionId::new("abl_000001");
    bucket.insert(id.clone(), RegistryEntry::new(id.clone()));

    let key = IndexKey::new("category", "active");
    bucket.add_index(key.clone(), id.clone());

    let results = bucket.query_index(&key);
    assert_eq!(results, vec![id]);
}

#[test]
fn test_definition_registry_new() {
    let registry = DefinitionRegistry::new();
    assert!(registry.is_empty());
    assert_eq!(registry.total_count(), 0);
    assert!(registry.abilities.is_empty());
    assert!(registry.terrains.is_empty());
    assert!(registry.custom.is_empty());
}

#[test]
fn test_definition_registry_bucket_access() {
    let mut registry = DefinitionRegistry::new();

    let id = DefinitionId::new("ter_000001");
    registry.terrains.insert(
        id.clone(),
        RegistryEntry::new(id.clone())
            .with_data(serde_json::json!({"name": "Grass", "move_cost": 1.0})),
    );

    assert_eq!(registry.total_count(), 1);
    assert!(!registry.terrains.is_empty());
    assert_eq!(registry.terrains.version(), 1);

    let bucket = registry.bucket("terrains");
    assert!(bucket.is_some());
    assert_eq!(bucket.unwrap().len(), 1);

    let bucket_mut = registry.bucket_mut("terrains");
    assert!(bucket_mut.is_some());

    let nonexistent = registry.bucket("nonexistent");
    assert!(nonexistent.is_none());
}

#[test]
fn test_definition_id_conversions() {
    let id1 = DefinitionId::new("abl_000001");
    let id2: DefinitionId = "abl_000001".into();
    let id3: DefinitionId = String::from("abl_000001").into();

    assert_eq!(id1, id2);
    assert_eq!(id2, id3);
    assert_eq!(id1.as_str(), "abl_000001");
    assert_eq!(id1.to_string(), "abl_000001");
}

#[test]
fn test_registry_entry_lifecycle() {
    let mut entry = RegistryEntry::new("abl_000001");
    assert!(!entry.deprecated);
    assert!(entry.data.is_none());

    entry.mark_deprecated();
    assert!(entry.deprecated);

    entry.supersede("abl_000002");
    assert!(entry.superseded_by.is_some());
    assert_eq!(entry.superseded_by.as_ref().unwrap().as_str(), "abl_000002");
}

#[test]
fn test_mark_changed() {
    let mut registry = DefinitionRegistry::new();
    assert!(registry.take_changed().is_none());

    registry.mark_changed("terrains");
    assert_eq!(registry.take_changed(), Some("terrains"));
    assert!(registry.take_changed().is_none());
}

#[test]
fn test_bucket_clear() {
    let mut bucket: RegistryBucket<RegistryEntry> = RegistryBucket::new();
    bucket.insert("abl_000001", RegistryEntry::new("abl_000001"));
    bucket.insert("abl_000002", RegistryEntry::new("abl_000002"));
    assert_eq!(bucket.len(), 2);

    bucket.clear();
    assert!(bucket.is_empty());
}

#[test]
fn test_definition_id_display() {
    let id = DefinitionId::new("eff_000042");
    assert_eq!(format!("{}", id), "eff_000042");
}
