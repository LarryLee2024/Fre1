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

| 类别 | 严重程度 | 数量 | 编号 |
|------|----------|------|------|
| 架构漂移 | High | 1 | Drift-ADR-002 |
| 抽象泄漏 | Critical | 2 | Leak-003, Leak-004 |
| 内容债务 | Medium | 1 | Content-002 |
| 测试债务 | Medium | 1 | TestDebt-002 |
| AI 可维护性 | None | 0 | — |
| 超大文件 | None | 0 | — |

---

## Debt-019: [Drift-ADR-002] camp_rest 系统反向依赖 Infra 层

- **状态**: Open
- **发现日期**: 2026-06-19
- **负责人**: @architect
- **关联 ADR**: ADR-001 (依赖方向), ADR-041 (Replay)
- **位置**: `src/core/domains/camp_rest/systems/camp_rest_system.rs:12`
- **严重程度**: **High**
- **问题描述**: `CampRestSystem` 直接 `use crate::infra::replay::resources::FrameCounter`。Core(L1) 模块反向依赖 Infra(L2)，违反 ADR 定义的 Shared ← Core ← Infrastructure 依赖方向。
- **影响**: Core 层代码依赖 Infra 实现细节，部署时 Core 无法独立复用；依赖方向检查工具产生误报。
- **建议修复**: `FrameCounter` 应通过 Core 层的 `runtime::replay` facade 暴露一个 read-only 的 GameFrame Resource，或 camp_rest 改为通过 Observer 监听 replay 事件而非直接引入 Resource。

---

## Debt-020: [Leak-003] turn_systems.rs 绕过 Combat integration/ 层

- **状态**: Open
- **发现日期**: 2026-06-19
- **负责人**: @architect
- **关联 ADR**: ADR-006 (Capabilities/Domains 双轴), ADR-021 (Combat)
- **位置**: `src/core/domains/combat/systems/turn_systems.rs:13-15`
- **严重程度**: **Critical**
- **问题描述**: `turn_systems.rs` 直接从 `capabilities::ability::mechanism::ActiveAbilityContainer` 和 `capabilities::trigger::mechanism::TriggerContainer` 导入类型。根据架构规则 (ADR-006 §6.2)，Domain 必须通过 `integration/` 模块（Facade + SystemParam）访问 Capabilities。
- **影响**: integration/ 层被绕过，后续修改 Capabilities 内部类型将直接影响 Domain 代码；Facade 的抽象价值丧失。
- **建议修复**: `turn_systems.rs` 应通过 `combat::integration::ability::CombatAbilityFacade` 和 `combat::integration::trigger::CombatTriggerFacade` 访问。具体来说：
  1. 在 `combat::integration::ability` 中新增 `tick_cooldowns_for_unit(unit: Entity, ability_query: &mut Query<&mut ActiveAbilityContainer>)` facade 函数
  2. 在 `combat::integration::trigger` 中新增 `evaluate_triggers_for_unit(unit: Entity, trigger_type: TriggerType, trigger_query: &mut Query<&TriggerContainer>)` facade 函数
  3. `turn_systems.rs` 改为调用这些 facade 函数

---

## Debt-021: [Leak-004] movement_system.rs 绕过 Tactical integration/ 层

- **状态**: Open
- **发现日期**: 2026-06-19
- **负责人**: @architect
- **关联 ADR**: ADR-006 (Capabilities/Domains 双轴)
- **位置**: `src/core/domains/tactical/systems/movement_system.rs:13-15`
- **严重程度**: **Critical**
- **问题描述**: `movement_system.rs` 直接导入 `AttributeContainer`, `ModifierContainer`, `TagHierarchy`, `TagSet` — 而 `tactical::integration::movement` 已经定义了 `MovementCapabilityParam` SystemParam 作为 facade。System 绕过了自己的 integration/ 层。
- **影响**: integration/ 层抽象被绕过，Capabilities 内部类型变更会直接破坏 tactial domain。
- **建议修复**: `movement_system.rs` 应使用 `MovementCapabilityParam`（已存在）替代直接访问 Capabilities 内部类型。检查 `MovementCapabilityParam` 是否暴露了当前 system 所需的所有 API，如缺失则补充。

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
| **Critical** | 2 | Leak-003, Leak-004 |
| **High** | 1 | Drift-ADR-002 |
| **Medium** | 2 | Content-002, TestDebt-002 |
| Low | 0 | — |
| **总计** | **5** | |

### 修复优先级建议

1. **P0**: Leak-003, Leak-004 — integration/ 层是 Domain 与 Capabilities 间的强制边界，绕过使整个架构模式失效。修复后再开发 combat 域新功能。
2. **P1**: Drift-ADR-002 — Core→Infra 反向依赖破坏分层架构。
3. **P2**: TestDebt-002, Content-002 — 渐进式改进。
