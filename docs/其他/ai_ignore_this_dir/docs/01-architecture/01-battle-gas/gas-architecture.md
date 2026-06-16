---
id: 01-architecture.gas-architecture
title: GAS Architecture
status: proposed
owner: architect
created: 2026-06-16
updated: 2026-06-16
tags:
  - architecture
  - GAS
  - ADR
---

# ADR: SRPG-GAS 系统架构设计

## 状态

Proposed

## 背景

项目需要一个完整的 Gameplay Ability System (GAS) 来管理战斗中的技能释放、效果执行、属性计算、Buff 管理等核心战斗逻辑。SRPG-GAS 是对 UE GAS 的 SRPG 裁切版架构，需适配 Bevy 0.18.1 的 ECS 架构和项目七层架构体系。

### 当前状态

- 14 个 GAS 子领域已在 `docs/02-domain/GAS/` 完成领域建模
- 效果管线（Generate→Modify→Execute）已实现核心框架
- 叠层策略（StackingRule 4-enum 模型）已实现并测试通过
- 属性修饰器栈已实现基础功能
- Tag 系统已完成 37 个标签的位掩码实现

### 核心挑战

1. 500+ 技能需收敛为 20-30 个可复用的 Effect Executor
2. 所有数值修改必须通过修饰器栈，禁止直接修改
3. 效果管线三步严格顺序不可跳步
4. 领域层纯函数化，不依赖 Bevy ECS 类型
5. Rule/Content 分离：新增内容只改 RON 不改代码

## 引用的领域规则

- `docs/02-domain/GAS/ability/ability-rules.md` — Ability 释放五阶段管线
- `docs/02-domain/GAS/effect/effect-rules.md` — Effect Pipeline 三步管线
- `docs/02-domain/GAS/attribute-modifier/attribute-modifier-rules.md` — 属性修饰管线
- `docs/02-domain/GAS/trigger/trigger-rules.md` — 触发器和 ExecutionStack
- `docs/02-domain/GAS/tag/tag-rules.md` — GameplayTag 位掩码系统
- `docs/02-domain/GAS/cue/cue-rules.md` — Logic/Presentation 分离
- `docs/02-domain/GAS/execution/execution-rules.md` — 执行算式 Trait 分发
- `docs/02-domain/GAS/formula/formula-rules.md` — 公式纯函数计算
- `docs/02-domain/GAS/condition/condition-rules.md` — 条件评估纯函数
- `docs/02-domain/GAS/cost/cost-rules.md` — 消耗校验与扣除分离
- `docs/02-domain/GAS/requirement/requirement-rules.md` — 释放前提检查
- `docs/02-domain/GAS/targeting/targeting-rules.md` — 目标选择纯函数
- `docs/02-domain/GAS/duration/duration-rules.md` — 持续策略
- `docs/02-domain/GAS/stack-policy/stack-policy-rules.md` — 叠层策略
- `docs/01-architecture/README.md` — 七层架构总纲

## 决策

SRPG-GAS 采用 **Trait Registry + 纯函数管线** 架构模式，核心设计决策如下：

1. **所有子系统通过 Trait + Registry 模式扩展**：新增类型只实现 Trait 并注册，不修改管线代码
2. **效果管线三步严格顺序**：Generate（纯函数）→ Modify（纯函数）→ Execute（有副作用）
3. **领域层纯函数化**：所有计算操作的参数为 Context 数据结构体，不依赖 ECS Query/Entity
4. **Modifier 不是 Effect**：暴击、克制、地形加成是 Modifier 而非 Effect
5. **CueEvent 纯数据信号**：业务逻辑只发射 CueEvent，不关心表现层消费

---

## Module Design

### 模块总览

