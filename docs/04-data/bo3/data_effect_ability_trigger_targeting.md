# Data Architecture Proposal V2 — BG3 业务数据层提取

> 来源：`docs/其他/79博德3.md` 第3节（能力体系）、第4节（Effect部分）、第7节（触发与反应）
> 提取角色：Data Architect
> 提取日期：2026-06-15
> V2变更：内嵌国际化架构（ADR-017），所有文本字段替换为本地化Key
> 国际化依据：`docs/08-decisions/ADR-017-国际化架构决策.md`

---

## Domain Ownership

| 领域 | 数据类别 | 来源章节 |
|------|----------|----------|
| **Ability** | 动作经济约束下的多类型能力 | 第3节 |
| **Effect** | 三类效果（瞬时/持续/常驻） | 第4节 |
| **Trigger** | 细粒度事件驱动 + 反应机制 | 第7节 |
| **Targeting** | 目标规则与范围 | 第3节（能力字段中） |
| **Cost** | 动作经济 + 法术位 + 专注 | 第3节 + 第8节 |
| **Duration** | 持续效果管理 | 第4节 + 第8节 |

---

## 国际化架构约束（ADR-017）

| 约束 | 规范 |
|------|------|
| Content数据只存Key | `name_key`/`desc_key` 替代 `name`/`description` |
| Key格式 | `namespace.id.suffix` |
| 禁止语义化Key | ❌ `ability.fireball.name` |
| 禁止硬编码文本 | ❌ `name: "火球术"` |

### Key命名空间与ID格式

| 领域 | 命名空间 | ID前缀 | 示例Key |
|------|---------|--------|---------|
| Ability | `ability` | `a_` | `ability.a_1001.name` |
| Effect | `effect` | `e_` | `effect.e_1001.name` |
| Trigger | `trigger` | `t_` | `trigger.t_1001.name` |
| Targeting | `targeting` | `tg_` | `targeting.tg_1001.name` |
| Requirement | `requirement` | `req_` | `requirement.req_1001.name` |
| Condition | `condition` | `c_` | `condition.c_1001.name` |

---

## Schema Design

### 1. Ability Domain — 动作经济约束下的多类型能力

#### 1.1 提取的数据元素

| 数据元素 | 数据层 | 类型 | 说明 | BG3来源 |
|----------|--------|------|------|---------|
| `ability_type` | Definition | `Enum` | 能力动作类型 | 标准动作/附赠动作/反应/自由动作 |
| `spell_slot_level` | Definition | `Option<u32>` | 法术位环级 | 1-9环 |
| `concentration_required` | Definition | `bool` | 是否需要专注 | 专注法术标记 |
| `ability_requirements` | Definition | `Vec<RequirementId>` | 前置校验列表 | 属性要求/熟练要求/标签限制 |
| `ability_cost` | Definition | `CostConfig` | 消耗配置 | 动作类型/法术位/材料/专注 |
| `ability_targeting` | Definition | `TargetingId` | 目标规则引用 | 目标类型/范围/射程/视线 |
| `ability_effects` | Definition | `Vec<EffectId>` | 执行效果列表 | 攻击检定/豁免检定/伤害/持续效果 |
| `ability_special_rules` | Definition | `Vec<SpecialRule>` | 特殊规则 | 可升环/长休恢复/短休恢复 |
| `cooldown_turns` | Definition | `u32` | 冷却回合数 | Lite-GAS替代法术位 |
| `current_cooldown` | Instance | `u32` | 当前冷却剩余 | 运行时状态 |

#### 1.2 BG3动作经济 → Lite-GAS能力分类映射

| BG3动作类型 | 资源模型 | Lite-GAS映射 | 说明 |
|-----------|---------|-------------|------|
| 标准动作 | 每回合1次 | `action_type: Standard` | 保留动作类型分类 |
| 附赠动作 | 每回合1次 | `action_type: Bonus` | 保留动作类型分类 |
| 反应 | 每回合1次，事件触发 | `action_type: Reaction` | 保留，但由Trigger驱动 |
| 自由动作 | 无消耗 | `action_type: Free` | 保留 |
| 法术位 | 总量限制+休息恢复 | `cost: Cooldown(N)` | ❌ 替换为冷却机制 |
| 专注 | 同角色同时间仅1个 | `concentration: true` | ⚠️ 按需吸收 |

