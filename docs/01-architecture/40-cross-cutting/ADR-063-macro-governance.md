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

## 背景

项目中已有 15+ 宏分布在 6 个模块中，没有统一的治理规则。主要问题包括：

1. **无治理规则**：哪些场景用宏、哪些用函数/泛型，无明确准入门槛
2. **万能宏文件**：`src/macros.rs` 出现了不同主题的宏混杂在同一文件的反模式
3. **Helper Macro 风险**：部分宏开始承担"隐藏控制流"的角色（如 `ok_or_return!`），降低代码可读性
4. **proc-macro 准入模糊**：宪法 §16.5 规定"过程宏需 ADR 审批"，但缺乏具体审批标准
5. **宏替换函数**：个别场景宏被用作函数替代品（如伪 DSL 封装业务逻辑），违反"宏只做重复结构"原则

### 当前宏分布健康度评估

| 模块 | 宏数量 | 状态 |
|------|--------|------|
| `infra/logging/rate_limit/once_guard.rs` | 1 | 健康 |
| `infra/logging/rate_limit/mod.rs` | 1 | 健康 |
| `shared/ids/mod.rs` | 1 | 健康 |
| `shared/ids/runtime_id.rs` | 2 | 健康 |
| `shared/ids/tests/mod.rs` | 1 | 健康 |
| `src/macros.rs` | 6 | **问题** — 不同主题宏混杂，违反"宏跟能力走" |
| `fre_macros/src/` | 3 | 健康 — proc-macro 在独立 crate |

**评估结论**：6/7 模块已健康，仅 `src/macros.rs` 需拆分迁移。

## 决策

### 1. 抽象优先级

宏是最后手段，非首选。优先级：Trait → 泛型 → 函数 → macro_rules! → proc_macro。只有前者无法解决时才允许升级。

### 2. 宏跟能力走

`macro_rules!` 必须归属于其服务的具体能力模块，禁止建立全局 `src/macros/` 目录或万能宏文件。

### 3. 禁止跨层宏依赖

Domain 层不得依赖 Infra 层的宏（如 `emit_info!` 不能在 `core/domains/` 中使用）。宏依赖方向必须遵循分层：Shared ← Core ← Infra。跨层通信应通过 Observer + 事件，而非直接宏调用。

### 4. Declarative vs Procedural 分离

`macro_rules!` 必须归属于其服务的具体能力模块，禁止建立全局 `src/macros/` 目录或万能宏文件。任何宏文件的首选位置是它服务的 trait/能力定义旁边。

```rust
// ✅ 正确：宏放在服务的能力模块中
// infra/logging/macros.rs
macro_rules! emit_info { ... }

// ❌ 错误：全局万能宏文件
// src/macros.rs  ← 禁止
```

### 2. Declarative vs Procedural 分离

- `macro_rules!` 声明式宏留在主 crate 的各模块中
- proc_macro 必须放独立 crate（`fre_macros/`）
- 禁止在 proc-macro crate 中定义 `macro_rules!` 再 re-export

### 4. Declarative vs Procedural 分离

- `macro_rules!` 声明式宏留在主 crate 的各模块中
- proc_macro 必须放独立 crate（`fre_macros/`）

### 5. Derive 必须服务于 Trait

`#[derive(...)]` 必须生成 Trait 实现，不得生成隐藏业务行为。允许：`#[derive(DomainEvent)]`、`#[derive(Observable)]`。禁止：`#[derive(AutoCombat)]`。

### 6. Cargo Expand 可读性原则

cargo expand 后代码必须可读、接近手写 Rust，禁止生成难以调试的嵌套代码。

### 7. 宏不得隐藏业务逻辑

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

### 4. 宏准入门槛

| 调用点 | 推荐方案 |
|--------|---------|
| < 5 处 | 用函数，禁止引入宏 |
| 5~20 处 | 考虑泛型或函数 |
| 20+ 处 | 考虑宏 |
| 100+ 处 | 考虑 proc-macro derive |

新增 proc-macro 必须经 ADR 审批。

### 5. 宏文件超过 10 个宏必须拆分

单文件 `macro_rules!` 超过 10 个时按主题拆分子文件。超过 50 行宏逻辑必须抽取帮助函数。

## Module Design

### 当前宏迁移计划

需要从 `src/macros.rs` 迁移的 6 个宏：

| 宏名 | 目标模块 | 优先级 |
|------|---------|--------|
| (宏1) | 按能力归属迁移 | P1 |
| (宏2) | 按能力归属迁移 | P1 |
| (宏3) | 按能力归属迁移 | P1 |
| (宏4) | 按能力归属迁移 | P2 |
| (宏5) | 按能力归属迁移 | P2 |
| (宏6) | 按能力归属迁移 | P2 |

迁移完成后的目标状态：`src/macros.rs` 文件删除，所有宏归属到对应的能力模块中。

### 不需要新增模块

本决策不产生新的编译单元。proc-macro crate（`fre_macros/`）已存在，不需要新 crate。

## Forbidden（禁止事项）

- **🟥 禁止建立全局 `src/macros/` 目录或万能宏文件**
- **🟥 禁止跨层宏依赖** — Domain 层不得使用 Infra 宏，Shared 层不得使用 Core/Infra 宏
- **🟥 禁止创建 Helper Macro 隐藏控制流** — `ok_or_return!`、`try_get!`、`some_or_continue!` 等
- **🟥 禁止用宏封装业务逻辑** — `do_damage!`、`spawn_enemy!`、`apply_buff!` 等
- **🟥 禁止在调用点 < 5 处引入宏**
- **🟥 禁止宏嵌套宏** — 宏展开深度不超过 2 层
- **🟥 禁止derive生成隐藏业务行为** — derive 必须生成 Trait 实现而非行为
- **🟥 禁止 proc-macro crate 依赖主 crate**

## 影响评估

### 当前影响

影响极小：只需从 `src/macros.rs` 迁移 6 个宏到各自归属模块，改动量约 2 个文件移动 + 更新引用。

### 长期影响

| 维度 | 影响 |
|------|------|
| 宏数量增长 | 准入门槛限制数量膨胀，< 5 调用点禁止引入宏 |
| 可维护性 | 宏跟能力走 + 10 宏拆分规则保证文件可读性 |
| 代码审查 | 新增 proc-macro 必须 ADR 审批，控制复杂度 |
| AI 友好度 | 禁止 Helper Macro 保证控制流透明，AI 可理解 |

### 后续任务

1. P1: 拆分 `src/macros.rs`，将各宏迁移到对应能力模块
2. P2: 清理遗留的 `use crate::macros::*` 引用
3. P2: 删除 `src/macros.rs` 文件
4. 长期: 新增宏前检查准入门槛，新增 proc-macro 前发起 ADR

## 参考

- 宪法 §16.6 宏治理宪法 — 本文的宪法级约束
- `docs/01-architecture/40-cross-cutting/ADR-058-derive-macro.md` — Derive 宏设计规范
- `.trae/rules/架构规则.md` — 架构规则宏治理章节
