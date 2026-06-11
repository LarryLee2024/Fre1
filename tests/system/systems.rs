//! 第 2 层 System Test：验证各系统在 App 中的行为
//!
//! 与 Feature Test 的区别：Feature Test 跨模块验证业务流程，
//! System Test 聚焦单个系统在 ECS 中的输入→输出行为。

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
use tactical_rpg::battle::{
    CharacterDied, DotApplied, HotApplied, StunApplied, execute_effects, trigger_on_attack_traits,
    trigger_on_kill_traits,
};
use tactical_rpg::buff::{ActiveBuffs, BuffInstance, BuffRegistry, resolve_status_effects};
use tactical_rpg::character::{
    Faction, GridPosition, PersistentTags, TraitCollection, TraitData, TraitEffect,
    TraitEffectHandlerRegistry, TraitRegistry, TraitSource, TraitTrigger, Unit, UnitName,
};
use tactical_rpg::core::attribute::{AttributeKind, Attributes, BuffInstanceId};
use tactical_rpg::core::effect::{EffectQueue, PendingEffectData};
use tactical_rpg::core::registry_loader::RegistryLoader;
use tactical_rpg::core::tag::{GameplayTag, GameplayTags};
use tactical_rpg::equipment::{
    EquipItem, EquipmentRegistry, EquipmentSlot, EquipmentSlots, Rarity, UnequipItem,
};
use tactical_rpg::inventory::container::{Container, ContainerKind};
use tactical_rpg::inventory::definition::{ItemDef, ItemRegistry, ItemType, UseEffect};
use tactical_rpg::inventory::instance::{InstanceIdCounter, ItemStack};
use tactical_rpg::inventory::transfer::TransferItem;
use tactical_rpg::inventory::use_item::UseItem;
use tactical_rpg::map::TerrainRegistry;
use tactical_rpg::skill::SkillCooldowns;
use tactical_rpg::skill::SkillSlots;
use tactical_rpg::turn::NeedsResolve;

use crate::common::app_builder::{combat_app, equipment_app, full_battle_app};
use crate::common::combat_helpers::{deal_damage, deal_heal, get_hp, tick};
use crate::common::fixtures::UnitBuilder;

// ── 测试辅助 ──

/// 创建带默认注册表的 BuffRegistry
fn test_buff_registry() -> BuffRegistry {
    let mut reg = BuffRegistry::default();
    RegistryLoader::register_defaults(&mut reg);
    reg
}

/// 在角色背包中放入指定物品，返回 instance_id
fn put_item_in_backpack(app: &mut App, entity: Entity, def_id: &str, count: u32) -> u64 {
    let item_def = app
        .world()
        .resource::<ItemRegistry>()
        .get(def_id)
        .cloned()
        .unwrap();
    let (instance_id, mut stack) = {
        let mut counter = app.world_mut().resource_mut::<InstanceIdCounter>();
        let stack = ItemStack::from_def(&mut counter, &item_def, count);
        (stack.instance.instance_id, stack)
    };
    app.world_mut()
        .resource_scope(|world, item_reg: Mut<ItemRegistry>| {
            let mut container = world.get_mut::<Container>(entity).unwrap();
            container.add_stack(&mut stack, &item_reg);
        });
    instance_id
}

/// 注册默认装备到 EquipmentRegistry + ItemRegistry
fn register_equipment_defaults(app: &mut App) {
    {
        let mut eq_reg = app.world_mut().resource_mut::<EquipmentRegistry>();
        RegistryLoader::register_defaults(&mut *eq_reg);
    }
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
    let mut item_reg = app.world_mut().resource_mut::<ItemRegistry>();
    item_reg.register(healing_potion);
}

