---
id: 11-refactor.debt-inventory-2026-06-17
title: "技术债清单 — 首次全量扫描"
status: active
scanner: refactor-guardian
created: 2026-06-17
updated: 2026-06-17 (P0 修复完成)
scan_scope: src/ (full codebase)
baseline_warnings: 433
---

# 技术债清单 — 首次全量扫描

> 扫描范围: `src/` 全量
> 扫描时间: 2026-06-17
> 基线: `cargo build` 产生 433 个 warning（dead_code 400 + unused_import 31 + other 2）
> 参照: `docs/00-governance/ai-constitution-complete.md` v5.0
> 验证: `cargo test --lib` 742 passed, 0 failed

---

## 扫描总览

| 类别 | 数量 | 严重程度 | 状态 |
|------|------|---------|------|
| **Debt-001~003** 可见性超标 (ADR-045) | 9 处 | **High** | ✅ 已修复 |
| **Debt-004** 未使用的 `pub use` 重导出 | 31 个模块 | Low | ❌ 预留 |
| **Debt-005a** Capabilities 预留 Dead Code | ~350 处 | Low | ❌ 预期行为 |
| **Debt-005b** 真正废弃的 Dead Code | ~50 处 | Medium | ❌ 待处理 |
| ~~Debt-006~~ components.rs / systems.rs | 11 处 | ~~N/A~~ | ~~不构成违规~~（已移除） |

---

## Debt-001: 可见性超标 — infra 层 `pub mod` 应为 `pub(crate)`

- **严重程度**: High
- **位置**:
  - `src/infra/pipeline/mod.rs:21` — `pub mod hooks`
  - `src/infra/input/mod.rs:10,12,13` — `pub mod action`, `pub mod resources`, `pub mod systems`
- **问题描述**: infra 层内部模块使用 `pub mod` 暴露为公共 API，违反 ADR-045 最小可见性原则。宪法 §7 明确要求"每个 Feature 只暴露必要的公共接口，所有内部实现必须设为私有"。
- **影响**: infra 层边界腐化，外部代码可直接访问内部子模块，AI 误用风险高。
- **建议修复**: `pub mod hooks` → `pub(crate) mod hooks`；`pub mod action/resources/systems` → `pub(crate) mod`。通过 `pub use plugin::*` 的 re-export 已足够对外暴露所需 API。

---

## Debt-002: 可见性超标 — domains/tactical 内部模块

- **严重程度**: High
- **位置**:
  - `src/core/domains/tactical/mod.rs:15` — `pub mod events`
  - `src/core/domains/tactical/mod.rs:17` — `pub mod integration`
  - `src/core/domains/tactical/mod.rs:23` — `pub mod systems`
  - `src/core/domains/tactical/rules/mod.rs:6,7` — `pub mod movement`, `pub mod range`
  - `src/core/domains/tactical/systems/mod.rs:3,4` — `pub mod grid_system`, `pub mod movement_system`
- **问题描述**: Domain 内部子模块全部 `pub mod`，暴露实现细节。宪法 §7 要求"绝对禁止外部模块直接访问其他 Feature 的 internal 子模块"。
- **影响**: 领域边界不清晰，其他 Domain 可直接 `use tactical::systems::movement_system` 绕过集成层。
- **建议修复**: `pub mod events/integration/systems/rules` → `pub(crate) mod`；子系统 mod 同理。

---

## Debt-003: 可见性超标 — Capabilities 内部子模块

- **严重程度**: High
- **位置**（代表性）:
  - `src/core/capabilities/cue/mechanism/mod.rs:3,4` — `pub mod components`, `pub mod dispatch`
  - `src/core/capabilities/cue/foundation/mod.rs:3,4` — `pub mod types`, `pub mod values`
  - `src/core/capabilities/stacking/foundation/mod.rs:3,4` — `pub mod types`, `pub mod values`
  - `src/core/capabilities/stacking/mechanism/mod.rs:3` — `pub mod decider`
  - `src/core/capabilities/aggregator/mechanism/mod.rs:4,7` — `pub mod pipeline`, `pub mod systems`
  - `src/core/capabilities/runtime/pipeline/mechanism/mod.rs:3` — `pub mod executor`
  - `src/core/capabilities/runtime/pipeline/foundation/mod.rs:3,4` — `pub mod types`, `pub mod values`
  - `src/core/capabilities/runtime/scheduler/mechanism/mod.rs:3` — `pub mod executor`
  - `src/core/capabilities/runtime/scheduler/foundation/mod.rs:3,4` — `pub mod types`, `pub mod values`
  - `src/core/capabilities/runtime/registry/mechanism/mod.rs:3` — `pub mod validator`
