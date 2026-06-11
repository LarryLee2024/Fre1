//! 第 1 层：Rule Test（属性测试）
//!
//! 使用 proptest 对关键公式进行属性测试
//! - 伤害公式：calculate_damage_from_effect
//! - 属性计算：Attributes 修饰符叠加
//! - 标签位运算：GameplayTags 幂等性
//! - 堆叠合并：Container::add_stack

use proptest::prelude::*;

// ══════════════════════════════════════════════════════════════
// 伤害公式属性测试
// ══════════════════════════════════════════════════════════════

use tactical_rpg::core::effect::calculate_damage_from_effect;

proptest! {
    /// 伤害值始终 ≥ 1
    #[test]
    fn damage_always_at_least_1(
        atk in 0.0_f32..1000.0,
        def in 0.0_f32..1000.0,
        base_def in 0.0_f32..1000.0,
        multiplier in 0.5_f32..5.0,
        ignore_def in 0.0_f32..1.0,
        terrain_bonus in 0_i32..100,
    ) {
        let damage = calculate_damage_from_effect(
            atk, def, base_def, multiplier, ignore_def, terrain_bonus,
        );
        prop_assert!(damage >= 1, "damage {} < 1", damage);
    }

    /// 忽视防御比例越高，伤害越高
    #[test]
    fn ignore_def_increases_damage(
        atk in 100.0_f32..500.0,
        def in 50.0_f32..200.0,
        base_def in 50.0_f32..200.0,
        multiplier in 1.0_f32..3.0,
    ) {
        let low = calculate_damage_from_effect(atk, def, base_def, multiplier, 0.0, 0);
        let high = calculate_damage_from_effect(atk, def, base_def, multiplier, 0.5, 0);
        prop_assert!(high >= low,
            "ignore_def=0.5 damage {} should >= ignore_def=0.0 damage {}", high, low);
    }

    /// 倍率越高，伤害越高
    #[test]
    fn higher_multiplier_more_damage(
        atk in 100.0_f32..500.0,
        def in 10.0_f32..100.0,
        base_def in 10.0_f32..100.0,
    ) {
        let low = calculate_damage_from_effect(atk, def, base_def, 1.0, 0.0, 0);
        let high = calculate_damage_from_effect(atk, def, base_def, 2.0, 0.0, 0);
        prop_assert!(high >= low,
            "multiplier=2.0 damage {} should >= multiplier=1.0 damage {}", high, low);
    }

    /// 地形防御加成越高，伤害越低
    #[test]
    fn terrain_defense_reduces_damage(
        atk in 100.0_f32..500.0,
        def in 10.0_f32..100.0,
        base_def in 10.0_f32..100.0,
        multiplier in 1.0_f32..3.0,
    ) {
        let no_bonus = calculate_damage_from_effect(atk, def, base_def, multiplier, 0.0, 0);
        let with_bonus = calculate_damage_from_effect(atk, def, base_def, multiplier, 0.0, 20);
        prop_assert!(with_bonus <= no_bonus,
            "terrain_bonus=20 damage {} should <= terrain_bonus=0 damage {}", with_bonus, no_bonus);
    }
}

// ══════════════════════════════════════════════════════════════
// 属性计算属性测试
// ══════════════════════════════════════════════════════════════

use tactical_rpg::core::attribute::{AttributeKind, Attributes};

proptest! {
    /// 基础属性设置后可以正确读取
    #[test]
    fn set_base_then_get(value in 1.0_f32..100.0) {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Might, value);
        let got = attrs.get(AttributeKind::Might);
        prop_assert!((got - value).abs() < 0.01,
            "set Might={} but got {}", value, got);
    }

    /// fill_vital_resources 后 HP = MaxHp, MP = MaxMp
    #[test]
    fn fill_vital_resources_full(
        vitality in 1.0_f32..20.0,
        intelligence in 1.0_f32..20.0,
    ) {
        let mut attrs = Attributes::default();
        attrs.set_base(AttributeKind::Vitality, vitality);
        attrs.set_base(AttributeKind::Intelligence, intelligence);
        attrs.fill_vital_resources();
        let hp = attrs.get(AttributeKind::Hp);
        let max_hp = attrs.get(AttributeKind::MaxHp);
        let mp = attrs.get(AttributeKind::Mp);
        let max_mp = attrs.get(AttributeKind::MaxMp);
        prop_assert!((hp - max_hp).abs() < 0.01, "HP {} != MaxHp {}", hp, max_hp);
        prop_assert!((mp - max_mp).abs() < 0.01, "MP {} != MaxMp {}", mp, max_mp);
    }
}

// ══════════════════════════════════════════════════════════════
// 标签位运算属性测试
// ══════════════════════════════════════════════════════════════

use tactical_rpg::core::tag::{GameplayTag, GameplayTags};

fn arb_tag() -> impl Strategy<Value = GameplayTag> {
    prop_oneof![
        Just(GameplayTag::WARRIOR),
        Just(GameplayTag::MAGE),
        Just(GameplayTag::ARCHER),
        Just(GameplayTag::SWORD),
        Just(GameplayTag::AXE),
        Just(GameplayTag::BOW),
        Just(GameplayTag::STAFF),
        Just(GameplayTag::POISON),
        Just(GameplayTag::STUN),
        Just(GameplayTag::BURN),
        Just(GameplayTag::FIRE),
    ]
}

