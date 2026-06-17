//! Tag bit 唯一不变量测试
//!
//! 不变量：同一 Tag 不能在位掩码中重复设置。
//! 来源：docs/02-domain/capabilities/tag_domain.md

#[cfg(test)]
mod tests {
    use crate::core::capabilities::tag::foundation::{TagId, TagNamespace, TagDefinition};
    use crate::core::capabilities::tag::mechanism::lifecycle::TagHierarchy;
    use crate::shared::testing::fixtures::{standard_damage_tags, TagDefBuilder};

    fn make_hierarchy() -> TagHierarchy {
        let mut h = TagHierarchy::default();
        for tag in standard_damage_tags() {
            h.register(tag).unwrap();
        }
        h
    }

    #[test]
    fn root_tag_bitmask_only_own_bit() {
        let hierarchy = make_hierarchy();
        let mask = hierarchy.inherited_mask(&TagId::new("tag_dmg_phys"));
        assert_eq!(mask, 1u128 << 3);
    }

    #[test]
    fn parent_tag_bitmask_contains_self_and_children() {
        let hierarchy = make_hierarchy();
        let mask = hierarchy.inherited_mask(&TagId::new("tag_dmg_elemental"));
        assert!(mask & (1u128 << 0) != 0);
        assert!(mask & (1u128 << 1) != 0);
        assert!(mask & (1u128 << 2) != 0);
    }

    #[test]
    fn child_tag_bitmask_only_own_bit() {
        let hierarchy = make_hierarchy();
        let mask = hierarchy.inherited_mask(&TagId::new("tag_dmg_fire"));
        assert_eq!(mask, 1u128 << 1);
    }

    #[test]
    fn different_tags_occupy_different_bits() {
        let hierarchy = make_hierarchy();
        let fire_mask = hierarchy.inherited_mask(&TagId::new("tag_dmg_fire"));
        let ice_mask = hierarchy.inherited_mask(&TagId::new("tag_dmg_ice"));
        let phys_mask = hierarchy.inherited_mask(&TagId::new("tag_dmg_phys"));
        assert_eq!(fire_mask & ice_mask, 0);
        assert_eq!(fire_mask & phys_mask, 0);
        assert_eq!(ice_mask & phys_mask, 0);
    }

    #[test]
    fn duplicate_same_id_tag_rejected() {
        let mut hierarchy = TagHierarchy::default();
        let tag = TagDefBuilder::new("tag_test_dup", TagNamespace::DamageType)
            .bit_index(0)
            .build();
        assert!(hierarchy.register(tag.clone()).is_ok());
        let result = hierarchy.register(tag);
        assert!(result.is_err());
    }
}