#### 1.3 Schema草案

```rust
// === Definition Layer ===

/// 能力定义（配置，运行时不可变）
///
/// 不变量：
/// - Ability不拥有行为（Law 004）
/// - 只处理 Cost、Cooldown、Targeting、Effects
/// - 所有事件驱动行为归属 Trigger 领域
/// - name_key/desc_key 必须符合 ADR-017 Key命名规范
struct AbilityDefinition {
    id: AbilityId,                   // 如 "a_1001"

    // 国际化字段（ADR-017）
    name_key: String,                // "ability.a_1001.name"
    desc_key: String,                // "ability.a_1001.desc"

    // 消耗与资源
    action_type: ActionType,
    cost: CostConfig,
    cooldown: u32,                   // 冷却回合数（0=无冷却）

    // 前置校验
    requirements: Vec<RequirementId>, // 引用ID（Law 003）

    // 目标与效果
    targeting: TargetingId,          // 引用ID（Law 003）
    effects: Vec<EffectId>,          // 引用ID（Law 003, Law 005）

    // 专注
    concentration: bool,

    // 特殊规则
    special_rules: Vec<SpecialRule>,
}

/// 动作类型
enum ActionType {
    Standard,    // 标准动作
    Bonus,       // 附赠动作
    Reaction,    // 反应（由Trigger驱动）
    Free,        // 自由动作
    Passive,     // 被动（不消耗动作）
}

/// 特殊规则（可配置扩展）
enum SpecialRule {
    Upcastable,              // 可升环
    LongRestRecovery,        // 长休恢复
    ShortRestRecovery,       // 短休恢复
    NoReactionChain,         // 不触发反应
    IgnoreLineOfSight,       // 忽略视线
}

// === Instance Layer ===

/// 能力实例（运行时状态）
///
/// 不变量：
/// - current_cooldown <= cooldown
/// - cooldown为0时必须移除（项目memory约束）
struct AbilityInstance {
    definition_id: AbilityId,
    current_cooldown: u32,
    is_available: bool,
}

// === Persistence Layer ===

/// 能力存档状态
struct AbilitySaveState {
    ability_id: AbilityId,
    current_cooldown: u32,
}
```

---

### 2. Effect Domain — 三类效果统一管线

#### 2.1 提取的数据元素

| 数据元素 | 数据层 | 类型 | 说明 | BG3来源 |
|----------|--------|------|------|---------|
| `effect_type` | Definition | `Enum` | 效果分类 | 瞬时/持续/常驻 |
| `effect_duration` | Definition | `DurationConfig` | 持续时间 | 回合数/永久/瞬时 |
| `effect_execution` | Definition | `ExecutionId` | 执行算式引用 | 攻击检定/豁免检定/伤害结算 |
| `effect_modifiers` | Definition | `Vec<ModifierId>` | 修正列表 | 数值加值/优势劣势 |
| `effect_cues` | Definition | `Vec<CueId>` | 表现信号列表 | 特效/音效/UI反馈 |
| `effect_stacking` | Definition | `StackingId` | 堆叠策略引用 | 叠加/替换/刷新 |
| `effect_source` | Instance | `Entity` | 效果来源实体 | 追踪来源 |
| `remaining_turns` | Instance | `Option<u32>` | 剩余回合数 | 持续效果倒计时 |

#### 2.2 BG3效果三大类 → Lite-GAS Effect分类映射

| BG3效果类型 | 特征 | Lite-GAS映射 | Duration配置 |
|-----------|------|-------------|-------------|
| 瞬时效果 | 执行即结束 | `Effect + Duration::Instant` | `turns: 0` |
| 持续效果 | 有持续回合数 | `Effect + Duration::TurnLimited(N)` | `turns: N` |
| 常驻效果 | 永久生效 | `Effect + Duration::Permanent` | `turns: None` |

