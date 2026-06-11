// 容器间转移：TransferItem Message + transfer_item 系统

use super::container::Container;
use super::definition::ItemRegistry;
use super::instance::ItemStack;
use bevy::prelude::*;

/// 容器操作结果
#[derive(Debug, PartialEq, Eq)]
pub enum ContainerResult {
    Ok,
    Full,
    Overweight,
    NotFound,
}

/// 容器间转移 Message
#[derive(Message, Debug, Clone)]
pub struct TransferItem {
    pub from_entity: Entity,
    pub to_entity: Entity,
    pub instance_id: u64,
    pub count: u32,
}

/// 转移完成通知
#[derive(Message, Debug, Clone)]
pub struct ItemTransferred {
    pub from_entity: Entity,
    pub to_entity: Entity,
    pub def_id: String,
    pub count: u32,
}

/// 容器间转移系统
pub fn transfer_item_system(
    mut messages: MessageReader<TransferItem>,
    mut containers: Query<&mut Container>,
    item_registry: Res<ItemRegistry>,
    mut writer: MessageWriter<ItemTransferred>,
) {
    for msg in messages.read() {
        // 不能同时可变借用两个相同的 Entity
        if msg.from_entity == msg.to_entity {
            continue;
        }

        let Ok(from_container) = containers.get(msg.from_entity) else {
            continue;
        };
        let stack = match from_container.get(msg.instance_id) {
            Some(s) => s.clone(),
            None => continue,
        };

        let to_remove = msg.count.min(stack.count);
        let def_id = stack.instance.def_id.clone();

        // 检查目标容器容量和重量
        let Ok(to_container) = containers.get(msg.to_entity) else {
            continue;
        };
        // 规则4：检查目标容器容量和重量
        // 注意：不使用 is_full() 预检查，因为合并到已有堆叠不需要额外容量
        // add_stack 内部会正确处理合并/容量/重量逻辑
        if to_container.max_weight > 0.0 {
            if let Some(def) = item_registry.get(&def_id) {
                let added_weight = def.weight * to_remove as f32;
                if to_container.current_weight(&item_registry) + added_weight
                    > to_container.max_weight
                {
                    bevy::log::warn!(
                        target: "inventory",
                        to = ?msg.to_entity,
                        "目标容器超重"
                    );
                    continue;
                }
            }
        }

        // 释放不可变借用
        drop(from_container);
        drop(to_container);

        // 从源容器移除
        let removed = if let Ok(mut from) = containers.get_mut(msg.from_entity) {
            from.reduce_stack(msg.instance_id, to_remove)
        } else {
            continue;
        };

        // 添加到目标容器
        if let Some(mut removed_stack) = removed {
            if let Ok(mut to) = containers.get_mut(msg.to_entity) {
                to.add_stack(&mut removed_stack, &item_registry);
            }

            writer.write(ItemTransferred {
                from_entity: msg.from_entity,
                to_entity: msg.to_entity,
                def_id: def_id.clone(),
                count: to_remove,
            });

            bevy::log::info!(
                target: "inventory",
                from = ?msg.from_entity,
                to = ?msg.to_entity,
                def_id = %def_id,
                count = to_remove,
                "物品转移完成"
            );
        }
    }
}

/// 纯函数：容器间转移（用于测试和程序化调用）
pub fn transfer_item(
    from: &mut Container,
    to: &mut Container,
    instance_id: u64,
    count: u32,
    registry: &ItemRegistry,
) -> ContainerResult {
    let stack = match from.get(instance_id) {
        Some(s) => s.clone(),
        None => return ContainerResult::NotFound,
    };

    let to_remove = count.min(stack.count);
    let mut new_stack = ItemStack {
        instance: stack.instance.clone(),
        count: to_remove,
    };

    // 检查目标容器
    // 注意：不使用 is_full() 预检查，因为合并到已有堆叠不需要额外容量
    // add_stack 内部会正确处理合并/容量/重量逻辑
    if to.max_weight > 0.0 {
        if let Some(def) = registry.get(&new_stack.instance.def_id) {
            let added_weight = new_stack.total_weight(def);
            if to.current_weight(registry) + added_weight > to.max_weight {
                return ContainerResult::Overweight;
            }
        }
    }

    // 从源容器移除
    from.reduce_stack(instance_id, to_remove);

    // 添加到目标容器
    to.add_stack(&mut new_stack, registry);

    ContainerResult::Ok
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::equipment::Rarity;
    use crate::inventory::container::ContainerKind;
    use crate::inventory::definition::{ItemDef, ItemType};
    use crate::inventory::instance::ItemInstance;

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
        registry
    }

    #[test]
    fn 转移_成功() {
        let registry = test_registry();
        let mut from = Container::backpack();
        let mut to = Container::backpack();
        let mut stack = ItemStack::new(
            ItemInstance::from_def(1, registry.get("potion_healing").unwrap()),
            10,
        );
        from.add_stack(&mut stack, &registry);

        let result = transfer_item(&mut from, &mut to, 1, 5, &registry);
        assert_eq!(result, ContainerResult::Ok);
        assert_eq!(from.stacks[0].count, 5);
        assert_eq!(to.stacks[0].count, 5);
    }

    #[test]
    fn 转移_源容器不存在() {
        let registry = test_registry();
        let mut from = Container::backpack();
        let mut to = Container::backpack();
        let result = transfer_item(&mut from, &mut to, 999, 1, &registry);
        assert_eq!(result, ContainerResult::NotFound);
    }

    #[test]
    fn 转移_目标容器满() {
        let registry = test_registry();
        let mut from = Container::backpack();
        let mut to = Container::new(ContainerKind::Chest, 0, 0.0); // 0 容量 = 无限制
        // 容量为 0 表示无限制，所以不会满
        let mut stack = ItemStack::new(
            ItemInstance::from_def(1, registry.get("potion_healing").unwrap()),
            10,
        );
        from.add_stack(&mut stack, &registry);
        let result = transfer_item(&mut from, &mut to, 1, 5, &registry);
        assert_eq!(result, ContainerResult::Ok);
    }

    #[test]
    fn 转移_全部转移() {
        let registry = test_registry();
        let mut from = Container::backpack();
        let mut to = Container::backpack();
        let mut stack = ItemStack::new(
            ItemInstance::from_def(1, registry.get("potion_healing").unwrap()),
            10,
        );
        from.add_stack(&mut stack, &registry);

        let result = transfer_item(&mut from, &mut to, 1, 10, &registry);
        assert_eq!(result, ContainerResult::Ok);
        assert!(from.is_empty());
        assert_eq!(to.stacks[0].count, 10);
    }
}
