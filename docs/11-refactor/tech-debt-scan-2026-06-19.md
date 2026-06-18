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
| 架构漂移 | High | 1 | Drift-ADR-002 | ⏳ Open |
| 抽象泄漏 | Critical | 2 | Leak-003, Leak-004 | ✅ Resolved |
| 内容债务 | Medium | 1 | Content-002 | ⏳ Open |
| 测试债务 | Medium, Low | 2 | TestDebt-002, TestDebt-003 | ⏳ Open |
| AI 可维护性 | None | 0 | — | ✅ |
| 超大文件 | None | 0 | — | ✅ |

---

## Debt-019: [Drift-ADR-002] camp_rest 系统反向依赖 Infra 层

- **状态**: 💡 Fix Planned (2026-06-20)
- **发现日期**: 2026-06-19
- **负责人**: @architect → @feature-developer
- **关联 ADR**: ADR-001 (依赖方向), ADR-041 (Replay)
- **位置**: `src/core/domains/camp_rest/systems/camp_rest_system.rs:12`
- **严重程度**: **High**
- **问题描述**: `CampRestSystem` 直接 `use crate::infra::replay::resources::FrameCounter`。Core(L1) 模块反向依赖 Infra(L2)，违反 ADR 定义的 Shared ← Core ← Infrastructure 依赖方向。
- **影响**: Core 层代码依赖 Infra 实现细节，部署时 Core 无法独立复用；依赖方向检查工具产生误报。
- **架构评估**: 
  - `FrameCounter` 是一个简单的 u64 包装 Resource，每帧在 PreUpdate 递增。
  - `shared::time::GameTime` (L0, Shared) 已存在，自 P0 接入（§1.3）已在 SharedPlugin 中注册并每帧推进。
  - `GameTime.frame()` 与 `FrameCounter.0` 语义等价（同为 PreUpdate 递增的单调帧计数）。
  - `RestState.last_long_rest_frame` 字段注释（第 96 行）已注明为 "GameTime 帧计数"。
  - camp_rest 使用 FrameCounter 的唯一位置是 `handle_long_rest_complete` 中记录长休完成帧。
- **建议修复**: 将 camp_rest 中对 `Res<FrameCounter>` 的依赖替换为 `Res<GameTime>`。`FrameCounter` 保留在 `infra::replay` 中供回放基础设施自用。此方案零新增类型、零新增模块、零架构变更，仅修复现存依赖方向违规。

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

- **状态**: Open
- **发现日期**: 2026-06-19
- **负责人**: @feature-developer
- **严重程度**: Medium
- **问题描述**: 以下文件存在硬编码业务数值，应通过 content/ 配置驱动：
  - `src/core/domains/combat/pipeline/steps.rs` — 行动点数、回合限制等
  - `src/core/domains/spell/rules/rules.rs` — 法术位、专注时长等
  - `src/core/domains/progression/rules/formulas.rs` — 升级曲线公式中的常数
- **影响**: 调整游戏平衡性需要修改 Rust 代码而非 RON 配置，违反 Rule/Content 分离原则。
- **建议修复**: 将这些数值提取到 `assets/config/` 下的 RON 文件中，通过 Bevy Asset 加载。

---

## Debt-023: [TestDebt-002] combat integration/ facade 测试不完整

- **状态**: Open
- **发现日期**: 2026-06-19
- **负责人**: @test-guardian
- **严重程度**: Medium
- **问题描述**: combat 的 `integration/` 下有 8 个 facade（ability, aggregator, condition, effect, event, execution, gameplay_context, targeting, trigger），但部分 facade 测试不完整。`effect` facade 的测试在 `tests/mod.rs` 而非独立的 `tests/facade_test.rs`，不一致。
- **建议修复**: 统一 facade 测试模式，确保每个 facade 有独立的 `tests/facade_test.rs`，并覆盖主线流程。

---

## Debt-024: [TestDebt-003] movement facade 测试位于 Domain 级 tests/unit/ 而非 integration/movement/tests/

- **状态**: Open
- **发现日期**: 2026-06-20
- **负责人**: @test-guardian
- **严重程度**: Low
- **位置**: `src/core/domains/tactical/tests/unit/facade_test.rs` (当前位置) → 应迁至 `src/core/domains/tactical/integration/movement/tests/facade_test.rs`
- **问题描述**: tactical 域的 movement facade 测试 (`facade_test.rs`, `movement_cost_test.rs`, `movement_points_test.rs`) 位于 `tactical/tests/unit/` 下，而非按照 ADR-046 标准的 `integration/movement/tests/facade_test.rs`。其他 integration facade（如 combat 域的 trigger facade）已将测试放在 `integration/{name}/tests/` 下，存在结构不一致。
- **影响**: 测试不跟随 facade 走，迁移/重构时需要额外搜索；新开发者容易混淆测试组织方式。
- **建议修复**: 将 `tactical/tests/unit/facade_test.rs` 等 3 个相关测试文件移至 `tactical/integration/movement/tests/` 下，调整 mod.rs 导入路径，确保 cargo test 无回归。

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
| **Critical** | 0| — (均已修复) |
| **High** | 1 | Drift-ADR-002 |
| **Medium** | 2 | Content-002, TestDebt-002 |
| Low | 1 | TestDebt-003 |
| **总计** | **4** | |

### 修复优先级建议

1. **P1**: Drift-ADR-002 — Core→Infra 反向依赖破坏分层架构。唯一剩余的 High 级别问题。需 @architect 评估修复方案。
2. **P2**: Content-002, TestDebt-002 — 渐进式改进，可随功能开发同时处理。
3. **P3**: TestDebt-003 — Low 级别，测试目录结构一致性，可在重构时顺带修复。
