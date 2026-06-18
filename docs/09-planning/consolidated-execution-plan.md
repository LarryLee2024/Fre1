---
id: 09-planning.consolidated-execution-plan
title: Consolidated Execution Plan — 全规划整合 + 骨架域填充路线图
status: active
owner: feature-developer
created: 2026-06-18
updated: 2026-06-18 (Phase E completed)
tags:
  - planning
  - implementation
  - roadmap
  - consolidated
---

# Consolidated Execution Plan — 全规划整合 + 骨架域填充路线图

> **前置规划文档**:
> - `feature-developer-implementation-roadmap.md` — 原路线图（Phase A~H）
> - `Phase-C-D-execution-plan.md` — M1 之前的并行执行计划
> - `Phase-post-M1-execution-plan.md` — M1 之后的执行跟踪
> - `integration-facade-plan.md` — Tactical Anti-Corruption Layer 计划
> - `Fre 项目领域文件清单与设计排序分析.md` — 30 领域依赖排序
> - `doc-conflict-evaluation.md` — 文档冲突修复记录
>
> **本文档目的**: 将以上 6 份规划的状态统一整合，提取所有未完成任务，与路径 B（骨架域批量填充）合并，形成可执行的最终计划。

---

## 0. 实测代码基线（2026-06-18 updated）

| 层 | 范畴 | 文件数 | 行数 | 状态 |
|----|------|--------|------|------|
| **Shared (L0)** | ids / random / time / error / testing | 30 | ~1,410 | ✅ 完成 |
| **Capabilities (L1)** | 15 机制域 + runtime | 199 | ~17,026 | ✅ Foundation+Mechanism+Events 完整 |
| **Domains (L1)** — 已实现 | tactical / combat / terrain / faction / narrative / progression / inventory / party / camp_rest | 182 | ~14,714 | ✅ 完整实现 |
| **Domains (L1)** — 已填充 | crafting / economy / quest / reaction / spell / summon | 96 | ~7,800 | ✅ Batch 3+4 完成 |
| **Infra (L2)** | pipeline / registry / replay / save / input | 53 | ~2,874 | ✅ 桥接层完成 |
| **横切层** | app / content / tools / modding | 11 | ~560 | 🟡 内容层实现中 |
| **测试** | 各域 tests/ | — | — | ✅ 1402 tests pass, 0 fail |

---

## 1. 全部规划完成状态总表

### 1.1 Phase A — Shared（✅ 全部完成）

| # | 任务 | 规划来源 | 状态 |
|---|------|---------|------|
| A-1 | SeededRng 实现 (`shared/random/`) | feature-developer-implementation-roadmap | ✅ 完成 |
| A-2 | GameTime 实现 (`shared/time/`) | feature-developer-implementation-roadmap | ✅ 完成 |
| A-3 | ErrorContext 实现 (`shared/error/`) | feature-developer-implementation-roadmap | ✅ 完成 |
| A-4 | ID 类型补齐 (`shared/ids/`) | feature-developer-implementation-roadmap | ✅ 完成 |
| A-T | Shared 层测试 | feature-developer-implementation-roadmap | ✅ 完成 |

### 1.2 Phase B — Capabilities（✅ 全部完成）

| # | 任务 | 规划来源 | 状态 |
|---|------|---------|------|
| B-1 | Systems 填充 — attribute/modifier/aggregator/condition | feature-developer-implementation-roadmap | ✅ 完成 |
| B-2 | EffectPlugin + RuntimePlugin 文档注释补充 | feature-developer-implementation-roadmap | ✅ 完成 |
| B-3 | 15 个 capability events.rs 补齐 | feature-developer-implementation-roadmap | ✅ 完成（摸底确认全部存在） |
| B-4 | Capability 单元测试 | feature-developer-implementation-roadmap | ✅ **已完成**（5 域不变量测试补齐，+31 测试） |

### 1.3 Phase C — Infrastructure（✅ 全部完成）

