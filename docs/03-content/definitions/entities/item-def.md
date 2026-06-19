---
id: 03-content.definitions.entities.item-def
title: ItemDef — Item Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# ItemDef — Item Content Def 定义

> **Content Layer**: L2 Entity | **领域规则**: `docs/02-domain/domains/inventory_domain.md` | **数据 Schema**: `docs/04-data/domains/inventory_schema.md` | **插件代码**: `src/content/plugins/item_plugin.rs`

---

## 1. Overview

ItemDef 定义了游戏中最基础的物品模板——仅包含物品属性本身，不涉及使用效果、装备属性。ItemDef 适用于：

- **材料物品**：矿石、草药、木材（无使用效果）
- **任务物品**：钥匙、信件、收藏品（不可使用、不可装备）
- **货币/代币**：金币、荣誉徽记、兑换券
- **基础交易品**：可以直接出售的低级物品

物品的两种特殊变体——装备（EquipmentDef）和消耗品（ConsumableDef）——各自有独立的 Def 类型，不与 ItemDef 共享记录。

### 关键设计原则

- **ItemBase 模式**：ItemDef、EquipmentDef、ConsumableDef 共享 ItemBase 字段集合。ItemDef 是 ItemBase 的"直译"（不加额外字段），EquipmentDef 和 ConsumableDef 在其上增加特有字段
- **ItemDef 不引用 EquipmentDef/ConsumableDef**：基础物品、装备、消耗品是**互斥分类**。一个物品不可能同时是三个类型中的两个。材料就是 ItemDef，装备就是 EquipmentDef，药水就是 ConsumableDef
- **ItemDef 不是"父类"**：EquipmentDef 和 ConsumableDef 不继承 ItemDef。它们在 Schema 层面共享 ItemBase 字段，但在 Def 层面各自独立——各自有自己的 ID 类型、Registry、资产文件

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `inventory_domain.md` | 物品分类、堆叠规则、背包管理 |
| `inventory_schema.md` | ItemDef 的完整字段结构、ItemType、Rarity 定义 |
| `tag-def.md` | 本 Def 的 `tags` 引用的 TagDef |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// ItemBase — 物品类 Def 的共享字段基座。
///
/// ItemDef、EquipmentDef 和 ConsumableDef 各自内联嵌入这些字段。
/// 这不是独立 Def，而是 Schema 层面的复用模式。
struct ItemBase {
    // ── 统一标识字段 ──
    pub id: ItemId,
    pub name_key: LocalizationKey,
    pub description_key: LocalizationKey,
    pub schema_version: u32,

    // ── 基础属性 ──
    /// 物品类型（通用/材料/任务物品）
    pub item_type: ItemType,

    /// 物品稀有度（常见/非凡/稀有/史诗/传说）
    pub rarity: Rarity,

    /// 基础价格（金币）
    pub base_price: u32,

    /// 重量（磅）
    pub weight: f32,

    /// 最大堆叠数（1 = 不可堆叠）
    pub max_stack: u32,

    /// 图标 Key
    pub icon_key: Option<String>,

    /// 模型 Key（3D 模型或 2D 图片的资源路径）
    pub model_key: Option<String>,

    /// 标签列表（引用 L0 TagDef，物品分类标记）
    pub tags: Vec<TagId>,
}

/// 基础物品模板定义——无使用效果、无装备属性的纯物品。
///
/// ItemDef 是 Content Asset，经 Load → Deserialize → Validate → Register → Freeze
/// 管线后进入 DefRegistry<ItemDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ItemDef {
    // ── 嵌入 ItemBase ──
    /// 全局唯一 ID（ItemDef 前缀: `itm_`）
    pub id: ItemId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    /// 物品类型
    pub item_type: ItemType,
    /// 物品稀有度
    pub rarity: Rarity,
    /// 基础价格（金币）
    pub base_price: u32,
    /// 重量（磅）
    pub weight: f32,
    /// 最大堆叠数（1 = 不可堆叠）
    pub max_stack: u32,
    /// 图标 Key
    pub icon_key: Option<String>,
    /// 模型 Key
    pub model_key: Option<String>,
    /// 标签列表
    pub tags: Vec<TagId>,

    // ── ItemDef 特有字段 ──
    /// 是否可出售
    pub sellable: bool,
    /// 是否可丢弃
    pub droppable: bool,
}
```

### 内嵌数据结构

```rust
/// 物品类型
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ItemType {
    /// 通用物品（如宝石、收藏品）
    Generic,
    /// 材料（锻造/炼金/制造用）
    Material,
    /// 任务物品
    Quest,
    /// 货币/代币
    Currency,
    /// 钥匙（开门/开箱）
    Key,
}