**Law 007检查**：Duration属于Effect，不属于独立Buff系统。✅ 通过。

#### 2.3 Schema草案

```rust
// === Definition Layer ===

/// 效果定义（配置，运行时不可变）
///
/// 不变量：
/// - Effect是唯一业务执行入口（Law 005）
/// - Duration属于Effect（Law 007）
/// - 所有表现必须经过Cue（Law 009）
/// - name_key/desc_key 必须符合 ADR-017 Key命名规范
struct EffectDefinition {
    id: EffectId,                    // 如 "e_1001"

    // 国际化字段（ADR-017）
    name_key: String,                // "effect.e_1001.name"
    desc_key: String,                // "effect.e_1001.desc"

    // 效果分类
    effect_type: EffectType,

    // 持续时间（Law 007: Duration属于Effect）
    duration: DurationConfig,

    // 执行算式（Law 005: Effect→Execution）
    execution: ExecutionId,

    // 修正列表（Law 005: Effect→Modifier，不跳过）
    modifiers: Vec<ModifierId>,

    // 表现信号（Law 009: Effect→Cue）
    cues: Vec<CueId>,

    // 堆叠策略（Law 008: 堆叠归属Stacking）
    stacking: StackingId,

    // 标签要求
    required_tags: Vec<TagId>,
    blocked_tags: Vec<TagId>,
}

/// 效果类型
enum EffectType {
    Instant,      // 瞬时
    Persistent,   // 持续
    Permanent,    // 常驻
}

/// 持续时间配置
///
/// Law 007: Duration属于Effect，不属于独立Buff系统。
enum DurationConfig {
    Instant,
    TurnLimited { turns: u32 },
    Permanent,
}

// === Instance Layer ===

/// 效果实例（运行时状态）
///
/// 不变量：
/// - 必须有来源（source entity）
/// - 必须有过期条件（duration）
/// - 过期后Modifier必须清理
struct EffectInstance {
    definition_id: EffectId,
    source: Entity,
    target: Entity,
    remaining_turns: Option<u32>,
    applied_modifiers: Vec<Entity>,
    active_tags: Vec<TagId>,
}

// === Persistence Layer ===

/// 效果存档状态
struct EffectSaveState {
    effect_id: EffectId,
    source: Entity,
    target: Entity,
    remaining_turns: Option<u32>,
}
```

---

### 3. Trigger Domain — 细粒度事件驱动 + 反应机制

#### 3.1 提取的数据元素

| 数据元素 | 数据层 | 类型 | 说明 | BG3来源 |
|----------|--------|------|------|---------|
| `trigger_event` | Definition | `Enum` | 触发事件类型 | 四级事件粒度 |
| `trigger_condition` | Definition | `Option<ConditionId>` | 触发条件 | 条件判断 |
| `trigger_effects` | Definition | `Vec<EffectId>` | 触发后执行的效果 | 反应效果列表 |
| `trigger_priority` | Definition | `u32` | 触发优先级 | 触发顺序 |
| `reaction_quota` | Definition | `u32` | 反应资源配额 | 每回合1次 |
| `reaction_used` | Instance | `bool` | 本回合是否已用反应 | 运行时状态 |
| `no_chain_reaction` | Definition | `bool` | 是否禁止连锁反应 | 防循环 |

#### 3.2 BG3四级触发事件 → Lite-GAS Trigger事件映射

| BG3事件级别 | 事件示例 | Lite-GAS映射 | 吸收策略 |
|-----------|---------|-------------|---------|
| 动作生命周期 | 施放前/后、攻击前/后、移动前/后 | `ActionLifecycle` 事件族 | ✅ 直接吸收 |
| 结算事件 | 造成伤害时、受到伤害时、通过豁免时、检定成功/失败时 | `Resolution` 事件族 | ✅ 吸收 |
| 状态事件 | 获得/失去状态、进入濒死、死亡时 | `StatusChange` 事件族 | ✅ 直接吸收 |
| 环境事件 | 进入/离开格子、接触地表、环境变化时 | `Environment` 事件族 | ⚠️ 按需吸收 |

