use crate::core::capabilities::ability::foundation::{
    AbilityError, AbilityInstanceId, AbilityState, ActivationContext, ActivationType,
    CooldownEntry, CostEntry,
};
use crate::core::capabilities::ability::mechanism::{
    ActivationRequest, ActiveAbilityContainer, advance_cast_progress, apply_block, cancel_ability,
    complete_ability, force_reset_cooldown, get_ready_abilities, remove_block, start_cooldown,
    start_multiple_cooldowns, tick_cooldowns, transition_to, try_activate,
};

fn make_request(spec_id: &str, def_id: &str, frame: u64) -> ActivationRequest {
    ActivationRequest {
        spec_id: spec_id.to_string(),
        def_id: def_id.to_string(),
        activation: ActivationType::Instant,
        context: ActivationContext::new("caster_001", frame),
        costs: vec![],
    }
}

fn make_request_casting(spec_id: &str, def_id: &str, frame: u64) -> ActivationRequest {
    ActivationRequest {
        spec_id: spec_id.to_string(),
        def_id: def_id.to_string(),
        activation: ActivationType::CastTime { frames: 3 },
        context: ActivationContext::new("caster_001", frame),
        costs: vec![],
    }
}

#[test]
fn activate_instant_ability_succeeds() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request("abl_001", "abl_000001", 1);

    let result = try_activate(&mut container, request);
    assert!(result.is_ok());

    let id = result.unwrap();
    let instance = container.get_instance(&id).unwrap();
    assert_eq!(instance.state, AbilityState::Active);
    assert_eq!(instance.def_id, "abl_000001");
}

#[test]
fn activate_casting_ability_succeeds() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request_casting("abl_002", "abl_000002", 1);

    let result = try_activate(&mut container, request);
    assert!(result.is_ok());

    let id = result.unwrap();
    let instance = container.get_instance(&id).unwrap();
    assert_eq!(instance.state, AbilityState::Casting);
    assert_eq!(instance.cast_total, 3);
}

#[test]
fn reject_activation_when_on_cooldown() {
    let mut container = ActiveAbilityContainer::empty();
    container.set_cooldown(CooldownEntry::new("abl_001", 2));

    let request = make_request("abl_001", "abl_000001", 1);
    let result = try_activate(&mut container, request);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AbilityError::OnCooldown { .. }
    ));
}

#[test]
fn reject_duplicate_activation_of_active_ability() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request("abl_001", "abl_000001", 1);
    try_activate(&mut container, request).unwrap();

    let request2 = make_request("abl_001", "abl_000001", 2);
    let result = try_activate(&mut container, request2);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        AbilityError::AlreadyActive { .. }
    ));
}

#[test]
fn activate_ability_with_costs() {
    let mut container = ActiveAbilityContainer::empty();
    let mut request = make_request("abl_001", "abl_000001", 1);
    request.costs = vec![CostEntry::new("mana", 20.0)];

    let result = try_activate(&mut container, request);
    assert!(result.is_ok());

    let id = result.unwrap();
    let instance = container.get_instance(&id).unwrap();
    assert_eq!(instance.costs.len(), 1);
    assert!(!instance.all_costs_consumed());
}

#[test]
fn transition_from_ready_to_active() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request_casting("abl_001", "abl_000001", 1);
    let id = try_activate(&mut container, request).unwrap();

    let result = transition_to(&mut container, &id, AbilityState::Active);
    assert!(result.is_ok());
    assert_eq!(
        container.get_instance(&id).unwrap().state,
        AbilityState::Active
    );
}

#[test]
fn transition_from_casting_to_ready() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request_casting("abl_001", "abl_000001", 1);
    let id = try_activate(&mut container, request).unwrap();

    let result = transition_to(&mut container, &id, AbilityState::Ready);
    assert!(result.is_ok());
    assert_eq!(
        container.get_instance(&id).unwrap().state,
        AbilityState::Ready
    );
}

#[test]
fn invalid_transition_from_ready_to_cooldown() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request_casting("abl_001", "abl_000001", 1);
    let id = try_activate(&mut container, request).unwrap();

    transition_to(&mut container, &id, AbilityState::Ready).unwrap();

    let result = transition_to(&mut container, &id, AbilityState::Cooldown);
    assert!(result.is_err());
}

#[test]
fn transition_from_any_state_to_removed() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request("abl_001", "abl_000001", 1);
    let id = try_activate(&mut container, request).unwrap();

    let result = transition_to(&mut container, &id, AbilityState::Removed);
    assert!(result.is_ok());
}

#[test]
fn cancel_casting_returns_to_ready() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request_casting("abl_001", "abl_000001", 1);
    let id = try_activate(&mut container, request).unwrap();

    cancel_ability(&mut container, &id).unwrap();
    let instance = container.get_instance(&id).unwrap();
    assert_eq!(instance.state, AbilityState::Ready);
    assert_eq!(instance.cast_progress, 0);
}

#[test]
fn cancel_active_removes_ability() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request("abl_001", "abl_000001", 1);
    let id = try_activate(&mut container, request).unwrap();

    cancel_ability(&mut container, &id).unwrap();
    let instance = container.get_instance(&id).unwrap();
    assert_eq!(instance.state, AbilityState::Removed);
}

#[test]
fn cancel_ready_ability_returns_error() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request("abl_001", "abl_000001", 1);
    let id = try_activate(&mut container, request).unwrap();

    cancel_ability(&mut container, &id).unwrap();

    let result = cancel_ability(&mut container, &id);
    assert!(result.is_err());
}

