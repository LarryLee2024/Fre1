---
id: 01-architecture.ADR-000
title: ADR-000 — Feature Module Map
status: proposed
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-000: Feature 模块划分总图

## 状态

**Proposed** — 等待 @domain-designer 和 @data-architect 评审确认。

## 背景

Fre 项目包含 30 个领域（15 Capabilities + 15 Business Domains）和 33+ 个数据 Schema。需要一个清晰的模块划分策略将这些领域映射到 Rust 模块结构，确保：
- 模块边界与领域边界对齐
- 依赖方向清晰可控
- 开发团队可以并行工作
- 代码组织结构反映业务概念（Feature First）

## 引用的领域规则与数据架构

- `docs/02-domain/` — 30 个领域规则文档（领域定义与不变量）
- `docs/04-data/README.md` — 四层数据架构与 ID 策略
- `.trae/rules/架构规则.md` — Feature First 原则、三层架构、禁止全局技术目录
- `.trae/rules/SRPG专项规则.md` — 战斗管线、回合状态机、命令层

## 决策

采用 **七层垂直堆叠 + 水平 Feature 独立** 的矩阵式模块划分方案。

### 核心原则

1. **一个 Feature 一个目录**：每个领域映射为一个独立的 `src/<feature>/` 目录
2. **Feature 禁止跨层**：每个 Feature 归属固定层（Layer 1–7），禁止跨层存在
3. **层间单向依赖**：Layer N 可以依赖 Layer N-1 及以下，禁止反向依赖
4. **同层独立**：同层 Feature 之间禁止直接引用类型，仅通过 Event 通信
5. **目录即边界**：`src/` 下的一级目录就是所有 Feature，禁止创建 `components/` 等全局技术目录

### 模块划分清单

#### Layer 1 — Tactical Foundation (5 features)

| 目录 | 领域文档 | 数据 Schema | 核心 ECS 产出 |
|------|---------|------------|--------------|
| `grid_map/` | `tactical_domain` | `tactical_schema` | `GridMap` Resource, `Tile` Entity |
| `terrain/` | `terrain_domain` | `terrain_schema` | `TerrainDef` Asset, `Terrain` Component |
| `faction/` | `faction_domain` | `faction_schema` | `Faction` Tag, `FactionRelation` Resource |
| `turn_phase/` | `tactical_domain` | `tactical_schema` | `TurnPhase` State, `TurnQueue` Resource |
| `movement/` | `tactical_domain` | `tactical_schema` | `MoveCommand` Event, Pathfinding Systems |

#### Layer 2 — Capability System (15 features)

| 目录 | 领域文档 | 数据 Schema | 核心 ECS 产出 |
|------|---------|------------|--------------|
| `tag/` | `tag_domain` | `tag_schema` | `Tag` Component, `TagHierarchy` Resource |
| `attribute/` | `attribute_domain` | `attribute_schema` | `AttributeDef` Asset, `AttributeSet` Component |
| `modifier/` | `modifier_domain` | `modifier_schema` | `Modifier` Component, `ModifierTarget` Component |
| `aggregator/` | `aggregator_domain` | `aggregator_schema` | `Aggregator` Resource, Aggregation Systems |
| `gameplay_context/` | `gameplay_context_domain` | `gameplay_context_schema` | `GameplayContext` Resource |
| `spec/` | `spec_domain` | `spec_schema` | `AbilitySpec` Component |
| `condition/` | `condition_domain` | `condition_schema` | `Condition` Trait + types |
| `trigger/` | `trigger_domain` | `trigger_schema` | `TriggerDef` Asset, `TriggerEvent` types |
| `ability/` | `ability_domain` | `ability_schema` | `AbilityDef` Asset, `AbilityInstance` Component |
| `targeting/` | `targeting_domain` | `targeting_schema` | Targeting Systems, `TargetSet` Component |
| `execution/` | `execution_domain` | `execution_schema` | Execution Pipeline Systems |
| `effect/` | `effect_domain` | `effect_schema` | `EffectDef` Asset, `ActiveEffect` Component |
| `stacking/` | `stacking_domain` | `stacking_schema` | Stacking Rules, `StackGroup` Component |
| `event/` | `event_domain` | `event_schema` | `DomainEvent` types, Event Bus |
| `cue/` | `cue_domain` | `cue_schema` | `CueDef` Asset, `CueSignal` Event |

#### Layer 3 — Combat Execution (3 features)

| 目录 | 领域文档 | 数据 Schema | 核心 ECS 产出 |
|------|---------|------------|--------------|
| `combat/` | `combat_domain` | `combat_schema` | `CombatIntent` Event, Pipeline Systems |
| `spell/` | `spell_domain` | `spell_schema` | `SpellDef` Asset, `SpellCast` Event |
| `reaction/` | `reaction_domain` | `reaction_schema` | `Reaction` Component, Observer Systems |

#### Layer 4 — Progression & Economy (5 features)

| 目录 | 领域文档 | 数据 Schema | 核心 ECS 产出 |
|------|---------|------------|--------------|
| `progression/` | `progression_domain` | `progression_schema` | `Experience` Component, LevelUp Systems |
| `inventory/` | `inventory_domain` | `inventory_schema` | `Inventory` Component, `Item` Entity |
| `economy/` | `economy_domain` | `economy_schema` | `Currency` Component, Shop Systems |
| `crafting/` | `crafting_domain` | `crafting_schema` | Crafting Systems, `RecipeDef` Asset |
| `summon/` | `summon_domain` | `summon_schema` | `Summon` Component, Minion Systems |

