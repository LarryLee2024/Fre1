---
id: 03-content.definitions.gameplay.shop-def
title: ShopDef — Shop Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# ShopDef — Shop Content Def 定义

> **Content Layer**: L3 Gameplay | **领域规则**: `docs/02-domain/domains/economy_domain.md` | **数据 Schema**: `docs/04-data/domains/economy_schema.md` | **插件代码**: `src/content/plugins/shop_plugin.rs`

---

## 1. Overview

ShopDef 定义了商店模板——商品的供应列表、价格系数、声望折扣、刷新规则和访问条件。ShopDef 是"商店配置"而非"商店实例"：每个 NPC 的商店实例由 Economy 领域的运行时状态管理。

### 关键设计原则

- **供需分离**：商店的商品列表（what sells）和价格策略（how much）在 ShopDef 中定义，玩家的购买/出售行为属于运行时交易
- **声望集成**：`faction_discounts` 字段引用 L0 FactionDef，通过声望等级自动调整价格，不将声望逻辑硬编码到商店配置中
- **多态商品引用**：ShopItemRef 枚举统一引用 ItemDef/EquipmentDef/ConsumableDef，无需为每种类型创建独立字段组
- **条件访问**：`unlock_conditions` 引用 L1 ConditionDef，使商店解锁条件可复用（如"完成任务后方可访问"、"特定阵营尊敬"）

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `economy_domain.md` | 货币体系、价格计算、交易流程 |
| `economy_schema.md` | ShopDef 完整字段结构、ShopItem 定义 |
| `tag-def.md` | 本 Def 的 `tags` 引用的 TagDef |
| `faction-def.md` | 本 Def 的 `faction_discounts` 和 `items[].faction_requirement` 引用的 FactionDef |
| `condition-def.md` | 本 Def 的 `unlock_conditions` 和 `items[].condition` 引用的 ConditionDef |
| `item-def.md` | 本 Def 的 `inventory` 引用的 ItemDef |
| `equipment-def.md` | 本 Def 的 `inventory` 引用的 EquipmentDef |
| `consumable-def.md` | 本 Def 的 `inventory` 引用的 ConsumableDef |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 商店配置定义——描述一个商店的商品、价格和访问规则。
///
/// ShopDef 是 Content Asset，经 Load → Deserialize → Validate → Register → Freeze
/// 管线后进入 DefRegistry<ShopDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ShopDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID（ShopDef 前缀: `shp_`）
    pub id: ShopId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 商店类型 ──
    /// 商店类型（决定 UI、交易规则和分类）
    pub shop_type: ShopType,

    // ── 商品列表 ──
    /// 商店商品清单
    pub inventory: Vec<ShopItem>,

    // ── 价格系数 ──
    /// 向玩家出售的价格倍率（1.0 = 标准价）
    pub buy_price_modifier: f32,
    /// 从玩家收购的价格倍率（0.5 = 半价收购）
    pub sell_price_modifier: f32,

    // ── 声望折扣 ──
    /// 基于阵营声望的价格折扣
    pub faction_discounts: Option<Vec<FactionDiscount>>,

    // ── 补货规则 ──
    /// 商品刷新规则（何时补充已售罄商品）
    pub refresh_rules: Option<ShopRefreshRules>,

    // ── 访问条件 ──
    /// 访问该商店需要满足的条件（引用 L1 ConditionDef）
    pub unlock_conditions: Option<Vec<ConditionId>>,

    // ── 元数据 ──
    /// 标签列表（引用 L0 TagDef，用于分类过滤）
    pub tags: Vec<TagId>,
    /// 商店图标 Key
    pub icon_key: Option<String>,
}
```

### 内嵌数据结构

```rust
/// 商店类型枚举
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ShopType {
    /// 杂货店——出售各类杂货
    General,
    /// 铁匠铺——武器与护甲
    Blacksmith,
    /// 炼金店——药水与卷轴
    Alchemist,
    /// 魔法商店——魔法物品与法术卷轴
    MagicVendor,
    /// 酒馆——食物、饮料、情报
    Tavern,
    /// 阵营商店——需要阵营声望解锁
    FactionStore,
    /// 黑市——赃物交易、违禁品
    BlackMarket,
    /// 自定义（Mod 扩展用）
    Custom(String),
}

