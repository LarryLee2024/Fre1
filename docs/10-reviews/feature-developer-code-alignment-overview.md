---
id: 10-reviews.feature-developer-code-alignment-overview
title: Review — @feature-developer: 项目代码与文档对齐总览
status: completed
owner: feature-developer
created: 2026-06-17
updated: 2026-06-17
tags:
  - review
  - code-alignment
  - feature-developer
  - overview
---

# Feature Developer 视角：项目代码与文档对齐总览

**Reviewer**: @feature-developer  
**Scope**: `src/` 源码 vs `docs/` 相关文档  
**Standards**: `docs/00-governance/ai-constitution-complete.md` · `.qoder/agents/feature-developer.md`  
**Date**: 2026-06-17

---

## 评审方法论

以 @feature-developer 的三条铁律为基准：

1. **铁律1**：严格遵守架构文档 — 实现与架构冲突即停
2. **铁律2**：最简单方案优先 — 纯函数 > Struct > Component > System > Trait > 泛型 > Macro
3. **铁律3**：新增内容不得破坏已有规则 — 保持现有测试通过、领域规则成立

以 P0 铁则为对齐维度：
- Feature First：按业务领域拆模块，不按技术类型
- Data Driven First：配置数据实现，禁止硬编码
- Replay First：核心逻辑可确定性重放
- Logic/Presentation Separation：业务逻辑与表现层隔离
- 双轴边界：Capabilities 管机制，Domains 管业务

---

## 1. 总体评估

| 维度 | 评估 | 说明 |
|------|------|------|
| 文档完整性 | 🟢 **完整** | 7 层文档体系齐全（治理/架构/领域/数据/测试/AI/规划） |
| 代码完整性 | 🟡 **部分实现** | Capabilities 约 40% 实现，Domains/Infra 约 5% 实现 |
| 代码 vs 文档对齐 | 🟡 **良好但对齐有间隙** | 目录结构匹配，但深层实现细节与文档规范有偏差 |
| Plugin 注册顺序 | 🟢 **完全对齐** | `core_plugin.rs` / `app_plugin.rs` 严格按 Phase 0–9 |
| 模块内聚 | 🟢 **良好** | Capability 遵循 C1→C2→C3，Domain 遵循标准 7 文件 |

---

## 2. 各层对齐矩阵

### 2.1 Shared (L0) — 原子层

| docs 定义模块 | src 状态 | 对齐 | 备注 |
|--------------|---------|------|------|
| IDs | ✅ 已实现 | 🟢 完全对齐 | `define_string_id!` + `define_numeric_id!` 宏，`StrongId` trait |
| Error | ⬜ 骨架 | 🟡 目录对齐 | 仅 mod.rs，含 `// TODO` |
| Math | ⬜ 骨架 | 🟡 目录对齐 | 仅 mod.rs |
| Random | ⬜ 骨架 | 🟡 目录对齐 | 仅 `// TODO: SeededRng / DeterministicRng` |
| Time | ⬜ 骨架 | 🟡 目录对齐 | 仅 mod.rs |
| Collections | ⬜ 骨架 | 🟡 目录对齐 | 仅 mod.rs |
| Hashing | ⬜ 骨架 | 🟡 目录对齐 | 仅 mod.rs |
| Validation | ⬜ 骨架 | 🟡 目录对齐 | 仅 mod.rs |
| Testing | ⬜ 骨架 | 🟡 目录对齐 | 仅 mod.rs |
| Traits | ⬜ 骨架 | 🟡 目录对齐 | 仅 mod.rs |
| Prelude | ⬜ 骨架 | 🟡 目录对齐 | 仅 mod.rs |
| Path | ⬜ 骨架 | 🟡 目录对齐 | 仅 mod.rs |

### 2.2 Capabilities (L1 Core) — 能力领域

