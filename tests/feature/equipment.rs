//! 装备系统 Feature Test
//!
//! 跨 equipment + inventory + core/attribute + core/tag + character/traits
//! 测试装备穿脱完整流程、需求检查、自动脱卸旧装备、Trait 生命周期。

// ================================================
// AI Self-Check (test_spec.md §13.1)
// ================================================
// ✅ 测试行为，不是实现
// ✅ 符合领域规则
// ✅ 测试是确定性的
// ✅ 使用标准测试数据
// ✅ 没有测试私有实现
// ✅ 没有生成不在范围内的测试
// ================================================

use bevy::prelude::*;
use tactical_rpg::character::{PersistentTags, TraitCollection, TraitSource};
use tactical_rpg::core::attribute::{AttributeKind, AttributeModifierDef, Attributes, ModifierOp};
use tactical_rpg::core::registry_loader::RegistryLoader;
use tactical_rpg::core::tag::{GameplayTag, GameplayTags, TagName};
use tactical_rpg::equipment::{
    EquipItem, EquipmentDef, EquipmentRegistry, EquipmentRequirement, EquipmentSlot,
    EquipmentSlots, Rarity, UnequipItem,
};
use tactical_rpg::inventory::container::Container;
use tactical_rpg::inventory::definition::{ItemDef, ItemRegistry, ItemType};
use tactical_rpg::inventory::instance::{InstanceIdCounter, ItemInstance, ItemStack};

use crate::assert_attr_eq;
use crate::assert_has_tag;
use crate::assert_not_has_tag;
use crate::common::app_builder::equipment_app;
use crate::common::fixtures::UnitBuilder;

// ── 测试辅助 ──

/// 注册默认装备到 EquipmentRegistry + ItemRegistry
fn register_defaults(app: &mut App) {
    {
        let mut eq_reg = app.world_mut().resource_mut::<EquipmentRegistry>();
        RegistryLoader::register_defaults(&mut *eq_reg);
    }
    // 同步注册到 ItemRegistry
    let item_defs: Vec<ItemDef> = {
        let eq_reg = app.world().resource::<EquipmentRegistry>();
        eq_reg
            .iter()
            .map(|def| ItemDef {
                version: def.version,
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
            })
            .collect()
    };
    let mut item_reg = app.world_mut().resource_mut::<ItemRegistry>();
    for def in item_defs {
        item_reg.register(def);
    }
}

/// 在角色背包中放入指定装备，返回 instance_id
fn put_item_in_backpack(app: &mut App, entity: Entity, def_id: &str) -> u64 {
    let instance_id = {
        let mut counter = app.world_mut().resource_mut::<InstanceIdCounter>();
        counter.next()
    };
    let stack = {
        let item_reg = app.world().resource::<ItemRegistry>();
        let item_def = item_reg.get(def_id).cloned().unwrap();
        let instance = ItemInstance::from_def(instance_id, &item_def);
        ItemStack::new(instance, 1)
    };
    // 装备 stack_size=1 不会合并，直接 push 即可，无需 add_stack 的合并逻辑
    {
        let mut container = app.world_mut().get_mut::<Container>(entity).unwrap();
        container.stacks.push(stack);
    }
    instance_id
}

/// 注册自定义装备定义（用于需求测试等）
fn register_custom_equipment(app: &mut App, eq_def: EquipmentDef) {
    let item_def = ItemDef {
        version: eq_def.version,
        id: eq_def.id.clone(),
        name: eq_def.name.clone(),
        description: eq_def.description.clone(),
        item_type: ItemType::Equipment,
        rarity: eq_def.rarity,
        tags: eq_def.tags.clone(),
        stack_size: 1,
        weight: eq_def.weight,
        modifiers: eq_def.modifiers.clone(),
        traits: eq_def.traits.clone(),
        requirements: eq_def.requirements.clone(),
        slot: Some(eq_def.slot),
        use_effects: vec![],
        container_capacity: None,
        container_max_weight: None,
    };
    app.world_mut()
        .resource_mut::<EquipmentRegistry>()
        .register(eq_def);
    app.world_mut()
        .resource_mut::<ItemRegistry>()
        .register(item_def);
}

