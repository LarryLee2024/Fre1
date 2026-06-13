# 测试体系方案 v2：5 层测试金字塔

基于 `docs/10.md` 的测试金字塔理念，结合项目现状（423 单元测试 + 7 集成测试文件）、Bevy 0.18 特性（Message/Hook/Observer/Required Components）、大型 SRPG 项目最佳实践，制定完整测试体系方案。

---

## 一、现状分析

### 1.1 测试资产盘点

| 维度 | 现状 | 评分 |
|------|------|------|
| 单元测试 | 423 个，覆盖 53 个源文件 | ★★★★★★★★★★ |
| 集成测试 | 7 个文件（buff_damage/buff_lifecycle/combat_pipeline/skill_system/terrain_combat/turn_flow/edge_cases） | ★★★★★☆☆☆☆☆ |
| App-based 系统测试 | ~25 个，覆盖部分 ECS 行为 | ★★★☆☆☆☆☆☆☆ |
| Feature Test | 无 | ☆☆☆☆☆☆☆☆☆☆ |
| Scenario/BDD Test | 无 | ☆☆☆☆☆☆☆☆☆☆ |
| Golden Battle Test | 无 | ☆☆☆☆☆☆☆☆☆☆ |
| 测试辅助工具 | `tests/common/` 已建立（UnitBuilder/combat_app/assertions） | ★★★★☆☆☆☆☆☆ |
| dev-dependencies | proptest 1.5 + insta 1.41（已声明，未使用） | ★★★☆☆☆☆☆☆☆ |

### 1.2 核心问题

1. **跨模块联动风险高**：Buff + 技能 + 装备 + 地形 + 回合 + AI 一起工作时出问题，这是当前最大风险
2. **缺少 Feature 级别测试**：单个 Feature 的完整生命周期（装备穿脱、消耗品使用、Buff 生命周期）没有端到端验证
3. **没有 Golden Test**：无法防止"改一个 Buff 悄悄搞坏整个战斗"
4. **proptest/insta 已引入但未使用**：基础设施就绪但零覆盖
5. **BattleRecord 已有序列化**：`BattleRecord` 已实现 `Serialize + Deserialize`，Golden Test 基础设施就绪

### 1.3 已有基础设施

```text
tests/common/
├── mod.rs              ← 公共导出
├── fixtures.rs         ← UnitBuilder (warrior/mage/goblin + with_hp/enemy/player + spawn)
├── app_builder.rs      ← minimal_app() / combat_app()
└── assertions.rs       ← assert_attr_eq! / assert_has_buff! / assert_has_tag! / assert_not_has_tag!
```

---

## 二、5 层测试金字塔

```text
                  ┌──────────────┐
                  │  Golden Test │  ← 战斗回放快照对比（insta）
                  └──────────────┘
               ┌───────────────────┐
               │  Scenario Test    │  ← BDD 风格战斗场景（20~50个封顶）
               └───────────────────┘
            ┌────────────────────────┐
            │    Feature Test        │  ← 单 Feature 完整流程（最优先补充）
            └────────────────────────┘
         ┌─────────────────────────────┐
         │     System Test             │  ← 单 Bevy System 验证
         └─────────────────────────────┘
┌──────────────────────────────────────────┐
│            Rule Test                      │  ← 纯函数/规则验证（含 proptest）
└──────────────────────────────────────────┘
```

### 各层占比目标

| 层 | 当前 | 目标 | 说明 |
|----|------|------|------|
| Rule Test | 423 (85%) | 70% | 保持，随新功能自然增长，增加 proptest |
| System Test | ~25 (5%) | 10% | 需要补充 App-based 测试 |
| Feature Test | 0 (0%) | 12% | **最优先补充** |
| Scenario Test | 0 (0%) | 5% | 20~50 个封顶 |
| Golden Test | 0 (0%) | 3% | 10~20 个战斗快照 |

---

## 三、第 0 阶段：测试基础设施增强

### 3.1 现状

`tests/common/` 已建立，但缺少：
- `combat_helpers.rs`：战斗测试辅助（攻击、施法、回合推进）
- 更多 App 构建器变体（装备测试、回合测试、完整战斗测试）
- insta/proptest 的公共辅助

### 3.2 新增 `combat_helpers.rs`

```rust
// tests/common/combat_helpers.rs
//! 战斗测试辅助：简化攻击、施法、回合推进等操作

use bevy::prelude::*;
use tactical_rpg::battle::pipeline::CombatIntent;
use tactical_rpg::core::effect::EffectQueue;
use tactical_rpg::core::attribute::AttributeKind;
use tactical_rpg::character::Unit;
use tactical_rpg::turn::{NeedsResolve, TurnPhase};

/// 发送攻击意图（近战）
pub fn attack(app: &mut App, attacker: Entity, target: Entity) {
    app.world_mut().send_message(CombatIntent::MeleeAttack {
        attacker,
        target,
    });
}

/// 推进一个 Update tick
pub fn tick(app: &mut App) {
    app.update();
}

/// 推进 N 个 Update tick
pub fn tick_n(app: &mut App, n: u32) {
    for _ in 0..n {
        app.update();
    }
}

/// 获取角色当前 HP
pub fn get_hp(app: &App, entity: Entity) -> f32 {
    app.world().get::<tactical_rpg::core::attribute::Attributes>(entity)
        .unwrap()
        .get(AttributeKind::Hp)
}

/// 获取角色当前 MP
pub fn get_mp(app: &App, entity: Entity) -> f32 {
    app.world().get::<tactical_rpg::core::attribute::Attributes>(entity)
        .unwrap()
        .get(AttributeKind::Mp)
}
```