#### 3.3 触发事件粒度详细清单

```rust
/// 触发事件枚举
///
/// 来源：BG3四级事件粒度，映射为Lite-GAS确定性事件。
enum TriggerEvent {
    // === 动作生命周期 ===
    BeforeCast, AfterCast,
    BeforeAttack, AfterAttack,
    BeforeMove, AfterMove,

    // === 结算事件 ===
    OnDealDamage, OnTakeDamage,
    OnSaveSucceeded, OnCheckSucceeded, OnCheckFailed,
    BeforeResolution, AfterResolution,

    // === 状态事件 ===
    OnStatusGained, OnStatusLost,
    OnDying, OnDeath,

    // === 环境事件 ===
    OnEnterTile, OnLeaveTile,
    OnContactSurface, OnEnvironmentChange,
}
```

#### 3.4 Schema草案

```rust
// === Definition Layer ===

/// 触发器定义（配置，运行时不可变）
///
/// 不变量：
/// - Trigger只负责事件监听+条件判断+效果触发
/// - 触发的效果必须经过Effect（Law 005）
/// - 反应不触发新反应（防循环）
/// - name_key 必须符合 ADR-017 Key命名规范
struct TriggerDefinition {
    id: TriggerId,                   // 如 "t_1001"

    // 国际化字段（ADR-017）
    name_key: String,                // "trigger.t_1001.name"
    desc_key: String,                // "trigger.t_1001.desc"

    // 触发事件
    event: TriggerEvent,
    condition: Option<ConditionId>,

    // 触发效果
    effects: Vec<EffectId>,

    // 反应管控
    is_reaction: bool,
    no_chain: bool,

    // 优先级
    priority: u32,
}

// === Instance Layer ===

/// 触发器实例
struct TriggerInstance {
    definition_id: TriggerId,
    owner: Entity,
    is_active: bool,
    reaction_used_this_turn: bool,
}

// === Runtime Layer ===

/// 反应资源配额（全局Resource）
///
/// 不变量：
/// - 每个角色每回合最多1次反应
/// - 回合开始时重置
struct ReactionQuota {
    used_reactions: HashSet<Entity>,
}
```

---

### 4. Targeting Domain — 目标规则与范围

#### 4.1 Schema草案

```rust
// === Definition Layer ===

/// 目标规则定义（配置，运行时不可变）
///
/// 不变量：
/// - Targeting只负责目标选择，不负责效果执行（纯函数）
/// - name_key 必须符合 ADR-017 Key命名规范
struct TargetingDefinition {
    id: TargetingId,                 // 如 "tg_1001"

    // 国际化字段（ADR-017）
    name_key: String,                // "targeting.tg_1001.name"

    target_type: TargetType,
    target_count: TargetCount,
    range: u32,
    aoe: Option<AoeConfig>,
    requires_los: bool,
    target_filters: Vec<TagId>,
}

/// 目标类型
enum TargetType {
    EnemySingle, EnemyAoe,
    AllySingle, AllyAoe,
    SelfOnly, EmptyTile, AnyUnit,
}

/// 目标数量
enum TargetCount {
    Fixed(u32),
    UpTo(u32),
    AllInRange,
}

/// AOE配置
struct AoeConfig {
    shape: AoeShape,
    size: u32,
}

/// AOE形状
enum AoeShape {
    Circle, Cone, Line, Cross,
}
```

---

### 5. Cost Domain — 消耗配置

#### 5.1 BG3消耗模型 → Lite-GAS映射

| BG3消耗 | Lite-GAS替代 | 说明 |
|---------|-------------|------|
| 法术位环级 | `cooldown: N` | 冷却回合数替代 |
| 长休恢复 | `recovery: LongRest` | 按需吸收 |
| 短休恢复 | `recovery: ShortRest` | 按需吸收 |
| 动作类型 | `action_type` | 直接保留 |

