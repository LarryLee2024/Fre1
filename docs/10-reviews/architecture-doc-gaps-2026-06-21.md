---
id: architecture.gap-analysis.2026-06-21
title: 架构文档差距分析 — 5个 Tier S Agent 输出完整性评估
status: proposed
owner: architect
created: 2026-06-21
tags:
  - gap-analysis
  - documentation
  - governance
  - agent
---

# 架构文档差距分析（2026-06-21）

> **分析角色**: @architect（系统集成者）
> **分析范围**: 5个 Tier S Agent 输出目录 + 代码现状对齐
> **版本**: 1.0

本文档系统评估 Fre SRPG 项目 Tier S 架构委员会各角色的文档输出完整性和质量。对每个文档差距给出具体文件路径、行号，并指定应由哪个 Agent 填补。

---

## 目录

1. [新 shared/ 模块零设计文档（代码已有，文档为零）](#1-新-shared-模块零设计文档代码已有文档为零)
2. [ADR 索引与附录 B 缺失项](#2-adr-索引与附录-b-缺失项)
3. [文档头部与实际状态不一致](#3-文档头部与实际状态不一致)
4. [TBD/占位内容](#4-tbd占位内容)
5. [交叉引用断裂（引用的文档不存在）](#5-交叉引用断裂引用的文档不存在)
6. [docs/04-data/ 状态表遗漏项](#6-docs04-data-状态表遗漏项)
7. [docs/03-content/ 状态表遗漏项](#7-docs03-content-状态表遗漏项)
8. [docs/06-ui/ 文档成熟度状态](#8-docs06-ui-文档成熟度状态)
9. [综合建议与优先级排序](#9-综合建议与优先级排序)

---

## 1. 新 shared/ 模块零设计文档（代码已有，文档为零）

以下模块已在 `src/shared/` 中实现完整的 Rust 代码和单元测试，但**没有任何对应的设计文档**——无领域规则（`docs/02-domain/`）、无数据 Schema（`docs/04-data/`）、无内容 Def（`docs/03-content/`）。

尽管 shared 层按架构约定允许无独立设计文档，但某些模块包含**业务语义相关**的类型，建议在 domain 规则文档中覆盖。

### 1.1 shared/collections/ — 通用集合扩展

| 项目 | 详情 |
|------|------|
| **路径** | `src/shared/collections/mod.rs`（第 1-18 行） |
| **代码文件** | `group_by_map.rs`, `partition_map.rs`, `take_while_inclusive.rs` |
| **核心类型** | `GroupByMap`, `PartitionMap`, `TakeWhileInclusive`, `TakeWhileInclusiveExt` |
| **领域文档** | 无（`docs/02-domain/` 未覆盖） |
| **数据 Schema** | 无（`docs/04-data/` 未覆盖） |
| **内容 Def** | 无（`docs/03-content/` 未覆盖） |
| **建议** | 零业务语义的纯集合工具，**无需**设计文档。架构 README §3.1 已正确标记为 `—`。 |

### 1.2 shared/hashing/ — 非加密高速哈希

| 项目 | 详情 |
|------|------|
| **路径** | `src/shared/hashing/mod.rs`（第 1-231 行） |
| **核心类型** | `FastHasher`, `FastBuildHasher`, `fast_hash()`, `new_fast_hashmap()`, `new_fast_hashset()` |
| **领域文档** | 无 |
| **数据 Schema** | 无 |
| **建议** | 零业务语义的基础设施工具，**无需**设计文档。 |

### 1.3 shared/math/ — 纯数学工具

| 项目 | 详情 |
|------|------|
| **路径** | `src/shared/math/mod.rs`（第 1-53 行）；子文件 `hex.rs`, `interpolation.rs` |
| **核心类型** | `HexCoord`, `hex_distance()`, `FloatEq` trait, `lerp()`, `inv_lerp()`, `smoothstep()` |
| **领域文档** | 无 |
| **数据 Schema** | 无 |
| **评估** | **HexCoord 是 gameplay-significant 类型**，直接影响 tactical/spell 领域的范围计算。建议在 `docs/02-domain/domains/tactical_domain.md` 中添加六边形坐标系规则。 |
| **建议负责 Agent** | @domain-designer |

### 1.4 shared/validation/ — 链式校验工具

| 项目 | 详情 |
|------|------|
| **路径** | `src/shared/validation/mod.rs`（第 1-603 行） |
| **核心类型** | `ValidationResult<T>`, `ValidationError`, `ValidationChain<T>`, `Validator<T>` trait, `NotEmpty`, `Range<T>`, `MinLength` |
| **领域文档** | 无 |
| **数据 Schema** | 无 |
| **建议** | 零业务语义的纯校验工具，但已在 `src/infra/replay/tests/` 中使用验证。**建议**在 `docs/03-content/` 中引用此模块作为 Validation Pipeline 的底层工具。 |

### 1.5 shared/path/ — 路径工具

| 项目 | 详情 |
|------|------|
| **路径** | `src/shared/path/mod.rs`（第 1-198 行） |
| **核心类型** | `ProjectDirs`, `ensure_dir()`, `asset_path()`, `config_path()` |
| **领域文档** | 无 |
| **数据 Schema** | 无 |
| **建议** | 零业务语义的工具，**无需**设计文档。 |

### 1.6 shared/prelude/ — 统一预导入

| 项目 | 详情 |
|------|------|
| **路径** | `src/shared/prelude/mod.rs`（第 1-27 行） |
| **内容** | 重新导出 `constants`, `diagnostics`（Domain, LogCode, ObservableEvent, DomainEvent）, `LocalizationKey`, `DeterministicRng`, `GameTime`, `RuleFailure` |
| **领域文档** | 无 |
| **建议** | **中等优先级**。prelude 的内容定义了 shared 层的"公共面部"，建议在 `docs/01-architecture/README.md` §3.1 的 shared 表格中添加 prelude 行。 |

### 1.7 shared/constants/ — 全局常量

| 项目 | 详情 |
|------|------|
| **路径** | `src/shared/constants/mod.rs`（第 1-23 行） |
| **常量** | `MAX_OBSERVER_DEPTH=10`, `MAX_PARTY_SIZE=6`, `MAX_INVENTORY_SIZE=100`, `MAX_BUFF_STACK=5` |
| **领域文档** | 无 |
| **建议** | constants 是跨领域决策值，应在对应的 domain 规则文档中记录每个常量的理由。例如 `MAX_OBSERVER_DEPTH` 引用 ADR-002，`MAX_PARTY_SIZE` 应在 `docs/02-domain/domains/party_domain.md` 中论证。 |
| **建议负责 Agent** | @domain-designer（在各自领域文档中添加常量理由） |

### 1.8 shared/shared_plugin.rs — Shared Plugin

| 项目 | 详情 |
|------|------|
| **路径** | `src/shared/shared_plugin.rs`（第 1-23 行） |
| **功能** | 注册 `GameTime` Resource、`DeterministicRng` Resource、`PreUpdate` 阶段的 `advance_game_time` 系统 |
| **架构文档** | 无独立文档；`docs/01-architecture/README.md` §6.1 提到 `SharedPlugin` |
| **建议** | 已有架构 README 覆盖。**无需**额外文档。 |

---

## 2. ADR 索引与附录 B 缺失项

**文档**: `docs/01-architecture/README.md`
**位置**: §9 架构决策索引（第 642-681 行）+ 附录 B 文件状态追踪（第 726-762 行）

### 2.1 ADR 索引（§9）缺失项

以下 ADR 文件存在于磁盘，但**未出现在 §9 索引表中**：

| 缺失 ADR | 磁盘路径 | 文件存在 | 索引中存在 |
|----------|---------|---------|-----------|
| **ADR-048** | `docs/01-architecture/40-cross-cutting/ADR-048-replay-combat-bridge.md` | 是 | **否** ❌ |
| **ADR-049** | `docs/01-architecture/40-cross-cutting/ADR-049-shared-cross-domain-events.md` | 是 | **否** ❌ |
| **ADR-052** | `docs/01-architecture/40-cross-cutting/ADR-052-logging-architecture.md` | 是 | **否** ❌ |

索引表目前从 ADR-047 跳到 ADR-050（第 667→668 行），又从 ADR-051 跳到 ADR-053（第 669→670 行）。

### 2.2 附录 B（文件状态表）缺失项

以下文件存在于磁盘但**未出现在附录 B**：

| 缺失文件 | 磁盘路径 | 文件存在 | 附录 B 中存在 |
|----------|---------|---------|-------------|
| **ADR-046** | `docs/01-architecture/00-foundation/ADR-046-module-interface-pattern.md` | 是 | **否** ❌ |
| **ADR-048** | `docs/01-architecture/40-cross-cutting/ADR-048-replay-combat-bridge.md` | 是 | **否** ❌ |
| **ADR-052** | `docs/01-architecture/40-cross-cutting/ADR-052-logging-architecture.md` | 是 | **否** ❌ |

附录 B 目前从 ADR-045（第 747 行）跳到 ADR-047（第 748 行），缺少 ADR-046。

### 2.3 附录 B 与索引状态不一致

| ADR | §9 索引状态 | 附录 B 状态 | 文件 frontmatter 状态 |
|-----|-----------|-----------|----------------------|
| ADR-045 | ✅ Accepted | ✅ stable | Accepted |
| ADR-046 | ✅ Accepted | ❌ 缺失 | （未知） |
| ADR-050 | ✅ Accepted | ✅ accepted | （未知） |

**建议修复**: 将 ADR-048、ADR-049、ADR-052 加入 §9 索引；将 ADR-046、ADR-048、ADR-052 加入附录 B；统一状态字段。

**建议负责 Agent**: @architect

---

## 3. 文档头部与实际状态不一致

### 3.1 ADR-056: frontmatter vs. body 状态矛盾

| 项目 | 详情 |
|------|------|
| **文件** | `docs/01-architecture/00-foundation/ADR-056-agent-governance.md` |
| **Frontmatter** | 第 4 行: `status: accepted` |
| **Body** | 第 20 行: `## 状态` → `提议中`（Proposed） |
| **类型** | **严重不一致** — 两个状态值互相矛盾 |
| **建议** | 统一为 `accepted`（因为 ADR-056 已在 §9 索引中标为 Accepted，且在 AGENTS.md 和 CLAUDE.md 中已有体现） |
| **建议负责 Agent** | @architect |

### 3.2 docs/04-data/ 中 logging_schema.md 创建日期为未来

| 项目 | 详情 |
|------|------|
| **文件** | `docs/04-data/README.md` |
| **行号** | 第 564 行 |
| **问题** | `infrastructure/logging_schema.md` → `完成日期: 2026-06-25`（当前日期为 2026-06-21，此日期在未来） |
| **类型** | 日期错误 |
| **建议** | 将日期修正为实际完成日期 |
| **建议负责 Agent** | @data-architect |

---

## 4. TBD/占位内容

### 4.1 docs/04-data/foundation/migration_policy.md — 完整占位符

| 项目 | 详情 |
|------|------|
| **文件** | `docs/04-data/foundation/migration_policy.md` |
| **状态行** | `docs/04-data/README.md` 第 543 行: `⬜ pending` |
| **完成度** | 仅第 1-3 节有极少量内容。第 1 节 Domain Ownership、第 2 节 Problem、第 3 节 Migration Strategy、第 4 节 Versioning Scheme、第 5 节 Rollback Policy、第 6 节 Testing Requirements 全部为 "TBD" |
| **文件内声明** | 第 52 行: "本文档是占位骨架，完整内容待 @data-architect 完成设计后填充" |
| **严重性** | 中等。migration policy 是 Save/Replay 系统的关键组件。 |
| **建议负责 Agent** | @data-architect |

### 4.2 docs/03-content/definitions/ability-def.md — 待完成

| 项目 | 详情 |
|------|------|
| **文件** | `docs/03-content/definitions/ability-def.md` |
| **状态** | `docs/03-content/README.md` 第 393 行: `🟡 TODO` |
| **对应 area** | L1 Capability, AbilityDef + SpellDef |
| **内容** | 标题标记为 TODO，内容未完成 |
| **严重性** | **高**。AbilityDef 是核心 Def 类型，影响所有技能/法术的实现。 |
| **建议负责 Agent** | @content-architect |

### 4.3 docs/03-content/ — L4 World Def 定义缺失

| 项目 | 详情 |
|------|------|
| **位置** | `docs/03-content/README.md` 第 314 行 |
| **问题** | L4 World 的 Def 定义为 "TBD" |
| **范围** | MapDef, RegionDef, SceneDef, CutsceneDef, NarrativeArcDef, StoryFlagDef, CompanionDef 共 7 种 Def 类型未定义 |
| **严重性** | 低（当前开发阶段聚焦 L0-L3，L4 叙事世界层不是当前重点） |
| **建议负责 Agent** | @content-architect（远期） |

---

## 5. 交叉引用断裂（引用的文档不存在）

### 5.1 replay_domain.md

| 项目 | 详情 |
|------|------|
| **引用位置** | `docs/01-architecture/40-cross-cutting/ADR-048-replay-combat-bridge.md` 第 23 行 |
| **引用内容** | `docs/02-domain/domains/replay_domain.md — 回放领域规则` |
| **括号说明** | 同一行注明："注：当前无此文档" |
| **现状** | 文件 `docs/02-domain/domains/replay_domain.md` **不存在**，且未出现在 `docs/02-domain/README.md` 的 31 个文件列表中 |
| **影响** | 回放系统的领域规则无文档化来源。当前回放规则分布在 ADR-041（回放确定性）和 ADR-048（桥接层）中。 |
| **建议** | 创建 `docs/02-domain/domains/replay_domain.md`，覆盖：ReplayFrame 格式、回放确定性边界、录制/回放协议、桥接层契约 |
| **建议负责 Agent** | @domain-designer |

### 5.2 event_history_architecture.md

| 项目 | 详情 |
|------|------|
| **引用位置** | `docs/01-architecture/40-cross-cutting/ADR-049-shared-cross-domain-events.md` 第 111 行 |
| **引用内容** | `docs/04-data/foundation/event_history_architecture.md` |
| **现状** | 文件 **不存在**。`docs/04-data/foundation/` 目录中仅包含 5 个文件：`id_strategy.md`, `id-taxonomy.md`, `migration_policy.md`, `replay_architecture.md`, `save_architecture.md` |
| **影响** | Event History 架构（ADR-059 定义）缺少独立的数据架构文档。数据 Schema、持久化策略、Replay 兼容性分析未正式记录。 |
| **建议** | 创建 `docs/04-data/foundation/event_history_architecture.md`，覆盖 Event History Store 的 Schema、分层归属、兼容性策略 |
| **建议负责 Agent** | @data-architect |

---

## 6. docs/04-data/ 状态表遗漏项

**文档**: `docs/04-data/README.md`
**位置**: 第 9 节 文件状态追踪（第 536-581 行）

### 6.1 遗漏文件

以下文件存在于磁盘但**不在状态表中**：

| 遗漏文件 | 路径 | 本应出现位置 |
|---------|------|------------|
| `capabilities/status_category_schema.md` | `docs/04-data/capabilities/` | 第 558-559 行之间（cue_schema 之后, ui-presentation-schema 之前） |
| `domains/element_schema.md` | `docs/04-data/domains/` | 第 568-569 行之间（terrain_schema 和 faction_schema 之间） |

### 6.2 其他遗漏

| 遗漏项 | 路径 | 说明 |
|-------|------|------|
| `domains/README.md` | `docs/04-data/domains/README.md` | 此索引文件存在（第 567-581 行区域有文件名，但状态表中无 README 条目其本身） |

**建议负责 Agent**: @data-architect

---

## 7. docs/03-content/ 状态表遗漏项

### 7.1 文件状态整体完整性

`docs/03-content/README.md` §8 文件状态表（第 372-409 行）整体组织良好。主要差距：

| 状态 | 文件 | 行号 |
|------|------|------|
| 🟡 TODO | `definitions/ability-def.md` | 第 393 行 |
| TBD | L4 World definitions | 第 314 行 |

**建议负责 Agent**: @content-architect

---

## 8. docs/06-ui/ 文档成熟度状态

**文档**: `docs/06-ui/README.md`
**位置**: §6 文件状态（第 218-231 行）

### 8.1 整体状态

UI 文档目录包含 11 个文档文件 + 1 个 README，全部标记为 `🟡 draft`。这是正常的——UI 架构是新建立的（2026-06-20），draft 状态合理。

| 文件 | 状态 | 行号 |
|------|------|------|
| `README.md` | 🟡 in_progress | 第 219 行 |
| `01-architecture/architecture.md` | 🟡 draft | 第 220 行 |
| `01-architecture/application-layer.md` | 🟡 draft | 第 221 行 |
| `01-architecture/implementation-patterns.md` | 🟡 draft（未在状态表中） | — |
| `02-design-system/widget-atoms.md` | 🟡 draft | 第 222 行 |
| `02-design-system/widget-composites.md` | 🟡 draft | 第 223 行 |
| `02-design-system/theme-localization.md` | 🟡 draft | 第 224 行 |
| `02-design-system/focus-binding.md` | 🟡 draft | 第 225 行 |
| `03-screens/screen-lifecycle.md` | 🟡 draft | 第 226 行 |
| `03-screens/screens.md` | 🟡 draft | 第 227 行 |
| `03-screens/navigation-overlay.md` | 🟡 draft | 第 228 行 |
| `03-screens/overlays.md` | 🟡 draft | 第 229 行 |
| `04-data-flow/projection-viewmodel.md` | 🟡 draft | 第 230 行 |
| `05-testing/testing.md` | 🟡 draft | 第 231 行 |

### 8.2 发现：06-ui 状态表遗漏 `implementation-patterns.md`

| 遗漏项 | 路径 | 说明 |
|-------|------|------|
| `01-architecture/implementation-patterns.md` | `docs/06-ui/01-architecture/` | 文件存在于目录（第 73 行 README 列出），但 §6 状态表中无此文件条目 |

**建议负责 Agent**: @presentation-architect

---

## 9. 综合建议与优先级排序

### 9.1 优先级矩阵

| 优先级 | 差距描述 | 负责 Agent | 类型 | 状态 |
|--------|---------|-----------|------|------|
| **P0** | ADR-056 frontmatter/body 状态矛盾 | @architect | 一致性 | ✅ 已修复 |
| **P0** | ADR 索引缺失 ADR-048/049/052 | @architect | 完整性 | ✅ 已修复 |
| **P0** | Appendix B 缺失 ADR-046/048/052 | @architect | 完整性 | ✅ 已修复 |
| **P1** | `ability-def.md` 为 TODO | @content-architect | 内容缺失 | ✅ 已修复 |
| **P1** | `replay_domain.md` 不存在 | @domain-designer | 缺失文档 | ✅ 已创建 |
| **P1** | `event_history_architecture.md` 不存在 | @data-architect | 缺失文档 | ✅ 已创建 |
| **P2** | `migration_policy.md` 为 TBD 骨架 | @data-architect | 内容缺失 | ✅ 已填充 |
| **P2** | shared/math/ HexCoord 无领域规则 | @domain-designer | 缺覆盖 | ✅ 已补充 |
| **P2** | docs/04-data/ 状态表遗漏 3 个文件 | @data-architect | 状态缺失 | ✅ 已修复 |
| **P2** | docs/06-ui/ 状态表遗漏 implementation-patterns.md | @presentation-architect | 状态缺失 | ✅ 已修复 |
| **P2** | logging_schema.md 日期为未来 | @data-architect | 数据错误 | ✅ 已修复 |
| **P2** | docs/08-knowledge/random-overview.md 全部陈旧 | @architect | 内容陈旧 | ✅ 已重写 |
| **P3** | L4 World Def 定义为 TBD | @content-architect | 远期内容 | ⏳ 远期 |
| **P3** | shared/constants/ 常量理由无文档 | @domain-designer | 改进项 | ⏳ 远期 |


### 9.2 建议修复顺序

```
立即（P0）
├── @architect: 修复 ADR-056 状态字段
├── @architect: 将 ADR-048/049/052 加入 §9 索引
└── @architect: 将 ADR-046/048/052 加入附录 B

优先（P1）
├── @content-architect: 完成 ability-def.md
├── @domain-designer: 创建 replay_domain.md
└── @data-architect: 创建 event_history_architecture.md

标准（P2）
├── @data-architect: 填充 migration_policy.md
├── @domain-designer: 在 tactical_domain.md 中覆盖 HexCoord
├── @data-architect: 更新 docs/04-data/README.md 状态表
├── @presentation-architect: 更新 docs/06-ui/README.md 状态表
└── @data-architect: 修复 logging_schema.md 日期

远期（P3）
├── @content-architect: L4 World Def 定义
└── @domain-designer: 在 domain 规则中记录常量理由
```

### 9.3 文件修改清单

| 文件 | 需修改内容 | 建议 Agent | 优先级 |
|------|-----------|-----------|--------|
| `docs/01-architecture/00-foundation/ADR-056-agent-governance.md` | 第 20 行 `提议中` → `Accepted` | @architect | P0 |
| `docs/01-architecture/README.md` | 第 667-670 行添加 ADR-048/049/052 索引行 | @architect | P0 |
| `docs/01-architecture/README.md` | 第 747-749 行添加 ADR-046/048/052 附录 B 行 | @architect | P0 |
| `docs/02-domain/domains/replay_domain.md` | 创建新文件（回放领域规则） | @domain-designer | P1 |
| `docs/04-data/foundation/event_history_architecture.md` | 创建新文件（Event History 数据架构） | @data-architect | P1 |
| `docs/03-content/definitions/ability-def.md` | 完成 AbilityDef/SpellDef 定义 | @content-architect | P1 |
| `docs/04-data/foundation/migration_policy.md` | 填充所有 TBD 节 | @data-architect | P2 |
| `docs/02-domain/domains/tactical_domain.md` | 添加 HexCoord 六边形坐标系规则 | @domain-designer | P2 |
| `docs/04-data/README.md` | 添加 `status_category_schema.md` + `element_schema.md` + `domains/README.md` 到状态表 | @data-architect | P2 |
| `docs/04-data/README.md` | 第 564 行修正 logging_schema.md 完成日期 | @data-architect | P2 |
| `docs/06-ui/README.md` | 第 219-231 行添加 `implementation-patterns.md` 到状态表 | @presentation-architect | P2 |
| `docs/03-content/README.md` | L4 World 栏位从 TBD 改为计划时间 | @content-architect | P3 |

---

## 附录 A：检测方法

本文档通过以下方法系统检测差距：

1. **文件系统扫描**：对比每个 Agent 输出目录的磁盘内容与 README 状态表
2. **代码扫描**：检查 `src/shared/` 的模块列表，与设计文档覆盖范围比对
3. **交叉引用验证**：跟踪每个 ADR 中引用的外部文档是否存在
4. **Frontmatter 验证**：检查文档 frontmatter 的 `status` 字段与 body 中声明的一致性
5. **索引完整性**：验证 `docs/01-architecture/README.md` §9 索引 + 附录 B 是否列出所有磁盘上的 ADR 文件

## 附录 B：测试不足

本文档不涵盖测试代码的差距分析。`docs/05-testing/` 的完整性评估应由 @test-guardian 完成。此处的差距分析仅限 **设计文档**（docs/ 目录）。

---

*本文档由 @architect 根据 2026-06-21 代码状态产出。建议执行顺序：P0 → P1 → P2 → P3。每个差距修复后应更新此文档的对应状态。*
