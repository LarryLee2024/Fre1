---
id: capabilities.trigger.schema.v1
title: Trigger Schema — 触发器数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition
replay-safe: true
---

# Trigger Schema — 触发器数据架构

> **领域归属**: Capabilities — 逻辑骨架层 | **依赖 Schema**: Tag, Condition, Event | **定义依据**: `docs/02-domain/capabilities/trigger_domain.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `TriggerDef` | Definition | 触发器的静态定义（触发类型、条件、目标技能） |
| `TriggerCondition` | Definition | 触发条件声明（触发类型 + 附加条件） |
| `TriggerContext` | Runtime | 触发时的上下文数据（传递给目标技能） |
| `TriggerContainer` | Instance | 实体上的触发器容器（ECS Component） |

---

## 2. Problem

Trigger 是「事件→技能」的激活桥梁——当某个事件发生时（如受到伤害、回合开始），Trigger 检测条件是否满足，满足则激活目标技能。Schema 必须解决：
- 触发类型与 Event 事件的映射关系
- 触发条件的附加条件（如「受到火焰伤害时才触发反击」）
- 单回合触发频率限制（如每回合一次反击）
- 触发时上下文（TriggerContext）的数据结构
- 触发优先级的排序（多个触发器同时命中时哪个先执行）

---

## 3. Schema Design

### 3.1 TriggerDef（Definition 层）

```rust
struct TriggerDef {
    /// 触发器唯一标识
    id: TriggerDefId,

    /// 触发类型
    trigger_type: TriggerType,

    /// 触发条件（附加过滤条件，可选）
    condition: Option<Condition>,

    /// 目标技能（触发器满足时激活哪个技能）
    target_ability: AbilityDefId,

    /// 触发优先级（多个触发器同时命中时的执行顺序）
    priority: TriggerPriority,

    /// 每回合最大触发次数
    /// 0 = 不限制
    max_triggers_per_turn: u32,

    /// 是否允许在技能执行过程中再次触发
    allow_concurrent: bool,

    /// 触发后是否消耗资源（如消耗反应动作）
    consumes_reaction: bool,

    /// 自定义参数（不同 TriggerType 可能需要附加参数）
    params: TriggerParams,
}
```

### 3.2 TriggerType & TriggerParams（Definition 层）

```rust
enum TriggerType {
    /// 标签被授予时触发
    OnTagAdded {
        watch_tags: Vec<TagId>,
        respect_hierarchy: bool,
    },
    /// 标签被移除时触发
    OnTagRemoved {
        watch_tags: Vec<TagId>,
    },
    /// 受到伤害时触发
    OnDamaged {
        /// 伤害类型过滤（None = 所有伤害）
        damage_type_filter: Option<Vec<TagId>>,
        /// 最小伤害量
        min_damage: Option<f32>,
    },
    /// 受到治疗时触发
    OnHealed,
    /// 发动攻击时触发
    OnAttack {
        attack_type_filter: Option<Vec<TagId>>,
    },
    /// 回合开始时触发
    OnTurnStart,
    /// 回合结束时触发
    OnTurnEnd,
    /// 单位死亡时触发
    OnDeath {
        /// 监听谁死（Self = 自己死，Ally = 友军死，Enemy = 敌人死）
        watcher: DeathWatcher,
    },
    /// 移动时触发
    OnMove,
    /// 技能被使用时触发
    OnAbilityUsed {
        ability_filter: Option<Vec<AbilityDefId>>,
    },
    /// 自定义触发
    OnCustom(CustomTriggerType),
    /// 特定 Condition 满足时触发（持续评估，不依赖事件）
    OnConditionMet(Condition),
}

enum DeathWatcher {
    Self,
    Ally,
    Enemy,
    Any,
}
```

### 3.3 TriggerPriority（Definition 层）

```rust
/// 触发优先级
/// HIGH 先执行 → MEDIUM → LOW
enum TriggerPriority {
    Critical = 0,   // 0: 最高优先级（如"必定反击"的专长）
    High = 25,      // 25: 高优先级
    Medium = 50,    // 50: 默认优先级
    Low = 75,       // 75: 低优先级
    Last = 100,     // 100: 最后执行
}
```

### 3.4 CustomTriggerType（Definition 层）

```rust
struct CustomTriggerType {
    /// 自定义触发类型标识
    type_id: String,

