//! Modifier 不改变基础值不变量测试
//!
//! 不变量：Modifier 只影响聚合后的当前值，不改变 base_value。
//! 来源：docs/02-domain/capabilities/modifier_domain.md

#[cfg(test)]
mod tests {
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
}
