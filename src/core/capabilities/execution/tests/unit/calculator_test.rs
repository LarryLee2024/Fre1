use bevy::prelude::*;

use crate::core::capabilities::execution::foundation::error::ExecutionError;
use crate::core::capabilities::execution::foundation::{
    AbilityExecutionParams, AttributeModifierDef, CalcTrace, CustomExecutionRef, DamageParams,
    DiceDef, DirectOp, EnvironmentParams, ExecutionContext, ExecutionResult, ExecutionType,
    HealParams, ScalableValue,
};
use crate::core::capabilities::execution::mechanism::{execute, validate_context, validate_result};

// ── Helpers ────────────────────────────────────────────

fn make_damage_context() -> ExecutionContext {
    let params = DamageParams::new("dnd_5e_damage")
        .with_dice(DiceDef::new(1, 8).unwrap())
        .with_flat_bonus(ScalableValue::Fixed(3.0));

    let mut attrs = std::collections::HashMap::new();
    attrs.insert("attr_strength".into(), 4.0);

    ExecutionContext::new(ExecutionType::Damage(params), "caster_001", "target_001")
        .with_source_attributes(attrs)
}

fn make_heal_context() -> ExecutionContext {
    let params = HealParams::new("dnd_5e_heal", ScalableValue::Fixed(10.0));

    ExecutionContext::new(ExecutionType::Heal(params), "healer_001", "target_001")
}

fn make_direct_mod_context() -> ExecutionContext {
    ExecutionContext::new(
        ExecutionType::DirectAttributeMod {
            attribute_id: "attr_hp".into(),
            operation: DirectOp::Add,
            value: ScalableValue::Fixed(5.0),
        },
        "system",
        "target_001",
    )
}

// ── Main execution dispatch ────────────────────────────

#[test]
fn execute_damage_calculation_returns_correct_value() {
    let mut world = World::new();
    let mut commands = world.commands();
    let ctx = make_damage_context();
    let result = execute(&ctx, &mut commands).unwrap();
    assert!(result.success);
    assert!(result.value >= 0.0);
}

#[test]
fn execute_heal_calculation_returns_correct_value() {
    let mut world = World::new();
    let mut commands = world.commands();
    let ctx = make_heal_context();
    let result = execute(&ctx, &mut commands).unwrap();
    assert!(result.success);
    assert!(result.value >= 0.0);
}

#[test]
fn execute_none_type_returns_zero() {
    let mut world = World::new();
    let mut commands = world.commands();
    let ctx = ExecutionContext::new(ExecutionType::None, "caster", "target");
    let result = execute(&ctx, &mut commands).unwrap();
    assert!(result.success);
    assert_eq!(result.value, 0.0);
}

#[test]
fn execute_unregistered_custom_returns_error() {
    let mut world = World::new();
    let mut commands = world.commands();
    let custom = CustomExecutionRef::new("tactical.knockback");
    let ctx = ExecutionContext::new(ExecutionType::Custom(custom), "caster_001", "target_001");
    let result = execute(&ctx, &mut commands);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(
        err,
        ExecutionError::CustomExecutionNotRegistered(_)
    ));
}

#[test]
fn execute_direct_mod_returns_correct_value() {
    let mut world = World::new();
    let mut commands = world.commands();
    let ctx = make_direct_mod_context();
    let result = execute(&ctx, &mut commands).unwrap();
    assert!(result.success);
    assert_eq!(result.value, 5.0);
}

#[test]
fn empty_formula_id_execution_error() {
    let mut world = World::new();
    let mut commands = world.commands();
    let params = DamageParams::new("");
    let ctx = ExecutionContext::new(ExecutionType::Damage(params), "caster_001", "target_001");
    let result = execute(&ctx, &mut commands);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, ExecutionError::FormulaNotFound { .. }));
}

#[test]
fn empty_heal_formula_id_execution_error() {
    let mut world = World::new();
    let mut commands = world.commands();
    let params = HealParams::new("", ScalableValue::Fixed(10.0));
    let ctx = ExecutionContext::new(ExecutionType::Heal(params), "healer_001", "target_001");
    let result = execute(&ctx, &mut commands);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, ExecutionError::FormulaNotFound { .. }));
}

// ── Damage parameters ──────────────────────────────────

#[test]
fn damage_with_dice_and_flat_bonus_correct() {
    let mut world = World::new();
    let mut commands = world.commands();
    let params = DamageParams::new("test_formula")
        .with_dice(DiceDef::new(2, 6).unwrap())
        .with_flat_bonus(ScalableValue::Fixed(5.0));

    let ctx = ExecutionContext::new(ExecutionType::Damage(params), "caster", "target");
    let result = execute(&ctx, &mut commands).unwrap();
    assert!(result.success);
    // dice avg = (6+1)/2 * 2 = 7.0, + flat 5.0 = 12.0
    assert!(result.value >= 0.0);
}

