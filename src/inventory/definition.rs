// 物品定义：ItemDef / ItemType / UseEffect / ItemRegistry

use crate::core::attribute::AttributeKind;
use crate::core::attribute::AttributeModifierDef;
use crate::core::registry_loader::RegistryLoader;
use crate::core::tag::TagName;
use crate::equipment::{EquipmentRequirement, EquipmentSlot, Rarity};
use bevy::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;

/// 物品类型
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum ItemType {
    Equipment,
    Consumable,
    Material,
    Quest,
    Ammo,
    Currency,
    Container,
}

impl ItemType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Equipment => "装备",
            Self::Consumable => "消耗品",
            Self::Material => "材料",
            Self::Quest => "任务物品",
            Self::Ammo => "弹药",
            Self::Currency => "货币",
            Self::Container => "容器",
        }
    }
}

/// 消耗品使用效果
#[derive(Clone, Debug, Deserialize)]
pub enum UseEffect {
    /// 恢复 HP/MP/Stamina
    RestoreVital { kind: AttributeKind, value: f32 },
    /// 施加 Buff
    ApplyBuff { buff_id: String, duration: u32 },
    /// 授予临时 Trait
    GrantTempTrait { trait_id: String, duration: u32 },
    /// 释放技能（卷轴）
    CastSkill { skill_id: String },
}

fn default_stack_size() -> u32 {
    1
}

/// 物品定义（RON 配置，不可变）
#[derive(Clone, Debug, Deserialize)]
pub struct ItemDef {
    #[serde(default)]
    pub version: u32,
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    /// 物品类型（默认 Equipment，兼容旧装备 RON）
    #[serde(default = "default_item_type")]
    pub item_type: ItemType,
    pub rarity: Rarity,
    /// 物品标签
    #[serde(default)]
    pub tags: Vec<TagName>,
    /// 最大堆叠数（1 = 不可堆叠，如装备；99 = 可堆叠，如药水）
    #[serde(default = "default_stack_size")]
    pub stack_size: u32,
    /// 重量（DND 风格，0 = 不占重量）
    #[serde(default)]
    pub weight: f32,
    /// 属性修饰
    #[serde(default)]
    pub modifiers: Vec<AttributeModifierDef>,
    /// 授予的 Trait
    #[serde(default)]
    pub traits: Vec<String>,
    /// 需求条件（仅装备）
    #[serde(default)]
    pub requirements: Vec<EquipmentRequirement>,
    /// 装备槽位（仅装备）
    #[serde(default)]
    pub slot: Option<EquipmentSlot>,
    /// 使用效果（仅消耗品）
    #[serde(default)]
    pub use_effects: Vec<UseEffect>,
    /// 容器容量（仅 Container 类型）
    #[serde(default)]
    pub container_capacity: Option<u32>,
    /// 容器最大重量（仅 Container 类型）
    #[serde(default)]
    pub container_max_weight: Option<f32>,
}

fn default_item_type() -> ItemType {
    ItemType::Equipment
}

/// 物品注册表资源
#[derive(Resource, Default)]
pub struct ItemRegistry {
    defs: HashMap<String, ItemDef>,
}

impl ItemRegistry {
    pub fn get(&self, id: &str) -> Option<&ItemDef> {
        self.defs.get(id)
    }

    pub fn register(&mut self, def: ItemDef) {
        self.defs.insert(def.id.clone(), def);
    }