### 3.3 新增 App 构建器变体

```rust
// 在 tests/common/app_builder.rs 中新增

/// 装备测试 App：combat_app + EquipmentRegistry + ItemRegistry + EquipItem/UnequipItem Message
pub fn equipment_app() -> App {
    let mut app = combat_app();
    // 注册穿脱 Message
    app.add_message::<tactical_rpg::equipment::EquipItem>()
        .add_message::<tactical_rpg::equipment::UnequipItem>()
        .add_message::<tactical_rpg::equipment::ItemEquipped>()
        .add_message::<tactical_rpg::equipment::ItemUnequipped>()
        .add_message::<tactical_rpg::equipment::EquipFailed>();
    // 注册装备/物品系统
    app.add_systems(Update, tactical_rpg::equipment::equip_item_system);
    app.add_systems(Update, tactical_rpg::equipment::unequip_item_system);
    app
}

/// 完整战斗 App：combat_app + Effect Pipeline + BattleRecord
pub fn full_battle_app() -> App {
    let mut app = combat_app();
    // 注册战斗 Message
    app.add_message::<tactical_rpg::battle::events::DamageApplied>()
        .add_message::<tactical_rpg::battle::events::HealApplied>()
        .add_message::<tactical_rpg::battle::events::CharacterDied>()
        .add_message::<tactical_rpg::battle::events::DotApplied>()
        .add_message::<tactical_rpg::battle::events::HotApplied>()
        .add_message::<tactical_rpg::battle::events::StunApplied>();
    // 初始化 BattleRecord
    app.init_resource::<tactical_rpg::battle::record::BattleRecord>();
    // 注册战斗记录系统
    app.add_systems(Update, tactical_rpg::battle::record::record_damage);
    app.add_systems(Update, tactical_rpg::battle::record::record_heal);
    app.add_systems(Update, tactical_rpg::battle::record::record_character_died);
    app
}
```

### 3.4 更新 `tests/common/mod.rs`

```rust
pub mod fixtures;
pub mod app_builder;
pub mod assertions;
pub mod combat_helpers;
```

---

## 四、第 1 层：Rule Test（规则测试）

### 4.1 现状

423 个单元测试，已经很好。保持现有风格，随新功能自然增长。

### 4.2 改进：引入 proptest 属性测试

对关键公式进行属性测试，覆盖边界条件。proptest 已在 `Cargo.toml` 的 `[dev-dependencies]` 中声明但未使用。

### 4.3 需要属性测试的规则

| 规则 | 位置 | 测试属性 | 文件 |
|------|------|----------|------|
| 伤害公式 | `battle/pipeline/execute.rs` → `calculate_damage_from_effect()` | 伤害 ≥ 1，暴击 > 普攻 | `tests/rules/damage_formula.rs` |
| 属性计算 | `core/attribute/types.rs` → `Attributes::get()` | 修饰符叠加后 ≥ 基础值下限 | `tests/rules/attribute_calc.rs` |
| 标签位运算 | `core/tag.rs` → `GameplayTags` | 标签组合无冲突，位运算幂等 | `tests/rules/tag_bitop.rs` |
| 堆叠合并 | `inventory/container.rs` → `Container::add_stack()` | 合并后 count ≤ stack_size | `tests/rules/stack_merge.rs` |

### 4.4 示例：伤害公式属性测试

```rust
// tests/rules/damage_formula.rs
//! 伤害公式属性测试

use proptest::prelude::*;
use tactical_rpg::battle::pipeline::execute::calculate_damage_from_effect;

proptest! {
    /// 伤害值始终 ≥ 1
    #[test]
    fn 伤害值始终非负(
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
        prop_assert!(damage >= 1, "伤害 {} < 1", damage);
    }

    /// 忽视防御比例越高，伤害越高（其他参数相同时）
    #[test]
    fn 忽视防御提高伤害(
        atk in 100.0_f32..500.0,
        def in 50.0_f32..200.0,
        base_def in 50.0_f32..200.0,
        multiplier in 1.0_f32..3.0,
    ) {
        let low_ignore = calculate_damage_from_effect(
            atk, def, base_def, multiplier, 0.0, 0,
        );
        let high_ignore = calculate_damage_from_effect(
            atk, def, base_def, multiplier, 0.5, 0,
        );
        prop_assert!(high_ignore >= low_ignore,
            "忽视防御50%伤害 {} 应 >= 忽视防御0%伤害 {}", high_ignore, low_ignore);
    }
}
```

### 4.5 示例：堆叠合并属性测试

```rust
// tests/rules/stack_merge.rs
//! 容器堆叠合并属性测试

use proptest::prelude::*;
use tactical_rpg::inventory::container::{Container, ContainerKind};
use tactical_rpg::inventory::definition::{ItemDef, ItemType, ItemRegistry};
use tactical_rpg::inventory::instance::{ItemInstance, ItemStack, InstanceIdCounter};

/// 生成测试用 ItemDef（stack_size 可控）
fn make_item_def(id: &str, stack_size: u32) -> ItemDef {
    ItemDef {
        id: id.into(),
        name: id.into(),
        description: String::new(),
        item_type: ItemType::Consumable,
        use_effects: vec![],
        stack_size,
        weight: 0.0,
        slot: None,
        tags: vec![],
    }
}

proptest! {
    /// 合并后 count 不超过 stack_size
    #[test]
    fn 合并后不超过堆叠上限(
        stack_size in 1_u32..99,
        add_count in 1_u32..200,
    ) {
        let mut registry = ItemRegistry::default();
        registry.register(make_item_def("potion", stack_size));
        let mut counter = InstanceIdCounter::default();

        let mut container = Container::new(ContainerKind::Backpack, 10);
        let instance = ItemInstance::new(counter.next(), "potion".into());
        let stack = ItemStack::new(instance, add_count);
        container.add_stack(stack, &registry);

        // 验证：所有堆叠的 count 都 ≤ stack_size
        for stack in container.stacks() {
            prop_assert!(stack.count <= stack_size,
                "堆叠 {} 超过上限 {}", stack.count, stack_size);
        }
    }
}
```

