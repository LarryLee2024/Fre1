---
id: 09-planning.feature-developer-implementation-roadmap
title: Implementation Roadmap — 基于代码审查的分阶段实施计划
status: draft
owner: feature-developer
created: 2026-06-17
updated: 2026-06-17
tags:
  - planning
  - implementation
  - roadmap
  - feature-developer
---

# Implementation Roadmap — 基于代码审查的分阶段实施计划

> **前置审查**: `docs/10-reviews/feature-developer-*-alignment.md`（4 文件）
> **审查结论**: 文档完整度 ~100%，代码实现进度 ~15%
> **目标**: 给出可执行的阶段性实施计划，明确每个阶段的执行 agent 和交付物

---

## 实施总纲

### 核心原则

1. **基础设施先行**：Shared 层工具就绪后，Capabilities 和 Domains 才有实现基础
2. **Capabilities 管线优先于 Domains**：Domains 是 Capabilities 的使用者，必须先有机制
3. **自底向上（Bottom-Up）**：L0 → L1/Capabilities → L1/Domains → L2 → 横切层
4. **每个阶段自包含**：每个阶段可独立编译、测试、交付，不产生跨阶段阻塞
5. 🟥 **文档先行**：@feature-developer 在编码前，必须确认{ADR + 领域规则 + Schema}存在且已阅读（启动条件 §强制前置）

### 角色分工速查

| 角色 | 职责 | 何时介入 |
|------|------|---------|
| **@domain-designer** | 定义"规则是什么" — 维护领域术语、不变量 | 新领域或领域规则变更时 |
| **@data-architect** | 定义"规则如何表达" — Schema 设计、ID 策略 | Schema 新增或变更时 |
| **@architect** | 定义"系统如何组织" — ADR、模块边界 | 架构决策或边界模糊时 |
| **@feature-developer** | 实现"如何做" — 编写 Rust 代码 | **每个阶段的执行主力** |
| **@test-guardian** | 验证"是否正确" — 测试代码、回放测试 | 每个阶段功能实现后 |
| **@code-reviewer** | 保证"质量合规" — 审查报告 | 每个阶段交付前 |
| **@refactor-guardian** | 监控技术健康 — 债务清单 | 定期或在重大重构前 |

### @feature-developer 启动条件（引自 `.qoder/agents/feature-developer.md`）

```text
🟥 强制前置：开始编码前必须确认以下文档存在且已阅读：
1. `docs/01-architecture/` 相关 ADR（架构决策）
2. `docs/02-domain/` 相关领域规则
3. `docs/04-data/` 相关 Schema 设计（如涉及数据结构）

最低要求：有 ADR + 领域规则。
理想输入：ADR + 领域模型 + Schema 设计 + 测试规范。
```

**本文档中每个 Phase 开头的「前置文档确认」板块即是为此启动条件服务的强制性检查清单。**

### 标准开发流程（引自 AGENTS.md）

```
@domain-designer（领域建模） → 输出：docs/02-domain/
@data-architect（数据架构）  → 输出：docs/04-data/
@architect（架构设计）       → 输出：docs/01-architecture/ + docs/08-decisions/（ADR 文档）
                                        ↓
@feature-developer（代码实现）→ 输出：src/（代码）+ docs/09-planning/（执行计划）
                                        ↓
@test-guardian（测试审查）    → 输出：docs/05-testing/ + tests/
@code-reviewer（代码审查）    → 输出：docs/10-reviews/
```

🟥 **当前状态中的文档缺口**：

| 标准流程定义的输出 | 当前存在？ | 缺口 |
|------------------|-----------|------|
| `docs/02-domain/`（@domain-designer） | ✅ 31 文件，完整 | — |
| `docs/04-data/`（@data-architect） | ✅ 33+ 文件，完整 | — |
| `docs/01-architecture/`（@architect 模块设计） | ✅ README + 19 ADR 文件 | — |
| `docs/08-decisions/`（@architect ADR 文档） | ❌ **目录不存在** | 🟥 @architect 未输出到此目录 |
| `docs/05-testing/` 测试计划（@test-guardian） | ⚠️ 总纲存在，无 per-domain 测试计划 | 🟡 理想输入需补齐 |

---

### 前置文档依赖总图

```
┌──────────────────────────────────────────────────────────────────┐
│                     Phase 0: 文档补齐阶段                          │
│  @architect: 创建 docs/08-decisions/ 并整理 ADR                   │
│  @test-guardian: 补齐 per-domain 测试计划                        │
└──────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌──────────────────────────────────────────────────────────────────┐
│          每个编码 Phase 的强制前置文档确认流程                       │
│                                                                   │
│  ① @feature-developer 阅读该 Phase 涉及的 ADR（01-architecture/） │
│  ② @feature-developer 阅读该 Phase 涉及的领域规则（02-domain/）  │
│  ③ @feature-developer 阅读该 Phase 涉及的 Schema（04-data/）    │
│  ④ 若前置文档缺失 → 停止，调用上游 agent 补齐                     │
│  ⑤ 确认完毕 → 开始编码                                           │
└──────────────────────────────────────────────────────────────────┘
                                │
                                ▼
                   ┌──── Phase A ────┐
                   │  Phase B ────   │
                   │  ...            │
                   └─────────────────┘
```

---

### 依赖总图

