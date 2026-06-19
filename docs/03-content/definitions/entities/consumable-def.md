---
id: 03-content.definitions.entities.consumable-def
title: ConsumableDef — Consumable Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# ConsumableDef — Consumable Content Def 定义

> **Content Layer**: L2 Entity | **领域规则**: `docs/02-domain/domains/inventory_domain.md` | **数据 Schema**: `docs/04-data/domains/inventory_schema.md` | **插件代码**: `src/content/plugins/consumable_plugin.rs`

---

## 1. Overview

ConsumableDef 定义了消耗品模板——使用后消耗（或减少堆叠数）并产生一次性效果的物品。ConsumableDef 嵌入 ItemBase 字段（名称、价格、重量、稀有度、图标），并额外包含使用相关的数据：使用效果、目标规则、使用条件、使用后处理。

### 关键设计原则

- **自包含物品定义**：ConsumableDef 嵌入完整的 ItemBase 字段。一个 ConsumableDef 记录就是该消耗品的"物品记录"
- **效果执行通过 Effect 管线**：消耗品的使用效果不是直接定义在 ConsumableDef 中的，而是通过引用 EffectDef 来执行。ConsumableDef 只负责"使用什么效果"，不负责"效果如何执行"
- **目标选择通过 TargetingDef**：消耗品的目标规则引用 TargetingDef。这允许 ConsumableDef 复用完整的 L1 目标选择能力
- **使用条件通过 ConditionDef**：什么条件下可以使用该消耗品（如"必须受伤"、"必须处于战斗"），通过 ConditionDef 引用定义
- **ConsumableDef ≠ 物品分类标签**：某些物品（如毒药）既可以作为消耗品使用，也可以涂抹在武器上（给予 Buff）。这种情况通过两个 Def 表示——一个 ConsumableDef（使用）和一个 EquipmentDef 或 BuffDef（涂抹效果）

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `inventory_domain.md` | 消耗品分类、消耗规则、堆叠与使用 |
| `inventory_schema.md` | ConsumableDef 完整字段结构、ConsumableCategory、ConsumeBehavior 定义 |
| `item-def.md` | 共享的 ItemBase 字段定义 |
| `effect-def.md` | 本 Def 的 `use_effect` 引用的 EffectDef |
| `condition-def.md` | 本 Def 的 `use_conditions` 引用的 ConditionDef |
| `targeting-def.md` | 本 Def 的 `target_rule` 引用的 TargetingDef |
| `cue-def.md` | 本 Def 的 `use_cue` 引用的 CueDef |
| `tag-def.md` | 本 Def 的 `tags` 引用的 TagDef |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 消耗品模板定义——描述一个消耗品的静态属性。
///
/// ConsumableDef 嵌入 ItemBase 所有字段（名称、价格、稀有度等），
/// 加上消耗品独有字段（使用效果、目标规则、使用条件、消耗行为）。
///
/// 经 Load → Deserialize → Validate → Register → Freeze 管线后
/// 进入 DefRegistry<ConsumableDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ConsumableDef {
    // ── 嵌入 ItemBase ──
    /// 全局唯一 ID（ConsumableDef 前缀: `con_`）
    pub id: ConsumableId,
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
    /// 最大堆叠数（消耗品通常可堆叠）
    pub max_stack: u32,
    /// 图标 Key
    pub icon_key: Option<String>,
    /// 模型 Key
    pub model_key: Option<String>,
    /// 标签列表（引用 L0 TagDef）
    pub tags: Vec<TagId>,

    // ── 消耗品特有字段 ──

    // ── 消耗品分类 ──
    /// 消耗品类别
    pub consumable_category: ConsumableCategory,

    // ── 使用效果 ──
    /// 使用效果（引用 L1 EffectDef，必须）
    ///
    /// 消耗品的使用效果始终为一个 EffectDef。
    /// 若需要复合效果（治疗 + Buff + 移除 Debuff），应创建一个复合 EffectDef 来编排。
    pub use_effect: EffectId,

    /// 目标规则（引用 L1 TargetingDef，可选）
    ///
    /// None = 对自己使用（默认）。Some = 按规则选择目标。
    pub target_rule: Option<TargetingId>,

    // ── 使用条件 ──
    /// 可以使用此消耗品的条件列表（引用 L1 ConditionDef）
    ///
    /// 所有条件必须满足才能使用。示例：
    /// - "生命值低于 50% 才能使用治疗药水"
    /// - "战斗中才能使用烟雾弹"
    pub use_conditions: Option<Vec<ConditionId>>,

    // ── 使用行为 ──
    /// 使用后的消耗行为
    pub consume_behavior: ConsumeBehavior,

    // ── 表现 ──
    /// 使用时的表现信号（引用 L1 CueDef）
    pub use_cue: Option<CueId>,
}
```

### 内嵌数据结构

```rust
/// 消耗品类别
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConsumableCategory {
    /// 药水——直接产生效果
    Potion,
    /// 卷轴——施放一个法术
    Scroll,
    /// 投掷物——投掷到目标位置产生效果
    Throwable,
    /// 食物——提供持续 Buff 或恢复
    Food,
    /// 毒药——涂抹到武器上
    Poison,
    /// 弹药——远程武器消耗
    Ammo,
    /// 制作材料——用于 Crafting 系统
    CraftingMaterial,
    /// 其他
    Custom(String),
}

