---
id: 01-architecture.ADR-030
title: ADR-030 — Progression & Inventory Architecture
status: approved
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-030: 成长与物品系统架构

## 状态

**Approved** — 依赖 ADR-011（Modifier Pipeline）和 ADR-020（Combat Pipeline），本架构决策正式生效。

## 背景

成长系统（经验/等级/技能解锁）和物品系统（装备/消耗品/物品栏）是 SRPG 角色养成的核心。它们都通过 Modifier Pipeline 影响角色的战斗能力——升级获得属性成长，装备提供 Modifier 加成。

## 引用的领域规则与数据架构

- `docs/02-domain/domains/progression_domain.md` — Progression 领域规则
- `docs/02-domain/domains/inventory_domain.md` — Inventory 领域规则
- `docs/04-data/domains/progression_schema.md` — Progression Schema
- `docs/04-data/domains/inventory_schema.md` — Inventory Schema
- `.trae/rules/SRPG专项规则.md` §一（角色分层扩展体系）

## 决策

### 1. Progression 架构

#### 1.1 经验与等级

```rust
/// Experience — 运行时状态
#[derive(Component)]
pub struct Experience {
    pub current_xp: u64,
    pub level: u32,
    pub total_xp_earned: u64,
}

/// ProgressionDef — 配置文件加载
#[derive(Asset, TypePath)]
pub struct ProgressionDef {
    pub class_id: ClassId,
    pub base_stats: HashMap<AttributeId, f32>,     // 基础属性
    pub growth_rates: HashMap<AttributeId, f32>,   // 每级成长
    pub xp_curve: XpCurve,                         // 升级曲线
    pub skill_unlocks: Vec<SkillUnlock>,            // 技能解锁表
}

pub struct XpCurve {
    /// 计算第 level 级所需的经验
    pub fn xp_for_level(&self, level: u32) -> u64;
}

pub struct SkillUnlock {
    pub at_level: u32,
    pub ability_id: AbilityDefId,
    pub slot_type: SkillSlotType,  // Active | Passive | Ultimate
}
```

#### 1.2 升级流程

```
CombatResult → XP 结算
       │
       ▼
AddXpSystem
  ├── 增加经验值
  └── 检查是否升级
         │
         ├── 否 → 无事
         │
         └── 是 → LevelUpEvent (Event)
                │
                ▼
         LevelUpSystem
           ├── 按 growth_rates 增加基础属性
           ├── 触发 ModifierPipeline（属性成长）
           ├── 检查 skill_unlocks
           └── 发布 LeveledUp 领域事件
```

#### 1.3 成长属性处理

属性成长走 Modifier Pipeline——每个升级带来的属性提升是一个永久的 Modifier：

```rust
fn apply_level_up(
    mut level_events: EventReader<LevelUpEvent>,
    growth_defs: Res<Assets<ProgressionDef>>,
    mut commands: Commands,
) {
    for ev in level_events.read() {
        let growth = growth_defs.get(ev.class_id).unwrap();
        for (attr_id, growth_rate) in &growth.growth_rates {
            // 创建永久 Modifier 表达属性成长
            commands.entity(ev.entity).trigger(ApplyModifier {
                stat: *attr_id,
                value: ModifierValue::Add(*growth_rate),
                source_type: SourceType::LevelUp,
                // 永久 Modifier，无 Duration
            });
        }
    }
}
```

### 2. Inventory 架构

#### 2.1 物品定义

```rust
/// ItemDef — 配置文件加载
#[derive(Asset, TypePath)]
pub struct ItemDef {
    pub id: ItemDefId,
    pub item_type: ItemType,          // Weapon | Armor | Consumable | KeyItem | Material
    pub equip_slot: Option<EquipSlot>, // Weapon | Head | Body | Accessory | ...
    pub modifiers: Vec<ModifierDefId>, // 装备状态下提供的 Modifier
    pub use_effects: Vec<EffectDefId>, // 使用时触发的 Effect（消耗品）
    pub stackable: bool,
    pub max_stack: u32,
    pub tags: Vec<TagId>,
}

/// Inventory — 运行时
#[derive(Component)]
pub struct Inventory {
    pub items: Vec<ItemStack>,
    pub max_slots: u32,
}

pub struct ItemStack {
    pub item_def_id: ItemDefId,
    pub quantity: u32,
    pub slot_index: u32,
}

/// Equipment — 当前装备
#[derive(Component)]
pub struct Equipment {
    pub slots: EnumMap<EquipSlot, Option<ItemStack>>,
}

pub enum EquipSlot {
    Weapon,
    Head,
    Body,
    Accessory1,
    Accessory2,
}
```

#### 2.2 装备 Modifier 链路