/// 商店商品条目
#[derive(Deserialize, Clone, Debug)]
pub struct ShopItem {
    /// 商品引用（多态：ItemDef/EquipmentDef/ConsumableDef）
    pub item: ShopItemRef,
    /// 库存数量（0 = 无限供应）
    pub quantity: u32,
    /// 价格覆盖（可选，覆盖默认价格计算）
    pub price_override: Option<u32>,
    /// 最低等级要求（可选，低于此等级不可购买）
    pub min_level: Option<u32>,
    /// 阵营声望要求（可选，(FactionId, 最低声望值)）
    pub faction_requirement: Option<(FactionId, i32)>,
    /// 购买条件（引用 L1 ConditionDef，可选）
    pub condition: Option<ConditionId>,
}

/// 多态商品引用
#[derive(Deserialize, Clone, Debug)]
pub enum ShopItemRef {
    /// 引用基础物品
    Item(ItemId),
    /// 引用装备
    Equipment(EquipmentId),
    /// 引用消耗品
    Consumable(ConsumableId),
}

/// 阵营声望折扣配置
#[derive(Deserialize, Clone, Debug)]
pub struct FactionDiscount {
    /// 阵营 ID（引用 L0 FactionDef）
    pub faction_id: FactionId,
    /// 触发此折扣所需的最低声望值
    pub reputation_threshold: i32,
    /// 折扣百分比（0.0-1.0，如 0.1 = 10% 折扣）
    pub discount_percent: f32,
}

/// 商品刷新规则
#[derive(Deserialize, Clone, Debug)]
pub struct ShopRefreshRules {
    /// 刷新类型
    pub refresh_type: RefreshType,
    /// 刷新值（与 refresh_type 配合使用）
    pub refresh_value: u32,
}

/// 刷新类型枚举
#[derive(Deserialize, Clone, Debug)]
pub enum RefreshType {
    /// 永不刷新（售罄即止）
    Never,
    /// 每天刷新（游戏内天数）
    Daily,
    /// 每次营地休息后刷新
    OnRest,
    /// 每个任务完成后刷新
    QuestCompletion,
    /// 每个战斗遭遇后刷新
    AfterEncounter,
}
```

### 字段说明

- **`buy_price_modifier` / `sell_price_modifier`**: 基础价格修正。如杂货店买入 1.0（标准价），卖出 0.5（半价）。黑市买入可能 2.0（溢价），卖出 0.3（压价）
- **`quantity: 0`**: 库存数量为 0 表示无限供应（如"永远出售治疗药水，不会断货"）
- **`faction_discounts`**: 应用最高可用折扣（不叠加）。如玩家在 Kingdom 阵营声望崇敬（20%折扣）与 HuntersGuild 友好（10%），取最高 20%
- **`refresh_rules`**: 控制商品补货。`Daily` + `refresh_value: 1` = 每天补货一次。`QuestCompletion` + `refresh_value: 3` = 每完成 3 个任务补货一次

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// ShopDef 注册插件
pub struct ShopDefPlugin;

impl Plugin for ShopDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<ShopDef>();
        app.init_asset_loader::<RonAssetLoader<ShopDef>>();
        app.insert_resource(DefRegistry::<ShopDef>::new());
        app.add_systems(
            PreUpdate,
            load_shop_defs
                .run_if(resource_changed::<Assets<ShopDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按商店类型过滤 ShopDef
pub fn get_shops_by_type(
    shop_type: ShopType,
    registry: &DefRegistry<ShopDef>,
) -> Vec<&ShopDef> {
    registry.iter()
        .filter(|def| def.shop_type == shop_type)
        .collect()
}

/// 按阵营查找商店（检查 faction_discounts 或 faction_requirement）
pub fn get_shops_by_faction(
    faction_id: &FactionId,
    registry: &DefRegistry<ShopDef>,
) -> Vec<&ShopDef> {
    registry.iter()
        .filter(|def| {
            def.faction_discounts
                .as_ref()
                .map_or(false, |discounts| {
                    discounts.iter().any(|d| &d.faction_id == faction_id)
                })
                || def.inventory.iter().any(|item| {
                    item.faction_requirement
                        .as_ref()
                        .map_or(false, |(fid, _)| fid == faction_id)
                })
        })
        .collect()
}
```

### 注册生命周期

