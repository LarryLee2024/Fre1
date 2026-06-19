---
id: 10-reviews.feature-developer-domains-alignment
title: Review — Business Domains 15 领域代码 vs 文档对齐分析
status: completed
owner: feature-developer
created: 2026-06-17
updated: 2026-06-17
tags:
  - review
  - domains
  - feature-developer
  - code-alignment
---

# Feature Developer 视角：Business Domains 15 领域代码与文档对齐分析

**Reviewer**: @feature-developer  
**Scope**: `src/core/domains/*/` vs `docs/02-domain/*_domain.md` + `docs/04-data/domains/*_schema.md`  
**Standards**: 架构文档 §6.2 Business Domains 标准 7 文件结构

---

## 总体状态

所有 15 个业务领域目前处于**骨架 (stub) 状态**。每个领域只有两个文件：

```
domains/<domain>/
├── mod.rs      # 标准模块头 + mod plugin + pub use
└── plugin.rs   # struct <Xxx>Plugin; impl Plugin { fn build(&self, _app: &mut App) { /* TODO */ } }
```

对比架构文档§6.2 定义的**标准 7 文件结构**：

```
domains/<domain>/
├── plugin.rs          # ✅ 存在
├── components.rs      # ❌ 缺失
├── systems/           # ❌ 缺失（含 mod.rs）
│   ├── mod.rs
│   ├── xxx_system.rs
│   └── yyy_system.rs
├── events.rs          # ❌ 缺失
├── error.rs           # ❌ 缺失
├── rules/             # ❌ 缺失
│   ├── formulas.rs
│   └── rules.rs
└── integration.rs     # ❌ 缺失
```

| 标准文件 | 实现率 | 说明 |
|---------|--------|------|
| `plugin.rs` | 100% (15/15) | 唯一实现的文件 |
| `components.rs` | 0% (0/15) | 全部缺失 |
| `systems/` | 0% (0/15) | 全部缺失 |
| `events.rs` | 0% (0/15) | 全部缺失 |
| `error.rs` | 0% (0/15) | 全部缺失 |
| `rules/` | 0% (0/15) | 全部缺失 |
| `integration.rs` | 0% (0/15) | 全部缺失 |

---

## 各领域分析与实现优先级建议

### ━━━━━━ Foundation Layer ━━━━━━

### 1. Tactical（战术空间）

| 维度 | 状态 |
|------|------|
| 领域文档 | `tactical_domain.md` — 完整 |
| 数据 Schema | `domains/tactical_schema.md` — 完整 |
| ADR 支持 | `ADR-022-grid-terrain-faction.md` |
| 代码状态 | `mod.rs` + `plugin.rs`（`// TODO`） |
| 实现建议 | **Phase C 优先** — 依赖 `shared/math` 网格工具、`infra/pipeline` 执行管线 |

**先决依赖**：
- `shared/math` — 网格坐标（已声明但未实现）
- `core/capabilities/tag` — 地形标签
- `infra/registry` — Tile 注册

### 2. Terrain（地形）

| 维度 | 状态 |
|------|------|
| 领域文档 | `terrain_domain.md` — 完整 |
| 数据 Schema | `domains/terrain_schema.md` — 完整 |
| ADR 支持 | `ADR-022-grid-terrain-faction.md` |
| 代码状态 | `mod.rs` + `plugin.rs`（`// TODO`） |
| 实现建议 | **Phase C** — 与 Tactical 紧密耦合 |

**注意**：Terrain 需要 `Effect` 管线来处理地形效果（毒池、冰面等），需确保 Capabilities 的 Effect 模块先就绪。

### 3. Faction（阵营关系）

| 维度 | 状态 |
|------|------|
| 领域文档 | `faction_domain.md` — 完整 |
| 数据 Schema | `domains/faction_schema.md` — 完整 |
| ADR 支持 | `ADR-022-grid-terrain-faction.md` |
| 代码状态 | `mod.rs` + `plugin.rs`（`// TODO`） |
| 实现建议 | **Phase C** — 依赖 Tag 系统 |

### ━━━━━━ Core Layer ━━━━━━

### 4. Combat（战斗）

