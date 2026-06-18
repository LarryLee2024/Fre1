---
id: 11-refactor.debt-inventory-2026-06-17
title: "技术债清单 — 首次全量扫描"
status: active
scanner: refactor-guardian
created: 2026-06-17
updated: 2026-06-18 (D-9 Delta 扫描 + ADR-024 修复 + C-4 Replay 桥接层扫描 + C-5 Save 桥接层扫描)
scan_scope: src/ (full codebase)
baseline_warnings: 433 (C-4 infra/replay 增量：0 新增 warning)
---

# 技术债清单 — 首次全量扫描

> 扫描范围: `src/` 全量
> 扫描时间: 2026-06-17
> 基线: `cargo build` 产生 433 个 warning（dead_code 400 + unused_import 31 + other 2）
> 参照: `docs/00-governance/ai-constitution-complete.md` v5.0
> 验证: `cargo test --lib` 742 passed, 0 failed

## 格式规范

每个 Debt 条目必须包含以下字段（Debt Lifecycle）：

```markdown
## [ID]: [类别] [简短描述]
- **状态**: Open / Accepted / In Progress / Resolved / WontFix
- **发现日期**: YYYY-MM-DD
- **负责人**: @role
- **关联 ADR**: ADR-XXX（如有）
- **位置**: file:line
- **严重程度**: Critical / High / Medium / Low
- **问题描述**: ...
- **影响**: ...
- **建议修复**: ...
```

**ID 命名规范：**

| 类别 | 前缀 | 示例 |
|------|------|------|
| 通用技术债 | `Debt-` | `Debt-001` |
| 架构漂移 | `Drift-ADR-` | `Drift-ADR-001` |
| 抽象泄漏 | `Leak-` | `Leak-001` |
| AI 可维护性 | `Maintain-` | `Maintain-001` |
| 测试债务 | `TestDebt-` | `TestDebt-001` |
| 内容债务 | `Content-` | `Content-001` |

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

## D-9 Delta 扫描 (2026-06-18) — 回合系统 (Effect Tick + 胜利条件)

> 扫描范围: `src/core/domains/combat/` + `src/core/capabilities/effect/foundation/values.rs`
> 基线: `cargo test` 908 passed, 0 failed, cargo build 0 errors
> 关联角色: @feature-developer (R4), @test-guardian (新增不变量测试), @code-reviewer (PASS)

### Debt-D9-001: Combat 域缺少 `integration/` 模块

- **位置**: `src/core/domains/combat/` — 无 `integration/` 目录
- **严重程度**: **Medium**
- **状态**: ✅ **已修复** (2026-06-18, ADR-024)
- **问题描述**: 架构法 §6.2 规定每个 Business Domain 必须有 `integration/` 模块作为调用 Capabilities 的唯一入口（Facade + SystemParam 模式）。当前 `effect_tick_system.rs` 直接 `use` Effect 能力层的 `ActiveEffectContainer` 和 `tick_durations`/`expire_effects`，绕过 integration 层。
- **影响**: 随着 Combat 域持续扩展（Spell, Reaction, Progression 等），Capabilities 直接 imports 将泛滥，导致架构边界退化和未来重构难度增加。
- **修复**: 创建了 `combat/integration/effect/` 模块（facade + types + system_param），重构 `effect_tick_system.rs` 使用 `EffectTickParam`。详见 ADR-024。

### Debt-D9-002: effect_tick_system 双重 Query 迭代

- **位置**: `src/core/domains/combat/systems/effect_tick_system.rs:36-70`
- **严重程度**: **Low**（接近 Medium，但文件仅 71 行）
- **状态**: ✅ **已修复** (2026-06-18)
- **问题描述**: `on_turn_end_tick_effects` 对 `container_query` 做了两次 `.iter_mut()` 遍历——一次做 `tick_durations` + 事件发射，一次做 `expire_effects`。`expire_effects` 只处理已标记为 `Expiring` 的效果，不影响 `Active` 实例后续 tick，因此合并为单次 pass 是安全的。
- **影响**: 轻微性能浪费（两次迭代 vs 一次）；更严重的是代码可读性——读者需要理解"为什么分两次"。
- **修复**: `EffectTickParam.tick_all()` 在 `tick_and_expire` 中合并为单次 pass。现 Observer 是 49 行（原 71 行）。

### Debt-D9-003: OnTurnEnd tick 行为与 domain doc 描述存在微妙偏差