```
Phase 0 (文档补齐) ←── 无外部依赖，最先执行
    │
    ▼
Phase A (Foundation Infra) ←── 无代码依赖，依赖 Phase 0 的文档确认
    │
    ▼
Phase B (Capabilities Core) ←── 依赖 Phase A 的 shared 工具
    │
    ▼
Phase C (Infra Core)       ←── 依赖 Phase B 的 capabilities 管线
    │
    ▼
Phase D (Domains Foundation) ←── 依赖 Phase B + C
    │
    ▼
Phase E (Domains Core)     ←── 依赖 Phase D
    │
    ▼
Phase F (Domains Narrative) ←── 依赖 Phase E
    │
    ▼
Phase G (Cross-cutting)    ←── 依赖 Phase B~F
```

---

## Phase 0: 前置文档补齐（[强制] 所有 Phase 的前提）

> **为什么需要 Phase 0**：AGENTS.md 标准开发流程要求 @architect 输出 ADR 到 `docs/08-decisions/`，但该目录不存在。所有 ADR 当前分散在 `docs/01-architecture/` 子目录中。不补齐此项，后续 Phase 中 @feature-developer 的"强制前置文档确认"无法成立。

### 0-1: 创建 `docs/08-decisions/` 并整理 ADR

| 项 | 内容 |
|---|------|
| **为什么** | AGENTS.md 标准流程定义：@architect 输出 = `docs/01-architecture/`（模块设计）+ `docs/08-decisions/`（ADR 文档）。当前 `08-decisions/` 不存在 |
| **做什么** | 创建 `docs/08-decisions/README.md`，将 `docs/01-architecture/` 下的 19 个 ADR 索引到此处（复制或建立交叉引用） |
| **交付物** | `docs/08-decisions/README.md`（ADR 索引清单，标注每个 ADR 对应的原始位置 `01-architecture/*.md`） |
| **执行 agent** | @architect |
| **依据** | `.qoder/agents/architect.md` §5: "ADR 可以直接保存到 `docs/08-decisions/` 目录" |
| **检查方式** | `ls docs/08-decisions/` 应存在且包含 README.md |

### 0-2: 补齐 per-domain 测试计划

| 项 | 内容 |
|---|------|
| **为什么** | @feature-developer 理想输入 = ADR + 领域模型 + Schema + **测试规范**。当前 `docs/05-testing/` 只有总纲，无 per-domain 测试计划 |
| **做什么** | 为 Phase D~G 涉及的 15 个业务 Domain 编写测试计划（每个 domain 的 4 层测试结构定义） |
| **交付物** | `docs/05-testing/plans/tactical-test-plan.md` 等 per-domain 测试计划 |
| **执行 agent** | @test-guardian |

### Phase 0 交付清单

| # | 任务 | agent | 交付物 |
|---|------|-------|--------|
| 0-1 | 创建 `docs/08-decisions/` + ADR 索引 | @architect | `docs/08-decisions/README.md` |
| 0-2 | 补齐 per-domain 测试计划 | @test-guardian | `docs/05-testing/plans/*.md` |

**Phase 0 门禁**：
- [ ] `docs/08-decisions/` 目录存在且包含 README.md
- [ ] README.md 引用了所有 19 个 ADR 并标注了原始位置
- [ ] `docs/05-testing/plans/` 存在且包含至少 Phase D 三个 domain 的测试计划

**Phase 0 完成后，才可以进入 Phase A。**

---

## Phase A: 基础基础设施（Foundation Infra）

> **目标**: 补齐 Shared 层缺失的核心工具模块  
> **依赖**: 无代码依赖。但需要 Phase 0 文档补齐完成。  
> **验收标准**: `cargo test` 通过，所有新增模块有单元测试  
> **预计工作量**: 5–7 文件，~300 行

### 🟥 前置文档确认（@feature-developer 必读）

在 Phase A 编码开始前，@feature-developer **必须**确认以下文档已存在且已阅读：

| 文档 | 位置 | 确认内容 | 涉及任务 |
|------|------|---------|---------|
| ADR-041 | `docs/01-architecture/40-cross-cutting/ADR-041-replay-determinism.md` | 确定性 RNG 要求 | A-1 |
| ADR-042 | `docs/01-architecture/40-cross-cutting/ADR-042-save-persistence.md` | 存档时间戳 | A-2 |
| ID 策略 | `docs/04-data/foundation/id_strategy.md` | ID 类型定义、prefix 规范 | A-4 |
| 编码规范 | `docs/00-governance/coding-rules.md` | 错误处理规则 | A-3 |

**如上述文档缺失 → 立即停止，调用 @architect（ADR 缺失）或 @data-architect（Schema 缺失）补齐。**

### A-1: 确定性随机数 — `shared/random/`

| 项 | 内容 |
|---|------|
| **为什么** | P0 铁律 #3 (Replay First) — 无确定性 RNG，所有战斗逻辑无法保证回放兼容 |
| **做什么** | 实现 `SeededRng` 结构体，基于种子（u64）的确定性 PRNG，实现标准 `RngCore` trait |
| **交付物** | `src/shared/random/mod.rs` — `SeededRng` 或 `DeterministicRng` |
| **执行 agent** | @feature-developer |
| **下游通知** | 完成后通知 @architect 更新 ADR-041 中的实现状态 |

### A-2: 游戏时间 — `shared/time/`