    /// 订阅的事件 Tag（Event 领域的 EventTag）
    /// 触发器监听此事件的发生
    subscribed_event: EventTag,

    /// 自定义参数
    params: HashMap<String, ConditionParam>,
}
```

### 3.5 TriggerContext（Runtime 层）

```rust
/// 触发器触发时创建的上下文，传递给目标技能作为激活参数。
struct TriggerContext {
    /// 触发的 TriggerDef ID
    trigger_def_id: TriggerDefId,

    /// 触发事件的发生者（谁/什么触发了这个条件）
    event_source: EntityId,

    /// 触发事件的目标（可选，如伤害事件中的受害者）
    event_target: Option<EntityId>,

    /// 触发事件的值（可选，如伤害量）
    event_value: Option<f32>,

    /// 触发事件的原始数据（可选，完整上下文）
    gameplay_context: Option<GameplayContextData>,

    /// 触发时间（帧号）
    trigger_frame: u64,

    /// 本轮战斗中第几次触发（用于频率限制追踪）
    turn_trigger_count: u32,
}
```

### 3.6 TriggerContainer（Instance 层 — ECS Component）

```rust
struct TriggerContainer {
    /// 所有注册的触发器
    triggers: Vec<RegisteredTrigger>,

    /// 每回合触发计数器 (trigger_def_id → count)
    turn_trigger_counts: HashMap<TriggerDefId, u32>,

    /// 当前回合号（用于计数器重置）
    current_turn: u32,
}

struct RegisteredTrigger {
    /// 定义引用
    def_id: TriggerDefId,
    /// 来源 Spec（谁给了我这个触发器）
    source_spec_id: SpecId,
    /// 是否活跃（可能被沉默/禁用）
    active: bool,
}
```

### 3.7 TriggerDefConfig（Definition 层 — 配置格式）

```yaml
# RON 配置示例 — 触发器定义
TriggerDefConfig:
  triggers:
    # 示例1: 反击（近战攻击者受到伤害时反击）
    - id: "trg_000001"
      trigger_type:
        OnDamaged:
          damage_type_filter: ["tag_000020", "tag_000021"]  # Physical.Slashing, Physical.Piercing
          min_damage: 1.0
      target_ability: "abl_000050"    # 基本攻击
      priority: Medium
      max_triggers_per_turn: 1
      consumes_reaction: true
      allow_concurrent: false

    # 示例2: 自动回血（回合开始时触发）
    - id: "trg_000002"
      trigger_type:
        OnTurnStart: ~
      condition:
        TagRequirement:
          mode: HasAll
          target_tags: ["tag_000080"]   # CombatState.InCombat
      target_ability: "abl_000060"    # 自动回血
      priority: Low
      max_triggers_per_turn: 0         # 不限制
      consumes_reaction: false

    # 示例3: 尸爆（死亡时触发）
    - id: "trg_000003"
      trigger_type:
        OnDeath:
          watcher: Self
      target_ability: "abl_000070"    # 尸爆
      priority: High
      max_triggers_per_turn: 1
      consumes_reaction: false

    # 示例4: 狂暴（生命 < 30% 时持续激活）
    - id: "trg_000004"
      trigger_type:
        OnConditionMet:
          AttributeCheck:
            attribute_id: "attr_000030"   # 当前生命值
            operator: LessOrEqual
            source_attribute: "attr_000020"  # 生命上限
            threshold: 0.3
      target_ability: "abl_000080"    # 狂暴
      priority: Medium
      max_triggers_per_turn: 1
      allow_concurrent: true
```

### 3.8 TriggerSnapshot（Persistence 层）

```rust
struct TriggerSnapshot {
    schema_version: u32,
    entity_id: EntityId,

