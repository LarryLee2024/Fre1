---
id: 02-domain.GAS.GAS_domain_overview
title: GAS Domain Overview
status: proposed
owner: domain-designer
created: 2026-06-16
updated: 2026-06-16
tags:
  - domain
  - GAS
  - overview
---

# SRPG-GAS 领域全景

Version: 1.0
Status: Proposed

> 本文档是 `docs/02-domain/GAS/` 下 14 个子领域的全景视图，阐明概念之间的内在联系、数据流转、业务流程和关键实现路径。各子领域的详细规则请参阅对应的 `*-rules.md` 文档。

---

## 1. 统一术语

### 1.1 GAS 核心链路术语

| 术语 | 定义 | 职责边界 | 详细文档 |
|------|------|----------|----------|
| **Ability** | 单位可执行的战斗能力，包含效果列表和消耗 | 负责：验证和路由；不负责：效果执行 | `ability/ability-rules.md` |
| **Targeting** | 决定 Ability 可以作用于哪些实体的规则引擎 | 负责：目标解析；不负责：效果执行 | `targeting/targeting-rules.md` |
| **Effect** | 能力执行后产生的原子性结果（Damage/Heal/ApplyBuff/Cleanse） | 负责：效果执行管线；不负责：数值计算 | `effect/effect-rules.md` |
| **Execution** | 将效果的计算逻辑封装为独立 Trait 的执行单元 | 负责：公式计算；不负责：属性修改 | `execution/execution-rules.md` |
| **Formula** | 游戏数值的纯函数计算方法 | 负责：数值计算；不负责：随机逻辑 | `formula/formula-rules.md` |
| **AttributeModifier** | 属性的三层分类和修饰器栈的计算管线 | 负责：属性计算和修饰；不负责：Buff 生命周期 | `attribute-modifier/attribute-modifier-rules.md` |
| **Tag** | 用 u64 位掩码实现的 O(1) 分类标签系统 | 负责：分类查询；不负责：业务逻辑 | `tag/tag-rules.md` |
| **Cue** | 统一的表现事件总线，强制 Logic/Presentation 分离 | 负责：表现信号定义；不负责：动画/音效实现 | `cue/cue-rules.md` |

### 1.2 辅助子系统术语

| 术语 | 定义 | 职责边界 | 详细文档 |
|------|------|----------|----------|
| **Requirement** | 技能释放前必须满足的条件（"能不能放"） | 负责：释放前检查；不负责：效果执行时检查 | `requirement/requirement-rules.md` |
| **Cost** | 技能释放需要支付的资源（MP/HP/怒气等） | 负责：消耗校验和扣除；不负责：冷却管理 | `cost/cost-rules.md` |
| **Condition** | 效果执行时的触发条件判断（"是否生效"） | 负责：效果执行时条件判断；不负责：释放前检查 | `condition/condition-rules.md` |
| **Duration** | Buff/Effect 的持续时长规则 | 负责：持续策略和过期检测；不负责：Buff 施加移除 | `duration/duration-rules.md` |
| **StackPolicy** | Buff 重复施加时的叠层、刷新和替换规则 | 负责：叠层判定；不负责：效果执行 | `stack-policy/stack-policy-rules.md` |
| **Trigger** | Buff 效果触发的时机点和触发链管理 | 负责：触发时机和链深度控制；不负责：效果执行 | `trigger/trigger-rules.md` |

### 1.3 核心概念辨析

#### "能不能放" vs "是否生效"

| 维度 | Requirement | Condition |
|------|-------------|-----------|
| 时机 | 技能释放前 | 效果执行时 |
| 语义 | 技能能不能放 | 效果是否生效 |
| 示例 | 冷却中、被沉默、无武器 | 目标 HP<30%、施法者已移动 |
| 失败行为 | 技能按钮灰显，不进入后续阶段 | 静默跳过该 ConditionalEffect |

#### Effect vs Modifier

