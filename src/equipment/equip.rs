// 装备穿脱逻辑：EquipItem/UnequipItem Message + 穿脱系统

use super::definition::{EquipmentDef, EquipmentRegistry, EquipmentSlot};
use super::instance::EquipmentInstance;
use super::requirements::check_equipment_requirements;
use super::slots::EquipmentSlots;
use crate::buff::ActiveBuffs;
use crate::buff::resolve::rebuild_tags as rebuild_tags_with_buffs;
use crate::character::PersistentTags;
use crate::character::{
    TraitCollection, TraitEffectHandlerRegistry, TraitRegistry, TraitSource, TraitTrigger,
};
use crate::core::attribute::{AttributeModifierInstance, Attributes, ModifierSource};
use crate::core::tag::GameplayTags;
use crate::inventory::container::Container;
use crate::inventory::definition::ItemRegistry;
use crate::inventory::instance::{ItemInstance, ItemStack};
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

/// 穿戴装备消息
#[derive(Message, Debug, Clone)]
pub struct EquipItem {
    pub target_entity: Entity,
    pub instance_id: u64,
}

/// 脱卸装备消息
#[derive(Message, Debug, Clone)]
pub struct UnequipItem {
    pub target_entity: Entity,
    pub slot: EquipmentSlot,
}

/// 装备已穿戴消息（供 UI/日志响应）
#[derive(Message, Debug, Clone)]
pub struct ItemEquipped {
    pub entity: Entity,
    pub slot: EquipmentSlot,
    pub def_id: String,
    pub instance_id: u64,
}

/// 装备已脱卸消息
#[derive(Message, Debug, Clone)]
pub struct ItemUnequipped {
    pub entity: Entity,
    pub slot: EquipmentSlot,
    pub def_id: String,
}

/// 穿戴失败消息（需求不满足）
#[derive(Message, Debug, Clone)]
pub struct EquipFailed {
    pub entity: Entity,
    pub instance_id: u64,
    pub reason: String,
}

/// 处理 EquipItem 消息的系统
pub fn equip_item_system(
    mut equip_reader: MessageReader<EquipItem>,
    mut equipped_writer: MessageWriter<ItemEquipped>,
    mut failed_writer: MessageWriter<EquipFailed>,
    equipment_registry: Res<EquipmentRegistry>,
    item_registry: Res<ItemRegistry>,
    trait_registry: Res<TraitRegistry>,
    effect_handlers: Res<TraitEffectHandlerRegistry>,
    mut units: Query<(
        Entity,
        &mut Attributes,
        &mut GameplayTags,
        &mut PersistentTags,
        &mut EquipmentSlots,
        &mut Container,
        &mut TraitCollection,
        &ActiveBuffs,
    )>,
) {
    for msg in equip_reader.read() {
        if let Ok((
            entity,
            mut attrs,
            mut tags,
            mut persistent,
            mut slots,
            mut container,
            mut trait_collection,
            buffs,
        )) = units.get_mut(msg.target_entity)
        {
            // 从背包查找装备堆叠
            let stack = match container.get(msg.instance_id) {
                Some(s) => s.clone(),
                None => {
                    bevy::log::warn!(
                        target: "equipment",
                        entity = ?msg.target_entity,
                        instance_id = msg.instance_id,
                        "背包中未找到装备实例"
                    );
                    continue;
                }
            };

            // 从注册表查找装备定义
            let def = match equipment_registry.get(&stack.instance.def_id) {
                Some(d) => d,
                None => {
                    bevy::log::warn!(
                        target: "equipment",
                        def_id = %stack.instance.def_id,
                        "装备定义不存在"
                    );
                    continue;
                }
            };

            let slot = def.slot;

            // 需求检查
            let check = check_equipment_requirements(def, &attrs, &tags);
            if !check.is_satisfied() {
                let reason = match &check {
                    super::requirements::RequirementCheckResult::Failed(r) => r.clone(),
                    _ => String::new(),
                };
                bevy::log::warn!(
                    target: "equipment",
                    entity = ?entity,
                    def_id = %def.id,
                    reason = %reason,
                    "装备需求不满足"
                );
                failed_writer.write(EquipFailed {
                    entity,
                    instance_id: msg.instance_id,
                    reason,
                });
                continue;
            }

            // 如果槽位已占用，先脱卸旧装备
            if let Some(old_instance_id) = slots.get(slot) {
                unequip_internal(
                    entity,
                    slot,
                    old_instance_id,
                    &equipment_registry,
                    &item_registry,
                    &trait_registry,
                    &effect_handlers,
                    &mut attrs,
                    &mut tags,
                    &mut persistent,
                    &mut slots,
                    &mut container,
                    &mut trait_collection,
                );
            }

            // 从背包移除
            container.remove(msg.instance_id);

            // 装备到槽位
            slots.equip(slot, msg.instance_id, def.id.clone());

            // 应用装备效果（构造 EquipmentInstance 用于兼容）
            let eq_instance = EquipmentInstance::new(
                msg.instance_id,
                stack.instance.def_id.clone(),
                stack.instance.durability,
            );
            apply_equipment_effects(
                def,
                &eq_instance,
                slot,
                &mut attrs,
                &mut persistent,
                &mut trait_collection,
                &equipment_registry,
                &trait_registry,
                &effect_handlers,
            );

            // 重建 Trait 效果（装备可能添加了新 Trait）
            rebuild_trait_effects(
                &trait_collection,
                &trait_registry,
                &effect_handlers,
                &mut attrs,
                &mut persistent,
            );

            // 重建 GameplayTags（三层：Trait + Equipment + Buff）
            rebuild_tags_with_buffs(&buffs, &mut tags, &persistent);

            bevy::log::trace!(
                target: "equipment",
                entity = ?entity,
                def_id = %def.id,
                slot = ?slot,
                "装备已穿戴"
            );

            equipped_writer.write(ItemEquipped {
                entity,
                slot,
                def_id: def.id.clone(),
                instance_id: msg.instance_id,
            });
        }
    }
}

