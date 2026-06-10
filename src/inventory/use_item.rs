// 消耗品使用系统：UseItem Message + use_item_system

use super::container::Container;
use super::definition::{ItemDef, ItemRegistry, ItemType, UseEffect};
use super::instance::ItemStack;
use crate::buff::{ActiveBuffs, BuffInstance};
use crate::core::attribute::{
    AttributeKind, AttributeModifierInstance, Attributes, BuffInstanceId, ModifierOp,
    ModifierSource,
};
use bevy::prelude::*;

/// 使用消耗品 Message
#[derive(Message, Debug, Clone)]
pub struct UseItem {
    pub user_entity: Entity,
    pub container_entity: Entity,
    pub instance_id: u64,
}

/// 物品使用完成通知
#[derive(Message, Debug, Clone)]
pub struct ItemUsed {
    pub user_entity: Entity,
    pub def_id: String,
}

/// 使用消耗品系统
pub fn use_item_system(
    mut messages: MessageReader<UseItem>,
    mut containers: Query<&mut Container>,
    mut units: Query<&mut Attributes>,
    mut buffs: Query<&mut ActiveBuffs>,
    item_registry: Res<ItemRegistry>,
    mut used_writer: MessageWriter<ItemUsed>,
) {
    for msg in messages.read() {
        let Ok(mut container) = containers.get_mut(msg.container_entity) else {
            continue;
        };
        let Some(stack) = container.get(msg.instance_id).cloned() else {
            continue;
        };
        let Some(def) = item_registry.get(&stack.instance.def_id) else {
            continue;
        };

        if def.item_type != ItemType::Consumable {
            continue;
        }

        // 应用使用效果
        if let Ok(mut attrs) = units.get_mut(msg.user_entity) {
            apply_use_effects(&def, &stack, &mut attrs, msg.user_entity);
        }

        // 如果有 Buff 效果，通过 Buff 系统处理
        if let Ok(mut active_buffs) = buffs.get_mut(msg.user_entity) {
            for effect in &def.use_effects {
                if let UseEffect::ApplyBuff { buff_id, duration } = effect {
                    let instance_id = active_buffs.next_instance_id();
                    let buff_instance = BuffInstance {
                        instance_id,
                        buff_id: buff_id.clone(),
                        name: buff_id.clone(),
                        remaining_turns: *duration,
                        source_entity: Some(msg.user_entity),
                        tags: vec![],
                        is_buff: true,
                        dot_damage: 0,
                        hot_heal: 0,
                    };
                    active_buffs.add(buff_instance);
                }
            }
        }

        // 消耗一个
        container.reduce_stack(msg.instance_id, 1);

        used_writer.write(ItemUsed {
            user_entity: msg.user_entity,
            def_id: def.id.clone(),
        });

        bevy::log::info!(
            target: "inventory",
            entity = ?msg.user_entity,
            item_id = %def.id,
            "消耗品已使用"
        );
    }
}

/// 应用消耗品效果到属性
fn apply_use_effects(def: &ItemDef, stack: &ItemStack, attrs: &mut Attributes, _user: Entity) {
    for effect in &def.use_effects {
        if let UseEffect::RestoreVital { kind, value } = effect {
            let source = ModifierSource::buff_source(stack.instance.instance_id);
            attrs.add_modifier(AttributeModifierInstance {
                kind: *kind,
                op: ModifierOp::Add,
                value: *value,
                source,
            });
        }
        // ApplyBuff 和 GrantTempTrait 由 Buff 系统处理
        // CastSkill 由技能系统处理
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::equipment::Rarity;
    use crate::inventory::instance::ItemInstance;

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
            use_effects: vec![UseEffect::RestoreVital {
                kind: AttributeKind::Hp,
                value: 50.0,
            }],
            container_capacity: None,
            container_max_weight: None,
        }
    }

    #[test]
    fn 消耗品_应用恢复效果() {
        let def = test_consumable_def();
        let stack = ItemStack::new(ItemInstance::from_def(1, &def), 10);
        let mut attrs = Attributes::default();
        apply_use_effects(&def, &stack, &mut attrs, Entity::PLACEHOLDER);
        // 验证修饰符已添加
        let hp_mods: Vec<_> = attrs
            .modifiers
            .iter()
            .filter(|m| m.kind == AttributeKind::Hp)
            .collect();
        assert_eq!(hp_mods.len(), 1);
        assert_eq!(hp_mods[0].value, 50.0);
    }

    #[test]
    fn 消耗品_非消耗品不处理() {
        let def = ItemDef {
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
        };
        let stack = ItemStack::new(ItemInstance::from_def(1, &def), 1);
        let mut attrs = Attributes::default();
        apply_use_effects(&def, &stack, &mut attrs, Entity::PLACEHOLDER);
        // 装备没有 use_effects，不应添加修饰符
        assert!(attrs.modifiers.is_empty());
    }
}
