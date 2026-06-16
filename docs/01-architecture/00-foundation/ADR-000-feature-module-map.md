---
id: 01-architecture.ADR-000
title: ADR-000 — Feature Module Map
status: approved
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-000: Feature 模块划分总图

## 状态

**Approved** — 经 @domain-designer 和 @data-architect 评审确认，本架构决策正式生效。

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

采用 **DDD 纵向三层（Shared/Core/Infra）+ 横切四层（App/Content/Tools/Modding）** 的矩阵式模块划分方案，源自 `docs/00-governance/Fre项目架构设计.md`。

### 核心原则

1. **领域即目录**：每个领域映射为 `src/core/capabilities/<domain>/` 或 `src/core/domains/<domain>/`
2. **内聚优于分层**：同一领域的代码（Foundations + Mechanism）放在同一目录下，而非按抽象层级拆散
3. **层间单向依赖**：Shared ← Core ← Infra，禁止反向依赖
4. **域间仅事件通信**：Domains 之间禁止直接引用，仅通过 Event 通信
5. **严格三层分离**：Def（配置）→ Spec（槽位）→ Instance（运行时），贯穿能力系统全链路
6. **目录即边界**：禁止创建 `components/`、`systems/` 等全局技术目录

### 模块划分清单

#### L0: Shared — 原子层 (`src/shared/`)

零业务语义的通用编程原子工具，不自成独立领域。

| 子模块 | 核心产出 |
|--------|---------|
| `ids/` | UnitId, SkillId, BuffId, ItemId, QuestId 等强类型 ID |
| `math/` | 距离、插值、网格坐标变换 |
| `random/` | 确定性 SeededRng Trait |
| `time/` | GameTime, TurnCount |
| `error/` | ErrorContext, LogIfError |
| `collections/` | SparseSet 等 ECS 友好集合 |
| `hashing/` | 非加密高速哈希封装 |
| `validation/` | 链式校验器 |
| `testing/` | TestBuilder, TestWorld |
| `traits/` | 横切能力抽象（日志/审计/事务） |

#### L1: Core — Capabilities (`src/core/capabilities/`)

15 个核心能力领域，通用机制骨架，与具体玩法无关。

| 目录 | 领域文档 | 数据 Schema | 分组 |
|------|---------|------------|------|
| `tag/` | `tag_domain` | `tag_schema` | 核心基石 |
| `attribute/` | `attribute_domain` | `attribute_schema` | 核心基石 |
| `modifier/` | `modifier_domain` | `modifier_schema` | 核心基石 |
| `aggregator/` | `aggregator_domain` | `aggregator_schema` | 核心基石 |
| `gameplay_context/` | `gameplay_context_domain` | `gameplay_context_schema` | 核心基石 |
| `spec/` | `spec_domain` | `spec_schema` | 逻辑骨架 |
| `condition/` | `condition_domain` | `condition_schema` | 逻辑骨架 |
| `trigger/` | `trigger_domain` | `trigger_schema` | 逻辑骨架 |
| `ability/` | `ability_domain` | `ability_schema` | 行为表现 |
| `targeting/` | `targeting_domain` | `targeting_schema` | 行为表现 |
| `execution/` | `execution_domain` | `execution_schema` | 行为表现 |
| `effect/` | `effect_domain` | `effect_schema` | 行为表现 |
| `stacking/` | `stacking_domain` | `stacking_schema` | 行为表现 |
| `event/` | `event_domain` | `event_schema` | 逻辑骨架 |
| `cue/` | `cue_domain` | `cue_schema` | 行为表现 |
| `runtime/` | — | `pipeline_schema` | C3 运行时编排 |

#### L1: Core — Business Domains (`src/core/domains/`)

15 个业务子系统，承载全部玩法复杂度。

