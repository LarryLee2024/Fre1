---
id: 01-architecture.00-foundation.ADR-045
title: "ADR-045: 模块可见性策略（可见性宪法）"
status: Accepted
owner: architect
created: 2026-06-17
updated: 2026-06-17
tags:
  - architecture
  - visibility
  - module
  - encapsulation
  - pub-crate
---

# ADR-045: 模块可见性策略（可见性宪法）

## 状态

**Accepted** 

## 背景

对于 Bevy 0.19+ / DDD / 14 领域 / 50万~100万行 / Workspace 多 Crate / AI 大量参与开发的项目，最怕的不是测试，而是**权限边界失控**。
最后整个 Workspace 全是公开 API。AI 根本不知道什么能调用，什么不能调用。

### 问题现状

当前项目存在以下可见性问题：

1. **过度暴露**：部分模块使用无条件 `pub mod`，导致实现细节在生产代码中也暴露为公共 API
2. **缺乏规范**：没有统一的可见性策略，开发者（包括 AI）随意使用 `pub`
3. **边界模糊**：领域间边界不清晰，可能导致耦合

## 引用的领域规则

- `docs/00-governance/ai-constitution-complete.md` §9 — 封装原则
- `docs/00-governance/coding-rules.md` — 最小可见性原则
- `docs/01-architecture/README.md` — DDD 三层 + 横切四层架构

## 决策

### 第一原则

**默认 private，不是 pub。**

能不用 pub 就不用 pub。

### 可见性优先级

```text
private        ★★★★★ 70%
pub(crate)     ★★★★★ 25%
pub(in path)   ★★★☆☆ 3%
pub            ★★☆☆☆ 2%
```

**目标比例**：

| 可见性 | 目标占比 | 说明 |
|--------|---------|------|
| `private` | 70% | 同文件、同模块、内部实现 |
| `pub(crate)` | 25% | 同领域共享、测试访问、crate 内协作 |
| `pub(in path)` | 3% | 仅用于层级边界约束（需 ADR 批准） |
| `pub` | 2% | 公共 API、跨 crate、Facade 模式 |

如果一个领域 `pub` 超过 20%，基本意味着领域边界已经开始腐化，需要重构。

### 可见性场景矩阵

| 场景 | 可见性 | 说明 |
|------|--------|------|
| 当前文件 | `private` | 辅助函数、内部状态机、公式计算 |
| 同模块 | `private` | 模块内部共享 |
| 同领域（capabilities/domains） | `pub(crate)` | 领域内部共享（foundation/mechanism/runtime） |
| 领域内 `tests/`（`src/<domain>/tests/`） | `pub(crate)` | 领域内测试（主 crate 的一部分，由 `#[cfg(test)] mod tests;` 声明） |
| 领域外 `tests/`（根 `tests/` 目录） | `pub` | 集成测试（独立 crate） |
| 同 Crate 不同模块 | `pub(crate)` | crate 内跨模块协作 |
| 跨 Crate | `pub` | 跨 crate 调用（如 Workspace 成员） |
| 跨领域调用 | Facade + `pub` | Anti-Corruption Layer 模式 |
| 测试工具 | `#[cfg(test)]` | 测试专用导出 |
| 测试共享库 | `test_utils` crate | Workspace 共享测试工具 |

### 项目实际结构

#### 测试结构（Feature First）

```
src/
├── shared/<module>/tests/           # L0 共享层测试
│   ├── mod.rs                       # #[cfg(test)] mod tests;
│   ├── unit/
│   └── ...
├── core/capabilities/<domain>/tests/ # L1 能力领域测试
│   ├── mod.rs                       # #[cfg(test)] mod tests;
│   ├── unit/
│   ├── invariant/
│   └── fixtures/
└── core/domains/<domain>/tests/      # L1 业务子系统测试
    ├── mod.rs                       # #[cfg(test)] mod tests;
    ├── unit/
    ├── invariant/
    ├── integration/
    └── fixtures/
```

**关键事实**：
- 测试文件在 `src/<domain>/tests/` 目录
- 由 `#[cfg(test)] mod tests;` 声明
- **是主 crate 的一部分**（非独立 crate）
- 编译时带 `--test` 标志
- 可以访问 `pub(crate)` 的模块