/// 注册测试用物品（用于转移测试）
fn register_transfer_items(app: &mut App) {
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

/// 生成一个带 Container 的 Entity（独立背包）
fn spawn_container(app: &mut App, kind: ContainerKind, capacity: u32, max_weight: f32) -> Entity {
    app.world_mut()
        .spawn(Container::new(kind, capacity, max_weight))
        .id()
}

/// 在容器中放入物品，返回 instance_id
fn put_item_in_container(app: &mut App, container_entity: Entity, def_id: &str, count: u32) -> u64 {
    let item_def = app
        .world()
        .resource::<ItemRegistry>()
        .get(def_id)
        .cloned()
        .unwrap();
    let (instance_id, mut stack) = {
        let mut counter = app.world_mut().resource_mut::<InstanceIdCounter>();
        let stack = ItemStack::from_def(&mut counter, &item_def, count);
        (stack.instance.instance_id, stack)
    };
    app.world_mut()
        .resource_scope(|world, item_reg: Mut<ItemRegistry>| {
            let mut container = world.get_mut::<Container>(container_entity).unwrap();
            container.add_stack(&mut stack, &item_reg);
        });
    instance_id
}

// ══════════════════════════════════════════════════════════════
// 场景一：效果执行 System Test — execute_effects 系统
// ══════════════════════════════════════════════════════════════

/// SYS-001: 伤害效果减少目标 HP
///
/// Given: 目标哥布林(Vitality=5, HP=30)和攻击者战士
/// When: 入队 10 点伤害效果并 tick
/// Then: 目标 HP < 30
#[test]
fn 伤害效果减少目标hp() {
    let mut app = full_battle_app();
    app.insert_resource(test_buff_registry());
    app.insert_resource(TerrainRegistry::default());
    // TerrainRegistry 需要 register_defaults
    {
        let mut terrain_reg = app.world_mut().resource_mut::<TerrainRegistry>();
        terrain_reg.register_defaults();
    }
    app.add_systems(Update, execute_effects);

    let target = app
        .world_mut()
        .spawn((
            Unit {
                faction: Faction::Enemy,
                acted: false,
            },
            {
                let mut a = Attributes::default();
                a.set_base(AttributeKind::Vitality, 5.0);
                a.fill_vital_resources();
                a
            },
            SkillSlots::default(),
            ActiveBuffs::default(),
            GameplayTags::default(),
            GridPosition { coord: IVec2::ZERO },
            UnitName("哥布林".into()),
        ))
        .id();
    let source = app
        .world_mut()
        .spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            UnitName("战士".into()),
        ))
        .id();

    // 入队伤害效果
    deal_damage(&mut app, source, target, 10);
    tick(&mut app);

    let hp = get_hp(&app, target);
    assert!(hp < 30.0, "伤害效果应减少目标 HP，实际 HP = {}", hp);
}

/// SYS-002: 治疗效果恢复目标 HP
///
/// Given: 目标战士(Vitality=5, HP=10/30)在场
/// When: 入队 15 点治疗效果并 tick
/// Then: HP 恢复到 10 < HP <= 30
#[test]
fn 治疗效果恢复目标hp() {
    let mut app = full_battle_app();
    app.insert_resource(test_buff_registry());
    app.insert_resource(TerrainRegistry::default());
    {
        let mut terrain_reg = app.world_mut().resource_mut::<TerrainRegistry>();
        terrain_reg.register_defaults();
    }
    app.add_systems(Update, execute_effects);

    let target = app
        .world_mut()
        .spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            {
                let mut a = Attributes::default();
                a.set_base(AttributeKind::Vitality, 5.0);
                a.fill_vital_resources();
                a.set_vital(AttributeKind::Hp, 10.0); // 受伤状态
                a
            },
            SkillSlots::default(),
            ActiveBuffs::default(),
            GameplayTags::default(),
            GridPosition { coord: IVec2::ZERO },
            UnitName("战士".into()),
        ))
        .id();

    // 入队治疗效果
    deal_heal(&mut app, target, 15);
    tick(&mut app);

    let hp = get_hp(&app, target);
    assert!(hp > 10.0, "治疗效果应恢复目标 HP，实际 HP = {}", hp);
    assert!(hp <= 30.0, "治疗不应超过 MaxHp，实际 HP = {}", hp);
}