| 项 | 内容 |
|---|------|
| **为什么** | Effect 生命周期、回放帧计数、Turn 系统均需要 GameTime |
| **做什么** | 实现 `GameTime`（帧计数 + 回合计数），纯 u64 递增，无 wall-clock |
| **交付物** | `src/shared/time/mod.rs` — `GameTime` + `TurnCount` |
| **执行 agent** | @feature-developer |

### A-3: 错误上下文工具 — `shared/error/`

| 项 | 内容 |
|---|------|
| **为什么** | 宪法禁止全局 AppError/anyhow，各领域需统一错误工具支撑错误链 |
| **做什么** | 提供 `ErrorContext`（包装源错误+领域标签+上下文信息） |
| **交付物** | `src/shared/error/mod.rs` |
| **执行 agent** | @feature-developer |

### A-4: ID 类型补齐

| 项 | 内容 |
|---|------|
| **为什么** | 当前缺失 `QuestId`、`SpellId`、`BuffDefId`、`TerrainDefId`、`RecipeDefId` 等类型；prefix 不一致（文档 `trg_` vs 代码 `trig`） |
| **做什么** | 在 `src/shared/ids/types.rs` 中补充缺失的 `define_string_id!` 调用；统一 prefix 与 `docs/04-data/foundation/id_strategy.md` 一致 |
| **执行 agent** | @feature-developer |

### Phase A 交付清单

| # | 任务 | agent | 预估 | 前置 |
|---|------|-------|------|------|
| A-1 | SeededRng 实现 | @feature-developer | 1 文件 · 50 行 | Phase 0 + 前置文档确认 |
| A-2 | GameTime 实现 | @feature-developer | 1 文件 · 30 行 | Phase 0 + 前置文档确认 |
| A-3 | ErrorContext 实现 | @feature-developer | 1 文件 · 80 行 | Phase 0 + 前置文档确认 |
| A-4 | ID 类型补齐 | @feature-developer | 1 文件 · 30 行 | Phase 0 + 前置文档确认 |
| | 单元测试 | @test-guardian | 每模块 ≥3 测试 | A-1~A-4 完成 |
| | 代码审查 | @code-reviewer | 审查报告 | 测试通过 |

---

## Phase B: Capabilities 核心管线（Capabilities Core）

> **目标**: 在已有的 C1 Foundation 基础上，填充 C2 Systems 和 Plugin 内容  
> **依赖**: Phase A（shared/random, shared/time, shared/error）。文档依赖 Phase 0。  
> **验收标准**: 每个 Capability Plugin 的 `build()` 至少注册 1 个 System，Capabilities 间可编译不报错  
> **预计工作量**: 15–25 文件，~1500 行

### 🟥 前置文档确认（@feature-developer 必读）

| 文档 | 位置 | 涉及任务 |
|------|------|---------|
| ADR-010 | `docs/01-architecture/10-capability-system/ADR-010-ability-pipeline.md` | B-1, B-2 (Effect/Ability) |
| ADR-011 | `docs/01-architecture/10-capability-system/ADR-011-modifier-pipeline.md` | B-1 (Modifier/Attribute) |
| ADR-012 | `docs/01-architecture/10-capability-system/ADR-012-stacking-trigger-cue.md` | B-3, B-4 |
| ADR-013 | `docs/01-architecture/10-capability-system/ADR-013-registry-hotreload.md` | B-2 |
| tag_domain | `docs/02-domain/tag_domain.md` | Tag 内部机制确认 |
| attribute_domain | `docs/02-domain/attribute_domain.md` | Attribute Systems |
| effect_domain | `docs/02-domain/effect_domain.md` | EffectPlugin 填充 |
| tag_schema | `docs/04-data/capabilities/tag_schema.md` | Tag Component 对齐 |
| effect_schema | `docs/04-data/capabilities/effect_schema.md` | Effect lifecycle 对齐 |
| coding-rules | `docs/00-governance/coding-rules.md` | 编码规范确认 |
| ECS 规则 | `.trae/rules/ECS规则.md` | 四级通信机制 |

### B-1: Systems 填充 — 优先第一批

以 Tag 模块为标准模板，为以下领域添加 Systems：

| 领域 | 建议首个 System | agent |
|------|----------------|-------|
| attribute | `AttributeInitializer`（初始化 Entity 属性） | @feature-developer |
| modifier | `ModifierApplier`（添加/移除 Modifier 响应） | @feature-developer |
| aggregator | `AggregationRunner`（触发属性聚合计算） | @feature-developer |
| spec | 无 systems 需要（Spec 是被动组件） | — |
| condition | `ConditionEvaluatorSystem`（定时重评估） | @feature-developer |

### B-2: Plugin 内容填充

| 当前为空的 Plugin | 需注册的内容 | agent |
|------------------|-------------|-------|
| `EffectPlugin` | Effect 相关 Event + Observer | @feature-developer |
| `RuntimePlugin` | Pipeline Event | @feature-developer |

### B-3: 领域事件补齐

当前只有 tag/attribute/effect 有 `events.rs`。需为以下领域补全：

| 缺少 events.rs 的领域 | agent |
|----------------------|-------|
| modifier | @feature-developer |
| aggregator | @feature-developer |
| gameplay_context | @feature-developer |
| spec | @feature-developer |
| condition | @feature-developer |
| trigger | @feature-developer |
| ability | @feature-developer |
| targeting | @feature-developer |
| execution | @feature-developer |
| stacking | @feature-developer |
| cue | @feature-developer |

### B-4: 单元测试 — 关键路径优先

