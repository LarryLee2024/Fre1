// 消耗品使用系统：UseItem Message + use_item_system

use super::container::Container;
use super::definition::{ItemDef, ItemRegistry, ItemType, UseEffect};
use crate::buff::{ActiveBuffs, BuffRegistry, apply_buff};
use crate::core::attribute::{AttributeKind, Attributes};
use crate::core::tag::GameplayTags;
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
    mut units: Query<(&mut Attributes, &mut ActiveBuffs, &mut GameplayTags)>,
    item_registry: Res<ItemRegistry>,
    buff_registry: Res<BuffRegistry>,
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

        // 应用使用效果（通过统一查询获取 Attributes + ActiveBuffs + GameplayTags）
        if let Ok((mut attrs, mut buffs, mut tags)) = units.get_mut(msg.user_entity) {
            apply_use_effects(&def, &mut attrs, &mut buffs, &mut tags, msg.user_entity, &buff_registry);
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

/// 应用消耗品效果
/// RestoreVital：直接修改生命资源当前值（set_vital）
/// ApplyBuff：通过 apply_buff() 统一管线处理（不变量6合规）
fn apply_use_effects(
    def: &ItemDef,
    attrs: &mut Attributes,
    buffs: &mut ActiveBuffs,
    tags: &mut GameplayTags,
    user_entity: Entity,
    buff_registry: &BuffRegistry,
) {
    for effect in &def.use_effects {
        match effect {
            UseEffect::RestoreVital { kind, value } => {
                // 直接修改生命资源当前值，不通过修饰符管线
                // 生命资源恢复是即时效果，不是属性修饰
                let current = attrs.get(*kind);
                let max = attrs.get(match kind {
                    AttributeKind::Hp => AttributeKind::MaxHp,
                    AttributeKind::Mp => AttributeKind::MaxMp,
                    AttributeKind::Stamina => AttributeKind::MaxStamina,
                    _ => *kind,
                });
                attrs.set_vital(*kind, (current + value).min(max));
            }
            UseEffect::ApplyBuff { buff_id, duration } => {
                // 通过 apply_buff() 统一管线处理（修饰符+标签+同源刷新）
                if let Some(buff_data) = buff_registry.get(buff_id) {
                    apply_buff(buffs, attrs, tags, buff_data, Some(user_entity), *duration);
                }
            }
            // GrantTempTrait 和 CastSkill 由其他系统处理
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buff::BuffData;
    use crate::core::attribute::ModifierOp;
    use crate::core::tag::GameplayTag;
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

    fn test_buff_registry() -> BuffRegistry {
        let mut registry = BuffRegistry::default();
        registry.register(BuffData {
            id: "attack_up".into(),
            name: "攻+5".into(),
            default_duration: 3,
            modifiers: vec![crate::core::attribute::AttributeModifierDef {
                kind: AttributeKind::Attack,
                op: ModifierOp::Add,
                value: 5.0,
            }],
            tags: vec![GameplayTag::BUFF],
            dot_damage: 0,
            hot_heal: 0,
            is_stun: false,
            is_cleanse: false,
            is_buff: true,
        });
        registry
    }

    #[test]
    fn 消耗品_应用恢复效果() {
        let def = test_consumable_def();
        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        // 先扣血，验证恢复
        attrs.set_vital(AttributeKind::Hp, 10.0);
        let mut buffs = ActiveBuffs::default();
        let mut tags = GameplayTags::default();
        let buff_registry = BuffRegistry::default();
        let user = Entity::from_bits(1);

        apply_use_effects(&def, &mut attrs, &mut buffs, &mut tags, user, &buff_registry);

        // HP 应恢复 50，但不超过 MaxHp
        let hp = attrs.get(AttributeKind::Hp);
        let max_hp = attrs.get(AttributeKind::MaxHp);
        assert_eq!(hp, (10.0 + 50.0).min(max_hp));
    }

    #[test]
    fn 消耗品_应用buff效果() {
        let mut def = test_consumable_def();
        def.use_effects = vec![UseEffect::ApplyBuff {
            buff_id: "attack_up".into(),
            duration: 2,
        }];

        let mut attrs = Attributes::default();
        attrs.fill_vital_resources();
        let mut buffs = ActiveBuffs::default();
        let mut tags = GameplayTags::default();
        let buff_registry = test_buff_registry();
        let user = Entity::from_bits(1);

        apply_use_effects(&def, &mut attrs, &mut buffs, &mut tags, user, &buff_registry);

        // Buff 实例应被添加
        assert_eq!(buffs.len(), 1);
        assert_eq!(buffs.instances[0].buff_id, "attack_up");
        // 标签应被添加
        assert!(tags.has(GameplayTag::BUFF));
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

        let mut attrs = Attributes::default();
        let mut buffs = ActiveBuffs::default();
        let mut tags = GameplayTags::default();
        let buff_registry = BuffRegistry::default();
        let user = Entity::from_bits(1);

        apply_use_effects(&def, &mut attrs, &mut buffs, &mut tags, user, &buff_registry);

        // 装备没有 use_effects，不应修改任何状态
        assert!(buffs.is_empty());
    }
}
