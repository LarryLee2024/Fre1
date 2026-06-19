---
id: 03-content.definitions.gameplay.recipe-def
title: RecipeDef — Recipe Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# RecipeDef — Recipe Content Def 定义

> **Content Layer**: L3 Gameplay | **领域规则**: `docs/02-domain/domains/crafting_domain.md` | **数据 Schema**: `docs/04-data/domains/crafting_schema.md` | **插件代码**: `src/content/plugins/recipe_plugin.rs`

---

## 1. Overview

RecipeDef 定义了制造配方——将输入物品（材料）转换为输出物品（装备/消耗品/材料）的规则。RecipeDef 是"配方配置"而非"制造操作实例"：每次制造操作由 Crafting 领域的运行时状态管理。

### 关键设计原则

- **输入输出分离**：配方定义明确的输入（材料列表）和输出（产物列表），制造系统按配方执行消耗与产出
- **技能检定**：`skill_requirement` 引用 L0 TagDef 标识技能类型 + DC 值，由 Execution 领域的技能检定系统处理
- **制造台绑定**：`station_type` 定义配方需要哪种制造台，制造台类型由 Crafting 领域管理
- **L2 依赖**：配方引用 ItemDef（材料/产物）、EquipmentDef（产物）、ConsumableDef（产物），不引用任何 L4 Def

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `crafting_domain.md` | 配方结构、制造台类型、附魔规则 |
| `crafting_schema.md` | RecipeDef 完整字段结构、RecipeOutput 定义 |
| `tag-def.md` | 本 Def 的 `skill_requirement.skill_tag` 和 `tags` 引用的 TagDef |
| `condition-def.md` | 本 Def 的 `unlock_conditions` 引用的 ConditionDef |
| `item-def.md` | 本 Def 的 `ingredients` 和 `outputs` 引用的 ItemDef |
| `equipment-def.md` | 本 Def 的 `outputs` 引用的 EquipmentDef |
| `consumable-def.md` | 本 Def 的 `outputs` 引用的 ConsumableDef |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 制造配方定义——描述材料组合到产物的转换规则。
///
/// RecipeDef 是 Content Asset，经 Load → Deserialize → Validate → Register → Freeze
/// 管线后进入 DefRegistry<RecipeDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct RecipeDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID（RecipeDef 前缀: `rcp_`）
    pub id: RecipeId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 制造条件 ──
    /// 需要的制造台类型
    pub station_type: CraftingStation,

    /// 技能要求（可选——无技能要求 = 自动成功）
    pub skill_requirement: Option<SkillRequirement>,

    /// 解锁条件（引用 L1 ConditionDef，可选——无条件 = 默认可用）
    pub unlock_conditions: Option<Vec<ConditionId>>,

    // ── 材料与产出 ──
    /// 材料列表（输入，制造时消耗）
    pub ingredients: Vec<RecipeIngredient>,

    /// 产物列表（输出，制造时产出）
    pub outputs: Vec<RecipeOutput>,

    // ── 制造参数 ──
    /// 制造所需时间（单位：交互次数/回合）
    pub craft_time: u32,

    /// 制造数量（一次配方产出几份产物）
    pub quantity_per_craft: u32,

    // ── 元数据 ──
    /// 标签列表（引用 L0 TagDef，用于分类过滤）
    pub tags: Vec<TagId>,
}
```

### 内嵌数据结构

```rust
/// 制造台类型
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum CraftingStation {
    /// 锻造台
    Forge,
    /// 附魔台
    EnchantingTable,
    /// 炼金台
    AlchemyLab,
    /// 裁缝台
    TailoringBench,
    /// 工程台
    EngineeringWorkbench,
    /// 烹饪/营火
    CookingFire,
    /// 任意制造台
    Any,
    /// 自定义（Mod 扩展用）
    Custom(String),
}

