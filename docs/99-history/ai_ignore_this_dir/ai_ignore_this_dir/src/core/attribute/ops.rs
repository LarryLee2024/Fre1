//! 属性算术操作与边界限制（ADR-031 §1.4）
//!
//! 提供安全的 i32 属性运算，所有结果钳制到属性定义的边界内。
//! 百分比以万分比表示（如 50% = 5000）。

use crate::shared::ids::AttributeId;
use crate::shared::registry::Registry;
use std::collections::HashMap;

use super::AttributeRegistry;
use super::def::AttributeDefinition;

// ============================================================================
// 边界查找
// ============================================================================

/// 从注册表中查找属性的边界信息，返回 (min, max)
///
/// 如果属性不在注册表中，返回 (i32::MIN, i32::MAX)（无限制）
pub fn attribute_boundaries(registry: &AttributeRegistry, attr_id: &AttributeId) -> (i32, i32) {
    match registry.get(attr_id) {
        Some(def) => (def.min, def.max),
        None => (i32::MIN, i32::MAX),
    }
}

/// 将值钳制到属性定义的 [min, max] 区间
pub fn clamp_attribute(registry: &AttributeRegistry, attr_id: &AttributeId, value: i32) -> i32 {
    match registry.get(attr_id) {
        Some(def) => def.clamp(value),
        None => value,
    }
}

// ============================================================================
// 安全运算
// ============================================================================

/// 带边界检查的加法：value + delta，结果 clamp 到 [min, max]
pub fn safe_add(value: i32, delta: i32, min: i32, max: i32) -> i32 {
    value.saturating_add(delta).clamp(min, max)
}

/// 带边界检查的减法：value - delta，结果 clamp 到 [min, max]
pub fn safe_sub(value: i32, delta: i32, min: i32, max: i32) -> i32 {
    value.saturating_sub(delta).clamp(min, max)
}

/// 带边界检查的乘法：value * multiplier / 10000（multiplier 为万分比）
///
/// 如 multiplier = 5000 表示乘以 50%。
/// 使用 i64 中间计算避免溢出。
pub fn safe_mul_percent(value: i32, multiplier: i32) -> i32 {
    let result = (value as i64) * (multiplier as i64) / 10000;
    result as i32
}

/// 应用加法修饰符列表到基础值
///
/// 所有加法修饰符的和加到基础值上，然后 clamp 到 [min, max]。
pub fn apply_add_modifiers(base: i32, modifiers: &[i32], min: i32, max: i32) -> i32 {
    let sum: i32 = modifiers.iter().copied().sum();
    base.saturating_add(sum).clamp(min, max)
}

/// 应用乘法修饰符列表到基础值
///
/// 所有乘法修饰符（万分比）连乘，再乘以基础值。
/// 如 modifiers = [5000, 20000] 表示 ×50% ×200% = ×100%
pub fn apply_mul_modifiers(base: i32, modifiers: &[i32]) -> i32 {
    let mut product: i64 = 10000; // 10000 = ×100%
    for &m in modifiers {
        product = product * (m as i64) / 10000;
    }
    ((base as i64) * product / 10000) as i32
}

/// 计算两个值的百分比差值（万分比）
///
/// 返回 (value - base) / base * 10000（万分比）
/// 如 base=100, value=150 → 返回 5000（表示增加了 50%）
pub fn percent_diff(value: i32, base: i32) -> i32 {
    if base == 0 {
        return 0;
    }
    let diff = (value as i64) - (base as i64);
    ((diff * 10000) / (base as i64)) as i32
}

// ============================================================================
// 批量计算
// ============================================================================

/// 属性快照：按注册表中的边界对所有属性进行 clamp
pub fn clamp_all_attributes(
    registry: &AttributeRegistry,
    values: &HashMap<AttributeId, i32>,
) -> HashMap<AttributeId, i32> {
    let mut result = HashMap::new();
    for (id, value) in values {
        let clamped = clamp_attribute(registry, id, *value);
        result.insert(id.clone(), clamped);
    }
    result
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_add_within_bounds() {
        assert_eq!(safe_add(50, 30, 0, 100), 80);
    }

    #[test]
    fn safe_add_clamps_to_max() {
        assert_eq!(safe_add(80, 30, 0, 100), 100);
    }

    #[test]
    fn safe_add_clamps_to_min() {
        assert_eq!(safe_add(10, -20, 0, 100), 0);
    }

    #[test]
    fn safe_sub_within_bounds() {
        assert_eq!(safe_sub(50, 20, 0, 100), 30);
    }

    #[test]
    fn safe_sub_clamps_to_min() {
        assert_eq!(safe_sub(10, 20, 0, 100), 0);
    }

    #[test]
    fn safe_mul_percent_half() {
        // 5000 = 50%
        assert_eq!(safe_mul_percent(100, 5000), 50);
    }

    #[test]
    fn safe_mul_percent_double() {
        // 20000 = 200%
        assert_eq!(safe_mul_percent(100, 20000), 200);
    }

    #[test]
    fn safe_mul_percent_full() {
        // 10000 = 100%
        assert_eq!(safe_mul_percent(100, 10000), 100);
    }

    #[test]
    fn safe_mul_percent_zero() {
        assert_eq!(safe_mul_percent(100, 0), 0);
    }

    #[test]
    fn apply_add_modifiers_sum() {
        let result = apply_add_modifiers(50, &[10, 20, -5], 0, 100);
        assert_eq!(result, 75);
    }

    #[test]
    fn apply_add_modifiers_clamp() {
        let result = apply_add_modifiers(90, &[20], 0, 100);
        assert_eq!(result, 100);
    }

    #[test]
    fn apply_mul_modifiers_single() {
        // 5000 = 50%
        assert_eq!(apply_mul_modifiers(100, &[5000]), 50);
    }

    #[test]
    fn apply_mul_modifiers_compound() {
        // 20000 = 200%, 5000 = 50% → total = 100%
        assert_eq!(apply_mul_modifiers(100, &[20000, 5000]), 100);
    }

    #[test]
    fn percent_diff_positive() {
        assert_eq!(percent_diff(150, 100), 5000); // +50%
    }

    #[test]
    fn percent_diff_negative() {
        assert_eq!(percent_diff(75, 100), -2500); // -25%
    }

    #[test]
    fn percent_diff_zero_base() {
        assert_eq!(percent_diff(50, 0), 0);
    }

    #[test]
    fn clamp_all_attributes_emptymap() {
        let registry = crate::core::attribute::AttributeRegistry::default();
        let result = clamp_all_attributes(&registry, &HashMap::new());
        assert!(result.is_empty());
    }
}