| 领域 | 测试内容 | agent |
|------|---------|-------|
| attribute | AttributeContainer 插入/查询/派生 | @test-guardian |
| modifier | Modifier 运算（Add/Mul/Override）+ 优先级排序 | @test-guardian |
| aggregator | AggregationPipeline 执行顺序 | @test-guardian |
| effect | Effect 生命周期状态机 | @test-guardian |
| ability | Ability 状态机切换 | @test-guardian |

### Phase B 交付清单

| # | 任务 | agent | 前置 |
|---|------|-------|------|
| B-1 | 5 个 capabilities 添加 Systems | @feature-developer | Phase A + 前置文档确认 |
| B-2 | EffectPlugin + RuntimePlugin 填充 | @feature-developer | Phase A + 前置文档确认 |
| B-3 | 11 个 capabilities 补 events.rs | @feature-developer | Phase A + 前置文档确认 |
| B-4 | 5 个 capability 单元测试 | @test-guardian | B-1~B-3 |
| | 代码审查 + 架构合规检查 | @code-reviewer | B-4 通过 |

---

## Phase C: 基础设施核心（Infrastructure Core）

> **目标**: 实现 Infra 层 5 个模块的核心骨架  
> **代码依赖**: Phase B 的 Capabilities 管线  
> **文档依赖**: 以下 ADR + Schema 需确认  
> **验收标准**: `cargo build` 通过，Infra module 可被 Core 层正常引用  
> **预计工作量**: 15–20 文件，~1000 行

### 🟥 前置文档确认

| 文档 | 位置 | 涉及任务 |
|------|------|---------|
| ADR-041 | `docs/01-architecture/40-cross-cutting/ADR-041-replay-determinism.md` | C-2 (Replay) |
| ADR-042 | `docs/01-architecture/40-cross-cutting/ADR-042-save-persistence.md` | C-4 (Save) |
| ADR-043 | `docs/01-architecture/40-cross-cutting/ADR-043-command-input.md` | C-5 (Input) |
| ADR-013 | `docs/01-architecture/10-capability-system/ADR-013-registry-hotreload.md` | C-3 (Registry) |
| pipeline_schema | `docs/04-data/infrastructure/pipeline_schema.md` | C-1 |
| replay_schema | `docs/04-data/infrastructure/replay_schema.md` | C-2 |
| registry_schema | `docs/04-data/infrastructure/registry_schema.md` | C-3 |
| save_architecture | `docs/04-data/foundation/save_architecture.md` | C-4 |

### C-1: Pipeline 引擎 — `infra/pipeline/`

| 项 | 内容 |
|---|------|
| **为什么** | Effect Pipeline、Combat Pipeline 需要通用管线执行引擎 |
| **做什么** | 实现通用 Stage Pipeline（Stage 注册 → 排序 → 执行），支持前置/后置 Hook |
| **交付物** | `infra/pipeline/` — PipelineStage trait, StageRegistry, PipelineExecutor |
| **执行 agent** | @feature-developer |
| **前置依赖** | Phase B Capabilities |

### C-2: Replay 框架 — `infra/replay/`

| 项 | 内容 |
|---|------|
| **为什么** | Replay First 铁则 — 所有核心战斗逻辑必须可回放 |
| **做什么** | 实现 ReplayRecorder（录制 Command → Frame）+ ReplayPlayer（Frame → Command 回放）|
| **交付物** | `infra/replay/` — ReplayFrame, ReplayRecorder, ReplayPlayer, SeededRng 集成 |
| **执行 agent** | @feature-developer |
| **前置依赖** | Phase A-1 (SeededRng), Phase C-1 (Pipeline) |

### C-3: Registry 注册中心 — `infra/registry/`

| 项 | 内容 |
|---|------|
| **为什么** | Content 层加载配置后需要注册到全局 Registry；冲突检测、ID 分配 |
| **做什么** | 实现 GenericRegistry<T: StrongId>，支持注册/查找/冲突检测/反注册 |
| **交付物** | `infra/registry/` — Registry trait, GenericRegistry, RegistryPlugin |
| **执行 agent** | @feature-developer |

### C-4: Save 存档框架 — `infra/save/`

| 项 | 内容 |
|---|------|
| **为什么** | 游戏需要持久化存档，ADR-042 已定义完整架构 |
| **做什么** | 实现 SaveHeader + 基础序列化 + VersionedSave（版本号 + 数据） |
| **交付物** | `infra/save/` — SaveHeader, VersionedSave, Serializer trait |
| **执行 agent** | @feature-developer |
| **前置依赖** | Phase C-2 (Replay) — 复用确定性子系统 |

### C-5: Input 输入抽象 — `infra/input/`

| 项 | 内容 |
|---|------|
| **为什么** | 命令层基础，连接玩家输入到游戏指令 |
| **做什么** | 实现 InputCommand trait + CommandDispatcher + ActionQueue |
| **交付物** | `infra/input/` — InputCommand trait, CommandDispatcher, ActionQueue |
| **执行 agent** | @feature-developer |

### Phase C 交付清单