| 维度 | Effect | Modifier |
|------|--------|----------|
| 本质 | "做什么"（造成伤害、恢复生命、施加 Buff） | "怎么调整数值"（暴击倍率、元素克制系数） |
| 阶段 | Execute 阶段执行，修改 World 状态 | Modify 阶段执行，只修改 PendingEffectData.amount |
| 示例 | Damage、Heal、ApplyBuff | 暴击、元素克制、地形加成 |

---

## 2. 领域全景架构图

```
┌─────────────────────────────────────────────────────────────────┐
│                    SRPG-GAS 领域全景                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐    │
│  │ Ability  │──→│Targeting │──→│ Effect   │──→│  Cue     │    │
│  │ 能力管理  │   │目标选择   │   │ 效果管线  │   │表现事件   │    │
│  └────┬─────┘   └──────────┘   └────┬─────┘   └──────────┘    │
│       │                              │                          │
│  ┌────▼─────┐                  ┌────▼─────┐                    │
│  │Requirement│                 │Execution │                    │
│  │释放前提   │                 │执行算式   │                    │
│  └────┬─────┘                  └────┬─────┘                    │
│       │                              │                          │
│  ┌────▼─────┐                  ┌────▼─────┐                    │
│  │  Cost    │                  │ Formula  │                    │
│  │消耗管理   │                 │公式系统   │                    │
│  └──────────┘                  └──────────┘                    │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              Modifier Chain（修饰链）                      │  │
│  │  暴击 → 元素克制 → 地形加成 → 职业克制 → Buff加成 → 天气   │  │
│  └──────────────────────────┬───────────────────────────────┘  │
│                              │                                  │
│  ┌──────────┐          ┌────▼─────┐   ┌──────────┐            │
│  │  Tag     │◄────────│Attribute │   │  Trigger │            │
│  │标签分类   │         │Modifier  │   │触发器系统  │            │
│  └──────────┘          │属性修饰   │   └────┬─────┘            │
│                        └──────────┘        │                    │
│                                           │                    │
│  ┌──────────┐   ┌──────────┐   ┌────────▼─────┐              │
│  │Duration  │◄──│StackPolicy│◄──│  Condition  │              │
│  │持续策略   │   │叠层策略   │   │条件系统      │              │
│  └──────────┘   └──────────┘   └──────────────┘              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. 数据流转关系

### 3.1 GAS 核心链路数据流

```
用户选择 Ability
    ↓
Requirement 检查（纯函数：冷却/消耗/标签/HP 阈值）
    ↓ 通过
Cost 校验（纯函数：MP/HP/怒气是否足够）
    ↓ 通过
Cost 扣除（副作用：set_vital 扣除资源）
    ↓
Targeting 解析（纯函数：施法者坐标 → 目标实体列表）
    ↓
AbilityEffect 路由到 Effect Pipeline
    ↓
┌─ Effect Pipeline ─────────────────────────────────────────┐
│                                                            │
│  Generate（纯函数）                                        │
│  ├─ EffectHandlerRegistry 查找处理器                       │
│  ├─ handler.generate(ctx) → PendingEffectData              │
│  └─ 可选：调用 Formula/Execution 计算初始值                 │
│                                                            │
│  Modify（纯函数）                                          │
│  ├─ ModifierRuleRegistry 遍历                              │
│  ├─ 标签匹配（source_tag + target_tag）                     │
│  ├─ ModifierCalculator 计算修饰                             │
│  ├─ 记录 ModifierEntry                                     │
│  └─ 伤害下限 ≥ 1，治疗下限 ≥ 0                             │
│                                                            │
│  Execute（有副作用）                                        │
│  ├─ EffectHandlerRegistry 查找处理器                       │
│  ├─ handler.execute(ctx) → EffectResult                    │
│  ├─ 发送 DamageApplied / HealApplied / BuffApplied         │
│  └─ 发送 CueEvent → Presentation 层                        │
│                                                            │
└────────────────────────────────────────────────────────────┘
    ↓
EffectResult → Trigger 系统（检查触发链）
    ↓