| # | 任务 | 规划来源 | 状态 |
|---|------|---------|------|
| C-1 | Pipeline 最小引擎 (`infra/pipeline/`) | Phase-C-D-execution-plan | ✅ 完成 |
| C-2 | Input 输入抽象 (`infra/input/`) | Phase-C-D-execution-plan | ✅ 完成 |
| C-3 | Registry 注册中心 (`infra/registry/`) | Phase-post-M1 | ✅ 完成 |
| C-4 | Replay 回放桥接层 (`infra/replay/`) | Phase-post-M1 | ✅ 完成 |
| C-5 | Save 存档桥接层 (`infra/save/`) | Phase-post-M1 | ✅ 完成 |
| C-T | Infra 层测试 | — | ✅ 完成 |
| C-R | 代码审查 | — | ✅ 完成 |

### 1.4 Phase D — Business Domains（✅ 全部 15 域完成）

| 域 | 规划编号 | 原始状态 | 实测文件数 | 实测行数 | 归档状态 |
|----|---------|---------|-----------|---------|---------|
| **Tactical** 战术 | D-1 | ✅ 已完成 | 32 | 2,344 | ✅ **已完成** |
| **Terrain** 地形 | D-2 | ✅ 已完成 | 22 | 1,108 | ✅ **已完成** |
| **Faction** 阵营 | D-3 | ✅ 已完成 | 18 | 1,097 | ✅ **已完成** |
| **Combat** 战斗 | D-5(原) | 🟡 骨架 | 27 | 2,326 | ✅ **已完成**（含 Turn 状态机） |
| **Narrative** 叙事 | D-13 | ✅ 已完成 | 16 | 902 | ✅ **已完成** |
| **CampRest** 营地 | — | 🟡 骨架 | 15 | 758 | ✅ **已完成** |
| **Crafting** 制造 | — | 🟡 骨架 | 16 | ~1,300 | ✅ **已完成** |
| **Economy** 经济 | D-14 | 🟡 骨架 | 16 | ~1,300 | ✅ **已完成** |
| **Inventory** 背包 | — | 🟡 骨架 | 18 | 1,559 | ✅ **已完成** |
| **Party** 队伍 | — | 🟡 骨架 | 14 | 805 | ✅ **已完成** |
| **Progression** 成长 | D-15 | 🟡 骨架 | 20 | 1,815 | ✅ **已完成** |
| **Quest** 任务 | D-12 | 🟡 骨架 | 16 | ~1,300 | ✅ **已完成** |
| **Reaction** 反应 | — | 🟡 骨架 | 8 | ~650 | ✅ **已完成** |
| **Spell** 法术 | — | 🟡 骨架 | 8 | ~650 | ✅ **已完成** |
| **Summon** 召唤 | — | 🟡 骨架 | 16 | ~1,300 | ✅ **已完成** |

### 1.5 其他任务

| # | 任务 | 规划来源 | 状态 |
|---|------|---------|------|
| M1 | 里程碑 "第一个端到端闭环" | Phase-C-D-execution-plan | ✅ 通过 |
| Capabilities 集成验证 | Tactical → Capabilities Observer 打通 | Phase-post-M1 | ✅ 完成 |
| Integration Facade | Tactical Anti-Corruption Layer 扩展 | integration-facade-plan | ✅ 完成 |
| 文档冲突修复 | 15 处全修复 | doc-conflict-evaluation | ✅ 完成 |
| 技术债扫描 | `docs/11-refactor/debt-inventory-2026-06-17.md` | Phase-post-M1 | ✅ 完成 |
| Per-domain 测试计划 | @test-guardian 补齐 | feature-developer-implementation-roadmap 0 | ⏳ 待执行 |
| 横切层 (Content/Tools/Modding) | Phase E~H | feature-developer-implementation-roadmap | 🔴 待定（核心域完成后） |
| 测试修复（4 个失败测试） | quest/spell/summon 域测试逻辑修正 | 2026-06-18 | ✅ 已完成（1396→1402 tests pass, 0 fail） |
| Replay Critical 修复 | 校验和验证静默丢弃 + total_commands 硬编码为 0 | 2026-06-18 | ✅ 已完成（1396 tests pass, 0 fail） |
| Batch 3+4 Code Review 修复 | M1-M4 + L1-L2 共 7 项整改 | 2026-06-18 | ✅ 已完成（621 warnings, 1402 pass） |
| Debt-005b cleanup | 删除真正废弃的 Dead Code（~13 处） | 2026-06-18 | ✅ 已完成 |
| Phase E ADR-047 | Content 加载管线架构设计 | 2026-06-18 | ✅ Approved |
| Phase E 加载模块骨架 | loading/（definition_type/discovery/ron_loader/errors） | 2026-06-18 | ✅ 已完成 |
| Phase E ContentPlugin | SpellDef Asset 注册 + RonAssetLoader + 同步加载 + 校验 + 存储（1402 tests, 0 fail） | 2026-06-18 | ✅ 已完成 |

