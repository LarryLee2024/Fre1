---
id: capabilities.effect.schema.v1
title: Effect Schema — 效果数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-20
layer: definition, instance
replay-safe: true
---

# Effect Schema — 效果数据架构

> **领域归属**: Capabilities — 行为表现层 | **依赖 Schema**: Tag, Attribute, Modifier, Spec, Condition, GameplayContext, Execution | **定义依据**: `docs/02-domain/capabilities/effect_domain.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `EffectDef` | Definition | 效果的完整静态定义 |
| `EffectDuration` | Definition | 持续时间类型（Instant/Duration/Infinite） |
| `EffectPeriod` | Definition | 周期 Tick 定义 |
| `EffectInstance` | Instance | 效果的运行时实例 |
| `ActiveEffectContainer` | Instance | 实体上的活跃效果容器 |

---

## 2. Problem

Effect 是能力系统所有「结果」的载体——伤害、治疗、Buff、Debuff、地形变化、召唤物，一切最终都表现为 Effect。Schema 必须解决：
- EffectDef 的完整数据结构（Modifier 列表、Tag 变更、持续/周期参数）
- 四阶段生命周期（Applying→Active→Expiring→Removed）的 Instance 状态追踪
- 周期 Tick 的计时和触发
- Effect 来源的可追溯性
- 与 Execution 和 Stacking 的衔接

---

## 3. Schema Design

### 3.1 EffectDef（Definition 层）

```rust
struct EffectDef {
    /// 效果唯一标识
    id: EffectDefId,

    /// 效果名称本地化 Key
    name_key: LocalizationKey,

    /// 效果描述本地化 Key
    desc_key: LocalizationKey,

    /// 效果图标 Key
    icon_key: Option<String>,

    /// 持续时间类型
    duration: EffectDuration,

    /// 周期 Tick（仅 Duration 类型有效）
    period: Option<EffectPeriod>,

    /// 效果携带的修改器列表（应用时注册到目标属性）
    modifiers: Vec<ModifierConfig>,

    /// 效果授予的标签（应用时添加到目标实体）
    granted_tags: Vec<TagId>,

    /// 效果需要的标签（目标必须拥有效果才能生效）
    required_tags: Option<Vec<TagId>>,

    /// 目标不能拥有的标签（否则效果应用失败，用于免疫检查）
    ignored_tags: Option<Vec<TagId>>,

    /// 效果移除时清理的标签
    removed_tags: Option<Vec<TagId>>,

    /// 应用此效果时，移除目标上具有这些标签的其他效果
    remove_effects_with_tags: Option<Vec<TagId>>,

    /// 应用条件（可选，满足此条件效果才能应用）
    ///
    /// ConditionId 引用 ConditionDef（Content 层设计）
    application_condition: Option<ConditionId>,

    /// 效果叠加策略（默认不堆叠）
    stacking: StackingConfig,

    /// 效果类型分类
    effect_category: EffectCategory,

    /// 关联的执行计算（可选，Instant 类效果需要）
    execution: Option<ExecutionConfig>,

    /// 视觉表现信号
    cues: Vec<CueBinding>,

    /// 可视性
    visible: bool,
    /// 是否可被驱散
    dispellable: bool,
    /// 优先级（用于显示排序）
    display_priority: u8,
}

enum EffectCategory {
    Buff,        // 增益
    Debuff,      // 减益
    Damage,      // 伤害
    Heal,        // 治疗
    Shield,      // 护盾
    Control,     // 控制（眩晕、沉默）
    Terrain,     // 地形变化
    Summon,      // 召唤
    Custom(String),
}
```

### 3.2 EffectDuration（Definition 层）

```rust
enum EffectDuration {
    /// 瞬时效果（立即执行，无持续阶段）
    Instant,

    /// 持续时间（回合数或帧数）
    HasDuration {
        /// 持续回合数
        turns: u32,
        /// 持续帧数（可选，与 turns 二选一或叠加）
        frames: Option<u64>,
        /// 持续时间计算方式
        calculation: DurationCalculation,
    },

    /// 无限期（需要显式移除）
    Infinite,
}

enum DurationCalculation {
    /// 固定值
    Fixed,
    /// 基于等级
    PerLevel { base: u32, per_level: u32 },
    /// 基于属性
    AttributeBased { attribute_id: AttributeId, multiplier: f32 },
}
```

### 3.3 EffectPeriod（Definition 层）

```rust
struct EffectPeriod {
    /// 间隔回合数
    interval_turns: u32,

    /// 间隔帧数（可选）
    interval_frames: Option<u64>,

    /// 首次 Tick 延迟
    initial_delay: Option<u64>,

    /// 最大 Tick 次数（None = 不限制）
    max_ticks: Option<u32>,

    /// Tick 时执行的计算
    tick_execution: Option<ExecutionConfig>,
}
```

### 3.4 StackingConfig（Definition 层）

```rust
struct StackingConfig {
    /// 堆叠类型
    stacking_type: StackingType,

