---
id: 03-content.definitions.modifier-def
title: ModifierDef — Modifier Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# ModifierDef — Modifier Content Def 定义

> **Content Layer**: L1 Capability | **领域规则**: `docs/02-domain/capabilities/modifier_domain.md` | **数据 Schema**: `docs/04-data/capabilities/modifier_schema.md` | **插件代码**: `src/content/plugins/modifier_plugin.rs`

---

## 1. Overview

ModifierDef 是**可复用的数值修改模板**——定义对某个属性的运算（Add/Multiply/Override）、幅度和优先级。ModifierDef 的核心价值在于**共享和统一**：

- **共享**：同一数值修改规则可在多个 EffectDef 之间共享（如所有"火焰伤害+"的效果都引用同一个 `mod:fire_damage_pct`）
- **统一**：全局修改某类数值时只需改一个 ModifierDef，所有引用它的 EffectDef 自动生效
- **组合**：一个 EffectDef 可以引用多个 ModifierDef，组合出复杂的属性修改效果

### 设计原则

- **ModifierDef ≠ ModifierConfig**：ModifierDef 是独立注册的 Asset，ModifierConfig 是内联在 EffectDef 中的匿名配置。两者在运行时合并为 ModifierData
- **ModifierDef 是纯数值描述**：不含业务逻辑、不含条件判断（条件判断由 ConditionDef 处理）
- **ModifierDef 可被任何 EffectDef 引用**：跨 Effect 共享同一修改规则

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `modifier_domain.md` | 运算类型规则、优先级排序、Override 冲突处理、ScalableValue 解析 |
| `modifier_schema.md` | ModifierConfig、ScalableValue、ModifierOp、ModifierSource 的数据结构 |
| `effect-def.md` | 本 Def 被引用的地方——EffectDef.modifier_defs, ModifierConfig.modifier_def_id |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// Modifier Def 定义——可复用的数值修改模板。
///
/// ModifierDef 是独立注册的 Content Asset，可被多个 EffectDef 引用。
/// 运行时 ModifierDef + ModifierConfig 合并解析为 ModifierData。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ModifierDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: ModifierId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号（用于未来迁移兼容）
    pub schema_version: u32,

    // ── 修改器核心 ──
    /// 运算类型
    pub op: ModifierOp,

    /// 目标属性 ID
    pub target_attribute: AttributeId,

    /// 幅度值（固定值 / 曲线 / 属性缩放）
    pub value: ScalableValue,

    /// 执行优先级（0-100，越小越优先，默认 50）
    pub priority: u8,

    // ── 来源追踪 ──
    /// 修改器来源类型（用于追溯和堆叠判定）
    pub source_type: ModifierSourceType,

    /// 来源描述 Key（可选，用于 UI 显示"来自XX装备"）
    pub source_description_key: Option<LocalizationKey>,

    // ── 过滤条件 ──
    /// 条件性生效规则（可选）
    pub filter: Option<ModifierFilter>,

    // ── 标签和分类 ──
    /// 修改器标签（用于分类过滤和条件检查）
    pub tags: Vec<TagId>,

    /// 当前版本说明（仅用于文档，不影响逻辑）
    pub changelog: Option<String>,
}

/// 修改器过滤规则（条件性生效）
#[derive(Deserialize, Clone, Debug)]
pub struct ModifierFilter {
    /// 仅在目标持有特定标签时生效
    pub required_tags: Option<TagQuery>,

    /// 仅在特定 GameplayContext 下生效
    pub required_context: Option<ContextCondition>,
}

/// 标签查询
#[derive(Deserialize, Clone, Debug)]
pub enum TagQuery {
    HasAll(Vec<TagId>),
    HasAny(Vec<TagId>),
    HasNone(Vec<TagId>),
}

/// 上下文条件
#[derive(Deserialize, Clone, Debug)]
pub enum ContextCondition {
    DuringCombat,
    OutOfCombat,
    DayTime,
    NightTime,
    Custom(String),
}
```

### 字段说明

- **`op`**: 三种运算类型——Add（加法）、Multiply（乘法）、Override（覆盖）。运行时按 `Add → Multiply → Override` 顺序应用
- **`value`**: ScalableValue 支持三种模式：`Fixed(f32)` 固定值、`Curve` 曲线查询、`AttributeScaling` 属性缩放
- **`priority`**: 0-100，越小越优先。Add 和 Multiply 类型的优先级影响同类运算内部的执行顺序。Override 类型同名属性上优先级最高的生效
- **`source_type`**: 用于堆叠判定和来源追溯。多个同源同类型的 Modifier 可能触发堆叠规则
- **`filter`**: 条件过滤，用于"仅在夜晚生效"、"仅在战斗中生效"等场景
- **`tags`**: 用于分类和条件检查，如 `tag:modifier_物理`、`tag:modifier_魔法`

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// ModifierDef 注册插件
pub struct ModifierDefPlugin;

impl Plugin for ModifierDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<ModifierDef>();
        app.init_asset_loader::<RonAssetLoader<ModifierDef>>();
        app.insert_resource(DefRegistry::<ModifierDef>::new());

        app.add_systems(
            PreUpdate,
            load_modifier_defs
                .run_if(resource_changed::<Assets<ModifierDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 ModifierDef
pub fn get_modifier_def(id: &ModifierId, registry: &DefRegistry<ModifierDef>) -> Option<&ModifierDef> {
    registry.get(id)
}

/// 按标签过滤 ModifierDef
pub fn get_modifiers_by_tag(tag_id: &TagId, registry: &DefRegistry<ModifierDef>) -> Vec<&ModifierDef> {
    registry.iter()
        .filter(|def| def.tags.contains(tag_id))
        .collect()
}
```