#### 5.2 Schema草案

```rust
// === Definition Layer ===

/// 消耗配置
///
/// 不变量：
/// - 消耗检查在Requirement检查之后执行
struct CostConfig {
    action_type: ActionType,
    resource_costs: Vec<ResourceCost>,
    material_cost: Option<ItemId>,
    concentration: bool,
}

/// 资源消耗
struct ResourceCost {
    resource: ResourceType,
    amount: u32,
}

/// 资源类型
enum ResourceType {
    Hp, Mp, ActionPoint, Item,
}
```

---

## Dependency Analysis

### 领域间依赖关系

```
Ability ────→ Cost (消耗配置)
   │
   ├──→ Targeting (目标规则)
   │
   ├──→ Effect (执行效果, Law 005)
   │       │
   │       ├──→ Execution (执行算式)
   │       ├──→ Modifier (数值修正)
   │       ├──→ Cue (表现信号, Law 009)
   │       ├──→ Stacking (堆叠策略, Law 008)
   │       └──→ Duration (持续时间, Law 007)
   │
   └──→ Requirement (前置校验)

Trigger ────→ Effect (触发效果, Law 005)
   │
   └──→ Condition (触发条件)

Targeting ──→ Tag (目标标签过滤)
```

### 关键数据流

```
玩家释放能力:
  Ability → Requirement检查 → Cost扣除 → Targeting选择目标
    → Effect生成 → Stacking判定 → Execution执行 → Modifier应用 → Cue表现

触发器响应:
  TriggerEvent → Condition判断 → Effect生成 → [同上管线]
```

---

## Validation Rules

### Ability校验

| 规则 | 校验时机 | 错误级别 |
|------|---------|---------|
| effects列表不能为空 | 加载时 | ERROR |
| targeting必须引用已注册ID | 加载时 | ERROR |
| requirements必须引用已注册ID | 加载时 | ERROR |
| cooldown=0时无冷却 | 加载时 | — |
| concentration=true时duration不能为Instant | 加载时 | WARN |
| name_key格式必须符合`ability.a_XXXX.suffix` | 加载时 | ERROR |
| desc_key格式必须符合`ability.a_XXXX.suffix` | 加载时 | WARN |

### Effect校验

| 规则 | 校验时机 | 错误级别 |
|------|---------|---------|
| execution必须引用已注册ID | 加载时 | ERROR |
| modifiers必须引用已注册ID | 加载时 | ERROR |
| cues必须引用已注册ID | 加载时 | ERROR |
| stacking必须引用已注册ID | 加载时 | ERROR |
| Duration::TurnLimited的turns > 0 | 加载时 | ERROR |
| Instant效果不能有modifiers | 加载时 | WARN |
| name_key格式必须符合`effect.e_XXXX.suffix` | 加载时 | ERROR |

### Trigger校验

| 规则 | 校验时机 | 错误级别 |
|------|---------|---------|
| effects列表不能为空 | 加载时 | ERROR |
| condition必须引用已注册ID或为None | 加载时 | ERROR |
| is_reaction=true时必须有reaction_quota | 加载时 | WARN |
| 不允许循环触发链 | 加载时 | ERROR |
| name_key格式必须符合`trigger.t_XXXX.suffix` | 加载时 | ERROR |

### Targeting校验

| 规则 | 校验时机 | 错误级别 |
|------|---------|---------|
| range > 0（SelfOnly除外） | 加载时 | ERROR |
| aoe配置与target_type一致 | 加载时 | ERROR |
| target_filters必须引用已注册TagId | 加载时 | ERROR |
| name_key格式必须符合`targeting.tg_XXXX.suffix` | 加载时 | ERROR |

### 国际化校验（ADR-017，适用于所有领域）

