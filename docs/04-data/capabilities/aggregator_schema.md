---
id: capabilities.aggregator.schema.v1
title: Aggregator Schema — 聚合器数据架构
status: draft
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: instance, runtime
replay-safe: true
---

# Aggregator Schema — 聚合器数据架构

> **领域归属**: Capabilities — 聚合层 | **依赖 Schema**: Attribute, Modifier | **定义依据**: `docs/02-domain/aggregator_domain.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `CalcPipeline` | Definition | 计算管线阶段定义（Add→Multiply→Override→Clamp） |
| `CalcStage` | Definition | 单个计算阶段的定义（运算类型、排序规则） |
| `AggregatorState` | Instance | 实体的聚合状态（ECS Component），含 Dirty 标记和缓存 |
| `AggregationResult` | Runtime | 单次聚合计算结果（中间产物，不持久化） |
| `AggregationSnapshot` | Persistence | 聚合快照（用于回滚和回放校验） |

---

## 2. Problem

Aggregator 是 Attribute + Modifier 的「计算引擎」——将 BaseValue 经过四阶段运算转化为 FinalValue。Schema 必须解决：
- 四阶段管线的严格顺序保证（Add→Multiply→Override→Clamp）
- Dirty 标记机制（惰性重算，一帧内合并多次变更）
- 聚合结果的缓存与失效
- 快照机制（回放校验点）
- 循环检测（属性 A → 属性 B → 属性 A 的触发循环）

---

## 3. Schema Design

### 3.1 CalcStage（Definition 层）

```rust
/// 计算阶段定义。
/// 严格按枚举顺序执行：Add → Multiply → Override → Clamp
enum CalcStage {
    /// 加法阶段: Sum(所有 Add 类型 Modifier 的值)
    Add,
    /// 乘法阶段: Product(所有 Multiply 类型 Modifier 的值)
    /// 注意：乘法叠加是连乘而非加法
    Multiply,
    /// 覆盖阶段: 取优先级最高的 Override Modifier 的值
    Override,
    /// 钳制阶段: 限制在 [MinValue, MaxValue] 范围内
    Clamp,
}
```

### 3.2 CalcPipeline（Definition 层 — 管线配置）

```rust
/// 计算管线配置。
/// 绝大多数属性使用默认管线，特殊属性可覆盖。
struct CalcPipeline {
    /// 属性 ID（配置的是哪个属性的计算管线）
    attribute_id: AttributeId,

    /// 启用哪些阶段（默认全启用）
    enabled_stages: Vec<CalcStage>,

    /// 各阶段内部 Modifier 的排序方向
    /// true: 优先级数值越小越先执行（默认）
    priority_ascending: bool,

    /// Clamp 边界（可覆盖 AttributeDefinition 的 min/max）
    clamp_override: Option<(f32, f32)>,

    /// 是否启用循环检测
    cycle_detection: bool,
}

/// 默认管线（适用于所有标准属性）
const DEFAULT_PIPELINE: CalcPipeline = CalcPipeline {
    enabled_stages: vec![CalcStage::Add, CalcStage::Multiply, CalcStage::Override, CalcStage::Clamp],
    priority_ascending: true,
    clamp_override: None,
    cycle_detection: true,
};
```

### 3.3 AggregatorState（Instance 层 — ECS Component）

```rust
struct AggregatorState {
    /// 属性 → 缓存的 FinalValue 映射
    cached_values: HashMap<AttributeId, f32>,

    /// Dirty 标记集合（哪些属性需要重算）
    dirty_attributes: HashSet<AttributeId>,

    /// 上次聚合计算的帧号（用于去重和性能诊断）
    last_aggregation_frame: u64,

    /// 聚合计数器（聚合次数，用于诊断和回放校验）
    aggregation_count: u64,
}
```

### 3.4 AggregationResult（Runtime 层 — 瞬时数据）

```rust
/// 单次聚合计算的完整结果。
/// 运行时中间产物，不持久化，用于调试和事件载荷。
struct AggregationResult {
    /// 计算时间
    frame: u64,

    /// 目标属性
    attribute_id: AttributeId,

    /// 各阶段的中间值（用于调试和审计）
    stage_values: HashMap<CalcStage, f32>,

    /// 参与计算的 Modifier 列表（含被 Override 抑制的）
    participating_modifiers: Vec<ModifierInstanceId>,

    /// 是否被 Override 抑制
    was_overridden: bool,

    /// 最终值
    final_value: f32,
}
```

### 3.5 AggregationSnapshot（Persistence 层）

```rust
struct AggregationSnapshot {
    /// 存档版本
    schema_version: u32,

    /// 快照时间（帧计数）
    frame: u64,

    /// 实体 ID
    entity_id: EntityId,

    /// 所有属性的最终值快照
    /// attribute_id → (base_value, final_value)
    attribute_values: HashMap<AttributeId, (f32, f32)>,

    /// 当前所有活跃 Modifier 列表（完整状态）
    active_modifiers: Vec<ModifierData>,