### 注册生命周期

```
Load (modifiers.ron) → Deserialize → Validate → Register (DefRegistry<ModifierDef>) → Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | ModifierId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `priority` 范围 | 0-100 |
| V4 | `value` 合法 | Fixed 模式 value >= 0 不做硬性限制但需记录；Curve 模式的 curve_id 非空 |
| V5 | `source_type` 有效 | 匹配 ModifierSourceType 的已知变体（Custom 除外） |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V6 | `target_attribute` 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V7 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V8 | `filter.required_tags` 中的 TagId 已注册 | 在 DefRegistry<TagDef> 中存在（如果设置） |
| V9 | `value` 为 Curve 时，curve_id 在 CurveTableRegistry 中已注册 | 曲线表必须预加载 |
| V10 | `value` 为 AttributeScaling 时，source_attribute 已注册 | 在 DefRegistry<AttributeDef> 中存在 |

### 4.3 逻辑一致性校验

| # | 规则 | 说明 |
|---|------|------|
| V11 | `value` 为 AttributeScaling 时，source_attribute != target_attribute | 防止逻辑循环 |
| V12 | ModifierDef 不得引用任何 L2+ Def | L1 内容不可引用 Entity/Gameplay/World 层内容 |

---

## 5. RON 示例

```ron
// ModifierDef 示例：火焰伤害增强（+25%）
//
// 这是一个可复用的百分比乘法修改器，可被任何 EffectDef 或 ModifierConfig 引用。
(
    id: "mod:fire_damage_pct",
    name_key: "modifier.fire_damage_pct.name",
    description_key: "modifier.fire_damage_pct.desc",
    schema_version: 1,

    op: Multiply,
    target_attribute: "attr:fire_damage",
    value: AttributeScaling(
        source_attribute: "attr:intelligence",
        ratio: 0.25,
    ),
    priority: 50,

    source_type: Passive,
    source_description_key: Some("modifier.fire_damage_pct.source"),

    tags: ["tag:modifier_magical", "tag:modifier_offensive"],
)
```

```ron
// ModifierDef 示例：固定防御值
//
// 一个简单的 Add 类型修改器，被装备类 EffectDef 引用。
(
    id: "mod:defense_flat",
    name_key: "modifier.defense_flat.name",
    description_key: "modifier.defense_flat.desc",
    schema_version: 1,

    op: Add,
    target_attribute: "attr:defense",
    value: Fixed(5.0),
    priority: 50,

    source_type: Equipment,
    source_description_key: Some("modifier.defense_flat.source"),

    tags: ["tag:modifier_physical", "tag:modifier_defensive"],
)
```

---

## 6. 设计说明

### ModifierDef vs ModifierConfig 的选择策略

| 场景 | 推荐方式 | 理由 |
|------|----------|------|
| 全局使用的通用修改（如"火焰伤害+25%"） | ModifierDef | 跨 Effect 共享，全局修改一处生效 |
| 特定 Effect 独有的数值调整（如"毒药 -2 敏捷"） | ModifierConfig (inline) | 无共享价值，内联定义更直接 |
| 装备属性修正（如"+5 力量"） | ModifierDef | 多件装备可能共享相同修正值 |
| 一次性消耗品效果（如"恢复 50HP"） | ModifierConfig (inline) | 无引用价值，使用即弃 |

### 运行时数据流

```
ModifierDef           ModifierConfig (inline)
    │                       │
    └───────┬───────────────┘
            ▼
    ModifierConfig (合并后)
            │
            ▼
    ModifierFactory.resolve(ScalableValue, GameplayContext)
            │
            ▼
    ModifierData (运行时实例, 含解析后的 magnitude: f32)
            │
            ▼
    ModifierContainer (ECS Component, 实体上的活跃修改器集合)
            │
            ▼
    Aggregator (按运算类型合并, 输出最终属性值)
```
