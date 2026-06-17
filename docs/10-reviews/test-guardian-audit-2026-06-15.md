---
id: 10-reviews.test-guardian-audit-2026-06-15
title: Test Guardian 全量测试审查报告
status: active
owner: test-guardian
created: 2026-06-15
updated: 2026-06-15
tags:
  - review
  - testing
---

# Test Guardian 全量测试审查报告

**审查人**：@test-guardian
**审查范围**：`src/` 全部代码 + `docs/05-testing/` 规范
**审查标准**：`test-spec.md` v4.0 + `testing-rules.md` v2.0
**审查日期**：2026-06-15

---

## 1. 总体评估

### Coverage Report: 🟥 FAIL

| 维度 | 状态 | 说明 |
|------|------|------|
| 测试存在性 | ⚠️ 部分存在 | 558+ `#[test]` 函数存在，但全部为内联测试 |
| 测试组织结构 | 🟥 严重违反 | 全部测试内嵌于源码，违反"禁止 `#[cfg(test)] mod tests`"铁律 |
| 领域测试覆盖 | 🟥 完全缺失 | 16 个业务域（domains/）零测试 |
| 不变量测试 | 🟥 完全缺失 | 无 `invariant/` 目录，7 个核心不变量无测试覆盖 |
| 跨领域测试 | 🟥 完全缺失 | 根 `tests/` 目录不存在 |
| 测试命名规范 | 🟥 违反 | 全部使用英文命名，未使用中文描述预期行为 |
| 标准测试数据 | 🟥 未使用 | 未使用 Unit_001/002/003 Builder 模式 |
| 测试工具库 | 🟥 空壳 | `shared/testing/mod.rs` 仅含 TODO 注释 |
| Testbed 沙盒 | 🟥 完全缺失 | `testbeds/` 目录不存在 |

---

## 2. 详细审查发现

### 2.1 🟥 致命问题：内联测试泛滥

**违反条款**：test-spec.md §1.0（🟥 宪法 12.2）

**现状**：全部 41 个含测试的文件均使用 `#[cfg(test)] mod tests` 内联模式，共 558+ 个测试函数。

**受影响文件**（按测试数量降序）：

| 文件 | 测试数 | 能力模块 |
|------|--------|----------|
| `effect/mechanism/lifecycle.rs` | 50+ | Effect 生命周期 |
| `targeting/mechanism/selector.rs` | 30+ | Targeting 选择器 |
| `trigger/mechanism/evaluator.rs` | 15+ | Trigger 评估器 |
| `stacking/mechanism/decider.rs` | 20+ | Stacking 堆叠决策 |
| `runtime/replay/foundation/values.rs` | 20+ | Replay 值对象 |
| `runtime/command/foundation/types.rs` | 15+ | Command 类型 |
| `runtime/pipeline/mechanism/executor.rs` | 10+ | Pipeline 执行器 |
| `runtime/registry/mechanism/validator.rs` | 10+ | Registry 验证器 |
| `attribute/mechanism/lifecycle.rs` | 7 | Attribute 生命周期 |
| `modifier/mechanism/lifecycle.rs` | 6 | Modifier 生命周期 |
| `tag/mechanism/query.rs` | 9 | Tag 查询 |
| `tag/mechanism/lifecycle.rs` | 6 | Tag 生命周期 |
| `condition/mechanism/evaluator.rs` | 10+ | Condition 评估器 |
| `cue/mechanism/dispatch.rs` | 10+ | Cue 分发 |
| `cue/foundation/values.rs` | 10+ | Cue 值对象 |
| `cue/foundation/types.rs` | 10+ | Cue 类型 |
| `aggregator/mechanism/pipeline.rs` | 5+ | Aggregator 管线 |
| `aggregator/mechanism/lifecycle.rs` | 3+ | Aggregator 生命周期 |
| `ability/mechanism/lifecycle.rs` | 10+ | Ability 生命周期 |
| `event/mechanism/bus.rs` | 5+ | Event 总线 |
| `execution/mechanism/calculator.rs` | 10+ | Execution 计算器 |
| `gameplay_context/mechanism/builder.rs` | 5+ | GameplayContext 构建器 |
| `gameplay_context/foundation/values.rs` | 3+ | GameplayContext 值对象 |
| `shared/random/mod.rs` | 5 | 随机数 |
| `shared/error/mod.rs` | 5 | 错误处理 |
| `shared/time/mod.rs` | 8 | 时间管理 |
| `runtime/scheduler/mechanism/executor.rs` | 5+ | Scheduler 执行器 |
| `runtime/scheduler/foundation/values.rs` | 5+ | Scheduler 值对象 |
| `runtime/scheduler/foundation/types.rs` | 5+ | Scheduler 类型 |
| `runtime/replay/mechanism/recorder.rs` | 5+ | Replay 录制器 |
| `runtime/replay/mechanism/player.rs` | 5+ | Replay 播放器 |
| `runtime/replay/foundation/types.rs` | 5+ | Replay 类型 |
| `runtime/registry/foundation/values.rs` | 5+ | Registry 值对象 |
| `runtime/registry/foundation/types.rs` | 5+ | Registry 类型 |
| `runtime/pipeline/foundation/types.rs` | 5+ | Pipeline 类型 |
| `runtime/pipeline/foundation/values.rs` | 3+ | Pipeline 值对象 |
| `runtime/command/foundation/values.rs` | 5+ | Command 值对象 |
| `runtime/command/mechanism/dispatch.rs` | 5+ | Command 分发 |
| `stacking/foundation/types.rs` | 5+ | Stacking 类型 |
| `stacking/foundation/values.rs` | 1+ | Stacking 值对象 |

