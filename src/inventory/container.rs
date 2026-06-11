// 统一容器：Container / ContainerKind
// 背包、仓库、宝箱、商店、尸体、掉落袋本质都是 Container

use super::definition::ItemRegistry;
use super::instance::{ItemBind, ItemStack};
use bevy::prelude::*;
use serde::Deserialize;

/// 容器类型
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect, Deserialize)]
pub enum ContainerKind {
    #[default]
    Backpack,
    Warehouse,
    Chest,
    Shop,
    Corpse,
    LootBag,
    Mail,
    BattleBag,
    GuildBank,
}

/// 容器组件
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct Container {
    /// 容器类型
    pub kind: ContainerKind,
    /// 物品堆叠列表
    pub stacks: Vec<ItemStack>,
    /// 最大格数（0 = 无限制）
    pub capacity: u32,
    /// 最大重量（0 = 无限制，DND 风格）
    pub max_weight: f32,
    /// 所属实体（如角色背包的 owner = 角色 Entity）
    pub owner: Option<Entity>,
    /// 容器 Trait（如 extra_capacity, consume_random_item）
    pub container_traits: Vec<String>,
}

impl Container {
    pub fn new(kind: ContainerKind, capacity: u32, max_weight: f32) -> Self {
        Self {
            kind,
            stacks: Vec::new(),
            capacity,
            max_weight,
            owner: None,
            container_traits: vec![],
        }
    }

    /// 创建角色背包
    pub fn backpack() -> Self {
        Self::new(ContainerKind::Backpack, 20, 100.0)
    }

    /// 创建战场背包
    pub fn battle_bag() -> Self {
        Self::new(ContainerKind::BattleBag, 8, 30.0)
    }

    /// 创建宝箱
    pub fn chest(capacity: u32, max_weight: f32) -> Self {
        Self::new(ContainerKind::Chest, capacity, max_weight)
    }

    /// 当前总重量
    pub fn current_weight(&self, registry: &ItemRegistry) -> f32 {
        self.stacks
            .iter()
            .filter_map(|s| registry.get(&s.instance.def_id).map(|d| s.total_weight(d)))
            .sum()
    }

    /// 是否超重
    pub fn is_overweight(&self, registry: &ItemRegistry) -> bool {
        if self.max_weight <= 0.0 {
            return false;
        }
        self.current_weight(registry) > self.max_weight
    }

    /// 是否已满
    pub fn is_full(&self) -> bool {
        self.capacity > 0 && self.stacks.len() >= self.capacity as usize
    }

    /// 添加物品堆叠（自动合并同类型，支持部分合并）
    /// 返回成功添加的数量。未添加的物品保留在 stack.count 中，调用方可处理剩余。
    pub fn add_stack(&mut self, stack: &mut ItemStack, registry: &ItemRegistry) -> u32 {
        let Some(def) = registry.get(&stack.instance.def_id) else {
            return 0;
        };
        let original_count = stack.count;

        // 尝试合并到已有堆叠（每次合并前检查重量）
        // 规则1：检查 def_id/bind/enhance_level/enchantments，允许部分合并
        if def.stack_size > 1 {
            let mut running_weight = self.current_weight(registry);
            for i in 0..self.stacks.len() {
                if stack.count == 0 {
                    break;
                }
                // 检查合并条件（不含数量，数量由部分合并处理）
                if self.stacks[i].instance.def_id != stack.instance.def_id
                    || self.stacks[i].instance.bind != ItemBind::None
                    || stack.instance.bind != ItemBind::None
                    || self.stacks[i].instance.enhance_level != stack.instance.enhance_level
                    || self.stacks[i].instance.enchantments != stack.instance.enchantments
                {
                    continue;
                }
                // 计算可合并数量（部分合并）
                let space = def.stack_size.saturating_sub(self.stacks[i].count);
                if space == 0 {
                    continue;
                }
                let to_merge = space.min(stack.count);
                if self.max_weight > 0.0 {
                    let added_weight = def.weight * to_merge as f32;
                    if running_weight + added_weight > self.max_weight {
                        continue; // 超重，跳过此堆叠
                    }
                }
                self.stacks[i].count += to_merge;
                running_weight += def.weight * to_merge as f32;
                stack.count -= to_merge;
            }
        }

        // 剩余部分作为新堆叠
        if stack.count > 0 && !self.is_full() {
            if self.max_weight > 0.0 {
                let added_weight = def.weight * stack.count as f32;
                if self.current_weight(registry) + added_weight > self.max_weight {
                    return original_count - stack.count;
                }
            }
            self.stacks.push(stack.clone());
            stack.count = 0;
            return original_count;
        }

        // 部分或完全无法添加（剩余物品保留在 stack 中，调用方决定如何处理）
        original_count - stack.count
    }

