---
id: 09-planning.adr-026-gap-analysis
title: ADR-026 源码差距分析与行动计划
status: Draft
owner: feature-developer
created: 2026-06-15
updated: 2026-06-15
tags:
  - planning
  - adr-026
  - gap-analysis
  - feature-developer
---

# ADR-026 SRPG Lite-GAS 架构对齐：源码差距分析与行动计划

> **版本说明**：本文档基于 2026-06-15 对 `src/`、`content/` 和 `docs/` 的实际文件扫描，对比 ADR-026 声明的 10+3 模块架构与当前源码实现的差距。

---

## 目录

1. [执行摘要](#1-执行摘要)
2. [ADR-026 13 模块 vs 源码状态总览](#2-adr-026-13-模块-vs-源码状态总览)
3. [重大架构差距详解](#3-重大架构差距详解)
4. [关键架构变更差距](#4-关键架构变更差距)
5. [内容管线差距](#5-内容管线差距)
6. [Plugin 注册顺序差距](#6-plugin-注册顺序差距)
7. [分阶段执行计划](#7-分阶段执行计划)
8. [依赖图与关键路径](#8-依赖图与关键路径)
9. [风险评估](#9-风险评估)
10. [架构自检](#10-架构自检)

---

## 1. 执行摘要

**核心结论**：ADR-026 声明的 10+3 架构**落地不足 30%**。7 个保留模块（Tag/Modifier/Effect/Ability/Trigger/Targeting）已存在，但 3 个新增业务模块（Execution/Cue/Stacking）和 3 个基建模块（Registry/Pipeline/Replay）**几乎为零代码**。

### 各模块实现程度

| 类别 | 已实现 | 部分实现 | 未实现 |
|------|--------|---------|--------|
| 10 业务领域 | Tag, Modifier, Effect, Ability, Trigger, Targeting | Attribute, Stacking | **Execution**, **Cue** |
| 3 基础设施 | — | Replay | **Registry**, **Pipeline** |

### 总工作量估算

```
P0 — 核心新模块创建（Execution + Cue + Stacking）：~8 天
P1 — 架构重构（Buff → Effect 吸收、Plugin DAG）：~5 天
P2 — 基建补齐（Registry + Pipeline + Replay）：~5 天
P3 — 内容管线对齐（content/ 新目录 + RON）：~2 天
P4 — 质量加固（测试 + 验证 + 文档同步）：~5 天
总计：~20-25 人天
```

---

## 2. ADR-026 13 模块 vs 源码状态总览

### 2.1 业务领域（10 模块）

| # | 模块 | ADR-026 目标 | 源码状态 | 差距等级 | 关键缺失 |
|---|------|-------------|----------|---------|---------|
| 1 | Attribute | 一级领域：types.rs + calculator.rs + components.rs + def.rs + 独立 Plugin | ⚠️ 部分实现 | 🟡 P1 | Attributes Component 内嵌在 mod.rs（无单独 `components.rs`）；DerivedCalculator 是 Attributes 方法而非独立模块；无 `calculator.rs` |
| 2 | Tag | 保留：types.rs + components.rs + registry.rs | ✅ 完整实现 | 🟢 无 | — |
| 3 | Modifier | 保留：types.rs + calculator.rs + registry.rs | ✅ 完整实现 | 🟢 无 | — |
| 4 | Effect | 统一效果层（吸收 Buff）：types.rs + handler.rs + pipeline.rs + data.rs | ⚠️ 部分实现 | 🟡 P1 | 无 `pipeline.rs`（当前在 battle/ 模块）；EffectDef::ApplyBuff 仍依赖独立 Buff 模块；无 Duration 系统 |
| 5 | Ability | 保留：types.rs + pipeline.rs + registry.rs | ✅ 完整实现 | 🟢 无 | — |
| 6 | Trigger | 保留：types.rs + stack.rs + registry.rs | ✅ 完整实现 | 🟢 无 | — |
| 7 | Targeting | 保留：types.rs + resolver.rs | ✅ 完整实现 | 🟢 无 | — |
| **8** | **Execution** | **新增：types.rs + damage.rs + heal.rs + shield.rs + registry.rs** | **❌ 不存在** | **🔴 P0** | 无目录、无文件、无 Plugin、无 Execution trait |
| **9** | **Stacking** | **升级为独立模块：types.rs（4-enum）+ resolver.rs（纯函数）** | **❌ 未独立** | **🔴 P0** | StackPolicy 嵌套在 `buff/domain/types.rs` 且为 3-enum（NoStack/Stackable/StackableNoRefresh），不是 ADR-026 的 4-enum（Replace/RefreshDuration/StackAdd/StackMax） |
| **10** | **Cue** | **新增：types.rs + emitter.rs** | **❌ 不存在** | **🔴 P0** | 无目录、无文件、无 Plugin、无 CueEvent |

### 2.2 基础设施（3 模块）

| # | 模块 | ADR-026 目标 | 源码状态 | 差距等级 | 关键缺失 |
|---|------|-------------|----------|---------|---------|
| **11** | **Registry** | **统一注册中心：ability_registry.rs + effect_registry.rs + execution_registry.rs + tag_registry.rs** | **❌ 不存在** | **🔴 P0** | 注册散落在各模块内 |
| **12** | **Pipeline** | **回合战斗执行管线：scheduler.rs** | **❌ 不存在** | **🔴 P0** | Effect Pipeline 在 battle/ 模块中，不是独立基础设施 |
| 13 | Replay | 确定性回放：record.rs + player.rs | ⚠️ 空壳 | 🟡 P2 | `src/infrastructure/replay/mod.rs` 仅 2 行注释，无实现 |

---

## 3. 重大架构差距详解

### 3.1 🔴 P0 — Execution 模块完全缺失

**ADR-026 要求**：独立的 `src/core/execution/` 模块，包含：
- `Execution` trait（`fn execute(&self, context: &ExecutionContext) -> ExecutionResult`）
- 内置实现：DamageExecution, TrueDamageExecution, CritExecution, HealExecution, ShieldExecution
- `ExecutionRegistry` Resource（按类型名分发）
- 所有公式计算从 EffectHandler 中剥离

**当前源码状态**：
- 伤害公式在 `core/effect/types.rs` 的 `calculate_damage_from_effect()` 纯函数中
- 公式调用在 `core/effect/handler.rs` 的 `DamageHandler::generate()` 中
- 无 Execution trait，无 ExecutionRegistry

**违反的 ADR-026 Forbidden**：
- 🟥 禁止 Effect 内部写公式 — Execution trait 统一管理所有数值计算
- 🟥 禁止大型 match 分发伤害类型
- 🟥 禁止跳过 Execution 直接在 Effect 中计算伤害

**修复路径**：新建 `src/core/execution/`，从 effect/handler.rs 提取公式逻辑。

### 3.2 🔴 P0 — Cue 模块完全缺失

**ADR-026 要求**：独立的 `src/core/cue/` 模块，包含：
- `CueEvent` 枚举（CueDamage, CueDeath, CueHeal, CueBuffApply, CueShield）
- `CueEmitter` Resource（下发表现事件）
- GAS 链末端：`... → Tag → Cue → Replay`

**当前源码状态**：
- 无 Cue 概念
- 业务层直接使用 `DamageApplied` / `HealApplied` / `CharacterDied` Message
- UI 层直接订阅业务事件，无信号总线中介

**违反的 ADR-026 Forbidden**：
- 🟥 禁止业务代码直接调用 UI/特效 — 必须通过 Cue 事件总线
- 🟥 禁止在 Cue 中包含业务逻辑 — Cue 只下发纯数据事件

**修复路径**：新建 `src/core/cue/`，将现有的业务 Message（DamageApplied/HealApplied/CharacterDied）包装为 CueEvent，或新增 CueMsg 包装层。

### 3.3 🔴 P0 — Stacking 未独立，未升级 4-enum

**ADR-026 要求**：
- 独立 `src/core/stacking/` 模块
- `StackingRule` 4-enum 冻结版：`Replace | RefreshDuration | StackAdd | StackMax(u32)`
- `resolve_stacking()` 纯函数
- Ability 和 Effect 通过 stacking 模块处理效果叠加

**当前源码状态**：
- `StackPolicy` 在 `buff/domain/types.rs` 中，是 3-enum：`NoStack | Stackable(u32) | StackableNoRefresh(u32)`
- 堆叠逻辑在 `buff/apply.rs` 的 `apply_buff_with_stack()` 中
- 堆叠绑定在 Buff 系统上，不是通用 Effect 设施

**关键行为差异**：

| 场景 | ADR-026 | 当前 | 问题 |
|------|---------|------|------|
| 中毒叠层 | `StackMax(5)` + `StackAdd` | `Stackable(5)` | 语义接近但命名不同 |
| 护盾叠加 | `StackAdd` | 仅 `Stackable(u32)` 有上限 | 护盾应无上限 |
| 狂暴刷新 | `RefreshDuration` | `NoStack`（等价） | 语义不清晰 |
| Boss 光环替换 | `Replace` | `NoStack`（等价） | 语义不清晰 |
| 上限后行为 | `StackAdd` 新增式 | `StackableNoRefresh`（不刷新） | Stackable 刷新最旧层 vs StackAdd 新增 |

**修复路径**：
1. 新建 `src/core/stacking/` 模块
2. 定义 `StackingRule` 4-enum + `resolve_stacking()` 纯函数
3. 从 `buff/apply.rs` 提取堆叠逻辑
4. 废弃旧的 StackPolicy 枚举

### 3.4 🟡 P1 — Attribute 模块未完全对齐

**ADR-026 要求**：
- `types.rs` → `PrimaryAttr, DerivedAttr, AttributeSet` ✅（AttributeKind 已覆盖）
- `calculator.rs` → `DerivedCalculator`（Primary → Derived 计算）❌ 当前在 Attributes 方法内
- `components.rs` → `AttributeComponent`（Entity 上的属性数据）⚠️ 当前在 mod.rs 内
- `def.rs` → `AttributeDef` ✅ 已存在

**差距**：主要是代码组织问题，功能已实现。但 `Attributes` 组件在 mod.rs 中（250+ 行），应拆分出 `components.rs`。

### 3.5 🟡 P1 — Effect 模块未吸收 Buff

**ADR-026 要求**：Buff 统一为带 Duration 的 Effect。
- `EffectDef` 增加 Duration 字段
- `PendingEffectData` 不再包含 `ApplyBuff` 变体
- 删除独立 Buff 模块

**当前源码状态**：
- `EffectDef::ApplyBuff { buff_id, duration }` 仍依赖独立 Buff 模块
- Buff 模块 7 个文件（domain/, apply.rs, instance.rs, resolve.rs, trigger.rs, mod.rs, id.rs）
- BuffPlugin 在 app/plugin.rs 中独立注册

**注意**：此任务为**大重构**，建议在 Execution + Stacking 基础建成后进行。当前可先标记废弃。

---

## 4. 关键架构变更差距

### 4.1 GAS 执行链对比

**ADR-026 目标链（冻结时序）**：
```
Ability → Targeting → Effect → Stacking → Execution → Modifier → Attribute → Tag → Cue → Replay
```

**当前实际链**：
```
Ability → Targeting → Effect（Generate → Modify → Execute）
                ↓
          Modifier → Attribute（在 Effect::generate 内直接调用）
```

**缺失环节**：`Stacking`、`Execution`、`Cue`、`Replay`

### 4.2 函数调用链对比

**ADR-026 目标**：
```
Ability::execute()
  ├── cost::check_and_pay()
  ├── targeting::resolver::resolve()
  ├── effect::pipeline::execute()
  │     ├── stacking::resolve()          ← 缺失
  │     ├── execution::execute()         ← 缺失
  │     ├── modifier::calculator::apply()
  │     └── attribute::calculate()
  ├── tag::update()
  ├── cue::emit()                        ← 缺失
  ├── replay::record()                   ← 缺失
  └── cooldown::set()
```

**当前实际**：
```
Ability::execute()
  ├── cost::check_and_pay()
  ├── targeting::resolver::resolve()
  ├── effect::pipeline::execute()
  │     └── handler.generate() 内联 formula（含 calculate_damage_from_effect）
  │     └── handler.execute() 直接修改 Attributes
  └── cooldown::set()
```

### 4.3 Message 对比

**ADR-026 目标 Message**：

| Message | 发送者 | 订阅者 | 当前状态 |
|---------|-------|--------|---------|
| `CueDamage` | Execution | UI、Trigger、Replay | 不存在（当前用 `DamageApplied`） |
| `CueHeal` | Execution | UI、Replay | 不存在（当前用 `HealApplied`） |
| `CueDeath` | Attribute | UI、Trigger | 不存在（当前用 `CharacterDied`） |
| `CueBuffApply` | Effect | UI、Trigger | 不存在（当前用 `BuffApplied`） |
| `AbilityCastStarted` | Ability | UI、Replay | 不存在 |
| `AbilityCastFinished` | Ability | UI、Replay、Turn | 不存在 |

### 4.4 Buff → Effect 吸收对比

**ADR-026 要求**：

| 原 Buff 类型 | 新 Effect 表达 | Duration |
|-------------|----------------|----------|
| 瞬时伤害/治疗 | `Effect::ApplyModifier(...)` | `Duration::Instant` |
| 回合 Buff | `Effect::ApplyModifier(...)` | `Duration::TurnLimited(u32)` |
| 永久常驻 | `Effect::ApplyModifier(...)` | `Duration::Permanent` |

**当前源码状态**：BuffData 有 `duration: DurationPolicy`（7 种策略）和 `stack: StackPolicy`。这两个字段正对应 ADR-026 的 Duration 和 Stacking，但附加在 Buff 系统上而非 Effect 系统。

---

## 5. 内容管线差距

### 5.1 content/ 目录对比

**ADR-026 目标 content/ 结构（迁移后）**：

```
content/characters/*.ron    → UnitTemplateRegistry    ✅ 存在（6 文件）
content/skills/*.ron        → SkillRegistry           ✅ 存在（6 文件）
content/buffs/*.ron         → BuffRegistry ❌ 废弃    ⚠️ 存在（8 文件，但应废弃）
content/effects/*.ron       → EffectRegistry          ❌ 不存在
content/executions/*.ron    → ExecutionRegistry       ❌ 不存在
content/cues/*.ron          → CueRegistry             ❌ 不存在
content/attributes/*.ron    → AttributeRegistry       ⚠️ 仅 1 个 definitions/attributes.ron
content/equipments/*.ron    → EquipmentRegistry       ❌ 目录不存在
content/items/*.ron         → ItemRegistry            ❌ 目录不存在
content/terrains/*.ron      → TerrainRegistry         ✅ 存在（4 文件）
content/stages/*.ron        → LevelRegistry           ✅ 存在（1 文件）
content/ai_behaviors/*.ron  → AiBehaviorRegistry      ✅ 存在（3 文件）
content/formulas/*.ron      → ModifierRuleRegistry    ✅ 存在（1 文件 modifiers/）
content/classes/*.ron       → TraitRegistry           ✅ 存在（5 文件）
```

**当前实际 content/ 目录**：
```
ai_behaviors/
buffs/
campaigns/
characters/
classes/
definitions/
modifiers/
skills/
stages/
terrains/
```

**缺失目录**：`effects/`、`executions/`、`cues/`、`attributes/`（有 definitions/ 但非独立属性目录）、`equipments/`、`items/`

### 5.2 当前注册表路径（过渡期）

`docs/01-architecture/README.md` 记录了当前的过渡路径和目标路径。**过渡期路径使用 `content/` 前缀但缺少新模块的目录**。

---

## 6. Plugin 注册顺序差距

### 6.1 当前注册顺序（ADR-025）

```
TagPlugin + TagDefPlugin + AttributeDefPlugin → ModifierRulePlugin → EffectPlugin → BuffPlugin + TargetingPlugin + TriggerPlugin → AbilityPlugin → ...
```

### 6.2 ADR-026 目标 DAG

```
RegistryPlugin → AttributePlugin + TagPlugin → ModifierPlugin → EffectPlugin → AbilityPlugin + TriggerPlugin + TargetingPlugin + StackingPlugin + ExecutionPlugin → CuePlugin → BattlePipelinePlugin + BattleReplayPlugin
```

### 6.3 需要的变更

| 变更 | 当前 | 目标 |
|------|------|------|
| 新增 ExecutionPlugin | 不存在 | 在 AbilityPlugin 之后注册 |
| 新增 StackingPlugin | 不存在 | 在 EffectPlugin 之后注册 |
| 新增 CuePlugin | 不存在 | 在 ExecutionPlugin 之后注册 |
| 移除 BuffPlugin | 独立注册 | 吸收到 EffectPlugin |
| 新增 RegistryPlugin | 不存在 | 最底层 |
| 新增 BattlePipelinePlugin | 不存在 | 最顶层 |
| 新增 BattleReplayPlugin | 不存在 | 最顶层 |
| AttributeDefPlugin → AttributePlugin | 仅 def 注册 | 扩展为完整模块 Plugin |
| **调整执行链时序** | 无 | Ability → Effect → Stacking → Execution → Modifier → Attribute → Tag → Cue |

---

## 7. 分阶段执行计划

### Phase 1：核心新模块创建 🔴（~8 天）

**目标**：创建 ADR-026 声明但完全不存在的 3 个核心模块（Execution、Cue、Stacking）。

#### P1.1 创建 Stacking 模块（2 天）

| 任务 | 交付物 | 说明 | 工时 |
|------|--------|------|------|
| 1.1.1 | `src/core/stacking/types.rs` | `StackingRule` 4-enum 冻结版（Replace / RefreshDuration / StackAdd / StackMax(u32)） | 0.5 天 |
| 1.1.2 | `src/core/stacking/resolver.rs` | `resolve_stacking()` 纯函数 — 输入当前 Effect + 目标状态，输出堆叠结果 | 0.5 天 |
| 1.1.3 | `src/core/stacking/mod.rs` | `StackingPlugin` + 公共 API 重导出 | 0.5 天 |
| 1.1.4 | 废弃旧 `StackPolicy` | 在 `buff/domain/types.rs` 标记 #[deprecated]，添加 From 转换到 StackingRule | 0.5 天 |

**验证**：`cargo build` + `cargo test`。

#### P1.2 创建 Execution 模块（3 天）

| 任务 | 交付物 | 说明 | 工时 |
|------|--------|------|------|
| 1.2.1 | `src/core/execution/types.rs` | `ExecutionContext`, `ExecutionResult` 结构体 | 0.5 天 |
| 1.2.2 | `src/core/execution/mod.rs` | `ExecutionPlugin` + `Execution` trait | 0.5 天 |
| 1.2.3 | `src/core/execution/damage.rs` | `DamageExecution`, `TrueDamageExecution`, `CritExecution` — 从 effect/handler.rs 提取公式 | 1 天 |
| 1.2.4 | `src/core/execution/heal.rs` | `HealExecution` — 从 effect/handler.rs 提取 | 0.5 天 |
| 1.2.5 | `src/core/execution/shield.rs` | `ShieldExecution`（新增，当前无护盾实现） | 0.5 天 |
| 1.2.6 | 从 effect/handler.rs 移除公式 | DamageHandler.generate() 改为调用 ExecutionRegistry | 0.5 天（与上叠加） |

**验证**：`cargo build` + 伤害计算测试保持通过。

#### P1.3 创建 Cue 模块（3 天）

| 任务 | 交付物 | 说明 | 工时 |
|------|--------|------|------|
| 1.3.1 | `src/core/cue/types.rs` | `CueEvent` 枚举（CueDamage / CueDeath / CueHeal / CueBuffApply / CueShield / CueAbilityCast） | 0.5 天 |
| 1.3.2 | `src/core/cue/emitter.rs` | `CueEmitter` Resource — 业务层调用的发送接口 | 1 天 |
| 1.3.3 | `src/core/cue/mod.rs` | `CuePlugin` + 公共 API | 0.5 天 |
| 1.3.4 | 表现层 Cue 订阅器 | `CueSubscriber` System — 监听 CueEvent，分发给对应 UI/Tooltip/动画模块 | 1 天 |

**验证**：`cargo build` + 手动测试基本 Cue 下发。

### Phase 2：架构重构 🟡（~5 天）

**目标**：对齐 ADR-026 的关键架构变更（Buff 吸收、Plugin DAG、GAS 链）。

#### P2.1 Buff → Effect 吸收（3 天）

| 任务 | 交付物 | 说明 | 工时 |
|------|--------|------|------|
| 2.1.1 | Effect Duration 系统 | 在 `effect/types.rs` 添加 `DurationDef` 枚举（Instant / TurnLimited(u32) / Permanent） | 0.5 天 |
| 2.1.2 | EffectDef 扩展 | EffectDef 增加 Duration 字段，新增 `ApplyModifier` 变体替代 `ApplyBuff` | 1 天 |
| 2.1.3 | Buff 数据迁移 | 将 BuffData 的 modifiers / tags / dot_damage / hot_heal 字段映射到新的 Effect 系统 | 1 天 |
| 2.1.4 | Buff 模块标记废弃 | buff/ 模块标记 #[deprecated]，添加迁移指南注释 | 0.5 天 |

**验证**：`cargo build` + 现有 buff 功能通过测试。

#### P2.2 Plugin DAG 对齐 + GAS 链集成（2 天）

| 任务 | 交付物 | 说明 | 工时 |
|------|--------|------|------|
| 2.2.1 | 新增 Plugin 注册 | 在 app/plugin.rs 按 ADR-026 DAG 添加 ExecutionPlugin, StackingPlugin, CuePlugin | 0.5 天 |
| 2.2.2 | 移除 BuffPlugin | 从主注册链移除，保留兼容性重导出 | 0.5 天 |
| 2.2.3 | effect::pipeline 扩展 | 在 Effect → Modify → Execute 之间插入 Stacking → Execution 环节 | 1 天 |

**验证**：`cargo build` + 完整战斗流程测试。

### Phase 3：基建补齐 🟡（~5 天）

**目标**：实现 Registry、Pipeline、Replay 三个基础设施模块。

#### P3.1 Registry 统一注册中心（2 天）

| 任务 | 交付物 | 说明 | 工时 |
|------|--------|------|------|
| 3.1.1 | `src/infrastructure/registry/mod.rs` | `RegistryPlugin` + 统一注册类型 | 0.5 天 |
| 3.1.2 | 各 Registry 迁移 | 将散落的 Registry 注册收拢到 infrastructure/registry/ | 1 天 |
| 3.1.3 | 内容管线接入 | 新增 content/effects/ + content/executions/ + content/cues/ RON 加载入口 | 0.5 天 |

#### P3.2 Pipeline 战役执行管线（1.5 天）

| 任务 | 交付物 | 说明 | 工时 |
|------|--------|------|------|
| 3.2.1 | `src/infrastructure/pipeline/scheduler.rs` | 回合内 System 调度（整合 GAS 链各环节） | 1 天 |
| 3.2.2 | `src/infrastructure/pipeline/mod.rs` | `BattlePipelinePlugin` | 0.5 天 |

#### P3.3 Replay 确定性回放（1.5 天）

| 任务 | 交付物 | 说明 | 工时 |
|------|--------|------|------|
| 3.3.1 | `src/infrastructure/replay/record.rs` | `BattleRecord` + `CommandEntry` 数据结构 | 0.5 天 |
| 3.3.2 | `src/infrastructure/replay/player.rs` | `ReplayPlayer` — 确定性回放执行器 | 1 天 |

### Phase 4：内容管线对齐 🟢（~2 天）

| 任务 | 交付物 | 说明 | 工时 |
|------|--------|------|------|
| 4.1 | `content/attributes/` | 从 definitions/attributes.ron 迁移 | 0.5 天 |
| 4.2 | `content/effects/` | 初始 RON 文件（EffectDef 配置） | 0.5 天 |
| 4.3 | `content/executions/` | 初始 RON 文件（Execution 配置） | 0.5 天 |
| 4.4 | `content/cues/` | 初始 RON 文件（Cue 配置，可选） | 0.5 天 |

### Phase 5：质量加固 🟢（~5 天）

| 任务 | 交付物 | 说明 | 工时 |
|------|--------|------|------|
| 5.1 | Stacking 单元测试 | 4-enum 各场景测试 | 1 天 |
| 5.2 | Execution 单元测试 | Damage/Heal/Shield 各 Executor 测试 | 1 天 |
| 5.3 | Cue 集成测试 | CueEvent 下发 → 订阅测试 | 1 天 |
| 5.4 | GAS 链集成测试 | Ability → ... → Cue → Replay 全链路 | 1.5 天 |
| 5.5 | Buff 兼容性测试 | 旧 Buff 经 Effect + Duration 系统行为一致 | 0.5 天 |

---

## 8. 依赖图与关键路径

```
Phase 1：核心新模块创建 🔴（~8 天）— 关键路径
├── P1.1 Stacking 模块（2d）← 先做，Execution 和 Effect 依赖它
├── P1.2 Execution 模块（3d）← 依赖 Stacking 的类型定义
└── P1.3 Cue 模块（3d）← 可并行于 P1.2

Phase 2：架构重构 🟡（~5 天）— 依赖 Phase 1
├── P2.1 Buff → Effect 吸收（3d）
├── P2.2 Plugin DAG + GAS 链（2d）← 依赖 Phase 1 全部完成

Phase 3：基建补齐 🟡（~5 天）— 可部分并行于 Phase 2
├── P3.1 Registry（2d）
├── P3.2 Pipeline（1.5d）
└── P3.3 Replay（1.5d）

Phase 4：内容管线 🟢（~2 天）— 可并行
└── content/ 新目录 RON 文件

Phase 5：质量加固 🟢（~5 天）— 依赖 Phase 1-2
└── 各模块测试 + 集成测试

关键路径：Phase 1 → Phase 2 → Phase 5 = ~18 天
Phase 3 + Phase 4 可独立并行，不增加总工期。
```

---

## 9. 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| Buff → Effect 吸收破坏现有功能 | 高 | 高 | 保留旧 Buff 模块直到 Phase 2 全部验证通过；渐进式替换，每步都跑测试 |
| Execution 模块拆分配置不当导致 API 不稳定 | 中 | 高 | 先在 effect/ 内部重构抽出 trait，稳定后再迁出到独立模块 |
| Cue 引入导致 UI 层大面积改动 | 中 | 中 | Cue 作为新事件层叠加在现有 Message 之上，不立即移除旧 Message |
| Stacking 4-enum 与旧 3-enum 不兼容 | 中 | 中 | 保留 From<StackPolicy> for StackingRule 转换，旧 RON 配置无缝升级 |
| Replay 确定性实现难度大 | 中 | 高 | 先使用 Command Stream 简化版，Phase 3 才完整实现 |
| 架构文档与代码不一致 | 低 | 中 | 每次 Phase 完成后同步更新 docs/ 交叉引用 |

---

## 10. 架构自检

- [x] 差距分析基于实际源码扫描（2026-06-15）
- [x] 每个模块的缺失程度明确分级（P0-P4）
- [x] 执行计划包含可验证的交付物和工时
- [x] GAS 执行链完全列出缺失环节
- [x] 内容管线和 Plugin 顺序差距已量化
- [x] 风险评估覆盖主要破坏性变更
- [ ] 需 attention：Buff → Effect 吸收是**最大重构风险**，建议在 Execution 和 Stacking 建成后再进行
- [ ] 需 attention：Cue 模块的设计需与 UI 架构文档（`docs/03-technical/ui-architecture-rules.md`）协调

---

## 附录：关键文件路径索引

| 引用 | 路径 | 作用 |
|------|------|------|
| ADR-026 | `docs/08-decisions/ADR-026-SRPG-Lite-GAS-架构对齐.md` | 架构对齐决策 |
| 架构总纲 | `docs/01-architecture/README.md` | 七层架构 + GAS 链说明 |
| 领域索引 | `docs/02-domain/README.md` | 所有领域文件索引 |
| Stacking 领域 | `docs/02-domain/stack-policy/stack-policy-rules.md` | StackingRule 4-enum 领域规则 |
| Execution 领域 | `docs/02-domain/execution/execution-rules.md` | Execution trait 领域规则 |
| Cue 领域 | `docs/02-domain/cue/cue-rules.md` | Cue 事件总线领域规则 |
| 当前 Plugin 注册 | `src/app/plugin.rs` | 28 个 Plugin 的 DAG 顺序 |
| 当前 Buff 实现 | `src/core/buff/` | 7 文件，待吸收 |
| 当前 Effect 实现 | `src/core/effect/` | 3 文件，待扩展 |
| 当前 Attribute 实现 | `src/core/attribute/` | 3 文件，待拆分 |
| 当前 content/ | `content/` | 10 目录，缺 6 个 |
| 现有计划参考 | `docs/09-planning/business-module-execution-plan.md` | 此前 v2.0 计划（ADR-025 视角） |