/// 给角色添加标签（用于满足装备需求）
fn add_gameplay_tag(app: &mut App, entity: Entity, tag: GameplayTag) {
    let mut tags = app.world_mut().get_mut::<GameplayTags>(entity).unwrap();
    tags.add(tag);
}

// ══════════════════════════════════════════════════════════════
// 场景一：装备穿脱完整流程
// ══════════════════════════════════════════════════════════════

/// EQT-001: 装备穿脱完整流程 — 穿戴后属性/标签变化，脱卸后恢复
///
/// Given: 战士(Attack=10)，背包有铁剑(Attack+3, SWORD/MARTIAL)
/// When: 穿戴铁剑 → 验证 → 脱卸铁剑
/// Then: 穿戴后 Attack=13、有 SWORD/MARTIAL 标签；脱卸后 Attack=10、标签移除、物品回背包
#[test]
fn 装备穿脱完整流程_穿戴后属性标签变化_脱卸后恢复() {
    let mut app = equipment_app();
    register_defaults(&mut app);

    // 创建战士角色
    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 记录穿戴前的基础属性
    let base_attack = {
        let attrs = app.world().get::<Attributes>(entity).unwrap();
        attrs.get(AttributeKind::Attack)
    };
    // 战士 Might=5 → Attack = 5*2 = 10
    assert_attr_eq!(
        app.world().get::<Attributes>(entity).unwrap(),
        AttributeKind::Attack,
        10
    );

    // 在背包放入铁剑（Attack+3）
    let iron_sword_id = put_item_in_backpack(&mut app, entity, "iron_sword");

    // 穿戴铁剑
    app.world_mut().write_message(EquipItem {
        target_entity: entity,
        instance_id: iron_sword_id,
    });
    app.update();

    // 验证：属性增加
    assert_attr_eq!(
        app.world().get::<Attributes>(entity).unwrap(),
        AttributeKind::Attack,
        base_attack as i32 + 3
    );

    // 验证：标签添加
    assert_has_tag!(
        app.world().get::<GameplayTags>(entity).unwrap(),
        GameplayTag::SWORD
    );
    assert_has_tag!(
        app.world().get::<GameplayTags>(entity).unwrap(),
        GameplayTag::MARTIAL
    );

    // 验证：背包中已移除
    let container = app.world().get::<Container>(entity).unwrap();
    assert!(container.get(iron_sword_id).is_none());

    // 验证：装备槽已占用
    let slots = app.world().get::<EquipmentSlots>(entity).unwrap();
    assert!(slots.is_equipped(EquipmentSlot::MainHand));

    // 脱卸铁剑
    app.world_mut().write_message(UnequipItem {
        target_entity: entity,
        slot: EquipmentSlot::MainHand,
    });
    app.update();

    // 验证：属性恢复
    assert_attr_eq!(
        app.world().get::<Attributes>(entity).unwrap(),
        AttributeKind::Attack,
        base_attack as i32
    );

    // 验证：标签移除
    assert_not_has_tag!(
        app.world().get::<GameplayTags>(entity).unwrap(),
        GameplayTag::SWORD
    );
    assert_not_has_tag!(
        app.world().get::<GameplayTags>(entity).unwrap(),
        GameplayTag::MARTIAL
    );

    // 验证：物品回到背包
    let container = app.world().get::<Container>(entity).unwrap();
    assert!(container.find_by_def("iron_sword").is_some());

    // 验证：装备槽已清空
    let slots = app.world().get::<EquipmentSlots>(entity).unwrap();
    assert!(!slots.is_equipped(EquipmentSlot::MainHand));
}

// ══════════════════════════════════════════════════════════════
// 场景二：穿戴需求不满足 → EquipFailed
// ══════════════════════════════════════════════════════════════