| 维度 | 状态 |
|------|------|
| 领域文档 | `combat_domain.md` — 完整 |
| 数据 Schema | `domains/combat_schema.md` — 完整 |
| ADR 支持 | `ADR-020-combat-pipeline.md`、`ADR-021-turn-state-machine.md` |
| 代码状态 | `mod.rs` + `plugin.rs`（`// TODO`） |
| 实现建议 | **Phase C 高优先级** — 核心玩法入口 |

**先决依赖**（较多）：
- `capabilities/ability` — 技能激活
- `capabilities/execution` — 伤害计算
- `capabilities/effect` — 效果应用
- `capabilities/modifier` — 属性修改
- `infra/replay` — 战斗回放

### 5. Spell（法术）

| 维度 | 状态 |
|------|------|
| 领域文档 | `spell_domain.md` — 完整 |
| 数据 Schema | `domains/spell_schema.md` — 完整 |
| ADR 支持 | `ADR-023-spell-reaction.md` |
| 代码状态 | `mod.rs` + `plugin.rs`（`// TODO`） |
| 实现建议 | **Phase C** — 复用 Ability Pipeline |

### 6. Reaction（反应）

| 维度 | 状态 |
|------|------|
| 领域文档 | `reaction_domain.md` — 完整 |
| 数据 Schema | `domains/reaction_schema.md` — 完整 |
| ADR 支持 | `ADR-023-spell-reaction.md` |
| 代码状态 | `mod.rs` + `plugin.rs`（`// TODO`） |
| 实现建议 | **Phase C** — 依赖 Combat 管线 |

### 7. Progression（成长）

| 维度 | 状态 |
|------|------|
| 领域文档 | `progression_domain.md` — 完整 |
| 数据 Schema | `domains/progression_schema.md` — 完整 |
| ADR 支持 | `ADR-030-progression-inventory.md` |
| 代码状态 | `mod.rs` + `plugin.rs`（`// TODO`） |
| 实现建议 | **Phase D** — 依赖 Modifier Pipeline 完善 |

### 8. Inventory（背包）

| 维度 | 状态 |
|------|------|
| 领域文档 | `inventory_domain.md` — 完整 |
| 数据 Schema | `domains/inventory_schema.md` — 完整 |
| ADR 支持 | `ADR-030-progression-inventory.md` |
| 代码状态 | `mod.rs` + `plugin.rs`（`// TODO`） |
| 实现建议 | **Phase D** — 依赖 Modifier + Effect |

### 9. Party（队伍）

| 维度 | 状态 |
|------|------|
| 领域文档 | `party_domain.md` — 完整 |
| 数据 Schema | `domains/party_schema.md` — 完整 |
| ADR 支持 | `ADR-031-party-camp-rest.md` |
| 代码状态 | `mod.rs` + `plugin.rs`（`// TODO`） |
| 实现建议 | **Phase D** |

### 10. CampRest（营地/休息）

| 维度 | 状态 |
|------|------|
| 领域文档 | `camp_rest_domain.md` — 完整 |
| 数据 Schema | `domains/camp_rest_schema.md` — 完整 |
| ADR 支持 | `ADR-031-party-camp-rest.md` |
| 代码状态 | `mod.rs` + `plugin.rs`（`// TODO`） |
| 实现建议 | **Phase D** |

### ━━━━━━ Narrative Layer ━━━━━━

### 11. Narrative（叙事）

| 维度 | 状态 |
|------|------|
| 领域文档 | `narrative_domain.md` — 完整 |
| 数据 Schema | `domains/narrative_schema.md` — 完整 |
| ADR 支持 | `ADR-033-narrative-quest-summon.md` |
| 代码状态 | `mod.rs` + `plugin.rs`（`// TODO`） |
| 实现建议 | **Phase E** — 依赖 Event 系统 |

### 12. Quest（任务）

| 维度 | 状态 |
|------|------|
| 领域文档 | `quest_domain.md` — 完整 |
| 数据 Schema | `domains/quest_schema.md` — 完整 |
| ADR 支持 | `ADR-033-narrative-quest-summon.md` |
| 代码状态 | `mod.rs` + `plugin.rs`（`// TODO`） |
| 实现建议 | **Phase E** — 依赖 Event + Execution |

### ━━━━━━ Economy Layer ━━━━━━

