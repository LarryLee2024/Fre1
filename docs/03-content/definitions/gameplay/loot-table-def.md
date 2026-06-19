---
id: 03-content.definitions.gameplay.loot-table-def
title: LootTableDef — LootTable Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# LootTableDef — LootTable Content Def 定义

> **Content Layer**: L3 Gameplay | **领域规则**: `docs/02-domain/domains/economy_domain.md` + `docs/02-domain/domains/combat_domain.md` | **数据 Schema**: `docs/04-data/domains/economy_schema.md` | **插件代码**: `src/content/plugins/loot_table_plugin.rs`

---

## 1. Overview

LootTableDef 定义了掉落表模板——列出可能掉落的物品及概率权重、数量范围、条件。LootTableDef 是 L3 层中**被 L2 MonsterDef 前向引用**的 Def 类型，是 L3 加载管线中需优先处理的类型之一。

### 关键设计原则

- **权重驱动**：掉落概率基于相对权重计算（weight 值），而非百分比。weight: 100 和 weight: 50 = 后者概率是前者的一半
- **条件过滤**：`conditions` 字段引用 L1 ConditionDef，允许条件性掉落（如"仅当玩家有特定 Buff 时额外掉落"）
- **子表嵌套**：LootEntry 可引用另一个 LootTableDef 作为子表，实现"掉落表 A 有 30% 概率进入掉落表 B"的多层嵌套
- **数量范围**：`quantity_min` / `quantity_max` 定义掉落数量随机范围，最终掉落数量在区间内随机
- **多态引用**：使用 LootItemRef 统一引用 ItemDef/EquipmentDef/ConsumableDef

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `economy_domain.md` | 货币体系、交易规则（掉落物价值参考） |
| `combat_domain.md` | 战斗中掉落时机、战利品结算 |
| `economy_schema.md` | LootTableDef 完整字段结构、LootEntry 定义 |
| `condition-def.md` | 本 Def 的 `entries[].conditions` 引用的 ConditionDef |
| `tag-def.md` | 本 Def 的 `tags` 引用的 TagDef |
| `item-def.md` | 本 Def 的 `entries` 引用的 ItemDef |
| `equipment-def.md` | 本 Def 的 `entries` 引用的 EquipmentDef |
| `consumable-def.md` | 本 Def 的 `entries` 引用的 ConsumableDef |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 掉落表定义——描述一组可能掉落的物品及其概率。
///
/// LootTableDef 是 Content Asset，经 Load → Deserialize → Validate → Register → Freeze
/// 管线后进入 DefRegistry<LootTableDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct LootTableDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID（LootTableDef 前缀: `loot_`）
    pub id: LootTableId,
    /// 显示名称（可选，仅用于调试/策划参考）
    pub name_key: Option<LocalizationKey>,
    /// 描述文本（可选）
    pub description_key: Option<LocalizationKey>,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 掉落条目 ──
    /// 掉落条目列表（基于权重概率选择）
    pub entries: Vec<LootEntry>,

    // ── 掉落数量 ──
    /// 最少掉落物品数量
    pub min_drops: u32,
    /// 最多掉落物品数量（在 entries 中按权重随机选取）
    pub max_drops: u32,

    // ── 掉落类型 ──
    /// 掉落表分类（影响掉落时机和规则）
    pub loot_type: LootType,

    // ── 元数据 ──
    /// 标签列表（引用 L0 TagDef，用于分类过滤）
    pub tags: Vec<TagId>,
}
```

### 内嵌数据结构

```rust
/// 掉落条目——每个可能的掉落物品及其概率权重
#[derive(Deserialize, Clone, Debug)]
pub struct LootEntry {
    /// 掉落物品引用（多态）
    pub item: LootItemRef,
    /// 相对权重（权重越高，掉落概率越大）
    pub weight: u32,
    /// 最小掉落数量
    pub quantity_min: u32,
    /// 最大掉落数量
    pub quantity_max: u32,
    /// 掉落条件（可选，满足时才参与权重计算）
    pub conditions: Option<Vec<ConditionId>>,
    /// 是否保证掉落（忽略权重，必定掉落）
    pub guaranteed: bool,
}

/// 多态掉落物品引用
#[derive(Deserialize, Clone, Debug)]
pub enum LootItemRef {
    /// 引用基础物品（带数量范围，如 1-5 个龙鳞）
    Item(ItemId),
    /// 引用装备
    Equipment(EquipmentId),
    /// 引用消耗品
    Consumable(ConsumableId),
    /// 金币掉落（(最小, 最大)）
    Currency(u32, u32),
    /// 子掉落表（同一层嵌套，允许递归引用）
    LootTable(LootTableId),
}