#[test]
fn complete_ability_enters_cooldown() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request("abl_001", "abl_000001", 1);
    let id = try_activate(&mut container, request).unwrap();

    complete_ability(&mut container, &id, 3).unwrap();
    assert!(container.get_instance(&id).is_none());
    assert!(container.is_on_cooldown("abl_001"));
    assert_eq!(container.cooldown_remaining("abl_001"), 3);
}

#[test]
fn complete_ability_no_cooldown() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request("abl_001", "abl_000001", 1);
    let id = try_activate(&mut container, request).unwrap();

    complete_ability(&mut container, &id, 0).unwrap();
    assert!(container.get_instance(&id).is_none());
    assert!(!container.is_on_cooldown("abl_001"));
}

#[test]
fn complete_casting_ability_returns_error() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request_casting("abl_001", "abl_000001", 1);
    let id = try_activate(&mut container, request).unwrap();

    let result = complete_ability(&mut container, &id, 2);
    assert!(result.is_err());
}

#[test]
fn cooldown_timer_expires() {
    let mut container = ActiveAbilityContainer::empty();
    container.set_cooldown(CooldownEntry::new("abl_001", 2));

    assert!(container.is_on_cooldown("abl_001"));

    let expired = tick_cooldowns(&mut container);
    assert!(expired.is_empty());
    assert!(container.is_on_cooldown("abl_001"));

    let expired = tick_cooldowns(&mut container);
    assert_eq!(expired, vec!["abl_001"]);
    assert!(!container.is_on_cooldown("abl_001"));
}

#[test]
fn test_force_reset_cooldown() {
    let mut container = ActiveAbilityContainer::empty();
    start_cooldown(&mut container, "abl_001", 5);
    assert!(container.is_on_cooldown("abl_001"));

    force_reset_cooldown(&mut container, "abl_001");
    assert!(!container.is_on_cooldown("abl_001"));
}

#[test]
fn start_cooldown_zero_turns_noop() {
    let mut container = ActiveAbilityContainer::empty();
    start_cooldown(&mut container, "abl_001", 0);
    assert!(!container.is_on_cooldown("abl_001"));
}

#[test]
fn apply_block_pauses_all_active() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request("abl_001", "abl_000001", 1);
    let id = try_activate(&mut container, request).unwrap();

    apply_block(&mut container);

    let instance = container.get_instance(&id).unwrap();
    assert_eq!(instance.state, AbilityState::Blocked);
    assert!(instance.paused);
}

#[test]
fn remove_block_restores_state() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request("abl_001", "abl_000001", 1);
    let id = try_activate(&mut container, request).unwrap();

    apply_block(&mut container);
    remove_block(&mut container);

    let instance = container.get_instance(&id).unwrap();
    assert_eq!(instance.state, AbilityState::Active);
    assert!(!instance.paused);
}

#[test]
fn block_casting_restores_to_casting() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request_casting("abl_001", "abl_000001", 1);
    let id = try_activate(&mut container, request).unwrap();

    apply_block(&mut container);
    remove_block(&mut container);

    let instance = container.get_instance(&id).unwrap();
    assert_eq!(instance.state, AbilityState::Casting);
}

#[test]
fn test_advance_cast_progress() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request_casting("abl_001", "abl_000001", 1);
    let id = try_activate(&mut container, request).unwrap();

    let done = advance_cast_progress(&mut container, &id, 2).unwrap();
    assert!(!done);

    let instance = container.get_instance(&id).unwrap();
    assert_eq!(instance.cast_progress, 2);

    let done = advance_cast_progress(&mut container, &id, 1).unwrap();
    assert!(done);
}

#[test]
fn advance_cast_progress_on_instant_returns_error() {
    let mut container = ActiveAbilityContainer::empty();
    let request = make_request("abl_001", "abl_000001", 1);
    let id = try_activate(&mut container, request).unwrap();

    let result = advance_cast_progress(&mut container, &id, 1);
    assert!(result.is_err());
}

#[test]
fn test_get_ready_abilities() {
    let mut container = ActiveAbilityContainer::empty();
    start_cooldown(&mut container, "abl_001", 2);
    let request = make_request("abl_002", "abl_000002", 1);
    try_activate(&mut container, request).unwrap();

    let all_specs = vec!["abl_001".into(), "abl_002".into(), "abl_003".into()];
    let ready = get_ready_abilities(&container, &all_specs);

    assert_eq!(ready, vec!["abl_003"]);
}

#[test]
fn test_start_multiple_cooldowns() {
    let mut container = ActiveAbilityContainer::empty();
    start_multiple_cooldowns(
        &mut container,
        vec![("abl_001".into(), 2), ("abl_002".into(), 3)],
    );

    assert!(container.is_on_cooldown("abl_001"));
    assert!(container.is_on_cooldown("abl_002"));
    assert_eq!(container.cooldown_remaining("abl_002"), 3);
}

#[test]
fn ability_error_display() {
    let err = AbilityError::NotReady {
        current_state: AbilityState::Cooldown,
        spec_id: "abl_001".into(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("abl_001"));
    assert!(msg.contains("Cooldown"));
}

#[test]
fn instance_id_display() {
    let id = AbilityInstanceId::from_u64(42);
    let msg = format!("{}", id);
    assert_eq!(msg, "inst_0000000042");
}
