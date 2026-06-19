---
id: 11-refactor.tech-debt-batch3-4
title: 技术债扫描 — 第 3+4 批领域（法术 / 反应 / 任务 / 经济 / 制作 / 召唤）
status: active
owner: refactor-guardian
created: 2026-06-18
dimensions:
  - architecture-drift
  - abstraction-leakage
  - ai-maintainability
  - test-debt
  - content-debt
  - lifecycle-management
priority: high
---

# 技术债扫描 — 第 3+4 批领域

## 扫描范围

骨架阶段实现的 6 个领域：

| 领域 | 文件数 | 代码行数 | 骨架完成度 |
|------|--------|----------|------------|
| 法术 | ~15 | ~1,200 | 85% |
| 反应 | ~15 | ~1,100 | 80% |
| 任务 | ~15 | ~1,000 | 70% |
| 经济 | ~15 | ~950 | 75% |
| 制作 | ~15 | ~800 | 65% |
| 召唤 | ~15 | ~700 | 70% |

---

## 维度一：架构漂移

### 评估：绿色（低漂移）

**通过项：**
- Feature First 结构一致遵循 — 无按技术类型拆分的目录
- 无跨领域直接导入 — 所有内部引用使用 `super::super::*`
- 领域事件使用 Observer 模式 — 无手动系统链式调用
- 插件注册集中在 `core_plugin.rs` 中
- 无 Capabilities 类型逻辑泄漏到领域代码

**轻微漂移（1 项）：**

| # | 严重程度 | 文件 | 问题 |
|---|----------|------|------|
| D1-1 | 低 | `economy/systems/economy_system.rs:104` | `use super::super::components::CurrencyType;` 放置在文件底部（第 104 行），而非与其他导入语句一起放在顶部。违反 Rust 导入规范。 |

---

## 维度二：抽象泄漏

### 评估：绿色（干净）

**所有领域通过：**
- 规则模块为纯函数，无 ECS 依赖 — 可独立于系统测试
- 每个领域定义了事件，用于所有跨领域通信
- 组件为纯数据，无业务逻辑（仅有访问器/辅助方法）
- 系统代码中无直接访问其他领域 Component
- 无手动 `Query` 参数跨领域访问

**未发现问题。** 规则逻辑与 ECS 系统之间的分离保持良好。

---

## 维度三：AI 可维护性

### 评估：黄色（中度关注）

**降低 AI 理解和未来开发速度的项目：**

| # | 严重程度 | 领域 | 问题 |
|---|----------|------|------|
| M1-1 | **中等** | 任务 | 空函数 `accept_quest()`（quest_system.rs:15-18）— 在 mod.rs 中注册，从未被调用，函数体为空。对未来的阅读者造成困惑。 |
| M1-2 | **中等** | 任务 | `on_advance_objective` 中有一个空的 `if entry.all_objectives_completed() { }` 代码块（第 64-66 行）— 看起来像 Bug 的空占位符。 |
| M1-3 | **中等** | 召唤 | 被注释掉的 `on_caster_died` 函数（summon_system.rs:53-60）— 死代码未被移除。 |
| M1-4 | **中等** | 制作 | 空 Observer `on_craft_item`（crafting_system.rs:17-25）— 函数体仅包含注释和一个已触发的事件，无实际逻辑。 |
| M1-5 | **低** | 所有领域 | 委派到 Component 方法的单行规则函数：`can_react(state)` → `state.can_react()`、`has_free_summon_slot(manager)` → `manager.has_free_slot()`、`check_upgrade_limit(level)` → `level.can_upgrade()`。这些增加认知负担而无抽象价值。当委派不提供映射/别名时，建议内联或移除。 |

---

## 维度四：测试债务

### 评估：黄色（部分缓解）— 2026-06-18 更新

这是前期最大的债务类别。项目自身的 `docs/05-testing/test-spec.md` 要求"领域内聚四层测试"（单元/集成/不变量/夹具）。

**2026-06-18 进展：** T1 已解决 — 已为全部 6 个领域添加集成测试（43 个测试用例），所有测试通过：

| 领域 | 测试文件 | 用例数 | 状态 |
|------|----------|--------|------|
| 法术 | `cast_flow_test.rs` | 10 | ✅ 全部通过 |
| 反应 | `reaction_queue_test.rs` | 7 | ✅ 全部通过 |
| 任务 | `quest_lifecycle_test.rs` | 7 | ✅ 全部通过 |
| 经济 | `transaction_test.rs` | 6 | ✅ 全部通过 |
| 制作 | `craft_flow_test.rs` | 6 | ✅ 全部通过 |
| 召唤 | `summon_flow_test.rs` | 7 | ✅ 全部通过 |

这些集成测试使用 Bevy `App` + `add_plugins(DomainPlugin)` + `world.trigger()` 模式，验证 Observer 从事件触发到组件状态变更的完整链路。