```
src/
├── core/
│   ├── tag.rs                    # GameplayTag 位掩码（u64, 37/64 bits）
│   ├── tag_def.rs                # TagRegistry + TagDefinition RON 加载
│   └── attribute.rs              # Attributes 组件 + 修饰器栈
├── battle/
│   ├── ability/
│   │   ├── ability_data.rs       # AbilityData / AbilityDef
│   │   ├── ability_slots.rs      # AbilitySlots / AbilityCooldowns
│   │   ├── ability_cast.rs       # AbilityCastResult / can_use()
│   │   └── ability_preview.rs    # AbilityPreview / preview_ability_effects()
│   ├── effect/
│   │   ├── effect_def.rs         # EffectDef 枚举（Damage/Heal/ApplyBuff/Cleanse）
│   │   ├── effect_handler.rs     # EffectHandler trait + EffectHandlerRegistry
│   │   ├── pipeline/
│   │   │   ├── generate.rs       # Generate 阶段调度
│   │   │   ├── modify.rs         # Modify 阶段调度（委托 AttributeModifier）
│   │   │   └── execute.rs        # Execute 阶段调度
│   │   ├── pending_effect.rs     # PendingEffect / PendingEffectData
│   │   └── effect_result.rs      # EffectResult / PendingMessage
│   ├── attribute_modifier/
│   │   ├── modifier.rs           # AttributeModifierInstance / ModifierSource
│   │   ├── modifier_rule.rs      # ModifierRule + ModifierCalculator trait
│   │   ├── modifier_entry.rs     # ModifierEntry（Modify 阶段记录）
│   │   └── modifier_registry.rs  # ModifierRuleRegistry
│   ├── execution/
│   │   ├── execution.rs          # Execution trait + ExecutionRegistry
│   │   ├── damage_execution.rs   # DamageExecution（四段伤害公式）
│   │   ├── heal_execution.rs     # HealExecution
│   │   └── shield_execution.rs   # ShieldExecution
│   ├── formula/
│   │   ├── formula.rs            # Formula trait + FormulaRegistry
│   │   └── formula_id.rs         # FormulaId 枚举（10 种）
│   ├── targeting/
│   │   ├── targeting_def.rs      # TargetingDef 枚举
│   │   ├── target_resolver.rs    # TargetResolver trait + TargetingRegistry
│   │   └── targeting_context.rs  # TargetingContext
│   ├── requirement/
│   │   ├── requirement_def.rs    # RequirementDef 枚举（9 种）
│   │   ├── requirement_checker.rs # RequirementChecker trait + RequirementRegistry
│   │   └── requirement_context.rs # RequirementContext
│   ├── cost/
│   │   ├── cost_def.rs           # CostDef 枚举（8 种）
│   │   ├── cost_validator.rs     # CostValidator trait + CostRegistry
│   │   └── cost_context.rs       # CostContext
│   ├── condition/
│   │   ├── condition_def.rs      # ConditionDef 枚举（10 种）
│   │   ├── condition_evaluator.rs # ConditionEvaluator trait + ConditionEvaluatorRegistry
│   │   └── condition_context.rs  # ConditionContext
│   ├── trigger/
│   │   ├── trigger.rs            # Trigger 枚举（16 种）
│   │   ├── trigger_handler.rs    # TriggerHandler trait + TriggerRegistry
│   │   ├── trigger_context.rs    # TriggerContext
│   │   └── execution_stack.rs    # ExecutionStack（LIFO, MAX=32）
│   ├── duration/
│   │   ├── duration_policy.rs    # DurationPolicy 枚举（7 种）
│   │   └── duration_marker.rs    # DurationMarker 组件
│   ├── stacking/
│   │   ├── stacking_rule.rs      # StackingRule 枚举（4 种）✅ 已实现
│   │   └── stack_count.rs        # StackCount 组件
│   └── cue/
│       ├── cue_event.rs          # CueEvent 独立 Struct（8 种）
│       ├── cue_handler.rs        # CueHandler 接口规范
│       └── cue_registry.rs       # CueRegistry
```

### 模块职责划分