**修复方案**：按 Feature First 原则，将测试从源码中提取到 `<domain>/tests/` 目录结构。这是 Phase D（测试完善）的核心任务。

---

### 2.2 🟥 致命问题：领域测试完全缺失

**违反条款**：testing-rules.md 规则 2（每条领域规则对应单元测试）

**现状**：16 个业务域全部没有测试：

| 领域 | 模块路径 | 测试状态 |
|------|----------|----------|
| combat | `core/domains/combat/` | 🟥 无测试 |
| tactical | `core/domains/tactical/` | 🟥 无测试 |
| spell | `core/domains/spell/` | 🟥 无测试 |
| inventory | `core/domains/inventory/` | 🟥 无测试 |
| party | `core/domains/party/` | 🟥 无测试 |
| progression | `core/domains/progression/` | 🟥 无测试 |
| quest | `core/domains/quest/` | 🟥 无测试 |
| faction | `core/domains/faction/` | 🟥 无测试 |
| economy | `core/domains/economy/` | 🟥 无测试 |
| crafting | `core/domains/crafting/` | 🟥 无测试 |
| terrain | `core/domains/terrain/` | 🟥 无测试 |
| summon | `core/domains/summon/` | 🟥 无测试 |
| reaction | `core/domains/reaction/` | 🟥 无测试 |
| narrative | `core/domains/narrative/` | 🟥 无测试 |
| camp_rest | `core/domains/camp_rest/` | 🟥 无测试 |

**影响**：Phase D（测试完善）开始前，必须为每个业务域补充测试。当前无任何领域规则被测试覆盖。

---

### 2.3 🟥 致命问题：不变量测试完全缺失

**违反条款**：test-spec.md §5 Invariant Test（最高价值）

**现状**：7 个核心不变量无一有测试：

| 不变量 | 说明 | 测试状态 |
|--------|------|----------|
| Tag bit 唯一 | 同一 Tag 不能在位掩码中重复设置 | 🟥 无测试 |
| Buff 不重复叠加 | 同源同类型 Buff 不会无限堆叠 | 🟥 无测试 |
| Effect 不修改不存在属性 | Effect 引用的 AttributeId 必须已注册 | 🟥 无测试 |
| HP 永远 >= 0 | HP 计算结果不能为负 | 🟥 无测试 |
| Modifier 不改变基础值 | Modifier 只影响聚合后的当前值 | 🟥 无测试 |
| 回合先攻排序稳定 | 同先攻值的单位顺序确定 | 🟥 无测试 |
| 技能消耗原子性 | 消耗失败时不产生部分效果 | 🟥 无测试 |

**影响**：不变量测试是架构稳定性的最后防线，完全缺失意味着架构完整性无法验证。

---

### 2.4 🟥 严重问题：测试命名违反规范

**违反条款**：test-spec.md §2.5（测试命名规范）

