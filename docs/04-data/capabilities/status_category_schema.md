---
id: capabilities.status-category.schema.v1
title: StatusCategory Schema — 状态类别数据架构
status: stable
owner: data-architect
created: 2026-06-20
updated: 2026-06-20
layer: definition
replay-safe: true
---

# StatusCategory Schema — 状态类别数据架构

> **领域归属**: Capabilities — 行为表现层 | **依赖 Schema**: 无（L0 自包含） | **定义依据**: `docs/03-content/definitions/vocabulary/status-category-def.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `StatusCategoryDef` | Definition | 状态类别的静态定义（驱散分组、有益性标识） |
| `DispelGroup` | Definition | 驱散分组枚举（Physical/Magical/None） |

---

## 2. Problem

StatusCategory 定义了游戏中 Buff/Debuff/StatusEffect 的分类标识，用于免疫系统、驱散系统和 UI 分类过滤。Schema 必须解决：

- 状态类别如何按驱散方式分组（物理驱散 / 魔法驱散 / 不可驱散）
- 状态类别如何区分有益/减益（用于 UI 显示和行为逻辑）
- 与 BuffDef.category 引用的数据格式

---

## 3. Schema Design

### 3.1 StatusCategoryDef（Definition 层）

```rust
/// 状态类别定义——Buff/Debuff/StatusEffect 的分类标识。
///
/// 来源：`docs/03-content/definitions/vocabulary/status-category-def.md` §2
struct StatusCategoryDef {
    /// 状态类别唯一标识（前缀: `status:`）
    id: StatusCategoryId,

    /// 状态类别名称本地化 Key
    name_key: LocalizationKey,

    /// 状态类别描述本地化 Key
    desc_key: LocalizationKey,

    /// Schema 版本号
    schema_version: u32,

    /// 驱散分组——此类别状态如何被驱散
    dispel_group: DispelGroup,

    /// 是否是有益状态
    /// true = 增益（Buff），false = 减益（Debuff）
    is_beneficial: bool,

    /// 图标 Key（可选，用于 Buff 栏的默认图标覆盖）
    icon_key: Option<String>,
}

/// 驱散分组——决定此类别状态可被哪种驱散手段移除
enum DispelGroup {
    /// 物理驱散——通过"净化"、"治疗"等物理/自然手段驱散
    Physical,
    /// 魔法驱散——通过"驱散魔法"、"解除诅咒"等魔法手段驱散
    Magical,
    /// 不可驱散——无法通过任何常规驱散手段移除
    None,
}
```

### 3.2 免疫系统数据结构（L1 EffectDef 引用）

```rust
// L1 EffectDef — 驱散效果配置
// 定义在 `docs/04-data/capabilities/effect_schema.md` §3.8
struct DispelEffectConfig {
    /// 目标驱散类别——只驱散匹配这些类别的状态
    target_categories: Vec<StatusCategoryId>,

    /// 可选：目标驱散分组——按 DispelGroup 批量驱散
    target_dispel_groups: Option<Vec<DispelGroup>>,

    /// 最多驱散数量
    max_dispel_count: u32,

    /// 是否区分有益/减益
    include_beneficial: bool,
}
```

### 3.3 运行时 ImmnuityCheck（Instance 层）

```rust
/// 免疫检查——检查实体是否免疫某类别状态。
/// 运行时由 Condition 系统调用。
struct ImmnuityCheck {
    /// 检查的实体
    entity_id: EntityId,

    /// 要检查的状态类别
    category_id: StatusCategoryId,

    /// 免疫标签列表（预先计算，用于 O(1) 匹配）
    immune_categories: Vec<StatusCategoryId>,
}
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `StatusCategoryDef` | Definition | 是（配置文件） | 是 | 状态类别定义 |
| `DispelGroup` | Definition | 是（Def 内嵌） | 是 | 驱散分组枚举 |
| `DispelEffectConfig` | Definition | 是（EffectDef 内嵌） | 是 | 驱散效果配置 |
| `ImmnuityCheck` | Instance | 否（瞬时计算） | 否 | 运行时免疫检查 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 被依赖 | ← EffectSchema | 驱散效果引用 StatusCategoryId |
| 被依赖 | ← BuffSchema | BuffDef.category 引用 StatusCategoryId |
| 被依赖 | ← ConditionSchema | 免疫条件引用 StatusCategoryId |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | ID 格式合法 | Def 加载 | 必须匹配 `^status:[a-z][a-z0-9_]+$` |
| V2 | `dispel_group` 有效 | Def 加载 | 必须是 DispelGroup 的三个变体之一 |
| V3 | `is_beneficial` 为 bool | Def 加载 | 必须为 true 或 false |
| V4 | 无跨 Def 引用 | Def 加载 | StatusCategoryDef 是 L0 Def，禁止引用其他 Def |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| 状态类别定义 | 🟩 完全确定 | Def 加载时静态确定 |
| 驱散判断 | 🟩 完全确定 | dispel_group 和 is_beneficial 确定 |

---

## 8. Save Compatibility

StatusCategoryDef 是纯 Definition 数据，通过配置文件加载，不参与运行时存档。

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 新增 DispelsGroup variant | 新增枚举 variant，旧 Def 不受影响 |

---

## 10. Future Extension

- **状态类别层级**: 支持父-子类别继承（如「魔法减益」包含「中毒」和「诅咒」）
- **条件性驱散**: 驱散效果支持条件判断（如"仅在目标生命值低于 50% 时驱散"）
- **驱散优先级**: 多个驱散效果同时命中时，按优先级选择生效

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| 分类粒度过粗 | Physical/Magical 二分无法覆盖特殊驱散方式 | 新增 None 组 + 允许 target_categories 精确指定 |
| 有益/减益分类争议 | 某些状态既非增益也非减益（如"标记"） | is_beneficial 配合自定义 DispelGroup 表达中性状态 |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| L0 自包含 | ✅ | StatusCategoryDef 不引用任何其他 Def 类型 |
| Def-Instance 分离 | ✅ | StatusCategoryDef 为纯 Definition |
| 分类是基础词汇 | ✅ | 状态类别是描述性元数据，不是可执行逻辑 |