#### 能力领域结构（Capabilities）

```
src/core/capabilities/<domain>/
├── mod.rs                 # pub — 插件入口
├── plugin.rs              # pub — Plugin 实现（通过 pub use * 重导出）
├── events.rs              # pub — 领域事件定义
├── foundation/
│   ├── mod.rs             # pub(crate) — 基础类型入口
│   ├── types.rs           # pub(crate) — 类型定义
│   └── values.rs          # pub(crate) — 值对象
├── mechanism/
│   ├── mod.rs             # pub(crate) — 机制实现入口
│   ├── lifecycle.rs       # pub(crate) — 生命周期管理
│   ├── components.rs      # private — ECS 组件
│   └── systems.rs         # pub — ECS 系统
└── tests/
    ├── mod.rs             # #[cfg(test)] — 测试模块入口
    ├── unit/              # 单元测试
    ├── invariant/         # 不变量测试
    └── fixtures/          # 测试数据
```

#### 业务子系统结构（Domains）

```
src/core/domains/<domain>/
├── mod.rs                 # pub — 插件入口
├── plugin.rs              # pub — Plugin 实现
├── integration.rs         # pub — 域间交互唯一入口
├── components/            # private — 业务组件
├── systems/               # pub — 业务系统
├── services/              # pub(crate) — 领域服务
└── tests/
    ├── mod.rs             # #[cfg(test)] — 测试模块入口
    ├── unit/
    ├── invariant/
    ├── integration/
    └── fixtures/
```

#### 共享层结构（Shared）

```
src/shared/
├── mod.rs                 # pub — 共享层入口
├── ids/                   # pub(crate) — 强类型 ID
│   ├── mod.rs
│   ├── types.rs
│   └── tests/
├── error/                 # private — 错误上下文工具
├── math/                  # pub(crate) — 纯数学工具
├── random/                # pub(crate) — 确定性随机数
├── time/                  # pub(crate) — GameTime, TurnCount
├── collections/           # private — 通用集合扩展
├── hashing/               # private — 非加密高速哈希
├── validation/            # pub(crate) — 链式校验器
├── testing/               # #[cfg(test)] — 测试构建工具
├── traits/                # pub(crate) — 横切能力抽象
├── prelude/               # pub — 统一导出
└── path/                  # pub(crate) — 路径工具
```

#### 技术实现层（Infra）

```
src/infra/
├── mod.rs                 # pub — Infra 层入口
├── registry/              # pub(crate) — ID 注册与热重载
├── pipeline/              # pub(crate) — 管线执行引擎
├── replay/                # pub(crate) — 回放系统
├── save/                  # pub(crate) — 存档系统
└── input/                 # pub(crate) — 输入抽象
```

#### 横切层

```
src/app/                   # pub — 启动装配层（Composition Root）
src/content/               # pub(crate) — 内容桥接层
src/tools/                 # #[cfg(debug)] — 开发工具层
src/modding/               # pub — Mod 扩展层
```

#### L1: Core（领域规则层）

##### Capabilities（能力领域）

```text
src/core/capabilities/<domain>/
├── mod.rs                 # pub — 插件入口
├── plugin.rs              # pub — Plugin 实现
├── integration.rs         # pub — 域间交互唯一入口
├── events.rs              # pub — 领域事件定义
├── foundation/
│   ├── mod.rs             # pub(crate) — 基础类型
│   ├── types.rs           # pub(crate) — 类型定义
│   └── values.rs          # pub(crate) — 值对象
├── mechanism/
│   ├── mod.rs             # pub(crate) — 机制实现
│   ├── lifecycle.rs       # pub(crate) — 生命周期管理
│   ├── components.rs      # private — ECS 组件
│   └── systems.rs         # pub — ECS 系统
└── tests/
    ├── unit/              # 单元测试
    ├── integration/       # 集成测试
    └── invariant/         # 不变量测试
```

##### Domains（业务子系统）

```text
src/core/domains/<domain>/
├── mod.rs                 # pub — 插件入口
├── plugin.rs              # pub — Plugin 实现
├── integration.rs         # pub — 域间交互唯一入口
├── components/            # private — 业务组件
├── systems/               # pub — 业务系统
├── services/              # pub(crate) — 领域服务
└── tests/
```

