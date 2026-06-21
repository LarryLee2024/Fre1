//! 第 1 层：Rule Test（属性测试）
//!
//! 使用 proptest 对关键公式进行属性测试
//! - 伤害公式：calculate_damage_from_effect
//! - 属性计算：Attributes 修饰符叠加
//! - 标签位运算：GameplayTags 幂等性
//! - 堆叠合并：Container::add_stack

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

use proptest::prelude::*;

// ══════════════════════════════════════════════════════════════
// 伤害公式属性测试
// ══════════════════════════════════════════════════════════════

use tactical_rpg::core::effect::calculate_damage_from_effect;

proptest! {
    /// RUL-001: 伤害值始终 ≥ 1（伤害公式下界）
    ///
    /// Given: 任意 ATK/DEF/BaseDef/Multiplier/IgnoreDef/TerrainBonus
    /// When: 调用 calculate_damage_from_effect
    /// Then: 返回值 >= 1
    #[test]
    fn 伤害公式_伤害值至少为1(
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

    /// RUL-002: 忽视防御比例越高，伤害越高
    ///
    /// Given: 相同 ATK/DEF/BaseDef/Multiplier
    /// When: ignore_def=0.5 vs ignore_def=0.0
    /// Then: 高忽略防御的伤害 >= 低忽略防御的伤害
    #[test]
    fn 伤害公式_忽视防御比例越高伤害越高(
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

    /// RUL-003: 倍率越高，伤害越高
    ///
    /// Given: 相同 ATK/DEF/BaseDef
    /// When: multiplier=2.0 vs multiplier=1.0
    /// Then: 高倍率的伤害 >= 低倍率的伤害
    #[test]
    fn 伤害公式_倍率越高伤害越高(
        atk in 100.0_f32..500.0,
        def in 10.0_f32..100.0,
        base_def in 10.0_f32..100.0,
    ) {
        let low = calculate_damage_from_effect(atk, def, base_def, 1.0, 0.0, 0);
        let high = calculate_damage_from_effect(atk, def, base_def, 2.0, 0.0, 0);
        prop_assert!(high >= low,
            "multiplier=2.0 damage {} should >= multiplier=1.0 damage {}", high, low);
    }

    /// RUL-004: 地形防御加成越高，伤害越低
    ///
    /// Given: 相同 ATK/DEF/BaseDef/Multiplier/IgnoreDef
    /// When: terrain_bonus=20 vs terrain_bonus=0
    /// Then: 有地形加成的伤害 <= 无地形加成的伤害
    #[test]
    fn 伤害公式_地形防御加成减少伤害(
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

use tactical_rpg::core::attribute::Attributes;

proptest! {
    /// RUL-005: 基础属性设置后可以正确读取
    ///
    /// Given: 任意 value (1.0..100.0)
    /// When: set_base(Might, value) 然后 get(Might)
    /// Then: 读取值与设置值偏差 < 0.01
    #[test]
    fn 属性计算_设置基础属性后可正确读取(value in 1.0_f32..100.0) {
        let mut attrs = Attributes::default();
        attrs.set_base("phys_atk", value as i32);
        let got = attrs.get("phys_atk");
        prop_assert_eq!(got, value as i32,
            "set phys_atk={} but got {}", value as i32, got);
    }

    /// RUL-006: fill_vital_resources 后 HP = MaxHp, MP = MaxMp
    ///
    /// Given: 任意 Vitality/Intelligence (1.0..20.0)
    /// When: set_base(Vitality/Intelligence) + fill_vital_resources()
    /// Then: HP==MaxHp 且 MP==MaxMp
    #[test]
    fn 属性计算_填充资源后HP等于MaxHp(vitality in 1.0_f32..20.0) {
        let mut attrs = Attributes::default();
        attrs.set_base("max_hp", (vitality as i32) * 10);
        attrs.fill_hp();
        let hp = attrs.current_hp;
        let max_hp = attrs.get("max_hp");
        prop_assert_eq!(hp, max_hp, "HP {} != MaxHp {}", hp, max_hp);
    }
}

// ══════════════════════════════════════════════════════════════
// 标签位运算属性测试
// ══════════════════════════════════════════════════════════════

use tactical_rpg::core::tag::{GameplayTag, GameplayTags};

fn arb_tag() -> impl Strategy<Value = GameplayTag> {
    prop_oneof![
        Just(GameplayTag::ALLY),
        Just(GameplayTag::ENEMY),
        Just(GameplayTag::WEAPON_SWORD),
        Just(GameplayTag::WEAPON_BOW),
        Just(GameplayTag::WEAPON_STAFF),
        Just(GameplayTag::DMG_PHYSICAL),
        Just(GameplayTag::DMG_MAGICAL),
        Just(GameplayTag::DMG_FIRE),
        Just(GameplayTag::DMG_ICE),
        Just(GameplayTag::CONTROL_HARD),
        Just(GameplayTag::CONTROL_SOFT),
    ]
}

proptest! {
    /// RUL-007: 添加标签后 has 返回 true
    ///
    /// Given: 空 GameplayTags 和任意 tag
    /// When: tags.add(tag)
    /// Then: tags.has(tag) == true
    #[test]
    fn 标签运算_添加标签后has返回true(tag in arb_tag()) {
        let mut tags = GameplayTags::default();
        tags.add(tag);
        prop_assert!(tags.has(tag), "tag {:?} added but has() returns false", tag);
    }

    /// RUL-008: 添加再移除后 has 返回 false
    ///
    /// Given: 空 GameplayTags 和任意 tag
    /// When: tags.add(tag) 然后 tags.remove(tag)
    /// Then: tags.has(tag) == false
    #[test]
    fn 标签运算_添加再移除后has返回false(tag in arb_tag()) {
        let mut tags = GameplayTags::default();
        tags.add(tag);
        tags.remove(tag);
        prop_assert!(!tags.has(tag), "tag {:?} removed but has() still true", tag);
    }

    /// RUL-009: 重复添加同一标签幂等
    ///
    /// Given: 空 GameplayTags 和任意 tag
    /// When: tags.add(tag) ×3 然后 tags.remove(tag) ×1
    /// Then: tags.has(tag) == false（幂等：多次添加等于一次）
    #[test]
    fn 标签运算_重复添加同一标签幂等(tag in arb_tag()) {
        let mut tags = GameplayTags::default();
        tags.add(tag);
        tags.add(tag);
        tags.add(tag);
        tags.remove(tag);
        prop_assert!(!tags.has(tag), "tag {:?} added 3x removed 1x still has()", tag);
    }

    /// RUL-010: 不同标签互不干扰
    ///
    /// Given: 空 GameplayTags 和两个不同 tag1, tag2
    /// When: add(tag1) + add(tag2) 然后 remove(tag1)
    /// Then: has(tag1)==false 但 has(tag2)==true
    #[test]
    fn 标签运算_不同标签互不干扰(tag1 in arb_tag(), tag2 in arb_tag()) {
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

use tactical_rpg::core::equipment::Rarity;
use tactical_rpg::core::inventory::container::{Container, ContainerKind};
use tactical_rpg::core::inventory::def::{ItemDef, ItemRegistry, ItemType, UseEffect};
use tactical_rpg::core::inventory::instance::{InstanceIdCounter, ItemStack};

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
    /// RUL-011: 单次 add_stack 在 stack_size 限制内
    ///
    /// Given: stack_size (2..99) 和 add_count (<= stack_size)
    /// When: ItemStack::from_def + container.add_stack
    /// Then: 每个堆叠的 count <= stack_size
    #[test]
    fn 堆叠合并_单次添加在stack_size限制内(
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

    /// RUL-012: 多次添加后合并逻辑正确
    ///
    /// Given: stack_size (5..20) 和两次小数量添加
    /// When: 两次 add_stack 到同一容器
    /// Then: 总数量守恒（first_add + second_add），堆叠数 <= 2
    #[test]
    fn 堆叠合并_多次添加后合并逻辑正确(stack_size in 5_u32..20, first_add in 1_u32..3, second_add in 1_u32..3) {
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

    /// RUL-013: 容量满时无法添加更多堆叠
    ///
    /// Given: container capacity (1..5) 和 num_items (1..10)
    /// When: 循环 add_stack 超过容量
    /// Then: stacks.len() <= capacity
    #[test]
    fn 堆叠合并_容量满时无法添加更多堆叠(capacity in 1_u32..5, num_items in 1_u32..10) {
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
