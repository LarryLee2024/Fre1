---
id: capabilities.stacking.schema.v1
title: Stacking Schema — 堆叠规则数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition
replay-safe: true
---

# Stacking Schema — 堆叠规则数据架构

> **领域归属**: Capabilities — 行为表现层 | **依赖 Schema**: Effect | **定义依据**: `docs/02-domain/capabilities/stacking_domain.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `StackingConfig` | Definition | 堆叠策略定义（类型、上限、溢出处理） |
| `StackingState` | Instance | 堆叠运行时状态（当前计数） |
| `StackIdentity` | Definition | 堆叠标识判定规则（同源/异源） |

---

## 2. Problem

堆叠规则管理「同一效果多次作用时如何叠加」——能否叠加、如何叠加、叠加上限是多少。Schema 必须解决：
- 四种堆叠策略（None/Aggregate/Refresh/Replace）的数据表达
- 同源/异源堆叠的判定规则
- 堆叠上限与溢出处理
- 堆叠层数变化时 Modifier 的重新计算

---

## 3. Schema Design

### 3.1 StackingConfig（Definition 层）

已在 EffectSchema 中定义，此处列出核心结构供 Stacking 领域管理：

```rust
struct StackingConfig {
    /// 堆叠策略
    stacking_type: StackingType,

    /// 最大堆叠层数
    max_stacks: u32,

    /// 是否允许异源堆叠
    allow_cross_source: bool,

    /// 溢出处理
    overflow_behavior: OverflowBehavior,

    /// 层数变化时是否重算 Modifier
    reapply_modifiers_on_stack: bool,
}
```

### 3.2 StackingType（Definition 层）

```rust
enum StackingType {
    /// 不堆叠——新实例被忽略
    None,
    /// 累加层数——层数叠加，受 max_stacks 限制
    Aggregate,
    /// 刷新持续时间——重置剩余时间，层数不变
    RefreshDuration,
    /// 替换——新实例替换旧实例（按优先级或数值）
    Replace,
}
```

### 3.3 StackIdentity（Definition 层）

```rust
/// 堆叠标识——用于判定两个 Effect 是否属于同一堆叠。
struct StackIdentity {
    /// 匹配的 EffectDefId
    effect_def_id: EffectDefId,

    /// 来源实体 ID（用于同源判定）
    source_entity: EntityId,

    /// 来源能力 ID（可选，进一步细化同源判定）
    source_ability: Option<AbilityDefId>,

    /// 分组标签（可选，按 Tag 分组堆叠）
    group_tag: Option<TagId>,
}

/// 堆叠匹配判定结果
enum StackMatchResult {
    /// 完全匹配（同 EffectDef + 同源）— 堆叠判定
    FullMatch,
    /// 类型匹配但异源 — 根据 allow_cross_source 决定
    CrossSource,
    /// 分组匹配 — 按 group_tag 堆叠
    GroupMatch,
    /// 不匹配 — 各自独立
    NoMatch,
}
```

### 3.4 StackingState（Instance 层）

```rust
struct StackingState {
    /// 当前堆叠层数
    stack_count: u32,

    /// 最大上限（来自 StackingConfig）
    max_stacks: u32,

    /// 堆叠历史（所有活跃的同堆叠实例 ID）
    stack_members: Vec<EffectInstanceId>,

    /// 每层对应的 Modifier 列表（如果 reapply_modifiers_on_stack = true）
    per_layer_modifiers: Vec<Vec<ModifierInstanceId>>,

    /// 堆叠类型
    stacking_type: StackingType,
}
```

### 3.5 StackingDecision（Runtime 层 — 瞬时）

```rust
/// 堆叠判定的结果。
enum StackingDecision {
    /// 拒绝——新实例不应用（None 类型或 Replace 但旧≥新）
    Reject,
    /// 累加——层数增加
    Accumulate {
        new_stack_count: u32,
        added_layers: u32,
    },
    /// 刷新——重置持续时间
    Refresh {
        refreshed_instance_id: EffectInstanceId,
        new_duration: u64,
    },
    /// 替换——移除旧的，应用新的
    Replace {
        replaced_instance_id: EffectInstanceId,
    },
}
```

### 3.6 StackingRuleConfig（Definition 层 — 配置格式）

```yaml
# RON 配置示例 — 堆叠规则配置
StackingConfig:
  # 示例1: 不可堆叠（默认）
  stacking_type: None

  # 示例2: 可堆叠，最多 5 层，同源累加
  stacking_type: Aggregate
  max_stacks: 5
  allow_cross_source: false
  overflow_behavior: IgnoreNew
  reapply_modifiers_on_stack: true

  # 示例3: 异源可堆叠（不同施法者的同效果叠加）
  stacking_type: Aggregate
  max_stacks: 3
  allow_cross_source: true
  overflow_behavior: RemoveOldest
  reapply_modifiers_on_stack: false

  # 示例4: 刷新持续时间
  stacking_type: RefreshDuration
  max_stacks: 1

  # 示例5: 替换（取最大值）
  stacking_type: Replace
  max_stacks: 1
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `StackingConfig` | Definition | 是（EffectDef 内嵌） | 是 | 堆叠策略定义 |
| `StackingState` | Instance | 是（通过 EffectSnapshot） | 否 | ECS Component |
| `StackIdentity` | Runtime | 否 | 否 | 判定时的瞬时数据 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → EffectSchema | StackingConfig 引用 EffectDef |
| 依赖 | → ModifierSchema | per_layer_modifiers 追踪 |
| 被依赖 | ← EffectSchema | Effect 应用时调用 Stacking 判定 |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | max_stacks ≥ 1 | Def 加载 | 堆叠上限至少为 1 |
| V2 | Aggregate 类型必须设上限 | Def 加载 | Aggreate 类型 max_stacks 必须 ≥ 2 |
| V3 | Replace 类型 max_stacks = 1 | Def 加载 | Replace 类型最多 1 层 |
| V4 | 层数不超上限 | 运行时 | stack_count ≤ max_stacks |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| 堆叠判定 | 🟩 完全确定 | 基于 EffectDef + Source，纯函数判定 |
| 层数叠加 | 🟩 确定 | stack_count 由确定性事件驱动 |
| 溢出处理 | 🟩 完全确定 | Clear 策略确定 |

---

## 8. Save Compatibility

堆叠配置（StackingConfig）随 EffectDef 保存。堆叠状态（StackingState）随 EffectSnapshot 保存。

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |

---

## 10. Future Extension

- **条件堆叠**: 只有在特定 Condition 满足时才累加层数
- **层数衰减**: 每回合层数自然减少（类似 D&D 的「豁免结束效果」）
- **交叉堆叠**: 不同 EffectDef 之间的交叉堆叠（如「灼烧+冰冻→蒸汽」）

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| 堆叠与 Effect 生命周期耦合 | Effect 移除时层数减少顺序复杂 | Effect 被移除后主动通知 Stacking 递减 |
| 异源堆叠难以追溯 | 多个施法者的同效果交叉管理复杂 | clear 分来源的索引 |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| 所有堆叠行为归属 Stacking | ✅ | 禁止在 Effect/Ability 中定义 max_stack |
| Replay First | ✅ | 堆叠判定确定 |