| 模块 | 职责 | 依赖 |
|------|------|------|
| `core::tag` | GameplayTag 位掩码、三层标签管理 | 无 |
| `battle::ability` | Ability 槽位、冷却、验证、预览 | requirement, cost, targeting, effect |
| `battle::effect` | Effect Pipeline 三步编排、EffectHandler 分发 | attribute_modifier, formula, execution |
| `battle::attribute_modifier` | 属性计算、修饰器栈、ModifierRule 匹配 | tag |
| `battle::execution` | 公式执行层、Execution trait 分发 | formula |
| `battle::formula` | 纯函数公式、FormulaId 注册 | 无 |
| `battle::targeting` | 目标解析、TargetResolver 分发 | tag |
| `battle::requirement` | 释放前提检查、RequirementChecker 分发 | tag |
| `battle::cost` | 消耗校验和扣除、CostValidator 分发 | — |
| `battle::condition` | 条件评估、ConditionEvaluator 分发 | tag |
| `battle::trigger` | 触发时机、触发链、ExecutionStack | effect, condition, duration |
| `battle::duration` | 持续策略、回合递减、条件终止 | trigger |
| `battle::stacking` | 叠层策略、层数管理 | duration |
| `battle::cue` | CueEvent 定义、CueRegistry | 无 |

---

## Communication Design

### 通信方式选择

| 场景 | 通信方式 | 选择理由 |
|------|----------|----------|
| Requirement/Cost/Condition/Targeting 检查 | 函数调用 | 纯函数直接调用即可，无需事件化 |
| Formula/Execution 计算 | 函数调用 | 纯函数，O(1) Registry 查找 |
| Effect Pipeline 内部调度 | 函数调用 | 同一 Feature 内的编排逻辑 |
| ModifierRule 标签匹配 | 函数调用 | 纯函数遍历 |
| 效果执行结果通知 | Message | 跨域广播（BattleRecord/UI） |
| CueEvent 表现信号 | Message | Logic/Presentation 分离，fire-and-forget |
| StackChanged/MaxStackReached | Message | 跨域通知 UI |
| BuffExpired/DurationTerminated | Message | 跨域通知 Buff 领域 |
| Buff 效果连锁响应 | Trigger（ExecutionStack） | Feature 内事件链，嵌套触发 |
| Duration 条件终止（UntilXxx） | Trigger | Feature 内事件驱动 |
| Duration 变化触发 UI | Observer | 局部状态变化响应 |

### 消息白名单

| 消息类型 | 发送方 | 接收方 | 携带数据 |
|----------|--------|--------|----------|
| DamageApplied | Effect Execute | BattleRecord, UI | source, target, amount, modifiers |
| HealApplied | Effect Execute | BattleRecord, UI | source, target, amount |
| BuffApplied | Effect Execute | Buff 领域 | target, buff_id, duration |
| BuffRemoved | Duration/StackPolicy | UI | target, buff_id |
| EntityDied | Effect Execute | Battle, Turn, UI | unit_id, killer_id |
| EffectCompleted | Effect Execute | Trigger | effect_result |
| StackChanged | StackPolicy | UI | buff_id, old_count, new_count |
| MaxStackReached | StackPolicy | UI | buff_id, max_count |
| TriggerEffectReady | TriggerRegistry | Effect Pipeline | effect_defs, trigger_context |
| TriggerChainExceeded | ExecutionStack | Debug | chain_depth |

### 四级通信机制在 GAS 中的应用

| 通信层级 | GAS 领域应用 |
|----------|-------------|
| Hook（2.2.1） | DurationMarker/StackCount 组件添加移除的轻量副作用 |
| Trigger（2.2.2） | **核心**：Buff 效果连锁响应（ExecutionStack） |
| Observer（2.2.3） | Duration 变化触发 UI 层数刷新 |
| Message（2.2.4） | CueEvent、DamageApplied、StackChanged 等跨域广播 |

---

## 边界定义

### 允许的依赖

