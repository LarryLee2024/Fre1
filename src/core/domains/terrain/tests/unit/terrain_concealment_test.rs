use crate::core::domains::terrain::components::Concealment;
use crate::core::domains::terrain::rules::concealment::{concealment_bonus, is_targetable};

#[test]
fn no_concealment_no_penalty() {
    assert_eq!(concealment_bonus(&Concealment::None), 0);
}

#[test]
fn half_concealment_minus_two() {
    assert_eq!(concealment_bonus(&Concealment::Half), -2);
}

#[test]
fn full_concealment_is_untargetable() {
    assert!(!is_targetable(&Concealment::Full));
}

#[test]
fn none_and_half_are_targetable() {
    assert!(is_targetable(&Concealment::None));
    assert!(is_targetable(&Concealment::Half));
}
