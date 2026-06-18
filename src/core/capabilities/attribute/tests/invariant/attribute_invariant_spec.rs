//! Attribute 注册不变量测试
//!
//! 不变量：属性注册必须满足唯一性、范围合法性、依赖完整性。
//! 来源：docs/02-domain/capabilities/attribute_domain.md

#[cfg(test)]
mod tests {
    use crate::core::capabilities::attribute::foundation::{AttributeCategory, AttributeId};
    use crate::core::capabilities::attribute::mechanism::lifecycle::{
        AttributeRegistrationError, AttributeRegistry,
    };
    use crate::shared::testing::fixtures::{AttributeDefBuilder, attributes_for_unit_001};

    #[test]
    fn attribute_ids_globally_unique() {
        let mut reg = AttributeRegistry::default();
        reg.register(
            AttributeDefBuilder::new("attr_hp")
                .category(AttributeCategory::Resource)
                .default_value(100.0)
                .range(0.0, 100.0)
                .build(),
        )
        .unwrap();
        let result = reg.register(
            AttributeDefBuilder::new("attr_hp")
                .category(AttributeCategory::Resource)
                .default_value(100.0)
                .range(0.0, 100.0)
                .build(),
        );
        assert!(matches!(
            result,
            Err(AttributeRegistrationError::DuplicateId(_))
        ));
    }

    #[test]
    fn default_value_must_be_in_range() {
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

    #[test]
    fn resource_attr_min_cannot_be_negative() {
        let mut reg = AttributeRegistry::default();
        let result = reg.register(
            AttributeDefBuilder::new("attr_hp")
                .category(AttributeCategory::Resource)
                .default_value(50.0)
                .range(-10.0, 100.0)
                .build(),
        );
        assert!(matches!(
            result,
            Err(AttributeRegistrationError::ResourceMinBelowZero(_))
        ));
    }

    #[test]
    fn derived_dependency_must_be_registered() {
        let mut reg = AttributeRegistry::default();
        let result = reg.register(
            AttributeDefBuilder::new("attr_dmg")
                .category(AttributeCategory::Derived)
                .default_value(50.0)
                .range(0.0, 100.0)
                .depends_on("attr_atk")
                .build(),
        );
        assert!(matches!(
            result,
            Err(AttributeRegistrationError::DerivedDependencyNotFound { .. })
        ));
    }

    #[test]
    fn batch_register_standard_attributes_succeeds() {
        let mut reg = AttributeRegistry::default();
        let results = reg.register_batch(attributes_for_unit_001());
        assert!(results.iter().all(|r| r.is_ok()));
        assert!(reg.contains(&AttributeId::new("attr_hp")));
        assert!(reg.contains(&AttributeId::new("attr_atk")));
        assert!(reg.contains(&AttributeId::new("attr_def")));
        assert!(reg.contains(&AttributeId::new("attr_spd")));
        assert!(reg.contains(&AttributeId::new("attr_range")));
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

    // ── 不变量 3.1: 基础值不可变性（运行时） ─────────────────────

    #[test]
    fn base_value_is_set_at_creation() {
        let def = AttributeDefBuilder::new("attr_hp")
            .category(AttributeCategory::Resource)
            .default_value(100.0)
            .range(0.0, 100.0)
            .build();
        assert_eq!(def.default_base_value, 100.0);
    }

    #[test]
    fn attribute_value_base_and_current_separated() {
        use crate::core::capabilities::attribute::foundation::{AttributeId, AttributeValue};

        let attr = AttributeValue {
            def_id: AttributeId::new("attr_hp"),
            base_value: 100.0,
            current_value: 80.0,
            aggregator_managed: true,
        };

        assert_eq!(attr.base_value, 100.0);
        assert_eq!(attr.current_value, 80.0);
        assert_ne!(attr.base_value, attr.current_value);
    }

    // ── 不变量 3.2: 当前值不能越界 ──────────────────────────────

    #[test]
    fn definition_range_enforced_at_registration() {
        let mut reg = AttributeRegistry::default();
        // default_value=150 超出 range [0, 100]，注册失败
        let result = reg.register(
            AttributeDefBuilder::new("attr_hp")
                .category(AttributeCategory::Resource)
                .default_value(150.0)
                .range(0.0, 100.0)
                .build(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn definition_default_within_range_succeeds() {
        let mut reg = AttributeRegistry::default();
        let result = reg.register(
            AttributeDefBuilder::new("attr_hp")
                .category(AttributeCategory::Resource)
                .default_value(50.0)
                .range(0.0, 100.0)
                .build(),
        );
        assert!(result.is_ok());
    }
}