/// 处理 UnequipItem 消息的系统
pub fn unequip_item_system(
    mut unequip_reader: MessageReader<UnequipItem>,
    mut unequipped_writer: MessageWriter<ItemUnequipped>,
    equipment_registry: Res<EquipmentRegistry>,
    item_registry: Res<ItemRegistry>,
    trait_registry: Res<TraitRegistry>,
    effect_handlers: Res<TraitEffectHandlerRegistry>,
    mut units: Query<(
        Entity,
        &mut Attributes,
        &mut GameplayTags,
        &mut PersistentTags,
        &mut EquipmentSlots,
        &mut Container,
        &mut TraitCollection,
        &ActiveBuffs,
    )>,
) {
    for msg in unequip_reader.read() {
        if let Ok((
            entity,
            mut attrs,
            mut tags,
            mut persistent,
            mut slots,
            mut container,
            mut trait_collection,
            buffs,
        )) = units.get_mut(msg.target_entity)
        {
            // 检查槽位是否有装备
            let instance_id = match slots.get(msg.slot) {
                Some(id) => id,
                None => continue,
            };

            let def_id = slots
                .get_def_id(msg.slot)
                .map(|s| s.to_string())
                .unwrap_or_default();

            unequip_internal(
                entity,
                msg.slot,
                instance_id,
                &equipment_registry,
                &item_registry,
                &trait_registry,
                &effect_handlers,
                &mut attrs,
                &mut tags,
                &mut persistent,
                &mut slots,
                &mut container,
                &mut trait_collection,
            );

            // 重建 Trait 效果（脱卸可能移除了 Trait）
            rebuild_trait_effects(
                &trait_collection,
                &trait_registry,
                &effect_handlers,
                &mut attrs,
                &mut persistent,
            );

            // 重建 GameplayTags（三层：Trait + Equipment + Buff）
            rebuild_tags_with_buffs(&buffs, &mut tags, &persistent);

            bevy::log::trace!(
                target: "equipment",
                entity = ?entity,
                slot = ?msg.slot,
                "装备已脱卸"
            );

            unequipped_writer.write(ItemUnequipped {
                entity,
                slot: msg.slot,
                def_id,
            });
        }
    }
}

/// 内部脱卸逻辑：移除修饰符 + 标签 + Trait，实例放回背包
fn unequip_internal(
    _entity: Entity,
    slot: EquipmentSlot,
    instance_id: u64,
    equipment_registry: &EquipmentRegistry,
    item_registry: &ItemRegistry,
    _trait_registry: &TraitRegistry,
    _effect_handlers: &TraitEffectHandlerRegistry,
    attrs: &mut Attributes,
    _tags: &mut GameplayTags,
    persistent: &mut PersistentTags,
    slots: &mut EquipmentSlots,
    container: &mut Container,
    trait_collection: &mut TraitCollection,
) {
    // 从槽位获取 def_id（装备穿戴时已不在背包中）
    let def_id = slots
        .get_def_id(slot)
        .map(|s| s.to_string())
        .unwrap_or_default();

    let def = match equipment_registry.get(&def_id) {
        Some(d) => d,
        None => return,
    };

    // 移除装备修饰符（Equipment 区间）
    let source = ModifierSource::equipment_source(instance_id);
    attrs.remove_modifiers_from(source);

    // 移除装备授予的标签
    for tag_name in &def.tags {
        persistent.from_equipment.remove(tag_name.to_tag());
    }

    // 移除装备授予的 Trait
    trait_collection.remove_by_source(&TraitSource::Equipment { slot });

    // 清除槽位
    slots.unequip(slot);

    // 创建 ItemStack 放回背包
    let item_instance = ItemInstance::from_def(
        instance_id,
        item_registry.get(&def_id).unwrap_or_else(|| {
            // fallback：如果没有 ItemDef，构造一个最小化的
            unreachable!("装备定义应在 ItemRegistry 中存在")
        }),
    );
    let mut stack = ItemStack::new(item_instance, 1);
    container.add_stack(&mut stack, item_registry);
}

