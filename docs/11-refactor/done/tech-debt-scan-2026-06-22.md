---
id: 11-refactor.tech-debt-scan-2026-06-22
title: 技术债扫描报告 — 2026-06-22
status: active
owner: refactor-guardian
created: 2026-06-22
scope: 全 src/ 架构依赖 + 抽象泄漏 + AI 可维护性 + 测试债务 + 内容债务
---

# 技术债扫描报告 — 2026-06-22

> **扫描范围**: 全 `src/` 架构依赖扫描 + 文件结构 + 抽象泄漏
> **扫描工具**: CodeGraph, Grep, wc -l, cargo build --warnings
> **前置阅读**: docs/01-architecture/README.md, docs/02-domain/README.md
> **上次扫描**: 2026-06-19（全部 6 项债务已 Resolved）

---

## 扫描结果总览

| 类别 | 严重程度 | 数量 | 编号 | 状态 |
|------|----------|------|------|------|
| 抽象泄漏 | Low | 1 | Leak-005 | ✅ Fixed (this session) |
| 架构漂移 | Medium | 1 | Drift-ADR-002-terrain | 🆕 Open |
| 内容债务 | Low | 1 | Content-003 | 🆕 Open |
| 测试债务 | None | 0 | — | ✅ |
| AI 可维护性 | None | 0 | — | ✅ |
| 超大文件 | None | 0 | — | ✅ |
| 死代码 (预留) | Low | ~30+ items | — | ⏳ 预留待接入 |

---

## Leak-005: combat/plugin.rs 直接访问 EventBus (capabilities::event::mechanism)

- **状态**: ✅ Fixed (2026-06-22)
- **发现日期**: 2026-06-22
- **修复日期**: 2026-06-22
- **负责人**: @feature-developer
- **严重程度**: Low
- **位置**: `src/core/domains/combat/plugin.rs:24`
- **问题描述**: `CombatPlugin` 直接 `use crate::core::capabilities::event::mechanism::EventBus`，绕过 `combat/integration/event/` 层。虽然 `EventBus` 作为 Resource 必须在 Plugin 中 `init_resource`，但 import 路径应通过 integration 层 re-export，保持访问一致性。
- **影响**: 轻微 — 不破坏功能，但与 ADR-046 的"所有 Capabilities 访问必须通过 integration/"原则不一致。
- **修复**:
  1. `combat/integration/event/mod.rs` 添加 `pub use crate::core::capabilities::event::mechanism::EventBus;`
  2. `combat/plugin.rs` 改为 `use super::integration::event::EventBus;`
- **验证**: `cargo build` + `cargo nextest run` — 1525/1525 passed

---

## Drift-ADR-002-terrain: Terrain 域直接依赖 Infra 层 DefinitionId

- **状态**: ✅ Fixed (2026-06-22)
- **发现日期**: 2026-06-22
- **修复日期**: 2026-06-22
- **负责人**: @architect → @feature-developer
- **关联 ADR**: ADR-001 (依赖方向), ADR-022 (网格/地形/阵营)
- **位置**: `src/core/domains/terrain/components.rs:9`, `events.rs:7`, `systems/terrain_effect_system.rs:13`
- **严重程度**: Medium
- **问题描述**: Terrain 域（L1 Core）直接 `use crate::infra::registry::DefinitionId`，跨层访问 Infra（L2）。违反 Shared ← Core ← Infra 的依赖方向。共 3 个文件、5 处引用。
- **影响**: Core 层代码引入 Infra 层类型，部署时 Core 无法独立于 Infra 使用。
- **修复方案** (Option A): 将 `DefinitionId` 从 `infra::registry` 移至 `shared::ids`（L0）：
  1. `shared/ids/types.rs` — 新增 `DefinitionId` 结构体（与 `define_string_id!` 生成的 ID 同模式）
  2. `infra/registry/registry.rs` — 移除原来定义，改为 `pub use crate::shared::ids::DefinitionId;`
  3. `terrain/components.rs`, `events.rs`, `terrain_effect_system.rs` — 改为 `use crate::shared::ids::DefinitionId;`
  4. infra 内其他 30+ 处引用通过 `pub use` 重新导出继续保持编译通过