// ══════════════════════════════════════════════════════════════
// 场景二：Buff tick System Test — resolve_status_effects 系统
// ══════════════════════════════════════════════════════════════

/// SYS-003: Buff tick 减少剩余回合数
///
/// Given: 战士拥有一个 remaining_turns=3 的 Buff
/// When: 设置 NeedsResolve(true) 并 tick（resolve_status_effects）
/// Then: remaining_turns 从 3 减为 2
#[test]
fn buff_tick减少剩余回合数() {
    let mut app = full_battle_app();
    app.add_message::<CharacterDied>()
        .add_message::<DotApplied>()
        .add_message::<HotApplied>()
        .add_message::<StunApplied>();
    app.init_resource::<NeedsResolve>();
    app.add_systems(Update, resolve_status_effects);

    // 创建角色并施加一个持续 3 回合的 Buff
    let entity = app
        .world_mut()
        .spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            {
                let mut a = Attributes::default();
                a.set_base(AttributeKind::Vitality, 5.0);
                a.fill_vital_resources();
                a
            },
            ActiveBuffs::default(),
            GameplayTags::default(),
            GridPosition { coord: IVec2::ZERO },
            UnitName("战士".into()),
            SkillCooldowns::default(),
            PersistentTags::default(),
        ))
        .id();

    // 手动添加一个 Buff 实例
    {
        let mut buffs = app.world_mut().get_mut::<ActiveBuffs>(entity).unwrap();
        buffs.add(BuffInstance {
            instance_id: BuffInstanceId(1),
            buff_id: "test_buff".into(),
            name: "测试Buff".into(),
            remaining_turns: 3,
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 0,
        });
    }

    // 设置 NeedsResolve 触发结算
    *app.world_mut().resource_mut::<NeedsResolve>() = NeedsResolve(true);
    tick(&mut app);

    // 验证 remaining_turns 递减
    let buffs = app.world().get::<ActiveBuffs>(entity).unwrap();
    let buff = buffs
        .instances
        .iter()
        .find(|b| b.buff_id == "test_buff")
        .unwrap();
    assert_eq!(
        buff.remaining_turns, 2,
        "tick 后 remaining_turns 应从 3 减为 2"
    );
}

/// SYS-004: Buff 过期后自动移除
///
/// Given: 战士拥有一个 remaining_turns=1 的 Buff
/// When: 连续 tick 两次（第一次 1→0，第二次移除）
/// Then: 第一次 tick 后 remaining_turns=0，第二次 tick 后 Buff 被移除
#[test]
fn buff过期后自动移除() {
    let mut app = full_battle_app();
    app.add_message::<CharacterDied>()
        .add_message::<DotApplied>()
        .add_message::<HotApplied>()
        .add_message::<StunApplied>();
    app.init_resource::<NeedsResolve>();
    app.add_systems(Update, resolve_status_effects);

    let entity = app
        .world_mut()
        .spawn((
            Unit {
                faction: Faction::Player,
                acted: false,
            },
            {
                let mut a = Attributes::default();
                a.set_base(AttributeKind::Vitality, 5.0);
                a.fill_vital_resources();
                a
            },
            ActiveBuffs::default(),
            GameplayTags::default(),
            GridPosition { coord: IVec2::ZERO },
            UnitName("战士".into()),
            SkillCooldowns::default(),
            PersistentTags::default(),
        ))
        .id();

    // 添加一个 remaining_turns=1 的 Buff
    {
        let mut buffs = app.world_mut().get_mut::<ActiveBuffs>(entity).unwrap();
        buffs.add(BuffInstance {
            instance_id: BuffInstanceId(1),
            buff_id: "expiring_buff".into(),
            name: "即将过期".into(),
            remaining_turns: 1,
            source_entity: None,
            tags: vec![GameplayTag::BUFF],
            is_buff: true,
            dot_damage: 0,
            hot_heal: 0,
        });
    }

    // 第一次 tick：remaining_turns 1→0，修饰符清理，但实例仍在
    *app.world_mut().resource_mut::<NeedsResolve>() = NeedsResolve(true);
    tick(&mut app);

    let buffs = app.world().get::<ActiveBuffs>(entity).unwrap();
    let buff = buffs
        .instances
        .iter()
        .find(|b| b.buff_id == "expiring_buff")
        .unwrap();
    assert_eq!(
        buff.remaining_turns, 0,
        "第一次 tick 后 remaining_turns 应为 0"
    );

    // 第二次 tick：remaining_turns=0 的实例被移除
    *app.world_mut().resource_mut::<NeedsResolve>() = NeedsResolve(true);
    tick(&mut app);

    let buffs = app.world().get::<ActiveBuffs>(entity).unwrap();
    assert!(
        !buffs.instances.iter().any(|b| b.buff_id == "expiring_buff"),
        "第二次 tick 后过期 Buff 应被移除"
    );
}