### 13. Economy（经济）

| 维度 | 状态 |
|------|------|
| 领域文档 | `economy_domain.md` — 完整 |
| 数据 Schema | `domains/economy_schema.md` — 完整 |
| ADR 支持 | `ADR-032-economy-crafting.md` |
| 代码状态 | `mod.rs` + `plugin.rs`（`// TODO`） |
| 实现建议 | **Phase E** |

### 14. Crafting（制造）

| 维度 | 状态 |
|------|------|
| 领域文档 | `crafting_domain.md` — 完整 |
| 数据 Schema | `domains/crafting_schema.md` — 完整 |
| ADR 支持 | `ADR-032-economy-crafting.md` |
| 代码状态 | `mod.rs` + `plugin.rs`（`// TODO`） |
| 实现建议 | **Phase E** |

### 15. Summon（召唤）

| 维度 | 状态 |
|------|------|
| 领域文档 | `summon_domain.md` — 完整 |
| 数据 Schema | `domains/summon_schema.md` — 完整 |
| ADR 支持 | `ADR-033-narrative-quest-summon.md` |
| 代码状态 | `mod.rs` + `plugin.rs`（`// TODO`） |
| 实现建议 | **Phase E** — 依赖 Ability + Effect |

---

## 关键发现

### 🟢 正确做法

1. **目录结构已建立**：所有 15 个 domain 目录存在且名字与文档一致
2. **Plugin 已注册**：所有 domain 的 Plugin 已在 `core_plugin.rs` 中按 Phase 5–7 顺序注册
3. **模块注释引用文档**：每个 `mod.rs` 都标注了对应的 `docs/02-domain/` 文件

### 🟡 待解决

1. **所有 domain 文件数量严重不足**：架构要求 7 文件标准，当前只有 2 文件，缺失 `components.rs`、`systems/`、`events.rs`、`error.rs`、`rules/`、`integration.rs`
2. **Plugin 注册但无实质内容**：所有 domain plugin 的 `build()` 为空，注册不会产生任何运行时效果
3. **缺少 `rules/` 目录**：这是 Domain 层最核心的部分 — 纯业务规则（纯函数，零 ECS 依赖）。当前所有业务规则只存在于 `docs/02-domain/` 文档中
4. **缺少 `integration.rs`**：这是 Domain 调用 Capabilities 的唯一入口，架构明确要求

### 🟥 红线风险

1. **所有 plugin.rs 的 TODO 不符合规范**：`// TODO: register components, systems, events` 缺少 `[P0-P3][领域][日期]` 格式，违反红线 #38
2. **Domain 模块结构不完整**：架构文档 §6.2 定义的标准结构未执行，可能影响后续 code review 通过

---

## 实现路线图

```
Phase C (高优) ─── 依赖 capabilities 管线完善后
  ├── tactical/     ── components, systems, events
  ├── terrain/      ── components, systems, events
  ├── faction/      ── components, systems, events
  ├── combat/       ── components, systems, events, rules
  ├── spell/        ── components, systems, events, rules
  └── reaction/     ── components, systems, events, rules

Phase D (中优) ─── 依赖 modifier/effect 管线完善后
  ├── progression/  ── components, systems, rules
  ├── inventory/    ── components, systems, rules
  ├── party/        ── components, systems
  └── camp_rest/    ── components, systems

Phase E (低优) ─── 依赖 event 系统和内容系统就绪后
  ├── narrative/    ── components, systems
  ├── quest/        ── components, systems, rules
  ├── economy/      ── components, systems, rules
  ├── crafting/     ── components, systems, rules
  └── summon/       ── components, systems
```

### 每个 domain 的第一批实现应为

1. `plugin.rs` — 更新 build() 注册 Component + Event + System
2. `components.rs` — 核心 ECS Component（参照 tag 的 TagSet 风格）
3. `events.rs` — 对外发布的领域事件
4. `error.rs` — 专属错误枚举
5. `rules/formulas.rs` + `rules/rules.rs` — 纯函数业务规则（从领域文档直接翻译）

---

*本报告由 @feature-developer 基于 `src/core/domains/*/` 源码与 `docs/02-domain/*_{domain}.md` 对齐性分析生成。*