**现状**：所有测试使用英文 snake_case 命名，如：
- `unit_001_instant_effect_starts_applying`
- `unit_013_apply_duplicate_effect_rejected`
- `same_seed_produces_same_sequence`

**规范要求**：测试函数名用**中文**描述预期行为，技术术语保留英文。

**正确示例**：
- `fn 即时效果立即应用()`
- `fn 重复效果被拒绝()`
- `fn 相同种子产生相同序列()`

**受影响范围**：全部 558+ 个测试函数。

---

### 2.5 🟥 严重问题：未使用标准测试数据

**违反条款**：test-spec.md §7.1 + test-guardian.md 标准测试数据

**规范要求**：
- Unit_001（战士）：HP=100, ATK=30, DEF=10, SPD=10, Range=1
- Unit_002（法师）：HP=80, ATK=40, DEF=5, SPD=12, Range=3
- Unit_003（坦克）：HP=150, ATK=20, DEF=20, SPD=5, Range=1

**现状**：各测试文件自定义 helper 函数构造测试数据，如：
- `make_instant_effect(id)`
- `make_duration_effect(id, turns)`
- `make_attr(id, category, default, min, max, deps)`
- `make_container()`

**影响**：测试数据分散，业务规则变更时维护成本高。

---

### 2.6 🟥 严重问题：测试工具库为空壳

**违反条款**：testing-rules.md 规则 6（shared/testing 提供公共工具）

**现状**：`src/shared/testing/mod.rs` 内容：
```rust
//! 测试构建工具
//!
//! 提供对领域无关的测试辅助函数和 mock 构造器。
//! 领域内聚测试参见各领域内的 `tests/` 模块。

// TODO: 实现 TestApp / TestWorld 构建器
```

**规范要求**：
- `shared/testing/fixtures.rs` — Builder 模式测试数据
- `shared/testing/deterministic.rs` — 确定性 RNG
- `shared/testing/assertions.rs` — 自定义断言

**影响**：无公共测试工具，每个测试文件重复造轮子。

---

### 2.7 🟥 严重问题：跨领域测试完全缺失

**违反条款**：test-spec.md §4（Test Pyramid）

**规范要求根 `tests/` 目录结构：
```
tests/
├── battle_flow/     # 完整战斗流程
├── save_load/       # 存档/读档完整性
├── regression/      # 回归测试（历史Bug复现）
├── replay/          # 回放确定性
├── golden/          # 金文件对比
├── simulation/      # 战斗模拟与数值平衡
├── performance/     # 性能回归
└── e2e/             # 端到端测试
```

**现状**：`tests/` 目录不存在。

---

### 2.8 🟥 严重问题：Testbed 沙盒完全缺失

**违反条款**：testing-rules.md 测试沙盒定义

**规范要求**：
- `testbeds/battle_simulator/`
- `testbeds/skill_playground/`
- `testbeds/ai_debug_arena/`
- `testbeds/balance_workbench/`
- `testbeds/replay_validator/`

**现状**：`testbeds/` 目录不存在。

---

## 3. 测试质量抽样分析

### 3.1 合规测试（少数）

`shared/random/mod.rs` 中的测试相对规范：
- ✅ 确定性（使用固定 Seed）
- ✅ 测试行为而非实现
- ❌ 但使用英文命名
- ❌ 但为内联测试

### 3.2 典型问题测试

`effect/mechanism/lifecycle.rs` 中的测试：
- ❌ 内联测试
- ❌ 英文命名
- ❌ 自定义 helper 而非标准 fixture
- ⚠️ 部分测试验证实现细节（如 `assert_eq!(container.count(), 1)`）

`attribute/mechanism/lifecycle.rs` 中的测试：
- ❌ 内联测试
- ❌ 英文命名
- ❌ 验证实现细节（如 `assert_eq!(reg.definitions.len(), 1)`）

---

## 4. 测试矩阵

### 4.1 能力模块测试覆盖

| 能力模块 | 单元测试 | 集成测试 | 不变量测试 | 测试数据 | 状态 |
|----------|----------|----------|------------|----------|------|
| ability | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| aggregator | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| attribute | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| condition | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| cue | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| effect | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| event | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| execution | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| gameplay_context | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| modifier | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| runtime/command | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| runtime/pipeline | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| runtime/registry | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| runtime/replay | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| runtime/scheduler | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| spec | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| stacking | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| tag | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| targeting | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| trigger | ⚠️ 内联 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |

### 4.2 业务域测试覆盖

| 业务域 | 单元测试 | 集成测试 | 不变量测试 | 测试数据 | 状态 |
|--------|----------|----------|------------|----------|------|
| combat | 🟥 缺失 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| tactical | 🟥 缺失 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| spell | 🟥 缺失 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| inventory | 🟥 缺失 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| party | 🟥 缺失 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| progression | 🟥 缺失 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| quest | 🟥 缺失 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| faction | 🟥 缺失 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| economy | 🟥 缺失 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| crafting | 🟥 缺失 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| terrain | 🟥 缺失 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| summon | 🟥 缺失 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| reaction | 🟥 缺失 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| narrative | 🟥 缺失 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |
| camp_rest | 🟥 缺失 | 🟥 缺失 | 🟥 缺失 | 🟥 无 fixture | FAIL |

### 4.3 共享层测试覆盖

| 模块 | 测试状态 | 说明 |
|------|----------|------|
| shared/random | ⚠️ 内联 | 5 个测试，英文命名 |
| shared/error | ⚠️ 内联 | 5 个测试，英文命名 |
| shared/time | ⚠️ 内联 | 8 个测试，英文命名 |
| shared/collections | 🟥 缺失 | 无测试 |
| shared/hashing | 🟥 缺失 | 无测试 |
| shared/ids | 🟥 缺失 | 无测试 |
| shared/math | 🟥 缺失 | 无测试 |
| shared/path | 🟥 缺失 | 无测试 |
| shared/traits | 🟥 缺失 | 无测试 |
| shared/validation | 🟥 缺失 | 无测试 |

---

## 5. 修复优先级

### P0 — 立即修复（阻断开发）

| # | 问题 | 修复方案 | 预估工时 |
|---|------|----------|----------|
| 1 | 内联测试泛滥 | 将 41 个文件的测试提取到 `<domain>/tests/unit/` | 3-5 天 |
| 2 | 测试命名违反规范 | 将 558+ 个测试函数重命名为中文 | 1-2 天 |
| 3 | shared/testing 为空 | 实现 TestApp / TestWorld Builder + fixtures | 1-2 天 |

### P1 — Phase D 前完成

| # | 问题 | 修复方案 | 预估工时 |
|---|------|----------|----------|
| 4 | 不变量测试缺失 | 创建 7 个 invariant 测试文件 | 2-3 天 |
| 5 | 领域测试缺失 | 为 16 个业务域补充测试 | 5-8 天 |
| 6 | 标准测试数据未使用 | 创建 Unit_001/002/003 Builder | 1 天 |

### P2 — Phase D 中完成

| # | 问题 | 修复方案 | 预估工时 |
|---|------|----------|----------|
| 7 | 跨领域测试缺失 | 创建根 tests/ 目录结构 | 2-3 天 |
| 8 | Testbed 沙盒缺失 | 创建 5 个 Testbed 工具 | 3-5 天 |
| 9 | 共享层测试缺失 | 补充 shared/ 模块测试 | 1-2 天 |

---

## 6. 交接建议

| 发现 | 建议调用角色 | 原因 |
|------|--------------|------|
| 领域规则文档未定义测试场景 | @domain-designer | 需要补充领域规则的测试用例定义 |
| 测试架构需要调整 | @architect | 内联测试提取涉及模块边界调整 |
| 需要为 Phase D 制定详细测试计划 | @test-guardian | 自身职责，需要制定分领域测试计划 |

---

## 7. 结论

当前代码库的测试状况**严重不合规**，主要问题：

1. **结构性违规**：全部测试内嵌于源码，违反"禁止 `#[cfg(test)] mod tests`"铁律
2. **覆盖性缺失**：16 个业务域零测试，7 个核心不变量零测试
3. **规范性违反**：测试命名、测试数据、测试工具均不符合规范
4. **基础设施缺失**：无跨领域测试目录、无 Testbed 沙盒、无测试工具库

**建议**：Phase D（测试完善）应作为最高优先级任务，在任何新功能开发前完成测试基础设施建设。

---

**审查完成时间**：2026-06-15
**下次审查建议**：Phase D 完成后