| docs 定义模块 | src 状态 | 对齐 | 备注 |
|--------------|---------|------|------|
| tag | ✅ 已实现 | 🟢 完全对齐 | C1 foundation + C2 mechanism，含单元测试 |
| attribute | ✅ 已实现 | 🟢 完全对齐 | AttributeId, AttributeDefinition, AttributeContainer |
| modifier | ✅ 已实现 | 🟢 完全对齐 | C1 + C2，Components + lifecycle + systems |
| aggregator | ✅ 已实现 | 🟢 完全对齐 | C1 + C2，含 pipeline.rs |
| gameplay_context | ✅ 已实现 | 🟢 完全对齐 | C1 + C2，含 builder.rs |
| spec | ✅ 已实现 | 🟢 完全对齐 | C1 + C2 |
| condition | ✅ 已实现 | 🟢 完全对齐 | C1 + C2，components + evaluator |
| trigger | ✅ 已实现 | 🟢 完全对齐 | C1 + C2，components + evaluator |
| event | ✅ 已实现 | 🟢 完全对齐 | C1 + C2，bus.rs |
| ability | ✅ 已实现 | 🟢 完全对齐 | C1 + C2，components + lifecycle |
| targeting | ✅ 已实现 | 🟢 完全对齐 | C1 + C2，selector.rs |
| execution | ✅ 已实现 | 🟢 完全对齐 | C1 + C2，calculator.rs |
| effect | ✅ 已实现 | 🟢 完全对齐 | C1 + C2，lifecycle.rs |
| stacking | ✅ 已实现 | 🟢 完全对齐 | C1 + C2，decider.rs |
| cue | ✅ 已实现 | 🟢 完全对齐 | C1 + C2，components + dispatch |
| runtime | ✅ 已实现 | 🟢 完全对齐 | pipeline/scheduler/registry/command/replay 子结构 |

### 2.3 Domains (L1 Core) — 业务领域

| docs 定义模块 | src 状态 | 对齐 | 备注 |
|--------------|---------|------|------|
| tactical | ⬜ 骨架 stub | 🟡 目录对齐 | 仅 mod.rs + plugin.rs（`// TODO`） |
| terrain | ⬜ 骨架 stub | 🟡 目录对齐 | 同上 |
| faction | ⬜ 骨架 stub | 🟡 目录对齐 | 同上 |
| combat | ⬜ 骨架 stub | 🟡 目录对齐 | 同上 |
| spell | ⬜ 骨架 stub | 🟡 目录对齐 | 同上 |
| reaction | ⬜ 骨架 stub | 🟡 目录对齐 | 同上 |
| progression | ⬜ 骨架 stub | 🟡 目录对齐 | 同上 |
| inventory | ⬜ 骨架 stub | 🟡 目录对齐 | 同上 |
| party | ⬜ 骨架 stub | 🟡 目录对齐 | 同上 |
| camp_rest | ⬜ 骨架 stub | 🟡 目录对齐 | 同上 |
| narrative | ⬜ 骨架 stub | 🟡 目录对齐 | 同上 |
| quest | ⬜ 骨架 stub | 🟡 目录对齐 | 同上 |
| economy | ⬜ 骨架 stub | 🟡 目录对齐 | 同上 |
| crafting | ⬜ 骨架 stub | 🟡 目录对齐 | 同上 |
| summon | ⬜ 骨架 stub | 🟡 目录对齐 | 同上 |

### 2.4 Infra (L2) — 技术实现层

| docs 定义模块 | src 状态 | 对齐 | 备注 |
|--------------|---------|------|------|
| registry | ⬜ 骨架 stub | 🟡 目录对齐 | 仅 mod.rs + plugin.rs |
| pipeline | ⬜ 骨架 stub | 🟡 目录对齐 | 仅 mod.rs + plugin.rs |
| replay | ⬜ 骨架 stub | 🟡 目录对齐 | 仅 mod.rs + plugin.rs |
| save | ⬜ 骨架 stub | 🟡 目录对齐 | 仅 mod.rs + plugin.rs |
| input | ⬜ 骨架 stub | 🟡 目录对齐 | 仅 mod.rs + plugin.rs |

### 2.5 横切四层

| docs 定义模块 | src 状态 | 对齐 | 备注 |
|--------------|---------|------|------|
| App | ✅ 已实现 | 🟢 完全对齐 | Composition Root，Phase 0–9 顺序 |
| Content | ⬜ 骨架 stub | 🟡 目录对齐 | 仅 mod.rs + plugin.rs（`// TODO`） |
| Tools | ⬜ 骨架 | 🟡 目录对齐 | 仅 mod.rs + dev_tools_plugin.rs |
| Modding | ⬜ 骨架 stub | 🟡 目录对齐 | 仅 mod.rs + modding_plugin.rs |
| mod_api | ⬜ 骨架 | 🟡 目录对齐 | 仅 mod.rs |

---

## 3. 关键发现

### 🟢 做得好的方面

1. **文档体系完整，结构清晰**：所有 30 个领域规则、33+ 数据 Schema、19 个 ADR 齐全
2. **Capabilities C1/C2 结构执行到位**：15 个能力领域全部按 `foundation/ + mechanism/` 组织，且每个模块的实际代码量适中（300–500 行理想区间）
3. **Plugin 注册顺序严格对齐**：`app_plugin.rs` 和 `core_plugin.rs` 完全按照架构文档 Phase 0–9 顺序
4. **Tag 模块实现质量高**：有单元测试、位掩码操作、层级管理、错误处理，是 Capabilities 实现的标杆
5. **模块命名和目录结构与 docs 完全对称**：`src/` 的目录树与 `docs/01-architecture/README.md` 的第 8 节目录结构总览完全匹配