#### L2: Infra（技术实现层）

```text
src/infra/
├── registry/              # pub(crate) — ID 注册与热重载
├── pipeline/              # pub(crate) — 管线执行引擎
├── replay/                # pub(crate) — 回放系统
├── save/                  # pub(crate) — 存档系统
└── input/                 # pub(crate) — 输入抽象
```

#### 横切层

```text
src/app/                   # pub — 启动装配层（Composition Root）
src/content/               # pub(crate) — 内容桥接层
src/tools/                 # #[cfg(debug)] — 开发工具层
src/modding/               # pub — Mod 扩展层
```

### 公共 API 设计模式

#### Facade 模式（推荐）

跨领域访问不暴露实现细节，通过 Facade 提供稳定接口：

```rust
// ❌ 错误：直接暴露内部结构
pub struct Character {
    pub hp: i32,
    pub mp: i32,
    pub skills: Vec<Skill>,
}

// ✅ 正确：Facade 模式
pub struct CharacterFacade;

impl CharacterFacade {
    pub fn get_hp(&self) -> i32 { ... }
    pub fn apply_damage(&self, amount: i32) { ... }
}

// 内部实现
pub(crate) struct Character {
    hp: i32,
    mp: i32,
    skills: Vec<Skill>,
}
```

#### integration.rs 模式（领域间唯一入口）

每个 Domain 必须有且仅有一个 `integration.rs` 作为与 Capabilities 交互的唯一入口：

```rust
// src/core/domains/combat/integration.rs

use crate::core::capabilities::attribute::mechanism::lifecycle::AttributeRegistry;

pub fn calculate_damage(attacker: &Entity, target: &Entity, registry: &AttributeRegistry) -> i32 {
    // 域间交互逻辑
}
```

### 测试可见性策略

#### 领域内测试（src/domain/tests/）

测试文件在 `src/<layer>/<domain>/tests/` 目录，是主 crate 的一部分（由 `#[cfg(test)] mod tests;` 声明）。

```
src/core/capabilities/attribute/
├── mod.rs           # #[cfg(test)] mod tests;
├── foundation/
├── mechanism/
└── tests/           # 主 crate 的一部分
    ├── mod.rs
    ├── unit/
    ├── invariant/
    └── fixtures/
```

- 可以访问 `pub(crate)` 的模块
- 不能访问 `private` 的模块
- 推荐使用 `pub(crate)` 而非 `pub` 访问内部模块

```rust
// src/core/capabilities/attribute/tests/unit/lifecycle_test.rs

use crate::core::capabilities::attribute::mechanism::lifecycle::AttributeRegistry;

#[test]
fn test_attribute_registration() {
    let registry = AttributeRegistry::default();
    // ...
}
```

**关键事实**：
- 测试文件在 `src/<layer>/<domain>/tests/` 目录
- 由 `#[cfg(test)] mod tests;` 声明
- **是主 crate 的一部分**（非独立 crate）
- 编译时带 `--test` 标志
- 可以访问 `pub(crate)` 的模块

#### 集成测试（根 tests/）

根 `tests/` 目录下的测试是独立 crate，只能访问 `pub` 的项：

```rust
// tests/integration/battle_flow.rs

use fre::core::capabilities::attribute::mechanism::AttributeRegistry;

#[test]
fn test_battle_flow() {
    // 只能通过 pub 接口访问
}
```

#### 测试专用导出

对于测试需要但生产代码不需要的工具，使用 `#[cfg(test)]`：

```rust
// #[cfg(any(test, feature = "test-utils"))]
#[cfg(test)]
pub mod test_support {
    pub fn create_test_character() -> Character { ... }
    pub fn create_test_battle() -> Battle { ... }
}
```

### pub(in path) 使用规范

`pub(in path)` 仅用于**层级边界约束**，不用于领域间封装。

#### 允许使用的场景

分层架构中的层级边界控制：

```rust
// ability/
// ├── l0_atom/
// ├── l1_base/
// ├── l2_execution/
// ├── l3_entity/
// └── facade/

// l1_base
pub(in crate::l2_execution) struct AbilityContext;

// 这样 l2_execution 可以访问
// 而 l3_entity、facade 不能访问
```

