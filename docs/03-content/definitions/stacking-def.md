---
id: 03-content.definitions.stacking-def
title: StackingDef — Stacking Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# StackingDef — Stacking Content Def 定义

> **Content Layer**: L1 Capability | **领域规则**: `docs/02-domain/capabilities/stacking_domain.md` | **数据 Schema**: `docs/04-data/capabilities/stacking_schema.md` | **插件代码**: `src/content/plugins/stacking_plugin.rs`

---

## 1. Overview

StackingDef 定义**效果堆叠规则**——当同一效果多次作用时如何叠加。Stacking 是防止效果失控的关键机制：

- None：不堆叠，新的被忽略（默认，适用于大多数唯一的 Buff/Debuff）
- Aggregate：累加层数，每层独立生效（适用于"每层中毒 +2 伤害"）
- RefreshDuration：刷新持续时间，层数不变（适用于"续杯"类 Buff）
- Replace：新实例替换旧实例（适用于同名护盾、武器附魔）

### 设计原则

- **StackingDef 是纯规则配置**：不含逻辑实现，只声明策略类型和参数
- **独立注册**：StackingDef 可被 EffectDef 和 BuffDef 引用，实现堆叠规则复用
- **层数变化驱动 Modifier 重算**：当 `reapply_modifiers_on_stack = true` 时，每层层数变化都会触发 Modifier 重新注册

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `stacking_domain.md` | 四种堆叠策略的完整行为定义、同源/异源判定逻辑、溢出处理策略 |
| `stacking_schema.md` | StackingConfig、StackingType、OverflowBehavior、StackIdentity 的数据结构 |
| `effect-def.md` | 本 Def 被 EffectDef.stacking_def 引用 |
| `buff-def.md` | 本 Def 被 BuffDef.stacking_def 引用 |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// Stacking Def 定义——效果堆叠规则。
///
/// 定义当同一效果多次作用时的叠加行为。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct StackingDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: StackingId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 堆叠策略 ──
    /// 堆叠策略类型
    pub stacking_type: StackingType,

    /// 最大堆叠层数
    pub max_stacks: u32,

    /// 是否允许异源堆叠（不同来源的同效果）
    pub allow_cross_source: bool,

    /// 溢出处理（超出上限时的行为）
    pub overflow_behavior: OverflowBehavior,

    /// 层数变化时是否重算 Modifier
    pub reapply_modifiers_on_stack: bool,

    // ── 元数据 ──
    /// 分类标签
    pub tags: Vec<TagId>,
}

/// 堆叠策略
#[derive(Deserialize, Clone, Debug)]
pub enum StackingType {
    /// 不堆叠——新实例被忽略
    None,
    /// 累加层数——层数叠加，受 max_stacks 限制
    Aggregate,
    /// 刷新持续时间——重置剩余时间，层数不变
    RefreshDuration,
    /// 替换——新实例替换旧实例（按优先级或数值）
    Replace,
}

/// 溢出处理
#[derive(Deserialize, Clone, Debug)]
pub enum OverflowBehavior {
    /// 忽略新实例（保持上限层数）
    IgnoreNew,
    /// 移除最早层
    RemoveOldest,
    /// 刷新持续时间并保持上限
    RefreshAndCap,
}
```

### 字段说明

- **`stacking_type`**: 四种策略——None 适合"眩晕"（不能叠两层眩晕）；Aggregate 适合"中毒"（每层增伤）；RefreshDuration 适合"护盾"（刷新时长）；Replace 适合"附魔"（新的覆盖旧的）
- **`max_stacks`**: 堆叠上限。None 和 Replace 类型为 1，Aggregate 类型 >= 2
- **`allow_cross_source`**: 异源堆叠——不同施法者施加的同一效果是否叠加。如两个法师对同一目标施放"灼烧"，如果允许异源则叠加层数
- **`overflow_behavior`**: 超出上限后的处理。IgnoreNew 是最常用的，RemoveOldest 用于"有限槽位"的场景
- **`reapply_modifiers_on_stack`**: 层数变化时是否重新应用 Modifier。为 true 时，Aggregate 类型每层的增加值与层数成比例

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// StackingDef 注册插件
pub struct StackingDefPlugin;

impl Plugin for StackingDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<StackingDef>();
        app.init_asset_loader::<RonAssetLoader<StackingDef>>();
        app.insert_resource(DefRegistry::<StackingDef>::new());

        app.add_systems(
            PreUpdate,
            load_stacking_defs
                .run_if(resource_changed::<Assets<StackingDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 StackingDef
pub fn get_stacking_def(id: &StackingId, registry: &DefRegistry<StackingDef>) -> Option<&StackingDef> {
    registry.get(id)
}
```

### 注册生命周期

```
Load (stackings.ron) → Deserialize → Validate → Register (DefRegistry<StackingDef>) → Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | StackingId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `max_stacks >= 1` | 堆叠上限至少为 1 |
| V4 | StackingType::None 时 `max_stacks` 应为 1 | 不堆叠策略的上限为 1 |
| V5 | StackingType::Aggregate 时 `max_stacks >= 2` | 累加策略至少可叠 2 层 |
| V6 | StackingType::Replace 时 `max_stacks` 应为 1 | 替换策略的上限为 1 |
| V7 | StackingDef 不得引用任何 L2+ Def | L1 内容不可引用 Entity/Gameplay/World 层内容 |

---

## 5. RON 示例

```ron
// StackingDef 示例：不可堆叠（默认）
//
// 适用于眩晕、沉默、变身等唯一性控制效果。
(
    id: "stk:unstackable",
    name_key: "stacking.unstackable.name",
    description_key: "stacking.unstackable.desc",
    schema_version: 1,

    stacking_type: None,
    max_stacks: 1,
    overflow_behavior: IgnoreNew,

    tags: ["tag:stacking_control"],
)
```

```ron
// StackingDef 示例：累加堆叠
//
// 适用于中毒、灼烧、易伤等可叠加效果。
// 最多 5 层，异源可叠，层数变化时 Modifier 不重算（每层固定值）。
(
    id: "stk:additive",
    name_key: "stacking.additive.name",
    description_key: "stacking.additive.desc",
    schema_version: 1,

    stacking_type: Aggregate,
    max_stacks: 5,
    allow_cross_source: true,
    overflow_behavior: IgnoreNew,
    reapply_modifiers_on_stack: false,

    tags: ["tag:stacking_damage"],
)
```

```ron
// StackingDef 示例：刷新持续时间
//
// 适用于"续杯"类 Buff——每次应用刷新剩余时间但不增加强度。
(
    id: "stk:refresh",
    name_key: "stacking.refresh.name",
    description_key: "stacking.refresh.desc",
    schema_version: 1,

    stacking_type: RefreshDuration,
    max_stacks: 1,
    overflow_behavior: RefreshAndCap,

    tags: ["tag:stacking_defensive"],
)
```

```ron
// StackingDef 示例：替换（取最强）
//
// 适用于护盾、附魔等——新效果替换旧效果。
(
    id: "stk:strongest",
    name_key: "stacking.strongest.name",
    description_key: "stacking.strongest.desc",
    schema_version: 1,

    stacking_type: Replace,
    max_stacks: 1,
    overflow_behavior: IgnoreNew,

    tags: ["tag:stacking_shield"],
)
```
