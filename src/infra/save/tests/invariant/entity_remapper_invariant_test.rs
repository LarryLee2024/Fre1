use crate::infra::save::resources::EntityRemapper;
use bevy::prelude::Entity;

#[test]
fn remapper_ids_strictly_increasing() {
    let mut r = EntityRemapper::default();
    let mut prev = 0u64;
    for i in 0..100u64 {
        let pid = r.assign(Entity::from_raw_u32(i as u32).unwrap());
        assert!(pid.0 > prev);
        prev = pid.0;
    }
}

#[test]
fn remapper_lookup_is_inverse_of_assign() {
    let mut r = EntityRemapper::default();
    let entities: Vec<_> = (0..50).map(|i| Entity::from_raw_u32(i).unwrap()).collect();
    let ids: Vec<_> = entities.iter().map(|e| r.assign(*e)).collect();
    for (e, pid) in entities.iter().zip(ids.iter()) {
        assert_eq!(r.lookup(*pid), Some(*e));
    }
}

#[test]
fn remapper_clear_invalidates_all() {
    let mut r = EntityRemapper::default();
    let pid = r.assign(Entity::from_raw_u32(1).unwrap());
    r.clear();
    assert!(r.lookup(pid).is_none());
}

#[test]
fn remapper_same_entity_multiple_ids() {
    let mut r = EntityRemapper::default();
    let e = Entity::from_raw_u32(42).unwrap();
    let p1 = r.assign(e);
    let p2 = r.assign(e);
    assert_ne!(p1, p2);
    assert!(p2.0 > p1.0);
}
