use crate::core::domains::faction::components::{
    FactionRelationType, RelationshipState, ReputationLevel,
};
use crate::core::domains::faction::rules::reputation::{
    apply_reputation_change, check_level_change, clamp_reputation, evaluate_relationship,
    is_reputation_change_allowed, reputation_level_to_state, safe_reputation_change,
};

#[test]
fn clamp_reputation_boundaries() {
    assert_eq!(clamp_reputation(-150), -100);
    assert_eq!(clamp_reputation(150), 100);
    assert_eq!(clamp_reputation(0), 0);
    assert_eq!(clamp_reputation(-100), -100);
    assert_eq!(clamp_reputation(100), 100);
}

#[test]
fn apply_change_with_clamp() {
    assert_eq!(apply_reputation_change(95, 10), 100);
    assert_eq!(apply_reputation_change(-95, -10), -100);
    assert_eq!(apply_reputation_change(50, -20), 30);
}

#[test]
fn level_change_detection() {
    assert_eq!(
        check_level_change(-5, 15),
        Some((ReputationLevel::Neutral, ReputationLevel::Friendly))
    );
    assert_eq!(check_level_change(0, 5), None);
    assert_eq!(
        check_level_change(-20, -60),
        Some((ReputationLevel::Hostile, ReputationLevel::Hated))
    );
}

#[test]
fn key_character_protection() {
    assert!(is_reputation_change_allowed(0, -100, false));
    assert!(is_reputation_change_allowed(0, -40, true));
    assert!(!is_reputation_change_allowed(0, -60, true));
    assert!(!is_reputation_change_allowed(-40, -20, true));
    assert!(is_reputation_change_allowed(-40, 10, true));
}

#[test]
fn safe_change_with_protection() {
    assert_eq!(safe_reputation_change(0, -40, true), Some(-40));
    assert_eq!(safe_reputation_change(0, -60, true), None);
    assert_eq!(safe_reputation_change(0, -60, false), Some(-60));
}

#[test]
fn reputation_level_mapping() {
    assert_eq!(
        reputation_level_to_state(ReputationLevel::Hated),
        RelationshipState::Hostile
    );
    assert_eq!(
        reputation_level_to_state(ReputationLevel::Neutral),
        RelationshipState::Neutral
    );
    assert_eq!(
        reputation_level_to_state(ReputationLevel::Revered),
        RelationshipState::Allied
    );
}

#[test]
fn evaluate_allied_with_hated_reputation() {
    let state = evaluate_relationship(FactionRelationType::Allied, ReputationLevel::Hated);
    assert_eq!(state, RelationshipState::Hostile);
}

#[test]
fn evaluate_hostile_with_revered_reputation() {
    let state = evaluate_relationship(FactionRelationType::Hostile, ReputationLevel::Revered);
    assert_eq!(state, RelationshipState::Neutral);
}

#[test]
fn evaluate_war_overrides_reputation() {
    for level in [
        ReputationLevel::Hated,
        ReputationLevel::Hostile,
        ReputationLevel::Neutral,
        ReputationLevel::Friendly,
        ReputationLevel::Honored,
        ReputationLevel::Revered,
    ] {
        assert_eq!(
            evaluate_relationship(FactionRelationType::War, level),
            RelationshipState::War
        );
    }
}

#[test]
fn evaluate_neutral_faction() {
    assert_eq!(
        evaluate_relationship(FactionRelationType::Neutral, ReputationLevel::Hostile),
        RelationshipState::Hostile
    );
    assert_eq!(
        evaluate_relationship(FactionRelationType::Neutral, ReputationLevel::Neutral),
        RelationshipState::Neutral
    );
    assert_eq!(
        evaluate_relationship(FactionRelationType::Neutral, ReputationLevel::Friendly),
        RelationshipState::Neutral
    );
}
