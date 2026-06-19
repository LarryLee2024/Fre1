---
id: 03-content.definitions.entities.equipment-def
title: EquipmentDef — Equipment Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# EquipmentDef — Equipment Content Def 定义

> **Content Layer**: L2 Entity | **领域规则**: `docs/02-domain/domains/inventory_domain.md` | **数据 Schema**: `docs/04-data/domains/inventory_schema.md` | **插件代码**: `src/content/plugins/equipment_plugin.rs`

---

## 1. Overview

EquipmentDef 定义了可穿戴装备的模板——武器、护甲、饰品、戒指等。EquipmentDef 嵌入 ItemBase 字段（名称、价格、重量、稀有度、图标），并额外包含装备特有的数据：槽位类型、属性修正、被动技能。

### 关键设计原则

- **自包含物品定义**：EquipmentDef 嵌入完整的 ItemBase 字段。一个 EquipmentDef 记录就是该装备的"物品记录"，不需要额外查询 ItemDef
- **属性通过 Modifier 管线的间接修正**：EquipmentDef 不存储属性值（如 "+2 力量"），而是引用 ModifierDef，通过 Modifier 管线应用到角色属性。禁止直接定义属性修正值
- **被动技能通过 AbilityDef**：装备提供的被动效果（如 "火焰抗性"、"夜视"）不是 EquipmentDef 直接定义的，而是通过 AbilityDef 引用。穿装备 = 注册 Ability，脱装备 = 注销 Ability
- **套装系统**：`set_id` 字段将装备分组为套装。套装加成效果不由 EquipmentDef 定义，而归属于 L3 的套装系统
- **Mod 支持**：EquipmentSlotType::Custom(String) 允许 Mod 定义新的装备槽位类型

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `inventory_domain.md` | 装备槽位体系、物品稀有度、穿戴规则 |
| `inventory_schema.md` | EquipmentDef 完整字段结构、EquipmentSlotType 定义 |
| `item-def.md` | 共享的 ItemBase 字段定义 |
| `modifier-def.md` | 本 Def 的 `stat_modifiers` 引用的 ModifierDef |
| `ability-def.md` | 本 Def 的 `passive_abilities` 引用的 AbilityDef |
| `buff-def.md` | 本 Def 的 `on_equip_buffs` 引用的 BuffDef |
| `tag-def.md` | 本 Def 的 `tags` 引用的 TagDef |
| `attribute-def.md` | ModifierDef 间接引用 |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 装备模板定义——描述一件可穿戴装备的静态属性。
///
/// EquipmentDef 嵌入 ItemBase 所有字段（名称、价格、稀有度等），
/// 加上装备独有字段（槽位、属性修正、被动技能、套装）。
///
/// 经 Load → Deserialize → Validate → Register → Freeze 管线后
/// 进入 DefRegistry<EquipmentDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct EquipmentDef {
    // ── 嵌入 ItemBase ──
    /// 全局唯一 ID（EquipmentDef 前缀: `equip_`）
    pub id: EquipmentId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    /// 物品稀有度
    pub rarity: Rarity,
    /// 基础价格（金币）
    pub base_price: u32,
    /// 重量（磅）
    pub weight: f32,
    /// 最大堆叠数（装备始终为 1）
    pub max_stack: u32,
    /// 图标 Key
    pub icon_key: Option<String>,
    /// 模型 Key
    pub model_key: Option<String>,
    /// 标签列表（引用 L0 TagDef）
    pub tags: Vec<TagId>,

    // ── 装备特有字段 ──

    // ── 槽位 ──
    /// 装备槽位类型（决定此装备可装备在哪个槽位）
    pub slot_type: EquipmentSlotType,

    /// 是否双手武器（占用 MainHand + OffHand）
    pub is_two_handed: bool,

    // ── 属性修正 ──
    /// 装备提供的属性修正器列表（引用 L1 ModifierDef）
    ///
    /// 示例：长剑 +1 攻击 ⇒ [`mod:longsword_magic_attack_bonus`]
    /// 注意：EquipmentDef 不能直接定义属性修正值，必须通过 ModifierDef 引用。
    pub stat_modifiers: Vec<ModifierId>,

    // ── 被动能力 ──
    /// 穿戴时激活的被动技能列表（引用 L1 AbilityDef）
    pub passive_abilities: Vec<AbilityId>,

    /// 穿戴时获得的常驻 Buff（引用 L1 BuffDef）
    pub on_equip_buffs: Vec<BuffId>,

    // ── 装备条件 ──
    /// 穿戴此装备的条件（引用 L1 ConditionDef）
    pub equip_conditions: Option<Vec<ConditionId>>,

    // ── 套装 ──
    /// 套装标识（同一 set_id 的装备构成套装）
    pub set_id: Option<String>,

    // ── 耐久度 ──
    /// 是否具有耐久度系统（true = 需要修理）
    pub has_durability: bool,
    /// 耐久度上限（has_durability = true 时有效）
    pub durability_max: Option<u32>,
}
```

### 内嵌数据结构

```rust
/// 装备槽位类型枚举
///
/// 基于 Inventory Domain 定义的 D&D 5e/BG3 装备槽位体系。
/// Custom 变体允许 Mod 扩展新槽位类型。
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum EquipmentSlotType {
    MainHand,
    OffHand,
    Helmet,
    Armor,
    Gloves,
    Boots,
    Cloak,
    Ring1,
    Ring2,
    Amulet,
    Special,
    Custom(String),
}
```

### 字段说明

- **`item_type` (ItemBase 中无此字段)**: EquipmentDef 不包含 `item_type` 字段——装备的物品类型就是"Equipment"，由其槽位类型隐含
- **`max_stack`**: 装备始终不可堆叠，值固定为 1
- **`stat_modifiers`**: 仅引用 ModifierDef。禁止通过 EquipmentDef 直接写入属性修正值。这是 P0 红线约束：所有属性修改必须通过 Modifier 管线
- **`passive_abilities` / `on_equip_buffs`**: 穿戴时被动激活的能力和 Buff。脱装备时自动注销
- **`equip_conditions`**: 穿戴条件，如 "需要 15 力量"、"仅限精灵"、"仅限战士"等。运行时检查这些条件是否满足
- **`set_id`**: 字符串标识，非 Def 引用。套装加成规则由 L3 系统定义。同 set_id 的装备穿戴 N 件时激活套装加成

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// EquipmentDef 注册插件
pub struct EquipmentDefPlugin;

impl Plugin for EquipmentDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<EquipmentDef>();
        app.init_asset_loader::<RonAssetLoader<EquipmentDef>>();
        app.insert_resource(DefRegistry::<EquipmentDef>::new());
        app.add_systems(
            PreUpdate,
            load_equipment_defs
                .run_if(resource_changed::<Assets<EquipmentDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 EquipmentDef
pub fn get_equipment_def(
    equipment_id: &EquipmentId,
    registry: &DefRegistry<EquipmentDef>,
) -> Option<&EquipmentDef> {
    registry.get(equipment_id)
}

/// 按槽位类型过滤 EquipmentDef
pub fn get_equipment_defs_by_slot(
    slot_type: &EquipmentSlotType,
    registry: &DefRegistry<EquipmentDef>,
) -> Vec<&EquipmentDef> {
    registry.iter()
        .filter(|def| def.slot_type == *slot_type)
        .collect()
}

/// 按套装过滤 EquipmentDef
pub fn get_equipment_defs_by_set(
    set_id: &str,
    registry: &DefRegistry<EquipmentDef>,
) -> Vec<&EquipmentDef> {
    registry.iter()
        .filter(|def| def.set_id.as_deref() == Some(set_id))
        .collect()
}

/// 按稀有度过滤 EquipmentDef
pub fn get_equipment_defs_by_rarity(
    rarity: Rarity,
    registry: &DefRegistry<EquipmentDef>,
) -> Vec<&EquipmentDef> {
    registry.iter()
        .filter(|def| def.rarity == rarity)
        .collect()
}
```

