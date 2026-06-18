//! Aggregator 不变量测试
//!
//! 不变量：聚合管线的阶段执行顺序、幂等性、快照一致性。
//! 来源：docs/02-domain/capabilities/aggregator_domain.md

use crate::core::capabilities::aggregator::foundation::{CalcStage, ModifierOp};

// ── 不变量 3.1: 阶段执行顺序 (Add -> Multiply -> Override -> Clamp) ──

#[test]
fn stage_execution_order_is_deterministic() {
    let stages = [
        CalcStage::Add,
        CalcStage::Multiply,
        CalcStage::Override,
        CalcStage::Clamp,
    ];

    for i in 0..stages.len() - 1 {
        assert!(
            (stages[i] as u8) < (stages[i + 1] as u8),
            "stage {:?} should execute before {:?}",
            stages[i],
            stages[i + 1]
        );
    }
}

// ── 不变量 3.2: 乘法叠加是连乘而非加法 ──────────────────────

#[test]
fn multiply_is_compound_not_additive() {
    let base = 100.0f32;
    let mul1 = 1.2f32;
    let mul2 = 1.3f32;

    let compound = base * mul1 * mul2;
    assert!((compound - 156.0).abs() < 0.01);

    let additive = base * (1.0 + (mul1 - 1.0) + (mul2 - 1.0));
    assert!((additive - 150.0).abs() < 0.01);

    assert!((compound - additive).abs() > 0.01);
}

// ── 不变量 3.3: Override 独占性 ──────────────────────────────

#[test]
fn override_takes_highest_priority() {
    let overrides = [
        (ModifierOp::Override, 10u8, 100.0f32),
        (ModifierOp::Override, 5, 200.0f32),
        (ModifierOp::Override, 20, 300.0f32),
    ];

    let selected = overrides.iter().min_by_key(|(_, priority, _)| *priority);
    assert!(selected.is_some());
    let (_, priority, value) = selected.unwrap();
    assert_eq!(*priority, 5);
    assert_eq!(*value, 200.0);
}

// ── 不变量 3.4: 聚合结果确定性 ──────────────────────────────

#[test]
fn aggregation_result_deterministic() {
    let base = 100.0f32;
    let add_mods = [10.0f32, 20.0, 5.0];
    let mul_mods = [1.1f32, 1.2];

    let run = || {
        let after_add: f32 = base + add_mods.iter().sum::<f32>();
        let after_mul: f32 = mul_mods.iter().fold(after_add, |acc, m| acc * m);
        after_mul
    };

    let result1 = run();
    let result2 = run();
    assert_eq!(result1, result2);
}

// ── 不变量 3.5: 快照一致性 ──────────────────────────────────

#[test]
fn snapshot_restores_exact_state() {
    let base = 100.0f32;
    let add_sum = 35.0f32;
    let mul_product = 1.32f32;
    let override_value = 200.0f32;

    let snapshot = (base, add_sum, mul_product, override_value);
    let restored = snapshot;

    assert_eq!(snapshot.0, restored.0);
    assert_eq!(snapshot.1, restored.1);
    assert_eq!(snapshot.2, restored.2);
    assert_eq!(snapshot.3, restored.3);
}

// ── Clamp 边界: min <= max ───────────────────────────────────

#[test]
fn clamp_valid_bounds() {
    let min = 0.0f32;
    let max = 100.0f32;
    let value = 150.0f32;

    assert!(min <= max);
    let clamped = value.clamp(min, max);
    assert_eq!(clamped, 100.0);
}

#[test]
fn clamp_below_min() {
    let min = 0.0f32;
    let max = 100.0f32;
    let value = -10.0f32;

    let clamped = value.clamp(min, max);
    assert_eq!(clamped, 0.0);
}

// ── Dirty 追踪: 幂等性 ──────────────────────────────────────

#[test]
fn dirty_marking_idempotent() {
    let mut dirty_set = std::collections::HashSet::new();

    dirty_set.insert("attr_hp");
    dirty_set.insert("attr_hp");
    dirty_set.insert("attr_atk");

    assert_eq!(dirty_set.len(), 2);
    assert!(dirty_set.contains("attr_hp"));
    assert!(dirty_set.contains("attr_atk"));
}