Buff 触发效果 → 进入同一 Effect Pipeline（深度 ≤ 3）
```

### 3.2 Buff 生命周期数据流

```
Buff Apply（施加）
    ├─ StackPolicy 判定：替换/刷新/叠加/忽略
    ├─ Duration 设置：Turns(n) / UntilXxx / BattleEnd / Permanent
    ├─ Modifier 添加：add_modifiers_from_def
    ├─ Tag 添加：GameplayTags.add()
    └─ CueEvent 发射：BuffAppliedCue

Buff Tick（回合结束）
    ├─ Duration 递减：tick()
    ├─ Trigger 匹配：TurnEnd 触发的 Buff 效果
    └─ Effect Pipeline 执行

Buff Expire（过期）
    ├─ DurationPolicy::Turns(n) → tick 归零
    ├─ DurationPolicy::BattleEnd → 战斗结束
    └─ 移除流程 ↓

Buff Remove（移除）
    ├─ Trigger 失效
    ├─ Modifier 清理：remove_modifiers_from(source)
    ├─ Tag 移除：GameplayTags.remove()
    └─ CueEvent 发射：BuffRemovedCue
```

### 3.3 Tag 三层合并数据流

```
Layer 1: Trait（种族/职业/天赋）
    │ from_traits: GameplayTags
    │
Layer 2: Equipment（装备穿戴）
    │ from_equipment: GameplayTags
    │
Layer 3: Buff（Buff 持续期间）
    │ 由 ActiveBuffs 运行时管理
    │
    ▼
GameplayTags = from_traits | from_equipment | from_buffs
    │
    ▼
下游系统查询（ModifierRule 标签匹配 / AI 决策 / UI 展示）
```

---

## 4. 业务流程逻辑

### 4.1 Ability 释放五阶段管线

严格按 Requirement → Cost → Targeting → Effect → Settlement 顺序执行：

| 阶段 | 职责 | 执行条件 | 失败行为 |
|------|------|----------|----------|
| 1. Requirement | 检查释放前提 | 纯函数验证 | 返回 RequirementError，UI 灰显 |
| 2. Cost | 检查和扣除消耗 | 先校验后扣除 | 返回 CostError，不扣除 |
| 3. Targeting | 解析目标实体列表 | 纯函数解析 | 返回空列表，输出日志 |
| 4. Effect | 通过 Effect Pipeline 执行效果 | Generate→Modify→Execute | 返回 EffectResult |
| 5. Settlement | 发送 CueEvent 和领域事件 | 异步通知 | 无（fire-and-forget） |

### 4.2 Effect Pipeline 三步管线

| 阶段 | 核心操作 | 副作用 | 扩展点 |
|------|----------|--------|--------|
| Generate | EffectHandler.generate() | 无（纯函数） | EffectHandlerRegistry |
| Modify | ModifierRuleRegistry + Calculator | 无（纯函数） | ModifierCalculatorRegistry |
| Execute | EffectHandler.execute() | 有（扣血/加Buff/触发事件） | EffectHandlerRegistry |

### 4.3 Buff 四阶段生命周期

```
Apply → Tick → Expire → Remove
```

| 阶段 | 触发时机 | 核心操作 |
|------|----------|----------|
| Apply | Buff 施加时 | StackPolicy 判定 + Modifier 添加 + Tag 添加 |
| Tick | TurnEnd 阶段 | Duration 递减 + Trigger 匹配 |
| Expire | Duration 归零/BattleEnd | 触发过期事件 |
| Remove | 被驱散/过期/手动 | Modifier 清理 + Tag 移除 + CueEvent |

### 4.4 Trigger 触发链执行流程

```
事件到达（如 AfterDamaged）
    ↓
TriggerRegistry 分发（按 priority 排序）
    ↓
匹配 Handler → Condition 检查 → EffectDef[] 生成
    ↓
压入 ExecutionStack（LIFO, MAX_STACK_DEPTH=32）
    ↓