```
Ability ──→ Requirement, Cost, Targeting, Effect, Cue
Effect ──→ AttributeModifier, Formula, Execution, Trigger, Cue
Execution ──→ Formula
Trigger ──→ Effect, Condition, Duration
AttributeModifier ──→ Tag
Targeting ──→ Tag
Requirement ──→ Tag
Condition ──→ Tag
Duration ──→ Trigger
StackPolicy ──→ Duration
Cue ──→ (无依赖，纯事件定义)
Formula ──→ (无依赖，纯函数)
Tag ──→ (无依赖，位掩码)
```

### 禁止的依赖

| 禁止路径 | 理由 |
|----------|------|
| Effect → Ability | Effect 不感知触发来源 |
| Execution → Effect | Execution 只管计算，不关心效果类型 |
| Formula → ECS World | Formula 是纯函数 |
| Cue → 业务层 | Cue 是单向信号，业务层不消费 CueEvent |
| Tag → 业务逻辑 | Tag 是纯分类机制 |
| 任意领域 → UI | Logic/Presentation 分离 |
| 任意计算 → ECS Query | 领域层纯函数化 |

---

## Forbidden（禁止事项）

### 架构级禁止

- 🟥 **禁止跳过 Effect Pipeline 直接修改游戏状态** — 理由：管线保证修饰规则和可观测性
- 🟥 **禁止在 Generate/Modify 阶段产生副作用** — 理由：纯函数原则
- 🟥 **禁止硬编码 match 分发效果/条件/消耗/目标类型** — 理由：必须通过 Registry 扩展
- 🟥 **禁止 Effect 感知触发来源（Skill/Buff）** — 理由：职责单一，只做效果执行
- 🟥 **禁止为不同 Skill 中的同一 Effect 类型实现不同 Handler** — 理由：统一实现
- 🟥 **禁止新增 Effect 类型时修改 Pipeline 调度代码** — 理由：扩展性
- 🟥 **禁止直接修改属性绕过修饰器栈** — 理由：破坏属性计算统一性
- 🟥 **禁止将暴击/克制/地形加成实现为独立 Effect** — 理由：暴击是 Modifier 不是 Effect
- 🟥 **禁止 Modifier 直接修改 World 状态** — 理由：Modifier 只修改 PendingEffectData.amount
- 🟥 **禁止 CueEvent 携带表现资源引用（Handle/AssetId/Path）** — 理由：Cue 是纯数据信号
- 🟥 **禁止业务逻辑消费 CueEvent** — 理由：Core 层反向依赖 Presentation 层
- 🟥 **禁止触发链深度超过 MAX_STACK_DEPTH（32）** — 理由：防止无限递归
- 🟥 **禁止为新增 Ability 修改 Rust 代码** — 理由：Rule/Content 分离
- 🟥 **禁止在非 TurnEnd 阶段递减 Duration 和 Cooldown** — 理由：与回合生命周期同步
- 🟥 **禁止 Buff 无 DurationPolicy** — 理由：禁止永久存在的 Buff
- 🟥 **禁止先扣除后校验消耗** — 理由：校验是安全检查

### 扩展性禁止

- 🟥 **禁止为未来需求过度设计** — 理由：当前 14 个子领域已覆盖所有已知场景
- 🟥 **禁止在计算层引入随机逻辑** — 理由：随机性由 Random 系统注入
- 🟥 **禁止 Formula/Execution 访问 ECS World** — 理由：纯函数，独立测试
- 🟥 **禁止运行时动态注册 Registry** — 理由：Registry 在启动时固定

---

## Definition / Instance Design

### Definition（不可变配置）