#### Layer 5 — Party & Camp (2 features)

| 目录 | 领域文档 | 数据 Schema | 核心 ECS 产出 |
|------|---------|------------|--------------|
| `party/` | `party_domain` | `party_schema` | `Party` Resource, `PartyMember` Component |
| `camp_rest/` | `camp_rest_domain` | `camp_rest_schema` | Camp State, Rest Systems |

#### Layer 6 — Narrative & Content (2 features)

| 目录 | 领域文档 | 数据 Schema | 核心 ECS 产出 |
|------|---------|------------|--------------|
| `narrative/` | `narrative_domain` | `narrative_schema` | Story State, Dialogue Systems |
| `quest/` | `quest_domain` | `quest_schema` | `Quest` Component, `QuestTracker` Resource |

#### Layer 7 — Infrastructure & Cross-cutting (6 features)

| 目录 | 数据 Schema | 核心 ECS 产出 |
|------|------------|--------------|
| `registry/` | `registry_schema` | `Registry` Resource, Hot-reload Systems |
| `pipeline/` | `pipeline_schema` | `Pipeline` Resource, Execution Engine |
| `replay/` | `replay_schema` | `ReplayRecorder` Resource, Deterministic RNG |
| `save/` | `save_architecture` | Save/Load Systems, Migration Systems |
| `input/` | — | `Command` types, Input Mapping Resource |
| `common/` | — | 纯工具函数，零业务逻辑 |

## Module Design

### 每个 Feature 目录的标准结构

```
src/<feature>/
├── mod.rs              # 重新导出，pub mod 声明
├── plugin.rs           # Plugin impl (唯一对外入口)
├── components.rs       # 该 Feature 的 ECS Components
├── systems.rs          # 内部 System 声明（调度顺序 in-code）
├── events.rs           # 该 Feature 的事件类型（含白名单标记）
├── resources.rs        # Resources（如果适用）
├── api.rs              # 公开类型/函数（跨 Feature 只读访问用）
└── internal/
    ├── mod.rs
    ├── helpers.rs      # 内部辅助函数
    └── pipelines.rs    # 内部管线分段（仅适用于 pipeline 承载 Feature）
```

### 可选文件

| 文件 | 适用场景 |
|------|---------|
| `bundles.rs` | 常用 Component 组合预配置 |
| `config.rs` | Feature 级别的配置 Resource |
| `definitions.rs` | 该 Feature 的 Definition 类型（如果与 components 分离） |
| `test_helpers.rs` | 测试 Builder 工具（`#[cfg(test)]` 门控） |

## Communication Design

- **同 Feature 内部**：Trigger + Observer（事件链）、Changed Filter（状态变化）
- **跨 Feature 同层**：Message (Event) — 通过 `events.rs` 声明
- **跨层调用**：上层直接调用下层的 `api.rs` 公开函数
- **只读跨层查询**：下层的 `api.rs` 暴露 pub Query 函数

详见 ADR-002（ECS 四级通信机制）。

## 边界定义

### 允许
- Layer N 的 Feature 调用 Layer N-1 及以下的 `api.rs` 公开函数
- 同层 Feature 之间发送/接收 Event
- 所有 Feature 读取 Cross-cutting Layer 的公共工具

### 🟥 禁止
- Layer N 的 Feature 直接引用 Layer N+1 的 Component/Resource/Event
- 同层 Feature A 直接修改 Feature B 的 Component
- Feature 越过 Plugin 直接暴露 `pub mod internal`（必须通过 `plugin.rs`）
- 在 `src/` 下创建全局 `components.rs`/`systems.rs`/`events.rs` 技术文件
- 任何 Feature 依赖上层 Feature 的内部实现

## Forbidden

- 🟥 **禁止跨层引用**：Layer 3 禁止引用 Layer 5 的 Party Component
- 🟥 **禁止同层直接引用**：`combat/` 禁止直接引用 `spell/` 的内部类型
- 🟥 **禁止 Feature 内部泄漏**：所有 Feature 的 `internal/` 模块不得在 `mod.rs` 中 pub
- 🟥 **禁止目录与领域不对齐**：不允许在已有 Feature 目录外创建新的顶层目录

## Definition / Instance Design

此 ADR 关注模块划分，具体数据结构定义见各 Schema 文档（`docs/04-data/`）。

## 后果

### 正面
- 30 个领域按依赖关系组织成 7 层，依赖方向清晰
- Feature 与领域 1:1 映射，查找代码无需猜测
- 团队可以并行开发不同层的 Feature
- 新开发者通过目录结构就能理解游戏架构

### 负面
- 跨层调用需要经过 `api.rs`，初期增加少量样板代码
- 35 个 Feature 目录在 IDE 中看起来较多，但每组有明确的层归属

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 按技术拆目录（components/systems/events） | 违反 Feature First 原则，无法反映业务边界 |
| 按宪法三层拆（domain/application/presentation） | 粒度太粗，35 个领域无法在 3 个目录下组织 |
| 单层全平铺 | 35 个目录平铺失去依赖方向信息 |

## 评审要点

- [ ] 每个 Feature 是否都有明确的领域对应？
- [ ] 层归属是否合理（有没有 Feature 放错层）？
- [ ] 35 个 Feature 是否过多需要合并？
- [ ] `movement/` 是否需要独立 Feature 还是并入 `grid_map/`？