Stack 弹出 → Effect Pipeline → 递归（深度 ≤ 3）
```

---

## 5. 关键实现路径

### 5.1 Registry 统一模式

所有子系统遵循相同的 Registry 模式：

| 子系统 | Registry | Handler Trait | 查找方式 |
|--------|----------|---------------|----------|
| Effect | EffectHandlerRegistry | EffectHandler | type_name → O(1) HashMap |
| Modifier | ModifierRuleRegistry | ModifierCalculator | type_name → O(1) HashMap |
| Condition | ConditionEvaluatorRegistry | ConditionEvaluator | type_name → O(1) HashMap |
| Cost | CostRegistry | CostValidator | type_name → O(1) HashMap |
| Requirement | RequirementRegistry | RequirementChecker | type_name → O(1) HashMap |
| Targeting | TargetingRegistry | TargetResolver | type_name → O(1) HashMap |
| Formula | FormulaRegistry | Formula | FormulaId → O(1) HashMap |
| Execution | ExecutionRegistry | Execution | type_name → O(1) HashMap |
| Trigger | TriggerRegistry | TriggerHandler | Trigger → Vec<Handler> |
| Tag | TagRegistry | — | GameplayTag → TagDefinition |

**统一扩展规则**：新增任意子系统类型只需实现对应 Trait 并注册，不修改管线代码。

### 5.2 纯函数计算模式

所有"读路径"操作均为纯函数，不修改游戏状态：

| 操作 | 纯函数保证 | 上下文数据 |
|------|-----------|-----------|
| Requirement 检查 | check() → Result | RequirementContext |
| Cost 校验 | validate() → Result | CostContext |
| Condition 评估 | evaluate() → bool | ConditionContext |
| Targeting 解析 | resolve_targets() → Vec | TargetingContext |
| Formula 计算 | calculate() → FormulaOutput | FormulaInput |
| Execution 计算 | calculate() → ExecutionResult | ExecutionContext |
| Effect Generate | generate() → Option<PendingEffectData> | GenerateContext |
| Effect Modify | ModifierRule 遍历 → ModifierEntry | PendingEffectData |
| AbilityPreview | preview() → Option<EffectPreview> | PreviewContext |

### 5.3 修饰器栈计算公式

```
最终属性值 = (base + sum(所有 Add 修饰器值)) × product(所有 Multiply 修饰器值)
```

修饰器链执行顺序：**加算 → 乘算 → 覆盖**

修饰器来源精确清理：`remove_modifiers_from(source)` 按来源批量移除。

### 5.4 GameplayTag 位掩码查询

```rust
// O(1) 查询
gameplay_tags.has(tag)        // 单标签查询
gameplay_tags.has_any(tags)   // 任一匹配
gameplay_tags.has_all(tags)   // 全部匹配

// 三层合并
gameplay_tags = from_traits | from_equipment | from_buffs
```

### 5.5 CueEvent 纯数据信号

```
业务逻辑执行 → 产生 EffectResult → 构造 CueEvent（纯数据）
    → EventWriter::send() → Bevy Event 分发 → Presentation 层消费
