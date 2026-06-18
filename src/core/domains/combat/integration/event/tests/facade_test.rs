use crate::core::capabilities::event::foundation::EventTag;
use crate::core::domains::combat::integration::event::CombatEventTag;

#[test]
fn combat_event_tag_converts_correctly() {
    assert_eq!(
        CombatEventTag::TurnStarted.to_event_tag(),
        EventTag::TurnStarted
    );
    assert_eq!(
        CombatEventTag::Kill.to_event_tag(),
        EventTag::Custom("Kill".to_string())
    );
}