| # | 任务 | agent | 前置 |
|---|------|-------|------|
| C-1 | Pipeline 引擎 | @feature-developer | Phase B + 前置文档确认 |
| C-2 | Replay 框架 | @feature-developer | C-1 + 前置文档确认 |
| C-3 | Registry 注册中心 | @feature-developer | Phase B + 前置文档确认 |
| C-4 | Save 存档框架 | @feature-developer | C-2 + 前置文档确认 |
| C-5 | Input 输入抽象 | @feature-developer | Phase B + 前置文档确认 |
| | 单元测试 | @test-guardian | C-1~C-5 |
| | 代码审查 | @code-reviewer | 测试通过 |
| | 架构审查 | @architect | Infra 层架构合规确认 |

---

## Phase D: 业务领域基础层（Domains Foundation）

> **目标**: 实现 3 个 Foundation 领域（Tactical / Terrain / Faction）的标准 7 文件结构  
> **代码依赖**: Phase B (Capabilities) + Phase C (Infra Pipeline, Registry)  
> **文档依赖**: 以下 3 组文档需逐一确认  
> **验收标准**: 每个 domain 的 7 文件齐全，`cargo test` 通过  
> **预计工作量**: 每个 domain ~5–8 文件，~400–600 行；共计 ~1500 行

### 🟥 前置文档确认

| domain | ADR | 领域规则 | Schema |
|--------|-----|---------|--------|
| Tactical | `ADR-022-grid-terrain-faction.md` | `tactical_domain.md` | `domains/tactical_schema.md` |
| Terrain | `ADR-022-grid-terrain-faction.md` | `terrain_domain.md` | `domains/terrain_schema.md` |
| Faction | `ADR-022-grid-terrain-faction.md` | `faction_domain.md` | `domains/faction_schema.md` |

@feature-developer 在编码每个 domain 前必须逐文档阅读并确认。

### D-1: Tactical — 战术空间

| 项 | 内容 |
|---|------|
| **领域文档** | `tactical_domain.md` |
| **数据 Schema** | `domains/tactical_schema.md` |
| **ADR** | `ADR-022-grid-terrain-faction.md` |
| **实现步骤** | 按标准流程：① @domain-designer 确认领域规则 → ② @data-architect 确认 Schema → ③ @feature-developer 实现 |

| 文件 | 内容 | agent |
|------|------|-------|
| `components.rs` | GridPosition, MovementComponent, CoverComponent | @feature-developer |
| `systems/mod.rs` + `movement_system.rs` | 移动逻辑系统 | @feature-developer |
| `events.rs` | UnitMoved, PositionChanged | @feature-developer |
| `error.rs` | TacticalError 枚举 | @feature-developer |
| `rules/formulas.rs` + `rules/rules.rs` | 移动消耗、掩体计算纯函数 | @domain-designer → @feature-developer |
| `integration.rs` | 调用 Capabilities (Tag, Modifier) | @feature-developer |
| `plugin.rs` | 注册 components + systems + events | @feature-developer |

### D-2: Terrain — 地形

| 项 | 内容 |
|---|------|
| **领域文档** | `terrain_domain.md` |
| **数据 Schema** | `domains/terrain_schema.md` |
| **实现步骤** | 同上 |

| 文件 | agent |
|------|-------|
| `components.rs` — TerrainTag, SurfaceType, MovementCost | @feature-developer |
| `systems/` — 地形效果系统 | @feature-developer |
| `events.rs` — TerrainEffectApplied | @feature-developer |
| `error.rs` — TerrainError | @feature-developer |
| `rules/` — 地形通行规则、效果规则 | @domain-designer → @feature-developer |
| `integration.rs` — 调用 Tag + Effect | @feature-developer |
| `plugin.rs` 更新 | @feature-developer |

### D-3: Faction — 阵营关系

| 项 | 内容 |
|---|------|
| **领域文档** | `faction_domain.md` |
| **数据 Schema** | `domains/faction_schema.md` |
| **实现步骤** | 同上 |

| 文件 | agent |
|------|-------|
| `components.rs` — FactionComponent, ReputationComponent | @feature-developer |
| `systems/` — 关系判定系统 | @feature-developer |
| `events.rs` — RelationChanged | @feature-developer |
| `error.rs` — FactionError | @feature-developer |
| `rules/` — 阵营关系公式、声望计算 | @domain-designer → @feature-developer |
| `integration.rs` — 调用 Tag | @feature-developer |
| `plugin.rs` 更新 | @feature-developer |

### Phase D 交付清单

| # | 任务 | agent | 前置 |
|---|------|-------|------|
| D-1 | Tactical Domain (7 文件) | 多 agent 协作 | Phase B + C + 前置文档确认 |
| D-2 | Terrain Domain (7 文件) | 多 agent 协作 | Phase B + C + 前置文档确认 |
| D-3 | Faction Domain (7 文件) | 多 agent 协作 | Phase B + C + 前置文档确认 |
| | 领域规则确认 | @domain-designer | D 开始前 |
| | Schema 确认 | @data-architect | D 开始前 |
| | 单元测试 + 领域不变量测试 | @test-guardian | D-1~D-3 完成 |
| | 代码审查 | @code-reviewer | 测试通过 |

---

## Phase E: 战斗核心域（Domains Core — Combat Layer）

> **目标**: 实现 3 个战斗核心领域（Combat / Spell / Reaction）  
> **代码依赖**: Phase D (Tactical, Terrain, Faction)  
> **文档依赖**: 以下 3 组文档需逐一确认  
> **预计工作量**: 每个 domain ~5–10 文件，~500–800 行；共计 ~2000 行

### 🟥 前置文档确认

