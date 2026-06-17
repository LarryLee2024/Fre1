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

#[cfg(test)]
mod tests {
    use crate::core::capabilities::ability::foundation::types::{AbilityState, ActivationType};
    use crate::core::capabilities::ability::foundation::values::{
        AbilityInstance, ActivationContext, CostEntry,
    };

    fn make_instance() -> AbilityInstance {
        AbilityInstance::new(
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

        // 只标记第一个消耗
        instance.costs[0].consumed = true;

        assert!(!instance.all_costs_consumed());
    }

    #[test]
    fn all_costs_consumed_returns_false_for_empty_costs() {
        let instance = make_instance();
        // costs 为空，all_costs_consumed 应返回 false
        assert!(!instance.all_costs_consumed());
    }

    #[test]
    fn multi_resource_consumption_all_or_nothing() {
        let mut instance = make_instance();
        instance.add_cost(CostEntry::new("attr_hp", 30.0));
        instance.add_cost(CostEntry::new("attr_mp", 20.0));

        // 消耗前：全部未完成
        assert!(instance.costs.iter().all(|c| !c.consumed));

        // 原子性标记后：全部完成
        instance.mark_costs_consumed();
        assert!(instance.costs.iter().all(|c| c.consumed));
    }

    #[test]
    fn insufficient_cost_error_contains_required_fields() {
        let err = crate::core::capabilities::ability::foundation::types::AbilityError::InsufficientCost {
            resource: "attr_mp".to_string(),
            required: 50.0,
            available: 20.0,
        };
        let msg = format!("{}", err);
        assert!(msg.contains("attr_mp"));
        assert!(msg.contains("50"));
        assert!(msg.contains("20"));
    }
}