| Definition | 存储位置 | 加载方式 |
|------------|----------|----------|
| AbilityDef | `assets/skills/*.ron` | RON 反序列化 → From<AbilityData> |
| EffectDef | 嵌入 AbilityDef/BuffDef | RON 反序列化 |
| ModifierRule | `assets/rules/*.ron` | RON 反序列化 |
| DurationDef | 嵌入 BuffDef | RON 反序列化 |
| StackingRuleDef | 嵌入 BuffDef | RON 反序列化 |
| TargetingDef | 嵌入 AbilityDef | RON 反序列化 |
| RequirementDef | 嵌入 AbilityDef | RON 反序列化 |
| CostDef | 嵌入 AbilityDef | RON 反序列化 |
| ConditionDef | 嵌入 EffectDef | RON 反序列化 |
| TriggerDef | 嵌入 BuffDef | RON 反序列化 |
| TagDefinition | `content/definitions/tags.ron` | RON 加载到 TagRegistry |

### Instance（运行时状态）

| Instance | 组件类型 | 存储位置 |
|----------|----------|----------|
| AbilitySlots | Component | 单位 Entity |
| AbilityCooldowns | Component | 单位 Entity |
| Attributes | Component | 单位 Entity |
| GameplayTags | Component | 单位 Entity |
| PersistentTags | Component | 单位 Entity |
| ActiveBuffs | Component | 单位 Entity |
| DurationMarker | Component | Buff 实例 |
| StackCount | Component | Buff 实例 |
| PendingEffect | 纯数据结构 | Effect Pipeline 中流转 |
| TriggerContext | 纯数据结构 | Trigger 匹配时构建 |
| ExecutionContext | 纯数据结构 | Execution 计算时构建 |

### Registry（全局 Resource）

| Registry | 存储内容 | 初始化时机 |
|----------|----------|-----------|
| EffectHandlerRegistry | HashMap<String, Box<dyn EffectHandler>> | App 启动 |
| ModifierRuleRegistry | Vec<ModifierRule> | App 启动（RON 加载） |
| ModifierCalculatorRegistry | HashMap<String, Box<dyn ModifierCalculator>> | App 启动 |
| ConditionEvaluatorRegistry | HashMap<String, Box<dyn ConditionEvaluator>> | App 启动 |
| CostRegistry | HashMap<String, Box<dyn CostValidator>> | App 启动 |
| RequirementRegistry | HashMap<String, Box<dyn RequirementChecker>> | App 启动 |
| TargetingRegistry | HashMap<String, Box<dyn TargetResolver>> | App 启动 |
| FormulaRegistry | HashMap<FormulaId, Box<dyn Formula>> | App 启动 |
| ExecutionRegistry | HashMap<String, Box<dyn Execution>> | App 启动 |
| TriggerRegistry | HashMap<Trigger, Vec<Box<dyn TriggerHandler>>> | App 启动 |
| TagRegistry | HashMap<GameplayTag, TagDefinition> | App 启动（RON 加载） |
| CueRegistry | Vec<TypeIdentifier> | App 启动 |

---

## 数据存储方案

### RON 文件组织

```
content/
├── skills/                    # AbilityDef 配置
│   ├── normal_attack.ron
│   ├── fireball.ron
│   └── heal.ron
├── buffs/                     # BuffDef 配置（含 Duration/Stacking/Trigger）
│   ├── burn.ron
│   ├── poison.ron
│   └── shield.ron
├── rules/                     # ModifierRule 配置
│   ├── element_modifier.ron
│   └── terrain_modifier.ron
├── definitions/
│   └── tags.ron               # TagDefinition 元数据
└── formulas/                  # 公式参数配置（可选）
    └── damage_boundaries.ron
```

### 数据分层

| 层级 | 数据 | 生命周期 |
|------|------|----------|
| Definition | AbilityDef, EffectDef, ModifierRule, TagDefinition | 不可变，加载后不变 |
| Instance | AbilitySlots, Attributes, GameplayTags, ActiveBuffs | 运行时可变 |
| Runtime | PendingEffect, TriggerContext, ExecutionContext | 单次计算，不持久化 |
| Persistence | SaveData（战斗状态快照） | 战斗结束持久化 |

---

## 扩展性考虑

### 新增 Ability

1. 在 `assets/skills/*.ron` 中添加新的 AbilityDef RON 文件
2. 确保 id 唯一且 effects 中的 EffectDef 类型已注册
3. **无需修改 Rust 代码**

