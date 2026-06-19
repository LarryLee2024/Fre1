use crate::core::domains::faction::components::{
    FactionId, FactionMembership, FactionRelationTable, FactionRelationType, Reputation,
    ReputationLevel,
};

#[test]
fn reputation_level_from_value() {
    assert_eq!(ReputationLevel::from_value(-100), ReputationLevel::Hated);
    assert_eq!(ReputationLevel::from_value(-51), ReputationLevel::Hated);
    assert_eq!(ReputationLevel::from_value(-50), ReputationLevel::Hostile);
    assert_eq!(ReputationLevel::from_value(0), ReputationLevel::Neutral);
    assert_eq!(ReputationLevel::from_value(49), ReputationLevel::Friendly);
    assert_eq!(ReputationLevel::from_value(50), ReputationLevel::Honored);
    assert_eq!(ReputationLevel::from_value(100), ReputationLevel::Revered);
}

#[test]
fn reputation_level_clamps_out_of_range() {
    assert_eq!(ReputationLevel::from_value(-999), ReputationLevel::Hated);
    assert_eq!(ReputationLevel::from_value(999), ReputationLevel::Revered);
}

#[test]
fn reputation_get_and_set() {
    let mut rep = Reputation::new();
    let faction = FactionId::new("faction_a");

    assert_eq!(rep.get(&faction), 0);

    let set_val = rep.set(faction.clone(), 50);
    assert_eq!(set_val, 50);
    assert_eq!(rep.get(&faction), 50);
    assert_eq!(rep.level(&faction), ReputationLevel::Honored);
}

#[test]
fn reputation_modify_clamps() {
    let mut rep = Reputation::new();
    let faction = FactionId::new("faction_a");

    rep.set(faction.clone(), 95);
    let new_val = rep.modify(&faction, 10);
    assert_eq!(new_val, 100);

    rep.set(faction.clone(), -95);
    let new_val = rep.modify(&faction, -10);
    assert_eq!(new_val, -100);
}

#[test]
fn faction_membership_join_leave() {
    let mut membership = FactionMembership::new();
    let f1 = FactionId::new("faction_a");
    let f2 = FactionId::new("faction_b");

    membership.join(f1.clone());
    assert!(membership.is_member(&f1));
    assert!(!membership.is_member(&f2));

    membership.join(f2.clone());
    assert_eq!(membership.factions.len(), 2);

    membership.join(f1.clone());
    assert_eq!(membership.factions.len(), 2);

    membership.leave(&f1);
    assert!(!membership.is_member(&f1));
    assert!(membership.is_member(&f2));
}

#[test]
fn relation_table_symmetry() {
    let mut table = FactionRelationTable::new();
    let f1 = FactionId::new("faction_a");
    let f2 = FactionId::new("faction_b");

    table.set_relation(f1.clone(), f2.clone(), FactionRelationType::Hostile);
    assert_eq!(table.get_relation(&f1, &f2), FactionRelationType::Hostile);
    assert_eq!(table.get_relation(&f2, &f1), FactionRelationType::Hostile);

    assert_eq!(table.get_relation(&f1, &f1), FactionRelationType::Allied);

    let f3 = FactionId::new("faction_c");
    assert_eq!(table.get_relation(&f1, &f3), FactionRelationType::Neutral);
}