/// 技能要求——制造成功所需的技能检定
#[derive(Deserialize, Clone, Debug)]
pub struct SkillRequirement {
    /// 技能标签（引用 L0 TagDef，如 "tag:smithing_tools"）
    pub skill_tag: TagId,
    /// 检定难度等级（DC）
    pub dc: u32,
}

/// 配方材料——制造中消耗的一个物品条目
#[derive(Deserialize, Clone, Debug)]
pub struct RecipeIngredient {
    /// 物品引用（引用 ItemDef）
    pub item_id: ItemId,
    /// 所需数量
    pub quantity: u32,
    /// 是否消耗（true = 制造时消耗，false = 工具/催化剂，不消耗）
    pub consumed: bool,
}

/// 配方产物——制造产出的物品/装备/消耗品
#[derive(Deserialize, Clone, Debug)]
pub enum RecipeOutput {
    /// 产出通用物品（(ItemId, 数量)）
    Item(ItemId, u32),
    /// 产出单件装备
    Equipment(EquipmentId),
    /// 产出消耗品（(ConsumableId, 数量)）
    Consumable(ConsumableId, u32),
}
```

### 字段说明

- **`station_type`**: 制造台类型决定了玩家在哪里执行此配方。锻造永远在锻造台，炼金在炼金台。`Custom(String)` 为 Mod 扩展制造台类型
- **`skill_requirement`**: DC 和技能标签共同决定制造成功率。无 skill_requirement = 无检定，制造必定成功
- **`unlock_conditions`**: 配方不是默认可见的。条件集合为空 = 默认可见，有条件 = 需条件满足时才在制造菜单显示
- **`ingredients[].consumed`**: 大部分材料 `consumed: true`（矿石、草药）。工具类材料（如"锻造锤"）设为 `consumed: false`，不消耗但必须拥有

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// RecipeDef 注册插件
pub struct RecipeDefPlugin;

impl Plugin for RecipeDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<RecipeDef>();
        app.init_asset_loader::<RonAssetLoader<RecipeDef>>();
        app.insert_resource(DefRegistry::<RecipeDef>::new());
        app.add_systems(
            PreUpdate,
            load_recipe_defs
                .run_if(resource_changed::<Assets<RecipeDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按制造台类型过滤 RecipeDef
pub fn get_recipes_by_station(
    station: CraftingStation,
    registry: &DefRegistry<RecipeDef>,
) -> Vec<&RecipeDef> {
    registry.iter()
        .filter(|def| def.station_type == station || def.station_type == CraftingStation::Any)
        .collect()
}

/// 按产物过滤 RecipeDef（查找"如何制造 X"）
pub fn get_recipe_for_output(
    output_id: &str,
    registry: &DefRegistry<RecipeDef>,
) -> Option<&RecipeDef> {
    registry.iter().find(|def| {
        def.outputs.iter().any(|output| match output {
            RecipeOutput::Item(id, _) => id.as_str() == output_id,
            RecipeOutput::Equipment(id) => id.as_str() == output_id,
            RecipeOutput::Consumable(id, _) => id.as_str() == output_id,
        })
    })
}
```

### 注册生命周期

