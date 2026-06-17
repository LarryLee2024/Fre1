//! Attribute 注册不变量测试
//!
//! 不变量：属性注册必须满足唯一性、范围合法性、依赖完整性。
//! 来源：docs/02-domain/capabilities/attribute_domain.md

#[cfg(test)]
mod tests {
    use crate::core::capabilities::attribute::foundation::{AttributeCategory, AttributeId};
    use crate::core::capabilities::attribute::mechanism::lifecycle::{
        AttributeRegistry, AttributeRegistrationError,
    };
    use crate::shared::testing::fixtures::{attributes_for_unit_001, AttributeDefBuilder};

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
        assert_eq!(reg.definitions.len(), 5);
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
}
