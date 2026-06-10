// 装备穿脱逻辑：EquipItem/UnequipItem Message + 穿脱系统

use super::definition::{EquipmentDef, EquipmentRegistry, EquipmentSlot};
use super::instance::{EquipmentInstance, Inventory};
use super::requirements::check_equipment_requirements;
use super::slots::EquipmentSlots;
use crate::character::PersistentTags;
use crate::character::{
    TraitCollection, TraitEffectHandlerRegistry, TraitRegistry, TraitSource, TraitTrigger,
};
use crate::core::attribute::{AttributeModifierInstance, Attributes, ModifierSource};
use crate::core::tag::{GameplayTag, GameplayTags};
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
    trait_registry: Res<TraitRegistry>,
    effect_handlers: Res<TraitEffectHandlerRegistry>,
    mut units: Query<(
        Entity,
        &mut Attributes,
        &mut GameplayTags,
        &mut PersistentTags,
        &mut EquipmentSlots,
        &mut Inventory,
        &mut TraitCollection,
    )>,
) {
    for msg in equip_reader.read() {
        if let Ok((
            entity,
            mut attrs,
            mut tags,
            mut persistent,
            mut slots,
            mut inventory,
            mut trait_collection,
        )) = units.get_mut(msg.target_entity)
        {
            // 从背包查找装备实例
            let instance = match inventory.get(msg.instance_id) {
                Some(inst) => inst.clone(),
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
            let def = match equipment_registry.get(&instance.def_id) {
                Some(d) => d,
                None => {
                    bevy::log::warn!(
                        target: "equipment",
                        def_id = %instance.def_id,
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
                    &trait_registry,
                    &effect_handlers,
                    &mut attrs,
                    &mut tags,
                    &mut persistent,
                    &mut slots,
                    &mut inventory,
                    &mut trait_collection,
                );
            }

            // 从背包移除
            inventory.remove(msg.instance_id);

            // 装备到槽位
            slots.equip(slot, msg.instance_id, def.id.clone());

            // 应用装备效果
            apply_equipment_effects(
                def,
                &instance,
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

            // 重建 GameplayTags
            rebuild_tags_from_components(
                &persistent,
                &attrs,
                &trait_collection,
                &trait_registry,
                &effect_handlers,
                &mut tags,
            );

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
    trait_registry: Res<TraitRegistry>,
    effect_handlers: Res<TraitEffectHandlerRegistry>,
    mut units: Query<(
        Entity,
        &mut Attributes,
        &mut GameplayTags,
        &mut PersistentTags,
        &mut EquipmentSlots,
        &mut Inventory,
        &mut TraitCollection,
    )>,
) {
    for msg in unequip_reader.read() {
        if let Ok((
            entity,
            mut attrs,
            mut tags,
            mut persistent,
            mut slots,
            mut inventory,
            mut trait_collection,
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
                &trait_registry,
                &effect_handlers,
                &mut attrs,
                &mut tags,
                &mut persistent,
                &mut slots,
                &mut inventory,
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

            // 重建 GameplayTags
            rebuild_tags_from_components(
                &persistent,
                &attrs,
                &trait_collection,
                &trait_registry,
                &effect_handlers,
                &mut tags,
            );

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
    _trait_registry: &TraitRegistry,
    _effect_handlers: &TraitEffectHandlerRegistry,
    attrs: &mut Attributes,
    _tags: &mut GameplayTags,
    persistent: &mut PersistentTags,
    slots: &mut EquipmentSlots,
    inventory: &mut Inventory,
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

    // 创建实例放回背包
    let instance = EquipmentInstance::new(instance_id, def_id, 100);
    inventory.add(instance);
}

/// 应用装备效果：修饰符 + 标签 + Trait
fn apply_equipment_effects(
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

/// 从组件重建 GameplayTags（三层：Trait + Equipment + Buff）
fn rebuild_tags_from_components(
    persistent: &PersistentTags,
    _attrs: &Attributes,
    _trait_collection: &TraitCollection,
    _trait_registry: &TraitRegistry,
    _effect_handlers: &TraitEffectHandlerRegistry,
    tags: &mut GameplayTags,
) {
    // 直接从 persistent 重建（不含 buff 层，buff 层由 resolve 系统管理）
    let mut new_tags = GameplayTags::default();
    new_tags.0 |= persistent.from_traits.0;
    new_tags.0 |= persistent.from_equipment.0;
    // Buff 层由 resolve_status_effects 中的 rebuild_tags 管理
    tags.0 = new_tags.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::attribute::AttributeKind;
    use crate::core::registry_loader::RegistryLoader;
    use crate::core::tag::TagName;

    /// 辅助：创建测试用属性
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

    #[test]
    fn 穿戴装备_属性修饰符生效() {
        let mut registry = EquipmentRegistry::default();
        registry.register_defaults();

        let mut attrs = make_test_attrs();
        let mut tags = GameplayTags::default();
        let mut persistent = PersistentTags::default();
        let mut slots = EquipmentSlots::default();
        let mut inventory = Inventory::new(10);
        let mut trait_collection = TraitCollection::default();

        // 创建装备实例放入背包
        let instance_id = slots.next_instance_id();
        let instance = EquipmentInstance::new(instance_id, "iron_sword".into(), 100);
        inventory.add(instance);

        let def = registry.get("iron_sword").unwrap();
        let slot = def.slot;

        // 穿戴
        apply_equipment_effects(
            def,
            inventory.get(instance_id).unwrap(),
            slot,
            &mut attrs,
            &mut persistent,
            &mut trait_collection,
            &registry,
            &TraitRegistry::default(),
            &TraitEffectHandlerRegistry::with_defaults(),
        );

        // Attack 应该增加 3
        let base_attack = 10.0; // Might*2
        assert_eq!(attrs.get(AttributeKind::Attack), base_attack + 3.0);
        // 应该有 SWORD 标签
        assert!(persistent.from_equipment.has(GameplayTag::SWORD));
        assert!(persistent.from_equipment.has(GameplayTag::MARTIAL));
    }

    #[test]
    fn 脱卸装备_属性恢复() {
        let mut registry = EquipmentRegistry::default();
        registry.register_defaults();

        let mut attrs = make_test_attrs();
        let mut tags = GameplayTags::default();
        let mut persistent = PersistentTags::default();
        let mut slots = EquipmentSlots::default();
        let mut inventory = Inventory::new(10);
        let mut trait_collection = TraitCollection::default();

        let instance_id = slots.next_instance_id();
        let instance = EquipmentInstance::new(instance_id, "iron_sword".into(), 100);
        inventory.add(instance);

        let def = registry.get("iron_sword").unwrap();
        let slot = def.slot;

        // 穿戴
        apply_equipment_effects(
            def,
            inventory.get(instance_id).unwrap(),
            slot,
            &mut attrs,
            &mut persistent,
            &mut trait_collection,
            &registry,
            &TraitRegistry::default(),
            &TraitEffectHandlerRegistry::with_defaults(),
        );

        slots.equip(slot, instance_id, "iron_sword".into());

        // 脱卸
        unequip_internal(
            Entity::from_bits(1),
            slot,
            instance_id,
            &registry,
            &TraitRegistry::default(),
            &TraitEffectHandlerRegistry::with_defaults(),
            &mut attrs,
            &mut tags,
            &mut persistent,
            &mut slots,
            &mut inventory,
            &mut trait_collection,
        );

        // Attack 恢复
        assert_eq!(attrs.get(AttributeKind::Attack), 10.0);
        // 标签清除
        assert!(!persistent.from_equipment.has(GameplayTag::SWORD));
        // 实例回到背包
        assert!(inventory.get(instance_id).is_some());
    }

    #[test]
    fn 穿戴装备_标签添加到persistent() {
        let mut registry = EquipmentRegistry::default();
        registry.register_defaults();

        let mut attrs = make_test_attrs();
        let mut tags = GameplayTags::default();
        let mut persistent = PersistentTags::default();
        let mut slots = EquipmentSlots::default();
        let mut inventory = Inventory::new(10);
        let mut trait_collection = TraitCollection::default();

        let instance_id = slots.next_instance_id();
        let instance = EquipmentInstance::new(instance_id, "flame_dragon_sword".into(), 100);
        inventory.add(instance);

        let def = registry.get("flame_dragon_sword").unwrap();

        apply_equipment_effects(
            def,
            inventory.get(instance_id).unwrap(),
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
        let mut inventory = Inventory::new(10);
        let mut trait_collection = TraitCollection::default();

        let instance_id = slots.next_instance_id();
        let instance = EquipmentInstance::new(instance_id, "flame_dragon_sword".into(), 100);
        inventory.add(instance);

        let def = registry.get("flame_dragon_sword").unwrap();

        apply_equipment_effects(
            def,
            inventory.get(instance_id).unwrap(),
            def.slot,
            &mut attrs,
            &mut persistent,
            &mut trait_collection,
            &registry,
            &TraitRegistry::default(),
            &TraitEffectHandlerRegistry::with_defaults(),
        );

        // 应该有 2 个 trait
        assert!(trait_collection.has("flaming_weapon"));
        assert!(trait_collection.has("dragon_bane"));
    }
}
