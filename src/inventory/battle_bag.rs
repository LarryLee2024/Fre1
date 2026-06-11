// 战场背包：BattleInventory 组件 + 战斗开始/结束时的创建与合并

use super::container::Container;
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

/// 战斗开始时：从角色背包生成战场背包（复制物品）
pub fn create_battle_inventory(
    backpacks: Query<(Entity, &Container)>,
    item_registry: Res<ItemRegistry>,
    mut commands: Commands,
) {
    for (entity, backpack) in &backpacks {
        let mut battle_bag = BattleInventory::new(entity);
        // 复制源背包物品到战场背包
        for stack in &backpack.stacks {
            battle_bag
                .container
                .add_stack(stack.clone(), &item_registry);
        }
        commands.spawn(battle_bag);
    }
}

/// 战斗结束时：战场背包物品归还角色背包
pub fn merge_battle_inventory(
    mut battle_bags: Query<&mut BattleInventory>,
    mut backpacks: Query<&mut Container>,
    item_registry: Res<ItemRegistry>,
    _commands: Commands,
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
                bag.container.add_stack(stack, &registry) > 0,
                "第 {} 个应成功",
                i
            );
        }
        // 第 9 个应失败
        let stack = ItemStack::new(
            ItemInstance::from_def(100, registry.get("item_8").unwrap()),
            1,
        );
        assert_eq!(bag.container.add_stack(stack, &registry), 0);
    }

    #[test]
    fn 战场背包_创建时复制物品() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(test_registry());

        // 创建源背包并添加物品
        let mut source = Container::backpack();
        let registry = app.world().resource::<ItemRegistry>();
        let potion = ItemStack::new(
            ItemInstance::from_def(1, registry.get("potion_healing").unwrap()),
            5,
        );
        let sword = ItemStack::new(
            ItemInstance::from_def(2, registry.get("iron_sword").unwrap()),
            1,
        );
        source.add_stack(potion, &registry);
        source.add_stack(sword, &registry);
        let source_entity = app.world_mut().spawn(source).id();

        // 运行 create_battle_inventory
        app.add_systems(Update, create_battle_inventory);
        app.update();

        // 找到生成的战场背包实体
        let mut query = app.world_mut().query::<(Entity, &BattleInventory)>();
        let (battle_entity, battle_bag) = query.single(app.world()).unwrap();
        assert_ne!(battle_entity, source_entity);
        assert_eq!(battle_bag.source_backpack, source_entity);
        assert_eq!(battle_bag.container.len(), 2);
        // 验证物品数量正确
        let potion_stack = battle_bag.container.find_by_def("potion_healing").unwrap();
        assert_eq!(potion_stack.count, 5);
        let sword_stack = battle_bag.container.find_by_def("iron_sword").unwrap();
        assert_eq!(sword_stack.count, 1);
    }
}