    /// 当前回合的触发计数器
    turn_trigger_counts: HashMap<TriggerDefId, u32>,

    /// 当前回合号
    current_turn: u32,
}
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `TriggerDef` | Definition | 是（配置文件） | 是 | 触发器定义 |
| `TriggerType` | Definition | 是（Def 内嵌） | 是 | 触发类型枚举 |
| `TriggerPriority` | Definition | 代码枚举 | 否 | 优先级层级 |
| `TriggerContext` | Runtime | 否 | 否 | 瞬时传递数据 |
| `TriggerContainer` | Instance | 是（通过 Snapshot） | 否 | ECS Component |
| `RegisteredTrigger` | Instance | 是 | 否 | 触发器实例状态 |
| `TriggerSnapshot` | Persistence | 是（存档） | 否 | 存档格式 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → TagSchema | OnDamaged 的 damage_type_filter、OnTagAdded 的 watch_tags |
| 依赖 | → ConditionSchema | condition 字段引用 Condition |
| 依赖 | → EventSchema | CustomTriggerType 订阅 EventTag |
| 被依赖 | ← AbilitySchema | Trigger 激活目标技能 |
| 被依赖 | ← CombatSchema | 战斗中的反应触发 |
| 被依赖 | ← ReactionSchema | 机会攻击、援护等反应触发 |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | 目标技能已注册 | TriggerDef 加载 | `target_ability` 在 AbilityDefRegistry 中存在 |
| V2 | TagId 存在 | TriggerDef 加载 | OnTagAdded/OnDamaged 等 Tag 过滤器中的 TagId 已注册 |
| V3 | 触发频率上限合法 | TriggerDef 加载 | max_triggers_per_turn 不超过战斗总回合数（防御性限制） |
| V4 | OnConditionMet 不无限重触发 | TriggerDef 加载 | 目标技能不能再次触发同一个 OnConditionMet Trigger |
| V5 | 触发器对应事件源存在 | 运行时 | CustomTriggerType 的 subscribed_event 在 EventBus 中有发布者 |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| Trigger 注册 | 🟩 完全确定 | 由 Spec 授予/移除触发 |
| 触发条件评估 | 🟩 完全确定 | 依赖确定性事件（TagChanged、Damaged 等） |
| 触发频率限制 | 🟩 确定 | turn_trigger_count + current_turn 确定 |
| 优先级排序 | 🟩 完全确定 | 优先级枚举定序，无随机性 |

---

## 8. Save Compatibility

| 场景 | 兼容性 | 版本策略 |
|------|--------|----------|
| 基础存档 | 🟩 | Save v1: 存 turn_trigger_counts |
| 新增 TriggerType | 🟩 前向兼容 | 枚举新 variant，旧存档不受影响 |
| 频率限制变化 | 🟩 配置级变更 | 旧存档读档后按新限制执行 |

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 增加触发条件冷却 | 新增全局 trigger_cooldown 字段 |

---

## 10. Future Extension

- **触发链**: 一个 Trigger 触发技能，技能再注册另一个 Trigger，形成动态行为链
- **触发上下文过滤**: 支持基于 GameplayContext 内容的更精细过滤（如「仅反击远程攻击」）
- **条件触发队列**: 当多个触发器同时命中时，提供队列管理和取消机制

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| 触发器无限链 | A 触发 B，B 再触发 A → 死循环 | ContextChain 的循环检测 + 一回合触发上限 |
| OnConditionMet 性能 | 持续评估触发条件导致每帧检查 | 每帧只检查有限频率（如每 10 帧一次），或依赖事件驱动 |
| 频率计数器与回合不同步 | 回合变更时计数器未重置 | TurnEnd/TurnStart 事件强制重置计数器 |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| Trigger 不拥有行为 | ✅ | Trigger 只描述条件→技能映射，不包含行为逻辑 |
| Replay First | ✅ | 触发条件评估确定，频率限制确定 |
| Composition Over Inheritance | ✅ | 条件通过 Condition 组合，不通过继承 |
