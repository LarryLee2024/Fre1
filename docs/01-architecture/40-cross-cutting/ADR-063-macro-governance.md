---
id: 01-architecture.40-cross-cutting.ADR-063
title: "ADR-063: 宏治理（Macro Governance）— 11条宏使用宪法原则"
status: Accepted
owner: architect
created: 2026-06-21
updated: 2026-06-21
tags:
  - architecture
  - macros
  - governance
  - cross-cutting
---

# ADR-063: 宏治理（Macro Governance）

## 状态

**Accepted** — 已被架构委员会接受。

## 引用的领域规则

- 宪法 §16.6 宏治理宪法 — 本 ADR 的直接宪法级约束，11 条原则的原始定义
- 宪法 §16.5 抽象与宏使用规范 — "三次才抽象"、"宏只做重复结构"、"Derive宏边界"
- `docs/01-architecture/40-cross-cutting/ADR-058-derive-macro.md` — Derive 宏 + Trait 组合模式规范

## 背景

项目中已有 15+ 宏分布在多个模块中，需要统一的治理规则确保一致性和可维护性。主要问题包括：

1. **无治理规则**：哪些场景用宏、哪些用函数/泛型，无明确准入门槛
2. **Helper Macro 风险**：部分宏开始承担"隐藏控制流"的角色（如 `ok_or_return!`），降低代码可读性
3. **proc-macro 准入模糊**：宪法 §16.5 规定"过程宏需 ADR 审批"，但缺乏具体审批标准
4. **宏替换函数**：个别场景宏被用作函数替代品（如伪 DSL 封装业务逻辑），违反"宏只做重复结构"原则
5. **一致性缺失**：宏分布在 8 个文件中，无统一命名/文档/review 标准

### 当前宏分布健康度评估

| 模块 | 宏数量 | 状态 |
|------|--------|------|
| `infra/logging/telemetry.rs` | 3 (`emit_info!`, `emit_warn!`, `emit_debug!`) | 健康 |
| `infra/logging/rate_limit/mod.rs` | 2 (`warn_once!`, `error_once!`) | 健康 |
| `shared/macros.rs` | 1 (`register_domain_types!`) | 健康 |
| `shared/traits/macros.rs` | 1 (`impl_rule_failure!`) | 健康 |
| `shared/diagnostics/macros.rs` | 1 (`impl_domain_event!`) | 健康 |
| `shared/ids/foundation/macros.rs` | 2 (`define_string_id!`, `define_numeric_id!`) | 健康 |
| `shared/testing/assertions.rs` | 7 (测试断言宏) | 健康 |
| `fre_macros/src/` | 3 (proc-macro) | 健康 — proc-macro 在独立 crate |

**评估结论**：所有宏已归属于对应能力模块，无全局万能宏文件。`src/macros.rs` 已于前期重构中拆除，宏均迁移至能力归属文件。

## 决策

本决策定义 11 条宏治理原则，与宪法 §16.6 的 11 条原则一一对应。

### 原则 1：抽象优先级

宏是最后手段，非首选。优先级：Trait → 泛型 → 函数 → macro_rules! → proc_macro。只有前者无法解决时才允许升级。

核心判断标准：
- 如果函数能解决问题（性能/可读性可接受），用函数
- 如果泛型能消除重复，用泛型
- 如果 Trait 能定义能力边界，用 Trait
- 只有上述均不适配的场景，再考虑宏

### 原则 2：宏跟能力走

`macro_rules!` 必须归属于其服务的具体能力模块，禁止建立全局 `src/macros/` 目录或万能宏文件。

```rust
// ✅ 正确：宏放在服务的能力模块中
// infra/logging/telemetry.rs
macro_rules! emit_info { ... }

// ❌ 错误：全局万能宏文件（已拆除）
// src/macros.rs  ← 禁止
```

任何宏文件的首选位置是它服务的 trait/能力定义旁边。例如 `impl_rule_failure!` 放在 `shared/traits/macros.rs`，与 `RuleFailure` trait 定义相邻。

### 原则 3：禁止跨层宏依赖

Domain 层不得依赖 Infra 层的宏（如 `emit_info!` 不能在 `core/domains/` 中使用）。宏依赖方向必须遵循分层：Shared ← Core ← Infra。跨层通信应通过 Observer + 事件，而非直接宏调用。

| 使用场景 | 允许的宏来源 |
|---------|-------------|
| Domain 使用 Shared 宏 | 允许 |
| Domain 使用 Core/Capabilities 宏 | 允许（同层） |
| Domain 使用 Infra 宏 | 禁止 |
| Infra 使用 Core 宏 | 允许 |

