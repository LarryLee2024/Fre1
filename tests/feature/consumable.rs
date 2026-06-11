//! 消耗品系统 Feature Test
//!
//! 跨 inventory/use_item + inventory/container + inventory/definition + core/attribute + buff
//! 测试消耗品使用完整流程：恢复属性、赋予 Buff、数量消耗。

use bevy::prelude::*;
use tactical_rpg::buff::ActiveBuffs;
use tactical_rpg::core::attribute::{
    AttributeKind, AttributeModifierInstance, Attributes, ModifierOp,
};
use tactical_rpg::equipment::Rarity;
use tactical_rpg::inventory::container::Container;
use tactical_rpg::inventory::definition::{ItemDef, ItemRegistry, ItemType, UseEffect};
use tactical_rpg::inventory::instance::{InstanceIdCounter, ItemStack};
use tactical_rpg::inventory::use_item::UseItem;

use crate::common::app_builder::combat_app;
use crate::common::fixtures::UnitBuilder;

// ── 测试辅助 ──

/// 注册消耗品到 ItemRegistry
fn register_consumables(app: &mut App) {
    let healing_potion = ItemDef {
        version: 1,
        id: "potion_healing".into(),
        name: "治疗药水".into(),
        description: "恢复 50 HP".into(),
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
    };

    let strength_potion = ItemDef {
        version: 1,
        id: "potion_strength".into(),
        name: "力量药水".into(),
        description: "赋予力量增强 Buff".into(),
        item_type: ItemType::Consumable,
        rarity: Rarity::Uncommon,
        tags: vec![],
        stack_size: 99,
        weight: 0.5,
        modifiers: vec![],
        traits: vec![],
        requirements: vec![],
        slot: None,
        use_effects: vec![UseEffect::ApplyBuff {
            buff_id: "strength_up".into(),
            duration: 3,
        }],
        container_capacity: None,
        container_max_weight: None,
    };

    let mut item_reg = app.world_mut().resource_mut::<ItemRegistry>();
    item_reg.register(healing_potion);
    item_reg.register(strength_potion);
}

/// 在角色背包中放入指定消耗品，返回 instance_id
fn put_consumable_in_backpack(app: &mut App, entity: Entity, def_id: &str, count: u32) -> u64 {
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
            let mut container = world.get_mut::<Container>(entity).unwrap();
            container.add_stack(&mut stack, &item_reg);
        });
    instance_id
}

/// 构建消耗品测试 App（InventoryPlugin 已注册 UseItem/ItemUsed 消息和 use_item_system）
fn consumable_app() -> App {
    combat_app()
}

// ══════════════════════════════════════════════════════════════
// 场景一：治疗药水恢复 HP
// ══════════════════════════════════════════════════════════════

#[test]
fn 治疗药水恢复hp_受伤角色使用后hp修饰符增加() {
    let mut app = consumable_app();
    register_consumables(&mut app);

    // 创建战士角色
    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 手动降低 HP
    {
        let mut attrs = app.world_mut().get_mut::<Attributes>(entity).unwrap();
        let max_hp = attrs.get(AttributeKind::MaxHp);
        attrs.set_vital(AttributeKind::Hp, max_hp - 80.0);
    }

    // 记录使用前 HP 和修饰符数量
    let hp_before = {
        let attrs = app.world().get::<Attributes>(entity).unwrap();
        attrs.get(AttributeKind::Hp)
    };
    let hp_mod_count_before = {
        let attrs = app.world().get::<Attributes>(entity).unwrap();
        attrs
            .modifiers
            .iter()
            .filter(|m| m.kind == AttributeKind::Hp)
            .count()
    };

    // 在背包放入治疗药水
    let potion_id = put_consumable_in_backpack(&mut app, entity, "potion_healing", 1);

    // 使用治疗药水
    app.world_mut().write_message(UseItem {
        user_entity: entity,
        container_entity: entity,
        instance_id: potion_id,
    });
    app.update();

    // 验证：HP 修饰符被添加（RestoreVital 通过 add_modifier 实现）
    let attrs = app.world().get::<Attributes>(entity).unwrap();
    let hp_mods: Vec<&AttributeModifierInstance> = attrs
        .modifiers
        .iter()
        .filter(|m| m.kind == AttributeKind::Hp)
        .collect();
    assert_eq!(
        hp_mods.len(),
        hp_mod_count_before + 1,
        "使用治疗药水后应添加 1 个 HP 修饰符"
    );
    assert_eq!(hp_mods[0].op, ModifierOp::Add);
    assert!(
        (hp_mods[0].value - 50.0).abs() < 0.01,
        "HP 修饰符值应为 50.0，实际 {}",
        hp_mods[0].value
    );

    // 验证：current_hp 未被直接修改（RestoreVital 走修饰符管线）
    let hp_after = attrs.get(AttributeKind::Hp);
    assert!(
        (hp_after - hp_before).abs() < 0.01,
        "current_hp 不应被直接修改，实际从 {} 变为 {}",
        hp_before,
        hp_after
    );
}

// ══════════════════════════════════════════════════════════════
// 场景二：药水赋予 Buff
// ══════════════════════════════════════════════════════════════

#[test]
fn 药水赋予buff_使用力量药水后获得buff() {
    let mut app = consumable_app();
    register_consumables(&mut app);

    // 创建战士角色
    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 使用前：没有 Buff
    let buffs_before = app.world().get::<ActiveBuffs>(entity).unwrap();
    assert!(
        !buffs_before.iter().any(|b| b.buff_id == "strength_up"),
        "使用前不应有 strength_up Buff"
    );

    // 在背包放入力量药水
    let potion_id = put_consumable_in_backpack(&mut app, entity, "potion_strength", 1);

    // 使用力量药水
    app.world_mut().write_message(UseItem {
        user_entity: entity,
        container_entity: entity,
        instance_id: potion_id,
    });
    app.update();

    // 验证：获得了 strength_up Buff
    let buffs = app.world().get::<ActiveBuffs>(entity).unwrap();
    assert!(
        buffs.iter().any(|b| b.buff_id == "strength_up"),
        "使用力量药水后应获得 strength_up Buff"
    );

    // 验证：Buff 持续时间为 3 回合
    let strength_buff = buffs.iter().find(|b| b.buff_id == "strength_up").unwrap();
    assert_eq!(
        strength_buff.remaining_turns, 3,
        "strength_up Buff 持续时间应为 3 回合"
    );
}

// ══════════════════════════════════════════════════════════════
// 场景三：消耗品使用后数量减少
// ══════════════════════════════════════════════════════════════

#[test]
fn 消耗品使用后数量减少_药水x3使用一个后变x2() {
    let mut app = consumable_app();
    register_consumables(&mut app);

    // 创建战士角色
    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 在背包放入 3 瓶治疗药水
    let potion_id = put_consumable_in_backpack(&mut app, entity, "potion_healing", 3);

    // 验证：使用前有 3 瓶
    {
        let container = app.world().get::<Container>(entity).unwrap();
        let stack = container.get(potion_id).unwrap();
        assert_eq!(stack.count, 3, "使用前应有 3 瓶药水");
    }

    // 使用一瓶
    app.world_mut().write_message(UseItem {
        user_entity: entity,
        container_entity: entity,
        instance_id: potion_id,
    });
    app.update();

    // 验证：使用后剩余 2 瓶
    {
        let container = app.world().get::<Container>(entity).unwrap();
        let stack = container.find_by_def("potion_healing").unwrap();
        assert_eq!(stack.count, 2, "使用一瓶后应剩余 2 瓶药水");
    }
}