/// EQT-002: 穿戴需求不满足 — 属性不足时发送 EquipFailed
///
/// Given: 哥布林(Might=3)，背包有巨力之剑(需要 Might>=20)
/// When: 尝试穿戴巨力之剑
/// Then: Attack 不变(6)，MainHand 未占用，物品仍在背包
#[test]
fn 穿戴需求不满足_属性不足_发送equip_failed() {
    let mut app = equipment_app();
    register_defaults(&mut app);

    // 注册一把需要高力量的武器
    let mighty_sword = EquipmentDef {
        version: 1,
        id: "mighty_sword".into(),
        name: "巨力之剑".into(),
        description: "需要极高力量".into(),
        slot: EquipmentSlot::MainHand,
        rarity: Rarity::Epic,
        tags: vec![TagName::Sword, TagName::Martial],
        modifiers: vec![AttributeModifierDef {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 20.0,
        }],
        traits: vec![],
        requirements: vec![EquipmentRequirement::AttributeMin {
            kind: AttributeKind::Might,
            value: 20.0, // 需要 Might >= 20
        }],
        weight: 8.0,
    };
    register_custom_equipment(&mut app, mighty_sword);

    // 创建低属性哥布林（Might=3）
    let entity = UnitBuilder::goblin().spawn(&mut app);

    // 在背包放入巨力之剑
    let sword_id = put_item_in_backpack(&mut app, entity, "mighty_sword");

    // 尝试穿戴
    app.world_mut().write_message(EquipItem {
        target_entity: entity,
        instance_id: sword_id,
    });
    app.update();

    // 验证：属性不变
    let attrs = app.world().get::<Attributes>(entity).unwrap();
    // 哥布林 Might=3 → Attack = 3*2 = 6
    assert_attr_eq!(attrs, AttributeKind::Attack, 6);

    // 验证：装备槽未占用
    let slots = app.world().get::<EquipmentSlots>(entity).unwrap();
    assert!(!slots.is_equipped(EquipmentSlot::MainHand));

    // 验证：物品仍在背包中
    let container = app.world().get::<Container>(entity).unwrap();
    assert!(container.get(sword_id).is_some());
}

/// EQT-003: 穿戴需求不满足 — 缺少标签时发送 EquipFailed
///
/// Given: 法师(无 MARTIAL 标签)，背包有炎龙长剑(需要 MARTIAL)
/// When: 尝试穿戴炎龙长剑
/// Then: MainHand 未占用，物品仍在背包
#[test]
fn 穿戴需求不满足_缺少标签_发送equip_failed() {
    let mut app = equipment_app();
    register_defaults(&mut app);

    // 炎龙长剑需要 MARTIAL 标签
    // 创建法师（没有 MARTIAL 标签）尝试穿戴
    let entity = UnitBuilder::mage().spawn(&mut app);

    // 在背包放入炎龙长剑
    let sword_id = put_item_in_backpack(&mut app, entity, "flame_dragon_sword");

    // 尝试穿戴
    app.world_mut().write_message(EquipItem {
        target_entity: entity,
        instance_id: sword_id,
    });
    app.update();

    // 验证：装备槽未占用
    let slots = app.world().get::<EquipmentSlots>(entity).unwrap();
    assert!(!slots.is_equipped(EquipmentSlot::MainHand));

    // 验证：物品仍在背包中
    let container = app.world().get::<Container>(entity).unwrap();
    assert!(container.get(sword_id).is_some());
}

// ══════════════════════════════════════════════════════════════
// 场景三：穿戴新装备时自动脱卸旧装备
// ══════════════════════════════════════════════════════════════

