---
id: 09-planning.consolidated-execution-plan
title: Consolidated Execution Plan — 全规划整合 + 骨架域填充路线图
status: active
owner: feature-developer
created: 2026-06-18
updated: 2026-06-18
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

## 0. 实测代码基线（2026-06-18）

| 层 | 范畴 | 文件数 | 行数 | 状态 |
|----|------|--------|------|------|
| **Shared (L0)** | ids / random / time / error / testing | 30 | ~1,410 | ✅ 完成 |
| **Capabilities (L1)** | 15 机制域 + runtime | 199 | ~17,026 | ✅ Foundation+Mechanism+Events 完整 |
| **Domains (L1)** — 已实现 | tactical / combat / terrain / faction / narrative | 115 | ~7,777 | ✅ 完整实现 |
| **Domains (L1)** — 骨架 | camp_rest / crafting / economy / inventory / party / progression / quest / reaction / spell / summon | 60 | ~160 | 🟡 骨架（仅 plugin.rs + mod.rs） |
| **Infra (L2)** | pipeline / registry / replay / save / input | 53 | ~2,874 | ✅ 桥接层完成 |
| **横切层** | app / content / tools / modding | 6 | ~120 | 🟡 骨架 |
| **测试** | 各域 tests/ | — | — | ✅ 914 tests pass |

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
| B-4 | Capability 单元测试 | feature-developer-implementation-roadmap | ⚠️ 部分完成 |

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

### 1.4 Phase D — Business Domains（部分完成）

| 域 | 规划编号 | 原始状态 | 实测文件数 | 实测行数 | 归档状态 |
|----|---------|---------|-----------|---------|---------|
| **Tactical** 战术 | D-1 | ✅ 已完成 | 32 | 2,344 | ✅ **已完成** |
| **Terrain** 地形 | D-2 | ✅ 已完成 | 22 | 1,108 | ✅ **已完成** |
| **Faction** 阵营 | D-3 | ✅ 已完成 | 18 | 1,097 | ✅ **已完成** |
| **Combat** 战斗 | D-5(原) | 🟡 骨架 | 27 | 2,326 | ✅ **已完成**（含 Turn 状态机） |
| **Narrative** 叙事 | D-13 | ✅ 已完成 | 16 | 902 | ✅ **已完成** |
| **CampRest** 营地 | — | 🟡 骨架 | 6 | 16 | 🟡 **待填充** |
| **Crafting** 制造 | — | 🟡 骨架 | 6 | 16 | 🟡 **待填充** |
| **Economy** 经济 | D-14 | 🟡 骨架 | 6 | 16 | 🟡 **待填充** |
| **Inventory** 背包 | — | 🟡 骨架 | 6 | 16 | 🟡 **待填充** |
| **Party** 队伍 | — | 🟡 骨架 | 6 | 16 | 🟡 **待填充** |
| **Progression** 成长 | D-15 | 🟡 骨架 | 6 | 16 | 🟡 **待填充** |
| **Quest** 任务 | D-12 | 🟡 骨架 | 6 | 16 | 🟡 **待填充** |
| **Reaction** 反应 | — | 🟡 骨架 | 6 | 16 | 🟡 **待填充** |
| **Spell** 法术 | — | 🟡 骨架 | 6 | 16 | 🟡 **待填充** |
| **Summon** 召唤 | — | 🟡 骨架 | 6 | 16 | 🟡 **待填充** |

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

---

## 2. 未完成任务清单（提炼自全部规划）

### 2.1 🔴 高优先级 — 骨架域填充（路径 B）

