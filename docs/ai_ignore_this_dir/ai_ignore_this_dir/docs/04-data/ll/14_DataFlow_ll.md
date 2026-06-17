---
id: 04-data.ll.14_DataFlow
title: "数据流总览（代码实现映射）"
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
tags:
  - data-architect
  - data-flow
  - implementation
  - ll
---

# Data Architecture Proposal — 数据流总览（代码实现映射）

## Domain Ownership

归属领域：**Pipeline + Registry**（跨领域）
管辖范围：从 RON 文件到运行时 ECS 的完整数据流、所有领域的代码实现映射

## Problem

`data_relationship_overview.md` 描述了概念层的数据关系，但缺乏与实际 Rust 代码的映射。本文档将 13 个领域的数据模型与 `src/` 中的实现一一对应。

## 代码实现总览

### 1. 加载链路：RON → Def → Data → Component

```
content/*.ron（磁盘）
    │
    │ RON 反序列化
    ↓
*Def 结构体（Definition 层，不可变）
    │
    │ From<Def> for Data 转换
    ↓
*Data 结构体（Definition 层，不可变，运行时使用）
    │
    │ 存入 Registry（Resource）
    ↓
Registry.get(&id) → &Data（只读查询）
    │
    │ spawn 时从 Template 复制
    ↓
ECS Component（Instance 层，可变）
```

### 2. 战斗执行链路：Action → Effect → Cue

```
玩家/AI 输入
    │
    │ UiCommand / CombatIntent
    ↓
Ability 系统（验证 Cost/Cooldown/Requirement）
    │
    │ SkillData → EffectDef 列表
    ↓
Effect Pipeline
    │
    ├─→ Generate（生成 PendingEffect）
    │       │
    │       ↓
    │   Stacking（堆叠策略匹配）
    │       │
    │       ↓
    │   Execution（公式计算，ExecutionContext → ExecutionResult）
    │       │
    │       ↓
    │   Modifier（属性修饰，ModifierRule 匹配）
    │       │
    │       ↓
    │   Attribute（属性刷新）
    │       │
    │       ↓
    │   Tag（标签变更）
    │       │
    │       ↓
    │   Cue（表现事件广播）
    │
    └─→ BattleRecord（审计记录）
```

### 3. 回合执行管线

```
TurnStart
    │
    ├─→ 触发 TurnStart 事件
    ├─→ Buff Tick（DoT/HoT 结算）
    ├─→ AP/CP 回复
    │
ActionPhase（按速度排序）
    │
    ├─→ 选择单位（SelectUnit）
    ├─→ 移动（MoveUnit）
    ├─→ 执行动作（ExecuteAction）
    │       │
    │       └─→ Skill 释放管线（7 步）
    │           Validate → TargetSelect → CostDeduct
    │           → EffectExecute → TriggerFire → StateUpdate → CooldownSet
    │
    └─→ 动作结束（ActionEnd）
        │
TurnEnd
    │
    ├─→ 触发 TurnEnd 事件
    ├─→ Buff Tick
    ├─→ CD-1
    └─→ 胜负检查
```

## 13 领域代码映射表

### Core Domain（10 领域）

| 领域 | Def 类型 | Data/Component 类型 | Registry | 源码路径 | RON 路径 |
|------|----------|---------------------|----------|----------|----------|
| **Attribute** | `AttributeDef` | `AttributeDefinition` + `Attributes`（Component） | `AttributeRegistry` | `src/core/attribute/` | `content/attributes/attributes.ronic` |
| **Tag** | `TagDef` | `TagDefinition` + `GameplayTags`（Component） | `TagRegistry` | `src/core/tag/` | `content/tags/tags.ron` |
| **Modifier** | `ModifierRuleDef` | `ModifierRule` + `ModifierEntry` | `ModifierRuleRegistry` | `src/core/modifier/` | `content/modifiers/*.ron` |
| **Effect** | `EffectDef`（enum） | `PendingEffect` + `EffectResult` | `EffectQueue`（Resource） | `src/core/effect/` | `content/effects/*.ron` + inline |
| **Ability** | `SkillDef` | `SkillData` + `SkillSlots`/`SkillCooldowns` | `SkillRegistry` | `src/core/ability/` | `content/skills/*.ron` |
| **Trigger** | `TriggerDef` | `TriggerContext` | — | `src/core/trigger/` | — |
| **Targeting** | `TargetingDef` | `TargetingContext` + `SkillTargeting`（enum） | — | `src/core/targeting/` | — |
| **Execution** | `ExecutionDef` | `ExecutionContext` + `ExecutionResult` | `ExecutionRegistry` | `src/core/execution/` | `content/executions/*.ron` |
| **Stacking** | `StackingRuleDef` | `StackingRule`（enum） + `StackingResult` | — | `src/core/stacking/` | — |
| **Cue** | `CueDef` | `CueEvent`（Message） | `CueRegistry` | `src/core/cue/` | `content/cues/cues.ron` |