/// 消耗行为
#[derive(Deserialize, Clone, Debug)]
pub enum ConsumeBehavior {
    /// 消耗一个（堆叠数 -1），堆叠为 0 时移除
    ConsumeOne,
    /// 不消耗（如无限次使用的物品）
    NoConsume,
    /// 消耗全部堆叠（如一次性抓取所有矿石进行熔炼）
    ConsumeAll,
    /// 概率消耗（% 概率不消耗）
    ChanceNoConsume(f32),
}

/// 使用时机限制
#[derive(Deserialize, Clone, Debug)]
pub enum UseTiming {
    /// 任何时候
    Any,
    /// 仅战斗中
    CombatOnly,
    /// 仅非战斗
    PeaceOnly,
    /// 仅特定地图/场景
    LocationOnly(Vec<ConditionId>),
}
```

### 字段说明

- **`use_effect`**: 消耗品的核心——使用后产生的效果。必须是 EffectId（引用 L1 EffectDef）。不支持内联 EffectConfig，因为效果可能被多个消耗品共享（如"治疗药水"和"治疗药水+"使用不同的参数但共享治疗 Effect）
- **`target_rule`**: None 表示"对自己使用"（默认）。当消耗品需要选择目标时（如"向敌人投掷毒药"），引用 TargetingDef
- **`use_conditions`**: 使用前的条件检查。与 equip_conditions 不同，这些条件在使用时检查（而非穿戴/持有时）
- **`consume_behavior`**: 定义消耗品的消耗规则。ConsumeOne 是最常见的（药水、食物），NoConsume 用于剧情物品（无限使用的道具），ConsumeAll 用于一次性消耗所有堆叠
- **`consumable_category`**: 用于分类和 UI 显示，不影响 Effect 的实际执行

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// ConsumableDef 注册插件
pub struct ConsumableDefPlugin;

impl Plugin for ConsumableDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<ConsumableDef>();
        app.init_asset_loader::<RonAssetLoader<ConsumableDef>>();
        app.insert_resource(DefRegistry::<ConsumableDef>::new());
        app.add_systems(
            PreUpdate,
            load_consumable_defs
                .run_if(resource_changed::<Assets<ConsumableDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 ConsumableDef
pub fn get_consumable_def(
    consumable_id: &ConsumableId,
    registry: &DefRegistry<ConsumableDef>,
) -> Option<&ConsumableDef> {
    registry.get(consumable_id)
}

/// 按类别过滤 ConsumableDef
pub fn get_consumable_defs_by_category(
    category: ConsumableCategory,
    registry: &DefRegistry<ConsumableDef>,
) -> Vec<&ConsumableDef> {
    registry.iter()
        .filter(|def| def.consumable_category == category)
        .collect()
}
```

### 注册生命周期