---

## 2. 未完成任务清单（提炼自全部规划）

### 2.1 ✅ 已完成 — 骨架域填充（路径 B）

6 个业务域（crafting / economy / quest / reaction / spell / summon）已全部完成全量实现，包括完整的 components/rules/systems/events/errors + 单元/集成/不变量测试 + 代码审查。

> **注**：Progression、Inventory、Party、CampRest 已在之前轮次中完成全量实现，从骨架域列表中移除。

**参考标准**：一个完整的 Business Domain 应有 ~18-22 文件，含以下结构：

```
domains/<domain>/
├── mod.rs              # 模块声明 + pub use
├── plugin.rs           # Plugin impl（注册 Component/System/Event/Observer）
├── components.rs       # ECS Components（#[derive(Component, Reflect)]）
├── events.rs           # 领域事件定义
├── error.rs            # 领域错误枚举
├── resources.rs        # 全局 Resource（如有）
├── rules/              # 纯业务函数（零 ECS 依赖）
│   ├── mod.rs
│   ├── formulas.rs     # 计算公式
│   └── rules.rs        # 业务规则
├── systems/            # ECS Systems
│   ├── mod.rs
│   ├── <domain>_system.rs
│   └── ...
├── integration/        # Anti-Corruption Layer（如引用 Capabilities）
│   ├── mod.rs
│   └── <capability>/   # 按 Capability 拆分
│       ├── facade.rs
│       ├── types.rs
│       └── system_param.rs
└── tests/              # 四层测试
    ├── mod.rs
    ├── unit/
    ├── integration/
    ├── invariant/
    └── fixtures/
```

#### 填充顺序（按依赖关系排序）

| 顺序 | 域 | 前置依赖 | 预估工作量 | 说明 |
|------|-----|---------|-----------|------|
| **1** | **Spell** 法术 | Ability + Effect + Combat | ~500 行 | 法术位/专注/豁免/升环 |
| **2** | **Reaction** 反应 | Trigger + Event + Combat | ~400 行 | 机会攻击/法术反制/护盾/援护 |
| **3** | **Quest** 任务 | Event + Condition + Inventory | ~400 行 | 目标追踪/奖励/前置条件 |
| **4** | **Economy** 经济 | Event + Faction | ~350 行 | 货币/商店/价格/交易 |
| **5** | **Crafting** 制造 | Effect + Modifier + Inventory | ~400 行 | 配方/附魔/装备升级 |
| **6** | **Summon** 召唤 | Effect + Condition + Event + Combat | ~400 行 | 召唤物模板/专注绑定/消失 |

### 2.2 🟡 中优先级 — 补齐与增强

