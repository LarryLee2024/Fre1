---
id: capabilities.targeting.schema.v1
title: Targeting Schema — 目标选择数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition, runtime
replay-safe: true
---

# Targeting Schema — 目标选择数据架构

> **领域归属**: Capabilities — 行为表现层 | **依赖 Schema**: Tag, Condition | **定义依据**: `docs/02-domain/targeting_domain.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `TargetingDef` | Definition | 目标选择配置（类型、形状、范围、最大目标数） |
| `TargetData` | Runtime | 目标选择结果 |
| `TargetShape` | Definition | 影响区域形状枚举 |
| `TargetType` | Definition | 目标类别枚举 |

---

## 2. Problem

Targeting 是技能/效果与战场实体之间的「桥梁」——定义能力作用于哪些实体、以什么形状选择目标、受哪些限制。Schema 必须解决：
- TargetType（选择何种目标）与 TargetShape（以什么范围选择）的组合表达
- 范围计算所需的参数（半径/长度/宽度/弹射次数）
- 目标筛选的额外条件（排除自身、排除特定标签等）
- 选择结果的标准化数据结构

---

## 3. Schema Design

### 3.1 TargetingDef（Definition 层）

```rust
struct TargetingDef {
    /// 目标类别
    target_type: TargetType,

    /// 范围形状
    shape: TargetShape,

    /// 最大射程（网格单位，None = 无限制）
    range: Option<f32>,

    /// 最小射程（None = 无限制）
    min_range: Option<f32>,

    /// 最大目标数
    max_targets: u32,

    /// 是否允许选择施法者自身
    include_self: bool,

    /// 排除条件（满足此条件的目标不被选中）
    exclude_condition: Option<Condition>,

    /// 附加过滤条件（只选中满足此条件的目标）
    filter_condition: Option<Condition>,

    /// 是否需要视野
    require_los: bool,

    /// 是否忽略障碍物
    ignore_obstacles: bool,

    /// 优先级排序规则（多个可选目标时的自动选择）
    priority_rule: Option<PriorityRule>,

    /// 能否选择已死亡实体
    allow_dead_targets: bool,
}
```

### 3.2 TargetType（Definition 层）

```rust
enum TargetType {
    /// 自身（施法者）
    Self_,
    /// 友方（同阵营非自身）
    Ally,
    /// 敌方（对立阵营）
    Enemy,
    /// 已死亡的实体
    Dead,
    /// 中立实体
    Neutral,
    /// 所有实体（无差别）
    Any,
    /// 召唤物
    Summon,
    /// 小队全体
    Party,
    /// 自定义（由 Domain 定义过滤逻辑）
    Custom(String),
}
```

### 3.3 TargetShape（Definition 层）

```rust
enum TargetShape {
    /// 单体（单一目标）
    Single,
    /// 圆形区域（半径）
    Area {
        /// 半径（网格单位）
        radius: f32,
    },
    /// 直线（长度、宽度）
    Line {
        /// 长度
        length: f32,
        /// 宽度
        width: f32,
    },
    /// 锥形（角度、长度）
    Cone {
        /// 锥形长度
        length: f32,
        /// 张开角度（度）
        angle: f32,
    },
    /// 链式弹射（跳数、每跳最大距离）
    Chain {
        /// 总弹射次数（含首次目标）
        bounces: u32,
        /// 每跳最大距离
        bounce_range: f32,
        /// 是否可重复弹射同一目标
        allow_retarget: bool,
    },
    /// 爆炸/迸发（以目标格为中心的二次范围）
    Burst {
        /// 中心半径
        center_radius: f32,
        /// 扩散半径
        burst_radius: f32,
    },
    /// 墙体/连线（起点到终点的所有格子）
    Wall {
        /// 墙体长度
        length: f32,
        /// 墙体宽度
        width: f32,
    },
}
```

### 3.4 PriorityRule（Definition 层）

```rust
enum PriorityRule {
    /// 最近优先
    Nearest,
    /// 最远优先
    Farthest,
    /// 血量最低优先
    LowestHealth,
    /// 血量最高优先
    HighestHealth,
    /// 属性最高优先
    HighestAttribute(AttributeId),
    /// 属性最低优先
    LowestAttribute(AttributeId),
    /// 随机（基于确定性 RNG）
    Random,
    /// 自定义
    Custom(String),
}
```

### 3.5 TargetData（Runtime 层）

```rust
struct TargetData {
    /// 选中的实体列表
    entities: Vec<EntityTarget>,

    /// 选中的位置列表（用于区域技能的位置标记）
    positions: Vec<GridPosition>,

    /// 选择时的上下文
    context: TargetContext,

    /// 是否有合法目标
    has_valid_targets: bool,

    /// 选择用时（调试用）
    select_duration_ms: Option<f32>,
}

struct EntityTarget {
    /// 目标实体 ID
    entity_id: EntityId,
    /// 目标在范围内的位置
    position: GridPosition,
    /// 目标距离
    distance: f32,
    /// 选择优先级顺序
    selection_order: u32,
}

