//! 背包系统 Feature Test
//!
//! 跨 inventory/container + inventory/transfer + inventory/definition + inventory/instance
//! 测试容器间物品转移、容量限制、纯函数调用。

use bevy::prelude::*;
use tactical_rpg::equipment::Rarity;
use tactical_rpg::inventory::container::{Container, ContainerKind};
use tactical_rpg::inventory::definition::{ItemDef, ItemRegistry, ItemType};
use tactical_rpg::inventory::instance::{InstanceIdCounter, ItemStack};
use tactical_rpg::inventory::transfer::{ContainerResult, TransferItem, transfer_item};

use crate::common::app_builder::combat_app;

// ── 测试辅助 ──

/// 注册测试物品到 ItemRegistry
fn register_test_items(app: &mut App) {
    let potion = ItemDef {
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
    };

    let mut item_reg = app.world_mut().resource_mut::<ItemRegistry>();
    item_reg.register(potion);
}

/// 构建背包测试 App（InventoryPlugin 已注册 TransferItem/ItemTransferred 消息和 transfer_item_system）
fn inventory_app() -> App {
    combat_app()
}

/// 生成一个带 Container 的 Entity（独立背包，不属于角色）
fn spawn_container(app: &mut App, kind: ContainerKind, capacity: u32, max_weight: f32) -> Entity {
    app.world_mut()
        .spawn(Container::new(kind, capacity, max_weight))
        .id()
}

/// 在容器中放入物品，返回 instance_id
fn put_item_in_container(app: &mut App, container_entity: Entity, def_id: &str, count: u32) -> u64 {
    // 先获取 ItemDef 的克隆，释放不可变借用
    let item_def = app
        .world()
        .resource::<ItemRegistry>()
        .get(def_id)
        .cloned()
        .unwrap();
    // 再获取可变借用生成 instance_id
    let (instance_id, mut stack) = {
        let mut counter = app.world_mut().resource_mut::<InstanceIdCounter>();
        let stack = ItemStack::from_def(&mut counter, &item_def, count);
        (stack.instance.instance_id, stack)
    };
    // 使用 resource_scope 避免同时持有 World 的可变和不可变借用
    app.world_mut()
        .resource_scope(|world, item_reg: Mut<ItemRegistry>| {
            let mut container = world.get_mut::<Container>(container_entity).unwrap();
            container.add_stack(&mut stack, &item_reg);
        });
    instance_id
}

// ══════════════════════════════════════════════════════════════
// 场景一：容器间转移物品
// ══════════════════════════════════════════════════════════════

#[test]
fn 容器间转移物品_从a到b_a减少b增加() {
    let mut app = inventory_app();
    register_test_items(&mut app);

    // 创建两个背包
    let bag_a = spawn_container(&mut app, ContainerKind::Backpack, 20, 100.0);
    let bag_b = spawn_container(&mut app, ContainerKind::Backpack, 20, 100.0);

    // 在 A 中放入 10 瓶药水
    let potion_id = put_item_in_container(&mut app, bag_a, "potion_healing", 10);

    // 验证初始状态
    {
        let container_a = app.world().get::<Container>(bag_a).unwrap();
        assert_eq!(container_a.stacks.len(), 1);
        assert_eq!(container_a.stacks[0].count, 10);
        let container_b = app.world().get::<Container>(bag_b).unwrap();
        assert!(container_b.is_empty());
    }

    // 从 A 转移 5 瓶到 B
    app.world_mut().write_message(TransferItem {
        from_entity: bag_a,
        to_entity: bag_b,
        instance_id: potion_id,
        count: 5,
    });
    app.update();

    // 验证：A 剩余 5 瓶
    {
        let container_a = app.world().get::<Container>(bag_a).unwrap();
        assert_eq!(container_a.stacks[0].count, 5, "A 应剩余 5 瓶");
    }

    // 验证：B 有 5 瓶
    {
        let container_b = app.world().get::<Container>(bag_b).unwrap();
        assert_eq!(container_b.stacks[0].count, 5, "B 应有 5 瓶");
    }
}