10 个业务域当前只有 `mod.rs` + `plugin.rs`（各约 16 行），需要按照已完成的 **Terrain** 或 **Faction** 为标准模板填充。

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
| **1** | **Progression** 成长 | Attribute + Event | ~500 行 | 经验/等级/职业/天赋/ASI，这是养成系统的基础 |
| **2** | **Inventory** 背包 | Condition + Effect + Modifier + Event | ~500 行 | 物品/装备槽位/消耗品/战利品 |
| **3** | **Party** 队伍 | Event + Modifier | ~400 行 | 成员名册/羁绊/阵型/换人 |
| **4** | **CampRest** 营地 | Event + Effect + Combat | ~400 行 | 短休/长休/生命骰/营地事件 |
| **5** | **Spell** 法术 | Ability + Effect + Combat | ~500 行 | 法术位/专注/豁免/升环 |
| **6** | **Reaction** 反应 | Trigger + Event + Combat | ~400 行 | 机会攻击/法术反制/护盾/援护 |
| **7** | **Quest** 任务 | Event + Condition + Inventory | ~400 行 | 目标追踪/奖励/前置条件 |
| **8** | **Economy** 经济 | Event + Faction | ~350 行 | 货币/商店/价格/交易 |
| **9** | **Crafting** 制造 | Effect + Modifier + Inventory | ~400 行 | 配方/附魔/装备升级 |
| **10** | **Summon** 召唤 | Effect + Condition + Event + Combat | ~400 行 | 召唤物模板/专注绑定/消失 |

### 2.2 🟡 中优先级 — 补齐与增强

| # | 任务 | 说明 | 前置 |
|---|------|------|------|
| D-3 | Faction integration/ 层 | 当前 Faction 无 integration/，如需引用 Capabilities 需补齐 | — |
| D-9 | Combat integration/ 层 | Combat 已有 integration/（7 文件），可能需要按 Facade 模式标准化 | — |
| C-4 | Replay 测试 | C-4 Replay 桥接层的单元测试 ⏳ 待编写 | — |
| C-4 | Replay 代码审查 | Replay 桥接层审查 ⏳ 待执行 | C-4 测试通过 |
| B-4 | Capability 单元测试补齐 | 5 个关键 capability（attribute/modifier/aggregator/effect/ability）的 @test-guardian 测试 | — |
| 测试 | Per-domain 测试计划 | @test-guardian 为每个域输出测试计划文档 | 每个域填充完毕后 |

### 2.3 🔴 遗留架构缺口

| # | 缺口 | 说明 | 位置 |
|---|------|------|------|
| GAP-1 | **空 plugin 警告** | 6 个 Capability 骨架 Plugin 的 `build()` 可能为空（Effect/Runtime/等），部分注册了空 observer 会被 Bevy 警告 | Phase B-2 标注 |
| GAP-2 | **integration/ 不完整** | 只有 Tactical 有完整 Facade，Faction/Terrain 有 `rules/` 但无 integration；Combat 有 integration/ 但无独立集成文档 | — |
| GAP-3 | **error.rs 缺失** | 除 Tactical 外，所有域都没有专属 error.rs | 每个域建立时补齐 |
| GAP-4 | **Content 加载未就绪** | ContentPlugin 为空，配置数据无法从 asset 加载到 Registry | Phase E |
| GAP-5 | **App 层渲染未就绪** | app_plugin.rs 在 Phase 9 注册，但当前无渲染/Bevy UI 系统 | Phase F |

### 2.4 🔴 远期（核心域完成后）

| 阶段 | 内容 | 前置条件 |
|------|------|---------|
| Phase E | Content 系统（配置加载 → 校验 → Registry 注册） | 全部 15 域骨架填充完成 + Registry 已就绪 |
| Phase F | 渲染/UI 系统 | Tactical 稳定 + 网格渲染需求明确 |
| Phase G | 音频系统 | 渲染基础完成 |
| Phase H | Modding 系统 | Content 系统完成 |

---

## 3. 路径 B 执行计划 — 骨架域批量填充

### 3.1 执行原则

1. **自底向上**：按依赖关系逐个填充，Progression 优先（被 Inventory/Party/Quest 依赖）
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

#### Batch 1 — 养成基础（Progression + Inventory）

