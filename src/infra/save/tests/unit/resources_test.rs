use crate::infra::save::resources::*;
use bevy::prelude::Entity;

#[test]
fn save_manager_default_is_new_game() {
    let m = SaveManager::default();
    assert!(m.current_save_path.is_none());
    assert!(!m.is_dirty);
    assert_eq!(m.save_version, 1);
}

#[test]
fn save_manager_default_metadata() {
    let m = SaveManager::default();
    assert_eq!(m.metadata.label, "Untitled Save");
    assert_eq!(m.metadata.player_level, 1);
}

#[test]
fn auto_save_config_default_enabled() {
    let c = AutoSaveConfig::default();
    assert!(c.enabled);
    assert_eq!(c.interval_minutes, 15);
    assert_eq!(c.max_auto_saves, 5);
}

#[test]
fn entity_remapper_default_empty() {
    let r = EntityRemapper::default();
    assert!(r.is_empty());
}

#[test]
fn entity_remapper_assign_and_lookup() {
    let mut r = EntityRemapper::default();
    let e = Entity::from_raw_u32(42).unwrap();
    let pid = r.assign(e);
    assert_eq!(pid.0, 1);
    assert_eq!(r.lookup(pid), Some(e));
}

#[test]
fn entity_remapper_lookup_missing() {
    let r = EntityRemapper::default();
    assert!(r.lookup(PersistentEntityId(999)).is_none());
}

#[test]
fn entity_remapper_clear_resets() {
    let mut r = EntityRemapper::default();
    r.assign(Entity::from_raw_u32(1).unwrap());
    r.assign(Entity::from_raw_u32(2).unwrap());
    assert_eq!(r.len(), 2);
    r.clear();
    assert!(r.is_empty());
    let pid = r.assign(Entity::from_raw_u32(3).unwrap());
    assert_eq!(pid.0, 1, "counter should reset after clear");
}
