---
id: domains.inventory.schema.v1
title: Inventory Schema — 背包/物品数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition, instance, persistence
replay-safe: false
---

# Inventory Schema — 背包/物品数据架构

> **领域归属**: Domains — 成长养成层 | **依赖 Schema**: Modifier, Effect, Event, Condition | **定义依据**: `docs/02-domain/inventory_domain.md`

---

## 1. Schema Design

### 1.1 ItemDef（Definition 层）

```rust
/// 物品模板的静态定义。内容团队配置，运行时只读。
struct ItemDef {
    /// 物品唯一标识（前缀: `itm_`）
    id: ItemDefId,

    /// 物品名称本地化 Key
    name_key: LocalizationKey,

    /// 物品描述本地化 Key
    desc_key: LocalizationKey,

    /// 物品图标 Key
    icon_key: Option<String>,

    /// 物品类型
    item_type: ItemType,

    /// 物品稀有度
    rarity: Rarity,

    /// 基础价格（金币）
    base_price: u32,

    /// 重量（磅）
    weight: f32,

    /// 最大堆叠数（1 = 不可堆叠）
    max_stack: u32,

    /// 是否为唯一装备（Legendary/剧情物品）
    is_unique: bool,

    /// 装备条件（等级/属性/职业/阵营 — 使用 ConditionDefId）
    equip_conditions: Vec<ConditionDefId>,

    /// 装备时提供的 Modifier 列表
    equipped_modifiers: Vec<ModifierDefId>,

    /// 使用时产生的 Effect 列表（消耗品）
    use_effects: Vec<EffectDefId>,
}

enum ItemType {
    Weapon(WeaponCategory),
    Armor(ArmorCategory),
    Accessory(AccessorySlot),
    Consumable,
    QuestItem,
    Material,
    Currency,
}

enum WeaponCategory {
    SimpleMelee, SimpleRanged,
    MartialMelee, MartialRanged,
    // 双手/轻型/灵巧/重型等属性通过 Tag 表达
}

enum ArmorCategory { Light, Medium, Heavy, Shield }

enum AccessorySlot { Ring, Amulet, Cloak, Gloves, Boots, Helmet }

enum Rarity { Common, Uncommon, Rare, VeryRare, Legendary }
```

### 1.2 ItemInstance（Instance 层/Persistence 层）

```rust
/// 物品实例——背包中物品的具体存在。
/// 每个不同的物品（含附魔/耐久/自定义属性）有独立实例。
struct ItemInstance {
    /// 实例唯一标识
    instance_id: ItemInstanceId,

    /// 引用的物品模板
    template_id: ItemDefId,

    /// 当前数量（可堆叠物品）
    quantity: u32,

    /// 耐久度（如果有耐久机制）
    durability: Option<DurabilityState>,

    /// 自定义附魔（如果有，引用 ModifierDefId 列表）
    enchants: Vec<ModifierDefId>,
}

struct DurabilityState {
    current: u32,
    max: u32,
    is_broken: bool,
}
```

### 1.3 Inventory（Instance 层/Persistence 层）

```rust
/// 背包容器。管理角色持有的所有物品。
struct Inventory {
    /// 所有物品实例
    items: Vec<ItemInstance>,

    /// 最大物品格数（默认值可通过升级/特性扩展）
    max_slots: u32,

    /// 当前总重量
    total_weight: f32,

    /// 最大负重（力量 × 15 磅）
    max_weight: f32,
}
```

### 1.4 EquipmentSlots（Instance 层/Persistence 层）

```rust
/// 装备槽位。每个槽位最多一件装备。
struct EquipmentSlots {
    main_hand: Option<ItemInstanceId>,
    off_hand: Option<ItemInstanceId>,
    helmet: Option<ItemInstanceId>,
    armor: Option<ItemInstanceId>,
    gloves: Option<ItemInstanceId>,
    boots: Option<ItemInstanceId>,
    cloak: Option<ItemInstanceId>,
    ring_1: Option<ItemInstanceId>,
    ring_2: Option<ItemInstanceId>,
    amulet: Option<ItemInstanceId>,
    special: Option<ItemInstanceId>,
}
```

### 1.5 LootTableDef（Definition 层）

