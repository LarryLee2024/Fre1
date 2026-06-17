use crate::core::domains::faction::components::{
    FactionId, FactionMembership, FactionRelationTable, FactionRelationType, RelationshipState,
    Reputation, ReputationLevel,
};
use crate::core::domains::faction::rules::relationship::relationship_between_entities;
use crate::core::domains::faction::rules::relationship::stronger_relationship;

fn make_membership(factions: &[&str]) -> FactionMembership {
    let mut m = FactionMembership::new();
    for f in factions {
        m.join(FactionId::new(*f));
    }
    m
}

fn make_reputation(entries: &[(&str, i32)]) -> Reputation {
    let mut r = Reputation::new();
    for (f, v) in entries {
        r.set(FactionId::new(*f), *v);
    }
    r
}

#[test]
fn shared_faction_is_allied() {
    let table = FactionRelationTable::new();
    let m1 = make_membership(&["faction_a"]);
    let m2 = make_membership(&["faction_a"]);
    let r1 = make_reputation(&[]);
    let r2 = make_reputation(&[]);

    let state = relationship_between_entities(&m1, &r1, &m2, &table);
    assert_eq!(state, RelationshipState::Allied);
}

#[test]
fn hostile_relation_without_reputation() {
    let mut table = FactionRelationTable::new();
    table.set_relation(
        FactionId::new("faction_a"),
        FactionId::new("faction_b"),
        FactionRelationType::Hostile,
    );

    let m1 = make_membership(&["faction_a"]);
    let m2 = make_membership(&["faction_b"]);
    let r1 = make_reputation(&[]);

    let state = relationship_between_entities(&m1, &r1, &m2, &table);
    assert_eq!(state, RelationshipState::Hostile);
}

#[test]
fn revered_reputation_overrides_hostile() {
    let mut table = FactionRelationTable::new();
    table.set_relation(
        FactionId::new("faction_a"),
        FactionId::new("faction_b"),
        FactionRelationType::Hostile,
    );

    let m1 = make_membership(&["faction_a"]);
    let m2 = make_membership(&["faction_b"]);
    let r1 = make_reputation(&[("faction_b", 90)]);

    let state = relationship_between_entities(&m1, &r1, &m2, &table);
    assert_eq!(state, RelationshipState::Neutral);
}

#[test]
fn war_overrides_everything() {
    let mut table = FactionRelationTable::new();
    table.set_relation(
        FactionId::new("faction_a"),
        FactionId::new("faction_b"),
        FactionRelationType::War,
    );

    let m1 = make_membership(&["faction_a"]);
    let m2 = make_membership(&["faction_b"]);
    let r1 = make_reputation(&[("faction_b", 100)]);

    let state = relationship_between_entities(&m1, &r1, &m2, &table);
    assert_eq!(state, RelationshipState::War);
}

#[test]
fn no_faction_uses_reputation() {
    let table = FactionRelationTable::new();
    let m1 = make_membership(&[]);
    let m2 = make_membership(&["faction_b"]);
    let r1 = make_reputation(&[("faction_b", 50)]);

    let state = relationship_between_entities(&m1, &r1, &m2, &table);
    assert_eq!(state, RelationshipState::Allied);
}

#[test]
fn stronger_relationship_logic() {
    use RelationshipState::*;
    assert_eq!(stronger_relationship(War, Allied), War);
    assert_eq!(stronger_relationship(Hostile, Neutral), Hostile);
    assert_eq!(stronger_relationship(Neutral, Allied), Neutral);
    assert_eq!(stronger_relationship(Allied, Allied), Allied);
}