| domain | ADR | 领域规则 | Schema |
|--------|-----|---------|--------|
| Combat | `ADR-020-combat-pipeline.md`, `ADR-021-turn-state-machine.md` | `combat_domain.md` | `domains/combat_schema.md` |
| Spell | `ADR-023-spell-reaction.md` | `spell_domain.md` | `domains/spell_schema.md` |
| Reaction | `ADR-023-spell-reaction.md` | `reaction_domain.md` | `domains/reaction_schema.md` |

### E-1: Combat — 战斗全流程

| 项 | 内容 |
|---|------|
| **关键依赖** | Capabilities: ability, execution, effect, modifier, aggregator |
| **核心挑战** | 这是最复杂的 domain，涉及 CombatIntent → Generate → Modify → Execute → Resolve 全管线 |

| 文件 | agent |
|------|-------|
| `components.rs` — CombatStats, Initiative, TurnOrder, ActionPoint | @feature-developer |
| `systems/` — 回合流程、先攻排序、行动解析 | @feature-developer |
| `events.rs` — RoundStarted, TurnChanged, CombatExecuted, UnitDied | @feature-developer |
| `error.rs` — CombatError | @feature-developer |
| `rules/formulas.rs` + `rules/rules.rs` | @domain-designer → @feature-developer |
| `integration.rs` — 调用 ability/targeting/execution/effect | @feature-developer |
| `plugin.rs` 更新 — 注册 Turn State Machine | @feature-developer |

### E-2: Spell — 法术系统

| 项 | 内容 |
|---|------|
| **关键** | Spell 复用 Ability Pipeline，核心差异在法术位管理 + 专注 + 豁免 |

| 文件 | agent |
|------|-------|
| `components.rs` — SpellSlot, Concentration, SaveDC | @feature-developer |
| `systems/` — 专注打断、法术位恢复 | @feature-developer |
| `events.rs` — SpellCast, ConcentrationBroken | @feature-developer |
| `error.rs` — SpellError | @feature-developer |
| `rules/` — 法术 DC 公式、豁免计算 | @domain-designer → @feature-developer |
| `integration.rs` — 调用 Ability + Execution | @feature-developer |

### E-3: Reaction — 反应/援护系统

| 项 | 内容 |
|---|------|
| **关键** | 基于 Event 的响应链（机会攻击/法术反制/护盾） |

| 文件 | agent |
|------|-------|
| `components.rs` — ReactionComponent, ReactionTrigger | @feature-developer |
| `systems/` — 反应链处理 | @feature-developer |
| `events.rs` — ReactionTriggered, ReactionConsumed | @feature-developer |
| `error.rs` — ReactionError | @feature-developer |
| `rules/` — 反应触发条件、反应消耗 | @domain-designer → @feature-developer |
| `integration.rs` | @feature-developer |

### Phase E 交付清单

| # | 任务 | agent | 前置 |
|---|------|-------|------|
| E-1 | Combat Domain | 多 agent 协作 | Phase D + 前置文档确认 |
| E-2 | Spell Domain | 多 agent 协作 | Phase D + 前置文档确认 |
| E-3 | Reaction Domain | 多 agent 协作 | Phase D + 前置文档确认 |
| | 单元测试 + 回放测试 | @test-guardian | E-1~E-3 |
| | 代码审查 + 架构审查 | @code-reviewer + @architect | 测试通过 |

---

## Phase F: 业务领域核心层（Domains Core — Progression Layer）

> **目标**: 实现 4 个成长/队伍领域（Progression / Inventory / Party / CampRest）  
> **代码依赖**: Phase D + E  
> **文档依赖**: 以下 4 组文档需逐一确认  
> **预计工作量**: 每个 domain ~3–6 文件，~300–500 行；共计 ~1600 行

### 🟥 前置文档确认

| domain | ADR | 领域规则 | Schema |
|--------|-----|---------|--------|
| Progression | `ADR-030-progression-inventory.md` | `progression_domain.md` | `domains/progression_schema.md` |
| Inventory | `ADR-030-progression-inventory.md` | `inventory_domain.md` | `domains/inventory_schema.md` |
| Party | `ADR-031-party-camp-rest.md` | `party_domain.md` | `domains/party_schema.md` |
| CampRest | `ADR-031-party-camp-rest.md` | `camp_rest_domain.md` | `domains/camp_rest_schema.md` |

| # | 任务 | agent | 前置 |
|---|------|-------|------|
| F-1 | Progression Domain | @domain-designer + @feature-developer + @test-guardian + @code-reviewer | Phase E + 前置文档确认 |
| F-2 | Inventory Domain | 同上 | Phase E + 前置文档确认 |
| F-3 | Party Domain | 同上 | Phase E + 前置文档确认 |
| F-4 | CampRest Domain | 同上 | Phase E + 前置文档确认 |

---

## Phase G: 叙事与经济域（Domains Narrative）

> **目标**: 实现 5 个叙事/经济领域（Narrative / Quest / Economy / Crafting / Summon）  
> **代码依赖**: Phase E + F  
> **文档依赖**: 以下 5 组文档需逐一确认  
> **预计工作量**: 每个 domain ~3–6 文件，~200–400 行；共计 ~1500 行

### 🟥 前置文档确认