- **位置**: `docs/02-domain/domains/combat_domain.md` §5.2 vs `combat/systems/effect_tick_system.rs:36`
- **严重程度**: **Low**
- **问题描述**: Domain doc 说 "(2) 当前单位回合效果 Tick" 和 "(5a) RoundEnd 所有效果 Tick"，但当前实现是每次 OnTurnEnd 都 tick 所有实体。这既是过度实现（tick 过多）也是领域文档与实际代码的偏差。
- **影响**: 领域规则和实现不一致，后续开发者在理解系统行为时会产生困惑。
- **建议修复**: 确认意图后选择 (a) 更新 domain doc 明确"每次 OnTurnEnd 推进所有效果" 或 (b) 修改 Observer 为只 tick 当前单位（OnTurnEnd.unit），RoundEnd Observer 再 tick 全部。

### Debt-D9-004: effect_tick_test.rs 包含与 effect 单元测试的实质性重复

- **位置**: `src/core/domains/combat/tests/integration/effect_tick_test.rs` — test 1-7 与 `effect/tests/unit/lifecycle_test.rs` 高度重叠
- **严重程度**: **Low**
- **状态**: ✅ **已修复** (2026-06-18)
- **问题描述**: effect_tick_test 前 7 个测试（tick_durations 递减、过期、周期 Tick、Infinite、Paused、多效果）在 effect 能力层的 `lifecycle_test.rs`（640 行，50+ 测试）中已有完整覆盖。这些测试在 combat 层重复了 effect 层的纯函数测试，而非测试"combat 与 effect 的集成"。
- **影响**: 维护成本增加——effect 行为变更需要同步修改两处测试。904 测试中 9 个（effect_tick_test）+ 4 个（invariant）覆盖了同一组函数，冗余比约 15:1。
- **修复**: effect_tick_test 测试 1-8 标记为 `#[ignore]`。`integration/effect/facade.rs` 新增 22 个 facade 专用测试，覆盖全部 facade 函数。

### Debt-D9-005: 领域文档日期未完全同步

- **位置**: `docs/02-domain/README.md` — `updated: 2026-06-17` 应更新为 2026-06-18（effect_domain.md 已更新）
- **严重程度**: **Low**
- **问题描述**: effect_domain.md 已更新日期到 2026-06-18（ActiveEffectContainer 添加 Component derive），但 README.md 索引中的 updated 日期仍为 2026-06-17。
- **建议修复**: 同步更新。

### D-9 修复优先级

| 优先级 | Debt ID | 状态 | 修复方式 | 执行人 |
|--------|---------|------|---------|--------|
| **P0** | D9-001 | ✅ 已修复 | 创建 integration/effect/ 模块 (ADR-024) | @feature-developer |
| **P1** | D9-002 | ✅ 已修复 | 合并为 tick_and_expire 单次 pass | @feature-developer |
| **P2** | D9-003 | ❌ 保留 | domain doc 对齐（OnTurnEnd tick 频率） | @domain-designer |
| **P3** | D9-004 | ✅ 已修复 | 测试精简 + facade 测试替代 | @test-guardian |
| **P3** | D9-005 | ❌ 保留 | 日期同步（README 元数据） | —— |

### C-4 Delta: Replay 桥接层扫描

| 检查项 | 结果 | 备注 |
|--------|------|------|
| Dead Code (未使用的 pub) | ✅ 零新增 | 所有 re-export 对应实际消费路径 |
| 可见性超标 (ADR-045) | ✅ 合规 | resources/systems 为 pub(crate)，仅 plugin + re-exports 为 pub |
| 超大文件 (>500 行) | ✅ 无 | 最大文件 212 行 (recording_lifecycle_test.rs) |
| 超大 Plugin | ✅ 合规 | ReplayPlugin 注册 5 资源 + 4 系统，66 行 |
| 禁止的文件名 | ✅ 无 | 无 utils.rs / helpers.rs |
| cargo build 警告 | ✅ 零新增 | 0 replay-specific warnings |
| Bevy 0.18 模式合规 | ✅ 通过 | observer-based events, FromWorld/Default, chain() |
| ADR-041 对齐 | ✅ 已确认 | Resource/System/Event 设计与 ADR §4-5 一致 |

**结论**: C-4 Replay 桥接层无新增技术债。5 个实现文件（145 行）+ 7 个测试文件（~250 行）均通过 @code-reviewer 审查。

### C-5 Delta: Save 桥接层扫描