```rust
/// 战利品表定义。描述掉落物品的概率和数量。
struct LootTableDef {
    /// 战利品表唯一标识（前缀: `oot_`）
    id: LootTableId,

    /// 掉落条目列表
    entries: Vec<LootEntry>,

    /// 最小掉落数
    min_drops: u32,

    /// 最大掉落数
    max_drops: u32,

    /// 是否每次必掉（所有条目独立概率）
    is_guaranteed: bool,
}

struct LootEntry {
    /// 物品模板 ID
    item_id: ItemDefId,

    /// 掉落概率（0.0 - 1.0）
    probability: f32,

    /// 最小掉落数量
    min_quantity: u32,

    /// 最大掉落数量
    max_quantity: u32,

    /// 掉落条件（如特定阵营/等级才掉落）
    condition: Option<ConditionDefId>,
}
```

### 1.6 InventoryState（Persistence 层）

```rust
/// 背包系统的持久化状态。
struct InventoryState {
    /// 所有物品实例
    items: Vec<ItemInstance>,

    /// 装备槽位状态
    equipment: EquipmentSlots,
}
```

---

## 2. Layer Summary

| Layer | Structures | 说明 |
|-------|-----------|------|
| **Definition** | `ItemDef`, `LootTableDef` | 物品模板和战利品表为静态配置 |
| **Spec** | — | Inventory 无 Spec 层；物品使用/装备逻辑由 Effect/Modifier 管线表达 |
| **Instance** | `ItemInstance`, `Inventory`, `EquipmentSlots` | 背包运行时状态 |
| **Persistence** | `InventoryState` | 物品实例和装备状态完整持久化 |

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → ModifierSchema | 装备提供 ModifierDefId 列表 |
| → EffectSchema | 消耗品使用引用 EffectDefId |
| → EventSchema | 物品事件发布（ItemAcquired, EquipmentChanged 等） |
| → ConditionSchema | 装备穿戴条件引用 ConditionDefId |
| ← CraftingSchema | 制造产出物品进入 Inventory |
| ← EconomySchema | 商店买卖操作 Inventory |
| ← LootTableDef | 战斗/开箱生成物品 |

---

## 4. Replay & Save

### Replay

- 标记 `replay-safe: false` — Inventory 是玩家进程数据，不参与战斗回放
- 消耗品使用在回放中通过 Effect 录制（消耗品的 Result 由 Effect 回放保证）

### Save

- `InventoryState` 完整持久化（所有物品实例 + 装备状态）
- ItemDef/LootTableDef 从配置加载，不存入存档
- 物品实例数量变化（使用/获得/丢弃）全部记录为 delta

---

## 5. Validation Rules

| 规则 | 说明 | 违反处理 |
|------|------|----------|
| 槽位独占性 | 每个 EquipmentSlot 最多一件装备 | 替换旧装备 |
| 双手武器占双槽 | 双手武器占 MainHand + OffHand | 装备时检查 |
| 装备条件检查 | 装备前检查等级/属性/职业/阵营 | 穿戴失败 |
| 堆叠上限 | quantity <= max_stack | 超过时另开新格 |
| 负重限制 | total_weight <= max_weight | 超重时阻止获取新物品 |
| 唯一装备限制 | is_unique=true 的物品不可重复拥有 | 拾取/购买时检查 |

---

## 6. Constitution Check

- ✅ **Data Law 001 (Def-Instance分离)**: ItemDef 为纯 Definition，ItemInstance/Inventory 为 Instance
- ✅ **Data Law 002 (Rule-Content分离)**: 装备规则（槽位/双手武器/负重）为代码规则
- ✅ **Data Law 003 (配置只引用ID)**: ItemDef 引用 ModifierDefId/EffectDefId/ConditionDefId
- ✅ **Data Law 005 (Effect是唯一业务执行入口)**: 消耗品效果通过 Effect 执行
- ✅ **Data Law 006 (Modifier不拥有业务逻辑)**: 装备 Modifier 仅改变数值，不包含穿戴/卸下逻辑
- ✅ **Data Law 011 (Schema版本化)**: InventoryState 携带版本号
- ✅ **Data Law 012 (域间禁止直接数据引用)**: Inventory 通过 Event 对外通信