| # | 任务 | 说明 | 前置 |
|---|------|------|------|
| D-3 | Faction integration/ 层 | 当前 Faction 无 integration/，如需引用 Capabilities 需补齐 | — |
| D-9 | Combat integration/ 层 | Combat 已有 integration/（7 文件），可能需要按 Facade 模式标准化 | — |
| C-4 | Replay 测试 | C-4 Replay 桥接层的单元测试 ⏳ 待编写 | — | ✅ **已完成**（+4 端到端测试，29 total） |
| C-4 | Replay 代码审查 | Replay 桥接层审查 ⏳ 待执行 | C-4 测试通过 | ✅ **已完成**（修复 2 Critical bug） |
| B-4 | Capability 单元测试补齐 | 5 个关键 capability（attribute/modifier/aggregator/effect/ability）的 @test-guardian 测试 | — | ✅ **已完成** |
| 测试 | Per-domain 测试计划 | @test-guardian 为每个域输出测试计划文档 | 每个域填充完毕后 |

### 2.3 🔴 遗留架构缺口

| # | 缺口 | 说明 | 位置 | 状态 |
|---|------|------|------|------|
| GAP-1 | **空 plugin 警告** | 6 个 Capability 骨架 Plugin 的 `build()` 可能为空（Effect/Runtime/等），部分注册了空 observer 会被 Bevy 警告 | Phase B-2 标注 | ✅ 已确认为预留设计，非 Bug |
| GAP-2 | **integration/ 不完整** | 只有 Tactical 有完整 Facade，Faction/Terrain 有 `rules/` 但无 integration；Combat 有 integration/ 但无独立集成文档 | — | ✅ 已确认：Faction/Terrain 无 Capabilities 引用，不需要 integration/ |
| GAP-3 | **error.rs 缺失** | 除 Tactical 外，所有域都没有专属 error.rs | 每个域建立时补齐 | ✅ **已完成**：全部 15 个域均有 error.rs（combat/faction/narrative/terrain 为本轮新增） |
| GAP-4 | **Content 加载未就绪** | ContentPlugin 现注册 SpellDef Asset 类型/RonAssetLoader/同步加载校验/存储至 Resource | Phase E | ✅ 已完成（SpellDef 端到端：fireball.ron → 反序列化 → validate() → LoadedSpellDefs） |
| GAP-5 | **App 层渲染未就绪** | app_plugin.rs 在 Phase 9 注册，但当前无渲染/Bevy UI 系统 | Phase F | 🔴 待定（Phase F） |

### 2.4 🔴 远期（核心域完成后）

| 阶段 | 内容 | 前置条件 |
|------|------|---------|
| Phase E | Content 系统（配置加载 → 校验 → Registry 注册） | ✅ 已完成（SpellDef MVP 端到端），待扩展到其他 Def 类型 |
| Phase F | 渲染/UI 系统 | Tactical 稳定 + 网格渲染需求明确 |
| Phase G | 音频系统 | 渲染基础完成 |
| Phase H | Modding 系统 | Content 系统完成 |

---

## 3. 路径 B 执行计划 — 骨架域批量填充

### 3.1 执行原则

1. **自底向上**：按依赖关系逐个填充，Spell 优先（6 个骨架域按 3.3 分步执行表顺序）
2. **以 Terrain/Faction 为模板**：参照已完成的 ~18 文件结构，每个域约 400-500 行
3. **每个域自包含**：完成后 `cargo build` + `cargo test` 无回归
4. **角色协作**：@feature-developer 主编码，@test-guardian 补测试，@code-reviewer 审代码
5. **🟥 前置文档强制确认**：每个域编码前必须阅读对应的 ADR + 领域规则 + Schema

### 3.2 前置文档确认清单

每个域开始前，@feature-developer 必须逐项确认：

| 域 | ADR | 领域规则 (02-domain) | Schema (04-data) |
|----|-----|---------------------|-----------------|
| Progression | ADR-030 | `progression_domain.md` | `domains/progression_schema.md` |
| Inventory | ADR-030 | `inventory_domain.md` | `domains/inventory_schema.md` |
| Party | ADR-031 | `party_domain.md` | `domains/party_schema.md` |
| CampRest | ADR-031 | `camp_rest_domain.md` | `domains/camp_rest_schema.md` |
| Spell | ADR-023 | `spell_domain.md` | `domains/spell_schema.md` |
| Reaction | ADR-023 | `reaction_domain.md` | `domains/reaction_schema.md` |
| Quest | ADR-033 | `quest_domain.md` | `domains/quest_schema.md` |
| Economy | ADR-032 | `economy_domain.md` | `domains/economy_schema.md` |
| Crafting | ADR-032 | `crafting_domain.md` | `domains/crafting_schema.md` |
| Summon | ADR-033 | `summon_domain.md` | `domains/summon_schema.md` |