| 目录 | 领域文档 | 数据 Schema | 核心职责 |
|------|---------|------------|---------|
| `tactical/` | `tactical_domain` | `tactical_schema` | 网格移动、掩体、夹击 |
| `terrain/` | `terrain_domain` | `terrain_schema` | 地形、通行性 |
| `faction/` | `faction_domain` | `faction_schema` | 阵营关系 |
| `combat/` | `combat_domain` | `combat_schema` | 回合流程、胜负判定 |
| `spell/` | `spell_domain` | `spell_schema` | 法术、专注、豁免 |
| `reaction/` | `reaction_domain` | `reaction_schema` | 机会攻击、援护 |
| `progression/` | `progression_domain` | `progression_schema` | 经验、等级、天赋 |
| `inventory/` | `inventory_domain` | `inventory_schema` | 背包、装备 |
| `party/` | `party_domain` | `party_schema` | 队伍、羁绊 |
| `camp_rest/` | `camp_rest_domain` | `camp_rest_schema` | 营地、休息 |
| `narrative/` | `narrative_domain` | `narrative_schema` | 对话树、演出 |
| `quest/` | `quest_domain` | `quest_schema` | 任务追踪 |
| `economy/` | `economy_domain` | `economy_schema` | 货币、商店 |
| `crafting/` | `crafting_domain` | `crafting_schema` | 配方、附魔 |
| `summon/` | `summon_domain` | `summon_schema` | 召唤物 |

#### L2: Infra — 技术实现层 (`src/infra/`)

| 目录 | 数据 Schema | 核心职责 |
|------|------------|---------|
| `registry/` | `infrastructure/registry_schema` | ID 注册、冲突检测、热重载 |
| `pipeline/` | `infrastructure/pipeline_schema` | 管线执行引擎 |
| `replay/` | `infrastructure/replay_schema` | 命令录制、确定性回放 |
| `save/` | `foundation/save_architecture` | 存档序列化、迁移 |
| `input/` | — | 输入抽象、命令层 |

#### 横切四层

| 层 | 路径 | 核心职责 |
|----|------|---------|
| App | `src/app/` | Composition Root |
| Content | `src/content/` | 配置加载、校验、注册 |
| Tools | `src/tools/` (feature-gated) | Dev 工具 |
| Modding | `src/modding/` | Mod 加载、API 稳定层 |

## Module Design

### 每个 Feature 目录的标准结构

**Capabilities** 位于 `src/core/capabilities/<domain>/`，内部按 C1→C2→C3 组织：

```
capabilities/<domain>/
├── plugin.rs              # 领域 Plugin（唯一对外入口）
├── foundation/            # C1：纯数据定义（类型、枚举、值对象）
│   ├── mod.rs
│   ├── types.rs
│   └── values.rs
├── mechanism/             # C2：规则与系统（Components, Systems, Events）
│   ├── mod.rs
│   ├── components.rs
│   ├── query.rs
│   ├── lifecycle.rs
│   └── systems/
│       ├── mod.rs
│       └── xxx_system.rs
└── events.rs              # 领域事件
```

**Business Domains** 位于 `src/core/domains/<domain>/`，标准 7 文件结构：

```
domains/<domain>/
├── plugin.rs          # 唯一对外入口
├── components.rs      # ECS Components
├── systems/           # 内部 Systems
│   ├── mod.rs
│   ├── xxx_system.rs
│   └── yyy_system.rs
├── events.rs          # 对外发布的领域事件
├── error.rs           # 专属错误枚举
├── rules/             # 纯业务规则（纯函数，零 ECS 依赖）
│   ├── formulas.rs
│   └── rules.rs
└── integration.rs     # 唯一调用 Capabilities 的入口
```

**Infrastructure** 位于 `src/infra/<domain>/`：

```
infra/<domain>/
├── mod.rs
├── plugin.rs
├── systems.rs
└── api.rs
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
- 30 个领域按 DDD三层+横切四层 组织，依赖方向清晰
- Capabilities 内聚 C1+C2+C3，不再跨目录拆分
- Domain 禁止直接引用，仅通过 Event 通信，解耦彻底
- 新开发者通过目录结构就能理解游戏架构

### 负面
- Capabilities 的 foundation/ + mechanism/ 子目录增加少量嵌套深度
- runtime/ 作为 C3 跨领域层需要谨慎维护公共接口

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 按技术拆目录（components/systems/events） | 违反 Feature First 原则，无法反映业务边界 |
| 按旧七层模型（flat src/<feature>） | 违反 Fre项目架构设计.md 的 DDD三层+横切四层模型，且 `src/common/` 违反宪法禁止的 common 模式 |
| 单层全平铺 | 30+ 目录平铺失去依赖方向信息 |

## 评审要点

- [ ] 每个 Feature 是否都有明确的领域对应？
- [ ] 层归属是否合理（有没有 Feature 放错层）？
- [ ] 35 个 Feature 是否过多需要合并？
- [ ] `movement/` 是否需要独立 Feature 还是并入 `grid_map/`？