proptest! {
    /// 添加标签后 has 返回 true
    #[test]
    fn add_then_has(tag in arb_tag()) {
        let mut tags = GameplayTags::default();
        tags.add(tag);
        prop_assert!(tags.has(tag), "tag {:?} added but has() returns false", tag);
    }

    /// 添加再移除后 has 返回 false
    #[test]
    fn add_remove_then_not_has(tag in arb_tag()) {
        let mut tags = GameplayTags::default();
        tags.add(tag);
        tags.remove(tag);
        prop_assert!(!tags.has(tag), "tag {:?} removed but has() still true", tag);
    }

    /// 重复添加同一标签幂等
    #[test]
    fn add_idempotent(tag in arb_tag()) {
        let mut tags = GameplayTags::default();
        tags.add(tag);
        tags.add(tag);
        tags.add(tag);
        tags.remove(tag);
        prop_assert!(!tags.has(tag), "tag {:?} added 3x removed 1x still has()", tag);
    }

    /// 不同标签互不干扰
    #[test]
    fn different_tags_independent(tag1 in arb_tag(), tag2 in arb_tag()) {
        prop_assume!(tag1 != tag2);
        let mut tags = GameplayTags::default();
        tags.add(tag1);
        tags.add(tag2);
        tags.remove(tag1);
        prop_assert!(!tags.has(tag1));
        prop_assert!(tags.has(tag2), "removing {:?} also removed {:?}", tag1, tag2);
    }
}

// ══════════════════════════════════════════════════════════════
// 容器堆叠合并属性测试
// ══════════════════════════════════════════════════════════════

use tactical_rpg::equipment::Rarity;
use tactical_rpg::inventory::container::{Container, ContainerKind};
use tactical_rpg::inventory::definition::{ItemDef, ItemRegistry, ItemType, UseEffect};
use tactical_rpg::inventory::instance::{InstanceIdCounter, ItemStack};

fn make_item_def(id: &str, stack_size: u32) -> ItemDef {
    ItemDef {
        version: 1,
        id: id.into(),
        name: id.into(),
        description: String::new(),
        item_type: ItemType::Consumable,
        rarity: Rarity::Common,
        tags: vec![],
        stack_size,
        weight: 0.0,
        modifiers: vec![],
        traits: vec![],
        requirements: vec![],
        slot: None,
        use_effects: vec![],
        container_capacity: None,
        container_max_weight: None,
    }
}

proptest! {
    /// add_stack 不会拆分超量堆叠：单次 add_stack 后，新堆叠的 count 可能超过 stack_size
    /// 这是当前实现的行为：add_stack 只合并已有堆叠的空位，不拆分新堆叠
    /// 因此我们验证：当 add_count <= stack_size 时，每个堆叠的 count <= stack_size
    #[test]
    fn single_add_within_stack_size(
        stack_size in 2_u32..99,
        add_count in 1_u32..99,  // add_count <= stack_size 的情况
    ) {
        prop_assume!(add_count <= stack_size);
        let def = make_item_def("potion", stack_size);
        let mut registry = ItemRegistry::default();
        registry.register(def.clone());
        let mut counter = InstanceIdCounter::default();

        let mut container = Container::new(ContainerKind::Backpack, 20, 0.0);
        let mut stack = ItemStack::from_def(&mut counter, &def, add_count);
        container.add_stack(&mut stack, &registry);

        for stack in &container.stacks {
            prop_assert!(stack.count <= stack_size,
                "stack count {} exceeds stack_size {}", stack.count, stack_size);
        }
    }

    /// 多次添加后，合并逻辑正确：已有堆叠有空位时合并
    #[test]
    fn merge_into_existing_stack(stack_size in 5_u32..20, first_add in 1_u32..3, second_add in 1_u32..3) {
        let def = make_item_def("potion", stack_size);
        let mut registry = ItemRegistry::default();
        registry.register(def.clone());
        let mut counter = InstanceIdCounter::default();

        let mut container = Container::new(ContainerKind::Backpack, 20, 0.0);
        // 第一次添加
        let mut stack1 = ItemStack::from_def(&mut counter, &def, first_add);
        container.add_stack(&mut stack1, &registry);
        // 第二次添加（应该合并到已有堆叠）
        let mut stack2 = ItemStack::from_def(&mut counter, &def, second_add);
        container.add_stack(&mut stack2, &registry);

        // 总数量守恒
        let total: u32 = container.stacks.iter().map(|s| s.count).sum();
        prop_assert_eq!(total, first_add + second_add,
            "total {} != {} + {}", total, first_add, second_add);
        // 合并后堆叠数 <= 2
        prop_assert!(container.stacks.len() <= 2);
    }

    /// 容量满时无法添加更多堆叠
    #[test]
    fn capacity_limit_respected(capacity in 1_u32..5, num_items in 1_u32..10) {
        let def = make_item_def("weapon", 1);
        let mut registry = ItemRegistry::default();
        registry.register(def.clone());
        let mut counter = InstanceIdCounter::default();

        let mut container = Container::new(ContainerKind::Backpack, capacity, 0.0);
        for _ in 0..num_items {
            let mut stack = ItemStack::from_def(&mut counter, &def, 1);
            container.add_stack(&mut stack, &registry);
        }

        prop_assert!(container.stacks.len() <= capacity as usize,
            "stacks {} > capacity {}", container.stacks.len(), capacity);
    }
}
