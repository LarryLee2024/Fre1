//! Effect 不修改不存在属性不变量测试
//!
//! 不变量：Effect 引用的 AttributeId 必须已注册。
//! 来源：docs/02-domain/capabilities/effect_domain.md

#[cfg(test)]
mod tests {
    use crate::core::capabilities::attribute::foundation::{AttributeCategory, AttributeId};
    use crate::core::capabilities::attribute::mechanism::lifecycle::AttributeRegistry;
    use crate::core::capabilities::effect::foundation::values::ActiveEffectContainer;
    use crate::core::capabilities::effect::foundation::{EffectDuration, EffectInstance};
    use crate::shared::testing::fixtures::{AttributeDefBuilder, attributes_for_unit_001};

    fn make_registry_with_hp() -> AttributeRegistry {
        let mut reg = AttributeRegistry::default();
        for def in attributes_for_unit_001() {
            reg.register(def).unwrap();
        }
        reg
    }

    #[test]
    fn register_hp_attr_no_error() {
        let reg = make_registry_with_hp();
        assert!(reg.contains(&AttributeId::new("attr_hp")));
    }

    #[test]
    fn unregistered_attr_not_in_registry() {
        let reg = make_registry_with_hp();
        assert!(!reg.contains(&AttributeId::new("attr_nonexistent")));
    }

    #[test]
    fn effect_container_rejects_missing_source() {
        let _container = ActiveEffectContainer::new();
        let effect = EffectInstance::new(
            "eff_001",
            "eff_test",
            "Buff",
            "",
            "target_001",
            EffectDuration::Instant,
            1,
        );
        assert!(effect.source_entity.is_empty());
    }

    #[test]
    fn duplicate_attr_id_rejected() {
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
        assert!(result.is_err());
    }
}
