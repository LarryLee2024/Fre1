# Cross-Reference Audit Report

**Date**: 2026-06-19  
**Scope**: `docs/` ↔ `src/` 文件引用、代码引用、模块声明  
**Tools**: CodeGraph, Repomix, rg (ripgrep), glob, Python  

---

## Summary

| 严重度 | 数量 | 状态 | 说明 |
|--------|------|------|------|
| CRITICAL | 43文件 | ✅ FIXED | 源码 doc comment 引用路径错误（capability domain docs） |
| HIGH | 1 | ✅ FIXED | 源码引用了错误位置的架构文档 |
| MEDIUM | 3处 | ✅ FIXED | 文档间交叉引用指向已迁移的旧路径 |
| LOW | 0 | PASS | 无断裂 mod 声明 |
| INFO | 2 | BY DESIGN | Content/Infra 层引用 domains（设计如此） |

> 注：MEDIUM 原始 60+ 处引用中，绝大部分位于 `docs/ai_ignore_this_dir/`（按 AGENTS.md 规则视为不存在），无需修复。

---

## CRITICAL: 源码 Doc Comment 路径错误（17处）

### 问题描述

`src/core/capabilities/` 下 15 个 capability 模块的 `mod.rs` 和子文件中，doc comment 引用的路径格式为：
```
docs/02-domain/xxx_domain.md
```
但实际文件位于：
```
docs/02-domain/capabilities/xxx_domain.md
```

### 受影响文件