| 检查项 | 结果 | 备注 |
|--------|------|------|
| Dead Code (未使用的 pub) | ✅ 零新增 | 所有 re-export 对应预期消费路径 |
| 可见性超标 (ADR-045) | ✅ 合规 | resources/systems 为 pub(crate)，events 为 private，仅 re-exports 为 pub |
| 超大文件 (>500 行) | ✅ 无 | 最大文件 ~100 行 |
| 超大 Plugin | ✅ 合规 | SavePlugin 注册 3 资源 + 2 observer，20 行 |
| 禁止的文件名 | ✅ 无 | 无 utils.rs / helpers.rs |
| cargo build 警告 | ✅ 零新增 | 0 save-specific warnings |
| Bevy 0.18 模式合规 | ✅ 通过 | observer-based events (On<T>), Default traits, app.add_observer |
| ADR-042 对齐 | ✅ 已确认 | Resource/Event 设计与 ADR §2-3 一致 |

**结论**: C-5 Save 桥接层无新增技术债。5 个实现文件（~260 行）+ 7 个测试文件（~250 行）均通过 @code-reviewer 审查。

---

*D-9 + C-4 + C-5 Deltas 由 @refactor-guardian 扫描生成。2026-06-18 验证：932 tests passed, 8 ignored, 0 failed, cargo build 0 errors。完整债务清单见首次全量扫描（上方 Debt-001~006）。*

---

### B-1 Delta: Progression & Inventory 域测试覆盖扫描

| 检查项 | 结果 | 备注 |
|--------|------|------|
| Dead Code (未使用的 pub) | ✅ 零新增 | 所有新 pub 函数均有测试消费 |
| 可见性超标 (ADR-045) | ✅ 合规 | tests 模块为 `#[cfg(test)]`，不侵入生产代码 |
| 超大文件 (>500 行) | ✅ 无 | 最大测试文件 components_test.rs ~100 行 |
| 禁止的文件名 | ✅ 无 | 无 utils.rs / helpers.rs |
| cargo build 警告 | ✅ 零新增 | 新代码未引入新 warning |
| Bevy 0.18 模式合规 | ✅ 通过 | observer-based events, reflect Components, Default traits |
| 测试接线完整 | ✅ 已确认 | unit/invariant/integration 三层 + fixtures 预留 |
| Bug 修复数 | 4 个 | B1-B4 均在修复当天验证，全部通过 |

**发现并修复的 Bug**（4 个，均已 resolved）：

| ID | 类别 | 位置 | 描述 | 严重程度 |
|----|------|------|------|----------|
| B1 | 索引错误 (off-by-one) | `progression/rules/formulas.rs: xp_to_next_level` | `idx = current_level` 应为 `current_level - 1`，导致 1→2 级消耗计算为 600 XP（应为 300） | High |
| B2 | 索引错误 (off-by-one) | `progression/components.rs: LevelProgressionTable::xp_for_level` | `idx = level` 应为 `level - 1`，导致 `xp_for_level(2)` 返回 900（应为 300） | High |
| B3 | 资源初始化错误 | `progression/components.rs: LevelProgressionTable` | `#[derive(Default)]` 与固有 `fn default()` 冲突，`init_resource` 创建全零实例 | High |
| B4 | 重量/数量逻辑错误 | `inventory/components.rs: Inventory::add_item` | 新格子路径未乘以 quantity 导致重量偏小；未对 quantity 上限 99 | Medium |

**新增 TestDebt**（本轮未修复，建议后续处理）：

- **TestDebt-B-001**: `tests/fixtures/` 在 progression 和 inventory 域中为空。建议填充共享辅助函数（如 `make_entity_with_xp(xp, level)`、`make_potion(qty)`）减少测试代码重复。
  - **严重程度**: Low
  - **建议修复**: 填充 `tests/fixtures/mod.rs`

- **TestDebt-B-002**: `Inventory::stackable_to_existing()` 返回 tuple `(u32, u32)`，其中第二元素 `space` 在 `add_item` 中未使用。
  - **严重程度**: Low
  - **建议修复**: 简化为只返回 `to_add`

- **TestDebt-B-003**: `Inventory::add_item` 的堆叠合并只匹配第一个同模板物品。若存在多个同模板堆叠（如两堆 potion），只合入第一个。
  - **严重程度**: Low
  - **建议修复**: 遍历所有匹配堆叠，依次合并剩余数量

**结论**: B-1 批次引入 4 个 High 级别 Bug（均已修复）+ 3 个 Low 级别 TestDebt（建议后续处理）。新测试 182 个已覆盖 3 层测试模型，1120 tests passed, 0 failed。全量验证通过后债务净减少。*2026-06-18 验证：1120 tests passed, 8 ignored, 0 failed, cargo build 0 errors。*