```

**关键约束**：CueEvent 只携带 Entity ID + 数值 + 类型，不携带 Handle/AssetId/Path。

---

## 6. 领域间依赖关系

### 6.1 依赖矩阵

| 领域 | 依赖 | 被依赖 |
|------|------|--------|
| **Ability** | Requirement, Cost, Targeting, Effect, Trigger | Battle, Turn |
| **Targeting** | Tag, Map | Ability, Battle |
| **Effect** | AttributeModifier, Formula, Execution, Trigger, Cue | Ability, Buff, Trigger |
| **Execution** | Formula | Effect |
| **Formula** | — | Execution, Effect |
| **AttributeModifier** | Tag | Effect, Buff |
| **Tag** | — | 所有领域 |
| **Cue** | — | Effect, Trigger, Buff, Battle |
| **Requirement** | Tag, Attribute, Cooldown | Ability |
| **Cost** | Attribute | Ability |
| **Condition** | Tag, Attribute | Effect, Trigger |
| **Duration** | Trigger | Buff |
| **StackPolicy** | Duration | Buff |
| **Trigger** | Effect, Condition, Duration | Buff, Ability |

### 6.2 通信方式

| 通信方式 | 适用场景 | GAS 领域应用 |
|----------|----------|-------------|
| **函数调用** | 纯函数计算 | Requirement/Cost/Condition/Targeting/Formula/Execution 检查 |
| **Message** | 跨域广播 | CueEvent（DamageCue/HealCue/DeathCue）、StackChanged、BuffExpired |
| **Trigger** | Feature 内事件链 | Buff 效果的连锁响应（伤害→反伤→死亡→爆炸） |
| **Observer** | 局部状态变化 | Duration 变化触发的 UI 刷新 |

---

## 7. 宪法合规摘要

### 7.1 跨领域核心不变量

| 不变量 | 宪法条款 | 适用领域 |
|--------|----------|----------|
| Effect 必须走 Generate→Modify→Execute 三步管线 | 11.2.1 | Effect, AttributeModifier |
| 属性修改必须通过修饰器栈 | 8.0.3 | AttributeModifier |
| Effect 只负责声明意图，Execution 负责计算 | 8.0.5 | Effect, Execution, Formula |
| 新增内容只改 RON 不改代码 | 1.1.3 | 所有领域 |
| 定义与实例分离 | 1.1.2 | 所有领域 |
| 领域事件是唯一业务事实源 | 2.2.6 | 所有领域 |
| 读路径无副作用 | 11.7.1 | Requirement, Cost, Condition, Targeting |
| Buff 四阶段标准化 | 11.3.1 | Duration, StackPolicy, Trigger |
| 核心规则支持离线仿真 | 11.8.1 | Formula, Execution, Trigger |
| 战斗完全可重现 | 18.4.1 | 所有领域 |

### 7.2 绝对禁止事项汇总

| 禁止 | 理由 |
|------|------|
| 跳过 Effect Pipeline 直接修改游戏状态 | 破坏修饰规则和可观测性 |
| 在 Generate/Modify 阶段产生副作用 | 纯函数原则 |
| 硬编码 match 分发效果/条件/消耗类型 | 必须通过 Registry |
| Effect 感知触发来源（Skill/Buff） | 职责单一 |
| 为不同 Skill 中的同一 Effect 类型实现不同 Handler | 统一实现 |
| 直接修改属性绕过修饰器栈 | 破坏属性计算统一性 |
| 将暴击/克制/地形加成实现为独立 Effect | 暴击是 Modifier 不是 Effect |
| CueEvent 携带表现资源引用 | Cue 是纯数据信号 |
| 触发链深度超过 3 级 | 防止无限递归 |
| 为新增 Ability 修改 Rust 代码 | Rule/Content 分离 |

---

## 8. 交叉引用索引

| 主题 | 详细文档 |
|------|----------|
| Ability 系统 | `docs/02-domain/GAS/ability/ability-rules.md` |
| AttributeModifier 管线 | `docs/02-domain/GAS/attribute-modifier/attribute-modifier-rules.md` |
| Condition 系统 | `docs/02-domain/GAS/condition/condition-rules.md` |
| Cost 系统 | `docs/02-domain/GAS/cost/cost-rules.md` |
| Cue 表现事件 | `docs/02-domain/GAS/cue/cue-rules.md` |
| Duration 持续策略 | `docs/02-domain/GAS/duration/duration-rules.md` |
| Effect 效果管线 | `docs/02-domain/GAS/effect/effect-rules.md` |
| Execution 执行算式 | `docs/02-domain/GAS/execution/execution-rules.md` |
| Formula 公式系统 | `docs/02-domain/GAS/formula/formula-rules.md` |
| Requirement 释放前提 | `docs/02-domain/GAS/requirement/requirement-rules.md` |
| StackPolicy 叠层策略 | `docs/02-domain/GAS/stack-policy/stack-policy-rules.md` |
| Tag 标签系统 | `docs/02-domain/GAS/tag/tag-rules.md` |
| Targeting 目标选择 | `docs/02-domain/GAS/targeting/targeting-rules.md` |
| Trigger 触发器系统 | `docs/02-domain/GAS/trigger/trigger-rules.md` |
| GAS 架构设计 | `docs/01-architecture/01-battle-gas/gas-architecture.md` |
| 技能/Buff/Effect 统一抽象 | `docs/01-architecture/01-battle-gas/skill-buff-abstraction.md` |
| 七层架构总纲 | `docs/01-architecture/README.md` |