### 原则 4：Declarative vs Procedural 分离

- `macro_rules!` 声明式宏留在主 crate 的各模块中
- proc_macro 必须放独立 crate（`fre_macros/`）
- 禁止在 proc-macro crate 中定义 `macro_rules!` 再 re-export

分离理由：
- `macro_rules!` 是文本展开，调试依赖 `trace_macros!`，适合语法糖和简单重复
- proc_macro 是 AST 操作，调试依赖 `cargo expand`，适合复杂的 Trait derive 代码生成
- 两者混用会导致调试路径交叉，增加认知负担

### 原则 5：Derive 必须服务于 Trait

`#[derive(...)]` 必须生成 Trait 实现，不得生成隐藏业务行为。允许：`#[derive(DomainEvent)]`、`#[derive(Observable)]`、`#[derive(Replayable)]`。禁止：`#[derive(AutoCombat)]`。

详见 `docs/01-architecture/40-cross-cutting/ADR-058-derive-macro.md`。

### 原则 6：Cargo Expand 可读性原则

`cargo expand` 后代码必须可读、接近手写 Rust，禁止生成难以调试的嵌套代码。

具体要求：
- derive 宏的输出必须能让人直接审查正确性
- 避免生成深层嵌套的 match/if-else 链
- 生成代码中的标识符应有意义（非随机后缀），便于阅读堆栈追踪

### 原则 7：宏不得隐藏业务逻辑

宏只能用于：注册/派生/埋点/DSL/样板代码消除。禁止创建隐藏控制流的 Helper Macro：

```rust
// ❌ 禁止：Helper Macro 隐藏控制流
macro_rules! ok_or_return {
    ($expr:expr) => { match $expr { Ok(v) => v, Err(e) => return Err(e.into()) } };
}

// ❌ 禁止：宏封装业务逻辑
macro_rules! do_damage {
    ($target:expr, $amount:expr) => { /* 业务规则隐藏在宏中 */ };
}

// ✅ 允许：样板代码消除
macro_rules! define_stat {
    ($name:ident) => { pub struct $name(pub f32); };
}
```

判断标准：如果宏展开后做了"决策"（if/else 分支业务选择），说明它在隐藏业务逻辑；如果宏做的是"重复"（枚举变体到方法的映射、多行注册语句），说明它是合法的样板消除。

### 原则 8：宏必须可被函数替代

宏优先作为语法糖，核心逻辑必须位于普通函数。宏的职责是减少重复调用处的样板代码，而不是承载核心实现。

```rust
// ✅ 允许：宏做语法糖，核心逻辑在函数
// infra/logging/telemetry.rs
macro_rules! emit_info {
    ($($arg:tt)+) => {
        // 真正实现在普通函数中
        $crate::infra::logging::telemetry::emit(Level::Info, format_args!($($arg)+))
    };
}

// ❌ 禁止：核心实现完全隐藏在宏中
// 宏展开后包含百行格式化/过滤/路由逻辑，无函数后备
// 单元测试、mock、profile 无法绕过宏直接测试函数
```

此原则确保：
- 单元测试可以绕过宏直接测试函数
- Mock/Profile 替换走函数签名而非宏参数
- 调试时可直接在函数上设置断点

### 原则 9：禁止宏嵌套宏

- 禁止业务宏调用业务宏（宏展开深度原则上不超过 2 层）
- 允许业务宏调用底层公共宏（如 `emit_info!` 内部调用 `tracing::info!`）
- 超过 2 层展开链时，必须重构为函数调用

```rust
// ✅ 允许：宏调用底层库宏（1 层展开）
macro_rules! emit_info {
    ($($arg:tt)+) => { tracing::info!($($arg)+) };
}

// ❌ 禁止：宏嵌套宏（2+ 层展开）
macro_rules! log_battle_event { ... }     // 第 1 层
macro_rules! battle_damage { ... }         // 第 2 层 — 调用 log_battle_event
macro_rules! execute_combat_step { ... }   // 第 3 层 — 调用 battle_damage（禁止）
```

### 原则 10：宏准入门槛

| 调用点 | 推荐方案 |
|--------|---------|
| < 5 处 | 用函数，禁止引入宏 |
| 5~20 处 | 考虑泛型或函数 |
| 20+ 处 | 考虑宏 |
| 100+ 处 | 考虑 proc-macro derive |

新增 proc-macro 必须经 ADR 审批。

### 原则 11：宏文件超过 10 个宏必须拆分

单文件 `macro_rules!` 超过 10 个时按主题拆分子文件。超过 50 行宏逻辑必须抽取帮助函数。

## Module Design

### 当前宏布局（已完成）