```
ShopDefPlugin::build
  │
  ├── ShopDef 从 assets/config/03_gameplay/shops.ron 加载
  │
  ├── Deserialize → Validate → Register → Freeze
  │
  └── Validate 具体规则：
        ├── ID 唯一性
        ├── L0 (TagId, FactionId) 引用存在性
        ├── L1 (ConditionId) 引用存在性
        ├── L2 (ItemId/EquipmentId/ConsumableId) 引用存在性
        ├── inventory 非空（至少 1 个商品）
        ├── buy_price_modifier 范围（0.1-10.0）
        ├── sell_price_modifier 范围（0.1-10.0）
        ├── discount_percent 范围（0.0-1.0）
        └── L4 禁止引用检查
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | ShopId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `inventory` 非空 | 商店必须有至少一个商品 |
| V4 | `buy_price_modifier` 范围 | 0.1-10.0（默认 1.0） |
| V5 | `sell_price_modifier` 范围 | 0.1-10.0（默认 0.5） |
| V6 | `discount_percent` 范围 | 0.0-1.0（每个 FactionDiscount） |
| V7 | `shop_type` 合法 | 必须匹配 ShopType 的已知变体 |
| V8 | `inventory[].quantity` 合法 | 无负数 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V9 | `faction_discounts` 中的每个 FactionId 已注册 | 在 DefRegistry<FactionDef> 中存在 |
| V10 | `inventory` 中的每个 ConditionId（如果设置）已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V11 | `inventory` 中的每个 ShopItemRef 已注册（ItemId/EquipmentId/ConsumableId） | 在对应 DefRegistry 中存在 |
| V12 | `inventory` 中的每个 FactionId（如果设置）已注册 | 在 DefRegistry<FactionDef> 中存在 |
| V13 | `unlock_conditions` 中的每个 ConditionId（如果设置）已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V14 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |

### 4.3 层间依赖校验

| # | 规则 | 说明 |
|---|------|------|
| V15 | ShopDef 不得引用任何 L4 World Def | 层间依赖方向规则 |

### 4.4 语义校验

| # | 规则 | 说明 |
|---|------|------|
| V16 | 价格覆盖合理性 | `price_override` > 0 且不超过商品基础价格的 10 倍 |
| V17 | 声望折扣不冲突 | 同一 Faction 不应有多个重叠的 reputation_threshold 区间 |
| V18 | 商品重复警告 | 同一商品不应在 inventory 中出现多次 |
| V19 | `BlackMarket` 类型应设置 `sell_price_modifier` 较低 | 黑市压价收购是预期行为 |

---

## 5. RON 示例

```ron
(
    id: "shp:blacksmith_ironforge",
    name_key: "shop.shp_blacksmith_ironforge.name",
    description_key: "shop.shp_blacksmith_ironforge.desc",
    schema_version: 1,

    shop_type: Blacksmith,

    inventory: [
        (
            item: Equipment("equip:longsword_iron"),
            quantity: 3,
            price_override: None,
            min_level: None,
            faction_requirement: None,
            condition: None,
        ),
        (
            item: Equipment("equip:chainmail"),
            quantity: 2,
            price_override: Some(500),
            min_level: Some(5),
            faction_requirement: None,
            condition: None,
        ),
        (
            item: Item("itm:iron_ingot"),
            quantity: 0,
            price_override: None,
            min_level: None,
            faction_requirement: None,
            condition: None,
        ),
        (
            item: Consumable("con:sharpening_stone"),
            quantity: 5,
            price_override: None,
            min_level: None,
            faction_requirement: None,
            condition: Some("cond:after_blacksmith_quest"),
        ),
    ],

    buy_price_modifier: 1.0,
    sell_price_modifier: 0.5,

    faction_discounts: Some([
        (faction_id: "faction:kingdom", reputation_threshold: 100, discount_percent: 0.1),
        (faction_id: "faction:kingdom", reputation_threshold: 500, discount_percent: 0.2),
        (faction_id: "faction:miners_guild", reputation_threshold: 200, discount_percent: 0.15),
    ]),

    refresh_rules: Some((
        refresh_type: OnRest,
        refresh_value: 1,
    )),

    unlock_conditions: None,

    tags: ["tag:blacksmith", "tag:weapon", "tag:armor"],
    icon_key: Some("icons/shops/blacksmith.png"),
)
```

---

## 6. 与其他 L3 Def 的关系

| L3 Def | ShopDef 的关系 |
|--------|---------------|
| QuestDef | ShopDef 通过 `rewards.unlock_shops` 被任务解锁。ShopDef 使用 `unlock_conditions` 实现更灵活的访问控制 |
| RecipeDef | 无直接引用关系。制造台在 Crafting 领域独立管理，不在 ShopDef 中定义 |
| EncounterDef | 无直接引用关系 |

**重要**：商店的物理位置（在哪张地图的哪个坐标）不在 ShopDef 中定义。此关联由 L4 MapDef 建立——L4 定义哪个 NPC 在哪个位置拥有哪个 ShopDef。这保证了 L3 保持地图无关性。

---

*本文档由 @content-architect 维护。*