#### 禁止使用的场景

领域间不要用 `pub(in path)`，用 Facade 模式：

```rust
// ❌ 错误：领域间用 pub(in path)
pub(in crate::ability) struct CharacterData;

// ✅ 正确：Facade 模式
pub struct CharacterFacade;
```

#### 使用条件

使用 `pub(in path)` 必须满足：

1. 用于层级边界控制（如 Capabilities 的 Foundation → Mechanism → Runtime）
2. 经过 @architect 评估并写入 ADR
3. 经过 @code-reviewer 审查

## Module Design

### 模块可见性模板

#### 私有模块（默认）

```rust
// 默认 private，不需要任何修饰符
mod components;
mod internal_state;
mod formula_calculation;
```

#### crate 内共享模块

```rust
// [ADR-045] pub(crate) — crate 内共享，测试可访问，外部不可访问
pub(crate) mod lifecycle;
pub(crate) mod types;
pub(crate) mod values;
```

#### 公共 API 模块

```rust
// [ADR-045] pub — 稳定公共 API，对外可见
pub mod plugin;
pub mod integration;
pub mod events;
```

#### 层级边界约束（需 ADR 批准）

```rust
// [ADR-045] pub(in path) — 仅用于层级边界约束
// 需要 @architect 批准并写入 ADR
pub(in crate::l2_execution) mod internal_context;
```

### 模块分类表

| 类型 | 可见性 | 示例 | 占比 |
|------|--------|------|------|
| 内部实现 | `private` | `components`、`systems`（内部）、`formula` | 70% |
| 域内共享 | `pub(crate)` | `lifecycle`、`types`、`values` | 25% |
| 层级边界 | `pub(in path)` | Capabilities 分层（需 ADR） | 3% |
| 公共 API | `pub` | `plugin`、`integration`、`events` | 2% |

### 可见性层级图

```
┌─────────────────────────────────────────────────────────────────┐
│  pub（稳定公共 API）                                              │
│  plugin.rs, integration.rs, events/, prelude/                   │
│  ──── 仅对外暴露的接口，占 2% ────────────────────────────────── │
├─────────────────────────────────────────────────────────────────┤
│  pub(in path)（层级边界约束，需 ADR 批准）                        │
│  Capabilities: Foundation → Mechanism → Runtime                  │
│  ──── 仅用于分层架构，占 3% ──────────────────────────────────── │
├─────────────────────────────────────────────────────────────────┤
│  pub(crate)（crate 内共享）                                      │
│  lifecycle, types, values, query, services                      │
│  ──── 域内共享 + 测试访问，占 25% ────────────────────────────── │
├─────────────────────────────────────────────────────────────────┤
│  #[cfg(test)]（仅测试可见）                                      │
│  test_support, test_builders                                    │
│  ──── 测试专用工具 ──────────────────────────────────────────── │
├─────────────────────────────────────────────────────────────────┤
│  private（私有，仅模块内部可见）                                   │
│  components, internal_state, formulas, helpers                  │
│  ──── 默认，占 70% ──────────────────────────────────────────── │
└─────────────────────────────────────────────────────────────────┘
```

## Communication Design

本决策不涉及通信机制变更。可见性策略是编译期约束，不引入新的运行时通信模式。

## 边界定义

### 允许

- 默认使用 `private`（不加任何修饰符）
- 需要域内共享时使用 `pub(crate)`
- 需要对外暴露时使用 `pub`（通过 Facade 或 integration.rs）
- 层级边界约束使用 `pub(in path)`（需 ADR 批准）

### 禁止

- 禁止无限制使用 `pub`（必须证明必要性）
- 禁止领域间用 `pub(in path)`（用 Facade 模式）
- 禁止新增 `pub(in path)` 未经 ADR 批准
- 禁止将 `pub(crate)` 改为 `pub` 除非有明确的外部 API 需求

## Forbidden（禁止事项）

### 绝对禁止

- 🟥 **禁止无限制使用 `pub`** — 必须证明必要性，否则默认 `private`
- 🟥 **禁止领域间用 `pub(in path)`** — 用 Facade 模式
- 🟥 **禁止新增 `pub(in path)` 未经 ADR 批准** — 需要 @architect 评估
- 🟥 **禁止将 `pub(crate)` 改为 `pub`** — 除非有明确的外部 API 需求

