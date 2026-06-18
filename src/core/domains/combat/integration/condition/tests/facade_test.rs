use std::collections::HashMap;

use crate::core::capabilities::condition::foundation::ConditionContext;
use crate::core::domains::combat::integration::condition::CombatConditionFacade;

#[test]
fn immunity_check_passed_when_no_immune_tag() {
    let context = ConditionContext {
        tag_ids: Some(vec!["Fire".to_string()]),
        attribute_values: HashMap::new(),
    };
    let result = CombatConditionFacade::check_effect_immunity(&context, "Fire");
    assert!(result.is_passed());
}

#[test]
fn immunity_check_failed_when_immune_tag_present() {
    let context = ConditionContext {
        tag_ids: Some(vec!["Immune.Fire".to_string()]),
        attribute_values: HashMap::new(),
    };
    let result = CombatConditionFacade::check_effect_immunity(&context, "Fire");
    assert!(!result.is_passed());
}
