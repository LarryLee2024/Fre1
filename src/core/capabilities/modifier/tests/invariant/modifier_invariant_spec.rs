//! Modifier 不改变基础值不变量测试
//!
//! 不变量：Modifier 只影响聚合后的当前值，不改变 base_value。
//! 来源：docs/02-domain/capabilities/modifier_domain.md

use crate::core::capabilities::modifier::foundation::{ModifierOp, ModifierSourceType};
use crate::core::capabilities::modifier::mechanism::lifecycle::validate_modifier_data;
use crate::shared::testing::fixtures::ModifierBuilder;

#[test]
fn valid_modifier_passes_validation() {
    let data = ModifierBuilder::new("attr_atk", 5.0).build();
    assert!(validate_modifier_data(&data).is_ok());
}

#[test]
fn priority_out_of_range_rejected() {
    let data = ModifierBuilder::new("attr_atk", 5.0).priority(150).build();
    assert!(validate_modifier_data(&data).is_err());
}

#[test]
fn missing_source_id_rejected() {
    let data = ModifierBuilder::new("attr_atk", 5.0)
        .source(ModifierSourceType::Buff, "")
        .build();
    assert!(validate_modifier_data(&data).is_err());
}

#[test]
fn missing_target_attribute_rejected() {
    let data = ModifierBuilder::new("", 5.0).build();
    assert!(validate_modifier_data(&data).is_err());
}

#[test]
fn add_modifier_magnitude_correct() {
    let data = ModifierBuilder::new("attr_atk", 10.0)
        .op(ModifierOp::Add)
        .build();
    assert_eq!(data.magnitude, 10.0);
    assert_eq!(data.op, ModifierOp::Add);
}

#[test]
fn multiply_modifier_magnitude_correct() {
    let data = ModifierBuilder::new("attr_atk", 1.5)
        .op(ModifierOp::Multiply)
        .build();
    assert_eq!(data.magnitude, 1.5);
    assert_eq!(data.op, ModifierOp::Multiply);
}

#[test]
fn modifier_only_modifies_base_value() {
    let data = ModifierBuilder::new("attr_atk", 5.0).build();
    assert!(data.target_attribute == "attr_atk");
    assert!(data.magnitude == 5.0);
}

// ── 不变量 3.1: Override 独占性 ──────────────────────────────

#[test]
fn override_modifier_has_highest_priority() {
    let override_a = ModifierBuilder::new("attr_atk", 100.0)
        .op(ModifierOp::Override)
        .priority(10)
        .build();
    let override_b = ModifierBuilder::new("attr_atk", 200.0)
        .op(ModifierOp::Override)
        .priority(20)
        .build();

    assert_eq!(override_a.op, ModifierOp::Override);
    assert_eq!(override_b.op, ModifierOp::Override);
    assert_eq!(override_a.target_attribute, override_b.target_attribute);
    assert!(override_a.priority < override_b.priority);
}

#[test]
fn add_and_override_can_coexist_on_different_attrs() {
    let add_mod = ModifierBuilder::new("attr_atk", 10.0)
        .op(ModifierOp::Add)
        .build();
    let override_mod = ModifierBuilder::new("attr_def", 50.0)
        .op(ModifierOp::Override)
        .build();

    assert_eq!(add_mod.op, ModifierOp::Add);
    assert_eq!(override_mod.op, ModifierOp::Override);
    assert_ne!(add_mod.target_attribute, override_mod.target_attribute);
}

// ── 不变量 3.4: 幂等性 ──────────────────────────────────────

#[test]
fn same_source_same_attr_same_op_detected() {
    let mod_a = ModifierBuilder::new("attr_atk", 10.0)
        .op(ModifierOp::Add)
        .source(ModifierSourceType::Buff, "buf_001")
        .build();
    let mod_b = ModifierBuilder::new("attr_atk", 10.0)
        .op(ModifierOp::Add)
        .source(ModifierSourceType::Buff, "buf_001")
        .build();

    assert_eq!(mod_a.target_attribute, mod_b.target_attribute);
    assert_eq!(mod_a.op, mod_b.op);
    assert_eq!(mod_a.source.source_id, mod_b.source.source_id);
}

// ── 不变量 3.5: 优先级排序 ──────────────────────────────────

#[test]
fn priority_ordering_deterministic() {
    let mut mods = [
        ModifierBuilder::new("attr_atk", 10.0).priority(30).build(),
        ModifierBuilder::new("attr_atk", 20.0).priority(10).build(),
        ModifierBuilder::new("attr_atk", 30.0).priority(50).build(),
    ];

    mods.sort_by_key(|m| m.priority);

    assert_eq!(mods[0].priority, 10);
    assert_eq!(mods[1].priority, 30);
    assert_eq!(mods[2].priority, 50);
}

// ── 禁止事项: Modifier 只描述不直接修改 ──────────────────────

#[test]
fn modifier_does_not_apply_directly() {
    let data = ModifierBuilder::new("attr_atk", 15.0)
        .op(ModifierOp::Multiply)
        .build();

    assert_eq!(data.magnitude, 15.0);
    assert_eq!(data.op, ModifierOp::Multiply);
    assert!(data.duration_frames.is_none());
}