// ══════════════════════════════════════════════════════════════
// 场景三：装备穿脱 System Test — equip_item_system / unequip_item_system
// ══════════════════════════════════════════════════════════════

/// SYS-005: 穿戴装备后槽位被占用
///
/// Given: 战士在铁剑在背包
/// When: 发送 EquipItem 消息并 tick
/// Then: MainHand 槽位已占用，铁剑从背包移除
#[test]
fn 穿戴装备后槽位被占用() {
    let mut app = equipment_app();
    register_equipment_defaults(&mut app);

    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 在背包放入铁剑
    let iron_sword_id = put_item_in_backpack(&mut app, entity, "iron_sword", 1);

    // 穿戴铁剑
    app.world_mut().write_message(EquipItem {
        target_entity: entity,
        instance_id: iron_sword_id,
    });
    tick(&mut app);

    // 验证：槽位已占用
    let slots = app.world().get::<EquipmentSlots>(entity).unwrap();
    assert!(
        slots.is_equipped(EquipmentSlot::MainHand),
        "穿戴后 MainHand 槽位应被占用"
    );

    // 验证：背包中已移除
    let container = app.world().get::<Container>(entity).unwrap();
    assert!(
        container.get(iron_sword_id).is_none(),
        "穿戴后物品应从背包移除"
    );
}

/// SYS-006: 脱卸装备后槽位清空
///
/// Given: 战士已穿戴铁剑
/// When: 发送 UnequipItem 消息并 tick
/// Then: MainHand 槽位清空，铁剑回到背包
#[test]
fn 脱卸装备后槽位清空() {
    let mut app = equipment_app();
    register_equipment_defaults(&mut app);

    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 在背包放入铁剑并穿戴
    let iron_sword_id = put_item_in_backpack(&mut app, entity, "iron_sword", 1);
    app.world_mut().write_message(EquipItem {
        target_entity: entity,
        instance_id: iron_sword_id,
    });
    tick(&mut app);

    // 确认已穿戴
    let slots = app.world().get::<EquipmentSlots>(entity).unwrap();
    assert!(slots.is_equipped(EquipmentSlot::MainHand));

    // 脱卸铁剑
    app.world_mut().write_message(UnequipItem {
        target_entity: entity,
        slot: EquipmentSlot::MainHand,
    });
    tick(&mut app);

    // 验证：槽位已清空
    let slots = app.world().get::<EquipmentSlots>(entity).unwrap();
    assert!(
        !slots.is_equipped(EquipmentSlot::MainHand),
        "脱卸后 MainHand 槽位应清空"
    );

    // 验证：物品回到背包
    let container = app.world().get::<Container>(entity).unwrap();
    assert!(
        container.find_by_def("iron_sword").is_some(),
        "脱卸后物品应回到背包"
    );
}

// ══════════════════════════════════════════════════════════════
// 场景四：消耗品使用 System Test — use_item_system
// ══════════════════════════════════════════════════════════════