| 文件 | 行号 | 引用路径 | 正确路径 |
|------|------|----------|----------|
| `src/core/capabilities/ability/mod.rs` | 15 | `docs/02-domain/ability_domain.md` | `docs/02-domain/capabilities/ability_domain.md` |
| `src/core/capabilities/aggregator/mod.rs` | 6 | `docs/02-domain/aggregator_domain.md` | `docs/02-domain/capabilities/aggregator_domain.md` |
| `src/core/capabilities/attribute/mod.rs` | 3 | `docs/02-domain/attribute_domain.md` | `docs/02-domain/capabilities/attribute_domain.md` |
| `src/core/capabilities/condition/mod.rs` | 11 | `docs/02-domain/condition_domain.md` | `docs/02-domain/capabilities/condition_domain.md` |
| `src/core/capabilities/cue/mod.rs` | 8 | `docs/02-domain/cue_domain.md` | `docs/02-domain/capabilities/cue_domain.md` |
| `src/core/capabilities/effect/mod.rs` | 16 | `docs/02-domain/effect_domain.md` | `docs/02-domain/capabilities/effect_domain.md` |
| `src/core/capabilities/effect/events.rs` | 6 | `docs/02-domain/effect_domain.md §6` | `docs/02-domain/capabilities/effect_domain.md` |
| `src/core/capabilities/effect/foundation/types.rs` | 5,12 | `docs/02-domain/effect_domain.md` | `docs/02-domain/capabilities/effect_domain.md` |
| `src/core/capabilities/effect/mechanism/lifecycle.rs` | 4,82 | `docs/02-domain/effect_domain.md` | `docs/02-domain/capabilities/effect_domain.md` |
| `src/core/capabilities/effect/plugin.rs` | 18 | `docs/02-domain/effect_domain.md` | `docs/02-domain/capabilities/effect_domain.md` |
| `src/core/capabilities/event/mod.rs` | 11 | `docs/02-domain/event_domain.md` | `docs/02-domain/capabilities/event_domain.md` |
| `src/core/capabilities/event/events.rs` | 6 | `docs/02-domain/event_domain.md §6` | `docs/02-domain/capabilities/event_domain.md` |
| `src/core/capabilities/event/mechanism/bus.rs` | 6 | `docs/02-domain/event_domain.md §5` | `docs/02-domain/capabilities/event_domain.md` |
| `src/core/capabilities/execution/mod.rs` | 16 | `docs/02-domain/execution_domain.md` | `docs/02-domain/capabilities/execution_domain.md` |
| `src/core/capabilities/execution/events.rs` | 6 | `docs/02-domain/execution_domain.md §6` | `docs/02-domain/capabilities/execution_domain.md` |
| `src/core/capabilities/execution/foundation/types.rs` | 5 | `docs/02-domain/execution_domain.md` | `docs/02-domain/capabilities/execution_domain.md` |
| `src/core/capabilities/execution/mechanism/calculator.rs` | 4,23,57,140 | `docs/02-domain/execution_domain.md` | `docs/02-domain/capabilities/execution_domain.md` |
| `src/core/capabilities/gameplay_context/mod.rs` | 6 | `docs/02-domain/gameplay_context_domain.md` | `docs/02-domain/capabilities/gameplay_context_domain.md` |
| `src/core/capabilities/modifier/mod.rs` | 3 | `docs/02-domain/modifier_domain.md` | `docs/02-domain/capabilities/modifier_domain.md` |
| `src/core/capabilities/spec/mod.rs` | 12 | `docs/02-domain/spec_domain.md` | `docs/02-domain/capabilities/spec_domain.md` |
| `src/core/capabilities/spec/events.rs` | 6 | `docs/02-domain/spec_domain.md §6` | `docs/02-domain/capabilities/spec_domain.md` |
| `src/core/capabilities/spec/mechanism/lifecycle.rs` | 6 | `docs/02-domain/spec_domain.md §5` | `docs/02-domain/capabilities/spec_domain.md` |
| `src/core/capabilities/stacking/mod.rs` | 7 | `docs/02-domain/stacking_domain.md` | `docs/02-domain/capabilities/stacking_domain.md` |
| `src/core/capabilities/stacking/events.rs` | 4 | `docs/02-domain/stacking_domain.md §6` | `docs/02-domain/capabilities/stacking_domain.md` |
| `src/core/capabilities/stacking/foundation/types.rs` | 5 | `docs/02-domain/stacking_domain.md` | `docs/02-domain/capabilities/stacking_domain.md` |
| `src/core/capabilities/stacking/mechanism/decider.rs` | 6,52,66 | `docs/02-domain/stacking_domain.md` | `docs/02-domain/capabilities/stacking_domain.md` |
| `src/core/capabilities/tag/mod.rs` | 3 | `docs/02-domain/tag_domain.md` | `docs/02-domain/capabilities/tag_domain.md` |
| `src/core/capabilities/targeting/mod.rs` | 15 | `docs/02-domain/targeting_domain.md` | `docs/02-domain/capabilities/targeting_domain.md` |
| `src/core/capabilities/targeting/events.rs` | 6 | `docs/02-domain/targeting_domain.md §6` | `docs/02-domain/capabilities/targeting_domain.md` |
| `src/core/capabilities/targeting/foundation/types.rs` | 5 | `docs/02-domain/targeting_domain.md` | `docs/02-domain/capabilities/targeting_domain.md` |
| `src/core/capabilities/targeting/mechanism/selector.rs` | 4,98 | `docs/02-domain/targeting_domain.md` | `docs/02-domain/capabilities/targeting_domain.md` |
| `src/core/capabilities/trigger/mod.rs` | 11 | `docs/02-domain/trigger_domain.md` | `docs/02-domain/capabilities/trigger_domain.md` |
| `src/core/capabilities/trigger/events.rs` | 6 | `docs/02-domain/trigger_domain.md §6` | `docs/02-domain/capabilities/trigger_domain.md` |

### 影响

这些路径错误导致开发者/审查者在 IDE 中点击 doc comment 链接时跳转失败，AI Agent 读取 doc comment 时获取不到正确文档。

### 修复方案

批量替换 `docs/02-domain/xxx_domain.md` → `docs/02-domain/capabilities/xxx_domain.md`，覆盖上述 15 个 capability 目录。

---

## HIGH: 架构文档路径错误（1处）

| 文件 | 行号 | 引用路径 | 正确路径 |
|------|------|----------|----------|
| `src/core/mod_api/mod.rs` | 6 | `docs/00-governance/Fre项目架构设计.md` | `docs/01-architecture/Fre项目架构设计.md` |