---

## 五、第 2 层：System Test（系统测试）

### 5.1 定义

测试单个 Bevy System 在 App 中的行为：Message 触发 → 组件变化 → Message 发送。

### 5.2 需要补充的 System Test

```text
tests/systems/
├── equip_item_system.rs       ← 穿戴：EquipItem → 属性/标签/Trait 变化
├── unequip_item_system.rs     ← 脱卸：UnequipItem → 属性恢复/物品回背包
├── use_item_system.rs         ← 消耗品：UseItem → 效果应用/数量减少
├── transfer_item_system.rs    ← 容器转移：TransferItem → 源减少/目标增加
├── buff_tick_system.rs        ← Buff tick：NeedsResolve → DoT/HoT/过期移除
├── execute_effects_system.rs  ← 效果执行：EffectQueue → HP变化/死亡/Message
└── trait_trigger_system.rs    ← Trait 触发：OnAttack → 效果生成
```

### 5.3 示例：装备穿戴 System Test

```rust
// tests/systems/equip_item_system.rs
//! 装备穿戴 System Test
//!
//! 验证：EquipItem Message → 属性变化 + 标签变化 + Trait 激活

use bevy::prelude::*;
use tactical_rpg::equipment::{EquipItem, EquipmentSlots, EquipmentRegistry, EquipmentSlot};
use tactical_rpg::inventory::container::Container;
use tactical_rpg::inventory::definition::ItemRegistry;
use tactical_rpg::inventory::instance::{ItemInstance, ItemStack, InstanceIdCounter};
use tactical_rpg::core::attribute::{AttributeKind, Attributes};
use tactical_rpg::character::PersistentTags;
use tactical_rpg::character::traits::TraitCollection;
use tactical_rpg::core::tag::GameplayTags;
use common::app_builder::equipment_app;
use common::fixtures::UnitBuilder;
use common::assertions::assert_attr_eq;

#[test]
fn 穿戴装备_属性增加() {
    let mut app = equipment_app();
    // 注册装备和物品定义
    let mut eq_reg = EquipmentRegistry::default();
    eq_reg.register_defaults();
    app.insert_resource(eq_reg);
    let item_reg = ItemRegistry::default();
    app.insert_resource(item_reg);

    // 生成角色
    let entity = UnitBuilder::warrior().spawn(&mut app);

    // 发送 EquipItem
    app.world_mut().send_message(EquipItem { target_entity: entity, instance_id: 1 });
    app.update();

    // 验证属性变化（具体值取决于 iron_sword 的 modifier）
    let attrs = app.world().get::<Attributes>(entity).unwrap();
    // 基础 Attack + 装备加成
    assert_attr_eq!(attrs, AttributeKind::Attack, attrs.get(AttributeKind::Attack));
}

#[test]
fn 穿戴装备_槽位占用() {
    let mut app = equipment_app();
    let mut eq_reg = EquipmentRegistry::default();
    eq_reg.register_defaults();
    app.insert_resource(eq_reg);
    let item_reg = ItemRegistry::default();
    app.insert_resource(item_reg);

    let entity = UnitBuilder::warrior().spawn(&mut app);

    app.world_mut().send_message(EquipItem { target_entity: entity, instance_id: 1 });
    app.update();

    let slots = app.world().get::<EquipmentSlots>(entity).unwrap();
    // 验证 MainHand 槽位已被占用
    assert!(slots.get(EquipmentSlot::MainHand).is_some());
}
```

### 5.4 示例：效果执行 System Test

```rust
// tests/systems/execute_effects_system.rs
//! 效果执行 System Test
//!
//! 验证：EffectQueue → HP 变化 → DamageApplied Message → 死亡判定

use bevy::prelude::*;
use tactical_rpg::core::effect::{EffectQueue, PendingEffect, PendingEffectData, EffectHandlerRegistry};
use tactical_rpg::core::attribute::{AttributeKind, Attributes};
use tactical_rpg::battle::pipeline::execute::execute_effects;
use tactical_rpg::battle::events::DamageApplied;
use tactical_rpg::character::Unit;
use common::app_builder::combat_app;
use common::fixtures::UnitBuilder;

#[test]
fn 伤害效果_减少目标HP() {
    let mut app = combat_app();
    app.add_message::<DamageApplied>();
    app.init_resource::<EffectQueue>();
    app.insert_resource(EffectHandlerRegistry::default());
    app.add_systems(Update, execute_effects);

    let target = UnitBuilder::warrior().with_hp(100.0).enemy().spawn(&mut app);

    // 入队伤害效果
    let mut queue = app.world_mut().resource_mut::<EffectQueue>();
    queue.pending.push(PendingEffect {
        target,
        data: PendingEffectData::Damage { amount: 30 },
    });

    app.update();

    // 验证 HP 减少
    let attrs = app.world().get::<Attributes>(target).unwrap();
    assert!((attrs.get(AttributeKind::Hp) - 70.0).abs() < 1.0);
}
```

---

## 六、第 3 层：Feature Test（最优先补充）

### 6.1 定义