/// SYS-007: 使用消耗品后数量减少
///
/// Given: 战士背包有 3 瓶治疗药水
/// When: 发送 UseItem 消息并 tick
/// Then: 药水数量从 3 减为 2
#[test]
fn 使用消耗品后数量减少() {
    let mut app = combat_app();
    register_consumables(&mut app);

    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 在背包放入 3 瓶治疗药水
    let potion_id = put_item_in_backpack(&mut app, entity, "potion_healing", 3);

    // 验证初始数量
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
    tick(&mut app);

    // 验证数量减少
    let container = app.world().get::<Container>(entity).unwrap();
    let stack = container.find_by_def("potion_healing").unwrap();
    assert_eq!(stack.count, 2, "使用一瓶后应剩余 2 瓶");
}

/// SYS-008: 非消耗品不可使用
///
/// Given: 战士背包有铁剑（Equipment 类型）
/// When: 发送 UseItem 消息并 tick
/// Then: 铁剑仍在背包中（未被消耗）
#[test]
fn 非消耗品不可使用() {
    let mut app = equipment_app();
    register_equipment_defaults(&mut app);

    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 在背包放入铁剑（Equipment 类型，非 Consumable）
    let sword_id = put_item_in_backpack(&mut app, entity, "iron_sword", 1);

    // 尝试使用铁剑
    app.world_mut().write_message(UseItem {
        user_entity: entity,
        container_entity: entity,
        instance_id: sword_id,
    });
    tick(&mut app);

    // 验证：物品仍在背包中（未被消耗）
    let container = app.world().get::<Container>(entity).unwrap();
    assert!(
        container.get(sword_id).is_some(),
        "非消耗品不应被消耗，物品应仍在背包中"
    );
}

// ══════════════════════════════════════════════════════════════
// 场景五：容器转移 System Test — transfer_item_system
// ══════════════════════════════════════════════════════════════

/// SYS-009: 转移成功后源减少目标增加
///
/// Given: 容器 A 有 10 瓶药水，容器 B 为空
/// When: 从 A 转移 5 瓶到 B
/// Then: A 剩余 5 瓶，B 有 5 瓶
#[test]
fn 转移成功后源减少目标增加() {
    let mut app = combat_app();
    register_transfer_items(&mut app);

    // 创建两个独立背包
    let bag_a = spawn_container(&mut app, ContainerKind::Backpack, 20, 100.0);
    let bag_b = spawn_container(&mut app, ContainerKind::Backpack, 20, 100.0);

    // 在 A 中放入 10 瓶药水
    let potion_id = put_item_in_container(&mut app, bag_a, "potion_healing", 10);

    // 从 A 转移 5 瓶到 B
    app.world_mut().write_message(TransferItem {
        from_entity: bag_a,
        to_entity: bag_b,
        instance_id: potion_id,
        count: 5,
    });
    tick(&mut app);

    // 验证：A 剩余 5 瓶
    let container_a = app.world().get::<Container>(bag_a).unwrap();
    assert_eq!(container_a.stacks[0].count, 5, "源容器应剩余 5 瓶");

    // 验证：B 有 5 瓶
    let container_b = app.world().get::<Container>(bag_b).unwrap();
    assert_eq!(container_b.stacks[0].count, 5, "目标容器应有 5 瓶");
}

/// SYS-010: 目标满时转移失败
///
/// Given: 容器 A 有 10 瓶药水，容器 B 已满（capacity=1）
/// When: 从 A 转移 5 瓶到 B
/// Then: A 仍有 10 瓶，B 仍只有 1 瓶（转移失败）
#[test]
fn 目标满时转移失败() {
    let mut app = combat_app();
    register_transfer_items(&mut app);

    // A：正常背包
    let bag_a = spawn_container(&mut app, ContainerKind::Backpack, 20, 100.0);
    let potion_id = put_item_in_container(&mut app, bag_a, "potion_healing", 10);

    // B：容量为 1 的背包，放入一个物品占满
    let bag_b = spawn_container(&mut app, ContainerKind::Chest, 1, 100.0);
    put_item_in_container(&mut app, bag_b, "potion_healing", 1);

    // 确认 B 已满
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
    tick(&mut app);

    // 验证：A 中物品未减少
    let container_a = app.world().get::<Container>(bag_a).unwrap();
    assert_eq!(
        container_a.stacks[0].count, 10,
        "转移失败后源容器物品应未减少"
    );

    // 验证：B 中仍只有 1 瓶
    let container_b = app.world().get::<Container>(bag_b).unwrap();
    assert_eq!(
        container_b.stacks[0].count, 1,
        "转移失败后目标容器物品应不变"
    );
}

