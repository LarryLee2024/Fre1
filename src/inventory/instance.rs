// 物品实例与堆叠：ItemInstance / ItemBind / ItemStack

use super::definition::{ItemDef, ItemType};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 绑定状态
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub enum ItemBind {
    #[default]
    None,
    Pickup,
    Equip,
    Account,
}

/// 物品实例（运行时，可变）
#[derive(Clone, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Serialize, Deserialize)]
pub struct ItemInstance {
    /// 唯一实例 ID
    pub instance_id: u64,
    /// 指向定义 ID
    pub def_id: String,
    /// 当前耐久度（仅装备）
    #[serde(default)]
    pub durability: u32,
    /// 最大耐久度（仅装备）
    #[serde(default)]
    pub max_durability: u32,
    /// 强化等级（仅装备）
    #[serde(default)]
    pub enhance_level: u32,
    /// 附魔 trait（仅装备）
    #[serde(default)]
    pub enchantments: Vec<String>,
    /// 绑定状态
    #[serde(default)]
    pub bind: ItemBind,
    /// 签名/定制标记
    #[serde(default)]
    pub signature: Option<String>,
    /// 任务状态标记
    #[serde(default)]
    pub quest_state: Option<String>,
}

impl ItemInstance {
    /// 从定义创建实例
    pub fn from_def(instance_id: u64, def: &ItemDef) -> Self {
        let is_equipment = def.item_type == ItemType::Equipment;
        Self {
            instance_id,
            def_id: def.id.clone(),
            durability: if is_equipment { 100 } else { 0 },
            max_durability: if is_equipment { 100 } else { 0 },
            enhance_level: 0,
            enchantments: vec![],
            bind: ItemBind::None,
            signature: None,
            quest_state: None,
        }
    }

    /// 是否是装备
    pub fn is_equipment(&self) -> bool {
        self.max_durability > 0
    }
}

/// 全局实例 ID 生成器
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct InstanceIdCounter(pub u64);

impl InstanceIdCounter {
    pub fn next(&mut self) -> u64 {
        self.0 += 1;
        self.0
    }
}

/// 物品堆叠：一个实例 × 数量
#[derive(Clone, Debug, Reflect)]
pub struct ItemStack {
    pub instance: ItemInstance,
    pub count: u32,
}

impl ItemStack {
    pub fn new(instance: ItemInstance, count: u32) -> Self {
        Self { instance, count }
    }

    /// 从定义创建堆叠
    pub fn from_def(counter: &mut InstanceIdCounter, def: &ItemDef, count: u32) -> Self {
        let instance = ItemInstance::from_def(counter.next(), def);
        Self { instance, count }
    }

    /// 能否与另一个堆叠合并
    pub fn can_merge_with(&self, other: &ItemStack, def: &ItemDef) -> bool {
        self.instance.def_id == other.instance.def_id
            && self.count + other.count <= def.stack_size
            && self.instance.bind == ItemBind::None
            && other.instance.bind == ItemBind::None
            && self.instance.enhance_level == other.instance.enhance_level
            && self.instance.enchantments == other.instance.enchantments
    }

    /// 总重量
    pub fn total_weight(&self, def: &ItemDef) -> f32 {
        def.weight * self.count as f32
    }