```
RecipeDefPlugin::build
  │
  ├── RecipeDef 从 assets/config/03_gameplay/recipes.ron 加载
  │
  ├── Deserialize → Validate → Register → Freeze
  │
  └── Validate 具体规则：
        ├── ID 唯一性
        ├── L0 (TagId) 引用存在性
        ├── L2 (ItemId/EquipmentId/ConsumableId) 引用存在性
        ├── L3 引用检查（仅自身，无同层循环风险）
        ├── ingredients 非空（至少 1 个材料）
        ├── outputs 非空（至少 1 个产物）
        ├── craft_time 范围（1-100）
        ├── quantity_per_craft >= 1
        ├── DC 范围（1-30，如果设置）
        └── L4 禁止引用检查
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | RecipeId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `ingredients` 非空 | 配方必须有至少一个材料 |
| V4 | `outputs` 非空 | 配方必须有至少一个产物 |
| V5 | `craft_time` 范围 | 1-100（交互次数/回合） |
| V6 | `quantity_per_craft` >= 1 | 每次制造至少产出 1 份 |
| V7 | `skill_requirement.dc` 范围（如果设置） | 1-30 |
| V8 | `ingredients` 中的 `quantity` >= 1 | 材料数量至少为 1 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V9 | `skill_requirement.skill_tag`（如果设置）已注册 | 在 DefRegistry<TagDef> 中存在 |
| V10 | `unlock_conditions` 中的每个 ConditionId（如果设置）已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V11 | `ingredients` 中的每个 ItemId 已注册 | 在 DefRegistry<ItemDef> 中存在 |
| V12 | `outputs` 中的每个 ItemId/EquipmentId/ConsumableId 已注册 | 在对应 DefRegistry 中存在 |
| V13 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |

### 4.3 语义校验

| # | 规则 | 说明 |
|---|------|------|
| V14 | 产物类型与物品类型一致 | Item 产出对应 ItemDef，Equipment 产出对应 EquipmentDef |
| V15 | 产出装备时 quantity 应为 1 | Equipment 不可批量产出（装备不可堆叠） |
| V16 | 材料不重复 | 同种物品不应作为两个独立 ingredient 条目出现 |
| V17 | ingredient 非 output | 防"永动机"配方检查（可选警告） |

### 4.4 层间依赖校验

| # | 规则 | 说明 |
|---|------|------|
| V18 | RecipeDef 不得引用任何 L4 World Def | 层间依赖方向规则 |

---

## 5. RON 示例

```ron
(
    id: "rcp:health_potion",
    name_key: "recipe.rcp_health_potion.name",
    description_key: "recipe.rcp_health_potion.desc",
    schema_version: 1,

    station_type: AlchemyLab,

    skill_requirement: Some((
        skill_tag: "tag:alchemist_tools",
        dc: 10,
    )),

    unlock_conditions: None,

    ingredients: [
        (item_id: "itm:healing_herb", quantity: 3, consumed: true),
        (item_id: "itm:empty_vial", quantity: 1, consumed: true),
        (item_id: "itm:alchemist_tools", quantity: 1, consumed: false),
    ],

    outputs: [
        Consumable("con:health_potion", 1),
    ],

    craft_time: 2,
    quantity_per_craft: 1,

    tags: ["tag:alchemy", "tag:healing", "tag:consumable"],
)
```

```ron
(
    id: "rcp:flame_sword",
    name_key: "recipe.rcp_flame_sword.name",
    description_key: "recipe.rcp_flame_sword.desc",
    schema_version: 1,

    station_type: Forge,

    skill_requirement: Some((
        skill_tag: "tag:smithing_tools",
        dc: 15,
    )),

    unlock_conditions: Some(["cond:has_dwarf_faction_quest_progress"]),

    ingredients: [
        (item_id: "itm:iron_ingot", quantity: 4, consumed: true),
        (item_id: "itm:fire_crystal", quantity: 2, consumed: true),
        (item_id: "itm:leather_wrap", quantity: 1, consumed: true),
        (item_id: "itm:smithing_hammer", quantity: 1, consumed: false),
    ],

    outputs: [
        Equipment("equip:flame_sword"),
    ],

    craft_time: 3,
    quantity_per_craft: 1,

    tags: ["tag:smithing", "tag:weapon", "tag:fire"],
)
```

---

## 6. 与其他 L3 Def 的关系

| L3 Def | RecipeDef 的关系 |
|--------|-----------------|
| QuestDef | RecipeDef 通过 `rewards.unlock_recipes` 被任务解锁。RecipeDef 不知晓哪个任务解锁它 |
| ShopDef | RecipeDef 不引用 ShopDef。商店出售的材料由 ShopDef 独立定义 |
| EncounterDef | 无直接关系 |

---

*本文档由 @content-architect 维护。*