// ══════════════════════════════════════════════════════════════
// 场景六：Trait 触发 System Test — trigger_on_attack_traits 等
// ══════════════════════════════════════════════════════════════

/// SYS-011: OnAttack Trait 触发后效果入队
///
/// Given: TraitCollection 包含 on_attack_buff Trait（ApplyBuff attack_up duration=2）
/// When: 调用 trigger_on_attack_traits
/// Then: EffectQueue 中产生 1 个 ApplyBuff 效果，source=attacker, target=target
#[test]
fn on_attack_trait触发后效果入队() {
    let mut trait_registry = TraitRegistry::default();
    trait_registry.traits.insert(
        "on_attack_buff".into(),
        TraitData {
            id: "on_attack_buff".into(),
            name: "攻击时加Buff".into(),
            description: String::new(),
            trigger: TraitTrigger::OnAttack,
            effects: vec![TraitEffect::ApplyBuff {
                buff_id: "attack_up".into(),
                duration: 2,
            }],
        },
    );
    let handlers = TraitEffectHandlerRegistry::with_defaults();

    let mut collection = TraitCollection::default();
    collection.add_entry("on_attack_buff".into(), TraitSource::Intrinsic);

    let mut queue = EffectQueue::default();
    let attacker = Entity::from_bits(1);
    let target = Entity::from_bits(2);

    trigger_on_attack_traits(
        attacker,
        target,
        &collection,
        &trait_registry,
        &handlers,
        &mut queue,
    );

    assert_eq!(queue.pending.len(), 1, "OnAttack Trait 应产生 1 个效果");
    assert_eq!(queue.pending[0].source, attacker);
    assert_eq!(queue.pending[0].target, target);
    if let PendingEffectData::ApplyBuff { buff_id, duration } = &queue.pending[0].data {
        assert_eq!(buff_id, "attack_up");
        assert_eq!(*duration, 2);
    } else {
        panic!("期望 ApplyBuff 效果");
    }
}

/// SYS-012: OnKill Trait 触发后效果入队
///
/// Given: TraitCollection 包含 on_kill_heal Trait（ApplyBuff regen duration=3）
/// When: 调用 trigger_on_kill_traits
/// Then: EffectQueue 中产生 1 个 ApplyBuff 效果，source=killer, target=victim
#[test]
fn on_kill_trait触发后效果入队() {
    let mut trait_registry = TraitRegistry::default();
    trait_registry.traits.insert(
        "on_kill_heal".into(),
        TraitData {
            id: "on_kill_heal".into(),
            name: "击杀回血".into(),
            description: String::new(),
            trigger: TraitTrigger::OnKill,
            effects: vec![TraitEffect::ApplyBuff {
                buff_id: "regen".into(),
                duration: 3,
            }],
        },
    );
    let handlers = TraitEffectHandlerRegistry::with_defaults();

    let mut collection = TraitCollection::default();
    collection.add_entry("on_kill_heal".into(), TraitSource::Intrinsic);

    let mut queue = EffectQueue::default();
    let killer = Entity::from_bits(1);
    let victim = Entity::from_bits(2);

    trigger_on_kill_traits(
        killer,
        victim,
        &collection,
        &trait_registry,
        &handlers,
        &mut queue,
    );

    assert_eq!(queue.pending.len(), 1, "OnKill Trait 应产生 1 个效果");
    // OnKill 效果的目标是 victim（trigger_traits 中 target_entity）
    assert_eq!(queue.pending[0].source, killer);
    assert_eq!(queue.pending[0].target, victim);
}