// ══════════════════════════════════════════════════════════════
// 场景二：目标容器满时转移失败
// ══════════════════════════════════════════════════════════════

#[test]
fn 目标容器满时转移失败_物品留在源容器() {
    let mut app = inventory_app();
    register_test_items(&mut app);

    // A：正常背包，放入药水
    let bag_a = spawn_container(&mut app, ContainerKind::Backpack, 20, 100.0);
    let potion_id = put_item_in_container(&mut app, bag_a, "potion_healing", 10);

    // B：容量为 0 的背包（0 表示无限制），但用 capacity=1 并塞满来测试
    // 先创建容量为 1 的背包，放入一个物品占满
    let bag_b = spawn_container(&mut app, ContainerKind::Chest, 1, 100.0);
    // 放入一个药水占位
    put_item_in_container(&mut app, bag_b, "potion_healing", 1);

    // 验证 B 已满
    {
        let container_b = app.world().get::<Container>(bag_b).unwrap();
        assert!(container_b.is_full(), "B 应已满");
    }

    // 尝试从 A 转移到 B
    app.world_mut().write_message(TransferItem {
        from_entity: bag_a,
        to_entity: bag_b,
        instance_id: potion_id,
        count: 5,
    });
    app.update();

    // 验证：A 中物品未减少
    {
        let container_a = app.world().get::<Container>(bag_a).unwrap();
        assert_eq!(container_a.stacks[0].count, 10, "A 中药水应未减少");
    }

    // 验证：B 中仍只有 1 瓶
    {
        let container_b = app.world().get::<Container>(bag_b).unwrap();
        assert_eq!(container_b.stacks[0].count, 1, "B 中应仍只有 1 瓶");
    }
}

// ══════════════════════════════════════════════════════════════
// 场景三：纯函数 transfer_item 测试
// ══════════════════════════════════════════════════════════════

#[test]
fn 纯函数transfer_item_成功转移() {
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

    let mut counter = InstanceIdCounter::default();
    let def = registry.get("potion_healing").cloned().unwrap();

    let mut from = Container::new(ContainerKind::Backpack, 20, 100.0);
    let mut to = Container::new(ContainerKind::Backpack, 20, 100.0);

    let mut stack = ItemStack::from_def(&mut counter, &def, 10);
    from.add_stack(&mut stack, &registry);

    let instance_id = from.stacks[0].instance.instance_id;

    // 转移 5 个
    let result = transfer_item(&mut from, &mut to, instance_id, 5, &registry);
    assert_eq!(result, ContainerResult::Ok);
    assert_eq!(from.stacks[0].count, 5, "源容器应剩余 5");
    assert_eq!(to.stacks[0].count, 5, "目标容器应有 5");
}

#[test]
fn 纯函数transfer_item_目标满返回full() {
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

    let mut counter = InstanceIdCounter::default();
    let def = registry.get("potion_healing").cloned().unwrap();

    let mut from = Container::new(ContainerKind::Backpack, 20, 100.0);
    // 目标容量为 1，已有一个堆叠
    let mut to = Container::new(ContainerKind::Chest, 1, 100.0);

    let mut stack_from = ItemStack::from_def(&mut counter, &def, 10);
    from.add_stack(&mut stack_from, &registry);

    let mut stack_to = ItemStack::from_def(&mut counter, &def, 1);
    to.add_stack(&mut stack_to, &registry);

    assert!(to.is_full());

    let instance_id = from.stacks[0].instance.instance_id;
    let result = transfer_item(&mut from, &mut to, instance_id, 5, &registry);
    assert_eq!(result, ContainerResult::Full);
    // 源容器未变
    assert_eq!(from.stacks[0].count, 10);
}

#[test]
fn 纯函数transfer_item_不存在返回not_found() {
    let registry = ItemRegistry::default();
    let mut from = Container::new(ContainerKind::Backpack, 20, 100.0);
    let mut to = Container::new(ContainerKind::Backpack, 20, 100.0);

    let result = transfer_item(&mut from, &mut to, 999, 1, &registry);
    assert_eq!(result, ContainerResult::NotFound);
}