| 规则 | 校验时机 | 错误级别 |
|------|---------|---------|
| name_key必选，不能为空 | 加载时 | ERROR |
| Key格式必须为`namespace.id.suffix` | 加载时 | ERROR |
| namespace必须是已注册领域 | 加载时 | ERROR |
| id必须是永久唯一ID格式 | 加载时 | ERROR |
| suffix必须是name/desc/short_desc/tooltip之一 | 加载时 | ERROR |
| 禁止语义化Key | 加载时 | ERROR |
| 禁止无意义编号Key | 加载时 | ERROR |
| Key不能包含中文/特殊字符 | 加载时 | ERROR |

---

## Replay Compatibility

| 数据元素 | Replay影响 | 处理策略 |
|---------|-----------|---------|
| 能力释放序列 | 必须确定性 | 记录AbilityId + 目标Entity |
| 效果执行顺序 | 必须确定性 | 按Effect定义顺序执行 |
| 触发器响应顺序 | 必须确定性 | 按priority排序 |
| 反应决策 | 必须确定性 | AI反应记录决策结果，非决策过程 |
| 目标选择 | 必须确定性 | 记录目标Entity列表 |
| 冷却计时 | 必须确定性 | 回合数驱动，不依赖实时时间 |
| name_key/desc_key | 不影响Replay | Key是确定性字符串 |

**Law 010检查**：所有业务操作均由回合数驱动，不依赖当前时间、系统随机数或外部状态。✅ 通过。

---

## Save Compatibility

| 数据元素 | Save版本策略 | 迁移考虑 |
|---------|-------------|---------|
| AbilityDefinition | 大版本+1才可删除 | 新能力不影响旧存档 |
| EffectDefinition | 小版本可新增 | 新效果默认不存在于旧存档 |
| TriggerDefinition | 小版本可新增 | 新触发器默认不存在于旧存档 |
| AbilityInstance.cooldown | 新字段需默认值 | 缺失时默认0 |
| EffectInstance.remaining_turns | 必须持久化 | 持续效果恢复必需 |
| ReactionQuota | 不持久化 | 每场战斗重新计算 |
| name_key/desc_key | 新字段需默认值 | 旧存档硬编码文本需迁移为Key |

---

## Migration Strategy

### 从BG3模型迁移到Lite-GAS

| 迁移项 | BG3模型 | Lite-GAS目标 | 迁移路径 |
|--------|---------|-------------|---------|
| 法术位体系 | 1-9环总量限制 | Cooldown回合冷却 | 完全替换 |
| 休息恢复 | 长休/短休 | 回合自动恢复/冷却归零 | 简化 |
| 专注机制 | 同角色仅1个 | 可配置是否启用 | 按需吸收 |
| 反应配额 | 每回合1次 | 保留，可配置 | 直接迁移 |
| 掷骰前/后事件 | 骰子驱动 | 结算前/后事件 | 概念映射 |
| 硬编码文本 | `name: "火球术"` | `name_key: "ability.a_1001.name"` | Key映射表 |

---

## Future Extension

| 扩展点 | 当前设计 | 未来可能 |
|--------|---------|---------|
| ActionType | 5种 | 可新增Channel(持续施法)、Charge(蓄力) |
| EffectType | 3种 | 可新增Delayed(延迟触发) |
| TriggerEvent | 20+种 | 可新增OnHeal、OnBuff、OnEquip |
| TargetType | 7种 | 可新增Chain(链式弹射)、Summon(召唤) |
| CostConfig | 4种消耗 | 可新增ComboPoint(连击点) |
| 反应机制 | 每回合1次 | 可配置为多次或按能力独立 |
| Key后缀 | 4种 | 可新增`.flavor`、`.lore` |

---

## Risks

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 法术位→冷却替换影响玩法节奏 | 玩家体验变化 | 可配置冷却+充能混合模式 |
| 触发器循环依赖 | 无限循环 | no_chain标志 + 加载时循环检测 |
| 反应机制AI决策复杂 | AI反应选择不确定 | 记录AI决策结果到Replay |
| 专注机制与多Buff叠加冲突 | 数值平衡问题 | 可配置开关，默认不启用 |
| Key映射表维护成本 | 旧存档迁移复杂 | 自动化迁移工具 |