### 需要 ADR 批准

- ⚠️ 使用 `pub(in path)` 进行层级边界约束
- ⚠️ 将 `pub(crate)` 升级为 `pub`
- ⚠️ 新增公共 API 模块

### 需要 @code-reviewer 审查

- ⚠️ 任何可见性变更
- ⚠️ 新增 `pub` 模块
- ⚠️ 修改 `pub(in path)` 范围

## Definition / Instance Design

- **Definition（不可变配置）**：ADR-045 策略文档本身，作为架构规范存在
- **Instance（运行时状态）**：不涉及运行时组件，属于编译期可见性约束

## 后果

### 正面

- **权限边界清晰**：70% private + 25% pub(crate) + 5% pub，AI 可以明确知道什么能调用
- **编译期强制隔离**：`pub(crate)` 确保外部 crate 无法访问内部模块
- **领域边界清晰**：通过 Facade 模式保护领域内部实现
- **易于维护**：可见性规则简单明确，不会出现 `pub(in path)` 满天飞的情况
- **支持跨域测试**：`pub(crate)` 让测试可以跨域访问模块

### 负面

- 需要开发者（包括 AI）理解并遵守可见性规则
- 初期可能需要重构部分现有代码
- `pub(crate)` 仍然暴露给整个 crate（不如 `pub(in path)` 精确）

## 替代方案

### 方案对比

| 方案 | 描述 | 优点 | 缺点 | 选择 |
|------|------|------|------|------|
| **A: 分层可见性策略** | private 70% + pub(crate) 25% + pub 5% | 简单明确，易于维护 | pub(crate) 精度不够 | ✅ **采用** |
| B: 全面 pub(in path) | 精确控制每个模块的可见性 | 最精确 | 维护成本高，AI 难理解 | ❌ |
| C: 无限制 pub | 最简单 | 无 | 权限边界失控 | ❌ |

### 长期演进路径

```
当前（ADR-045）: 分层可见性策略（private 70% + pub(crate) 25% + pub 5%）
     ↓ （当项目规模超过 100 万行时评估）
中期: 引入 Workspace 多 crate，使用 pub 跨 crate 访问
     ↓ （当需要更严格的域间隔离时评估）
远期: 考虑 pub(in path) 用于关键层级边界（需 ADR 批准）
```

## 验证清单

- [ ] 符合 DDD 三层 + 横切四层架构
- [ ] 符合最小可见性原则（默认 private）
- [ ] 可见性比例符合目标（private 70% + pub(crate) 25% + pub 5%）
- [ ] 领域间使用 Facade 模式，不直接暴露内部实现
- [ ] `pub(in path)` 仅用于层级边界约束，且经过 ADR 批准
- [ ] Forbidden 列表已明确列出禁止行为
- [ ] 与现有 ADR 无冲突
- [ ] @code-reviewer 已知晓此策略
- [ ] 所有 AI Agent 已知晓此策略

## 附录：可见性检查工具

### 定期审查

建议每季度审查一次项目的可见性比例：

```bash
# 统计 pub 数量
grep -r "^pub " src/ | wc -l

# 统计 pub(crate) 数量
grep -r "^pub(crate) " src/ | wc -l

# 统计 private 数量（没有 pub 修饰的 mod/struct/fn）
grep -r "^(mod|struct|fn|enum|trait) " src/ | wc -l
```

### 自动化检查

建议在 CI 中添加可见性比例检查：

```yaml
# .github/workflows/visibility-check.yml
- name: Check visibility ratio
  run: |
    PUB_COUNT=$(grep -r "^pub " src/ | wc -l)
    CRATE_COUNT=$(grep -r "^pub(crate) " src/ | wc -l)
    TOTAL=$((PUB_COUNT + CRATE_COUNT))
    PUB_RATIO=$((PUB_COUNT * 100 / TOTAL))
    if [ $PUB_RATIO -gt 10 ]; then
      echo "Warning: pub ratio is $PUB_RATIO%, should be < 10%"
      exit 1
    fi
```