| # | 严重程度 | 领域 | 缺口 |
|---|----------|------|------|
| T1 | ~~严重~~ → **已解决** | 全部 6 个 | ~~无集成测试~~ ✅ 43 个集成测试已添加并全部通过。 |
| T2 | **严重** | 全部 6 个 | **不变量测试仍缺失。** 领域规则文件记录了不变量（例如"不变量 3.1：货币非负"），但没有任何一个被显式测试。 |
| T3 | **高** | 反应、任务、经济、制作、召唤 | ~~只有法术有单元测试~~ → 集成测试已覆盖所有领域，但**单元测试**仍有缺口。 |
| T4 | **低** | 法术 | 法术单元测试使用 `#[cfg(test)] mod { ... }` 包装器 — 与项目中其他领域使用扁平 `#[test]` 函数的规范不一致。 |

**建议：** 集成测试已建立，可以在此基础上添加**不变量测试**（T2）和补充**单元测试**（T3）。T1 作为最高优先级债务已被清除。

---

## 维度五：内容债务

### 评估：黄色（中度）

| # | 严重程度 | 领域 | 问题 |
|---|----------|------|------|
| C1 | **中等** | 经济 | `on_purchase_request` 创建 `Price { ..., reputation_modifier: 1.0, supply_modifier: 1.0, stolen_modifier: 1.0 }` — 全部硬编码。配置加载激活后应使用 `EconomyConfig` 中的值。 |
| C2 | **中等** | 经济 | 商店库存管理被简化掉并带有一个 TODO（"待 ShopEntity 挂接后完善"）。两个观察者函数都声明了 `shop_query: Query<&mut ShopInstance>` 但从未使用。 |
| C3 | **中等** | 制作 | `perform_skill_check()`（rules.rs:94-97）是一个桩函数：`skill_bonus > 0`。无 RNG 集成，无 DC 比较。 |
| C4 | **低** | 经济 | `reputation_to_price_modifier("Hated")` 返回 `f32::MAX` 作为哨兵值。这很脆弱 — 任何与 `f32::MAX` 的乘法都会导致 NaN/inf。建议使用 `Option<f32>` 或专门的枚举变体。 |
| C5 | **低** | 任务 | `on_advance_objective` 触发 `ObjectiveCompleted` 但注册了无事件消费者。奖励发放未实现。 |

---

## 维度六：生命周期管理

### 评估：黄色（中度）

| # | 严重程度 | 领域 | 问题 |
|---|----------|------|------|
| L1 | **中等** | 法术 | `resources.rs` 仅包含一条注释（"SpellConfig 已在 components.rs 中定义并注册为 Resource"）— 作为模块结构占位符的空文件。应删除或明确文档说明为有意为之。 |
| L2 | **中等** | 全部 6 个 | **无任何 Component 的生命周期文档。** 哪个系统创建每个 Component？哪个清理它？在什么条件下？这使得调试生命周期 Bug 非常耗时。 |
| L3 | **低** | 全部 6 个 | TODO 标记没有跟踪状态、负责人或问题引用。当前 TODO 包括：spell_system.rs（2 个关于 SpellDefRegistry 的 TODO）、summon_system.rs（关于 UnitDied 的 TODO）、economy_system.rs（关于 ShopEntity 的 TODO）。 |
| L4 | **低** | 全部 6 个 | 所有资源都使用 `Default::default()` 初始化。尚无运行时配置加载。`EconomyConfig`、`SummonConfig`、`SpellConfig` 等资源具有硬编码的 Default 实现值。 |

---

## 优先级行动计划

| 优先级 | 项 | 领域 | 工作量 | 影响 |
|--------|------|------|--------|------|
| **P0** | ~~T1：添加集成测试~~ ✅ **已完成**（43 个测试，全部通过） | 全部 6 个 | 已解决 | 🟥 回归安全 |
| **P0** | T2：添加不变量测试 | 全部 6 个 | 2-3 天 | 🟥 验证领域规则的关键 |
| **P1** | L2：为所有 Component 添加生命周期文档 | 全部 6 个 | 1-2 天 | 🟧 防止生命周期 Bug |
| **P2** | C1 + C2：接入经济配置和商店库存 | 经济 | 0.5 天 | 🟨 解锁经济功能 |
| **P3** | M1-1、M1-2、M1-3、M1-4：清理骨架死代码 | 任务、召唤、制作 | 0.5 天 | 🟩 提升代码清晰度 |
| **P4** | D1-1、M1-5：修复规范 + 内联琐碎规则 | 全部 6 个 | 0.5 天 | 🟩 内务清理 |
| **P5** | L3：将 TODO 转换为跟踪项 | 全部 6 个 | 0.5 天 | 🟩 过程债务 |

## 库存：优先"删而非包"

有一项适合直接删除而非包裹：

- **L1（空的 resources.rs）**：直接删除文件而非添加更多内容。模块结构规范应允许在无内容时省略文件。或者将其模块声明合并到 mod.rs 中并添加注释。

---

*由 @refactor-guardian 生成，2026-06-18*
*建议下次扫描时间：在集成测试建立后，或添加新领域之前。*