struct TargetContext {
    /// 施法者位置
    caster_position: GridPosition,
    /// 施法者阵营
    caster_faction: FactionId,
    /// 施法时帧号
    frame: u64,
}
```

### 3.6 TargetShapeParams（Definition 层 — 配置格式）

```yaml
# RON 配置示例 — 目标选择配置
TargetingDef:
  # 示例1: 单体近战攻击
  target_type: Enemy
  shape: Single
  range: 1.5
  max_targets: 1
  require_los: true
  priority_rule: Nearest

  # 示例2: 火球术范围
  target_type: Enemy
  shape:
    Area:
      radius: 2.0
  range: 10.0
  max_targets: 6
  require_los: true
  filter_condition:
    TagRequirement:
      mode: HasNone
      target_tags: ["tag_000030"]   # Tag.Immune.Fire

  # 示例3: 治疗术（单体友方）
  target_type: Ally
  shape: Single
  range: 5.0
  max_targets: 1
  require_los: true
  priority_rule: LowestHealth

  # 示例4: 连锁闪电
  target_type: Enemy
  shape:
    Chain:
      bounces: 3
      bounce_range: 4.0
      allow_retarget: false
  range: 8.0
  max_targets: 4
  require_los: false   # 弹射不需要视野
```

### 3.7 TargetingSnapshot（Persistence 层）

```rust
/// Targeting 结果本身是运行时数据，不需要常规持久化。
/// 存档时只保存 TargetData（由 AbilitySnapshot 间接保存）。
struct TargetingSnapshot {
    schema_version: u32,
    selected_entities: Vec<EntityId>,
    selected_positions: Vec<GridPosition>,
}
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `TargetingDef` | Definition | 是（Def 内嵌） | 是 | 内嵌在 AbilityDef/EffectDef |
| `TargetType` / `TargetShape` | Definition | 是（Def 内嵌） | 是 | 枚举定义 |
| `PriorityRule` | Definition | 是（Def 内嵌） | 是 | 自动选择规则 |
| `TargetData` | Runtime | 否（通过 Snapshot） | 否 | 选择结果 |
| `TargetContext` | Runtime | 否 | 否 | 选择时的环境数据 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → TagSchema | exclude_condition/filter_condition 中的 Tag 引用 |
| 依赖 | → ConditionSchema | exclude_condition/filter_condition |
| 被依赖 | ← AbilitySchema | AbilityDef.targeting |
| 被依赖 | ← EffectSchema | EffectApplication.targeting_override |
| 被依赖 | ← TacticalSchema | 网格/距离/掩体计算 |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | 形状参数合法 | Def 加载 | Area.radius > 0, Chain.bounces ≥ 1 |
| V2 | 最大目标数合法 | Def 加载 | max_targets ≥ 1 |
| V3 | 射程合法 | Def 加载 | min_range ≤ range |
| V4 | TargetType 与形状不矛盾 | Def 加载 | Single 形状时 max_targets = 1 |
| V5 | 筛选条件引用有效 | Def 加载 | filter_condition 中的 TagId/AttributeId 已注册 |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| 目标选择（自动） | 🟩 确定 | PriorityRule 确定选择（Random 由确定性 RNG 驱动） |
| 目标选择（手动） | 🟩 完全确定 | 手动选择的 Command 直接录制目标 EntityId |
| 射程/视野校验 | 🟩 确定 | 网格数据和实体位置确定 |
| 校验结果 | 🟩 完全确定 | 纯函数：实体状态 + 环境 → 合法/非法 |

---

## 8. Save Compatibility

Targeting 定义嵌入在 AbilityDef 中存档。运行时 TargetData 通过 AbilitySnapshot 间保存。选择时的环境数据（TargetContext）不持久化，读档后重新计算。

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 新增 TargetShape variant | 新增枚举 variant，旧 Def 不受影响 |

---

## 10. Future Extension

- **目标记忆**: 技能对同一目标的多次选择产生累积效果（如「连续攻击同一目标伤害递增」）
- **复合形状**: 多个 TargetShape 组合（如锥形 + 爆炸的组合区域）
- **预测性高亮**: Targeting 系统提前计算并标记所有合法目标（用于 UI 高亮）

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| 目标数据不一致 | TargetData 包含已销毁的实体 | 使用 target 前校验 EntityId 有效性 |
| 形状计算复杂 | 六角网格的锥形/墙体算法复杂 | 抽象 GridTargeting trait，支持注入不同实现 |
| 大量目标选择性能 | Burst 技能在拥挤战场上选择大量目标 | 设 max_targets 硬上限，超过时按优先级截断 |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| Logic/Presentation Separation | ✅ | Targeting 只做数据选择，不做表现 |
| Replay First | ✅ | 目标选择结果确定（手动由 Command 确定，自动由 RNG 确定） |