架构设计文档位于 `docs/01-architecture/` 而非 `docs/00-governance/`。

---

## MEDIUM: 文档间交叉引用旧路径（60+处）

`docs/01-architecture/` 下的大量文档引用了迁移前的扁平路径结构。例如：

### 旧路径 → 新路径（典型示例）

| 旧引用 | 新位置 |
|--------|--------|
| `docs/01-architecture/00-overview/*.md` | 已不存在（内容合并到 README.md） |
| `docs/01-architecture/01-battle-gas/*.md` | 已不存在 |
| `docs/01-architecture/02-ecs-patterns/*.md` | 已不存在 |
| `docs/01-architecture/03-data-config-asset/*.md` | 已不存在 |
| `docs/01-architecture/04-events-logging-error/*.md` | 已不存在 |
| `docs/01-architecture/05-ui/*.md` | 已不存在 |
| `docs/01-architecture/06-map-pathfinding/*.md` | 已不存在 |
| `docs/01-architecture/07-tools-testing-quality/*.md` | 已不存在 |
| `docs/01-architecture/08-i18n-modding-collaboration/*.md` | 已不存在 |
| `docs/01-architecture/09-infrastructure-migration/*.md` | 已不存在 |
| `docs/01-architecture/20-tactical-combat/ADR-021-combat-domain.md` | 已不存在 |
| `docs/01-architecture/ADR-000-feature-module-map.md` | `docs/01-architecture/00-foundation/ADR-000-feature-module-map.md` |

### 受影响文档（部分列表）

- `docs/01-architecture/README.md` — 多处引用旧子目录路径
- `docs/00-governance/ai-constitution-complete.md` — 引用旧架构子文档
- `docs/02-domain/README.md` — 引用旧 domain 规则文件
- `docs/05-testing/test-spec.md` — 引用旧架构文件
- `docs/10-reviews/done/*.md` — 早期审查报告引用旧路径

### 说明

这些是项目重构过程中文档迁移遗留的引用未更新问题。文档内容已迁移到新位置（ADR 子目录、capabilities/domains 子目录），但文档间的交叉引用未同步更新。

---

## INFO: 跨层引用（设计如此）

以下引用属于**正常架构设计**，不是违规：

| 文件 | 引用 | 说明 |
|------|------|------|
| `src/content/content_plugin.rs` | `core::domains::{camp_rest,crafting,economy,party,progression,quest,spell,summon}::*Def` | Content 层加载 Definition 是设计职责 |
| `src/content/hot_reload.rs` | 同上 | 热重载需要访问 Def 类型 |
| `src/content/def_impls.rs` | 同上 | DefinitionType trait 实现在 Content 层 |
| `src/infra/save/save_system.rs` | `core::domains::{combat,party,progression}::*` | Save 系统序列化需要访问运行时类型 |
| `src/infra/save/load_system.rs` | 同上 | Load 系统反序列化需要访问运行时类型 |

根据架构文档，Content 层是 Core↔Content 的桥接层，Infra 层是技术实现层，它们引用 Core domains 类型是正确的依赖方向。

---

## PASS: 模块声明完整性

**所有 mod 声明均匹配实际文件/目录**，无断裂。

验证覆盖：`src/lib.rs`, `src/core/mod.rs`, `src/core/domains/mod.rs`, `src/core/capabilities/mod.rs`, `src/core/capabilities/runtime/mod.rs`, `src/infra/mod.rs`, `src/shared/mod.rs`, `src/content/mod.rs`

---

## 修复优先级建议

1. **CRITICAL** (立即修复): 源码 doc comment 中 17 处 `docs/02-domain/xxx_domain.md` → `docs/02-domain/capabilities/xxx_domain.md`
2. **HIGH** (本周修复): `src/core/mod_api/mod.rs` 中架构文档路径
3. **MEDIUM** (排期修复): 文档间 60+ 处旧路径引用（批量搜索替换）