| domain | ADR | 领域规则 | Schema |
|--------|-----|---------|--------|
| Narrative | `ADR-033-narrative-quest-summon.md` | `narrative_domain.md` | `domains/narrative_schema.md` |
| Quest | `ADR-033-narrative-quest-summon.md` | `quest_domain.md` | `domains/quest_schema.md` |
| Economy | `ADR-032-economy-crafting.md` | `economy_domain.md` | `domains/economy_schema.md` |
| Crafting | `ADR-032-economy-crafting.md` | `crafting_domain.md` | `domains/crafting_schema.md` |
| Summon | `ADR-033-narrative-quest-summon.md` | `summon_domain.md` | `domains/summon_schema.md` |

| # | 任务 | agent | 前置 |
|---|------|-------|------|
| G-1 | Narrative Domain | @domain-designer + @feature-developer + @test-guardian + @code-reviewer | Phase F + 前置文档确认 |
| G-2 | Quest Domain | 同上 | Phase F + 前置文档确认 |
| G-3 | Economy Domain | 同上 | Phase F + 前置文档确认 |
| G-4 | Crafting Domain | 同上 | Phase F + 前置文档确认 |
| G-5 | Summon Domain | 同上 | Phase F + 前置文档确认 |

---

## Phase H: 横切层 + 内容系统（Cross-cutting）

> **目标**: 补齐 4 个横切层的实现  
> **代码依赖**: Phase B~G（所有 Core 层完成）  
> **文档依赖**: 以下文档需逐一确认  
> **预计工作量**: 8–12 文件，~800 行

### 🟥 前置文档确认

| 层 | ADR | 相关文档 |
|----|-----|---------|
| Content | — | `docs/04-data/foundation/id_strategy.md`（Registry 集成） |
| mod_api | — | 无独立 ADR，需 @architect 输出 API 边界 ADR |
| Tools | — | 无独立 ADR |
| Modding | — | 无独立 ADR，需 @architect 输出 Mod 架构 ADR |

| # | 任务 | agent | 前置 | 说明 |
|---|------|-------|------|------|
| H-1 | ContentPlugin | @feature-developer | Phase C-3 (Registry) | Asset loader + Config watcher + Validation pipeline |
| H-2 | mod_api | @feature-developer | Phase D~G | 导出 stable API 给 Mod 使用 |
| H-3 | ModdingPlugin | @feature-developer | H-2 | Mod 加载沙箱 |
| H-4 | DevToolsPlugin | @feature-developer | Phase F | Debug 面板（仅 dev feature） |
| | 全量集成测试 | @test-guardian | H-1~H-4 | `cargo test --all-features` |
| | 最终代码审查 + 架构审查 | @code-reviewer + @architect | 集成测试通过 | 全项目最终审查 |

---

## 阶段拦截与质量门禁

每个阶段完成后必须通过以下门禁才能进入下一阶段：

### 门禁清单

| 检查项 | 通过标准 | 审查者 |
|--------|---------|--------|
| 🟥 **前置文档确认** | Phase 定义的所有前置文档已阅读并确认 | @feature-developer（Phase 启动时自检） |
| `cargo build` | ✅ 零错误 | CI |
| `cargo test` | ✅ 所有测试通过 | CI |
| `cargo clippy -- -D warnings` | ✅ 零 warning | CI |
| 目录结构合规 | ✅ 符合 Feature First + C1/C2 标准 | @architect |
| Plugin 注册顺序 | ✅ 符合 Phase 0–9 | @architect |
| TODO 注释合规 | ✅ 无裸 TODO，格式 `[Px][domain][date]:` | @code-reviewer |
| 红线检查 | ✅ 不违反宪法 21 条红线 | @code-reviewer |
| 领域不变量 | ✅ Domain invariants 测试通过 | @test-guardian |
| Replay 兼容性 | ✅ 回放测试通过（如有 replay 相关变更） | @test-guardian |
| 🟥 **异常处理** | 发现 ADR/领域模型/Schema 问题 → 立即停止，调用上游 agent 补齐 | @feature-developer（引用铁律1） |

### 阶段退出条件

```
Phase 0 ──→ Phase A ──→ Phase B ──→ Phase C ──→ Phase D ──→ ... ──→ Phase H
   │           │           │           │           │                    │
   ▼           ▼           ▼           ▼           ▼                    ▼
 文档门禁    编码门禁     编码门禁    编码门禁    前置文档              最终门禁
                         + 前置文档   + 前置文档  + 编码门禁
```

**如果门禁失败，则停留在当前阶段，修复后才能继续。**

### @feature-developer 异常处理流程（引自 prompt §发现问题时的处理流程）

如果在前置文档确认或编码过程中发现问题：

```
1. 立即停止编码
2. 输出反馈，包含：
   - 问题描述
   - 违反的规则编号
   - 建议的解决方案
3. 等待确认
4. 绝不私自修改架构、领域模型或数据架构

升级路径：
- 发现架构问题 → 调用 @architect
- 发现领域规则缺失 → 调用 @domain-designer
- 发现数据架构问题 → 调用 @data-architect
- 发现测试缺失 → 调用 @test-guardian
```

---

## 工作量总估

| Phase | 领域数 | 预估文件数 | 预估行数 | 主要 agent |
|-------|--------|-----------|---------|-----------|
| 0 | 0 (文档) | 2–5 | ~100 | @architect + @test-guardian |
| A | 4 (shared) | 5–7 | ~300 | @feature-developer |
| B | 15 (capabilities) | 15–25 | ~1500 | @feature-developer + @test-guardian |
| C | 5 (infra) | 15–20 | ~1000 | @feature-developer |
| D | 3 (domains foundation) | 15–24 | ~1500 | 全角色协作 |
| E | 3 (domains combat) | 15–30 | ~2000 | 全角色协作 |
| F | 4 (domains progression) | 12–24 | ~1600 | 全角色协作 |
| G | 5 (domains narrative) | 15–30 | ~1500 | 全角色协作 |
| H | 4 (cross-cutting) | 8–12 | ~800 | @feature-developer |
| **总计** | **43 领域** | **~100–170** | **~10,000** | |