### Infrastructure Domain（3 领域）

| 领域 | 核心类型 | 源码路径 | 说明 |
|------|----------|----------|------|
| **Registry** | `Registry` trait + `LoadableRegistry` + `ValidatableRegistry` | `src/shared/registry/` | 统一注册表接口 |
| **Pipeline** | `EffectQueue` + `PendingEffect` + `EffectResult` | `src/core/battle/` + `src/core/effect/` | 战斗效果管线 |
| **Replay** | `BattleRecord` + `CommandEntry` + `ReplayPlayer` | `src/infrastructure/replay/` | 回放系统 |

## 四层数据边界统计

| 数据层 | 条目数 | 占比 | 说明 |
|--------|--------|------|------|
| **Definition** | 101 | 78% | 静态配置（RON → Def → Data） |
| **Instance** | 14 | 11% | 实例状态（Component，可变） |
| **Runtime** | 6 | 5% | 临时计算（ExecutionContext 等） |
| **Persistence** | 9 | 7% | 存档状态（Save/Replay） |
| **合计** | **130** | 100% | |

## ID 引用完整性矩阵

所有引用链必须在启动时校验闭合：

| 源（字段） | 目标 Registry | 必需 | 示例 |
|------------|---------------|------|------|
| `UnitTemplate.skill_ids[]` | SkillRegistry | 是 | `"basic_attack"` → SkillRegistry |
| `UnitTemplate.trait_ids[]` | TagRegistry | 是 | `"warrior_mastery"` → TagRegistry |
| `UnitTemplate.ai_behavior` | AiBehaviorRegistry | 是 | `"default"` → AiBehaviorRegistry |
| `UnitTemplate.initial_equipment[]` | EquipmentRegistry | 否 | `"iron_sword"` → EquipmentRegistry |
| `SkillDef.effects[]` | EffectRegistry | 否 | inline 或 ID 引用 |
| `SkillDef.tags[]` | TagRegistry | 是 | `FIRE` → TagRegistry |
| `SkillDef.conditions[]` | — | 否 | 内联条件 |
| `BuffDef.tags[]` | TagRegistry | 是 | `DEBUFF` → TagRegistry |
| `BuffDef.modifiers[]` | ModifierRuleRegistry | 否 | 内联修饰 |
| `ModifierRule.source_tag` | TagRegistry | 是 | `FIRE` → TagRegistry |
| `ModifierRule.target_tag` | TagRegistry | 是 | `FIRE` → TagRegistry |
| `LevelConfig.player_units[].template` | UnitTemplateRegistry | 是 | `"player_warrior"` → UnitTemplateRegistry |
| `LevelConfig.enemy_units[].template` | UnitTemplateRegistry | 是 | `"enemy_goblin"` → UnitTemplateRegistry |
| `LevelConfig.terrain_grid` | TerrainRegistry | 是 | `P`/`F`/`M`/`W` → TerrainRegistry |
| `Campaign.stages[].level_id` | LevelRegistry | 是 | `"tutorial"` → LevelRegistry |
| `AttributeDef.name_key` | FTL 文件 | 是 | `"attr.a_001.name"` → attr.ftl |

## 数值约定

| 约定 | 说明 | 示例 |
|------|------|------|
| 整数为主 | 游戏数值统一使用 `i32` | HP=100, ATK=5 |
| 万分比 | 百分比用万分比表示 | 50% = 5000, 150% = 15000 |
| 向下取整 | 伤害计算最终结果向下取整 | 37.8 → 37 |
| 最小值 | 伤害 ≥ 1（真实伤害 ≥ 0），治疗 ≥ 0 | — |
| 封顶值 | 暴击率 ≤ 95%，闪避率 ≤ 80%，减伤 ≤ 90% | — |

## Data Laws 合规总结

| Data Law | 状态 | 验证位置 |
|----------|------|----------|
| 001: Definition/Instance 分离 | ✅ | 所有 Def 与 Component 分离 |
| 002: Rule/Content 分离 | ✅ | 规则在代码中，内容在 RON 中 |
| 003: 配置只能引用 ID | ✅ | Registry 间通过 ID 引用 |
| 004: Ability 不拥有行为 | ✅ | 行为归属 Trigger |
| 005: Effect 是唯一执行入口 | ✅ | 所有业务结果经过 Effect |
| 006: Modifier 不拥有业务逻辑 | ✅ | Modifier 只改变数值 |
| 007: Duration 属于 Effect | ✅ | DurationDef 在 EffectDef 中 |
| 008: 堆叠行为归属 Stacking | ✅ | StackingRule 统一管理 |
| 009: 所有表现经过 Cue | ✅ | Cue 是表现层唯一入口 |
| 010: Replay 优先于便利 | ✅ | 所有设计可序列化、确定性 |
