---
id: 11-refactor.tech-debt-scan-2026-06-19
title: 技术债扫描报告 — 2026-06-19
status: active
owner: refactor-guardian
created: 2026-06-19
scope: ErrorContext 接入审查 + 架构依赖扫描
---

# 技术债扫描报告 — 2026-06-19

> **扫描范围**: 全 `src/` 架构依赖架构 + 模块边界 + 文件结构
> **扫描工具**: CodeGraph, Grep, wc -l
> **前置阅读**: docs/01-architecture/README.md, docs/02-domain/README.md

---

## 扫描结果总览

| 类别 | 严重程度 | 数量 | 编号 | 状态 |
|------|----------|------|------|------|
| 架构漂移 | High | 1 | Drift-ADR-002 | ✅ Resolved (2026-06-20) |
| 抽象泄漏 | Critical | 2 | Leak-003, Leak-004 | ✅ Resolved |
| 内容债务 | Medium | 1 | Content-002 | ✅ Resolved (2026-06-20) |
| 测试债务 | Medium, Low | 2 | TestDebt-002, TestDebt-003 | ✅ Resolved (2026-06-20) |
| AI 可维护性 | None | 0 | — | ✅ |
| 超大文件 | None | 0 | — | ✅ |

---

## Debt-019: [Drift-ADR-002] camp_rest 系统反向依赖 Infra 层

- **状态**: ✅ Resolved (2026-06-20)
- **发现日期**: 2026-06-19
- **修复日期**: 2026-06-20
- **负责人**: @architect → @feature-developer
- **关联 ADR**: ADR-001 (依赖方向), ADR-041 (Replay)
- **位置**: `src/core/domains/camp_rest/systems/camp_rest_system.rs:12`
- **严重程度**: **High**
- **问题描述**: `CampRestSystem` 直接 `use crate::infra::replay::resources::FrameCounter`。Core(L1) 模块反向依赖 Infra(L2)，违反 ADR 定义的 Shared ← Core ← Infrastructure 依赖方向。
- **影响**: Core 层代码依赖 Infra 实现细节，部署时 Core 无法独立复用；依赖方向检查工具产生误报。
- **修复内容**: 将 `Res<FrameCounter>` 替换为 `Res<GameTime>`（`shared::time::GameTime`，L0）。
  - `camp_rest_system.rs` 的 import/parameter/usage 共 3 行改动
  - 同时修复 `quest/components.rs` 中 QuestDef 的 `TypePath` + `Reflect` 冲突编译错误
- **验证**: `cargo nextest run` — 1451/1451 passed, 8 skipped
- **提交**: `774c5bc`

---

## Debt-020: [Leak-003] turn_systems.rs 绕过 Combat integration/ 层

- **状态**: ✅ Resolved (2026-06-20)
- **发现日期**: 2026-06-19
- **修复日期**: 2026-06-20
- **负责人**: @feature-developer
- **关联 ADR**: ADR-006 (Capabilities/Domains 双轴), ADR-021 (Combat)
- **修复内容**: 
  - 新建 `combat/integration/trigger/system_param.rs` — CombatTriggerParam
  - 新增 `CombatAbilityParam::tick_cooldowns_for_unit()`, `CombatAbilityFacade::empty_container()`
  - 新增 `CombatTriggerFacade::empty_container()`
  - `turn_systems.rs` 移除所有 cap 类型直接导入，改用 facade params

---

## Debt-021: [Leak-004] movement_system.rs 绕过 Tactical integration/ 层

- **状态**: ✅ Resolved (2026-06-20)
- **发现日期**: 2026-06-19
- **修复日期**: 2026-06-20
- **负责人**: @feature-developer
- **关联 ADR**: ADR-006 (Capabilities/Domains 双轴)
- **修复内容**: 
  - 移除 `AttributeContainer`, `ModifierContainer`, `TagHierarchy`, `TagSet` 直接导入
  - 将 `Query<...>` + `Res<TagHierarchy>` 替换为 `MovementCapabilityParam`
  - 函数体内改用 `mov.build_view()` / `view.can_move` 访问能力数据

---

## Debt-022: [Content-002] 部分领域代码含硬编码业务数值