一个 Feature = 一个业务系统的完整生命周期，跨多个 System/Module。这是 `docs/10.md` 强调的"下一步最应该补的"。

### 6.2 Feature Test 清单

```text
tests/features/
├── equipment_feature.rs       ← 装备系统完整流程
├── consumable_feature.rs      ← 消耗品系统完整流程
├── inventory_feature.rs       ← 背包系统完整流程
├── buff_feature.rs            ← Buff 系统完整流程
├── skill_feature.rs           ← 技能系统完整流程
├── trait_feature.rs           ← Trait 系统完整流程
└── death_feature.rs           ← 死亡处理完整流程
```

### 6.3 装备系统 Feature Test

```rust
// tests/features/equipment_feature.rs
//! 装备系统 Feature Test
//!
//! 完整流程：穿戴 → 属性变化 → 标签变化 → Trait 激活 → 脱卸 → 属性恢复 → 物品回背包

/// 场景1：战士穿戴铁剑 → Attack+3, SWORD标签 → 脱卸 → Attack恢复, 物品回背包
#[test]
fn 装备穿脱完整流程() {
    // 1. 创建角色，背包中放一把铁剑
    // 2. 穿戴铁剑
    // 3. 验证 Attack 增加
    // 4. 验证 SWORD 标签
    // 5. 验证 Trait 激活（如果铁剑有 Trait）
    // 6. 脱卸铁剑
    // 7. 验证 Attack 恢复
    // 8. 验证 SWORD 标签消失
    // 9. 验证物品回到背包
}

/// 场景2：穿戴需求不满足的装备 → EquipFailed
#[test]
fn 装备需求不满足() {
    // 1. 创建低属性角色
    // 2. 尝试穿戴需要高属性的装备
    // 3. 验证收到 EquipFailed Message
    // 4. 验证属性未变化
}

/// 场景3：穿戴新装备时自动脱卸旧装备
#[test]
fn 穿戴时自动脱卸旧装备() {
    // 1. 穿戴铁剑到 MainHand
    // 2. 穿戴钢剑到 MainHand
    // 3. 验证铁剑回到背包
    // 4. 验证属性是钢剑的加成
}

/// 场景4：装备授予 Trait → Trait 授予标签和修饰符 → 脱卸后全部清除
#[test]
fn 装备trait完整生命周期() {
    // 1. 穿戴带 Trait 的装备
    // 2. 验证 TraitCollection 中有对应 TraitEntry
    // 3. 验证 Trait 效果（标签/修饰符）生效
    // 4. 脱卸装备
    // 5. 验证 TraitEntry 被移除
    // 6. 验证 Trait 效果消失
}
```

### 6.4 消耗品系统 Feature Test

```rust
// tests/features/consumable_feature.rs
//! 消耗品系统 Feature Test
//!
//! 完整流程：背包有药水 → 使用 → HP恢复 → 数量-1 → 用完自动消失

/// 场景1：治疗药水使用完整流程
#[test]
fn 消耗品使用完整流程() {
    // 1. 角色受伤 HP=20/30
    // 2. 背包有治疗药水 x3
    // 3. 使用治疗药水
    // 4. 验证 HP+50（不超过 MaxHp）
    // 5. 验证背包药水 x2
    // 6. 再使用 2 次
    // 7. 验证背包中药水消失
}

/// 场景2：药水赋予 Buff
#[test]
fn 消耗品使用_药水赋予buff() {
    // 1. 使用"力量药水"（ApplyBuff 效果）
    // 2. 验证获得 Attack+5 Buff
    // 3. 推进 3 回合
    // 4. 验证 Buff 过期，Attack 恢复
}

/// 场景3：非消耗品不可使用
#[test]
fn 非消耗品不可使用() {
    // 1. 背包有装备类物品
    // 2. 尝试使用
    // 3. 验证无效果，数量不变
}
```

### 6.5 死亡处理 Feature Test

```rust
// tests/features/death_feature.rs
//! 死亡处理 Feature Test
//!
//! 完整流程：HP降为0 → Dead标记 → Hook触发(acted=true, 移除Selected) → Observer响应(从队列移除)

/// 场景1：角色死亡完整流程
#[test]
fn 死亡完整流程() {
    // 1. 角色受到致命伤害
    // 2. 验证 Dead 标记添加
    // 3. 验证 Hook 触发：acted = true
    // 4. 验证 Hook 触发：Selected 被移除
    // 5. 验证 Observer 触发：从 TurnOrder 队列移除
}

/// 场景2：死亡角色不再被 Buff tick 影响
#[test]
fn 死亡角色不受buff_tick影响() {
    // 1. 角色有 Poison Buff
    // 2. 角色死亡
    // 3. 推进回合
    // 4. 验证死亡角色不再受到 DoT
}

/// 场景3：全灭一方触发战斗结束
#[test]
fn 全灭触发战斗结束() {
    // 1. 只剩一个敌方
    // 2. 敌方死亡
    // 3. 验证战斗结束状态
}
```

### 6.6 Buff 系统 Feature Test

```rust
// tests/features/buff_feature.rs
//! Buff 系统 Feature Test
//!
//! 完整流程：添加Buff → 持续N回合 → 每回合效果 → 过期自动移除

/// 场景1：Poison 完整生命周期
#[test]
fn poison完整生命周期() {
    // 1. 角色获得 Poison Buff（3回合，每回合5伤害）
    // 2. 推进1回合 → HP-5，Poison 仍在
    // 3. 推进2回合 → HP-5，Poison 仍在
    // 4. 推进3回合 → HP-5，Poison 过期移除
    // 5. 推进4回合 → HP 不变
}

/// 场景2：增攻 Buff 修改属性
#[test]
fn 增攻buff修改属性() {
    // 1. 角色获得 Attack+5 Buff
    // 2. 验证 Attack 增加 5
    // 3. Buff 过期
    // 4. 验证 Attack 恢复
}

/// 场景3：Cleanse 移除所有 Debuff
#[test]
fn cleanse移除所有debuff() {
    // 1. 角色有 Poison + Stun 两个 Debuff
    // 2. 使用 Cleanse 效果
    // 3. 验证两个 Debuff 都被移除
    // 4. 验证属性恢复
}
```

