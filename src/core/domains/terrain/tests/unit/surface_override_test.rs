use crate::core::domains::terrain::components::{SurfaceOverride, SurfaceRecovery, SurfaceType};

#[test]
fn timed_override_is_not_expired_initially() {
    let ov = SurfaceOverride::timed(SurfaceType::Burning, SurfaceType::Normal, 3);
    assert!(!ov.is_expired());
    assert_eq!(ov.remaining_duration, Some(3));
}

#[test]
fn timed_override_expires_at_zero() {
    let mut ov = SurfaceOverride::timed(SurfaceType::Burning, SurfaceType::Normal, 1);
    ov.remaining_duration = Some(0);
    assert!(ov.is_expired());
}

#[test]
fn timed_override_not_expired_at_one() {
    let mut ov = SurfaceOverride::timed(SurfaceType::Burning, SurfaceType::Normal, 2);
    ov.remaining_duration = Some(1);
    assert!(!ov.is_expired());
}

#[test]
fn dispel_override_never_expires_by_countdown() {
    let ov = SurfaceOverride::dispel(SurfaceType::Poison, SurfaceType::Normal);
    assert_eq!(ov.remaining_duration, None);
    assert!(!ov.is_expired());
}

#[test]
fn permanent_override_never_expires() {
    let ov = SurfaceOverride::permanent(SurfaceType::Lava, SurfaceType::Normal);
    assert_eq!(ov.remaining_duration, None);
    assert!(!ov.is_expired());
}

#[test]
fn override_preserves_original_surface() {
    let ov = SurfaceOverride::timed(SurfaceType::Ice, SurfaceType::Water, 5);
    assert_eq!(ov.current, SurfaceType::Ice);
    assert_eq!(ov.original, SurfaceType::Water);
}

#[test]
fn override_recovery_type_matches_constructor() {
    let timed = SurfaceOverride::timed(SurfaceType::Burning, SurfaceType::Normal, 3);
    assert!(matches!(
        timed.recovery,
        SurfaceRecovery::Timed { total_duration: 3 }
    ));

    let dispel = SurfaceOverride::dispel(SurfaceType::Poison, SurfaceType::Normal);
    assert!(matches!(dispel.recovery, SurfaceRecovery::Dispel));

    let perm = SurfaceOverride::permanent(SurfaceType::Lava, SurfaceType::Normal);
    assert!(matches!(perm.recovery, SurfaceRecovery::Permanent));
}