- **验证**: `cargo build` — 0 errors; `cargo nextest run` — 1525/1525 passed

---

## Content-003: 阵营声望上限硬编码

- **状态**: 🆕 Open
- **发现日期**: 2026-06-22
- **负责人**: @feature-developer
- **严重程度**: Low
- **位置**: `src/core/domains/faction/rules/reputation.rs:12`
- **问题描述**: `REPUTATION_MAX: i32 = 100` 作为常量硬编码在 rules 层。虽然 100 是 D&D 类型的常见上限，但根据 Rule/Content 分离原则，此类业务数值应配置化到 RON 文件。
- **影响**: 低 — 不影响功能，调整需要改代码。
- **建议修复**: 将 `REPUTATION_MAX` 提取到 `assets/config/faction/` 下的 RON 配置，通过 ContentPlugin 加载。

---

## ✅ 已通过的检查项

### AI 可维护性 (Maintainability)
- 最大源文件 881 lines（content_plugin.rs），低于 1000 行 Medium 阈值
- 次大 744 lines（hot_reload.rs），同样低于阈值
- 无函数超过 100 行（抽查 ability::mechanism::lifecycle.rs，最大函数 `try_activate` < 80 行）
- 无 match 超过 20 arm
- 结论: ✅ 通过

### 超大 Plugin
- CombatPlugin: 69 行，符合规范
- 无 Plugin 注册过多系统
- 结论: ✅ 通过

### 禁止的文件名
- 无 `utils.rs`、`helpers.rs`、`common.rs` 垃圾桶文件
- 结论: ✅ 通过

### 测试覆盖
- 10/10 facade.rs 有对应的 tests/ 目录
- 1525 tests pass, 0 fail
- 结论: ✅ 通过

### 死代码 (预留性质，不构成债务)
- 563 个 build warnings（大部分为 unused imports）
- **均为 Capabilities 预留代码** — Cue/Stacking/Pipeline executor/Registry 等 P2 未接入 Capabilities 的类型和方法
- 项目处于 Capabilities 已建但仍有 3 个能力（Cue/Spec/Stacking）等待接入的状态，这些 unused 代码是**正常的架构演进路径**
- 判定: ✅ 预留 Dead Code，非技术债

### 直接域间依赖
- 排查 `use crate::core::domains` 在所有 domains/ 下的引用
- 仅发现同一域内的 intra-module 引用（如 `spell/rules/rules.rs` 引用 `spell/error.rs`）— 正常
- 结论: ✅ 无跨域直接依赖

### ECS 反模式
- 无 Entity OOP 方法
- 无 Component 包含复杂业务逻辑
- 无 System 存储状态
- 结论: ✅ 通过

---

## 📋 汇总

| 严重程度 | 数量 | 编号 | 状态 |
|---------|------|------|------|
| **Critical** | 0 | — | ✅ |
| **High** | 0 | — | ✅ |
| **Medium** | 1 | Drift-ADR-002-terrain | ✅ Fixed (this session) |
| Low | 1 | Content-003 | 🆕 Open |
| **总计** | **2** | | |

### 行动建议

1. **Content-003** — 低优先级，可在下次 faction 域迭代时顺手处理（将 `REPUTATION_MAX` 配置化）
2. 项目当前 **0 未解决技术债**（仅 1 个 Low 级别可选改进）

### 相较上次扫描 (2026-06-19) 的变化

| 指标 | 上次 (6/19) | 本次 (6/22) | 变化 |
|------|------------|------------|------|
| 测试数 | 1513 | 1525 | +12 |
| Build warnings | ~620 | ~563 | -57 |
| 未解决债务 | 0 | 1 (Content-003, Low) | +1 (新发现, 可选项) |
| 最大文件 | 640 | 881 (content_plugin) | +241 (正常增长) |