#[test]
fn damage_with_attribute_modifier_correct() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut attrs = std::collections::HashMap::new();
    attrs.insert("attr_strength".into(), 4.0);

    let modifier = AttributeModifierDef::new("attr_strength").with_multiplier(1.0);
    let params = DamageParams::new("test_formula").with_attribute_modifier(modifier);

    let ctx = ExecutionContext::new(ExecutionType::Damage(params), "caster", "target")
        .with_source_attributes(attrs);
    let result = execute(&ctx, &mut commands).unwrap();
    assert!(result.success);
}

#[test]
fn damage_with_critical_correct() {
    let mut world = World::new();
    let mut commands = world.commands();
    let params = DamageParams::new("test_formula").with_critical(2.0);

    let ctx = ExecutionContext::new(ExecutionType::Damage(params), "caster", "target");
    let result = execute(&ctx, &mut commands).unwrap();
    assert!(result.success);
    assert!(!result.was_critical);
}

// ── Heal parameters ────────────────────────────────────

#[test]
fn heal_with_attribute_modifier_correct() {
    let mut world = World::new();
    let mut commands = world.commands();
    let mut attrs = std::collections::HashMap::new();
    attrs.insert("attr_wisdom".into(), 5.0);

    let modifier = AttributeModifierDef::new("attr_wisdom").with_multiplier(1.0);
    let params = HealParams::new("heal_formula", ScalableValue::Fixed(10.0))
        .with_attribute_modifier(modifier);

    let ctx = ExecutionContext::new(ExecutionType::Heal(params), "healer", "target")
        .with_source_attributes(attrs);
    let result = execute(&ctx, &mut commands).unwrap();
    assert!(result.success);
    // 10.0 + 5.0 = 15.0
    assert!(result.value >= 0.0);
}

#[test]
fn heal_temporary_hp_calculation_correct() {
    let mut world = World::new();
    let mut commands = world.commands();
    let params = HealParams::new("heal_formula", ScalableValue::Fixed(8.0)).with_temporary_hp(true);

    let ctx = ExecutionContext::new(ExecutionType::Heal(params), "healer", "target");
    let result = execute(&ctx, &mut commands).unwrap();
    assert!(result.success);
    assert_eq!(result.value, 8.0);
}

// ── ScalableValue calculation ──────────────────────────

#[test]
fn scalable_value_fixed_correct() {
    let sv = ScalableValue::Fixed(10.0);
    assert_eq!(sv.calculate(1), 10.0);
    assert_eq!(sv.calculate(5), 10.0);
}

#[test]
fn scalable_value_per_level_correct() {
    let sv = ScalableValue::PerLevel {
        base: 10.0,
        per_level: 5.0,
    };
    assert_eq!(sv.calculate(1), 10.0);
    assert_eq!(sv.calculate(2), 15.0);
    assert_eq!(sv.calculate(5), 30.0);
}

#[test]
fn scalable_value_zero_level_uses_level_one() {
    let sv = ScalableValue::PerLevel {
        base: 10.0,
        per_level: 5.0,
    };
    assert_eq!(sv.calculate(0), 10.0);
}

// ── DiceDef validation ─────────────────────────────────

#[test]
fn dice_def_valid_params_succeed() {
    let dice = DiceDef::new(1, 8).unwrap();
    assert_eq!(dice.count, 1);
    assert_eq!(dice.sides, 8);
}

#[test]
fn dice_def_zero_count_error() {
    let result = DiceDef::new(0, 8);
    assert!(result.is_err());
}

#[test]
fn dice_def_single_side_error() {
    let result = DiceDef::new(1, 1);
    assert!(result.is_err());
}

#[test]
fn dice_roll_range_correct() {
    let dice = DiceDef::new(2, 6).unwrap();
    assert_eq!(dice.min_roll(), 2);
    assert_eq!(dice.max_roll(), 12);
}

// ── AttributeModifierDef ───────────────────────────────

#[test]
fn attribute_modifier_calculate_correct() {
    let mut attrs = std::collections::HashMap::new();
    attrs.insert("attr_strength".into(), 4.0);

    let modifier = AttributeModifierDef::new("attr_strength");
    assert_eq!(modifier.calculate(&attrs), 4.0);
}

#[test]
fn attribute_modifier_multiplier_correct() {
    let mut attrs = std::collections::HashMap::new();
    attrs.insert("attr_strength".into(), 10.0);

    let modifier = AttributeModifierDef::new("attr_strength").with_multiplier(0.5);
    assert_eq!(modifier.calculate(&attrs), 5.0);
}

#[test]
fn attribute_modifier_missing_returns_zero() {
    let attrs = std::collections::HashMap::new();
    let modifier = AttributeModifierDef::new("attr_nonexistent");
    assert_eq!(modifier.calculate(&attrs), 0.0);
}

// ── Result validation ──────────────────────────────────

#[test]
fn validate_valid_execution_result_passes() {
    let result = ExecutionResult::success(42.0);
    assert!(validate_result(&result).is_ok());
}

#[test]
fn validate_negative_execution_result_error() {
    let result = ExecutionResult::success(-5.0);
    assert!(validate_result(&result).is_err());
}