    /// 最大堆叠层数
    max_stacks: u32,

    /// 是否允许异源堆叠（不同来源的同效果）
    allow_cross_source: bool,

    /// 堆叠超出上限时的处理
    overflow_behavior: OverflowBehavior,

    /// 堆叠层数变化时是否重新计算 Modifier
    /// true: 每层叠加/减少时重新注册所有 Modifier（层数×值）
    /// false: 只记录层数，Modifier 不随层数变化
    reapply_modifiers_on_stack: bool,
}

enum OverflowBehavior {
    /// 忽略新实例（保持上限层数）
    IgnoreNew,
    /// 移除最早层（FIFO）
    RemoveOldest,
    /// 刷新持续时间并保持上限
    RefreshAndCap,
}
```

### 3.5 CueBinding（Definition 层）

```rust
struct CueBinding {
    /// 触发时机
    cue_tag: CueTag,
    /// 要触发的 Cue ID
    cue_def_id: CueDefId,
    /// 延迟（帧），延迟触发
    delay_frames: Option<u64>,
}
```

### 3.6 EffectInstance（Instance 层 — ECS Component）

```rust
struct EffectInstance {
    /// 实例唯一标识
    instance_id: EffectInstanceId,

    /// 关联的 EffectSpec ID
    spec_id: SpecId,

    /// EffectDef ID
    def_id: EffectDefId,

    /// 当前阶段
    stage: EffectStage,

    /// 来源实体
    source_entity: EntityId,

    /// 目标实体
    target_entity: EntityId,

    /// 剩余持续回合数
    remaining_turns: i64,

    /// 已存活帧数
    elapsed_frames: u64,

    /// 总持续帧数
    total_duration_frames: Option<u64>,

    /// 周期 Tick 状态
    tick_state: Option<TickState>,

    /// 当前堆叠层数
    stack_count: u32,

    /// 是否暂停（如目标处于石化/时间停止状态）
    paused: bool,

    /// 实例创建帧号
    created_at_frame: u64,

    /// 应用时的上下文
    gameplay_context: Option<GameplayContextData>,
}

enum EffectStage {
    Applying,
    Active,
    Expiring,
    Removed,
}

struct TickState {
    /// 已触发的 Tick 次数
    tick_count: u32,
    /// 距下次 Tick 的剩余帧数
    remaining_frames: u64,
    /// 总 Tick 上限
    max_ticks: Option<u32>,
}
```

### 3.7 ActiveEffectContainer（Instance 层 — ECS Component）

```rust
struct ActiveEffectContainer {
    /// 所有活跃的效果实例
    effects: Vec<EffectInstance>,

    /// 按来源分组的索引（用于来源追溯和批量移除）
    by_source: HashMap<EntityId, Vec<EffectInstanceId>>,

    /// 按效果 Def 分组的索引（用于堆叠判定）
    by_def: HashMap<EffectDefId, Vec<EffectInstanceId>>,

    /// 效果槽位上限
    max_effects: u32,
}
```

### 3.8 EffectDefConfig（Definition 层 — 配置格式）

```yaml
# RON 配置示例 — 效果定义
EffectDefConfig:
  effects:
    # 示例1: 即时火焰伤害
    - id: "eff_000001"
      name_key: "effect.eff_000001.name"
      desc_key: "effect.eff_000001.desc"
      duration: Instant
      effect_category: Damage
      modifiers:
        - op: Add
          target_attribute: "attr_000030"   # 当前生命值
          value:
            Fixed: -25.0
          priority: 50
      execution:
        execution_type:
          Damage:
            formula_id: "dnd_5e_damage"
            damage_type: ["tag_000012"]   # DamageType.Elemental.Fire
            damage_dice:
              count: 8
              sides: 6
            can_critical: true
      cues:
        - cue_tag: OnApply
          cue_def_id: "cue_000001"  # 火焰爆炸特效
        - cue_tag: OnRemove
          cue_def_id: "cue_000002"  # 火焰消散

    # 示例2: 持续中毒（DoT）
    - id: "eff_000010"
      name_key: "effect.eff_000010.name"
      desc_key: "effect.eff_000010.desc"
      duration:
        HasDuration:
          turns: 3
          calculation: Fixed
      period:
        interval_turns: 1
        tick_execution:
          execution_type:
            Damage:
              formula_id: "dnd_5e_poison_damage"
              damage_type: ["tag_000015"]   # DamageType.Elemental.Poison
              damage_dice:
                count: 1
                sides: 4
      modifiers:
        - op: Add
          target_attribute: "attr_000002"   # 敏捷
          value:
            Fixed: -2.0
          priority: 50
      stacking:
        stacking_type: Aggregate
        max_stacks: 5
        overflow_behavior: IgnoreNew
      dispellable: true
      cues:
        - cue_tag: OnApply
          cue_def_id: "cue_000010"   # 中毒绿色闪光
        - cue_tag: OnTick
          cue_def_id: "cue_000011"   # 毒伤数字
        - cue_tag: OnRemove
          cue_def_id: "cue_000012"   # 净化光效