### 3.3 分步执行表

#### ✅ Batch 1 — 养成基础（Progression + Inventory）— 已完成

> Progression（1,815 行 / 20 文件）和 Inventory（1,559 行 / 18 文件）已在之前轮次中完成全量实现。本批次视为已完成。

#### ✅ Batch 2 — 队伍与休整（Party + CampRest）— 已完成

> Party（805 行 / 14 文件）和 CampRest（758 行 / 15 文件）已在之前轮次中完成全量实现。本批次视为已完成。

#### ✅ Batch 3 — 战斗扩展（Spell + Reaction）— 已完成

| 步骤 | 任务 | 预估 | 状态 |
|------|------|------|------|
| P3-1 | 前置文档确认（ADR-023 + spell/reaction domain + schema） | — | ✅ |
| P3-2 | Spell: 法术位/专注/豁免/升环 组件 + 系统 + 规则 | ~500 行 | ✅ |
| P3-3 | Reaction: 机会攻击/法术反制/护盾 组件 + 系统 + 规则 | ~400 行 | ✅ |
| P3-T | 测试 | @test-guardian | ✅ |
| P3-R | 代码审查 | — | ✅ 已完成（详见 `docs/10-reviews/code-review-batch3-4-2026-06-18.md`） |

#### ✅ Batch 4 — 叙事与经济（Quest + Economy + Crafting + Summon）— 已完成

| 步骤 | 任务 | 预估 | 状态 |
|------|------|------|------|
| P4-1 | 前置文档确认（ADR-032 + ADR-033 + 各 domain + schema） | — | ✅ |
| P4-2 | Quest: 目标追踪/奖励/前置条件 | ~400 行 | ✅ |
| P4-3 | Economy: 货币/商店/价格/交易 | ~350 行 | ✅ |
| P4-4 | Crafting: 配方/附魔/装备升级 | ~400 行 | ✅ |
| P4-5 | Summon: 召唤物模板/专注绑定/消失 | ~400 行 | ✅ |
| P4-T | 测试 | @test-guardian | ✅ |
| P4-R | 代码审查 | — | ✅ 已完成（详见 `docs/10-reviews/code-review-batch3-4-2026-06-18.md`） |

---

## 4. 标准域模板（供 @feature-developer 参考）

每个骨架域填充应遵守以下质量门禁：

### 4.1 标准文件清单

```
domains/<domain>/
├── mod.rs            ✅ 必须（已存在）
├── plugin.rs         ✅ 必须（已存在，但需要补充 build() 内容）
├── components.rs     🔴 必须新增
├── events.rs         🔴 必须新增（至少 2-3 个领域事件）
├── error.rs          🔴 必须新增（至少 4-6 个错误变体）
├── resources.rs      🟡 可选（如需要全局状态）
├── rules/            🔴 必须新增
│   ├── mod.rs
│   ├── formulas.rs   🟡 如有计算公式
│   └── rules.rs      🔴 至少 2-3 条业务规则纯函数
├── systems/          🔴 必须新增
│   ├── mod.rs
│   └── <domain>_system.rs
├── integration/      🟡 如引用 Capabilities（否则可省略）
└── tests/            🔴 必须（至少 unit + invariant）
```

### 4.2 质量门禁