---

## 风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| `docs/08-decisions/` 未及时创建 | 中 | 高（阻塞 Phase 0 门禁） | Phase 0 独立，A 阶段开始前完成即可 |
| Shared 层实现被延误 | 低 | 高（阻塞 Phase B） | Phase A 每个模块独立实现，可并行 |
| Capabilities Plugin 注册空但被依赖 | 中 | 中 | B-2 优先处理 Effect + Runtime 两个空 Plugin |
| Domain 规则确认耗时 | 中 | 中 | 领域规则已在文档中，@domain-designer 只需确认而非重写 |
| 回放系统复杂度高 | 中 | 高 | Phase C-2 只做最小可回放框架，后续逐步增强 |
| 🟥 **前置文档缺失导致编码被阻塞** | 低 | 高 | 按照异常处理流程，立即停止并调用上游 agent |
| 某领域实现时发现架构缺陷 | 低 | 高 | 立即停止，调用 @architect 输出 ADR 修正后再继续 |

---

## 附录 A：Phase 0 文档补齐详细说明

### `docs/08-decisions/` 需包含的内容

当前 ADR 位于 `docs/01-architecture/` 子目录中。@architect 需创建 `docs/08-decisions/` 目录，提供索引 README.md：

```markdown
# Architecture Decision Records (ADR)

> ADR 原始文档保存在 `docs/01-architecture/` 对应子目录中。
> 本文档为索引，方便快速查阅。

## Foundation
| 编号 | 标题 | 位置 | 状态 |
|------|------|------|------|
| ADR-000 | Feature 模块划分总图 | `01-architecture/00-foundation/ADR-000-*.md` | ✅ |
| ADR-001 | Plugin 组合与注册顺序 | `01-architecture/00-foundation/ADR-001-*.md` | ✅ |
| ... | ... | ... | ... |
```

未来新增 ADR 可以直接保存到 `docs/08-decisions/ADR-NNN-title.md`（参考 `architect.md` §5）。

### Per-domain 测试计划需包含的内容

每个 domain 的测试计划（由 @test-guardian 输出）应包含：

```markdown
# <Domain> Test Plan

## unit tests
- 测试目标（纯函数列表）
- 测试数据（fixtures）

## integration tests
- 协作场景（systems 配合测试）

## invariant tests
- 领域不变量清单
- 每个不变量对应的测试用例

## fixtures
- Builder 模式 / RON 文件
```

---

## 附录 B：前置文档确认清单模板（供 @feature-developer 使用）

每个 Phase 启动时，@feature-developer 应将以下清单填入执行记录：

```markdown
## [Phase X] 前置文档确认清单

| 文档 | 位置 | 已阅读 | 备注 |
|------|------|--------|------|
| ADR-NNN | `docs/01-architecture/.../ADR-NNN-*.md` | ✅/❌ | 关键决策摘要 |
| xxx_domain.md | `docs/02-domain/xxx_domain.md` | ✅/❌ | 领域规则摘要 |
| xxx_schema.md | `docs/04-data/xxx/xxx_schema.md` | ✅/❌ | Schema 结构摘要 |

确认结论：[通过 / 阻塞] — 如阻塞，描述缺失内容及需哪个 agent 补齐。
```

---

## 附录 C：与 AGENTS.md 协作流程的完整映射

| 活动 | 对应 AGENTS.md 流程 | 执行阶段 |
|------|-------------------|---------|
| 创建 ADR 索引 | @architect → 输出到 `docs/08-decisions/` | Phase 0 |
| 新增领域规则 | @domain-designer → 输出到 `docs/02-domain/` | Phase D 开始前 |
| 数据架构设计 | @data-architect → 输出到 `docs/04-data/` | Phase D 开始前 |
| 新增功能（已有领域规则） | 标准开发流程（skip domain-designer） | Phase D~G |
| 新增功能（需新领域规则） | @domain-designer → @data-architect → @architect → @feature-developer | Phase D 开始前 |
| Bug 修复 | @test-guardian（写失败测试）→ @feature-developer（修代码）→ @test-guardian（验证）→ @code-reviewer | 任何阶段 |
| 重构优化 | @refactor-guardian（发现债务）→ @architect（评估）→ @feature-developer（实现）→ @code-reviewer | Phase B 后定期 |
| 数据变更 | @data-architect（Schema）→ @architect（架构适配）→ @feature-developer（迁移实现） | 任何涉及 Schema 变更时 |
| 代码实现 | @feature-developer → 输出到 `src/` + `docs/09-planning/` | Phase A~H |
| 测试审查 | @test-guardian → 输出到 `docs/05-testing/` + `tests/` | 每个 Phase 交付前 |
| 代码审查 | @code-reviewer → 输出到 `docs/10-reviews/` | 每个 Phase 交付前 |

---

*本文档基于 `docs/10-reviews/feature-developer-*-alignment.md` 的审查发现编制，经 @feature-developer 分析后输出。*