### 注册生命周期

```
EquipmentDefPlugin::build
  │
  ├── EquipmentDef 从 assets/config/02_entities/equipment.ron 加载
  │
  ├── Deserialize (ron::from_str)
  │
  ├── Validate
  │     ├── ID 唯一性（与 ItemDef/ConsumableDef 独立命名空间）
  │     ├── ItemBase 字段校验同 ItemDef
  │     ├── slot_type 合法性验证
  │     ├── is_two_handed 与 slot_type 一致性
  │     ├── stat_modifiers 中每个 ModifierId 存在
  │     ├── passive_abilities 中每个 AbilityId 存在
  │     ├── on_equip_buffs 中每个 BuffId 存在
  │     ├── equip_conditions 引用存在性
  │     ├── has_durability + durability_max 一致性
  │     ├── max_stack 强制为 1
  │     └── 层间依赖检查（禁止 L3/L4 引用）
  │
  ├── Register
  │
  └── Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | EquipmentId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `base_price` 范围 | 0-999999 |
| V4 | `weight` 范围 | 0.0-1000.0 |
| V5 | `max_stack` 必须为 1 | 装备不可堆叠 |
| V6 | `slot_type` 合法 | EquipmentSlotType 的已知变体之一 |
| V7 | `is_two_handed` 与 `slot_type` 一致性 | 双手武器 ⇒ slot_type 可为 MainHand 或 Custom |
| V8 | `has_durability` + `durability_max` 一致性 | has_durability=true ⇒ durability_max Some 且 >= 1 |
| V9 | `durability_max`（如果设置）范围 | 1-9999 |
| V10 | `rarity` 合法 | 必须匹配 Rarity 的已知变体 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V11 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V12 | `stat_modifiers` 中的每个 ModifierId 已注册 | 在 DefRegistry<ModifierDef> 中存在 |
| V13 | `passive_abilities` 中的每个 AbilityId 已注册 | 在 DefRegistry<AbilityDef> 中存在 |
| V14 | `on_equip_buffs` 中的每个 BuffId 已注册 | 在 DefRegistry<BuffDef> 中存在 |
| V15 | `equip_conditions` 中的每个 ConditionId（如果设置）已注册 | 在 DefRegistry<ConditionDef> 中存在 |

### 4.3 层间依赖校验

| # | 规则 | 说明 |
|---|------|------|
| V16 | EquipmentDef 不得引用任何 L3 Gameplay Def | 层间依赖方向规则 |
| V17 | EquipmentDef 不得引用任何 L4 World Def | 同上 |
| V18 | EquipmentDef 不得直接引用 AttributeId | 必须通过 ModifierDef 间接引用 |

### 4.4 语义校验

| # | 规则 | 说明 |
|---|------|------|
| V19 | 传说装备的唯一性提醒 | Rarity::Legendary 应在所有内容中唯一（不可有两个同名传说装备） |
| V20 | 双手武器警告 | is_two_handed=true 时，穿戴者必须有 MainHand 和 OffHand 槽位 |
| V21 | 套装的同一 set_id 装备槽位不重复 | 同一套装在一件装备上不应出现两个同槽位的装备（套装配置校验） |

---

## 5. RON 示例

```ron
(
    // ── ItemBase ──
    id: "equip:longsword_iron",
    name_key: "equipment.equip_longsword_iron.name",
    description_key: "equipment.equip_longsword_iron.desc",
    schema_version: 1,

    rarity: Common,
    base_price: 15,
    weight: 3.0,
    max_stack: 1,

    icon_key: Some("icons/equipment/longsword_iron.png"),
    model_key: Some("models/equipment/longsword_iron.glb"),

    tags: ["tag:weapon", "tag:martial_weapon", "tag:melee", "tag:metal"],

    // ── 装备特有 ──
    slot_type: MainHand,
    is_two_handed: false,

    stat_modifiers: [
        "mod:longsword_iron_attack_bonus",
        "mod:longsword_iron_damage_bonus",
    ],

    passive_abilities: [],
    on_equip_buffs: [],

    equip_conditions: Some([
        "cond:proficiency_martial_weapons",
    ]),

    set_id: None,

    has_durability: true,
    durability_max: Some(100),
)
```

```ron
(
    id: "equip:plate_armor_mithral",
    name_key: "equipment.equip_plate_armor_mithral.name",
    description_key: "equipment.equip_plate_armor_mithral.desc",
    schema_version: 1,

    rarity: VeryRare,
    base_price: 5000,
    weight: 15.0,
    max_stack: 1,

    icon_key: Some("icons/equipment/plate_armor_mithral.png"),
    model_key: Some("models/equipment/plate_armor_mithral.glb"),

    tags: ["tag:armor", "tag:heavy_armor", "tag:metal", "tag:mithral"],

    slot_type: Armor,
    is_two_handed: false,

    stat_modifiers: [
        "mod:mithral_plate_armor_class",
        "mod:mithral_plate_weight_reduction",
    ],

    passive_abilities: [
        "ability:mithral_grace",
    ],

    on_equip_buffs: [],

    equip_conditions: Some([
        "cond:proficiency_heavy_armor",
        "cond:strength_minimum_15",
    ]),

    set_id: Some("set:mithral_guardian"),

    has_durability: true,
    durability_max: Some(500),
)
```

---

## 6. 与 ItemDef / ConsumableDef 的关系

| 对比维度 | ItemDef | EquipmentDef | ConsumableDef |
|----------|---------|-------------|---------------|
| 核心用途 | 无使用/装备效果的纯物品 | 穿戴提供属性修正+被动 | 使用消耗产生一次效果 |
| ItemBase 嵌入 | 全量 + sellable/droppable | 全量（无 sellable/droppable） | 全量（无 sellable/droppable） |
| 特有字段 | sellable, droppable | slot_type, stat_modifiers, passive_abilities, on_equip_buffs, equip_conditions, set_id, durability | use_effect, target_rule, use_conditions, stack_behavior |
| L1 引用 | 无 | ModifierDef, AbilityDef, BuffDef, ConditionDef | EffectDef, ConditionDef, TargetingDef |
| 堆叠性 | 可堆叠（材料和货币） | 不可堆叠 | 通常不可堆叠 |
| 运行时创建 | 直接使用 ItemDef | 创建装备实例 + 应用 ModifierDef | 使用 EffectDef 执行效果 |

---

*本文档由 @content-architect 维护。*