---

## 七、第 4 层：Scenario Test（BDD 风格）

### 7.1 定义

跨 Feature 的战斗场景，Given-When-Then 风格。20~50 个封顶。维护成本极高，宁缺毋滥。

### 7.2 场景清单

```text
tests/scenarios/
├── fireball_vs_knight.rs       ← 法师火球 vs 骑士：伤害 + Burning
├── poison_battle.rs            ← 毒伤战斗：DoT + 回合推进 + 死亡
├── equipment_swap_battle.rs    ← 战斗中换装：脱卸旧 → 穿戴新 → 属性变化
├── terrain_advantage.rs        ← 地形优势：高地 + 森林 + 水域
└── boss_phase_transition.rs    ← Boss 阶段转换：HP阈值 → 新Trait → 新技能
```

### 7.3 示例：火球 vs 骑士

```rust
// tests/scenarios/fireball_vs_knight.rs
//! Scenario: 法师火球 vs 骑士
//!
//! Given: 法师(Int=10, Attack=8) 骑士(HP=100, Defense=5)
//! When:  法师释放火球术(Attack*2.5, 附带Burning 3回合)
//! Then:  骑士受到伤害
//! And:   骑士获得 Burning Buff
//! And:   后续 3 回合每回合受到 DoT
//! And:   3 回合后 Burning 消失

#[test]
fn 法师火球vs骑士_伤害与燃烧() {
    let mut app = full_battle_app();
    let mage = UnitBuilder::mage().spawn(&mut app);
    let knight = UnitBuilder::warrior().enemy().with_hp(100.0).spawn(&mut app);

    // 法师释放火球术（通过 Effect Pipeline）
    // ... 构造 PendingEffect 并入队 ...

    // 验证即时伤害
    let knight_attrs = app.world().get::<Attributes>(knight).unwrap();
    assert!(knight_attrs.get(AttributeKind::Hp) < 100.0);

    // 验证 Burning Buff
    let buffs = app.world().get::<ActiveBuffs>(knight).unwrap();
    assert_has_buff!(buffs, "burning");

    // 推进 3 回合
    for _ in 0..3 {
        // 触发 NeedsResolve + resolve_status_effects
        app.update();
    }

    // 验证 DoT 伤害
    let knight_attrs = app.world().get::<Attributes>(knight).unwrap();
    assert!(knight_attrs.get(AttributeKind::Hp) < 80.0); // 受到 DoT

    // 验证 Burning 过期
    let buffs = app.world().get::<ActiveBuffs>(knight).unwrap();
    assert!(!buffs.iter().any(|b| b.buff_id == "burning"));
}
```

### 7.4 示例：毒伤战斗

```rust
// tests/scenarios/poison_battle.rs
//! Scenario: 毒伤战斗
//!
//! Given: 战士(HP=30) vs 哥布林(HP=15)
//! When:  战士攻击哥布林 → 哥布林攻击战士（附带 Poison）
//! Then:  战士受到 Poison
//! And:   后续回合战士每回合受到 3 DoT
//! And:   战士 HP=30-5-3-3=19（3回合后）
//! And:   Poison 过期后战士 HP 不再下降

#[test]
fn 毒伤战斗_完整流程() {
    // 1. 战士攻击哥布林，哥布林死亡
    // 2. 哥布林攻击战士，附带 Poison
    // 3. 推进回合，验证 DoT
    // 4. Poison 过期，验证 HP 稳定
}
```

---

## 八、第 5 层：Golden Battle Test（战斗快照）

### 8.1 定义

运行预设战斗 → 生成 BattleRecord → 与 insta 快照对比。这是防止"改一个 Buff 搞坏整个战斗"的最强防线。

`docs/10.md` 原文：**很多大型策略游戏最终都是靠 Battle Replay + Golden Test + Snapshot Compare 来防止更新一个 Buff 后把整个战斗系统悄悄搞坏。**

### 8.2 基础设施

项目已有 `BattleRecord`（`Serialize + Deserialize`），结构：

```rust
#[derive(Resource, Default, Debug, Serialize, Deserialize)]
pub struct BattleRecord {
    pub entries: Vec<BattleEntry>,
    pub turn_number: u32,
}

pub enum BattleEntry {
    TurnStarted { turn: u32 },
    TurnEnded { turn: u32 },
    DamageApplied { target: Entity, target_name: String, amount: i32, ... },
    HealApplied { target: Entity, target_name: String, amount: i32 },
    DotApplied { target: Entity, target_name: String, amount: i32, ... },
    HotApplied { target: Entity, target_name: String, amount: i32 },
    StunApplied { target: Entity, target_name: String },
    CharacterDied { entity: Entity, name: String, faction: Faction },
}
```

### 8.3 目录结构

```text
tests/golden/
├── mod.rs                  ← Golden Test 框架 + BattleSimulator
├── battle_001_basic.rs     ← 基础战斗：战士 vs 哥布林
├── battle_002_buff.rs      ← Buff 战斗：中毒 + 治疗
├── battle_003_equipment.rs ← 装备战斗：换装影响伤害
└── snapshots/              ← insta 自动管理
    ├── battle_001_basic@basic.snap.yml
    ├── battle_002_buff@poison_and_heal.snap.yml
    └── battle_003_equipment@swap_affects_damage.snap.yml
```