`src/macros.rs` 已于前期重构中拆除。所有宏已迁移至对应的能力模块：

| 宏名 | 位置 | 原则合规 |
|------|------|---------|
| `register_domain_types!` | `shared/macros.rs` | 跟能力走（Shared 类型注册）|
| `impl_rule_failure!` | `shared/traits/macros.rs` | 跟能力走（RuleFailure Trait 旁）|
| `impl_domain_event!` | `shared/diagnostics/macros.rs` | 跟能力走（Diagnostics 旁）|
| `define_string_id!` | `shared/ids/foundation/macros.rs` | 跟能力走（ID 体系旁）|
| `define_numeric_id!` | `shared/ids/foundation/macros.rs` | 跟能力走（ID 体系旁）|
| `assert_approx_eq!` 等 7 个 | `shared/testing/assertions.rs` | 测试专用，合规 |
| `emit_info!` / `emit_warn!` / `emit_debug!` | `infra/logging/telemetry.rs` | 跟能力走，核心实现在普通函数 |
| `warn_once!` / `error_once!` | `infra/logging/rate_limit/mod.rs` | 跟能力走 |
| `#[derive(DomainEvent)]` 等 | `fre_macros/src/` | proc-macro 在独立 crate |

**目标状态已达成**：无全局万能宏文件，所有宏归属到对应的能力模块。

### 治理执行

本决策不产生新的编译单元。现有结构已经符合所有原则：

| 治理项 | 当前状态 |
|--------|---------|
| 全局万能宏文件 | `src/macros.rs` 已拆除 |
| proc-macro crate | `fre_macros/` 已存在且独立 |
| 宏归属 | 每个宏已按能力归属到对应模块 |
| 新宏准入 | 使用原则 1（抽象优先级）+ 原则 10（准入门槛）检查 |
| 新 proc-macro | 必须发起新 ADR 审批 |

## Communication Design

本决策不涉及 ECS 运行时通信机制（Hook/Trigger/Observer/Message）。宏治理是编译期代码规范，不产生运行时通信。

但需注意原则 3（禁止跨层宏依赖）对运行时通信的影响：
- Domain 需要 Infra 功能时，应通过 Observer + 事件，而非直接调用 Infra 层的宏
- 跨层通信的数据沿 Event 通道传递，不经过宏展开

## 边界定义

### 允许
- 宏放在其服务的能力模块目录中（与 trait/能力定义相邻）
- proc-macro 在独立 crate 中定义
- 宏做样板代码消除（类型注册、Trait 派生、枚举到方法的映射）
- 宏做语法糖，核心实现在普通函数
- 通过 `cargo expand` 审查宏展开结果
- 宏调用底层公共库宏（`tracing::info!` 等）

### 禁止
- 禁止建立全局 `src/macros/` 目录或万能宏文件
- 禁止跨层宏依赖（Domain 依赖 Infra 宏）
- 禁止创建隐藏控制流的 Helper Macro
- 禁止用宏封装业务逻辑
- 禁止在调用点 < 5 处引入宏
- 禁止宏嵌套宏（展开深度不超过 2 层）
- 禁止 derive 生成隐藏业务行为
- 禁止 proc-macro crate 依赖主 crate
- 禁止宏承担核心实现（核心逻辑必须在普通函数中）

## Definition / Instance Design

本决策不涉及 Definition/Instance 分离。宏是编译期代码生成工具，不产生运行时数据：

| 元素 | 适用性 | 说明 |
|------|--------|------|
| **Definition** | 不适用 | 宏定义是编译期代码，非运行时配置 |
| **Instance** | 不适用 | 宏不影响运行时 Component 或 Instance |
| **proc-macro crate** | 独立的编译期依赖 | `fre_macros/` 保持独立 |

## Forbidden（禁止事项）

- **🟥 禁止建立全局 `src/macros/` 目录或万能宏文件**
- **🟥 禁止跨层宏依赖** — Domain 层不得使用 Infra 宏，Shared 层不得使用 Core/Infra 宏。宏依赖方向必须遵循 Shared ← Core ← Infra
- **🟥 禁止创建 Helper Macro 隐藏控制流** — `ok_or_return!`、`try_get!`、`some_or_continue!` 等
- **🟥 禁止用宏封装业务逻辑** — `do_damage!`、`spawn_enemy!`、`apply_buff!` 等
- **🟥 禁止在调用点 < 5 处引入宏**
- **🟥 禁止宏嵌套宏** — 宏展开深度不超过 2 层，禁止业务宏调用业务宏
- **🟥 禁止 derive 生成隐藏业务行为** — derive 必须生成 Trait 实现而非执行行为
- **🟥 禁止 proc-macro crate 依赖主 crate** — proc-macro crate 必须独立编译
- **🟥 禁止宏承担核心实现** — 核心逻辑必须在普通函数中，宏只做语法糖
- **🟥 禁止为 < 10 个实现者的 trait 创建 derive 宏** — 避免过度抽象（来自 ADR-058）