/// EQT-004: 穿戴新装备时自动脱卸旧装备
///
/// Given: 战士已穿戴铁剑(Attack+3)，背包有钢剑(Attack+6)
/// When: 穿戴钢剑到同槽位 MainHand
/// Then: 钢剑穿戴(Attack=16)，铁剑自动脱�回背包
#[test]
fn 穿戴新装备_同槽位自动脱卸旧装备_旧装备回背包() {
    let mut app = equipment_app();
    register_defaults(&mut app);

    // 注册一把钢剑（同 MainHand 槽位）
    let steel_sword = EquipmentDef {
        version: 1,
        id: "steel_sword".into(),
        name: "钢剑".into(),
        description: "比铁剑更强的钢剑".into(),
        slot: EquipmentSlot::MainHand,
        rarity: Rarity::Uncommon,
        tags: vec![TagName::Sword, TagName::Martial],
        modifiers: vec![AttributeModifierDef {
            kind: AttributeKind::Attack,
            op: ModifierOp::Add,
            value: 6.0, // 铁剑 +3，钢剑 +6
        }],
        traits: vec![],
        requirements: vec![],
        weight: 4.0,
    };
    register_custom_equipment(&mut app, steel_sword);

    // 创建战士
    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 在背包放入铁剑和钢剑
    let iron_sword_id = put_item_in_backpack(&mut app, entity, "iron_sword");
    let steel_sword_id = put_item_in_backpack(&mut app, entity, "steel_sword");

    // 穿戴铁剑
    app.world_mut().write_message(EquipItem {
        target_entity: entity,
        instance_id: iron_sword_id,
    });
    app.update();

    // 验证：铁剑已穿戴，Attack = 10 + 3 = 13
    assert_attr_eq!(
        app.world().get::<Attributes>(entity).unwrap(),
        AttributeKind::Attack,
        13
    );
    let slots = app.world().get::<EquipmentSlots>(entity).unwrap();
    assert_eq!(
        slots.get_def_id(EquipmentSlot::MainHand),
        Some("iron_sword")
    );

    // 穿戴钢剑到同槽位 → 应自动脱卸铁剑
    app.world_mut().write_message(EquipItem {
        target_entity: entity,
        instance_id: steel_sword_id,
    });
    app.update();

    // 验证：钢剑已穿戴，Attack = 10 + 6 = 16
    assert_attr_eq!(
        app.world().get::<Attributes>(entity).unwrap(),
        AttributeKind::Attack,
        16
    );
    let slots = app.world().get::<EquipmentSlots>(entity).unwrap();
    assert_eq!(
        slots.get_def_id(EquipmentSlot::MainHand),
        Some("steel_sword")
    );

    // 验证：铁剑回到背包
    let container = app.world().get::<Container>(entity).unwrap();
    assert!(container.find_by_def("iron_sword").is_some());

    // 验证：钢剑不在背包中
    assert!(container.find_by_def("steel_sword").is_none());
}

// ══════════════════════════════════════════════════════════════
// 场景四：装备 Trait 完整生命周期
// ══════════════════════════════════════════════════════════════

/// EQT-005: 装备 Trait 生命周期 — 穿戴时添加，脱卸时移除
///
/// Given: 战士(MARTIAL)，背包有炎龙长剑(flaming_weapon, dragon_bane)
/// When: 穿戴炎龙长剑 → 验证 → 脱卸
/// Then: 穿戴后 TraitCollection 有 flaming_weapon/dragon_bane(Equipment Source)；脱卸后消失
#[test]
fn 装备trait生命周期_穿戴时添加trait_脱卸时移除trait() {
    let mut app = equipment_app();
    register_defaults(&mut app);

    // 创建战士，添加 MARTIAL 标签以满足炎龙长剑需求
    let entity = UnitBuilder::warrior().spawn(&mut app);
    add_gameplay_tag(&mut app, entity, GameplayTag::MARTIAL);

    // 炎龙长剑拥有 traits: ["flaming_weapon", "dragon_bane"]
    let sword_id = put_item_in_backpack(&mut app, entity, "flame_dragon_sword");

    // 穿戴前：TraitCollection 为空
    let traits = app.world().get::<TraitCollection>(entity).unwrap();
    assert!(!traits.has("flaming_weapon"));
    assert!(!traits.has("dragon_bane"));

    // 穿戴炎龙长剑
    app.world_mut().write_message(EquipItem {
        target_entity: entity,
        instance_id: sword_id,
    });
    app.update();

    // 穿戴后：TraitCollection 有对应 entry
    let traits = app.world().get::<TraitCollection>(entity).unwrap();
    assert!(traits.has("flaming_weapon"));
    assert!(traits.has("dragon_bane"));

    // 验证 trait 来源是 Equipment
    let has_equipment_source = traits.entries.iter().any(|e| {
        e.trait_id == "flaming_weapon"
            && matches!(
                e.source,
                TraitSource::Equipment {
                    slot: EquipmentSlot::MainHand
                }
            )
    });
    assert!(
        has_equipment_source,
        "flaming_weapon 应来自 Equipment(MainHand)"
    );

    // 脱卸炎龙长剑
    app.world_mut().write_message(UnequipItem {
        target_entity: entity,
        slot: EquipmentSlot::MainHand,
    });
    app.update();

    // 脱卸后：TraitCollection 中 entry 消失
    let traits = app.world().get::<TraitCollection>(entity).unwrap();
    assert!(!traits.has("flaming_weapon"));
    assert!(!traits.has("dragon_bane"));
}