### 新增 Effect 类型

1. 实现 EffectHandler trait（type_name / generate / preview / execute）
2. 在 EffectHandlerRegistry 中注册
3. 添加对应的 EffectDef 变体
4. **无需修改 Pipeline 调度代码**

### 新增条件/消耗/目标类型

1. 实现对应的 Trait（ConditionEvaluator / CostValidator / TargetResolver）
2. 注册到对应的 Registry
3. **无需修改管线代码**

### 新增修饰规则

1. 在 `assets/rules/*.ron` 中添加新的 ModifierRule 配置
2. 确保 source_tag 和 target_tag 在 GameplayTag 中有定义
3. **无需修改 Rust 代码**

### 新增 Trigger 时机点

1. 在 Trigger 枚举中添加新变体
2. 添加对应的 TriggerHandler 实现
3. 注册到 TriggerRegistry

---

## 安全架构

### 确定性保证

| 机制 | 保证 |
|------|------|
| Formula 纯函数 | 相同输入 → 相同输出 |
| Execution 纯函数 | 相同输入 → 相同输出 |
| 随机性外置 | Random 系统注入，Formula 不含随机 |
| Effect Pipeline 三步顺序 | Generate→Modify→Execute 不可跳步 |
| ModifierEntry 记录 | 每步修饰可审计 |
| Trigger 链深度限制 | MAX_STACK_DEPTH = 32，防止无限递归 |

### 回放兼容

- Replay 记录触发 Cue 的上游事件（Ability 使用、Effect 执行）
- CueEvent 本身不参与回放（派生产物）
- EffectResult 的 EffectCompleted 事件确保回放可重现

### 数值边界

| 约束 | 值 |
|------|-----|
| 伤害下限 | ≥ 1 |
| 治疗下限 | ≥ 0 |
| 暴击率上限 | ≤ 95% |
| 闪避率上限 | ≤ 80% |
| 减伤上限 | ≤ 90% |
| 触发链深度 | ≤ 3（逻辑层）/ ≤ 32（Stack 层） |
| GameplayTag 位掩码 | 37/64 bits 已使用 |

---

## 后果

### 正面

- **高度可扩展**：新增 Ability/Effect/Condition/Cost 类型只需实现 Trait + 注册
- **数据驱动**：新内容 = 新 RON 文件，不改 Rust 代码
- **可测试**：所有计算为纯函数，可脱离 Bevy 运行
- **可审计**：ModifierEntry 记录每步修饰，ExecutionResult.breakdown 记录计算过程
- **确定性**：Formula/Execution 纯函数 + 随机性外置，支持回放
- **关注点分离**：Effect 不感知来源，Cue 不关心表现

### 负面

- **抽象层级多**：14 个子领域 + 多个 Registry，学习曲线陡峭
- **Trait 对象开销**：Box<dyn Trait> 的动态分发有一定性能开销
- **配置文件多**：每个 Ability/Buff/Rule 需要独立 RON 文件
- **调试复杂**：Effect Pipeline 三步 + Trigger 链 + ExecutionStack 增加调试难度

## 替代方案

### 方案 A：硬编码 Effect 函数（已弃用）

每个 Ability 写独立的 execute_xxx() 函数。100 个 Ability = 100 个函数 = 100 个 Bug 来源。无法复用，违反 Rule/Content 分离。

### 方案 B：单一 EffectHandler + match 分发（已弃用）

所有 Effect 类型在一个 Handler 中用 match 分发。新增类型需修改核心代码，违反开闭原则。

### 方案 C：UE GAS 完整移植（已弃用）

完整移植 UE GAS 的 GameplayEffect/AbilitySystemComponent 等概念。SRPG 场景不需要如此复杂的系统，且与 Bevy ECS 模型不兼容。

**当前方案（Trait Registry + 纯函数管线）** 在扩展性、可测试性和架构简洁性之间取得最佳平衡。