- **问题描述**: Capabilities 的 foundation/mechanism 子模块中的 `components`、`types`、`values`、`dispatch`、`executor`、`validator`、`pipeline`、`decider` 等均为内部实现，应为 `pub(crate)`。
- **影响**: 双轴边界（Capabilities ↔ Domains）模糊，AI 可能直接引用 mechanism 内部类型。
- **建议修复**: foundation/mechanism 下的子模块统一改为 `pub(crate) mod`，通过 mod.rs 的 `pub use` re-export 所需 API。

---

## Debt-004: 未使用的 `pub use` 重导出（Low — 预留）

- **严重程度**: Low
- **位置**: 31 个模块有 unused import 警告
- **说明**: 这些 re-export 是"预留"性质的 API 设计——Capabilities 作为通用骨架，需要对外暴露完整接口供未来 Domains 消费。随着 Tactical 等业务域逐步接入，这些 re-export 会被自然使用。删除后未来又要加回来，反而增加维护成本。
- **建议**: 暂不处理。待 3-5 个 Domain 完成接入后，再扫描哪些 re-export 确实未被使用。

---

## Debt-005: Dead Code 分析

### 5a. Capabilities 预留 Dead Code（Low — 预期行为）

- **严重程度**: Low
- **数量**: ~350 处
- **说明**: Capabilities 系统（21k 行）已完整实现但尚无业务域消费。大量类型/枚举变体/字段/函数处于"已定义但未使用"状态——这是**正常的架构演进路径**，不是技术债。宪法 §1.3 区分了 🟥 绝对禁止、🟩 必须遵守、🟨 优先选择、🟦 最佳实践。Dead code 属于"可优化但非违规"范畴。
- **建议**: 暂不处理。待业务域接入后自然消除。

### 5b. 真正废弃的 Dead Code（Low — 预期行为）

- **严重程度**: Low
- **数量**: ~50 处
- **TOP 文件**:

  | 文件 | 警告数 | 问题 |
  |------|--------|------|
  | `src/shared/ids/types.rs` | 36 | `define_string_id!` 宏生成的 ID 类型未被使用（ModifierId, EffectId 等 17 个） |
  | `src/core/capabilities/ability/mechanism/lifecycle.rs` | 15 | 整个 ability lifecycle 文件（15 个函数）从未被任何系统调用 |
  | `src/core/capabilities/effect/mechanism/lifecycle.rs` | 12 | `can_apply` 等函数未使用 |
  | `src/core/capabilities/event/mechanism/bus.rs` | 5 | `filter_subscribers_by_tag`、`create_event_id` 未使用 |
  | `src/core/domains/tactical/tests/fixtures/tactical_fixtures.rs` | 6 | 所有 fixture 函数均未被测试引用 |
  | `src/core/domains/tactical/components.rs` | 2 | `Facing::new`、`HexDirection::ALL/delta` 未使用 |

- **建议**: 这些是明确的 dead code，应在后续重构中删除。

---

## 修复优先级

| 优先级 | Debt ID | 修复方式 | 状态 |
|--------|---------|---------|------|
| ~~P0~~ | ~~001, 002, 003~~ | ~~`pub mod` → `pub(crate) mod`~~ | ✅ 已完成 |
| **P1** | 005b | 删除明确 dead code | 待处理 |
| **P2** | 004, 005a | 暂不处理，待域接入后复查 | — |

---

*本清单由 @refactor-guardian 扫描生成，已对照 `ai-constitution-complete.md` v5.0 修正评估。*