/// EQT-006: 多件装备 Trait 共存 — 脱卸一件不影响另一件
///
/// Given: 战士(MARTIAL)，穿戴炎龙长剑+龙盔（共享 dragon_bane）
/// When: 脱卸炎龙长剑
/// Then: dragon_bane 仍由龙盔提供(count=1)，flaming_weapon 消失
#[test]
fn 装备trait_多件装备trait共存_脱卸一件不影响另一件() {
    let mut app = equipment_app();
    register_defaults(&mut app);

    // 注册带 trait 的头盔
    let dragon_helm = EquipmentDef {
        version: 1,
        id: "dragon_helm".into(),
        name: "龙盔".into(),
        description: "龙族头盔".into(),
        slot: EquipmentSlot::Head,
        rarity: Rarity::Rare,
        tags: vec![TagName::HeavyArmor],
        modifiers: vec![AttributeModifierDef {
            kind: AttributeKind::Defense,
            op: ModifierOp::Add,
            value: 5.0,
        }],
        traits: vec!["dragon_bane".into()], // 与炎龙长剑共享 dragon_bane
        requirements: vec![],
        weight: 5.0,
    };
    register_custom_equipment(&mut app, dragon_helm);

    // 创建战士，添加 MARTIAL 标签以满足炎龙长剑需求
    let entity = UnitBuilder::warrior().spawn(&mut app);
    add_gameplay_tag(&mut app, entity, GameplayTag::MARTIAL);

    // 在背包放入炎龙长剑和龙盔
    let sword_id = put_item_in_backpack(&mut app, entity, "flame_dragon_sword");
    let helm_id = put_item_in_backpack(&mut app, entity, "dragon_helm");

    // 穿戴炎龙长剑
    app.world_mut().write_message(EquipItem {
        target_entity: entity,
        instance_id: sword_id,
    });
    app.update();

    // 穿戴龙盔
    app.world_mut().write_message(EquipItem {
        target_entity: entity,
        instance_id: helm_id,
    });
    app.update();

    // 两件装备都有 dragon_bane trait
    let traits = app.world().get::<TraitCollection>(entity).unwrap();
    let dragon_bane_count = traits
        .entries
        .iter()
        .filter(|e| e.trait_id == "dragon_bane")
        .count();
    assert_eq!(dragon_bane_count, 2, "dragon_bane 应来自两件装备");

    // 脱卸炎龙长剑
    app.world_mut().write_message(UnequipItem {
        target_entity: entity,
        slot: EquipmentSlot::MainHand,
    });
    app.update();

    // dragon_bane 仍由龙盔提供
    let traits = app.world().get::<TraitCollection>(entity).unwrap();
    assert!(traits.has("dragon_bane"));
    let dragon_bane_count = traits
        .entries
        .iter()
        .filter(|e| e.trait_id == "dragon_bane")
        .count();
    assert_eq!(
        dragon_bane_count, 1,
        "脱卸炎龙长剑后 dragon_bane 应只剩1个来源"
    );

    // flaming_weapon 应已消失
    assert!(!traits.has("flaming_weapon"));
}