## 影响评估

### 当前影响

无直接影响。`src/macros.rs` 已于前期重构中拆除，所有宏已按能力归属迁移到位。本 ADR 以文档形式确认已完成的状态并建立未来治理规则。

### 长期影响

| 维度 | 影响 |
|------|------|
| 宏数量增长 | 准入门槛限制数量膨胀，< 5 调用点禁止引入宏 |
| 可维护性 | 宏跟能力走 + 10 宏拆分规则保证文件可读性 |
| 代码审查 | 新增 proc-macro 必须 ADR 审批，控制复杂度；新增 macro_rules! 需对照 11 条原则审查 |
| AI 友好度 | 禁止 Helper Macro 保证控制流透明，禁止宏嵌套宏保证展开可追踪 |

### 后续任务

| 优先级 | 任务 | 说明 |
|--------|------|------|
| 持续 | 新增宏前检查 11 条原则合规 | 纳入 code review checklist |
| 持续 | 新增 proc-macro 前发起 ADR | 宪法 §16.5 要求 |
| 审计 | 定期 cargo expand 抽查宏展开质量 | 确保原则 6（可读性）不被违反 |

## DomainEvent 演进路线图

DomainEvent 的标记方式随项目规模增长分三阶段演进，禁止跳过阶段直接升级。

| 阶段 | 事件数量 | 方案 | 核心原则 |
|------|---------|------|---------|
| 阶段1（已关闭） | 20~50 | `impl_domain_event!()` 宏 | 显式可 grep |
| **阶段2（当前）** | **50~150** | **Blanket Impl** | **零宏零重复** |
| 阶段3（未来） | 150+ | `#[derive(DomainEvent)]` + 元数据 | 只生成 const |

### 阶段2：Blanket Impl（当前）

```rust
pub trait DomainEvent: Event + Debug + Clone + Send + Sync + 'static {}
impl<T> DomainEvent for T where T: Event + Debug + Clone + Send + Sync + 'static {}
```

任何 `#[derive(Event, Debug, Clone)]` 的 struct 自动是 DomainEvent。`impl_domain_event!()` 宏已废弃并删除。

### 阶段3：#[derive(DomainEvent)] 准入铁律

当项目超过 150 个事件、需要 DOMAIN/CODE 等元数据时，才允许引入 derive。且必须遵守以下三条铁律：

**🔒 铁律1：只生成 const**
```rust
#[derive(DomainEvent)]
#[domain(Combat)]
#[code(COM001)]
pub struct TurnEnded;
// 展开后只生成：
// impl DomainEvent for TurnEnded {
//     const DOMAIN: Domain = Domain::Combat;
//     const CODE: EventCode = EventCode::COM001;
// }
```
❌ 禁止生成函数、系统、资源、spawn 逻辑

**🔒 铁律2：不允许访问 AST 语义**
- ❌ 禁止读取 struct 字段
- ❌ 禁止根据字段名/类型生成 impl
- ✅ 只允许 `#[...]` attribute + ident

**🔒 铁律3：必须可手写为等价代码**
- cargo expand 后必须是能手动写出的 Rust 代码
- 禁止生成黑盒行为

## 替代方案

| 方案 | 缺点 | 结论 |
|------|------|------|
| 不制定治理规则，任其发展 | 宏可能无序增长，Helper Macro 泛滥，降低代码可读性 | 拒绝 |
| 完全禁止宏 | 失去宏在样板消除和 DSL 场景的价值，增加重复代码 | 拒绝 |
| 只定义规则不枚举 Forbidden | 缺乏可执行边界，review 时无明确检查点 | 拒绝 — 本 ADR 包含详细 Forbidden |
| 通过 Clippy lint 强制执行 | Clippy 自定义 lint 维护成本高，部分约束（<5 调用点）难以静态检测 | 暂缓 — 先文档治理，后续考虑自动化 |

## 参考

- 宪法 §16.6 宏治理宪法 — 本文的宪法级约束，11 条原则的原始定义
- 宪法 §16.5 抽象与宏使用规范 — "三次才抽象"、"宏只做重复结构"
- `docs/01-architecture/40-cross-cutting/ADR-058-derive-macro.md` — Derive 宏设计规范（本 ADR 原则 5 的详细实现）
- `.trae/rules/架构规则.md` — 架构规则宏治理章节