```
装备穿戴
       │
       ▼
EquipSystem
  ├── 从 Inventory 中移除物品
  ├── 写入 Equipment.slots
  └── 触发 ApplyModifier
         │
         ▼
  ModifierPipeline (ADR-011)
    ├── 收集装备提供的 ModifierDefIds
    ├── 创建 ModifierEntry
    └── 重新计算属性
         │
         ▼
  装备卸下
       │
       ▼
  UnequipSystem
    └── 触发 RemoveModifier
```

#### 2.3 物品使用（消耗品）

```
使用物品 (玩家/AI 输入)
       │
       ▼
UseItemSystem
  ├── 检查 Inventory 中是否有物品
  ├── 扣减数量（stackable）
  ├── 触发 ItemUsed 领域事件
  └── 进入 Ability Pipeline
         │
         ▼
  Ability Pipeline (ADR-010)
    └── 执行 use_effects
```

### 3. 跨 Feature 协作

```
装备提供 Modifier → modifier Feature 负责属性计算
     ↑
物品掉落 ← combat Feature 战斗结算
     ↑
等级提升 ← progression Feature 经验积累
     ↑
Ability 解锁 ← progression Feature 技能表
     ↓
ability Feature 负责 Ability 注册
```

## Module Design

```
src/core/domains/progression/
  ├── plugin.rs              — ProgressionPlugin
  ├── components.rs          — Experience, ClassComponent
  ├── systems.rs             — add_xp, level_up, skill_unlock
  ├── events.rs              — XpGained, LevelUpEvent, SkillUnlocked
  └── integration/           — 跨域访问 ACL（ADR-046） ProgressionDef, XpCurve

src/core/domains/inventory/
  ├── plugin.rs              — InventoryPlugin
  ├── components.rs          — Inventory, Equipment, ItemStack
  ├── systems.rs             — equip, unequip, use_item, pickup_item
  ├── events.rs              — ItemUsed, EquipmentChanged
  └── integration/           — 跨域访问 ACL（ADR-046） ItemDef, EquipSlot
```

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| Combat → XP | Event (`XpGained`) | combat → progression |
| 升级 → Modifier | Trigger (`ApplyModifier`) | progression → modifier |
| 装备 → Modifier | Trigger (`ApplyModifier`, `RemoveModifier`) | inventory → modifier |
| 使用物品 → Ability | Event (`UseItemEvent`) → Ability Pipeline | inventory → ability |
| 装备变更 → UI | Observer (`EquipmentChanged`) | inventory → ui |

## 边界定义

### 允许
- Progression 读取 Attribute 数据（计算成长率）
- Inventory 通过 Trigger 添加/移除 Modifier
- Combat 结算后发送 XpGained Event
- 装备销毁时自动清理 Modifier

### 🟥 禁止
- Progression 直接修改战斗属性字段（必须走 Modifier Pipeline）
- Inventory 直接执行 Effect（必须通过 Ability Pipeline）
- 物品 Definition 中包含运行时状态
- 装备 Modifier 在卸下后残留

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| `level_up` 直接 `hero.attack += 5` | 违反 Modifier Pipeline |
| 消耗品 effect 直接 `health.current += 50` | 必须经过 Effect Pipeline |
| ItemDef 中硬编码角色名 | 违反 Rule/Content 分离 |
| 装备槽位硬编码（不支持种族特有槽位） | 配置应可扩展 |
| 升级时不检查 skill_unlock 表 | 技能解锁逻辑不一致 |

## Definition / Instance Design

- **Definition**: `ProgressionDef` (Asset), `ItemDef` (Asset), `XpCurve` (config)
- **Instance**: `Experience` (Component), `Inventory` (Component), `Equipment` (Component)
- **Persistence**: `Experience`, `Inventory.items`, `Equipment.slots`

## 后果

### 正面
- 属性成长和装备加成统一走 Modifier Pipeline
- 物品使用复用 Ability Pipeline
- 经验/等级逻辑独立，不耦合到战斗系统

### 负面
- 升级属性成长全部走 Modifier 会使 ModifierSet 偏大（需要关注性能）
- 装备/卸下的 Modifier 变更可能触发 Observer 链，需要注意频率

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 升级直接写属性字段 | 违反 Modifier Pipeline，无法追溯 |
| 物品使用自建 Effect 系统 | 复用已有 Effect Pipeline |
| 装备 Modifier 缓存在 Equipment Component 中 | 数据冗余，需与 ModifierSet 同步 |
| 每件物品一个 Entity | 物品数量多时 Entity 膨胀 |

## 评审要点

- [ ] ProgressionDef 的职业成长率：多少职业、每种多少属性？配置复杂度可控？
- [ ] Inventory 的最大槽位数——固定还是可扩展（背包升级）？
- [ ] 装备 Modifier 叠加上限——多件装备加同一属性如何处理？
- [ ] 技能解锁是自动学习还是需要消耗资源（技能点/SP）？