```
□ cargo build 零错误
□ cargo test 新增测试全通过（无回归）
□ cargo clippy -- -D warnings 零警告
□ error.rs 存在且不为空（至少 4 个变体）
□ events.rs 存在至少 2 个领域事件
□ rules/ 至少 2 条纯函数业务规则
□ systems/ 至少 1 个 ECS System
□ 无 #[allow(...)] / as any / 类型逃逸
□ 符合 Feature First + 标准 7 文件结构
□ Plugin 注册顺序符合 CorePlugin 位置要求
□ 跨域通信仅通过 Event（无直接数据结构引用）
□ 遵守宪法红线（Effect/Modifier 管线不绕过等）
```

---

## 5. 工作量汇总

| 批次 | 内容 | 预估行数 | 编码轮次 | 前置依赖 | 状态 |
|------|------|---------|---------|---------|------|
| Batch 1 | Progression + Inventory | ~900 | 2-3 次 | ADR-030 + domain docs | ✅ **已完成** |
| Batch 2 | Party + CampRest | ~800 | 2 次 | ADR-031 + domain docs | ✅ **已完成** |
| Batch 3 | Spell + Reaction | ~900 | 2 次 | ADR-023 + domain docs | ✅ **已完成** |
| Batch 4 | Quest + Economy + Crafting + Summon | ~1,550 | 3-4 次 | ADR-032 + ADR-033 + domain docs | ✅ **已完成** |
| **已完成合计** | **10 个域** | **~12,000（全部 15 业务域中的 10 个）** | — | — | ✅ |
| **剩余合计** | **5 个域** | **~5,000（Capabilities infra 域 + 跨域集成）** | — | — | 🟡 |

---

## 6. 风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| Capabilities 系统存在未被发现的架构缺陷 | 中 | 高 | 每个新域都会自然验证 Capabilities；发现即调用 @architect |
| Spell/Reaction 依赖 Combat 但 Combat 未完成 | 低 | 高 | Combat 已实现 2,326 行含 Turn 状态机，但 Effect Pipeline 未全链路验证 |
| 单人编码瓶颈 | 中 | 中 | 剩余 6 个骨架域按顺序进行，暂无可避免 |
| Domain 规则文档与代码实现有偏差 | 低 | 中 | 严格前置文档确认流程 |
| 测试滞后 | 中 | 中 | 每个批次完成后 @test-guardian 补测试，不留债 |

---

## 7. 文件间的追踪关系

| 本计划引用 | 来源 | 内容 |
|-----------|------|------|
| 架构总纲 | `docs/01-architecture/README.md` | 纵向三层+横切四层、模块边界 |
| ADR-023 | `docs/01-architecture/20-tactical-combat/ADR-023-spell-reaction.md` | 法术与反应机制 |
| ADR-030 | `docs/01-architecture/30-progression-narrative/ADR-030-progression-inventory.md` | 成长与物品系统 |
| ADR-031 | `docs/01-architecture/30-progression-narrative/ADR-031-party-camp-rest.md` | 队伍与休整系统 |
| ADR-032 | `docs/01-architecture/30-progression-narrative/ADR-032-economy-crafting.md` | 经济与制造系统 |
| ADR-033 | `docs/01-architecture/30-progression-narrative/ADR-033-narrative-quest-summon.md` | 叙事/任务/召唤 |
| 领域规则 | `docs/02-domain/domains/` | 各 domain 规则文档 |
| 数据 Schema | `docs/04-data/domains/` | 各 domain Schema |
| 原路线图 | `feature-developer-implementation-roadmap.md` | Phase A-H 原始规划 |
| M1 计划 | `Phase-C-D-execution-plan.md` | Phase C+D 并行执行 |
| Post-M1 计划 | `Phase-post-M1-execution-plan.md` | M1 后任务跟踪 |
| Integration Facade | `integration-facade-plan.md` | Tactical ACL 计划 |
| 文档冲突修复 | `doc-conflict-evaluation.md` | 15 处冲突全部修复 |

---

*本文档合并了 `docs/09-planning/` 下全部 6 份规划文件的状态，与路径 B（骨架域填充路线图）整合而成。每完成一个 Batch，对应的域应更新为 ✅ 状态。*