- **状态**: ✅ Resolved (2026-06-20)
- **发现日期**: 2026-06-19
- **修复日期**: 2026-06-20
- **负责人**: @feature-developer
- **严重程度**: Medium
- **问题描述**: 以下文件存在硬编码业务数值，应通过 content/ 配置驱动：
  - `src/core/domains/combat/pipeline/steps.rs` — 行动点数、回合限制等
  - `src/core/domains/spell/rules/rules.rs` — 法术位、专注时长等
  - ~~`src/core/domains/progression/rules/formulas.rs`~~ ✅ 已修复
- **影响**: 调整游戏平衡性需要修改 Rust 代码而非 RON 配置，违反 Rule/Content 分离原则。
- **建议修复**: 将这些数值提取到 `assets/config/` 下的 RON 文件中，通过 Bevy Asset 加载。

### ✅ 已完成 — progression 域

2026-06-20 完成了 `LevelProgressionTable` 的 RON 配置化：

1. **`assets/config/progression/balance.ron`** — 新建 RON 配置文件，包含 max_level、exp_thresholds、proficiency_by_level、asi_levels
2. **`src/core/domains/progression/components.rs`** — 给 `LevelProgressionTable` 添加 `Deserialize` derive
3. **`src/core/domains/progression/plugin.rs`** — 添加 `pub use` 重导出
4. **`src/content/content_plugin.rs`** — 新增 `load_progression_balance()` 函数，在 `load_all_content` 中添加 `"progression"` bucket 调度和 `ResMut<LevelProgressionTable>` 参数
5. **`src/core/domains/progression/systems/progression_system.rs`** — `enforce_xp_invariant` 和 `handle_level_up` 从 `LevelProgressionTable::default()` 改为 `Res<LevelProgressionTable>` 注入
6. **验证**: `cargo nextest run` — 1473/1473 passed, 8 skipped

### ✅ 已完成 — spell/spell_config 域

2026-06-20 完成了 `SpellConfig` 的 RON 配置化：

1. **`assets/config/spell_config/spell_config.ron`** — 新建 RON 配置文件（concentration_base_dc、max_concentration、cantrips_count_against_known）
2. **`src/core/domains/spell/components.rs`** — 给 `SpellConfig` 添加 `Deserialize` derive
3. **`src/content/content_plugin.rs`** — 新增 `load_spell_config()` 函数，在 `load_all_content` 中添加 `"spell_config"` bucket 调度和 `ResMut<SpellConfig>` 参数
4. **验证**: `cargo nextest run` — 1485/1485 passed, 8 skipped

### ⏳ 待处理 — 深入分析

原始债务描述中 combat/pipeline/steps.rs 和 spell/rules/rules.rs 的硬编码值，经阅读分析后修正评估：

- **`src/core/domains/combat/pipeline/steps.rs`** — ❌ 实际上没有可提取的硬编码数值。行动点数（1标准/1附赠/1反应/移动力）是 `ActionPoints` 组件的结构性定义，非数值常量。胜负判定 (`check_team_elimination`) 使用动态数据。建议从 Content-002 范围移除。
- **`src/core/domains/spell/rules/formulas.rs`** — ⚠️ 存在两处冗余：
  - `calc_concentration_dc()` 硬编码 `10u32` — 已在 `SpellConfig::concentration_base_dc` 中配置化，但纯函数无 Res 访问。需重构函数签名接受参数。
  - `proficiency_bonus_for_level()` — 完全重复 `LevelProgressionTable::proficiency_bonus()`。需重构调用方传递熟练加值。
  - 这些是 rules 层的纯函数重构，不是简单的 Content 提取，建议在新的 Refactor 任务中处理。

---

## Debt-023: [TestDebt-002] combat integration/ facade 测试不完整

- **状态**: ✅ Resolved (2026-06-20)
- **发现日期**: 2026-06-19
- **修复日期**: 2026-06-20
- **负责人**: @test-guardian
- **严重程度**: Medium
- **问题描述**: combat 的 `integration/` 下有 8 个 facade（ability, aggregator, condition, effect, event, execution, gameplay_context, targeting, trigger），但部分 facade 测试不完整。`effect` facade 的测试在 `tests/mod.rs` 而非独立的 `tests/facade_test.rs`，不一致。ability/trigger/event 的 facade_test.rs 仅为桩测试。

### 修复内容