```

### 3.9 EffectSnapshot（Persistence 层）

```rust
struct EffectSnapshot {
    schema_version: u32,
    entity_id: EntityId,

    /// 所有活跃效果的快照
    active_effects: Vec<EffectInstanceSnapshot>,
}

struct EffectInstanceSnapshot {
    instance_id: EffectInstanceId,
    spec_id: SpecId,
    def_id: EffectDefId,
    stage: EffectStage,
    source_entity: EntityId,
    remaining_turns: i64,
    elapsed_frames: u64,
    total_duration_frames: Option<u64>,
    tick_state: Option<TickState>,
    stack_count: u32,
    created_at_frame: u64,
}
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `EffectDef` | Definition | 是（配置文件） | 是 | 效果定义 |
| `EffectDuration` / `EffectPeriod` | Definition | 是（Def 内嵌） | 是 | 持续/周期参数 |
| `StackingConfig` | Definition | 是（Def 内嵌） | 是 | 堆叠配置 |
| `CueBinding` | Definition | 是（Def 内嵌） | 是 | 表现绑定 |
| `EffectInstance` | Instance | 是（通过 Snapshot） | 否 | ECS Component |
| `ActiveEffectContainer` | Instance | 是（通过 Snapshot） | 否 | 容器 |
| `EffectSnapshot` | Persistence | 是（存档） | 否 | 存档格式 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → ExecutionSchema | period.tick_execution, execution |
| 依赖 | → ModifierSchema | modifiers 引用 ModifierConfig |
| 依赖 | → TagSchema | granted_tags, required_tags, ignored_tags, removed_tags, remove_effects_with_tags |
| 依赖 | → ConditionSchema | application_condition |
| 依赖 | → StackingSchema | stacking 配置 |
| 依赖 | → CueSchema | cues 引用 CueDefId |
| 依赖 | → GameplayContextSchema | 效果应用上下文 |
| 被依赖 | ← AbilitySchema | Ability.effects 引用 EffectDef |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | 来源可追溯 | Instance 创建 | source_entity 必须有效 |
| V2 | Condition 先于应用 | 运行时 | Applying 阶段必须先检查 application_condition |
| V3 | 持续期间非负 | 运行时 | remaining_turns ≥ 0 |
| V4 | 移除时 Modifier 回退 | Expiring 阶段 | 所有关联 Modifier 从目标移除 |
| V5 | 堆叠不超上限 | Stacking 判定 | stack_count ≤ max_stacks |
| V6 | 周期参数合法 | Def 加载 | interval_turns ≥ 1, max_ticks ≥ 1 |
| V7 | CueDef 存在 | Def 加载 | cues 中的 cue_def_id 在 CueRegistry 中 |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| Effect 应用 | 🟩 完全确定 | 由 Ability/Trigger 确定性触发 |
| Duration 计时 | 🟩 确定 | 回合/帧计数，与游戏时钟同步 |
| Period Tick | 🟩 确定 | tick_count 和 remaining_frames 确定 |
| Effect 移除 | 🟩 确定 | Duration 耗尽 / 显式移除 / 驱散 均确定 |

---

## 8. Save Compatibility

| 场景 | 兼容性 | 版本策略 |
|------|--------|----------|
| 基础存档 | 🟩 | Save v1: EffectSnapshot 含活跃效果列表 |
| 新增 EffectCategory | 🟩 前向兼容 | enum 新 variant |
| 持续时间公式变化 | 🟩 运行时重算 | 存档存剩余回合/帧数，不受公式影响 |
| 堆叠配置变化 | 🟨 存档时重建 | 旧存档堆叠效果按新配置重新计算 |

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 效果增加衰减曲线 | EffectDef 新增 optional falloff 字段 |

---

## 10. Future Extension

- **效果衰减**: 持续效果的强度随持续时间递减（如毒伤每跳递减）
- **效果合并**: 多个同源同类型效果合并计算而非各自独立
- **效果免疫链**: 免疫某效果后自动免疫其衍生效果
- **动态效果**: 运行时根据上下文动态生成 EffectDef

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| 效果数量爆炸 | 长时间战斗后 ActiveEffectContainer 膨胀 | max_effects 上限 + 自动合并同类型效果 |
| Tick 计算性能 | 大量周期性效果同时 Tick | 帧级批量 Tick，限制每帧最大 Tick 数 |
| 驱散回滚复杂 | 驱散效果时需回退所有 Modifier 和 Tag | 效果被移除时自动级联清理 |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| Effect 是唯一业务执行入口 | ✅ | 所有属性变更通过 Effect |
| Duration 属于 Effect | ✅ | EffectDuration 是 EffectDef 固有属性 |
| 表现必须经过 Cue | ✅ | CueBinding 连接 Cue |
| Replay First | ✅ | 确定性生命周期 |
