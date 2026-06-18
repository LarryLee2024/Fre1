//! HP >= 0 不变量测试
//!
//! 不变量：Resource 属性（如 HP）的 CurrentValue 必须 >= 0。
//! 来源：docs/02-domain/capabilities/attribute_domain.md §3.2, §3.3
//!
//! 验证：
//! 1. Resource 属性注册时 min_value 必须 >= 0
//! 2. Aggregator Clamp 阶段将值限制在 [min, max] 范围内
//! 3. HP 的 min_value 为 0，因此 Clamp 后 HP >= 0

use crate::core::capabilities::attribute::foundation::{AttributeCategory, AttributeId};
use crate::core::capabilities::attribute::mechanism::lifecycle::{
    AttributeRegistrationError, AttributeRegistry,
};
use crate::shared::testing::fixtures::{AttributeDefBuilder, attributes_for_unit_001};

#[test]
fn resource_attr_min_cannot_be_negative() {
    let mut reg = AttributeRegistry::default();
    let result = reg.register(
        AttributeDefBuilder::new("attr_hp")
            .category(AttributeCategory::Resource)
            .default_value(100.0)
            .range(-10.0, 100.0)
            .build(),
    );
    assert!(matches!(
        result,
        Err(AttributeRegistrationError::ResourceMinBelowZero(_))
    ));
}

#[test]
fn resource_attr_min_zero_registration_succeeds() {
    let mut reg = AttributeRegistry::default();
    let result = reg.register(
        AttributeDefBuilder::new("attr_hp")
            .category(AttributeCategory::Resource)
            .default_value(100.0)
            .range(0.0, 100.0)
            .build(),
    );
    assert!(result.is_ok());
}

#[test]
fn hp_attr_min_value_is_zero() {
    let attrs = attributes_for_unit_001();
    let hp = attrs
        .iter()
        .find(|a| a.id == AttributeId::new("attr_hp"))
        .unwrap();
    assert_eq!(hp.min_value, 0.0);
    assert_eq!(hp.category, AttributeCategory::Resource);
}

#[test]
fn hp_attr_max_equals_default() {
    let attrs = attributes_for_unit_001();
    let hp = attrs
        .iter()
        .find(|a| a.id == AttributeId::new("attr_hp"))
        .unwrap();
    assert_eq!(hp.default_base_value, 100.0);
    assert_eq!(hp.max_value, 100.0);
}

#[test]
fn primary_attr_allows_negative_min() {
    let mut reg = AttributeRegistry::default();
    let result = reg.register(
        AttributeDefBuilder::new("attr_test")
            .category(AttributeCategory::Primary)
            .default_value(0.0)
            .range(-100.0, 100.0)
            .build(),
    );
    assert!(result.is_ok());
}

#[test]
fn default_value_out_of_range_fails_registration() {
    let mut reg = AttributeRegistry::default();
    let result = reg.register(
        AttributeDefBuilder::new("attr_hp")
            .category(AttributeCategory::Resource)
            .default_value(200.0)
            .range(0.0, 100.0)
            .build(),
    );
    assert!(matches!(
        result,
        Err(AttributeRegistrationError::DefaultValueOutOfRange { .. })
    ));
}