### 8.4 BattleSimulator 设计

```rust
// tests/golden/mod.rs
//! Golden Battle Test 框架

use bevy::prelude::*;
use tactical_rpg::battle::record::BattleRecord;
use tactical_rpg::battle::events::*;
use tactical_rpg::core::effect::EffectQueue;
use tactical_rpg::core::attribute::{AttributeKind, Attributes};
use tactical_rpg::character::Unit;
use common::app_builder::full_battle_app;
use common::fixtures::UnitBuilder;

/// 战斗模拟器：驱动 App 执行战斗，收集 BattleRecord
pub struct BattleSimulator {
    app: App,
}

impl BattleSimulator {
    pub fn new() -> Self {
        let mut app = full_battle_app();
        // 注册战斗记录系统
        app.add_systems(Update, tactical_rpg::battle::record::record_turn_started);
        app.add_systems(Update, tactical_rpg::battle::record::record_turn_ended);
        app.add_systems(Update, tactical_rpg::battle::record::record_dot);
        app.add_systems(Update, tactical_rpg::battle::record::record_hot);
        app.add_systems(Update, tactical_rpg::battle::record::record_stun);
        Self { app }
    }

    /// 添加角色
    pub fn add_unit(&mut self, builder: UnitBuilder) -> Entity {
        builder.spawn(&mut self.app)
    }

    /// 入队伤害效果
    pub fn deal_damage(&mut self, target: Entity, amount: i32) {
        let mut queue = self.app.world_mut().resource_mut::<EffectQueue>();
        queue.pending.push(tactical_rpg::core::effect::PendingEffect {
            target,
            data: tactical_rpg::core::effect::PendingEffectData::Damage { amount },
        });
    }

    /// 执行一个 Update tick
    pub fn tick(&mut self) {
        self.app.update();
    }

    /// 执行 N 个 tick
    pub fn tick_n(&mut self, n: u32) {
        for _ in 0..n {
            self.app.update();
        }
    }

    /// 获取战斗记录
    pub fn record(&self) -> &BattleRecord {
        self.app.world().resource::<BattleRecord>()
    }

    /// 获取角色 HP
    pub fn get_hp(&self, entity: Entity) -> f32 {
        self.app.world().get::<Attributes>(entity).unwrap().get(AttributeKind::Hp)
    }
}
```

### 8.5 Golden Test 示例

```rust
// tests/golden/battle_001_basic.rs
//! Golden Test: 基础战斗 — 战士 vs 哥布林

use super::*;
use insta;

#[test]
fn 基础战斗_战士vs哥布林() {
    let mut sim = BattleSimulator::new();
    let warrior = sim.add_unit(UnitBuilder::warrior().with_hp(30.0));
    let goblin = sim.add_unit(UnitBuilder::goblin().enemy().with_hp(15.0));

    // 战士攻击哥布林
    sim.deal_damage(goblin, 15);
    sim.tick();

    // 快照对比
    insta::assert_yaml_snapshot!(sim.record(), {
        ".entries[].DamageApplied.target" => "[entity]",
        ".entries[].DamageApplied.attacker" => "[entity]",
    });
}
```

### 8.6 Entity 不确定性处理

Golden Test 中 Entity ID 是运行时分配的，每次可能不同。解决方案：

```rust
// 方案1：insta redaction（推荐）
insta::assert_yaml_snapshot!(record, {
    ".entries[].DamageApplied.target" => "[entity]",
    ".entries[].DamageApplied.attacker" => "[entity]",
    ".entries[].CharacterDied.entity" => "[entity]",
});

// 方案2：BattleRecord 中已有 target_name/attacker_name
// 快照中只比较 name 字段，Entity 用 redaction 替换
```

### 8.7 Golden Test 工作流

```text
1. 首次运行：insta 自动生成 .snap.new 文件
2. 审阅快照：确认战斗记录符合预期
3. 接受快照：cargo insta accept
4. 后续运行：如果 BattleRecord 与快照不一致 → FAIL
5. 有意修改：cargo insta review → cargo insta accept
```

---

## 九、Bevy 0.18 特定注意事项

### 9.1 Message 必须注册

Bevy 0.18 的 Message 需要先 `app.add_message::<T>()` 注册，否则 `MessageReader<T>` 参数会导致系统 panic。

**测试中必须确保**：
- 每个使用 `MessageReader<T>` 的系统，其对应 Message 类型已在 App 中注册
- `combat_app()` / `equipment_app()` / `full_battle_app()` 中统一注册常用 Message
- 参考错误：`turn_flow.rs` 中缺少 `ForceEndTurn` Message 注册导致 panic

### 9.2 Hook 自动触发

`Dead` 组件的 `on_add` Hook 会在测试中自动触发：
- 设置 `unit.acted = true`
- 移除 `Selected` 组件

测试中不需要手动模拟这些行为，只需验证 Hook 的副作用。

### 9.3 Required Components

`Unit` 组件通过 `#[require(...)]` 自动注入 `Attributes`、`EquipmentSlots`、`Container` 等。测试中使用 `UnitBuilder::spawn()` 时，这些组件会自动创建。

### 9.4 Observer

`on_character_died` Observer 监听 `CharacterDied` Message，从 `TurnOrder` 队列移除死亡单位。测试中需要注册此 Observer 才能验证完整死亡流程。

---