/// 应用装备效果：修饰符 + 标签 + Trait
pub fn apply_equipment_effects(
    def: &EquipmentDef,
    instance: &EquipmentInstance,
    slot: EquipmentSlot,
    attrs: &mut Attributes,
    persistent: &mut PersistentTags,
    trait_collection: &mut TraitCollection,
    _equipment_registry: &EquipmentRegistry,
    _trait_registry: &TraitRegistry,
    _effect_handlers: &TraitEffectHandlerRegistry,
) {
    // 1. 添加修饰符（Equipment 区间）
    let source = ModifierSource::equipment_source(instance.instance_id);
    attrs.add_modifiers_from_def(&def.modifiers, source);

    // 2. 添加标签到 PersistentTags.from_equipment
    for tag_name in &def.tags {
        persistent.from_equipment.add(tag_name.to_tag());
    }

    // 3. 添加 Trait 到 TraitCollection
    for trait_id in &def.traits {
        trait_collection.add_entry(trait_id.clone(), TraitSource::Equipment { slot });
    }
}

/// 全量重建 Trait 效果：清除旧 Trait 修饰符，重新应用所有被动 Trait
/// 在装备穿脱后调用，确保 Trait 授予的标签和属性修饰符正确
pub fn rebuild_trait_effects(
    trait_collection: &TraitCollection,
    trait_registry: &TraitRegistry,
    effect_handlers: &TraitEffectHandlerRegistry,
    attrs: &mut Attributes,
    persistent: &mut PersistentTags,
) {
    // 1. 清除所有 Trait 来源的修饰符
    attrs.remove_trait_modifiers();

    // 2. 清除 Trait 授予的标签
    persistent.from_traits = GameplayTags::default();

    // 3. 重新应用所有被动 Trait 的效果
    let mut trait_source_index = 0u64;
    for entry in &trait_collection.entries {
        if let Some(trait_data) = trait_registry.get(&entry.trait_id) {
            if trait_data.trigger != TraitTrigger::Passive {
                continue;
            }
            // 授予标签
            for tag in trait_data.granted_tags(effect_handlers) {
                persistent.from_traits.add(tag);
            }
            // 授予属性修饰符
            let source = ModifierSource::trait_source(trait_source_index);
            for mod_def in trait_data.attribute_modifiers(effect_handlers) {
                attrs.add_modifier(AttributeModifierInstance {
                    kind: mod_def.kind,
                    op: mod_def.op,
                    value: mod_def.value,
                    source,
                });
            }
            trait_source_index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::attribute::AttributeKind;
    use crate::core::registry_loader::RegistryLoader;
    use crate::core::tag::TagName;
    use crate::inventory::definition::{ItemDef, ItemType};

    fn make_test_attrs() -> Attributes {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Might, 5.0);
        attrs.set_base(AttributeKind::Vitality, 5.0);
        attrs.set_base(AttributeKind::Agility, 6.0);
        attrs.set_base(AttributeKind::Dexterity, 3.0);
        attrs.set_base(AttributeKind::Intelligence, 2.0);
        attrs.set_base(AttributeKind::Willpower, 3.0);
        attrs.set_base(AttributeKind::Presence, 2.0);
        attrs.set_base(AttributeKind::Luck, 2.0);
        attrs.set_base_attack_range(1);
        attrs.fill_vital_resources();
        attrs
    }

    /// 辅助：创建包含装备定义的 ItemRegistry
    fn make_item_registry(equipment_registry: &EquipmentRegistry) -> ItemRegistry {
        let mut item_registry = ItemRegistry::default();
        for def in equipment_registry.iter() {
            let item_def = ItemDef {
                version: 1,
                id: def.id.clone(),
                name: def.name.clone(),
                description: def.description.clone(),
                item_type: ItemType::Equipment,
                rarity: def.rarity,
                tags: def.tags.clone(),
                stack_size: 1,
                weight: def.weight,
                modifiers: def.modifiers.clone(),
                traits: def.traits.clone(),
                requirements: def.requirements.clone(),
                slot: Some(def.slot),
                use_effects: vec![],
                container_capacity: None,
                container_max_weight: None,
            };
            item_registry.register(item_def);
        }
        item_registry
    }

    #[test]
    fn 穿戴装备_属性修饰符生效() {
        let mut registry = EquipmentRegistry::default();
        registry.register_defaults();

        let mut attrs = make_test_attrs();
        let mut tags = GameplayTags::default();
        let mut persistent = PersistentTags::default();
        let mut slots = EquipmentSlots::default();
        let mut trait_collection = TraitCollection::default();

        let instance_id = slots.next_instance_id();
        let instance = EquipmentInstance::new(instance_id, "iron_sword".into(), 100);

        let def = registry.get("iron_sword").unwrap();
        let slot = def.slot;

        apply_equipment_effects(
            def,
            &instance,
            slot,
            &mut attrs,
            &mut persistent,
            &mut trait_collection,
            &registry,
            &TraitRegistry::default(),
            &TraitEffectHandlerRegistry::with_defaults(),
        );

        let base_attack = 10.0;
        assert_eq!(attrs.get(AttributeKind::Attack), base_attack + 3.0);
        assert!(persistent.from_equipment.has(GameplayTag::SWORD));
        assert!(persistent.from_equipment.has(GameplayTag::MARTIAL));
    }

    #[test]
    fn 脱卸装备_属性恢复() {
        let mut registry = EquipmentRegistry::default();
        registry.register_defaults();
        let item_registry = make_item_registry(&registry);

        let mut attrs = make_test_attrs();
        let mut tags = GameplayTags::default();
        let mut persistent = PersistentTags::default();
        let mut slots = EquipmentSlots::default();
        let mut container = Container::backpack();
        let mut trait_collection = TraitCollection::default();

        let instance_id = slots.next_instance_id();
        let instance = EquipmentInstance::new(instance_id, "iron_sword".into(), 100);

        let def = registry.get("iron_sword").unwrap();
        let slot = def.slot;

        apply_equipment_effects(
            def,
            &instance,
            slot,
            &mut attrs,
            &mut persistent,
            &mut trait_collection,
            &registry,
            &TraitRegistry::default(),
            &TraitEffectHandlerRegistry::with_defaults(),
        );

        slots.equip(slot, instance_id, "iron_sword".into());

        unequip_internal(
            Entity::PLACEHOLDER,
            slot,
            instance_id,
            &registry,
            &item_registry,
            &TraitRegistry::default(),
            &TraitEffectHandlerRegistry::with_defaults(),
            &mut attrs,
            &mut tags,
            &mut persistent,
            &mut slots,
            &mut container,
            &mut trait_collection,
        );

        assert_eq!(attrs.get(AttributeKind::Attack), 10.0);
        assert!(!persistent.from_equipment.has(GameplayTag::SWORD));
        assert!(container.get(instance_id).is_some());
    }

    #[test]
    fn 穿戴装备_标签添加到persistent() {
        let mut registry = EquipmentRegistry::default();
        registry.register_defaults();

        let mut attrs = make_test_attrs();
        let mut tags = GameplayTags::default();
        let mut persistent = PersistentTags::default();
        let mut slots = EquipmentSlots::default();
        let mut trait_collection = TraitCollection::default();

        let instance_id = slots.next_instance_id();
        let instance = EquipmentInstance::new(instance_id, "flame_dragon_sword".into(), 100);

        let def = registry.get("flame_dragon_sword").unwrap();

        apply_equipment_effects(
            def,
            &instance,
            def.slot,
            &mut attrs,
            &mut persistent,
            &mut trait_collection,
            &registry,
            &TraitRegistry::default(),
            &TraitEffectHandlerRegistry::with_defaults(),
        );

        assert!(persistent.from_equipment.has(GameplayTag::SWORD));
        assert!(persistent.from_equipment.has(GameplayTag::FIRE));
        assert!(persistent.from_equipment.has(GameplayTag::MARTIAL));
        assert!(persistent.from_equipment.has(GameplayTag::TWO_HANDED));
    }

    #[test]
    fn 穿戴装备_trait添加到集合() {
        let mut registry = EquipmentRegistry::default();
        registry.register_defaults();

        let mut attrs = make_test_attrs();
        let mut tags = GameplayTags::default();
        let mut persistent = PersistentTags::default();
        let mut slots = EquipmentSlots::default();
        let mut trait_collection = TraitCollection::default();

        let instance_id = slots.next_instance_id();
        let instance = EquipmentInstance::new(instance_id, "flame_dragon_sword".into(), 100);

        let def = registry.get("flame_dragon_sword").unwrap();

        apply_equipment_effects(
            def,
            &instance,
            def.slot,
            &mut attrs,
            &mut persistent,
            &mut trait_collection,
            &registry,
            &TraitRegistry::default(),
            &TraitEffectHandlerRegistry::with_defaults(),
        );

        assert!(trait_collection.has("flaming_weapon"));
        assert!(trait_collection.has("dragon_bane"));
    }
}
