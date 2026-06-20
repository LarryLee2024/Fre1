# Domain 层深度解析——双轴架构全景

> 30 个领域模块、700 个 Rust 文件、57,000 行代码：Fre 项目的领域层（L1 Core）是如何分割成能力机制和业务域的。

---

## 目录

1. [Domain 层在整个架构中的位置](#1-domain-层在整个架构中的位置)
2. [双轴架构的设计哲学](#2-双轴架构的设计哲学)
3. [Capability 模块内部结构（C1 → C2 → C3）](#3-capability-模块内部结构c1--c2--c3)
4. [Domain 模块内部结构（7 文件标准）](#4-domain-模块内部结构7-文件标准)
5. [15 个 Capability 全景（能力机制层）](#5-15-个-capability-全景能力机制层)
6. [15 个 Domain 全景（业务域层）](#6-15-个-domain-全景业务域层)
7. [集成层（Integration / Anti-Corruption Layer）](#7-集成层integration--anti-corruption-layer)
8. [四级通信体系](#8-四级通信体系)
9. [失效模式：Rule Failure vs 程序 Error](#9-失效模式rule-failure-vs-程序-error)
10. [跨领域关键流程](#10-跨领域关键流程)
11. [当前实现状态](#11-当前实现状态)
12. [全部相关文件索引](#12-全部相关文件索引)

---

## 1. Domain 层在整个架构中的位置

Domain 层就是 **L1 Core**，是项目最核心的代码层。它位于 Shared（L0）之上、Infrastructure（L2）之下：

```
L3: UI / Presentation（src/ui/）
    ↑ 依赖 Core，但不被 Core 依赖
L2: Infrastructure（src/infra/）
    ↑ 依赖 Core，提供技术实现
L1: Core（src/core/）
    双轴结构：15 Capabilities + 15 Domains + 共享基础设施
    ↑ 只依赖 Shared
L0: Shared（src/shared/）
    强类型 ID、数学工具、确定性 RNG、基础类型
```

核心规则：**Domain 层（Core）不依赖上层的任何东西**。Infrastructure 和 UI 可以引用 Core，反之不行。

---

## 2. 双轴架构的设计哲学

为什么是"双轴"？因为游戏开发中的"机制"和"业务"本质上是不同维度的东西。

### 传统游戏架构的问题

大多数游戏项目会按业务直接划分模块：

```
combat/  ← 战斗逻辑 + 命中判定 + 技能系统 + 属性系统 + Buff 系统
spell/   ← 法术逻辑 + 法术消耗 + 法术效果
inventory/ ← 背包逻辑 + 物品属性 + 装备效果
```

这种方案的问题是**机制重复**——命中判定用一套"数值+修饰器"逻辑、法术消耗用另一套、装备效果用第三套。每套都自己实现了"计算数值→应用修饰器→产生效果"的流程，但彼此之间不兼容、不可组合。

### Capabilities/Domains 双轴解耦

Fre 把"通用机制"和"业务规则"拆成两个正交的维度：

```
Capabilities（能力机制轴）           Domains（业务域轴）
┌─────────────────────────┐     ┌──────────────────────────┐
│  "通用机制骨架"           │     │  "具体业务规则"            │
│                         │     │                          │
│  Tag      ← 分类标记     │     │  Tactical  ← 战术空间      │
│  Attribute ← 数值属性    │     │  Combat    ← 战斗流程      │
│  Modifier ← 数值修饰器   │     │  Spell     ← 法术规则      │
│  Effect   ← 效果生命周期  │     │  Progression ← 成长养成    │
│  Execution ← 伤害/治疗    │     │  Crafting  ← 制作锻造      │
│  ... 共 15 个             │     │  ... 共 15 个              │
└─────────────────────────┘     └──────────────────────────┘
```

**Capability = 机制层**——告诉你"数值修饰器怎么工作"（Modifier 是 Add/Multiply/Override，按优先级排序），但它不关心修饰器是来自装备还是 Buff。

**Domain = 业务层**——告诉你"装备加的攻击力应该用 Add 还是 Multiply"，但它不关心 Add/Multiply 的具体实现机制。

### 依赖方向

```
Capabilities ←────── Domains     ✅ Domain 引用 Capability（使用机制）
Domains ←────── Domains          ✅ 域间只通过 Event 通信，不直接引用
Capabilities →─── Capabilities   ✅ Capability 依赖链（Tag → Modifier → Effect）
Domains ──────→ Capabilities     ❌ 禁止反向依赖（Capability 不感知 Domain）
```

### Plugin 注册顺序

```
Phase 1-4: Capabilities（先注册机制）
Phase 5-7: Domains（后注册业务）
```

具体顺序（从 `docs/01-architecture/README.md`）：

```
Phase 1: Capabilities — Foundation  (Tag, Attribute, Modifier, Aggregator, GameplayContext)
Phase 2: Capabilities — Logic       (Spec, Condition, Trigger, Event)
Phase 3: Capabilities — Behavior    (Ability, Targeting, Execution, Effect, Stacking, Cue)
Phase 4: Capabilities — Runtime     (RuntimePlugin: pipeline, scheduler, registry, command, replay)
──────────────────────────────────────────────────────────────────────────────────────────
Phase 5: Domains — Foundation       (Tactical, Terrain, Faction)
Phase 6: Domains — Core             (Combat, Spell, Reaction, Progression, Inventory, Party, CampRest)
Phase 7: Domains — Narrative/Economy(Narrative, Quest, Economy, Crafting, Summon)
```

Capabilities 必须先注册，因为 Domains 在 build 时需要引用 Capabilities 提供的类型和接口。

---

## 3. Capability 模块内部结构（C1 → C2 → C3）

每个 Capability 模块在 `src/core/capabilities/<name>/` 下，遵循 C1→C2→C3 分层：

```
capabilities/<name>/
├── mod.rs                                # 模块入口，公开/私有控制
├── plugin.rs                             # Bevy Plugin（唯一对外入口）
├── events.rs                             # 领域事件枚举
│
├── foundation/              (C1 层)       # 纯数据定义层——无行为逻辑
│   ├── mod.rs
│   ├── types.rs                          # 核心枚举与结构体
│   ├── values.rs                         # 值对象
│   └── error.rs                          # 错误类型（可选）
│
├── mechanism/               (C2 层)       # 规则与系统层——ECS 行为
│   ├── mod.rs
│   ├── components.rs                     # ECS Component 定义
│   ├── lifecycle.rs                      # 生命周期管理
│   ├── query.rs                          # 查询接口（可选）
│   └── systems/                          # ECS System（可选）
│       ├── mod.rs
│       └── <name>_system.rs
│
└── tests/                                # 领域内聚测试
    ├── unit/
    ├── integration/
    ├── invariant/
    └── fixtures/
```

**C1（Foundation）层**只包含纯数据类型——没有 ECS Component、没有 System、没有副作用。可以单独编译和测试。这是业务规则文档的"直接翻译层"。

**C2（Mechanism）层**把 C1 的类型适配到 ECS 世界——定义 Component、实现 System、管理生命周期。这是"让类型动起来"的层。

**C3（Runtime）层**是一个特殊的跨能力层——`runtime/` 模块包含 pipeline/scheduler/registry/command/replay 五个子模块，在多个 Capabilities 之上提供编排能力。它不是 Capability，而是 Capability 的使用者。

---

## 4. Domain 模块内部结构（7 文件标准）

每个 Domain 模块在 `src/core/domains/<name>/` 下，遵循 7 文件标准：

```
domains/<name>/
├── mod.rs                                # 模块入口
├── plugin.rs                             # Bevy Plugin（唯一对外入口）
├── components.rs                         # 私有 ECS Component（public 包不可见）
├── events.rs                             # 公开领域事件
├── error.rs                              # 程序错误枚举
├── failure.rs                            # 规则失败枚举（业务流程分支）
│
├── rules/                                # 纯业务规则——没有 ECS 依赖
│   ├── mod.rs
│   ├── rules.rs                          # 业务约束
│   └── formulas.rs                       # 计算公式（可选）
│
├── systems/                              # ECS 业务系统
│   ├── mod.rs
│   └── <name>_system.rs
│
├── integration/                          # Anti-Corruption Layer（防腐层）
│   ├── mod.rs
│   ├── facade.rs                         # 外观接口
│   ├── query.rs                          # 查询封装
│   │
│   └── <capability>/                     # 按 Capability 拆分的集成层
│       ├── facade.rs
│       ├── types.rs
│       └── system_param.rs
│
└── tests/
    ├── unit/
    ├── integration/
    ├── invariant/
    └── fixtures/
```

关键的目录是 **`rules/`** 和 **`integration/`**：

- `rules/` 里的函数是**纯业务逻辑**——例如 Combat 的 `calculate_damage()` 函数接受 `(base_damage, modifiers, critical_multiplier)`，返回 `DamageResult`。它不关心 ECS、不关心实体、不关心 Modifier 管线是怎么实现的。它只是"算数"。

- `integration/` 是 Domain 与 Capabilities 之间的**唯一接口**。Domain 的 System 不能直接 `Query<&AttributeValue>`——必须通过 `integration/` 里的 Facade 来读取。这个隔离层的意思是：Capability 重构时，只需要改 `integration/` 而不需要改 Domain 的业务逻辑。

---

## 5. 15 个 Capability 全景（能力机制层）

302 个 Rust 文件，约 25,000 行代码。分为 4 个层次：

### 5.1 层次一：Foundation Layer（基础数据层）

这 5 个 Capability 是整座大厦的地基——它们定义了"数据长什么样"。

#### Tag——语义分类

> 文件：~20 个 | 关键文件: `types.rs`, `components.rs`, `query.rs`, `lifecycle.rs`, `content.rs`

Tag 系统的核心是回答**"这个实体是什么类型的？"**——而不是"这个实体值多少钱？"（那是 Attribute 的事）。

```rust
// 不是数值，是分类
entity.has_tag("character.humanoid");   // ✅ 语义分类
entity.has_tag("damage.fire");          // ✅ 元素类型
entity.get_attribute("health");         // ❌ 数值查询不是 Tag 的事
```

**关键设计**：
- 使用 bitmask 实现 O(1) 层级判定（父 Tag 自动包含子 Tag）
- TagNamespace 给出 13 个受控命名空间（Character/Ability/Status/Equipment/Item/Damage 等），不允许随便造命名空间
- TagHierarchy 在加载时固定——运行时不能创建新 Tag
- Tag 不参与业务规则——你不能写 `if target.has_tag("boss") { damage *= 2 }`，这应该是强类型（`DamageType::Fire`）+ Condition 的职责
- 标签不能表达动态状态——`Character.Dead` 应该是 ECS Component，不是 Tag

**事件**：`TagAdded`, `TagRemoved`, `TagHierarchyChanged`

#### Attribute——数值属性

> 文件：~20 个 | 关键文件: `types.rs`, `values.rs`, `components.rs`, `lifecycle.rs`

Attribute 定义"这个实体有多少"——HP、MP、力量、敏捷、等级。它只描述数值的"存在"，不描述数值的计算方式。

```rust
pub struct AttributeValue {
    pub base_value: f32,        // 职业/种族等给的原始值
    pub current_value: f32,     // 经过 Modifier 管线计算后的最终值
}
```

**关键设计**：
- AttributeCategory 分 4 类：Primary（力量/敏捷/体质/智力/感知/魅力）、Secondary（AC/先攻/熟练）、Derived（生命值/法术位上限）、Resource（当前 HP/MP）
- BaseValue 和 CurrentValue 严格分离——任何来自 Modifier 的变化都只影响 CurrentValue
- 永远不能绕过 Aggregator 管线直接设置 CurrentValue

**事件**：`AttributeChanged`, `AttributeInitialized`, `SnapshotTaken`, `AttributeClamped`

#### Modifier——数值修饰原子操作

> 文件：~16 个 | 关键文件: `types.rs`, `values.rs`, `components.rs`, `lifecycle.rs`

Modifier 是整个系统的"最小修改单位"。它只是一个描述——"我想改这个数值，用 Add +5 的方式"——它不执行修改，只是被注册到容器里等 Aggregator 来聚合。

```rust
pub enum ModifierOp {
    Add(f32),           // +5
    Multiply(f32),      // ×1.2
    Override(f32),      // = 15（覆盖为 15）
}

pub struct Modifier {
    pub target_attribute: AttributeId,  // 目标属性
    pub op: ModifierOp,                 // 操作类型
    pub priority: u8,                   // 优先级（越低越优先）
    pub source: ModifierSourceType,     // 来源类型（Buff/装备/天赋/环境）
}
```

**关键设计**：
- 一个 Modifier 只改一个属性
- 活跃期间不可变——必须先移除再重新注册
- 必须有可追溯的来源——不允许"孤儿 Modifier"
- Modifier 本身不包含业务逻辑——修饰器不知道自己是来自"火焰刀 Buff"还是"铁甲装备"

**事件**：`ModifierApplied`, `ModifierRemoved`, `ModifierSuppressed`, `ModifierStaleDetected`

#### Aggregator——属性聚合管线

> 文件：~18 个 | 关键文件: `types.rs`, `values.rs`, `pipeline.rs`, `events.rs`

Aggregator 把一个个独立的 Modifier 聚合成最终的属性值。这是纯粹的数学计算——不涉及任何业务逻辑。

**四阶段管线**：

```
BaseValue（100）
    │
    ▼ Stage 1: Add
    100 + 5 (来自装备) + 3 (来自 Buff) = 108
    │
    ▼ Stage 2: Multiply
    108 × 1.2 (来自天赋) = 129.6
    │
    ▼ Stage 3: Override
    （如果没有 Override 则跳过） 
    │
    ▼ Stage 4: Clamp
    clamp(129.6, 0, 999) = 129.6
    │
    ▼ FinalValue = 129.6
```

**脏标记模式**：

```
Clean → [ModifierApplied/Removed 事件] → Dirty → [帧末批量重算] → Computing → [完成] → Clean
```

不是每次 Modifier 变化都立即重算——而是在帧末批量处理被标记为脏的属性。重复的脏标记会被去重。

**事件**：`AggregationComplete`, `AggregateDirty`, `PipelineCycleDetected`

#### GameplayContext——行为数据载体

> 文件：~13 个 | 关键文件: `types.rs`, `values.rs`, `error.rs`, `builder.rs`

一次游戏行为（攻击/施法/使用物品）中，有很多数据需要在各个系统之间传递——来源是谁、目标是谁、用了什么能力、什么武器、什么元素类型。GameplayContext 就是装这些东西的"信封"。

```rust
pub struct GameplayContext {
    pub source: Entity,           // 来源
    pub target: Entity,           // 目标
    pub ability: Option<AbilityDefId>,
    pub weapon: Option<ItemId>,
    pub element: ElementType,
    pub context_chain: ContextChain,   // 链式追溯
    pub status: ContextStatus,         // Building → Validated → Active → Consumed → Archived
}
```

**关键设计**：
- 验证后不可变——一旦进入 Active 状态，所有字段只读
- ContextChain 带循环检测——最多 10 跳，防止无限触发链
- 必须用 ContextBuilder 构造——禁止直接构造

---

### 5.2 层次二：Logic Skeleton Layer（配置/条件层）

这 4 个 Capability 是"让数据能做什么"的桥梁。

#### Spec——Def → Instance 桥梁

> 文件：~10 个 | 关键文件: `types.rs`, `values.rs`, `components.rs`

Spec 是 Definition（配置数据，在 `content/` 中）和 Instance（运行时实体）之间的桥梁。

```rust
// Def（来自配置文件，全局只读）
AbilityDef { id: "abl_000042", name: "火球术", damage: 50 }

// Spec（运行时、绑定到实体）
AbilitySpec { def_id: "abl_000042", level: 3, cooldown_override: None }

// Instance（运行时、每个激活实例一份）
AbilityInstance { spec_id: ?, current_cooldown: 0, state: Ready }
```

**关键设计**：
- Def 运行时不可变—Spec 可变
- Spec 必须引用一个已注册的 Def
- Spec 被移除时，级联终止所有关联的 Instance

**事件**：`SpecGranted`, `SpecRemoved`, `SpecLevelChanged`, `SpecSnapshotTaken`

#### Condition——条件/免疫判定

> 文件：~14 个 | 关键文件: `types.rs`, `values.rs`, `evaluator.rs`

统一的条件判定系统。纯查询——没有副作用。

```rust
pub enum ConditionOp { And, Or, Not }          // 逻辑组合
pub enum ComparisonOp { Equal, GreaterThan, LessThan, ... }  // 比较操作
pub enum ConditionResult { Passed, Failed { reason } }       // 判定结果
```

典型的条件检查：
- TagRequirement：实体有没有某个 Tag
- AttributeCheck：HP 是不是 ≤ 0
- ConditionGroup：((Tag.Immune.Fire) OR (Resistance.Fire ≥ 0.5)) AND (不是无敌状态)

**关键设计**：
- 免疫有最高优先级——`Tag.Immune.Fire` 自动使 Fire 相关条件失败。不需要单独的 ImmunitySystem
- AND/OR/NOT 支持短路求值

**事件**：`ConditionPassed`, `ConditionFailed`, `ImmunityTriggered`

#### Trigger——事件驱动能力激活

> 文件：~12 个 | 关键文件: `types.rs`, `values.rs`, `evaluator.rs`

Trigger 是"什么事件触发什么技能"的映射。它监听 Event，检查 Condition，然后触发 Ability。

```rust
pub enum TriggerType {
    OnDamaged, OnHealed, OnAttack, OnTurnStart, OnTurnEnd,
    OnDeath, OnMove, OnAbilityUsed, OnConditionMet, OnCustom,
    // ... 共 11 种
}
```

**关键设计**：
- 必须有目标能力——不能为空触发
- 每回合频率限制——默认每回合 1 次
- 禁止自触发——Trigger 不能监听自己发射的事件

#### Event——Domain 通信基础设施

> 文件：~11 个 | 关键文件: `types.rs`, `values.rs`, `bus.rs`

Event 是**域间通信的唯一通道**。不要和 Trigger 混淆——Event 是"系统到系统的数据通知"，Trigger 是"检测条件→激活技能"。

```rust
pub struct GameplayEvent {
    pub id: EventId,
    pub tag: EventTag,        // DamageDealt, UnitDied, TurnStarted...
    pub source: Entity,
    pub priority: EventPriority,
    pub payload: EventPayload,
}
```

**关键设计**：
- 每个 Event 必须有来源
- 单个订阅者失败不影响其他订阅者
- 同优先级按 FIFO 顺序
- 循环检测：同 EventTag 最多 5 次传播，超出时触发 `PipelineCycleDetected` 告警

---

### 5.3 层次三：Behavior Layer（行为表现层）

这 6 个 Capability 是"数据最终能做什么"的具体实现。

#### Ability——技能生命周期管理

> 文件：~17 个 | 关键文件: `def.rs`, `types.rs`, `error.rs`, `failure.rs`, `lifecycle.rs`, `components.rs`

Ability 是"谁能做什么"的管理者——激活→读条→执行→冷却的生命周期。

```rust
pub enum AbilityState {
    Ready, Casting, Active, Cooldown, Blocked, Removed,
}
```

**状态机**：

```
Ready →（条件检查→消耗资源）→ Casting →（读条完成）→ Active →（执行完成）→ Cooldown →（冷却结束）→ Ready
任何状态 ↔ Blocked（沉默/眩晕）
任何状态 → Removed
```

**组合优先于新建**的设计原则：

| 想要什么 | 错误做法 | 正确做法 |
|---------|---------|---------|
| 法力消耗 | 写个 ManaSystem | Attribute(Mana) + Cost（通过 Effect/Modifier） |
| 冷却 | 写个 CooldownSystem | Tag(Cooldown.X) + Effect(Duration) |
| DoT/HoT | 写个 DoTSystem | Effect(Duration + Period) |
| 免疫 | 写个 ImmunitySystem | Tag(Immune.X) + Condition(Check) |
| 光环 | 写个 AuraSystem | Targeting(Area) + Effect(Infinite) |

**事件**：`AbilityActivated`, `AbilityCompleted`, `AbilityCancelled`, `AbilityCooldownStarted`

#### Targeting——目标选择

> 文件：~12 个 | 关键文件: `types.rs`, `values.rs`, `selector.rs`, `error.rs`

定义一个技能能影响哪些目标，以及怎么选择目标。

**双轴选择器**：

```
TargetType：谁来（Self/Ally/Enemy/Any/Party...）
TargetShape：怎么打（Single/Area/Line/Cone/Chain/Burst/Wall）

验证流程：
TargetType 过滤 → TargetShape 计算 → Selector 精炼 → 范围/视野/阵营/数量校验 → 优先级排序 → max_targets 截断
```

**事件**：`TargetSelected`, `NoValidTarget`

#### Execution——伤害/治疗计算分派器

> 文件：~12 个 | 关键文件: `types.rs`, `values.rs`, `calculator.rs`, `error.rs`

Execution 本身**不包含任何公式**——它只负责把计算需求分派到各个 Domain 的 `rules/formulas.rs`。

```rust
pub enum ExecutionType {
    Damage(DamageParams),
    Heal(HealParams),
    Custom(CustomExecutionRef),
    DirectAttributeMod { attribute_id, operation, value },
    None,
}
```

**分派架构**：

```
Execution（Capability 层）                    Domains/rules/（业务域层）
  ExecutionType::Damage                          damage_formula.rs（Combat 域）
    → dispatch to                                heal_formula.rs
  ExecutionType::Heal                            critical_rules.rs
    → dispatch to                                advantage_rules.rs
  ExecutionType::Custom                          spell_formulas.rs（Spell 域）
    → lookup CustomExecutionRegistry
```

**关键规则**：
- 禁止在 Execution 中出现公式——公式在 Domain 的 `rules/` 里
- 确定性：相同输入→相同输出
- 伤害/治疗 ≥ 0（负值 clamp 到 0）

**事件**：`ExecutionCompleted`, `ExecutionFailed`

#### Effect——效果生命周期

> 文件：~17 个 | 关键文件: `def.rs`, `types.rs`, `error.rs`, `failure.rs`, `lifecycle.rs`

Effect 是所有"结果"的载体——伤害、治疗、Buff、Debuff、DoT、HoT、光环。管理从应用到移除的完整生命周期。

```rust
pub enum EffectStage {
    Applying,           // 条件检查阶段
    Active,             // 活跃阶段（DoT/HoT 周期性触发）
    Expiring,           // 到期阶段（Modifier 回滚）
    Removed,            // 已移除
}
```

**生命周期**：

```
Applying → [条件检查] → Active → [持续时间到期] → Expiring → [Modifier 回滚] → Removed
  └──→ 瞬发效果：执行后直接到 Removed
```

Effect 可以携带 Tag 操作：

```rust
pub struct EffectTags {
    pub granted_tags: Vec<TagId>,            // 应用时给目标加的 Tag
    pub required_tags: Vec<TagId>,           // 目标必须有这些 Tag
    pub ignored_tags: Vec<TagId>,            // 目标不能有这些 Tag（免疫）
    pub removed_tags: Vec<TagId>,            // 移除时清理的 Tag
    pub remove_effects_with_tags: Vec<TagId>, // 应用时移除带这些 Tag 的 Effect
}
```

**事件**：`EffectApplied`, `EffectRemoved`, `EffectTicked`, `EffectImmunityTriggered`

#### Stacking——堆叠规则

> 文件：~14 个 | 关键文件: `types.rs`, `values.rs`, `decider.rs`, `error.rs`

同一种效果连续多次施加时，Stacking 决定"第二次怎么办"。

```rust
pub enum StackingType {
    None,                        // 忽略第二次
    Aggregate,                   // 层叠（有上限）
    RefreshDuration,             // 重置持续时间
    Replace,                     // 新的替换旧的
}
```

**决策流程**：

```
新的 Effect 到达 → 搜索已有 Effect 中同 EffectDefId 的
  → 没找到：首次施加，堆叠数=1
  → 找到了：StackingType 策略：
      None → 拒绝新的
      Aggregate → 堆叠层数 += n，检查上限
      RefreshDuration → 重置剩余时间
      Replace → 比较优先级（新的高→替换；旧的≥新→保留）
```

**事件**：`StackAdded`, `StackRemoved`, `StackRefreshed`, `StackReplaced`, `StackOverflow`

#### Cue——表现信号

> 文件：~15 个 | 关键文件: `types.rs`, `values.rs`, `dispatch.rs`, `components.rs`

Cue 是"应该播放什么表现"的信号——不关心怎么播，只关心该播了。

```rust
pub enum CueType {
    VFX(VFXParams),
    SFX(SFXParams),
    Animation(AnimationParams),
    Shake(ShakeParams),
    Popup(PopupParams),
}
```

**关键设计**：
- 单向：表现层不能通过 Cue 反向影响逻辑
- 非阻塞：Cue 失败绝不阻塞游戏逻辑
- 可独立禁用：支持无头模式/性能模式
- 不包含敏感数据：Cue 不得泄露隐藏游戏信息

---

## 6. 15 个 Domain 全景（业务域层）

394 个 Rust 文件，约 32,400 行代码。分为 4 个战略层：

### 6.1 Foundation Layer（战术空间）

#### Tactical——战术空间（33 文件·2674 行）

SRPG 的物理基础：网格坐标、单位位置、移动范围、战术状态判定（夹击、背刺、掩体、高地）。

**关键类型**：`GridPosition`, `MovementPoints`, `Facing`, `FlankingState`, `BackstabState`, `CoverState`, `HighgroundState`, `PathData`

**关键规则**：
- 执行纯几何/空间判定——从不计算数值加成（加成是 Combat 的事）
- 所有移动必须消耗行动力，路径必须连续
- 单位不可重叠
- **禁止**修改单位属性（通过 Modifier 管线完成）

**集成层**：`integration/movement/` — 通过 `MovementCapabilityParam` SystemParam 向 Combat 暴露移动能力

#### Terrain——地形（27 文件·1713 行）

瓦片属性、表面类型、地形效果、危险区域/陷阱、表面转变。

**关键类型**：`TileProperties`, `SurfaceType`, `TerrainEffect`, `HazardZone`, `TerrainInteraction`

**关键规则**：
- 地形效果绑定到瓦片，而非单位——单位效果归 Effect 域
- 所有表面变化必须可逆（定时或驱散恢复）
- 同一瓦片不能同时存在互斥表面类型（如冰面+燃烧）

**事件**：`TileEntered`, `SurfaceChanged`, `HazardTriggered`, `TerrainEffectApplied`

#### Faction——阵营关系（23 文件·1562 行）

阵营归属、声望管理（-100 到 +100）、阵营间关系（盟友/中立/敌对/战争）。

**关键类型**：`Faction`, `FactionMembership`, `Reputation`, `ReputationLevel`, `FactionRelation`, `RelationshipState`

**关键规则**：
- 声望值 clamp 在 [-100, +100]，变化必须有原因
- 声望等级必须逐级跨越（不可跳过中间等级）
- **禁止**控制 AI 行为、修改交易价格或对话选项

**事件**：`ReputationChanged`, `FactionRelationChanged`, `ReputationLevelUp`, `RelationshipEvaluated`

### 6.2 Core Layer（战斗核心）

#### Combat——战斗（87 文件·6986 行）

★ 项目的中心枢纽。回合流程、伤害结算、死亡处理、胜负判定。

**关键类型**：`CombatState`, `TurnOrder`, `InitiativeValue`, `CombatParticipant`, `Dead`, `DamageResult`, `DamageDecision`, `VictoryCondition`, `CombatIntent`

**关键规则**：
- **CombatIntent 是唯一攻击入口**——没有绕过路径
- 回合严格按先攻顺序交替，先攻排序在战斗中不变
- 同时只有一个单位处于"行动中"状态
- 战斗结束不可逆
- 伤害规则基于 D&D 5e：d20 + 熟练 + 属性调整 >= AC；自然 20 = 暴击，自然 1 = 必然未命中

**集成层（最大的 Integration）**：通过 9 个 Capability 集成：
`ability/`, `aggregator/`, `condition/`, `effect/`, `event/`, `execution/`, `gameplay_context/`, `targeting/`, `trigger/`
+ `replay/`（4 个子目录：录制/播放/注册表）
+ `turn/`（回合集成）

**Pipeline**：`pipeline/` 模块驱动回合循环：TurnStart → PhaseCheck → UnitAction（暂停等待输入）→ TurnSettlement → TurnEnd

**事件（8 个——所有 Domain 中最多）**：`CombatStarted`, `TurnBegin`, `TurnEnd`, `RoundStart`, `RoundEnd`, `DamageDealt`, `CombatEnded`, `UnitDied`

#### Spell——法术（21 文件·2510 行）

法术施放、专注管理、豁免检定、升环施法。

**关键类型**：`SpellDef`, `SpellDefId`, `SpellLevel`, `SpellComponents`, `CastingTime`, `SpellRange`, `SpellDuration`, `SpellSlots`, `SaveType`, `SaveDC`

**关键规则**：
- 法术位不可透支
- 专注唯一性：同一时间仅维持一个专注法术
- 专注打断 DC = max(10, 所受伤害/2)
- 升环施法使用更高级法术位以获得更强效果

**事件**：`SpellCast`, `SpellSlotChanged`, `ConcentrationStarted`, `ConcentrationBroken`, `SaveRolled`

#### Reaction——反应（20 文件·1967 行）

回合外行动机制：反应槽位管理、机会攻击、法术反制、护盾术、援护。

**关键类型**：`ReactionState`, `ReactionQueue`

**关键规则**：
- 每回合 1 次反应（回合外触发）
- 机会攻击仅因主动离开威胁范围而触发（非强制移动）
- 护盾术在命中判定后、伤害计算前使用（AC +5）
- **禁止**己方回合主动反应

**事件**：`ReactionTriggered`, `ReactionExecuted`, `ReactionDeclined`, `OpportunityAttackExecuted`, `CounterspellExecuted`

#### Progression——成长养成（23 文件·2413 行）

经验获取、升级、多职业、天赋树、子职业选择、属性值提升（ASI）。

**关键类型**：`Experience`, `ClassLevels`, `ClassLevelEntry`, `ClassId`, `TalentTree`, `TalentId`, `SubclassChoice`, `SubclassId`

**关键规则**：
- 最高等级 20，经验只增不减
- 熟练加值基于总等级（1-20 级分 4 段）
- ASI 在 4/8/12/16/19 级触发，每项属性最高 20
- 天赋前置链必须完整，子职业选择唯一且不可更改

**事件**：`ExperienceGained`, `LevelUp`, `TalentUnlocked`, `SubclassChosen`, `ASICompleted`, `ClassGained`

#### Inventory——背包/物品（22 文件·2183 行）

物品管理、装备系统（11 槽位）、消耗品使用、战利品生成。

**关键类型**：`Inventory`, `EquipmentSlots`, `ItemInstance`

**关键规则**：
- 11 个装备槽位（主手/副手/头盔/铠甲/手套/靴子/披风/戒指×2/项链/特殊）
- 双手武器占用主手+副手
- 装备必须检查穿戴条件
- 消耗品在效果确认生效后消耗
- **禁止**装备直接修改属性（通过 Modifier 管线）

**事件**：`ItemAcquired`, `ItemUsed`, `EquipmentChanged`, `ItemRemoved`, `LootGenerated`

#### Party——队伍（18 文件·1390 行）

队伍成员管理、战斗换人、羁绊、队伍全局增益。

**关键类型**：`PartyRoster`, `BondState`, `PartyBuff`

**关键规则**：
- 战斗人数上限 4 人，队伍总上限 12 人
- 战斗换人消耗 1 行动力
- 队伍成员变化时实时重新评估羁绊

#### CampRest——营地/休息（19 文件·1320 行）

短休、长休、生命骰管理、营地事件与 NPC 交互。

**关键类型**：`RestState`, `ShortRest`, `LongRest`, `HitDice`, `CampNPC`, `CampEvent`

**关键规则**：
- D&D 5e 规则：短休 1 小时，长休 8 小时
- 长休每 24 小时 1 次，中断累计超过 1 小时失效
- 生命骰恢复上限 = 角色等级的一半（向下取整）
- **禁止**战斗中休息

### 6.3 Narrative Layer（叙事内容）

#### Narrative——叙事/对话（21 文件·1530 行）

对话树流程、故事标记（仅增不减）、过场动画、条件分支过滤。

**关键类型**：`DialogueState`, `StoryFlag`, `CutsceneState`, `DialogueNode`, `ChoiceCondition`

**关键规则**：
- 对话树必须无环
- StoryFlag 仅增不减——关键剧情选择不可撤销
- 互斥分支不可同时出现
- **禁止**对话选项直接修改属性/物品（通过事件解耦）
- **禁止**对话节点使用自然语言文本（必须使用 LocalizationKey）

**事件**：`DialogueStarted`, `ChoiceMade`, `StoryFlagSet`, `CutsceneStarted`, `CutsceneEnded`

#### Quest——任务（20 文件·1767 行）

任务生命周期（不可用→可用→进行中→可交付→已完成/失败）、7 种目标类型、事件驱动进度。

**关键类型**：`QuestDef`, `QuestDefId`, `ObjectiveDef`, `ObjectiveId`, `ObjectiveType`, `QuestRewardDef`, `QuestType`

**关键规则**：
- 前置条件完整链，进度不可倒退
- 奖励仅发放一次，已完成任务不可重新激活
- 关键任务不可放弃或失败
- **禁止**轮询进度——必须完全事件驱动

**事件**：`QuestAccepted`, `ObjectiveCompleted`, `QuestTurnedIn`, `QuestFailed`, `QuestProgressUpdated`

### 6.4 Economy Layer（经济系统）

#### Economy——经济/交易（20 文件·1651 行）

货币系统（三级 GP/SP/CP）、商店、交易流程、价格计算、赃物处理。

**关键类型**：`Wallet`, `ShopInventory`, `PriceModifier`, `Currency`

**关键规则**：
- 货币必须非负，价格在同条件下确定
- 声望折扣在交易时刻锁定
- 赃物标记不可清除，售价 ×0.5
- **禁止** Economy 直接创建/销毁物品（归 Inventory 管理）

#### Crafting——制作/锻造（19 文件·1280 行）

5 种制作类型、附魔系统、装备升级。

**关键类型**：`RecipeBook`, `CraftingStation`, `EnchantmentSlot`

**关键规则**：
- 附魔槽位有限（武器 ≤3/铠甲 ≤2/饰品 ≤1），同类型词条互斥
- 增强等级受稀有度限制（普通 ≤0/非凡 ≤+1/稀有 ≤+2/史诗 ≤+3）
- **禁止**产出直接包含 Modifier/Effect（产出的是物品实例）

#### Summon——召唤（20 文件·1381 行）

召唤物创建与绑定、持续时间管理、AI 模式控制、槽位限制。

**关键类型**：`SummonBond`, `SummonDuration`, `SummonAI`, `SummonSlot`

**关键规则**：
- 召唤者死亡→所有相关召唤物立即消失
- 召唤物拥有独立行动回合
- **禁止**嵌套召唤

---

## 7. 集成层（Integration / Anti-Corruption Layer）

这是 Domain 轴和 Capability 轴之间最重要的结构。

### 为什么需要集成层

假设 Combat Domain 需要查询某个实体的 HP 值。HP 是 Attribute Capability 管理的。如果不加隔离，Combat 的 System 里会出现：

```rust
// ❌ 错误做法——直接查询 Capability 的内部
fn combat_system(query: Query<(&CombatParticipant, &AttributeValue)>) { ... }
```

这样 Capability 如果重构 `AttributeValue` 的结构（比如改成 `AttributeContainer`），所有引用它的 Domain 都得改。这就是"耦合"。

### Facade + SystemParam 模式

集成层用两个东西解决这个问题：

```rust
// ✅ 正确做法——通过 Facade 查询
// integration/combat/combat_capability_param.rs
pub struct CombatCapabilityParam<'w, 's> {
    attribute_facade: AttributeQuery<'w, 's>,
    condition_facade: ConditionQuery<'w, 's>,
    effect_facade: EffectQuery<'w, 's>,
}

impl CombatCapabilityParam<'_, '_> {
    pub fn get_hp(&self, entity: Entity) -> Option<f32> {
        self.attribute_facade.get_current_value(entity, HP_ATTRIBUTE_ID)
    }
    pub fn is_immune_to(&self, entity: Entity, damage_type: &str) -> bool {
        self.condition_facade.check_immunity(entity, damage_type)
    }
}
```

**关键设计点**：
- Facade 是 Domain 和 Capability 之间的唯一接口——Domain 的 System 不能持有 Capability 的 Component Query
- Facade 提供业务语义 API——`get_hp()` 而不是 `get_attribute("attr_000001").current_value`
- Capability 重构时，只需要改 Facade 内部映射，不改 Domain 业务代码
- Facade 方法是纯查询——不触发副作用

### 集成层文件结构

```
integration/
├── mod.rs
├── facade.rs                    # 通用 Facade 入口
├── query.rs                     # 通用查询封装
│
└── <capability>/                # 按 Capability 拆分
    ├── facade.rs                # 该 Capability 的外观接口
    ├── types.rs                 # 视图类型（View Types）
    └── system_param.rs          # SystemParam 实现
```

---

## 8. 四级通信体系

Domain 层内部和跨层的通信遵循四级体系（在 `docs/08-knowledge/communication-overview.md` 中有完整说明）：

| 级别 | 机制 | 方向 | 用途 |
|------|------|------|------|
| Hook | trait 回调 | Capability 内部 | 执行过程中的扩展点（如 PipelineHook） |
| Trigger | 条件→能力映射 | Capability → Capability | 事件驱动能力激活 |
| Observer | Bevy 0.19 `On<>` | Domain → Domain / Capability → Domain | 域间解耦通信 |
| Message | CommandQueue | 外部 → Core | 输入/UI → Domain |

**域间通信的原则**：Domain 之间不能直接引用对方的数据结构。Combat 想知道 Inventory 有没有某件物品？通过 Event（Combat 发射 `DamageDealt`，Inventory 监听）——而不是 `Query<&Inventory>`。

---

## 9. 失效模式：Rule Failure vs 程序 Error

项目遵循 ADR-051 的错误隔离体系。Domain 层同时使用两种失效机制：

### Rule Failure（业务流程分支）

当用户的操作不符合业务规则时——比如"装备需要等级 10 但你只有 5"——这不是程序错误，而是业务预期内的分支。

```rust
// combat/failure.rs —— 业务规则失败
pub enum CombatFailure {
    OutOfRange,
    LineOfSightBlocked,
    InvalidTarget(InvalidTargetReason),
    InsufficientActionPoints,
}
```

这些 failure 类型会被返回给调用方（UI/输入层），用于显示"目标不在射程内"这样的提示。

### Program Error（系统错误）

当数据缺失或系统状态不一致时——比如"应该存在的 Modifier 找不到了"——这是程序错误，需要修复代码而不是提示用户。

```rust
// combat/error.rs —— 系统错误
pub enum CombatError {
    MissingComponent(&'static str),
    StateMismatch { expected: &'static str, actual: &'static str },
    PipelineNotFound(String),
}
```

**关键规则**：
- Rule Failure 永远不能 panic——它是业务预期分支
- Program Error 可以 panic 或返回 Result——它表示系统状态不该这样
- failure 枚举在 `failure.rs` 中，error 枚举在 `error.rs` 中，严格分离

---

## 10. 跨领域关键流程

几个最重要的跨域流程：

### 战斗攻击流程（跨域最多的流程）

```
1. 玩家选择"攻击"按钮
2. CombatIntent 创建 → 唯一攻击入口
3. Tactical：判定射程/夹击/掩体/高地
4. Targeting：选择目标
5. Reaction：插入回合外行动？(守护/反击/法术反制)
6. Execution：分派到 Combat/rules/ 的伤害公式
7. Effect：应用伤害结果（可能触发 Buff/Debuff）
8. Event：广播 DamageDealt
    ├─ Progression → 检查是否获得经验
    ├─ Inventory → 检查战利品掉落
    ├─ Quest → 检查任务进度更新
    └─ Spell → 检查专注是否被打断
9. Dead 判定 → 战斗结束检查
```

### 休息流程

```
1. CombatEnded（Event）
2. CampRest → 开始休息流程
3. 短休：消耗生命骰 → 恢复 HP
4. 长休：
    ├─ Spell → 恢复法术位
    ├─ Party → 调整队伍
    ├─ Narrative → 触发营地事件
    └─ Progression → 检查升级
```

### 交易流程

```
1. 玩家打开商店
2. Faction（声望）→ Economy（计算折扣后价格）
3. 交易确认：
    ├─ Wallet → 扣钱
    └─ Inventory → 给物品
4. Event → Quest（检查任务进度）
```

### 制作流程

```
1. Crafting → 检查材料/工具/技能 DC
2. 消耗材料 → 等待制作时间
3. 成品进入 Inventory
4. 穿戴时通过 Modifier 管线应用属性加成
```

---

## 11. 当前实现状态

### 数量总览

| 群体 | Rust 文件数 | 代码行数 | Domain 规则文档数 |
|------|------------|---------|-----------------|
| Capabilities | 302 | ~25,000 | 15（✅ stable） |
| Domains | 394 | ~32,400 | 15（✅ stable） |
| **总计** | **696** | **~57,400** | **30** |

### 各 Domain 规模排名

| 排名 | Domain | 文件数 | 行数 | 特点 |
|------|--------|--------|------|------|
| 1 | Combat | 87 | 6,986 | 项目枢纽，9 个 Capability 集成 |
| 2 | Tactical | 33 | 2,674 | 网格+空间判定 |
| 3 | Terrain | 27 | 1,713 | 地形系统 |
| 4 | Progression | 23 | 2,413 | 成长+天赋树 |
| 5 | Faction | 23 | 1,562 | 阵营声望 |
| 6 | Inventory | 22 | 2,183 | 背包+装备 |
| 7 | Spell | 21 | 2,510 | 法术系统 |
| 8 | Narrative | 21 | 1,530 | 叙事+对话 |
| 9 | Reaction | 20 | 1,967 | 回合外行动 |
| 10 | Economy | 20 | 1,651 | 经济交易 |
| 11 | Quest | 20 | 1,767 | 任务系统 |
| 12 | Summon | 20 | 1,381 | 召唤系统 |
| 13 | CampRest | 19 | 1,320 | 营地休息 |
| 14 | Crafting | 19 | 1,280 | 制作锻造 |
| 15 | Party | 18 | 1,390 | 队伍管理 |

### Capability 依赖链完成度

```
Tag ──────→ Condition ──────→ Trigger
  │                              │
  ├──→ Modifier ──→ Aggregator   │
  │                    │         │
  │                    ▼         ▼
  │              Attribute    Ability
  │                    │         │
  │                    │         ▼
  │                    └──→ Targeting ←── GameplayContext
  │                              │
  │                              ▼
  │                         Execution
  │                              │
  │                              ▼
  └────────────────────────→ Effect ←── Spec
                                   │
                            ┌──────┼──────┐
                            ▼      ▼      ▼
                         Stacking Cue   Event
```

所有 Capability 的依赖链在架构层面是完整的——但在代码实现层面，部分集成层（Integration Facade）尚未完全连接。

---

## 12. 全部相关文件索引

### Domain 规则文档（34 文件·9,122 行）

```
docs/02-domain/
├── README.md                                     344 行 — 索引 + 依赖图
├── ubiquitous_language.md                         30 行 — 项目通用术语
├── factories.md                                  430 行 — 实体工厂约定
├── capabilities/（16 文件）
│   ├── tag_domain.md                             390 行
│   ├── attribute_domain.md                       233 行
│   ├── modifier_domain.md                        223 行
│   ├── aggregator_domain.md                      239 行
│   ├── gameplay_context_domain.md                215 行
│   ├── spec_domain.md                            255 行
│   ├── condition_domain.md                       229 行
│   ├── trigger_domain.md                         256 行
│   ├── event_domain.md                           242 行
│   ├── ability_domain.md                         252 行
│   ├── targeting_domain.md                       220 行
│   ├── execution_domain.md                       233 行
│   ├── effect_domain.md                          253 行
│   ├── stacking_domain.md                        279 行
│   ├── cue_domain.md                             237 行
│   └── ui-presentation.md                        595 行
└── domains/（15 文件）
    ├── tactical_domain.md                        304 行
    ├── terrain_domain.md                         227 行
    ├── faction_domain.md                         230 行
    ├── combat_domain.md                          356 行
    ├── spell_domain.md                           280 行
    ├── reaction_domain.md                        288 行
    ├── progression_domain.md                     287 行
    ├── inventory_domain.md                       260 行
    ├── party_domain.md                           235 行
    ├── camp_rest_domain.md                       265 行
    ├── narrative_domain.md                       230 行
    ├── quest_domain.md                           257 行
    ├── economy_domain.md                         238 行
    ├── crafting_domain.md                        261 行
    └── summon_domain.md                          249 行
```

### 架构 ADR（10+ 关键文件）

```
docs/01-architecture/
├── README.md                                     752+行 — 架构总纲（含 §2 双轴说明）
├── 00-foundation/
│   ├── ADR-000-feature-module-map.md              — Capabilities/Domains 分界
│   ├── ADR-001-plugin-composition.md              — 7 阶段注册顺序
│   ├── ADR-045-module-visibility-strategy.md       — 模块可见性规则
│   ├── ADR-046-module-interface-pattern.md         — integration/ ACL 模式
│   └── ADR-056-agent-governance.md                 — Agent 角色定义
├── 10-capability-system/
│   ├── ADR-010-ability-pipeline.md                 — Ability 六阶段管线
│   ├── ADR-011-modifier-pipeline.md                — Modifier 四阶段管线
│   └── ADR-012-stacking-trigger-cue.md             — Stacking/Trigger/Cue 边界
└── 20-tactical-combat/
    ├── ADR-020-combat-pipeline.md                  — Combat 七阶段管线
    ├── ADR-021-turn-state-machine.md               — 回合状态机
    ├── ADR-024-combat-integration-layer.md          — Combat integration/ 详细设计
    └── ADR-022-grid-terrain-faction.md             — 网格/地形/阵营
```

### 代码实现

```
src/core/
├── capabilities/（302 文件·~25,000 行）
│   ├── tag/              ~20 文件   — 语义分类，bitmask O(1)
│   ├── attribute/        ~20 文件   — 数值属性，Base/Current 分离
│   ├── modifier/         ~16 文件   — 修饰原子操作（Add/Multiply/Override）
│   ├── aggregator/       ~18 文件   — 四阶段属性聚合管线
│   ├── gameplay_context/ ~13 文件   — 行为数据载体
│   ├── spec/             ~10 文件   — Def→Instance 桥梁
│   ├── condition/        ~14 文件   — 统一条件判定
│   ├── trigger/          ~12 文件   — 事件驱动能力激活
│   ├── event/            ~11 文件   — 域间通信基础设施
│   ├── ability/          ~17 文件   — 技能生命周期
│   ├── targeting/        ~12 文件   — 目标选择
│   ├── execution/        ~12 文件   — 公式分派器
│   ├── effect/           ~17 文件   — 效果生命周期
│   ├── stacking/         ~14 文件   — 堆叠规则
│   ├── cue/              ~15 文件   — 表现信号
│   ├── rule/              ~? 文件   — 统一规则引擎
│   └── runtime/           ~? 文件   — C3 编排（pipeline/scheduler/registry/command/replay）
│
└── domains/（394 文件·~32,400 行）
    ├── tactical/         33 文件/2,674 行   — 战术空间
    ├── terrain/          27 文件/1,713 行   — 地形
    ├── faction/          23 文件/1,562 行   — 阵营关系
    ├── combat/           87 文件/6,986 行   — ★ 战斗枢纽
    ├── spell/            21 文件/2,510 行   — 法术系统
    ├── reaction/         20 文件/1,967 行   — 回合外行动
    ├── progression/      23 文件/2,413 行   — 成长养成
    ├── inventory/        22 文件/2,183 行   — 背包物品
    ├── party/            18 文件/1,390 行   — 队伍管理
    ├── camp_rest/        19 文件/1,320 行   — 营地休息
    ├── narrative/        21 文件/1,530 行   — 叙事对话
    ├── quest/            20 文件/1,767 行   — 任务系统
    ├── economy/          20 文件/1,651 行   — 经济交易
    ├── crafting/         19 文件/1,280 行   — 制作锻造
    └── summon/           20 文件/1,381 行   — 召唤系统
```

### 数据 Schema（docs/04-data/ 中的对应文档）

每个 Domain/Capability 有对应的 Schema 文档（`docs/04-data/capabilities/<name>_schema.md` 和 `docs/04-data/domains/<name>_schema.md`），详细定义了四层数据结构（Def/Spec/Instance/Persistence）。

---

> **本篇为 Knowledge Base 文档，目标是"理解 Domain 层是如何组织、为什么这样设计"。**
> 如需正式的规范定义，见 `docs/02-domain/` 和 `docs/01-architecture/` 的 ADR。
> 如需代码实现细节，见 `src/core/capabilities/` 和 `src/core/domains/` 各模块。
