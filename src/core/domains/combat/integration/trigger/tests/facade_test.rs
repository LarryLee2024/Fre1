//! CombatTriggerFacade 测试
//!
//! 验证触发器 facade 的条目创建、触发评估、批量评估、空容器创建。

use crate::core::capabilities::trigger::foundation::{TriggerCondition, TriggerEntry, TriggerType};
use crate::core::capabilities::trigger::mechanism::{TriggerContainer, TriggerEvalResult};
use crate::core::domains::combat::integration::trigger::{CombatTriggerFacade, CombatTriggerType};

#[test]
fn create_trigger_entry_has_correct_fields() {
    let entry = CombatTriggerFacade::create_trigger_entry(
        "trg_001",
        CombatTriggerType::TurnStarted,
        "def_counter_attack",
    );
    assert_eq!(entry.id, "trg_001");
    assert_eq!(entry.trigger_type, TriggerType::OnTurnStart);
    assert_eq!(entry.target_ability_def_id, "def_counter_attack");
}

#[test]
fn create_trigger_entry_with_damage_taken_type() {
    let entry = CombatTriggerFacade::create_trigger_entry(
        "trg_002",
        CombatTriggerType::DamageTaken,
        "def_retaliate",
    );
    assert_eq!(entry.trigger_type, TriggerType::OnDamaged);
}

#[test]
fn create_trigger_entry_with_kill_type() {
    let entry = CombatTriggerFacade::create_trigger_entry(
        "trg_003",
        CombatTriggerType::Kill,
        "def_bloodlust",
    );
    assert_eq!(entry.trigger_type, TriggerType::OnDeath);
}

#[test]
fn can_trigger_check_passes_when_type_matches() {
    let entry = CombatTriggerFacade::create_trigger_entry(
        "trg_001",
        CombatTriggerType::TurnStarted,
        "def_buff",
    );
    let result = CombatTriggerFacade::can_trigger_check(&entry, &TriggerType::OnTurnStart, None);
    assert!(matches!(result, TriggerEvalResult::Ready(_)));
}

#[test]
fn can_trigger_check_fails_when_type_mismatches() {
    let entry = CombatTriggerFacade::create_trigger_entry(
        "trg_001",
        CombatTriggerType::TurnStarted,
        "def_buff",
    );
    let result = CombatTriggerFacade::can_trigger_check(&entry, &TriggerType::OnDamaged, None);
    assert!(matches!(result, TriggerEvalResult::Blocked(_)));
}

#[test]
fn evaluate_triggers_filters_by_type() {
    let entries = vec![
        CombatTriggerFacade::create_trigger_entry(
            "trg_001",
            CombatTriggerType::TurnStarted,
            "def_buff",
        ),
        CombatTriggerFacade::create_trigger_entry(
            "trg_002",
            CombatTriggerType::DamageTaken,
            "def_retaliate",
        ),
        CombatTriggerFacade::create_trigger_entry(
            "trg_003",
            CombatTriggerType::Attack,
            "def_berserk",
        ),
    ];

    let ready =
        CombatTriggerFacade::evaluate_triggers(&entries, CombatTriggerType::TurnStarted, None);
    assert_eq!(ready.len(), 1);
    assert_eq!(ready[0].id, "trg_001");
}

#[test]
fn evaluate_triggers_returns_empty_when_no_match() {
    let entries = vec![CombatTriggerFacade::create_trigger_entry(
        "trg_001",
        CombatTriggerType::DamageTaken,
        "def_retaliate",
    )];

    let ready =
        CombatTriggerFacade::evaluate_triggers(&entries, CombatTriggerType::TurnStarted, None);
    assert!(ready.is_empty());
}

#[test]
fn evaluate_triggers_condition_check_blocks() {
    let entry = CombatTriggerFacade::create_trigger_entry(
        "trg_001",
        CombatTriggerType::TurnStarted,
        "def_buff",
    )
    .with_condition(TriggerCondition::with_condition("has_buff"));

    let condition_check = |cond: &str| -> bool { cond != "has_buff" };

    let ready = CombatTriggerFacade::evaluate_triggers(
        &[entry],
        CombatTriggerType::TurnStarted,
        Some(&condition_check),
    );
    assert!(
        ready.is_empty(),
        "condition 'has_buff' should block trigger"
    );
}

#[test]
fn evaluate_triggers_condition_check_passes() {
    let entry = CombatTriggerFacade::create_trigger_entry(
        "trg_001",
        CombatTriggerType::TurnStarted,
        "def_buff",
    )
    .with_condition(TriggerCondition::with_condition("has_buff"));

    let condition_check = |cond: &str| -> bool { cond == "has_buff" };

    let ready = CombatTriggerFacade::evaluate_triggers(
        &[entry],
        CombatTriggerType::TurnStarted,
        Some(&condition_check),
    );
    assert_eq!(ready.len(), 1, "condition 'has_buff' should pass");
}

#[test]
fn empty_container_has_no_entries() {
    let container = CombatTriggerFacade::empty_container();
    assert!(container.triggers.is_empty());
}