```
ConsumableDefPlugin::build
  │
  ├── ConsumableDef 从 assets/config/02_entities/consumables.ron 加载
  │
  ├── Deserialize (ron::from_str)
  │
  ├── Validate
  │     ├── ID 唯一性（与 ItemDef/EquipmentDef 独立命名空间）
  │     ├── ItemBase 字段校验同 ItemDef
  │     ├── consumable_category 合法性
  │     ├── use_effect 引用存在性
  │     ├── target_rule 引用存在性（如果设置）
  │     ├── use_conditions 引用存在性
  │     ├── use_cue 引用存在性（如果设置）
  │     ├── consume_behavior 参数合法性（ChanceNoConsume 的 0.0-1.0）
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
| V1 | `id` 非空 | ConsumableId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `base_price` 范围 | 0-999999 |
| V4 | `weight` 范围 | 0.0-1000.0 |
| V5 | `max_stack` >= 1 | 堆叠数至少为 1 |
| V6 | `consumable_category` 合法 | 必须匹配 ConsumableCategory 的已知变体 |
| V7 | `consume_behavior` 参数合法 | ChanceNoConsume 的概率值 0.0-1.0 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V8 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V9 | `use_effect` 已注册 | 在 DefRegistry<EffectDef> 中存在 |
| V10 | `target_rule`（如果设置）已注册 | 在 DefRegistry<TargetingDef> 中存在 |
| V11 | `use_conditions` 中的每个 ConditionId（如果设置）已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V12 | `use_cue`（如果设置）已注册 | 在 DefRegistry<CueDef> 中存在 |

### 4.3 层间依赖校验

| # | 规则 | 说明 |
|---|------|------|
| V13 | ConsumableDef 不得引用任何 L3 Gameplay Def | 层间依赖方向规则 |
| V14 | ConsumableDef 不得引用任何 L4 World Def | 同上 |
| V15 | ConsumableDef 不得直接内联 Effect 逻辑 | 必须通过 EffectDef 引用 |

### 4.4 语义校验

| # | 规则 | 说明 |
|---|------|------|
| V16 | `use_effect` 的 duration 应为 Instant（建议） | 消耗品效果通常为瞬时。若为 HasDuration，确认是否为设计意图 |
| V17 | 投掷物应有 target_rule | consumable_category: Throwable ⇒ target_rule 应为 Some |
| V18 | 同类消耗品的 use_effect 可共享 | 检测相同使用效果的不同消耗品，提示是否可复用 EffectDef |

---

## 5. RON 示例

```ron
(
    // ── ItemBase ──
    id: "con:health_potion_minor",
    name_key: "consumable.con_health_potion_minor.name",
    description_key: "consumable.con_health_potion_minor.desc",
    schema_version: 1,

    rarity: Common,
    base_price: 50,
    weight: 0.5,
    max_stack: 10,

    icon_key: Some("icons/consumables/health_potion_minor.png"),
    model_key: Some("models/consumables/potion_red.glb"),

    tags: ["tag:potion", "tag:healing", "tag:consumable"],

    // ── 消耗品特有 ──
    consumable_category: Potion,

    use_effect: "eff:heal_minor",

    target_rule: None,

    use_conditions: None,

    consume_behavior: ConsumeOne,

    use_cue: Some("cue:potion_drink"),
)
```

```ron
(
    id: "con:scroll_fireball",
    name_key: "consumable.con_scroll_fireball.name",
    description_key: "consumable.con_scroll_fireball.desc",
    schema_version: 1,

    rarity: Rare,
    base_price: 500,
    weight: 0.1,
    max_stack: 5,

    icon_key: Some("icons/consumables/scroll_fireball.png"),
    model_key: None,

    tags: ["tag:scroll", "tag:fire", "tag:consumable", "tag:magic"],

    consumable_category: Scroll,

    use_effect: "eff:fireball",

    target_rule: Some("tgt:target_point_radius_2"),

    use_conditions: Some([
        "cond:has_arcana_proficiency",
    ]),

    consume_behavior: ConsumeOne,

    use_cue: Some("cue:scroll_cast_fire"),
)
```

---

## 6. 与 ItemDef / EquipmentDef 的关系

| 对比维度 | ItemDef | EquipmentDef | ConsumableDef |
|----------|---------|-------------|---------------|
| 核心用途 | 无效果的基础物品 | 穿戴提供属性修正 | 使用消耗产生效果 |
| 特有字段 | sellable, droppable | slot_type, stat_modifiers, passive_abilities, set_id, durability | consumable_category, use_effect, target_rule, use_conditions, consume_behavior |
| L1 引用 | 无 | ModifierDef, AbilityDef, BuffDef, ConditionDef | EffectDef, ConditionDef, TargetingDef, CueDef |
| 运行时创建 | 直接使用 ItemDef | 创建装备实例 + 应用 ModifierDef | 使用 EffectDef 执行效果 |
| 消耗性 | 否 | 否 | 是（堆叠减少） |

---

*本文档由 @content-architect 维护。*