## 十、现有集成测试迁移计划

### 10.1 迁移映射

| 现有文件 | 迁移目标 | 说明 |
|----------|----------|------|
| `tests/buff_damage.rs` | `tests/features/buff_feature.rs` | 合并到 Buff Feature Test |
| `tests/buff_lifecycle.rs` | `tests/features/buff_feature.rs` | 合并到 Buff Feature Test |
| `tests/combat_pipeline.rs` | `tests/systems/execute_effects_system.rs` | 迁移到 System Test |
| `tests/skill_system.rs` | `tests/features/skill_feature.rs` | 迁移到 Feature Test |
| `tests/terrain_combat.rs` | `tests/scenarios/terrain_advantage.rs` | 迁移到 Scenario Test |
| `tests/turn_flow.rs` | `tests/features/turn_feature.rs` | 迁移到 Feature Test |
| `tests/edge_cases.rs` | 保留原位 | 边界情况测试，不迁移 |

### 10.2 迁移原则

- **渐进式迁移**：先创建新目录和文件，再逐步迁移，不一次性删除旧文件
- **保持测试通过**：迁移过程中旧文件和新文件共存，确保始终绿色
- **迁移完成后删除旧文件**：确认新文件覆盖所有旧测试后删除

---

## 十一、最终目录结构

```text
tests/
├── common/
│   ├── mod.rs
│   ├── fixtures.rs           ← UnitBuilder, 角色模板
│   ├── app_builder.rs        ← minimal_app(), combat_app(), equipment_app(), full_battle_app()
│   ├── assertions.rs         ← assert_attr_eq!, assert_has_buff!, assert_has_tag!
│   └── combat_helpers.rs     ← attack(), tick(), get_hp(), get_mp()
│
├── rules/                    ← 第 1 层：属性测试（proptest）
│   ├── damage_formula.rs
│   ├── attribute_calc.rs
│   ├── tag_bitop.rs
│   └── stack_merge.rs
│
├── systems/                  ← 第 2 层：单 System 测试
│   ├── equip_item_system.rs
│   ├── unequip_item_system.rs
│   ├── use_item_system.rs
│   ├── transfer_item_system.rs
│   ├── buff_tick_system.rs
│   ├── execute_effects_system.rs
│   └── trait_trigger_system.rs
│
├── features/                 ← 第 3 层：Feature 完整流程（最优先）
│   ├── equipment_feature.rs
│   ├── consumable_feature.rs
│   ├── inventory_feature.rs
│   ├── buff_feature.rs
│   ├── skill_feature.rs
│   ├── trait_feature.rs
│   ├── death_feature.rs
│   └── turn_feature.rs
│
├── scenarios/                ← 第 4 层：BDD 风格战斗场景
│   ├── fireball_vs_knight.rs
│   ├── poison_battle.rs
│   ├── equipment_swap_battle.rs
│   ├── terrain_advantage.rs
│   └── boss_phase_transition.rs
│
├── golden/                   ← 第 5 层：战斗快照（insta）
│   ├── mod.rs                ← BattleSimulator
│   ├── battle_001_basic.rs
│   ├── battle_002_buff.rs
│   ├── battle_003_equipment.rs
│   └── snapshots/            ← insta 自动管理
│
├── edge_cases.rs             ← 现有，保留
├── buff_damage.rs            ← 现有，后续迁移到 features/
├── buff_lifecycle.rs         ← 现有，后续迁移到 features/
├── combat_pipeline.rs        ← 现有，后续迁移到 systems/
├── skill_system.rs           ← 现有，后续迁移到 features/
├── terrain_combat.rs         ← 现有，后续迁移到 scenarios/
└── turn_flow.rs              ← 现有，后续迁移到 features/
```

---

## 十二、实施计划

### 阶段 0：测试基础设施增强

| 任务 | 产出 |
|------|------|
| 新增 `combat_helpers.rs` | `tests/common/combat_helpers.rs` |
| 新增 `equipment_app()` / `full_battle_app()` | `tests/common/app_builder.rs` 更新 |
| 更新 `tests/common/mod.rs` | 添加 `pub mod combat_helpers` |

### 阶段 1：Rule Test 增强（proptest）

| 任务 | 产出 |
|------|------|
| 伤害公式 proptest | `tests/rules/damage_formula.rs` |
| 属性计算 proptest | `tests/rules/attribute_calc.rs` |
| 标签位运算 proptest | `tests/rules/tag_bitop.rs` |
| 堆叠合并 proptest | `tests/rules/stack_merge.rs` |

### 阶段 2：System Test 补充

| 任务 | 产出 |
|------|------|
| 装备穿脱 System Test | `tests/systems/equip_item_system.rs` |
| 消耗品使用 System Test | `tests/systems/use_item_system.rs` |
| 容器转移 System Test | `tests/systems/transfer_item_system.rs` |
| Buff tick System Test | `tests/systems/buff_tick_system.rs` |
| 效果执行 System Test | `tests/systems/execute_effects_system.rs` |
| Trait 触发 System Test | `tests/systems/trait_trigger_system.rs` |

### 阶段 3：Feature Test（最优先）

| 任务 | 产出 |
|------|------|
| 装备系统 Feature Test | `tests/features/equipment_feature.rs` |
| 消耗品系统 Feature Test | `tests/features/consumable_feature.rs` |
| 背包系统 Feature Test | `tests/features/inventory_feature.rs` |
| Buff 系统 Feature Test | `tests/features/buff_feature.rs` |
| 死亡处理 Feature Test | `tests/features/death_feature.rs` |
| 技能系统 Feature Test | `tests/features/skill_feature.rs` |
| Trait 系统 Feature Test | `tests/features/trait_feature.rs` |
| 回合系统 Feature Test | `tests/features/turn_feature.rs` |

