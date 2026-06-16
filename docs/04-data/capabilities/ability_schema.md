---
id: capabilities.ability.schema.v1
title: Ability Schema — 技能逻辑数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition, instance
replay-safe: true
---

# Ability Schema — 技能逻辑数据架构

> **领域归属**: Capabilities — 行为表现层 | **依赖 Schema**: Tag, Attribute, Spec, Condition, Trigger, Targeting, Execution, Effect, GameplayContext | **定义依据**: `docs/02-domain/ability_domain.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `AbilityDef` | Definition | 技能的完整静态定义（消耗、冷却、效果链） |
| `AbilityInstance` | Instance | 技能激活后的运行时实例 |
| `AbilityState` | Runtime | 技能当前的生命周期阶段 |
| `CostDef` | Definition | 资源消耗定义 |

---

## 2. Problem

Ability 是整个能力系统的「执行核心」——它编排 Condition 检查、Cost 消耗、Targeting 选择、Execution 计算、Effect 应用的完整流程。Schema 必须解决：
- 技能定义（Def）的数据结构——消耗、冷却、目标选择、效果链的完整描述
- 运行时实例（Instance）的生命周期状态追踪
- 消耗（Cost）和冷却（Cooldown）作为 Effect 复用机制的数据表达
- 施法时间/瞬发/反应等不同技能类别的数据区分

---

## 3. Schema Design

### 3.1 AbilityDef（Definition 层）

```rust
struct AbilityDef {
    /// 技能唯一标识
    id: AbilityDefId,

    /// 技能名称本地化 Key
    name_key: LocalizationKey,

    /// 技能描述本地化 Key
    desc_key: LocalizationKey,

    /// 技能分类
    category: AbilityCategory,

    /// 激活类型
    activation: ActivationType,

    /// 资源消耗列表
    costs: Vec<CostDef>,

    /// 冷却定义
    cooldown: CooldownDef,

    /// 目标选择定义
    targeting: TargetingDef,

    /// 效果链——技能执行时按顺序产生的效果
    effects: Vec<EffectApplication>,

    /// 激活条件（可选）
    activation_condition: Option<Condition>,

    /// 技能等级参数（每级的数值变化）
    level_scaling: Option<LevelScaling>,

    /// 可用职业/种族限制
    restrictions: Option<Restrictions>,

    /// 技能标签（用于分类过滤）
    tags: Vec<TagId>,

    /// 最大等级
    max_level: u8,

    /// 元数据
    metadata: AbilityMetadata,
}

enum AbilityCategory {
    /// 主动技能
    Active,
    /// 被动技能（常驻效果，不需要激活）
    Passive,
    /// 反应技能（回合外触发）
    Reaction,
    /// 内在能力（种族/职业自带，不可移除）
    Innate,
}

enum ActivationType {
    /// 瞬发（无施法时间，立即生效）
    Instant,
    /// 需要施法时间（帧数）
    CastTime { frames: u64 },
    /// 需要保持专注
    Concentration,
    /// 需要蓄力（可中断）
    Charge { max_charge_frames: u64 },
    /// 反应动作（回合外）
    Reaction,
}

struct AbilityMetadata {
    /// 是否在快捷栏中显示
    visible: bool,
    /// 是否可被打断
    interruptible: bool,
    /// 技能图标 Key
    icon_key: Option<String>,
    /// 施法动画 Key
    cast_animation: Option<String>,
}
```

### 3.2 CostDef（Definition 层）

```rust
struct CostDef {
    /// 消耗的资源属性
    resource_attribute: AttributeId,

    /// 消耗量（正数表示消耗，负数表示获得）
    amount: ScalableValue,

    /// 是否可选消耗（如"可消耗最多 3 层法术位"）
    optional: bool,

    /// 消耗类型
    cost_type: CostType,
}

enum CostType {
    /// 固定消耗
    Fixed,
    /// 百分比消耗（如消耗当前 HP 的 20%）
    PercentageCurrent,
    /// 百分比最大（如消耗最大 MP 的 30%）
    PercentageMax,
    /// 每级额外消耗
    PerLevel { base: f32, per_level: f32 },
}
```

### 3.3 CooldownDef（Definition 层）

```rust
struct CooldownDef {
    /// 冷却时长（回合数）
    turns: u32,

    /// 冷却是否从技能激活时开始计时
    /// true: 激活即开始（技能执行期间也在走冷却）
    /// false: 执行完毕后才开始
    starts_on_activate: bool,

    /// 是否与其他技能共享冷却
    shared_cooldown_group: Option<String>,

    /// 强制冷却 Tag（可选，自定义冷却标签名称）
    cooldown_tag: Option<TagId>,
}
```

### 3.4 EffectApplication（Definition 层）

```rust
/// 技能效果链中的一个环节。
/// 技能激活后按顺序执行 effects 列表中的每个 EffectApplication。
struct EffectApplication {
    /// 要应用的效果定义
    effect_def_id: EffectDefId,