    pub fn iter(&self) -> impl Iterator<Item = &ItemDef> {
        self.defs.values()
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    /// 按类型筛选
    pub fn iter_by_type(&self, item_type: ItemType) -> impl Iterator<Item = &ItemDef> {
        self.defs.values().filter(move |d| d.item_type == item_type)
    }

    /// 注册内置默认物品
    fn register_defaults(&mut self) {
        if !self.defs.is_empty() {
            return;
        }

        // 装备（兼容现有 EquipmentDef）
        let defaults: Vec<ItemDef> = vec![
            ItemDef {
                version: 1,
                id: "iron_sword".into(),
                name: "铁剑".into(),
                description: "普通的铁剑".into(),
                item_type: ItemType::Equipment,
                rarity: Rarity::Common,
                tags: vec![TagName::Sword, TagName::Melee, TagName::Martial],
                stack_size: 1,
                weight: 3.0,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Attack,
                    op: crate::core::attribute::ModifierOp::Add,
                    value: 3.0,
                }],
                traits: vec![],
                requirements: vec![],
                slot: Some(EquipmentSlot::MainHand),
                use_effects: vec![],
                container_capacity: None,
                container_max_weight: None,
            },
            ItemDef {
                version: 1,
                id: "leather_armor".into(),
                name: "皮甲".into(),
                description: "轻便的皮甲".into(),
                item_type: ItemType::Equipment,
                rarity: Rarity::Common,
                tags: vec![TagName::LightArmor],
                stack_size: 1,
                weight: 4.0,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Defense,
                    op: crate::core::attribute::ModifierOp::Add,
                    value: 2.0,
                }],
                traits: vec![],
                requirements: vec![],
                slot: Some(EquipmentSlot::Body),
                use_effects: vec![],
                container_capacity: None,
                container_max_weight: None,
            },
            ItemDef {
                version: 1,
                id: "flame_dragon_sword".into(),
                name: "炎龙长剑".into(),
                description: "蕴含龙焰的古老长剑".into(),
                item_type: ItemType::Equipment,
                rarity: Rarity::Epic,
                tags: vec![
                    TagName::Sword,
                    TagName::Fire,
                    TagName::Martial,
                    TagName::TwoHanded,
                ],
                stack_size: 1,
                weight: 5.0,
                modifiers: vec![
                    AttributeModifierDef {
                        kind: AttributeKind::Attack,
                        op: crate::core::attribute::ModifierOp::Add,
                        value: 15.0,
                    },
                    AttributeModifierDef {
                        kind: AttributeKind::CritRate,
                        op: crate::core::attribute::ModifierOp::Add,
                        value: 5.0,
                    },
                ],
                traits: vec!["flaming_weapon".into(), "dragon_bane".into()],
                requirements: vec![EquipmentRequirement::RequireTag(TagName::Martial)],
                slot: Some(EquipmentSlot::MainHand),
                use_effects: vec![],
                container_capacity: None,
                container_max_weight: None,
            },
            ItemDef {
                version: 1,
                id: "iron_shield".into(),
                name: "铁盾".into(),
                description: "坚固的铁盾".into(),
                item_type: ItemType::Equipment,
                rarity: Rarity::Common,
                tags: vec![TagName::Shield],
                stack_size: 1,
                weight: 6.0,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Defense,
                    op: crate::core::attribute::ModifierOp::Add,
                    value: 3.0,
                }],
                traits: vec![],
                requirements: vec![],
                slot: Some(EquipmentSlot::OffHand),
                use_effects: vec![],
                container_capacity: None,
                container_max_weight: None,
            },
            ItemDef {
                version: 1,
                id: "mage_staff".into(),
                name: "法师法杖".into(),
                description: "蕴含魔力的法杖".into(),
                item_type: ItemType::Equipment,
                rarity: Rarity::Uncommon,
                tags: vec![TagName::Staff, TagName::Simple],
                stack_size: 1,
                weight: 2.0,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::MagicAttack,
                    op: crate::core::attribute::ModifierOp::Add,
                    value: 5.0,
                }],
                traits: vec![],
                requirements: vec![],
                slot: Some(EquipmentSlot::MainHand),
                use_effects: vec![],
                container_capacity: None,
                container_max_weight: None,
            },
            // 消耗品
            ItemDef {
                version: 1,
                id: "potion_healing".into(),
                name: "治疗药水".into(),
                description: "恢复 50 HP".into(),
                item_type: ItemType::Consumable,
                rarity: Rarity::Common,
                tags: vec![TagName::Consumable],
                stack_size: 99,
                weight: 0.5,
                modifiers: vec![],
                traits: vec![],
                requirements: vec![],
                slot: None,
                use_effects: vec![UseEffect::RestoreVital {
                    kind: AttributeKind::Hp,
                    value: 50.0,
                }],
                container_capacity: None,
                container_max_weight: None,
            },
            ItemDef {
                version: 1,
                id: "potion_mana".into(),
                name: "法力药水".into(),
                description: "恢复 30 MP".into(),
                item_type: ItemType::Consumable,
                rarity: Rarity::Common,
                tags: vec![TagName::Consumable],
                stack_size: 99,
                weight: 0.5,
                modifiers: vec![],
                traits: vec![],
                requirements: vec![],
                slot: None,
                use_effects: vec![UseEffect::RestoreVital {
                    kind: AttributeKind::Mp,
                    value: 30.0,
                }],
                container_capacity: None,
                container_max_weight: None,
            },
            // 弹药
            ItemDef {
                version: 1,
                id: "arrow".into(),
                name: "箭矢".into(),
                description: "普通箭矢".into(),
                item_type: ItemType::Ammo,
                rarity: Rarity::Common,
                tags: vec![TagName::Ammo],
                stack_size: 99,
                weight: 0.1,
                modifiers: vec![AttributeModifierDef {
                    kind: AttributeKind::Attack,
                    op: crate::core::attribute::ModifierOp::Add,
                    value: 1.0,
                }],
                traits: vec![],
                requirements: vec![],
                slot: None,
                use_effects: vec![],
                container_capacity: None,
                container_max_weight: None,
            },
        ];

        for def in defaults {
            let id = def.id.clone();
            self.defs.insert(id, def);
        }
    }
}

