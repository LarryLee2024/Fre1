// 装备需求检查：验证单位是否满足装备穿戴条件

use super::definition::{EquipmentDef, EquipmentRequirement};
use crate::core::attribute::Attributes;
use crate::core::tag::GameplayTags;

/// 需求检查结果
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RequirementCheckResult {
    /// 满足所有需求
    Satisfied,
    /// 不满足，附带失败原因
    Failed(String),
}

impl RequirementCheckResult {
    pub fn is_satisfied(&self) -> bool {
        matches!(self, Self::Satisfied)
    }
}

/// 检查单位是否满足装备的所有需求条件
pub fn check_equipment_requirements(
    def: &EquipmentDef,
    attrs: &Attributes,
    tags: &GameplayTags,
) -> RequirementCheckResult {
    for req in &def.requirements {
        match req {
            EquipmentRequirement::RequireTag(tag_name) => {
                let tag = tag_name.to_tag();
                if !tags.has(tag) {
                    return RequirementCheckResult::Failed(format!("缺少标签: {:?}", tag_name));
                }
            }
            EquipmentRequirement::AttributeMin { kind, value } => {
                let current = attrs.get(*kind);
                if current < *value {
                    return RequirementCheckResult::Failed(format!(
                        "属性不足: {:?} 当前={} 需要={}",
                        kind, current, value
                    ));
                }
            }
        }
    }
    RequirementCheckResult::Satisfied
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::attribute::AttributeKind;
    use crate::core::tag::{GameplayTag, TagName};
    use crate::equipment::definition::{EquipmentSlot, Rarity};

    /// 辅助：创建测试用属性
    fn make_test_attrs() -> Attributes {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Might, 5.0);
        attrs.set_base(AttributeKind::Attack, 10.0);
        attrs
    }

    #[test]
    fn 无需求_总是满足() {
        let def = EquipmentDef {
            version: 0,
            id: "test".into(),
            name: "测试".into(),
            description: String::new(),
            slot: EquipmentSlot::MainHand,
            rarity: Rarity::Common,
            tags: vec![],
            modifiers: vec![],
            traits: vec![],
            requirements: vec![],
            weight: 0.0,
        };
        let attrs = make_test_attrs();
        let tags = GameplayTags::default();
        assert!(check_equipment_requirements(&def, &attrs, &tags).is_satisfied());
    }

    #[test]
    fn 标签需求_满足() {
        let def = EquipmentDef {
            version: 0,
            id: "test".into(),
            name: "测试".into(),
            description: String::new(),
            slot: EquipmentSlot::MainHand,
            rarity: Rarity::Common,
            tags: vec![],
            modifiers: vec![],
            traits: vec![],
            requirements: vec![EquipmentRequirement::RequireTag(TagName::Martial)],
            weight: 0.0,
        };
        let attrs = make_test_attrs();
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::MARTIAL);
        assert!(check_equipment_requirements(&def, &attrs, &tags).is_satisfied());
    }

    #[test]
    fn 标签需求_不满足() {
        let def = EquipmentDef {
            version: 0,
            id: "test".into(),
            name: "测试".into(),
            description: String::new(),
            slot: EquipmentSlot::MainHand,
            rarity: Rarity::Common,
            tags: vec![],
            modifiers: vec![],
            traits: vec![],
            requirements: vec![EquipmentRequirement::RequireTag(TagName::Martial)],
        };
        let attrs = make_test_attrs();
        let tags = GameplayTags::default();
        let result = check_equipment_requirements(&def, &attrs, &tags);
        assert!(!result.is_satisfied());
        if let RequirementCheckResult::Failed(reason) = result {
            assert!(reason.contains("标签"));
        }
    }

    #[test]
    fn 属性需求_满足() {
        let def = EquipmentDef {
            version: 0,
            id: "test".into(),
            name: "测试".into(),
            description: String::new(),
            slot: EquipmentSlot::MainHand,
            rarity: Rarity::Common,
            tags: vec![],
            modifiers: vec![],
            traits: vec![],
            requirements: vec![EquipmentRequirement::AttributeMin {
                kind: AttributeKind::Attack,
                value: 8.0,
            }],
        };
        let attrs = make_test_attrs();
        let tags = GameplayTags::default();
        assert!(check_equipment_requirements(&def, &attrs, &tags).is_satisfied());
    }

    #[test]
    fn 属性需求_不满足() {
        let def = EquipmentDef {
            version: 0,
            id: "test".into(),
            name: "测试".into(),
            description: String::new(),
            slot: EquipmentSlot::MainHand,
            rarity: Rarity::Common,
            tags: vec![],
            modifiers: vec![],
            traits: vec![],
            requirements: vec![EquipmentRequirement::AttributeMin {
                kind: AttributeKind::Attack,
                value: 20.0,
            }],
        };
        let attrs = make_test_attrs();
        let tags = GameplayTags::default();
        let result = check_equipment_requirements(&def, &attrs, &tags);
        assert!(!result.is_satisfied());
        if let RequirementCheckResult::Failed(reason) = result {
            assert!(reason.contains("属性不足"));
        }
    }

    #[test]
    fn 多个需求_部分不满足() {
        let def = EquipmentDef {
            version: 0,
            id: "test".into(),
            name: "测试".into(),
            description: String::new(),
            slot: EquipmentSlot::MainHand,
            rarity: Rarity::Common,
            tags: vec![],
            modifiers: vec![],
            traits: vec![],
            requirements: vec![
                EquipmentRequirement::RequireTag(TagName::Martial),
                EquipmentRequirement::AttributeMin {
                    kind: AttributeKind::Attack,
                    value: 20.0,
                },
            ],
        };
        let attrs = make_test_attrs();
        let mut tags = GameplayTags::default();
        tags.add(GameplayTag::MARTIAL);
        // 标签满足但属性不足
        let result = check_equipment_requirements(&def, &attrs, &tags);
        assert!(!result.is_satisfied());
    }
}
