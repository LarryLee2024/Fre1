//! 技能消耗原子性不变量测试
//!
//! 不变量：消耗失败时不产生部分效果（原子性）。
//! 来源：docs/02-domain/capabilities/ability_domain.md §3.1, §3.5, §5.1
//!
//! 验证：
//! 1. CostEntry 初始状态为未消耗
//! 2. mark_costs_consumed() 原子性地标记所有消耗
//! 3. all_costs_consumed() 在全部消耗后返回 true
//! 4. 部分消耗时 all_costs_consumed() 返回 false

use crate::core::capabilities::ability::foundation::types::{AbilityInstanceId, ActivationType};
use crate::core::capabilities::ability::foundation::values::{
    AbilityInstance, ActivationContext, CostEntry,
};
use crate::shared::ids::types::runtime_id::RuntimeId;

fn make_instance() -> AbilityInstance {
    AbilityInstance::new(
        AbilityInstanceId::new(RuntimeId::new(0, 0)),
        "spec_001",
        "abl_001",
        ActivationType::Instant,
        ActivationContext::new("caster_001", 1),
    )
}

#[test]
fn cost_entry_default_state_not_consumed() {
    let cost = CostEntry::new("attr_hp", 30.0);
    assert!(!cost.consumed);
    assert_eq!(cost.resource, "attr_hp");
    assert_eq!(cost.amount, 30.0);
}

#[test]
fn all_costs_consumed_returns_false_when_no_costs() {
    let instance = make_instance();
    assert!(!instance.all_costs_consumed());
}

#[test]
fn all_costs_consumed_returns_false_when_one_cost_unmarked() {
    let mut instance = make_instance();
    instance.add_cost(CostEntry::new("attr_hp", 30.0));
    assert!(!instance.all_costs_consumed());
}

#[test]
fn mark_costs_consumed_atomically_marks_all() {
    let mut instance = make_instance();
    instance.add_cost(CostEntry::new("attr_hp", 30.0));
    instance.add_cost(CostEntry::new("attr_mp", 20.0));
    instance.add_cost(CostEntry::new("attr_sp", 10.0));

    instance.mark_costs_consumed();

    assert!(instance.all_costs_consumed());
    assert!(instance.costs.iter().all(|c| c.consumed));
}

#[test]
fn all_costs_consumed_returns_false_on_partial_consumption() {
    let mut instance = make_instance();
    instance.add_cost(CostEntry::new("attr_hp", 30.0));
    instance.add_cost(CostEntry::new("attr_mp", 20.0));

    instance.costs[0].consumed = true;

    assert!(!instance.all_costs_consumed());
}

#[test]
fn all_costs_consumed_returns_false_for_empty_costs() {
    let instance = make_instance();
    assert!(!instance.all_costs_consumed());
}

#[test]
fn multi_resource_consumption_all_or_nothing() {
    let mut instance = make_instance();
    instance.add_cost(CostEntry::new("attr_hp", 30.0));
    instance.add_cost(CostEntry::new("attr_mp", 20.0));

    assert!(instance.costs.iter().all(|c| !c.consumed));

    instance.mark_costs_consumed();
    assert!(instance.costs.iter().all(|c| c.consumed));
}

#[test]
fn insufficient_cost_error_contains_required_fields() {
    let err =
        crate::core::capabilities::ability::foundation::AbilityError::InsufficientCost {
            resource: "attr_mp".to_string(),
            required: 50.0,
            available: 20.0,
        };
    let msg = format!("{}", err);
    assert!(msg.contains("attr_mp"));
    assert!(msg.contains("50"));
    assert!(msg.contains("20"));
}

// ── 不变量 3.1: 激活前 Condition 检查 ───────────────────────

#[test]
fn condition_failed_error_structured() {
    let err =
        crate::core::capabilities::ability::foundation::AbilityError::ConditionFailed {
            reason: "silenced".to_string(),
        };
    let msg = format!("{}", err);
    assert!(msg.contains("silenced"));
}

#[test]
fn ability_state_can_activate_only_ready() {
    use crate::core::capabilities::ability::foundation::types::AbilityState;

    assert!(AbilityState::Ready.can_activate());
    assert!(!AbilityState::Casting.can_activate());
    assert!(!AbilityState::Active.can_activate());
    assert!(!AbilityState::Cooldown.can_activate());
    assert!(!AbilityState::Blocked.can_activate());
    assert!(!AbilityState::Removed.can_activate());
}

// ── 不变量 3.3: 冷却互斥 ───────────────────────────────────

#[test]
fn on_cooldown_error_contains_remaining_turns() {
    let err = crate::core::capabilities::ability::foundation::AbilityError::OnCooldown {
        spec_id: "abl_fireball".to_string(),
        remaining_turns: 3,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("abl_fireball"));
    assert!(msg.contains("3"));
}

#[test]
fn already_active_error_contains_instance_id() {
    let iid = crate::core::capabilities::ability::foundation::types::AbilityInstanceId::new(
        crate::shared::ids::types::runtime_id::RuntimeId::new(42, 0),
    );
    let err = crate::core::capabilities::ability::foundation::AbilityError::AlreadyActive {
        spec_id: "abl_heal".to_string(),
        instance_id: iid,
    };
    let msg = format!("{}", err);
    assert!(msg.contains("abl_heal"));
    assert!(msg.contains("42"));
}

// ── 不变量 3.4: 级联取消 ───────────────────────────────────

#[test]
fn invalid_transition_blocked_state() {
    use crate::core::capabilities::ability::foundation::{AbilityError, types::AbilityState};

    let err = AbilityError::InvalidTransition {
        from: AbilityState::Blocked,
        to: AbilityState::Cooldown,
        reason: "blocked abilities cannot enter cooldown".to_string(),
    };
    let msg = format!("{}", err);
    assert!(msg.contains("Blocked"));
    assert!(msg.contains("Cooldown"));
}

// ── 不变量 3.5: Cost 不可逆 ────────────────────────────────

#[test]
fn cost_marking_is_permanent() {
    let mut instance = make_instance();
    instance.add_cost(CostEntry::new("attr_hp", 30.0));

    instance.mark_costs_consumed();
    assert!(instance.all_costs_consumed());

    for cost in &instance.costs {
        assert!(cost.consumed);
    }
}