    /// 效果参数覆盖（可选，覆盖 EffectDef 中的默认值）
    override_params: Option<EffectOverride>,

    /// 该效果的目标选择覆盖（可选，默认使用技能的 targeting 结果）
    targeting_override: Option<TargetingDef>,

    /// 该效果的应用条件（可选，在效果链中独立判断）
    condition: Option<Condition>,

    /// 执行延迟（帧数，效果链中延迟执行本效果）
    delay_frames: Option<u64>,

    /// 执行概率（0.0–1.0，用于"有几率触发"的效果）
    chance: Option<f32>,
}

struct EffectOverride {
    duration_override: Option<ScalableValue>,
    magnitude_override: Option<ScalableValue>,
    period_override: Option<u64>,
}
```

### 3.5 LevelScaling（Definition 层）

```rust
struct LevelScaling {
    /// 每级伤害增加值
    damage_per_level: Option<ScalableValue>,
    /// 每级范围增加值
    range_per_level: Option<f32>,
    /// 每级消耗变化
    cost_per_level: Option<Vec<(AttributeId, ScalableValue)>>,
    /// 每级冷却变化
    cooldown_reduction_per_level: Option<u32>,
    /// 等级突破阈值（如 3 级/5 级时质变）
    breakpoints: Vec<LevelBreakpoint>,
}

struct LevelBreakpoint {
    /// 触发等级
    level: u8,
    /// 质变描述
    description_key: LocalizationKey,
    /// 新增效果（可选）
    additional_effects: Vec<EffectDefId>,
}
```

### 3.6 AbilityInstance（Instance 层 — ECS Component）

```rust
struct AbilityInstance {
    /// 实例唯一标识
    instance_id: AbilityInstanceId,

    /// 关联的 Spec ID
    spec_id: SpecId,

    /// 引用的 AbilityDef ID
    def_id: AbilityDefId,

    /// 当前状态
    state: AbilityState,

    /// 激活时的 GameplayContext
    context: GameplayContextData,

    /// 选中的目标数据
    targets: TargetData,

    /// 施法进度（Casting 状态下使用）
    cast_progress: u64,       // 已施法帧数
    cast_total: u64,           // 总施法帧数

    /// 是否暂停（如被沉默/眩晕打断）
    paused: bool,

    /// 当前正在执行的效果链索引
    current_effect_index: usize,

    /// 实例创建帧号
    created_at_frame: u64,

    /// 来源实体
    caster_entity: EntityId,
}

enum AbilityState {
    Ready,
    Casting,
    Active,
    Cooldown,
    Blocked,
    Removed,
}
```

### 3.7 ActiveAbilityContainer（Instance 层 — ECS Component）

```rust
struct ActiveAbilityContainer {
    /// 所有活跃的技能实例
    active_instances: Vec<AbilityInstance>,

    /// 冷却中的技能列表 (spec_id → remaining_turns)
    cooldowns: HashMap<SpecId, u32>,

    /// 共享冷却组状态 (group_name → remaining_turns)
    shared_cooldowns: HashMap<String, u32>,
}
```

### 3.8 Restrictions（Definition 层）

```rust
struct Restrictions {
    /// 所需职业标签
    required_class: Option<Vec<TagId>>,
    /// 所需种族标签
    required_race: Option<Vec<TagId>>,
    /// 最低等级
    min_level: Option<u8>,
    /// 所需属性条件
    required_attributes: Option<Vec<AttributeCheck>>,
    /// 所需前置技能
    prerequisite_abilities: Option<Vec<AbilityDefId>>,
}
```

### 3.9 AbilityDefConfig（Definition 层 — 配置格式）

```yaml
# RON 配置示例 — 技能定义
AbilityDefConfig:
  abilities:
    # 示例: 火球术 Lv.1
    - id: "abl_000001"
      name_key: "ability.abl_000001.name"
      desc_key: "ability.abl_000001.desc"
      category: Active
      activation: Instant
      max_level: 5

      costs:
        - resource_attribute: "attr_000031"   # 法力值
          amount:
            Fixed: 20.0
          cost_type: Fixed

      cooldown:
        turns: 2
        starts_on_activate: false

      targeting:
        target_type: Enemy
        target_shape:
          Area:
            radius: 2
        max_targets: 6
        range: 10

      effects:
        - effect_def_id: "eff_000001"         # 火焰爆发伤害
          override_params:
            magnitude_override:
              PerLevel:
                base: 8.0
                per_level: 2.0

        - effect_def_id: "eff_000002"         # 灼烧 DOT
          condition:
            TagRequirement:
              mode: HasNone
              target_tags: ["tag_000030"]    # Tag.Immune.Fire

      level_scaling:
        damage_per_level:
          Fixed: 2.0
        range_per_level: 0.5
        cost_per_level:
          - ["attr_000031", { Fixed: 5.0 }]
        breakpoints:
          - level: 3
            description_key: "ability.abl_000001.breakpoint.3"
            additional_effects: ["eff_000003"]   # 爆炸溅射

      tags: ["tag_000012"]   # DamageType.Elemental.Fire