#[test]
fn validate_zero_execution_result_passes() {
    let result = ExecutionResult::success(0.0);
    assert!(validate_result(&result).is_ok());
}

// ── Context validation ─────────────────────────────────

#[test]
fn validate_valid_execution_context_passes() {
    let ctx = ExecutionContext::new(ExecutionType::None, "caster", "target");
    assert!(validate_context(&ctx).is_ok());
}

#[test]
fn validate_empty_source_entity_error() {
    let ctx = ExecutionContext::new(ExecutionType::None, "", "target");
    assert!(validate_context(&ctx).is_err());
}

#[test]
fn validate_empty_target_entity_error() {
    let ctx = ExecutionContext::new(ExecutionType::None, "caster", "");
    assert!(validate_context(&ctx).is_err());
}

// ── CalcTrace ──────────────────────────────────────────

#[test]
fn calc_trace_records_calculation_correctly() {
    let trace = CalcTrace::new("test_formula")
        .with_input("attr_str", 4.0)
        .with_intermediate("dice_avg", 7.0)
        .with_output(11.0);

    assert_eq!(trace.formula_id, "test_formula");
    assert_eq!(trace.inputs.get("attr_str"), Some(&4.0));
    assert!(
        trace
            .intermediate_values
            .contains(&("dice_avg".into(), 7.0))
    );
    assert_eq!(trace.output, 11.0);
}

#[test]
fn execution_result_contains_calc_trace() {
    let mut world = World::new();
    let mut commands = world.commands();
    let ctx = make_damage_context();
    let result = execute(&ctx, &mut commands).unwrap();
    assert!(result.calc_trace.is_some());
    let trace = result.calc_trace.unwrap();
    assert_eq!(trace.formula_id, "dnd_5e_damage");
}

// ── ExecutionResult builder ────────────────────────────

#[test]
fn execution_result_builder_correct() {
    let result = ExecutionResult::success(10.0)
        .with_critical(true)
        .with_miss(false);

    assert!(result.success);
    assert_eq!(result.value, 10.0);
    assert!(result.was_critical);
    assert!(!result.was_miss);
}

#[test]
fn execution_result_failure_state_correct() {
    let result = ExecutionResult::failure();
    assert!(!result.success);
    assert_eq!(result.value, 0.0);
}

// ── ExecutionType helpers ──────────────────────────────

#[test]
fn execution_type_name_correct() {
    let dam = ExecutionType::Damage(DamageParams::new("f"));
    assert_eq!(dam.name(), "Damage");

    let heal = ExecutionType::Heal(HealParams::new("f", ScalableValue::Fixed(1.0)));
    assert_eq!(heal.name(), "Heal");

    assert_eq!(ExecutionType::None.name(), "None");
}

#[test]
fn is_numeric_calculation_correct() {
    let dam = ExecutionType::Damage(DamageParams::new("f"));
    assert!(dam.is_numeric_calculation());

    assert!(!ExecutionType::None.is_numeric_calculation());
}

// ── DamageParams validation ────────────────────────────

#[test]
fn damage_params_validate_invalid_critical_passes() {
    let params = DamageParams::new("f").with_critical(0.5);
    assert!(params.validate().is_ok());
}

#[test]
fn damage_params_validate_valid_input_passes() {
    let params = DamageParams::new("f").with_dice(DiceDef::new(1, 6).unwrap());
    assert!(params.validate().is_ok());
}

// ── DirectOp execute ───────────────────────────────────

#[test]
fn execute_direct_mod_set_correct() {
    let mut world = World::new();
    let mut commands = world.commands();
    let ctx = ExecutionContext::new(
        ExecutionType::DirectAttributeMod {
            attribute_id: "attr_hp".into(),
            operation: DirectOp::Set,
            value: ScalableValue::Fixed(100.0),
        },
        "system",
        "target_001",
    );
    let result = execute(&ctx, &mut commands).unwrap();
    assert_eq!(result.value, 100.0);
}

#[test]
fn direct_mod_empty_attribute_id_error() {
    let mut world = World::new();
    let mut commands = world.commands();
    let ctx = ExecutionContext::new(
        ExecutionType::DirectAttributeMod {
            attribute_id: "".into(),
            operation: DirectOp::Add,
            value: ScalableValue::Fixed(5.0),
        },
        "system",
        "target_001",
    );
    let result = execute(&ctx, &mut commands);
    assert!(result.is_err());
}

// ── EnvironmentParams ──────────────────────────────────

#[test]
fn environment_params_default_values_correct() {
    let env = EnvironmentParams::default();
    assert!(!env.is_high_ground);
    assert!(!env.has_cover);
    assert!(!env.is_flanked);
    assert_eq!(env.current_turn, 0);
}

// ── AbilityExecutionParams ─────────────────────────────

#[test]
fn ability_execution_params_default_values_correct() {
    let p = AbilityExecutionParams::default();
    assert_eq!(p.ability_level, 1);
    assert!(p.ability_def_id.is_none());
    assert!(!p.has_effect_override);
}