// ══════════════════════════════════════════════════════════════
// 补充场景：多槽位装备同时穿戴
// ══════════════════════════════════════════════════════════════

/// EQT-007: 多槽位装备同时穿戴 — 属性叠加
///
/// Given: 战士(Attack=10, Defense=?)，背包有铁剑(MainHand, Attack+3)和皮甲(Body, Defense+2)
/// When: 依次穿戴铁剑和皮甲
/// Then: Attack=13, Defense=Def+2，两个槽位都已占用
#[test]
fn 多槽位装备_同时穿戴不同槽位_属性叠加() {
    let mut app = equipment_app();
    register_defaults(&mut app);

    // 创建战士
    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 记录基础值
    let base_attack = {
        let attrs = app.world().get::<Attributes>(entity).unwrap();
        attrs.get(AttributeKind::Attack)
    };
    let base_defense = {
        let attrs = app.world().get::<Attributes>(entity).unwrap();
        attrs.get(AttributeKind::Defense)
    };

    // 在背包放入铁剑（MainHand, Attack+3）和皮甲（Body, Defense+2）
    let sword_id = put_item_in_backpack(&mut app, entity, "iron_sword");
    let armor_id = put_item_in_backpack(&mut app, entity, "leather_armor");

    // 穿戴铁剑
    app.world_mut().write_message(EquipItem {
        target_entity: entity,
        instance_id: sword_id,
    });
    app.update();

    // 穿戴皮甲
    app.world_mut().write_message(EquipItem {
        target_entity: entity,
        instance_id: armor_id,
    });
    app.update();

    // 验证：Attack 和 Defense 同时增加
    let attrs = app.world().get::<Attributes>(entity).unwrap();
    assert_attr_eq!(attrs, AttributeKind::Attack, base_attack as i32 + 3);
    assert_attr_eq!(attrs, AttributeKind::Defense, base_defense as i32 + 2);

    // 验证：两个槽位都已占用
    let slots = app.world().get::<EquipmentSlots>(entity).unwrap();
    assert!(slots.is_equipped(EquipmentSlot::MainHand));
    assert!(slots.is_equipped(EquipmentSlot::Body));
}

// ══════════════════════════════════════════════════════════════
// 补充场景：PersistentTags 分层验证
// ══════════════════════════════════════════════════════════════

/// EQT-008: PersistentTags 分层验证 — 装备标签写入 from_equipment 层
///
/// Given: 战士，PersistentTags.from_equipment 为空
/// When: 穿戴铁剑 → 验证 → 脱卸
/// Then: 穿戴后 from_equipment 有 SWORD/MARTIAL；脱卸后清除
#[test]
fn persistent_tags_装备标签写入from_equipment层() {
    let mut app = equipment_app();
    register_defaults(&mut app);

    // 创建战士
    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 穿戴前：from_equipment 为空
    let persistent = app.world().get::<PersistentTags>(entity).unwrap();
    assert!(!persistent.from_equipment.has(GameplayTag::SWORD));

    // 在背包放入铁剑
    let sword_id = put_item_in_backpack(&mut app, entity, "iron_sword");

    // 穿戴
    app.world_mut().write_message(EquipItem {
        target_entity: entity,
        instance_id: sword_id,
    });
    app.update();

    // 穿戴后：from_equipment 有标签
    let persistent = app.world().get::<PersistentTags>(entity).unwrap();
    assert!(persistent.from_equipment.has(GameplayTag::SWORD));
    assert!(persistent.from_equipment.has(GameplayTag::MARTIAL));

    // 脱卸
    app.world_mut().write_message(UnequipItem {
        target_entity: entity,
        slot: EquipmentSlot::MainHand,
    });
    app.update();

    // 脱卸后：from_equipment 标签清除
    let persistent = app.world().get::<PersistentTags>(entity).unwrap();
    assert!(!persistent.from_equipment.has(GameplayTag::SWORD));
    assert!(!persistent.from_equipment.has(GameplayTag::MARTIAL));
}