    /// 移除指定实例 ID 的物品
    pub fn remove(&mut self, instance_id: u64) -> Option<ItemStack> {
        if let Some(idx) = self
            .stacks
            .iter()
            .position(|s| s.instance.instance_id == instance_id)
        {
            Some(self.stacks.remove(idx))
        } else {
            None
        }
    }

    /// 减少指定堆叠的数量（用于消耗品）
    pub fn reduce_stack(&mut self, instance_id: u64, count: u32) -> Option<ItemStack> {
        if let Some(idx) = self
            .stacks
            .iter()
            .position(|s| s.instance.instance_id == instance_id)
        {
            let stack = &mut self.stacks[idx];
            let to_remove = count.min(stack.count);
            stack.count -= to_remove;
            let removed = ItemStack {
                instance: stack.instance.clone(),
                count: to_remove,
            };
            if stack.count == 0 {
                self.stacks.remove(idx);
            }
            Some(removed)
        } else {
            None
        }
    }

    /// 查找指定实例 ID
    pub fn get(&self, instance_id: u64) -> Option<&ItemStack> {
        self.stacks
            .iter()
            .find(|s| s.instance.instance_id == instance_id)
    }

    /// 按定义 ID 查找第一个堆叠
    pub fn find_by_def(&self, def_id: &str) -> Option<&ItemStack> {
        self.stacks.iter().find(|s| s.instance.def_id == def_id)
    }

    /// 按定义 ID 查找第一个可变堆叠
    pub fn find_by_def_mut(&mut self, def_id: &str) -> Option<&mut ItemStack> {
        self.stacks.iter_mut().find(|s| s.instance.def_id == def_id)
    }