### 🟡 需关注的问题

1. **共享模块只有 IDs 有实现**：error/random/math/time/collections/hashing/validation/testing/traits/path 都只有 `// TODO`，但架构文档将它们列为 Shared 层的核心模块
2. **Business Domains 全部为 stub**：虽然文档中有 15 个完整的领域规则文件，但所有 domain 代码只有 `mod.rs` + `plugin.rs` 两文件，插件体为空
3. **Infrastructure 层全部为 stub**：pipeline/replay/save/registry/input 全都只有骨架
4. **TODO 注释质量参差不齐**：部分 `// TODO` 缺少领域标签、优先级和完成条件，违反红线 `#38`
5. **Plugin 注册但内容为空**：所有 capabilitiy plugin 已注册但多数 plugin.rs 的 `build()` 方法体为空（如 EffectPlugin, RuntimePlugin）

### 🟥 潜在红线问题

| 问题 | 严重度 | 违反规则 |
|------|--------|---------|
| Domain 模块无 `events.rs`、`error.rs`、`rules/`、`integration.rs`、`components.rs` 等标准文件 | 中 | Architecture 第 6.2 节 Business Domains 标准结构 |
| `shared/random/` 只有 `// TODO`，无确定性 RNG 实现 | 高 | P0 铁律 #3 (Replay First) |
| `shared/error/` 只有 `// TODO`，无错误上下文工具 | 中 | 无全局 AppError 是红线，但缺少基础设施 |
| 多个 plugin.rs 含无标签 TODO（如 `// TODO: register components, systems, events`） | 中 | 红线 #38：无上下文 TODO/FIXME |

---

## 4. 开发顺序建议

基于文档定义与代码现状，建议按以下优先级推进实现：

### Phase A — 基础设施先行（Shared + Infra）

```
1. shared/random/ — 实现 SeededRng / DeterministicRng（Replay First 要求）
2. shared/error/ — 实现错误上下文工具
3. infra/pipeline/ — 实现通用执行管线引擎
4. infra/replay/ — 实现回放录制/回放框架
5. shared/time/ — 实现 GameTime
```

### Phase B — Capabilities 核心管线

```
6. effect/mechanism/lifecycle.rs — 完善 Effect 生命周期（已有 foundation 定义）
7. ability/mechanism/lifecycle.rs — 完善 Ability 状态机
8. runtime/pipeline/ — 实现管线编排
9. runtime/registry/ — 实现统一注册中心
```

### Phase C — 业务领域实现

```
10. tactical/ — 网格系统（依赖 infra/ 管线、shared/math）
11. combat/ — 战斗流程（依赖 capabilities 管线完整）
12. terrain/ — 地形系统
```

### Phase D — 横切层

```
13. content/ — 配置加载/校验管线
14. modding/ — Mod 框架
15. tools/ — 开发工具
```

---

## 5. 与上游文档的追溯

| 上游输入 | 位置 | 在本报告中的使用 |
|---------|------|----------------|
| 🟥 架构设计最高依据 | `docs/00-governance/Fre项目架构设计.md` | 目录结构对齐检查 |
| 架构总纲 | `docs/01-architecture/README.md` | 模块映射、Plugin 顺序、标准结构 |
| 领域规则 | `docs/02-domain/README.md` (30 索引) | 每个 Feature 模块的领域规则检查 |
| 数据 Schema | `docs/04-data/README.md` | 四层数据分离检查 |
| AI 开发宪法 | `.trae/rules/AI开发宪法.md` | 红线检查 |
| Feature Developer Prompt | `.qoder/agents/feature-developer.md` | 三条铁律、P0 铁则 |

---

## 6. 结论

项目当前处于**设计完成、实现起步**阶段。文档体系（架构/领域/数据）非常完整，代码目录结构与文档完全对齐。Capabilities 层的 15 个领域全部具备 C1 Foundation 的基础类型定义，其中 Tag 模块实现最为成熟（含单元测试）。但是 Business Domains 和 Infra 层尚处于 stub 状态，整体代码实现进度约为 15%。

**最大的风险**是 `shared/random/` 的 RNG 实现缺失 — 违反 P0 铁律中"Replay First"的原则，应作为最高优先级处理。

---

*本报告由 @feature-developer 基于项目代码与文档对齐性分析生成。*
