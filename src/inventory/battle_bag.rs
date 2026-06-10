// 战场背包：BattleInventory 组件 + 战斗开始/结束时的创建与合并

use super::container::{Container, ContainerKind};
use super::definition::ItemRegistry;
use bevy::prelude::*;

/// 战场背包组件（战斗中生成，战斗结束合并回角色背包）
#[derive(Component, Debug, Clone)]
pub struct BattleInventory {
    pub container: Container,
    /// 原始背包 Entity（战斗结束后归还）
    pub source_backpack: Entity,
}

impl BattleInventory {
    pub fn new(source_backpack: Entity) -> Self {
        Self {
            container: Container::battle_bag(),
            source_backpack,
        }
    }
}

/// 战斗开始时：从角色背包生成战场背包
pub fn create_battle_inventory(backpacks: Query<(Entity, &Container)>, mut commands: Commands) {
    for (entity, _backpack) in &backpacks {
        let battle_bag = BattleInventory::new(entity);
        commands.spawn(battle_bag);
    }
}

/// 战斗结束时：战场背包物品归还角色背包
pub fn merge_battle_inventory(
    mut battle_bags: Query<&mut BattleInventory>,
    mut backpacks: Query<&mut Container>,
    item_registry: Res<ItemRegistry>,
    mut commands: Commands,
) {
    for mut battle_bag in &mut battle_bags {
        if let Ok(mut backpack) = backpacks.get_mut(battle_bag.source_backpack) {
            // 将战场背包中所有物品转移回角色背包
            let stacks: Vec<_> = battle_bag.container.stacks.drain(..).collect();
            for stack in stacks {
                backpack.add_stack(stack, &item_registry);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::equipment::Rarity;
    use crate::inventory::definition::{ItemDef, ItemType};
    use crate::inventory::instance::ItemInstance;
    use crate::inventory::instance::ItemStack;

    #[test]
    fn 战场背包_创建() {
        let bag = BattleInventory::new(Entity::PLACEHOLDER);
        assert_eq!(bag.container.kind, ContainerKind::BattleBag);
        assert_eq!(bag.container.capacity, 8);
    }

    #[test]
    fn 战场背包_容量限制() {
        let mut bag = BattleInventory::new(Entity::PLACEHOLDER);
        let mut registry = ItemRegistry::default();

        // 注册 10 种不同物品（不可堆叠）
        for i in 0..10 {
            registry.register(ItemDef {
                version: 1,
                id: format!("item_{}", i),
                name: format!("物品{}", i),
                description: String::new(),
                item_type: ItemType::Consumable,
                rarity: Rarity::Common,
                tags: vec![],
                stack_size: 1, // 不可堆叠
                weight: 0.1,
                modifiers: vec![],
                traits: vec![],
                requirements: vec![],
                slot: None,
                use_effects: vec![],
                container_capacity: None,
                container_max_weight: None,
            });
        }

        // 战场背包容量 8，添加 8 个不同物品应成功
        for i in 0..8 {
            let stack = ItemStack::new(
                ItemInstance::from_def(i, registry.get(&format!("item_{}", i)).unwrap()),
                1,
            );
            assert!(
                bag.container.add_stack(stack, &registry),
                "第 {} 个应成功",
                i
            );
        }
        // 第 9 个应失败
        let stack = ItemStack::new(
            ItemInstance::from_def(100, registry.get("item_8").unwrap()),
            1,
        );
        assert!(!bag.container.add_stack(stack, &registry));
    }
}
