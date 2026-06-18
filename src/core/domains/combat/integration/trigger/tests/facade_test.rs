use crate::core::capabilities::trigger::foundation::TriggerType;
use crate::core::domains::combat::integration::trigger::CombatTriggerType;

#[test]
fn combat_trigger_type_converts_correctly() {
    assert_eq!(
        CombatTriggerType::TurnStarted.to_trigger_type(),
        TriggerType::OnTurnStart
    );
    assert_eq!(
        CombatTriggerType::DamageTaken.to_trigger_type(),
        TriggerType::OnDamaged
    );
}