### 阶段 4：Scenario Test

| 任务 | 产出 |
|------|------|
| 火球 vs 骑士 | `tests/scenarios/fireball_vs_knight.rs` |
| 毒伤战斗 | `tests/scenarios/poison_battle.rs` |
| 装备换装战斗 | `tests/scenarios/equipment_swap_battle.rs` |
| 地形优势 | `tests/scenarios/terrain_advantage.rs` |

### 阶段 5：Golden Battle Test

| 任务 | 产出 |
|------|------|
| BattleSimulator | `tests/golden/mod.rs` |
| 基础战斗快照 | `tests/golden/battle_001_basic.rs` |
| Buff 战斗快照 | `tests/golden/battle_002_buff.rs` |
| 装备战斗快照 | `tests/golden/battle_003_equipment.rs` |

### 阶段 6：迁移现有集成测试

| 任务 | 产出 |
|------|------|
| 迁移 buff_damage + buff_lifecycle → buff_feature | 合并到 `tests/features/buff_feature.rs` |
| 迁移 combat_pipeline → execute_effects_system | 迁移到 `tests/systems/execute_effects_system.rs` |
| 迁移 skill_system → skill_feature | 迁移到 `tests/features/skill_feature.rs` |
| 迁移 terrain_combat → terrain_advantage | 迁移到 `tests/scenarios/terrain_advantage.rs` |
| 迁移 turn_flow → turn_feature | 迁移到 `tests/features/turn_feature.rs` |
| 删除已迁移的旧文件 | 清理 `tests/` 根目录 |

---

## 十三、关键设计决策

### 13.1 为什么不用 cucumber-rs？

| 方案 | 优点 | 缺点 |
|------|------|------|
| cucumber-rs | Gherkin 语法，非程序员可读 | 引入重依赖，测试运行慢，维护成本高 |
| 中文场景命名 | 零依赖，可读性好，Rust 原生 | 非程序员不可读 |

**决策**：用中文场景命名 + 结构化注释代替 BDD 框架。项目目前没有非程序员写测试的需求。

### 13.2 为什么用 insta 而不是手写 YAML 对比？

| 方案 | 优点 | 缺点 |
|------|------|------|
| 手写 YAML 对比 | 无依赖 | 维护 expected 文件痛苦，diff 不友好 |
| insta | 自动快照、友好 diff、review 工作流 | 引入依赖 |

**决策**：insta 是 Rust 生态快照测试的事实标准，BattleRecord 已有 Serialize，零成本接入。

### 13.3 为什么 Rule Test 保留在 src/？

**决策**：Rule Test 保留在 `src/` 的 `#[cfg(test)] mod tests` 中。它们与实现代码紧密耦合，放在源文件旁边更易维护。`tests/rules/` 只放 proptest 属性测试。

### 13.4 Entity 不确定性处理

Golden Test 中 Entity ID 是运行时分配的。解决方案：

```rust
// 方案1：insta redaction（推荐）
insta::assert_yaml_snapshot!(record, {
    ".entries[].DamageApplied.target" => "[entity]",
    ".entries[].DamageApplied.attacker" => "[entity]",
    ".entries[].CharacterDied.entity" => "[entity]",
});

// 方案2：BattleRecord 中已有 target_name/attacker_name
// 快照中 Entity 用 redaction 替换，只比较 name 字段
```

### 13.5 测试与 Bevy 0.18 Message 系统

Bevy 0.18 的 Message 需要先 `app.add_message::<T>()` 注册。测试辅助函数中应统一注册所有常用 Message，避免遗漏导致 panic。

### 13.6 迁移策略

**渐进式迁移**而非一次性重写。旧文件和新文件共存期间保持全部绿色，迁移完成后再删除旧文件。这避免了"迁移期间测试全红"的风险。

---

## 十四、验收标准

| 阶段 | 验收标准 |
|------|----------|
| 阶段 0 | `combat_helpers.rs` + `equipment_app()` + `full_battle_app()` 可用，现有测试全部通过 |
| 阶段 1 | 4 个 proptest 文件，每个至少 2 个属性 |
| 阶段 2 | 6 个 System Test 文件，每个至少 2 个测试 |
| 阶段 3 | 8 个 Feature Test 文件，每个至少 3 个场景 |
| 阶段 4 | 4 个 Scenario Test 文件，每个 1~2 个场景 |
| 阶段 5 | BattleSimulator + 3 个 Golden Test + insta 快照 |
| 阶段 6 | 旧文件全部迁移，`tests/` 根目录只剩 `edge_cases.rs` |

**总目标**：从 423 单元测试 + 7 集成测试增长到 ~700 测试，测试金字塔从"底重顶空"变为"5 层均衡"。

---

## 十五、优先级排序（来自 docs/10.md）

```text
1. Feature Test          ★★★★★  ← 下一步最应该补的
2. Golden Battle Test    ★★★★★  ← 防止改一个Buff搞坏整个战斗
3. Battle Replay System  ★★★★★  ← 后期最强防线
4. Scenario (BDD)        ★★★★☆  ← 典型场景验证
5. 更多 Unit Test        ★★☆☆☆  ← 已经够多了
```

**核心洞察**（来自 `docs/10.md`）：

> 你现在最大的风险已经不是"伤害公式写错"，而是"Buff + 技能 + 装备 + 地形 + 回合 + AI 一起工作时出问题"。这时候 Feature Test + Golden Battle Test 的收益会远高于纯 BDD。