```

### 3.10 AbilitySnapshot（Persistence 层）

```rust
struct AbilitySnapshot {
    schema_version: u32,
    entity_id: EntityId,

    /// 活跃的技能实例快照
    active_instances: Vec<InstanceSnapshot>,

    /// 冷却状态
    cooldowns: HashMap<SpecId, u32>,
    shared_cooldowns: HashMap<String, u32>,
}

struct InstanceSnapshot {
    instance_id: AbilityInstanceId,
    spec_id: SpecId,
    def_id: AbilityDefId,
    state: AbilityState,
    cast_progress: u64,
    current_effect_index: usize,
    targets: TargetData,
    created_at_frame: u64,
    caster_entity: EntityId,
}
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `AbilityDef` | Definition | 是（配置文件） | 是 | 技能完整定义 |
| `CostDef` / `CooldownDef` | Definition | 是（Def 内嵌） | 是 | 消耗/冷却定义 |
| `EffectApplication` | Definition | 是（Def 内嵌） | 是 | 效果链 |
| `LevelScaling` | Definition | 是（Def 内嵌） | 是 | 等级缩放 |
| `AbilityInstance` | Instance | 是（通过 Snapshot） | 否 | ECS Component |
| `ActiveAbilityContainer` | Instance | 是（通过 Snapshot） | 否 | ECS Component |
| `AbilitySnapshot` | Persistence | 是（存档） | 否 | 存档格式 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → ConditionSchema | activation_condition, EffectApplication.condition |
| 依赖 | → SpecSchema | AbilityInstance.spec_id |
| 依赖 | → TargetingSchema | AbilityDef.targeting 引用 TargetType/Shape |
| 依赖 | → ExecutionSchema | 效果链间接通过 Execution 执行计算 |
| 依赖 | → EffectSchema | effects 列表引用 EffectDef |
| 依赖 | → GameplayContextSchema | 激活时创建上下文 |
| 依赖 | → TagSchema | restrictions.tags, AbilityDef.tags |
| 依赖 | → AttributeSchema | costs.resource_attribute |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | 目标技能效果链有效 | Def 加载 | effects 中每个 effect_def_id 已注册 |
| V2 | 消耗属性存在 | Def 加载 | costs 中每个 resource_attribute 已注册 |
| V3 | Condition 先于 Cost | 运行时 | 激活流程中 Condition 检查必须先于 Cost 消耗 |
| V4 | 冷却中禁止激活 | 运行时 | spec_id 在 cooldowns 中时拒绝激活 |
| V5 | 同技能唯一实例 | 运行时 | 同一 SpecId 在没有特殊规则时只能有一个活跃实例 |
| V6 | Blocked 状态可恢复 | 运行时 | Blocked 后恢复时回到之前的状态 |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| 技能激活 | 🟩 完全确定 | 由 Command/Trigger 确定性触发 |
| Condition→Cost→Target→Execute 链 | 🟩 完全确定 | 全链路由 Ability 编排，确定执行 |
| 冷却计数 | 🟩 确定 | 回合计数，与 Turn 事件同步 |
| LevelScaling | 🟩 完全确定 | 等级→数值映射是确定性的 |

---

## 8. Save Compatibility

| 场景 | 兼容性 | 版本策略 |
|------|--------|----------|
| 基础存档 | 🟩 | Save v1: 存活跃实例 + 冷却状态 |
| 新增 AbilityCategory | 🟩 前向兼容 | 枚举新 variant，旧存档缺省判断 |
| 效果链变化 | 🟨 存档时无效 | 存档不存效果链执行进展（已执行到第几步），只存 created_at_frame |
| LevelScaling 变化 | 🟩 运行时重算 | 读档后等级→值按新规则重算 |

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 增加动态效果注入 | AbilityDef 新增 optional 字段 |

---

## 10. Future Extension

- **能力组合**: 技能链支持条件分支（如根据 TargetType 选择不同的效果链）
- **充能系统**: 多充能技能（如「可使用 3 次，随时间恢复」）
- **变形/替换技能**: 特定条件下技能临时变为另一个技能

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| 效果链过长 | 单技能产生数十个 Effect 导致性能问题 | 效果链默认上限 10 步 |
| 激活路径绕过 | 外部系统直接创建 Effect 跳过技能流程 | Ability 领域强制唯一激活入口 |
| 存档不一致 | 进度恢复时活跃实例状态不完整 | active_instances 持久化完整状态 |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| Ability 不拥有行为 | ✅ | on_hit/on_death 等行为归 Trigger |
| Effect 是唯一业务执行入口 | ✅ | Ability → Effect 链，不直接修改属性 |
| 组合优于创建 | ✅ | Cost=Attribute+Effect, Cooldown=Tag+Effect |
| Replay First | ✅ | 全链路由确定 |