/// 掉落类型枚举
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum LootType {
    /// 标准战斗掉落（击杀后）
    Standard,
    /// Boss 掉落（保证高稀有度）
    Boss,
    /// 宝箱/容器掉落（非战斗）
    Container,
    /// 任务指定掉落（Quest 领域驱动）
    Quest,
    /// 采集掉落（草药/采矿/钓鱼）
    Gathering,
}
```

### 字段说明

- **`weight`**：相对权重，非百分比。如条目 A weight: 100, 条目 B weight: 50，则 A 被选中的概率是 B 的 2 倍。实际概率 = weight / 总权重
- **`guaranteed`**: 设为 true 的条目必定掉落，不参与权重概率计算。适用于"每个怪物必掉金币"的场景
- **`min_drops` / `max_drops`**: 最终掉落数量在此范围内随机。如果 entries 中 guaranteed 条目数量超过 max_drops，以 guaranteed 为准
- **`LootItemRef::LootTable`**: 子表引用——实现多层嵌套（如"10% 概率进入稀有的龙之宝藏表，否则进入普通宝藏表"）。需避免循环子表引用
- **`conditions`**: 条件性掉落——如"仅当怪物死于火焰伤害时才掉落灰烬"。所有条件满足时条目才参与权重计算

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// LootTableDef 注册插件
pub struct LootTableDefPlugin;

impl Plugin for LootTableDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<LootTableDef>();
        app.init_asset_loader::<RonAssetLoader<LootTableDef>>();
        app.insert_resource(DefRegistry::<LootTableDef>::new());
        app.add_systems(
            PreUpdate,
            load_loot_table_defs
                .run_if(resource_changed::<Assets<LootTableDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按掉落类型过滤 LootTableDef
pub fn get_loot_tables_by_type(
    loot_type: LootType,
    registry: &DefRegistry<LootTableDef>,
) -> Vec<&LootTableDef> {
    registry.iter()
        .filter(|def| def.loot_type == loot_type)
        .collect()
}
```

### 注册生命周期

