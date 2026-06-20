//! CombatExecutionFacade 测试

use bevy::prelude::*;

use crate::core::capabilities::execution::foundation::error::ExecutionError;
use crate::core::capabilities::execution::foundation::{
    DamageParams, ExecutionContext, ExecutionType, ScalableValue,
};
use crate::core::domains::combat::integration::execution::CombatExecutionFacade;

fn make_test_damage_params() -> DamageParams {
    DamageParams {
        formula_id: "physical_damage".to_string(),
        damage_type: vec!["physical".to_string()],
        damage_dice: None,
        flat_bonus: Some(ScalableValue::Fixed(30.0)),
        attribute_modifier: None,
        can_critical: false,
        critical_multiplier: 1.0,
    }
}

#[test]
fn build_damage_context_has_attack_defense() {
    let ctx = CombatExecutionFacade::build_damage_context(
        "source_1",
        "target_1",
        make_test_damage_params(),
        100.0,
        50.0,
    );
    assert_eq!(ctx.source_entity, "source_1");
    assert_eq!(ctx.target_entity, "target_1");
    assert!(CombatExecutionFacade::validate_context(&ctx).is_ok());
}

#[test]
fn execute_empty_formula_fails() {
    let mut world = World::new();
    let mut commands = world.commands();
    let params = DamageParams {
        formula_id: "".to_string(),
        damage_type: vec!["physical".to_string()],
        damage_dice: None,
        flat_bonus: None,
        attribute_modifier: None,
        can_critical: false,
        critical_multiplier: 1.0,
    };
    let ctx = ExecutionContext::new(ExecutionType::Damage(params), "src", "tgt");
    let result = CombatExecutionFacade::execute(&ctx, &mut commands);
    assert!(matches!(
        result,
        Err(ExecutionError::FormulaNotFound { .. })
    ));
}