| 步骤 | 任务 | 输出 | 预估 |
|------|------|------|------|
| P1-1 | 前置文档确认（ADR-030 + progression/inventory domain + schema） | — | — |
| P1-2 | Progression: `components.rs` — 经验/等级/职业/天赋/ASI 组件 | ~80 行 | 1 次 |
| P1-3 | Progression: `events.rs` — LeveledUp, ExperienceGained, ClassChanged | ~40 行 | 同上 |
| P1-4 | Progression: `error.rs` — ProgressionError 枚举 | ~30 行 | 同上 |
| P1-5 | Progression: `rules/formulas.rs` — 升级经验公式、属性成长 | ~80 行 | 同上 |
| P1-6 | Progression: `rules/rules.rs` — 职业解锁、天赋激活规则 | ~60 行 | 同上 |
| P1-7 | Progression: `systems/` — 经验结算、等级晋升 system | ~80 行 | 同上 |
| P1-8 | Progression: `plugin.rs` 更新 | ~30 行 | 同上 |
| P1-9 | Progression: 四层测试（unit/integration/invariant/fixtures） | @test-guardian | 后续 |
| P1-10 | **重复以上步骤 1-9 于 Inventory 域**（物品/装备/消耗品） | ~400 行 | 2 次 |
| P1-R | 代码审查 | @code-reviewer | — |

#### Batch 2 — 队伍与休整（Party + CampRest）

| 步骤 | 任务 | 预估 |
|------|------|------|
| P2-1 | 前置文档确认（ADR-031 + party/camp_rest domain + schema） | — |
| P2-2 | Party: components + events + error + rules + systems + plugin | ~400 行 |
| P2-3 | CampRest: components + events + error + rules + systems + plugin | ~400 行 |
| P2-T | 测试 | @test-guardian |
| P2-R | 代码审查 | @code-reviewer |

#### Batch 3 — 战斗扩展（Spell + Reaction）

| 步骤 | 任务 | 预估 |
|------|------|------|
| P3-1 | 前置文档确认（ADR-023 + spell/reaction domain + schema） | — |
| P3-2 | Spell: 法术位/专注/豁免/升环 组件 + 系统 + 规则 | ~500 行 |
| P3-3 | Reaction: 机会攻击/法术反制/护盾 组件 + 系统 + 规则 | ~400 行 |
| P3-T | 测试 | @test-guardian |
| P3-R | 代码审查 | @code-reviewer |

#### Batch 4 — 叙事与经济（Quest + Economy + Crafting + Summon）

| 步骤 | 任务 | 预估 |
|------|------|------|
| P4-1 | 前置文档确认（ADR-032 + ADR-033 + 各 domain + schema） | — |
| P4-2 | Quest: 目标追踪/奖励/前置条件 | ~400 行 |
| P4-3 | Economy: 货币/商店/价格/交易 | ~350 行 |
| P4-4 | Crafting: 配方/附魔/装备升级 | ~400 行 |
| P4-5 | Summon: 召唤物模板/专注绑定/消失 | ~400 行 |
| P4-T | 测试 | @test-guardian |
| P4-R | 代码审查 | @code-reviewer |

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

| 批次 | 内容 | 预估行数 | 编码轮次 | 前置依赖 |
|------|------|---------|---------|---------|
| Batch 1 | Progression + Inventory | ~900 | 2-3 次 | ADR-030 + domain docs |
| Batch 2 | Party + CampRest | ~800 | 2 次 | ADR-031 + domain docs |
| Batch 3 | Spell + Reaction | ~900 | 2 次 | ADR-023 + domain docs |
| Batch 4 | Quest + Economy + Crafting + Summon | ~1,550 | 3-4 次 | ADR-032 + ADR-033 + domain docs |
| **总计** | **10 个域** | **~4,150** | **~10 次编码** | **全部前置文档就绪** |

---

## 6. 风险与缓解

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| Capabilities 系统存在未被发现的架构缺陷 | 中 | 高 | 每个新域都会自然验证 Capabilities；发现即调用 @architect |
| Spell/Reaction 依赖 Combat 但 Combat 未完成 | 低 | 高 | Combat 已实现 2,326 行含 Turn 状态机，但 Effect Pipeline 未全链路验证 |
| 单人编码瓶颈 | 高 | 高 | 10 个域按顺序进行，暂无可避免 |
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