```
LootTableDefPlugin::build
  │
  ├── LootTableDef 从 assets/config/03_gameplay/loot_tables.ron 加载
  │
  ├── Deserialize → Validate → Register → Freeze
  │
  └── Validate 具体规则：
        ├── ID 唯一性
        ├── L0 (TagId) 引用存在性
        ├── L1 (ConditionId) 引用存在性
        ├── L2 (ItemId/EquipmentId/ConsumableId) 引用存在性
        ├── L3 同层引用（LootTableId 子表）存在性 + 循环检测
        ├── entries 非空
        ├── min_drops <= max_drops
        ├── quantity_min <= quantity_max
        ├── weight >= 1（非 guaranteed 条目）
        ├── L4 禁止引用检查
        └── 子表嵌套深度限制（不超过 5 层）
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | LootTableId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `entries` 非空 | 掉落表必须有至少一个掉落条目 |
| V4 | `min_drops` <= `max_drops` | 最少掉落数不能超过最多掉落数 |
| V5 | `quantity_min` <= `quantity_max` | 每个条目的数量范围必须有效 |
| V6 | `weight` >= 1（非 guaranteed） | 权重至少为 1 |
| V7 | `loot_type` 合法 | 必须匹配 LootType 的已知变体 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V8 | `entries` 中的每个 ItemId（Item 变体）已注册 | 在 DefRegistry<ItemDef> 中存在 |
| V9 | `entries` 中的每个 EquipmentId 已注册 | 在 DefRegistry<EquipmentDef> 中存在 |
| V10 | `entries` 中的每个 ConsumableId 已注册 | 在 DefRegistry<ConsumableDef> 中存在 |
| V11 | `entries` 中的每个 ConditionId（如果设置）已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V12 | `entries` 中的每个 LootTableId（子表，如果设置）已注册 | 在 DefRegistry<LootTableDef> 中存在 |
| V13 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |

### 4.3 循环引用检测

| # | 规则 | 说明 |
|---|------|------|
| V14 | 子表引用不得形成 A→B→A 循环 | 遍历 LootTableId 子表引用图，检测循环 |
| V15 | 子表嵌套深度不超过 5 层 | 防止运行时栈溢出 |

### 4.4 层间依赖校验

| # | 规则 | 说明 |
|---|------|------|
| V16 | LootTableDef 不得引用任何 L4 World Def | 层间依赖方向规则 |

### 4.5 Forward Reference 校验（接收来自 L2 的引用）

| # | 规则 | 说明 |
|---|------|------|
| V17 | L3 加载完成后，二次校验所有 MonsterDef 的 `loot_table` 引用 | 验证每个引用的 LootTableId 在 DefRegistry<LootTableDef> 中存在 |

### 4.6 语义校验

| # | 规则 | 说明 |
|---|------|------|
| V18 | guaranteed 条目不应设置 weight | 当 guaranteed = true 时 weight 字段无效 |
| V19 | Currency 类型的 quantity_min/quantity_max 应合理 | 金币掉落不应超出经济系统设定的范围 |
| V20 | LootType 与上下文一致 | Boss 掉落表至少应有 1 个 guaranteed 条目 |

---

## 5. RON 示例

```ron
(
    id: "loot:goblin_raider_drops",
    name_key: Some("loot_table.loot_goblin_raider.name"),
    description_key: None,
    schema_version: 1,

    entries: [
        (
            item: Currency(5, 15),
            weight: 100,
            quantity_min: 1,
            quantity_max: 1,
            conditions: None,
            guaranteed: true,
        ),
        (
            item: Item("itm:goblin_tooth"),
            weight: 60,
            quantity_min: 1,
            quantity_max: 3,
            conditions: None,
            guaranteed: false,
        ),
        (
            item: Equipment("equip:rusty_dagger"),
            weight: 30,
            quantity_min: 1,
            quantity_max: 1,
            conditions: None,
            guaranteed: false,
        ),
        (
            item: Consumable("con:minor_health_potion"),
            weight: 20,
            quantity_min: 1,
            quantity_max: 1,
            conditions: None,
            guaranteed: false,
        ),
    ],

    min_drops: 1,
    max_drops: 3,

    loot_type: Standard,

    tags: ["tag:goblinoid", "tag:low_level", "tag:standard_loot"],
)
```

```ron
(
    id: "loot:dragon_hoard",
    name_key: Some("loot_table.loot_dragon_hoard.name"),
    description_key: None,
    schema_version: 1,

    entries: [
        (
            item: Currency(500, 2000),
            weight: 100,
            quantity_min: 1,
            quantity_max: 1,
            conditions: None,
            guaranteed: true,
        ),
        (
            item: Equipment("equip:dragon_scale_armor"),
            weight: 10,
            quantity_min: 1,
            quantity_max: 1,
            conditions: None,
            guaranteed: false,
        ),
        (
            item: LootTable("loot:dragon_magic_items"),
            weight: 50,
            quantity_min: 1,
            quantity_max: 2,
            conditions: None,
            guaranteed: false,
        ),
        (
            item: Item("itm:dragon_scale"),
            weight: 80,
            quantity_min: 3,
            quantity_max: 8,
            conditions: None,
            guaranteed: true,
        ),
    ],

    min_drops: 2,
    max_drops: 5,

    loot_type: Boss,

    tags: ["tag:dragon", "tag:boss_loot", "tag:high_level"],
)
```

---

## 6. 与其他 L3 Def 的关系

| L3 Def | LootTableDef 的关系 |
|--------|-------------------|
| EncounterDef | EncounterDef 可通过 `loot_bonus` 引用一个额外的 LootTableDef（如 Boss 战额外掉落）。怪物本身的掉落来自 MonsterDef 的 `loot_table` Forward Reference |
| MonsterDef | 见 Forward Reference：MonsterDef 的 `loot_table` 字段指向 LootTableDef——在 L2 加载时仅记录，L3 加载后二次校验 |
| QuestDef | QuestDef 的 `rewards` 是静态字段，不引用 LootTableDef。LootTableDef 更适合动态掉落场景 |

**L2-L3 Forward Reference 解析流程**：

```
Phase 3: Load L3 (Gameplay)
  │
  ├── 1. Load LootTableDef（优先加载——被 L2 前向引用）
  │
  ├── 2. Load other L3 Defs（EncounterDef, QuestDef 等）
  │
  └── 3. 二次校验：遍历所有 MonsterDef，解析 loot_table 引用
        ├── 每个 LootTableId → 在 DefRegistry<LootTableDef> 中查找
        └── 未找到 → ContentError::UnresolvedForwardRef
```

---

*本文档由 @content-architect 维护。*
