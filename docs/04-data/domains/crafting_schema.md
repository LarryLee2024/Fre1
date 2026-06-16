---
id: domains.crafting.schema.v1
title: Crafting Schema — 制作/锻造数据架构
status: draft
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition, instance, persistence
replay-safe: false
---

# Crafting Schema — 制作/锻造数据架构

> **领域归属**: Domains — 经济系统层 | **依赖 Schema**: Inventory, Modifier, Effect, Event | **定义依据**: `docs/02-domain/crafting_domain.md`

---

## 1. Schema Design

### 1.1 RecipeDef（Definition 层）

```rust
/// 配方定义。内容团队配置，运行时只读。
struct RecipeDef {
    /// 配方唯一标识（前缀: `rcp_`）
    id: RecipeDefId,

    /// 配方名称本地化 Key
    name_key: LocalizationKey,

    /// 所需制作台类型
    station: CraftingStation,

    /// 技能要求
    skill_requirement: Option<SkillRequirement>,

    /// 所需材料列表
    materials: Vec<MaterialCost>,

    /// 产出物品
    output: CraftOutput,

    /// 制作时间（回合数/秒）
    craft_time: u32,

    /// 制作类型
    craft_type: CraftType,
}

enum CraftingStation { Forge, EnchantingTable, AlchemyLab, TailoringBench, EngineeringBench }

struct SkillRequirement {
    skill_id: String,
    dc: u32,              // 技能检定 DC
}

struct MaterialCost {
    item_id: ItemDefId,
    quantity: u32,
}

struct CraftOutput {
    item_id: ItemDefId,
    quantity: u32,
    enchantment_slots: u32,   // 预留附魔槽位数
}

enum CraftType { Smithing, Enchanting, Alchemy, Tailoring, Engineering }
```

### 1.2 EnchantmentDef（Definition 层）

```rust
/// 附魔定义。
struct EnchantmentDef {
    /// 附魔唯一标识（前缀: `enc_`）
    id: EnchantmentDefId,

    /// 附魔名称本地化 Key
    name_key: LocalizationKey,

    /// 附魔的 Modifier 效果
    modifier_id: ModifierDefId,

    /// 互斥类型（同类型词条不可共存）
    exclusive_group: Option<String>,

    /// 占用槽位类型
    slot_type: EnchantmentSlotType,
}

enum EnchantmentSlotType {
    Weapon { max_slots: u32 },
    Armor { max_slots: u32 },
    Accessory { max_slots: u32 },
}
```

### 1.3 EnchantmentSlot（Instance 层）

```rust
/// 装备的附魔槽位运行时状态。
struct EnchantmentSlot {
    /// 该装备允许的最大附魔数
    max_slots: u32,

    /// 当前已附魔的词条
    active_enchants: Vec<EnchantmentDefId>,
}
```

### 1.4 UpgradeLevel（Instance 层）

```rust
/// 装备升级等级。
struct UpgradeLevel {
    /// 当前升级等级（0 = 未升级）
    current: u32,

    /// 最大升级等级（取决于稀有度）
    max: u32,

    /// 升级提供的 Modifier
    level_modifiers: HashMap<u32, Vec<ModifierDefId>>,
}
```

### 1.5 CraftingState（Persistence 层）

```rust
/// 制作系统的持久化状态。
struct CraftingState {
    /// 已解锁的配方列表
    unlocked_recipes: Vec<RecipeDefId>,

    /// 制作中的进度（制作完成前存档退出需要保存）
    in_progress_crafts: Vec<CraftInProgress>,
}

struct CraftInProgress {
    recipe_id: RecipeDefId,
    remaining_time: u32,
    station: CraftingStation,
}
```

---

## 2. Layer Summary

| Layer | Structures | 说明 |
|-------|-----------|------|
| **Definition** | `RecipeDef`, `EnchantmentDef` | 配方和附魔的静态配置 |
| **Spec** | — | Crafting 无 Spec 层；制作技能检定为代码逻辑 |
| **Instance** | `EnchantmentSlot`, `UpgradeLevel` | 装备的附魔/升级运行时状态 |
| **Persistence** | `CraftingState` | 已解锁配方和进行中的制作进度 |

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → InventorySchema | 制作消耗/产出物品 |
| → ModifierSchema | 附魔和升级的效果引用 ModifierDefId |
| → EffectSchema | 炼金产出（药水）的使用效果 |
| → EventSchema | 制作事件发布（ItemCrafted, EnchantmentApplied 等） |
| ← EconomySchema | 制作材料可商店购买 |

---

## 4. Replay & Save

### Replay

- 标记 `replay-safe: false` — 制作是玩家进程数据

### Save

- `CraftingState` 持久化（已解锁配方 + 制作中的进度）
- RecipeDef/EnchantmentDef 从配置加载
- 制作中的 CraftInProgress 在存档时保存剩余时间，读档后继续计时

---

## 5. Validation Rules

| 规则 | 说明 | 违反处理 |
|------|------|----------|
| 材料充足 | 制作前检查背包材料数量 >= 配方需求 | 制作无法开始 |
| 制作台匹配 | 制作必须在配方指定的制作台上进行 | 制作无法开始 |
| 附魔槽位上限 | 已附魔数 <= max_slots | 附魔失败 |
| 升级等级上限 | current < max | 升级失败 |
| 互斥词条防冲突 | 同 exclusive_group 的附魔替换旧词条 | 自动替换 |

---

## 6. Constitution Check

- ✅ **Data Law 001 (Def-Instance分离)**: RecipeDef/EnchantmentDef 为 Definition，EnchantmentSlot/UpgradeLevel 为 Instance
- ✅ **Data Law 003 (配置只引用ID)**: RecipeDef 引用 ItemDefId，EnchantmentDef 引用 ModifierDefId
- ✅ **Data Law 005 (Effect是唯一业务执行入口)**: 炼金产出通过 Effect 执行
- ✅ **Data Law 006 (Modifier不拥有业务逻辑)**: 附魔/升级的数值加成通过 Modifier 管线
- ✅ **Data Law 011 (Schema版本化)**: CraftingState 携带版本号
- ✅ **Data Law 012 (域间禁止直接数据引用)**: Crafting 通过 Event 与 Inventory 通信