---

## Constitution Check

| Data Law / 规范 | 检查结果 | 说明 |
|----------------|---------|------|
| Law 001 | ✅ 通过 | AbilityDefinition/AbilityInstance分离；EffectDefinition/EffectInstance分离；TriggerDefinition/TriggerInstance分离 |
| Law 002 | ✅ 通过 | 执行算式通过ExecutionId引用，不内联公式 |
| Law 003 | ✅ 通过 | Ability引用EffectId/TargetingId/RequirementId，不重复定义 |
| Law 004 | ✅ 通过 | Ability只处理Cost/Cooldown/Targeting/Effects，事件行为归Trigger |
| Law 005 | ✅ 通过 | Ability→Effect, Trigger→Effect，不直接调用Modifier |
| Law 007 | ✅ 通过 | Duration属于Effect，不属于独立Buff |
| Law 008 | ✅ 通过 | 堆叠策略通过StackingId引用 |
| Law 009 | ✅ 通过 | 表现通过CueId引用，Effect不直接播放特效 |
| Law 010 | ✅ 通过 | 所有操作确定性，无随机依赖 |
| ADR-017 | ✅ 通过 | 所有文本字段使用name_key/desc_key，Key格式符合规范 |
| 宪法§17.2.2 | ✅ 通过 | 禁止硬编码玩家可见文本 |

**[Data Exemption]**：无。

---

## 数据清单汇总

### Ability Domain

| # | 数据元素 | 数据层 | 类型 | 必选 | 来源 |
|---|---------|--------|------|------|------|
| AB-01 | AbilityId | Definition | String | ✅ | BG3§3 |
| AB-02 | ActionType | Definition | Enum(5种) | ✅ | BG3§3 |
| AB-03 | AbilityDefinition.name_key | Definition | String(LocalizedKey) | ✅ | BG3§3 + ADR-017 |
| AB-04 | AbilityDefinition.desc_key | Definition | String(LocalizedKey) | ✅ | BG3§3 + ADR-017 |
| AB-05 | AbilityDefinition.cost | Definition | CostConfig | ✅ | BG3§3 |
| AB-06 | AbilityDefinition.cooldown | Definition | u32 | ✅ | BG3§3 |
| AB-07 | AbilityDefinition.requirements | Definition | Vec<RequirementId> | ❌ | BG3§3 |
| AB-08 | AbilityDefinition.targeting | Definition | TargetingId | ✅ | BG3§3 |
| AB-09 | AbilityDefinition.effects | Definition | Vec<EffectId> | ✅ | BG3§3 |
| AB-10 | AbilityDefinition.concentration | Definition | bool | ❌ | BG3§3 |
| AB-11 | AbilityDefinition.special_rules | Definition | Vec<SpecialRule> | ❌ | BG3§3 |
| AB-12 | AbilityInstance.current_cooldown | Instance | u32 | ✅ | BG3§3 |
| AB-13 | AbilityInstance.is_available | Instance | bool | ✅ | BG3§3 |
| AB-14 | AbilitySaveState | Persistence | Struct | ✅ | BG3§3 |

### Effect Domain