/// 物品稀有度
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    VeryRare,
    Legendary,
}
```

### 字段说明

- **`sellable` / `droppable`**: 控制物品在商店中的可售性和在地面上的可丢弃性。任务物品通常为 `sellable: false, droppable: false`
- **`max_stack`**: 物品在背包中的最大堆叠数。1 表示不可堆叠（如装备、钥匙）。材料类通常可堆叠（如矿石 max_stack: 99）
- **`item_type`**: ItemDef 的 `item_type` 不包含 Equipment 或 Consumable 变体——这两者由各自的 Def 类型覆盖

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// ItemDef 注册插件
pub struct ItemDefPlugin;

impl Plugin for ItemDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<ItemDef>();
        app.init_asset_loader::<RonAssetLoader<ItemDef>>();
        app.insert_resource(DefRegistry::<ItemDef>::new());
        app.add_systems(
            PreUpdate,
            load_item_defs
                .run_if(resource_changed::<Assets<ItemDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 ItemDef
pub fn get_item_def(
    item_id: &ItemId,
    registry: &DefRegistry<ItemDef>,
) -> Option<&ItemDef> {
    registry.get(item_id)
}

/// 按稀有度过滤 ItemDef
pub fn get_item_defs_by_rarity(
    rarity: Rarity,
    registry: &DefRegistry<ItemDef>,
) -> Vec<&ItemDef> {
    registry.iter()
        .filter(|def| def.rarity == rarity)
        .collect()
}
```

### 注册生命周期

```
ItemDefPlugin::build
  │
  ├── ItemDef 从 assets/config/02_entities/items.ron 加载
  │
  ├── Deserialize (ron::from_str)
  │     └── 校验: RON 语法正确性、枚举合法性
  │
  ├── Validate
  │     ├── ID 唯一性
  │     ├── 字段范围检查 (base_price 范围, weight 范围, max_stack 范围)
  │     ├── tag 存在性检查
  │     └── Rarity/ItemType 枚举合法性
  │
  ├── Register（注入 DefRegistry<ItemDef>）
  │
  └── Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | ItemId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `base_price` 范围 | 0-999999（整数，无负数） |
| V4 | `weight` 范围 | 0.0-1000.0（磅，0 = 无重量如货币） |
| V5 | `max_stack` >= 1 | 堆叠数至少为 1 |
| V6 | `rarity` 合法 | 必须匹配 Rarity 的已知变体 |
| V7 | `item_type` 合法 | 必须匹配 ItemType 的已知变体 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V8 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |

### 4.3 语义校验

| # | 规则 | 说明 |
|---|------|------|
| V9 | 任务物品不可出售 | `item_type: Quest` ⇒ `sellable: false` |
| V10 | 任务物品不可丢弃 | `item_type: Quest` ⇒ `droppable: false` |
| V11 | 货币物品 weight 应为 0 | `item_type: Currency` 时 weight 应强制为 0.0 |

---

## 5. RON 示例

```ron
(
    id: "itm:iron_ore",
    name_key: "item.itm_iron_ore.name",
    description_key: "item.itm_iron_ore.desc",
    schema_version: 1,

    item_type: Material,
    rarity: Common,

    base_price: 5,
    weight: 1.0,
    max_stack: 99,

    icon_key: Some("icons/items/iron_ore.png"),
    model_key: None,

    tags: ["tag:ore", "tag:smithing_material", "tag:crafting"],

    sellable: true,
    droppable: true,
)
```

```ron
(
    id: "itm:ancient_key",
    name_key: "item.itm_ancient_key.name",
    description_key: "item.itm_ancient_key.desc",
    schema_version: 1,

    item_type: Quest,
    rarity: Uncommon,

    base_price: 0,
    weight: 0.1,
    max_stack: 1,

    icon_key: Some("icons/items/ancient_key.png"),
    model_key: None,

    tags: ["tag:key", "tag:quest_item"],

    sellable: false,
    droppable: false,
)
```

---

## 6. 与 EquipmentDef / ConsumableDef 的关系

ItemDef、EquipmentDef、ConsumableDef 共享 ItemBase 字段结构，但彼此独立：

| 对比维度 | ItemDef | EquipmentDef | ConsumableDef |
|----------|---------|-------------|---------------|
| ID 前缀 | `itm_` | `equip_` | `con_` |
| Rust 类型 | ItemDef | EquipmentDef | ConsumableDef |
| ID 类型 | ItemId | EquipmentId | ConsumableId |
| Registry | DefRegistry\<ItemDef\> | DefRegistry\<EquipmentDef\> | DefRegistry\<ConsumableDef\> |
| 资产文件 | items.ron | equipment.ron | consumables.ron |
| 使用效果 | 无 | 无（穿戴提供被动） | 有（EffectDef 引用） |
| 装备属性 | 无 | 有（ModifierDef 引用） | 无 |
| 是否可堆叠 | 是（材料类） | 否（max_stack: 1） | 通常否 |
| 典型物品 | 铁矿、钥匙、金币 | 长剑、板甲、戒指 | 治疗药水、火球卷轴 |

---

*本文档由 @content-architect 维护。*