    /// 拆分出指定数量，返回新的 ItemStack
    /// 如果数量不足或等于当前数量，返回 None
    pub fn split(&mut self, count: u32) -> Option<Self> {
        if count == 0 || count >= self.count {
            return None;
        }
        self.count -= count;
        let mut split_instance = self.instance.clone();
        split_instance.instance_id = 0; // 拆分出的需要新 ID，由调用方分配
        Some(Self {
            instance: split_instance,
            count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::equipment::Rarity;

    fn test_equipment_def() -> ItemDef {
        ItemDef {
            version: 1,
            id: "iron_sword".into(),
            name: "铁剑".into(),
            description: String::new(),
            item_type: ItemType::Equipment,
            rarity: Rarity::Common,
            tags: vec![],
            stack_size: 1,
            weight: 3.0,
            modifiers: vec![],
            traits: vec![],
            requirements: vec![],
            slot: None,
            use_effects: vec![],
            container_capacity: None,
            container_max_weight: None,
        }
    }

    fn test_consumable_def() -> ItemDef {
        ItemDef {
            version: 1,
            id: "potion_healing".into(),
            name: "治疗药水".into(),
            description: String::new(),
            item_type: ItemType::Consumable,
            rarity: Rarity::Common,
            tags: vec![],
            stack_size: 99,
            weight: 0.5,
            modifiers: vec![],
            traits: vec![],
            requirements: vec![],
            slot: None,
            use_effects: vec![],
            container_capacity: None,
            container_max_weight: None,
        }
    }

    #[test]
    fn 实例_从装备定义创建() {
        let def = test_equipment_def();
        let instance = ItemInstance::from_def(1, &def);
        assert_eq!(instance.def_id, "iron_sword");
        assert_eq!(instance.durability, 100);
        assert_eq!(instance.max_durability, 100);
        assert!(instance.is_equipment());
    }

    #[test]
    fn 实例_从消耗品定义创建() {
        let def = test_consumable_def();
        let instance = ItemInstance::from_def(2, &def);
        assert_eq!(instance.def_id, "potion_healing");
        assert_eq!(instance.durability, 0);
        assert!(!instance.is_equipment());
    }

    #[test]
    fn 堆叠_从定义创建() {
        let mut counter = InstanceIdCounter::default();
        let def = test_consumable_def();
        let stack = ItemStack::from_def(&mut counter, &def, 10);
        assert_eq!(stack.count, 10);
        assert_eq!(stack.instance.def_id, "potion_healing");
    }

    #[test]
    fn 堆叠_可合并() {
        let def = test_consumable_def();
        let stack1 = ItemStack::new(ItemInstance::from_def(1, &def), 10);
        let stack2 = ItemStack::new(ItemInstance::from_def(2, &def), 20);
        assert!(stack1.can_merge_with(&stack2, &def));
    }

    #[test]
    fn 堆叠_不可合并_超出堆叠上限() {
        let def = test_consumable_def();
        let stack1 = ItemStack::new(ItemInstance::from_def(1, &def), 50);
        let stack2 = ItemStack::new(ItemInstance::from_def(2, &def), 50);
        assert!(!stack1.can_merge_with(&stack2, &def)); // 50 + 50 > 99
    }

    #[test]
    fn 堆叠_不可合并_绑定物品() {
        let def = test_consumable_def();
        let mut instance1 = ItemInstance::from_def(1, &def);
        instance1.bind = ItemBind::Pickup;
        let stack1 = ItemStack::new(instance1, 10);
        let stack2 = ItemStack::new(ItemInstance::from_def(2, &def), 10);
        assert!(!stack1.can_merge_with(&stack2, &def));
    }

    #[test]
    fn 堆叠_不可合并_不同定义() {
        let def1 = test_consumable_def();
        let def2 = test_equipment_def();
        let stack1 = ItemStack::new(ItemInstance::from_def(1, &def1), 10);
        let stack2 = ItemStack::new(ItemInstance::from_def(2, &def2), 1);
        assert!(!stack1.can_merge_with(&stack2, &def1));
    }

    #[test]
    fn 堆叠_总重量() {
        let def = test_consumable_def();
        let stack = ItemStack::new(ItemInstance::from_def(1, &def), 10);
        assert!((stack.total_weight(&def) - 5.0).abs() < f32::EPSILON); // 0.5 * 10
    }

    #[test]
    fn 堆叠_拆分() {
        let def = test_consumable_def();
        let mut stack = ItemStack::new(ItemInstance::from_def(1, &def), 20);
        let split = stack.split(5).unwrap();
        assert_eq!(stack.count, 15);
        assert_eq!(split.count, 5);
    }

    #[test]
    fn 堆叠_拆分_数量不足返回none() {
        let def = test_consumable_def();
        let mut stack = ItemStack::new(ItemInstance::from_def(1, &def), 10);
        assert!(stack.split(10).is_none());
        assert!(stack.split(15).is_none());
        assert!(stack.split(0).is_none());
    }

    #[test]
    fn 实例id计数器() {
        let mut counter = InstanceIdCounter::default();
        assert_eq!(counter.next(), 1);
        assert_eq!(counter.next(), 2);
        assert_eq!(counter.next(), 3);
    }
}