    /// 快照校验哈希（用于回放校验）
    checksum: u64,
}
```

### 3.6 CalcPipelineConfig（Definition 层 — 配置格式）

```yaml
# RON 配置示例 — 特殊管线覆盖
# 注意：大多数属性不需要单独配置，使用 DEFAULT_PIPELINE 即可
CalcPipelineConfig:
  overrides:
    # 力量属性的标准管线（使用默认）
    - attribute_id: "attr_000001"
      # 无覆盖，使用默认

    # 生命值上限：禁用 Override 阶段（不允许外力覆盖生命上限）
    - attribute_id: "attr_000020"
      enabled_stages: ["Add", "Multiply", "Clamp"]

    # 某些特殊状态属性：跳过 Add/Multiply，直接 Override
    - attribute_id: "attr_000099"
      enabled_stages: ["Override", "Clamp"]
      clamp_override: [0.0, 1.0]
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `CalcStage` | Definition | 代码内枚举 | 否 | 枚举在代码中定义 |
| `CalcPipeline` | Definition | 可选（配置覆盖） | 是 | 默认值在代码中 |
| `AggregatorState` | Instance | 否（通过 Snapshot） | 否 | ECS Component |
| `AggregationResult` | Runtime | 否 | 否 | 瞬时中间产物 |
| `AggregationSnapshot` | Persistence | 是（存档） | 否 | 回放校验点 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → AttributeSchema | 获取 BaseValue、MinValue、MaxValue |
| 依赖 | → ModifierSchema | 按属性 ID 查询所有活跃 Modifier |
| 被依赖 | ← ExecutionSchema | 伤害计算前确保属性值是最新的 |
| 被依赖 | ← EffectSchema | Effect 应用/移除后触发重算 |
| 被依赖 | ← StackingSchema | 堆叠变更后触发重算 |
| 被依赖 | ← CombatSchema | 战斗中的属性读取 |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | 阶段顺序不可逆 | 运行时 | Add → Multiply → Override → Clamp，跳过报错 |
| V2 | 乘法非加法 | 运行时 | Multiply 阶段使用 product 而非 sum 运算 |
| V3 | Override 互斥 | 运行时应答 | 同一属性多个 Override → 最高优先级生效 |
| V4 | Clamp 边界法定 | 配置加载 | clamp_override 的 min ≤ max |
| V5 | 循环检测 | 运行时 | A 的重算引起 B 的重算引起 A 的重算 → 中断并报 CycleDetected |
| V6 | Dirty → Clean 幂等 | 运行时 | 同一属性一帧内被多次标记 Dirty 只触发一次重算 |
| V7 | 快照一致性 | 快照创建 | 快照时所有属性必须为 Clean 状态 |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| 聚合计算 | 🟩 完全确定 | BaseValue + ModifierList → FinalValue，纯确定性 |
| Dirty→Clean 转换 | 🟩 确定 | 帧级批量合并，帧号确定 |
| 快照比对 | 🟩 完全确定 | checksum 确保快照一致性 |
| 循环检测 | 🟩 确定 | 循环路径确定，中断行为确定 |

**结论**: Aggregator Schema 是天然 Replay-safe 的。管线计算是纯函数，不含任何随机或外部状态。

---

## 8. Save Compatibility

| 场景 | 兼容性 | 版本策略 |
|------|--------|----------|
| 基础存档 | 🟩 | Save v1: 存 AggregationSnapshot |
| 新增计算阶段 | 🟨 需要迁移 | 新增阶段改变管线顺序 → 全量重新计算所有属性值 |
| Clamp 边界变化 | 🟩 运行时重算 | 旧存档加载后按新 Clamp 边界重算 |
| 管线配置变化 | 🟩 运行时重算 | 旧存档加载后按新管线重新计算 |

**关键设计**: 存档不直接存储 FinalValue（除快照外）。单位加载后 AttributeValue 从存档恢复 base/current，然后由 Aggregator 按当前管线配置重算 FinalValue。这使得管线配置的演化不破坏存档兼容性。

快照中的 checksum 用于回放校验，不是存档恢复的依据（校验失败只告警不阻止加载）。

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 新增计算阶段（如 BasePercent 阶段） | 新增 stage 枚举值，旧管线自动适配默认值 |
| v3（未来） | 引入多线程并行聚合 | AggregatorState 增加并行标记，计算逻辑不变 |

---

## 10. Future Extension

- **条件聚合**: 支持 Modifier 仅在满足特定 TagQuery 时参与计算（减少不必要的 Modifier 移除/重加）
- **逐帧插值**: 支持属性值在帧之间平滑过渡（如逐渐减少的减伤效果）
- **聚合钩子**: 在管线的每个阶段插入自定义计算（如「所有加法修改器取整」）
- **批量快照**: 支持一次对多个实体拍摄批量快照（用于战斗复盘）

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| 循环触发（A→B→A） | 无限递归，帧计算爆炸 | 循环检测：检测到同帧内同一属性两次触发 → 中断并报警 |
| 大量属性连续 Dirty | 一帧内重算数百属性导致卡顿 | 批量合并 + 帧级延迟重算（本帧 Dirty，下帧初执行） |
| 快照膨胀 | 大型战斗每秒快照导致内存溢出 | 仅关键节点（战斗开始/结束/关键决策）拍摄快照 |
| Modifier 数据变化未通知 Aggregator | 缓存 stale，读取到过期值 | 强制所有 Modifier 增删必须通过 ModifierContainer API，API 内部自动发布事件 |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| 三层分离（Def→Spec→Instance） | ✅ | CalcPipeline → AggregatorState → AggregationResult |
| Replay First | ✅ | 纯函数管线，确定性计算 |
| Composition Over Inheritance | ✅ | 通过 Modifier 组合影响属性，非继承 |
| 宪法 §16.3 Aggregator 管线 | ✅ | Add→Multiply→Override→Clamp 顺序合规 |
| 宪法 §16.4 属性快照 | ✅ | AggregationSnapshot 支持回滚和校验 |