impl RegistryLoader for ItemRegistry {
    type Item = ItemDef;

    fn register_item(&mut self, item: ItemDef) {
        let id = item.id.clone();
        self.register(item);
        bevy::log::info!(target: "inventory", id = %id, "物品定义已加载");
    }

    fn register_defaults(&mut self) {
        ItemRegistry::register_defaults(self);
    }

    fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    fn registry_name() -> &'static str {
        "Item"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ron::de::from_bytes;

    #[test]
    fn ron_反序列化_装备物品() {
        let ron_str = r#"
            (
                id: "iron_sword",
                name: "铁剑",
                description: "普通的铁剑",
                item_type: Equipment,
                rarity: Common,
                tags: [SWORD, MELEE, MARTIAL],
                stack_size: 1,
                weight: 3.0,
                modifiers: [
                    (kind: Attack, op: Add, value: 3.0),
                ],
                slot: Some(MainHand),
            )
        "#;
        let def: ItemDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "iron_sword");
        assert_eq!(def.item_type, ItemType::Equipment);
        assert_eq!(def.stack_size, 1);
        assert_eq!(def.weight, 3.0);
        assert_eq!(def.slot, Some(EquipmentSlot::MainHand));
    }

    #[test]
    fn ron_反序列化_消耗品() {
        let ron_str = r#"
            (
                id: "potion_healing",
                name: "治疗药水",
                description: "恢复 50 HP",
                item_type: Consumable,
                rarity: Common,
                tags: [CONSUMABLE],
                stack_size: 99,
                weight: 0.5,
                use_effects: [
                    RestoreVital(kind: Hp, value: 50.0),
                ],
            )
        "#;
        let def: ItemDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "potion_healing");
        assert_eq!(def.item_type, ItemType::Consumable);
        assert_eq!(def.stack_size, 99);
        assert_eq!(def.use_effects.len(), 1);
    }

    #[test]
    fn ron_反序列化_兼容旧装备格式() {
        // 旧 EquipmentDef 格式没有 item_type 字段，默认为 Equipment
        let ron_str = r#"
            (
                id: "old_sword",
                name: "旧剑",
                description: "",
                rarity: Common,
                slot: Some(MainHand),
            )
        "#;
        let def: ItemDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.id, "old_sword");
        assert_eq!(def.item_type, ItemType::Equipment);
        assert_eq!(def.stack_size, 1);
    }

    #[test]
    fn 物品注册表_查询() {
        let mut registry = ItemRegistry::default();
        registry.register_defaults();
        assert!(registry.get("iron_sword").is_some());
        assert!(registry.get("potion_healing").is_some());
        assert!(registry.get("arrow").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn 物品注册表_按类型筛选() {
        let mut registry = ItemRegistry::default();
        registry.register_defaults();
        let equipment: Vec<_> = registry.iter_by_type(ItemType::Equipment).collect();
        let consumables: Vec<_> = registry.iter_by_type(ItemType::Consumable).collect();
        let ammo: Vec<_> = registry.iter_by_type(ItemType::Ammo).collect();
        assert_eq!(equipment.len(), 5);
        assert_eq!(consumables.len(), 2);
        assert_eq!(ammo.len(), 1);
    }

    #[test]
    fn 物品类型_label() {
        assert_eq!(ItemType::Equipment.label(), "装备");
        assert_eq!(ItemType::Consumable.label(), "消耗品");
        assert_eq!(ItemType::Material.label(), "材料");
        assert_eq!(ItemType::Ammo.label(), "弹药");
        assert_eq!(ItemType::Currency.label(), "货币");
        assert_eq!(ItemType::Container.label(), "容器");
    }

    #[test]
    fn ron_反序列化_弹药() {
        let ron_str = r#"
            (
                id: "arrow",
                name: "箭矢",
                description: "普通箭矢",
                item_type: Ammo,
                rarity: Common,
                stack_size: 99,
                weight: 0.1,
                modifiers: [
                    (kind: Attack, op: Add, value: 1.0),
                ],
            )
        "#;
        let def: ItemDef = from_bytes(ron_str.as_bytes()).unwrap();
        assert_eq!(def.item_type, ItemType::Ammo);
        assert_eq!(def.slot, None);
    }
}