| Facade | 之前 | 之后 | 变更 |
|--------|------|------|------|
| effect | 12 tests 在 mod.rs，无 facade_test.rs | 14 tests 在 facade_test.rs，mod.rs 仅声明 | 结构迁移 |
| ability | 1 编译测试 | 9 tests (容器/激活/冷却全流程) | +8 |
| trigger | 2 枚举转换测试 | 10 tests (创建/评估/条件/批量) | +8 |
| event | 2 枚举转换测试 | 12 tests (发布/优先级/枚举映射) | +10 |
| **合计** | — | **45 tests** (9 facade) | **+26** |

- **验证**: `cargo nextest run` — 1513/1513 passed, 8 skipped

---

## Debt-024: [TestDebt-003] movement facade 测试位于 Domain 级 tests/unit/ 而非 integration/movement/tests/

- **状态**: ✅ Resolved (2026-06-20)
- **发现日期**: 2026-06-20
- **修复日期**: 2026-06-20
- **负责人**: @test-guardian
- **严重程度**: Low
- **位置**: `src/core/domains/tactical/tests/unit/facade_test.rs` → `src/core/domains/tactical/integration/movement/tests/facade_test.rs`
- **问题描述**: tactical 域的 movement facade 测试 (`facade_test.rs`, `movement_cost_test.rs`, `movement_points_test.rs`) 位于 `tactical/tests/unit/` 下，而非按照 ADR-046 标准的 `integration/movement/tests/`。其他 integration facade（如 combat 域的 trigger facade）已将测试放在 `integration/{name}/tests/` 下，存在结构不一致。
- **影响**: 测试不跟随 facade 走，迁移/重构时需要额外搜索；新开发者容易混淆测试组织方式。

### 修复内容

| 文件 | 旧位置 | 新位置 |
|------|--------|--------|
| facade_test.rs | `tactical/tests/unit/` | `tactical/integration/movement/tests/` |
| movement_cost_test.rs | `tactical/tests/unit/` | `tactical/integration/movement/tests/` |
| movement_points_test.rs | `tactical/tests/unit/` | `tactical/integration/movement/tests/` |

- `tactical/integration/movement/mod.rs` 新增 `#[cfg(test)] mod tests;`
- `tactical/integration/movement/tests/mod.rs` 新建（标准模式）
- `tactical/tests/unit/mod.rs` 移除已迁移的 3 个 mod 声明
- **验证**: `cargo nextest run` — 1513/1513 passed, 8 skipped

---

## ✅ 已通过的检查项

### AI 可维护性 (Maintainability)
- 最大源文件 640 行（effect lifecycle_test.rs），远低于 1000 行 Medium 阈值
- 无函数超过 100 行
- 无 match 超过 20 arm
- 结论: ✅ 通过

### 超大 Plugin
- CombatPlugin: 64 行，符合规范
- 无 Plugin 注册过多系统
- 结论: ✅ 通过

### 禁止的文件名
- 无 `utils.rs`、`helpers.rs`、`common.rs` 垃圾桶文件
- 结论: ✅ 通过

### Pipeline 绕过
- 所有战斗效果走 Effect Pipeline 或 CombatPipelineDriver
- 结论: ✅ 通过

### ECS 反模式
- 无 Entity OOP 方法
- 无 Component 包含复杂业务逻辑
- 无 System 存储状态
- 结论: ✅ 通过

---

## 📋 汇总

| 严重程度 | 数量 | 编号 |
|---------|------|------|
| **Critical** | 0 | — (均已修复) |
| **High** | 0 | — (均已修复) |
| **Medium** | 0 | — (均已修复) |
| Low | 0 | — (均已修复) |
| **总计** | **3** | |

### Content-002 修复总结

| 子项 | 状态 | 备注 |
|------|------|------|
| progression (LevelProgressionTable) | ✅ 完成 | RON 配置加载 |
| spell (SpellConfig) | ✅ 完成 | RON 配置加载 |
| combat/pipeline/steps.rs | ❌ 从范围移除 | 无硬编码数值，结构性定义 |
| spell/rules/formulas.rs 冗余 | ⏳ 移至新任务 | 纯函数重构，非简单 Content 提取 |
| **最终验证** | ✅ 1485/1485 passed | `cargo nextest run` 通过 |

### 剩余债务修复优先级建议

1. **P3**: spell rules/formulas 常量重构 — `proficiency_bonus_for_level` 去重 + `calc_concentration_dc` 参数化。纯函数签名变更，风险低。