| # | 数据元素 | 数据层 | 类型 | 必选 | 来源 |
|---|---------|--------|------|------|------|
| EF-01 | EffectId | Definition | String | ✅ | BG3§4 |
| EF-02 | EffectType | Definition | Enum(3种) | ✅ | BG3§4 |
| EF-03 | DurationConfig | Definition | Enum(3种) | ✅ | BG3§4 |
| EF-04 | EffectDefinition.name_key | Definition | String(LocalizedKey) | ✅ | BG3§4 + ADR-017 |
| EF-05 | EffectDefinition.desc_key | Definition | String(LocalizedKey) | ❌ | BG3§4 + ADR-017 |
| EF-06 | EffectDefinition.execution | Definition | ExecutionId | ✅ | BG3§4 |
| EF-07 | EffectDefinition.modifiers | Definition | Vec<ModifierId> | ❌ | BG3§4 |
| EF-08 | EffectDefinition.cues | Definition | Vec<CueId> | ❌ | BG3§4 |
| EF-09 | EffectDefinition.stacking | Definition | StackingId | ✅ | BG3§4 |
| EF-10 | EffectDefinition.required_tags | Definition | Vec<TagId> | ❌ | BG3§4 |
| EF-11 | EffectDefinition.blocked_tags | Definition | Vec<TagId> | ❌ | BG3§4 |
| EF-12 | EffectInstance.source | Instance | Entity | ✅ | BG3§4 |
| EF-13 | EffectInstance.remaining_turns | Instance | Option<u32> | ✅ | BG3§4 |
| EF-14 | EffectInstance.applied_modifiers | Instance | Vec<Entity> | ✅ | BG3§4 |
| EF-15 | EffectSaveState | Persistence | Struct | ✅ | BG3§4 |

### Trigger Domain

| # | 数据元素 | 数据层 | 类型 | 必选 | 来源 |
|---|---------|--------|------|------|------|
| TR-01 | TriggerId | Definition | String | ✅ | BG3§7 |
| TR-02 | TriggerEvent | Definition | Enum(20+种) | ✅ | BG3§7 |
| TR-03 | TriggerDefinition.name_key | Definition | String(LocalizedKey) | ✅ | BG3§7 + ADR-017 |
| TR-04 | TriggerDefinition.desc_key | Definition | String(LocalizedKey) | ❌ | BG3§7 + ADR-017 |
| TR-05 | TriggerDefinition.condition | Definition | Option<ConditionId> | ❌ | BG3§7 |
| TR-06 | TriggerDefinition.effects | Definition | Vec<EffectId> | ✅ | BG3§7 |
| TR-07 | TriggerDefinition.is_reaction | Definition | bool | ❌ | BG3§7 |
| TR-08 | TriggerDefinition.no_chain | Definition | bool | ❌ | BG3§7 |
| TR-09 | TriggerDefinition.priority | Definition | u32 | ❌ | BG3§7 |
| TR-10 | TriggerInstance.reaction_used_this_turn | Instance | bool | ✅ | BG3§7 |
| TR-11 | ReactionQuota | Runtime | Resource | ✅ | BG3§7 |

### Targeting Domain

| # | 数据元素 | 数据层 | 类型 | 必选 | 来源 |
|---|---------|--------|------|------|------|
| TG-01 | TargetingId | Definition | String | ✅ | BG3§3 |
| TG-02 | TargetType | Definition | Enum(7种) | ✅ | BG3§3 |
| TG-03 | TargetCount | Definition | Enum(3种) | ✅ | BG3§3 |
| TG-04 | TargetingDefinition.name_key | Definition | String(LocalizedKey) | ✅ | BG3§3 + ADR-017 |
| TG-05 | TargetingDefinition.range | Definition | u32 | ✅ | BG3§3 |
| TG-06 | AoeConfig | Definition | Struct | ❌ | BG3§3 |
| TG-07 | TargetingDefinition.requires_los | Definition | bool | ❌ | BG3§3 |
| TG-08 | TargetingDefinition.target_filters | Definition | Vec<TagId> | ❌ | BG3§3 |

### Cost Domain

| # | 数据元素 | 数据层 | 类型 | 必选 | 来源 |
|---|---------|--------|------|------|------|
| CO-01 | CostConfig | Definition | Struct | ✅ | BG3§3 |
| CO-02 | ActionType (in Cost) | Definition | Enum | ✅ | BG3§3 |
| CO-03 | ResourceCost | Definition | Struct | ❌ | BG3§3 |
| CO-04 | ResourceType | Definition | Enum(4种) | ❌ | BG3§3 |
| CO-05 | CostConfig.concentration | Definition | bool | ❌ | BG3§3 |