    /// 按物品类型筛选
    pub fn filter_by_type(
        &self,
        item_type: super::definition::ItemType,
        registry: &ItemRegistry,
    ) -> Vec<&ItemStack> {
        self.stacks
            .iter()
            .filter(|s| {
                registry
                    .get(&s.instance.def_id)
                    .map(|d| d.item_type == item_type)
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.stacks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stacks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::super::definition::{ItemDef, ItemType};
    use super::super::instance::{InstanceIdCounter, ItemInstance};
    use super::*;
    use crate::equipment::Rarity;

    fn test_registry() -> ItemRegistry {
        let mut registry = ItemRegistry::default();
        registry.register(ItemDef {
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
        });
        registry.register(ItemDef {
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
        });
        registry
    }

    #[test]
    fn 容器_创建背包() {
        let bag = Container::backpack();
        assert_eq!(bag.kind, ContainerKind::Backpack);
        assert_eq!(bag.capacity, 20);
        assert!(bag.is_empty());
    }

    #[test]
    fn 容器_添加物品() {
        let mut bag = Container::backpack();
        let registry = test_registry();
        let mut stack = ItemStack::new(
            ItemInstance::from_def(1, registry.get("potion_healing").unwrap()),
            10,
        );
        assert!(bag.add_stack(&mut stack, &registry) > 0);
        assert_eq!(bag.len(), 1);
    }

    #[test]
    fn 容器_自动合并() {
        let mut bag = Container::backpack();
        let registry = test_registry();
        let mut stack1 = ItemStack::new(
            ItemInstance::from_def(1, registry.get("potion_healing").unwrap()),
            10,
        );
        let mut stack2 = ItemStack::new(
            ItemInstance::from_def(2, registry.get("potion_healing").unwrap()),
            20,
        );
        bag.add_stack(&mut stack1, &registry);
        bag.add_stack(&mut stack2, &registry);
        assert_eq!(bag.len(), 1);
        assert_eq!(bag.stacks[0].count, 30);
    }

    #[test]
    fn 容器_不可合并装备() {
        let mut bag = Container::backpack();
        let registry = test_registry();
        let mut stack1 = ItemStack::new(
            ItemInstance::from_def(1, registry.get("iron_sword").unwrap()),
            1,
        );
        let mut stack2 = ItemStack::new(
            ItemInstance::from_def(2, registry.get("iron_sword").unwrap()),
            1,
        );
        bag.add_stack(&mut stack1, &registry);
        bag.add_stack(&mut stack2, &registry);
        assert_eq!(bag.len(), 2); // 装备 stack_size=1，不能合并
    }

    #[test]
    fn 容器_容量满() {
        let mut bag = Container::new(ContainerKind::Backpack, 2, 0.0);
        let registry = test_registry();
        let mut s1 = ItemStack::new(
            ItemInstance::from_def(1, registry.get("iron_sword").unwrap()),
            1,
        );
        let mut s2 = ItemStack::new(
            ItemInstance::from_def(2, registry.get("iron_sword").unwrap()),
            1,
        );
        let mut s3 = ItemStack::new(
            ItemInstance::from_def(3, registry.get("iron_sword").unwrap()),
            1,
        );
        assert!(bag.add_stack(&mut s1, &registry) > 0);
        assert!(bag.add_stack(&mut s2, &registry) > 0);
        assert_eq!(bag.add_stack(&mut s3, &registry), 0); // 容量满
    }

    #[test]
    fn 容器_移除物品() {
        let mut bag = Container::backpack();
        let registry = test_registry();
        let mut stack = ItemStack::new(
            ItemInstance::from_def(42, registry.get("potion_healing").unwrap()),
            10,
        );
        bag.add_stack(&mut stack, &registry);
        let removed = bag.remove(42);
        assert!(removed.is_some());
        assert!(bag.is_empty());
    }

    #[test]
    fn 容器_减少堆叠() {
        let mut bag = Container::backpack();
        let registry = test_registry();
        let mut stack = ItemStack::new(
            ItemInstance::from_def(1, registry.get("potion_healing").unwrap()),
            10,
        );
        bag.add_stack(&mut stack, &registry);
        let removed = bag.reduce_stack(1, 3);
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().count, 3);
        assert_eq!(bag.stacks[0].count, 7);
    }

    #[test]
    fn 容器_减少堆叠至零自动移除() {
        let mut bag = Container::backpack();
        let registry = test_registry();
        let mut stack = ItemStack::new(
            ItemInstance::from_def(1, registry.get("potion_healing").unwrap()),
            5,
        );
        bag.add_stack(&mut stack, &registry);
        let removed = bag.reduce_stack(1, 5);
        assert!(removed.is_some());
        assert!(bag.is_empty());
    }

    #[test]
    fn 容器_超重检测() {
        let mut bag = Container::new(ContainerKind::Backpack, 0, 5.0);
        let registry = test_registry();
        let mut stack = ItemStack::new(
            ItemInstance::from_def(1, registry.get("iron_sword").unwrap()),
            1,
        );
        bag.add_stack(&mut stack, &registry);
        // 铁剑 3.0 < 5.0，不超重
        assert!(!bag.is_overweight(&registry));
        // 再加一把 3.0+3.0=6.0 > 5.0，add_stack 应拒绝
        let mut stack2 = ItemStack::new(
            ItemInstance::from_def(2, registry.get("iron_sword").unwrap()),
            1,
        );
        assert_eq!(bag.add_stack(&mut stack2, &registry), 0);
        // 容量仍为 1，超重检测仍为 false
        assert_eq!(bag.len(), 1);
        assert!(!bag.is_overweight(&registry));
    }

    #[test]
    fn 容器_添加物品_超重拒绝() {
        let mut bag = Container::new(ContainerKind::Backpack, 0, 5.0);
        let registry = test_registry();
        // 铁剑 3.0，添加一把不超重
        let mut s1 = ItemStack::new(
            ItemInstance::from_def(1, registry.get("iron_sword").unwrap()),
            1,
        );
        assert!(bag.add_stack(&mut s1, &registry) > 0);
        // 再加一把 3.0+3.0=6.0 > 5.0，应被拒绝
        let mut s2 = ItemStack::new(
            ItemInstance::from_def(2, registry.get("iron_sword").unwrap()),
            1,
        );
        assert_eq!(bag.add_stack(&mut s2, &registry), 0);
        assert_eq!(bag.len(), 1);
    }

    #[test]
    fn 容器_添加物品_堆叠超重也拒绝() {
        let mut bag = Container::new(ContainerKind::Backpack, 0, 2.0);
        let registry = test_registry();
        // 药水 0.5，添加 5 个 = 2.5 > 2.0，应被拒绝
        let mut stack = ItemStack::new(
            ItemInstance::from_def(1, registry.get("potion_healing").unwrap()),
            5,
        );
        assert_eq!(bag.add_stack(&mut stack, &registry), 0);
        assert!(bag.is_empty());
    }

    #[test]
    fn 容器_按类型筛选() {
        let mut bag = Container::backpack();
        let registry = test_registry();
        let mut potion = ItemStack::new(
            ItemInstance::from_def(1, registry.get("potion_healing").unwrap()),
            10,
        );
        let mut sword = ItemStack::new(
            ItemInstance::from_def(2, registry.get("iron_sword").unwrap()),
            1,
        );
        bag.add_stack(&mut potion, &registry);
        bag.add_stack(&mut sword, &registry);
        let consumables = bag.filter_by_type(ItemType::Consumable, &registry);
        assert_eq!(consumables.len(), 1);
        let equipment = bag.filter_by_type(ItemType::Equipment, &registry);
        assert_eq!(equipment.len(), 1);
    }

    #[test]
    fn 容器_查找物品() {
        let mut bag = Container::backpack();
        let registry = test_registry();
        let mut stack = ItemStack::new(
            ItemInstance::from_def(1, registry.get("potion_healing").unwrap()),
            10,
        );
        bag.add_stack(&mut stack, &registry);
        assert!(bag.get(1).is_some());
        assert!(bag.get(999).is_none());
        assert!(bag.find_by_def("potion_healing").is_some());
        assert!(bag.find_by_def("nonexistent").is_none());
    }

    #[test]
    fn 容器_添加物品_部分成功返回已添加数量() {
        let registry = test_registry();
        // 药水 weight=0.5, stack_size=99
        let mut bag = Container::new(ContainerKind::Backpack, 0, 5.0);
        let mut s1 = ItemStack::new(
            ItemInstance::from_def(1, registry.get("potion_healing").unwrap()),
            6,
        );
        // 6 * 0.5 = 3.0 < 5.0，全部成功
        assert_eq!(bag.add_stack(&mut s1, &registry), 6);
        assert_eq!(bag.stacks[0].count, 6);

        // 再加 8 个：3.0 + 4.0 = 7.0 > 5.0，超重
        // 但如果没有已有堆叠可合并，全部失败
        let mut s2 = ItemStack::new(
            ItemInstance::from_def(2, registry.get("potion_healing").unwrap()),
            8,
        );
        assert_eq!(bag.add_stack(&mut s2, &registry), 0);
        assert_eq!(bag.stacks.len(), 1); // 未添加新堆叠

        // 关键回归测试：部分合并后超重，应返回成功数量而非 0
        // 构造：已有堆叠占部分空间，新堆叠放剩余但超重
        let mut bag2 = Container::new(ContainerKind::Backpack, 0, 1.5);
        // 先加 2 个药水(1.0)
        let mut s3 = ItemStack::new(
            ItemInstance::from_def(3, registry.get("potion_healing").unwrap()),
            2,
        );
        assert_eq!(bag2.add_stack(&mut s3, &registry), 2);
        // 再加 4 个药水(2.0)：1.0+2.0=3.0 > 1.5，超重 → 全部失败
        let mut s4 = ItemStack::new(
            ItemInstance::from_def(4, registry.get("potion_healing").unwrap()),
            4,
        );
        assert_eq!(bag2.add_stack(&mut s4, &registry), 0);
    }
}
