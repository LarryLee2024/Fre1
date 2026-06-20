---
id: 08-knowledge.error-handling-overview
title: 错误处理架构深度解析 — Error vs Failure 严格分离
status: stable
owner: architect
created: 2026-06-20
updated: 2026-06-28
tags:
  - knowledge
  - error-handling
  - domain-driven
  - rust
  - adr-051
---

# 错误处理架构深度解析

> 目标读者：新加入项目的开发者，或其他想理解错误处理系统全貌的人。
> 读完本文，你会知道错误是怎么分类的、代码放在哪、一条错误从产生到处理经历了什么。

---

## 目录

1. [核心思想：为什么错误要这么设计？](#1-核心思想为什么错误要这么设计)
2. [整体架构全景图](#2-整体架构全景图)
3. [Error vs Failure：两种错误的严格分离](#3-error-vs-failure两种错误的严格分离)
4. [第一层：共享类型层 — 错误的「方言」](#4-第一层共享类型层--错误的方言)
5. [第二层：领域错误层 — 各领域的错误定义](#5-第二层领域错误层--各领域的错误定义)
6. [第三层：基础设施层 — 特殊错误处理](#6-第三层基础设施层--特殊错误处理)
7. [数据流全景：一条错误的诞生与处理](#7-数据流全景一条错误的诞生与处理)
8. [实战：如何添加新的错误类型](#8-实战如何添加新的错误类型)
9. [现状盘点：已经做了什么，还缺什么](#9-现状盘点已经做了什么还缺什么)
10. [规则速查：该做什么和不该做什么](#10-规则速查该做什么和不该做什么)

---

## 1. 核心思想：为什么错误要这么设计？

### 1.1 最大的原则：Error 与 Failure 严格分离

传统游戏项目的错误处理长这样：

```rust
// ❌ 传统做法：所有错误混在一起
fn use_skill(entity: Entity, skill: &Skill) -> Result<(), String> {
    if !skill.can_use(entity) {
        return Err("技能不可用".to_string());  // 这是业务规则失败，不是程序错误
    }
    if skill.cooldown > 0 {
        return Err("技能冷却中".to_string());  // 这也是业务规则失败
    }
    // ...
    Ok(())
}
```

看起来没问题，但在 Fre 这种 50 万行级别的项目里会出现三个问题：

1. **语义混淆** — 「技能冷却中」是正常业务结果，不应该当作程序错误处理
2. **错误处理不一致** — 有些地方用 `Err`，有些地方用 `Result`，有些地方用事件
3. **无法统一处理** — 基础设施层不知道该把哪些错误记录日志、哪些错误触发告警

所以 Fre 的做法是反过来：**严格区分 Error（程序错误）和 Failure（业务规则失败）。**

```rust
// ✅ Fre 的做法：Error 与 Failure 分离
fn use_skill(entity: Entity, skill: &Skill) -> Result<(), SkillError> {
    // 程序错误：不应该发生的情况
    if !skill.is_registered {
        return Err(SkillError::NotRegistered);  // 程序 bug
    }
    Ok(())
}

fn check_skill_use(entity: Entity, skill: &Skill) -> Result<(), SkillFailure> {
    // 业务规则失败：正常业务结果
    if skill.cooldown > 0 {
        return Err(SkillFailure::OnCooldown);  // 正常业务限制
    }
    Ok(())
}
```

这种设计叫做**错误语义分离**，记录在宪法和 ADR 中。

### 1.2 两种错误的下游处理

一个错误产生后，根据类型有不同的处理方式：

```
错误产生
  │
  ├── Error（程序错误）
  │     ├── 应该被修复（是 bug）
  │     ├── 可能需要 panic 或 error!() 日志
  │     └── 通过 Err() 传播，最终被上层捕获处理
  │
  └── Failure（业务规则失败）
        ├── 是正常业务结果（如「材料不足」）
        ├── 不应该被修复（是设计如此）
        └── 通过 Result<T, Failure> 返回，由 UI 或其他系统处理
```

### 1.3 三层架构中的错误位置

Fre 的代码分为三层，错误系统跨越其中两层：

```
┌─ L0: Shared（原子层）─────────────────────────────────┐
│  shared/error/                                         │
│  ├── mod.rs              ← ErrorContext<E> 包装器     │
│  ├── tests/              ← 单元测试                   │
│  shared/traits/                                         │
│  └── mod.rs              ← RuleFailure trait          │
│  这些是纯类型定义和 trait，定义错误的「方言」            │
├─ L1: Core（领域规则层）─────────────────────────────────│
│  core/domains/*/                                        │
│  ├── error.rs            ← 各领域的 Error 枚举        │
│  └── failure.rs          ← 各领域的 Failure 枚举      │
│  每个领域独立定义自己的错误类型                         │
├─ L2: Infra（技术实现层）────────────────────────────────│
│  infra/localization/                                    │
│  └── error.rs            ← 基础设施层错误（不派生Event）│
└─────────────────────────────────────────────────────────┘
```

---

## 2. 整体架构全景图

```
┌─────── 领域代码 (L1 Core) ───────┐
│                                  │
│  // 程序错误                      │
│  return Err(CombatError::UnitDead│
│    { reason: "..." })            │──────────┐
│                                  │          │
│  // 业务规则失败                  │          │
│  return Err(CombatFailure::      │          │
│    NotYourTurn)                  │──────────┤
│                                  │          │
└──────────────────────────────────┘          │
                                              ▼
┌─────── 错误传播层 ──────────────────────────────────────────────────────┐
│                                                                         │
│  ErrorContext<E> 包装器                                                  │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │  fn do_combat() -> Result<(), ErrorContext<CombatError>> {     │    │
│  │      risky_operation()                                         │    │
│  │          .domain("combat")     ← 添加领域标签                  │    │
│  │          ?;                                                     │    │
│  │  }                                                              │    │
│  │                                                                  │    │
│  │  // 输出: [combat] Insufficient participants: required=2, ...   │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
│  ContextExt trait 为 Result 提供扩展方法                                 │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │  .domain(tag)           ← 只添加领域标签                       │    │
│  │  .with_context(tag, msg)← 添加领域标签 + 上下文信息            │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
│  RuleFailure trait 为 Failure 提供统一错误码                            │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │  impl RuleFailure for CombatFailure {                          │    │
│  │      fn code(&self) -> &'static str {                         │    │
│  │          match self {                                           │    │
│  │              Self::NotYourTurn => "COMBAT_NOT_YOUR_TURN",     │    │
│  │          }                                                      │    │
│  │      }                                                          │    │
│  │  }                                                              │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────── 基础设施层处理 ──────────────────────────────────────────────┐
│                                                                     │
│  管线系统 (Pipeline)                                                 │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  StepResult::Failure(ErrorContext<String>)                  │    │
│  │      ↓                                                      │    │
│  │  FailureStrategy 决定: Abort / SkipAndContinue / Retry     │    │
│  └─────────────────────────────────────────────────────────────┘    │
│                                                                     │
│  存档系统 (Save)                                                     │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  SaveError { error_context: ErrorContext<String> }          │    │
│  │      ↓                                                      │    │
│  │  记录日志 + 返回错误给调用方                                │    │
│  └─────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 3. Error vs Failure：两种错误的严格分离

### 3.1 定义对比

| 维度 | Error (`*Error`) | Failure (`*Failure`) |
|------|------------------|----------------------|
| **语义** | 程序错误，不应该发生 | 业务规则失败，正常结果 |
| **示例** | `UnitDead`（单位已死亡） | `NotYourTurn`（不是你的回合） |
| **处理方式** | 通过 `Err()` 传播 | 通过 `Result<T, Failure>` 返回 |
| **是否需要修复** | 是（是 bug） | 否（是设计如此） |
| **日志级别** | `error!()` 或 panic | `warn!()` 或 `debug!()` |
| **派生 trait** | `Event`（可作为 Bevy 事件） | 无（不派生 Event） |
| **统一接口** | 无 | `RuleFailure` trait |

### 3.2 为什么有些变体在两者中都存在？

你可能会注意到 `CombatError` 和 `CombatFailure` 中有些变体看起来一样（如 `InsufficientParticipants`、`NotYourTurn`）。这是故意的：

```rust
// 程序错误版本：表示系统内部状态异常
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum CombatError {
    /// 参与单位不足，无法开始战斗。
    /// 这是系统配置错误或数据损坏
    InsufficientParticipants { required: usize, actual: usize },
}

// 业务规则失败版本：表示正常业务限制
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CombatFailure {
    /// 不是该单位的回合。
    /// 这是正常业务流程
    NotYourTurn,
}
```

**关键区别**：虽然名字相似，但它们的**上下文不同**：
- `CombatError::InsufficientParticipants` = 系统配置错误（需要修复）
- `CombatFailure::InsufficientParticipants` = 玩家尝试开始战斗但人数不够（正常游戏流程）

### 3.3 派生 trait 的区别

```rust
// Error: 派生 Event，可作为 Bevy 事件发送
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum CombatError { ... }

// Failure: 不派生 Event，只通过 Result 返回
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CombatFailure { ... }
```

为什么 Error 要派生 `Event`？因为程序错误可能需要被基础设施层监听（如记录日志、触发告警），而 Failure 是正常业务结果，不需要事件机制。

---

## 4. 第一层：共享类型层 — 错误的「方言」

在 `src/shared/` 下，定义了两种核心类型，它们是错误的「方言」，所有层都能引用。

### 4.1 ErrorContext<E> — 错误包装器

`ErrorContext<E>` 为错误附加**领域标签**和**上下文信息**，类似于 anyhow::Context，但限域到领域标签，不引入全局错误类型。

```rust
/// 带领域上下文的错误包装。
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorContext<E> {
    /// 来源领域标识（如 "combat"、"inventory"）
    pub domain: &'static str,
    
    /// 原始错误
    pub source: E,
    
    /// 额外上下文说明
    pub context: Option<String>,
}
```

**使用示例**：

```rust
use fre_shared::error::ContextExt;

fn do_combat() -> Result<(), ErrorContext<CombatError>> {
    let result = risky_operation()
        .domain("combat")?;  // 添加领域标签
    
    let result2 = other_op()
        .with_context("combat", "during damage calculation")?;  // 添加标签+上下文
    
    Ok(())
}
```

**输出格式**：

```
[combat] Insufficient participants: required=2, actual=1 (during damage calculation)
```

### 4.2 ContextExt trait — Result 扩展方法

为 `Result<T, E>` 提供两个方法：

| 方法 | 用途 | 示例 |
|------|------|------|
| `.domain(tag)` | 只添加领域标签 | `result.domain("combat")?` |
| `.with_context(tag, msg)` | 添加标签 + 上下文信息 | `result.with_context("combat", "伤害计算中")?` |

```rust
pub trait ContextExt<T, E> {
    fn domain(self, tag: &'static str) -> Result<T, ErrorContext<E>>;
    fn with_context(self, tag: &'static str, msg: impl Into<String>) -> Result<T, ErrorContext<E>>;
}
```

### 4.3 RuleFailure trait — Failure 统一接口

为所有 `*Failure` 枚举提供统一的错误码接口：

```rust
/// 规则失败标记 trait。
///
/// 业务规则不满足是正常结果（非 Err），与程序错误严格区分。
/// 每个 domain 独立定义各自的 `*Failure` 枚举并实现此 trait。
pub trait RuleFailure: std::fmt::Debug + Send + Sync + 'static {
    /// 返回机器可读的规则失败码。
    fn code(&self) -> &'static str;
}
```

**使用示例**：

```rust
impl RuleFailure for CombatFailure {
    fn code(&self) -> &'static str {
        match self {
            Self::InsufficientParticipants { .. } => "COMBAT_INSUFFICIENT_PARTICIPANTS",
            Self::NotYourTurn => "COMBAT_NOT_YOUR_TURN",
            Self::NoActionRemaining => "COMBAT_NO_ACTION",
            Self::UnitDead => "COMBAT_UNIT_DEAD",
        }
    }
}
```

**错误码命名规则**：`{域前缀}_{失败类型}`，如 `COMBAT_NOT_YOUR_TURN`。

---

## 5. 第二层：领域错误层 — 各领域的错误定义

在 `src/core/domains/` 下，每个领域独立定义自己的错误类型。

### 5.1 文件组织

```
src/core/domains/
├── combat/
│   ├── error.rs          # CombatError（5 个程序错误）
│   └── failure.rs        # CombatFailure（4 个业务失败）
├── crafting/
│   └── failure.rs        # CraftingFailure（7 个业务失败，无程序错误）
├── inventory/
│   ├── error.rs          # InventoryError（1 个程序错误）
│   └── failure.rs        # InventoryFailure（8 个业务失败）
├── progression/
│   └── failure.rs        # ProgressionFailure（7 个业务失败，无程序错误）
└── ... (13 个领域有 error.rs + failure.rs，2 个领域只有 failure.rs)
```

> **注意**：Progression 和 Crafting 域没有程序错误，因此没有 `error.rs` 文件。

### 5.2 Error 枚举模板

以 `CombatError` 为例（迁移后只包含程序错误）：

```rust
//! 领域错误 — Combat 域程序错误枚举。
//!
//! 涵盖战斗系统的程序错误（不应发生的异常情况）。
//! 业务规则失败请使用 `CombatFailure`（failure.rs）。
//! 详见 ADR-051

use bevy::prelude::*;
use thiserror::Error;

/// 战斗系统程序错误。
///
/// 这些错误表示系统内部状态异常，属于程序缺陷或环境问题。
/// 业务规则不满足的结果（如"不是你的回合"）请使用 [`CombatFailure`]。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum CombatError {
    /// 单位未注册为战斗参与者。
    #[error("entity is not a combat participant")]
    NotCombatParticipant,
    
    /// 战斗尚未开始。
    #[error("combat has not started")]
    CombatNotStarted,
    
    /// 战斗已结束，不可再操作。
    #[error("combat has already ended")]
    CombatAlreadyEnded,
    
    /// 先攻排序为空。
    #[error("turn order is empty")]
    EmptyTurnOrder,
    
    /// 伤害已被结算，禁止重复结算。
    #[error("damage already resolved, duplicate forbidden")]
    DamageAlreadyResolved,
}
```

**关键点**：
- 派生 `Event`（可作为 Bevy 事件）
- 派生 `Error`（来自 thiserror）
- 使用 `#[error("...")]` 定义 Display 输出
- **只包含程序错误**，业务规则失败在 `*Failure` 中

### 5.3 Failure 枚举模板

以 `CombatFailure` 为例：

```rust
//! 规则失败 — Combat 域业务规则不满足结果。
//!
//! 与 `CombatError`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 docs/02-domain/domains/combat_domain.md §4

use crate::shared::traits::RuleFailure;
use thiserror::Error;

/// 战斗系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum CombatFailure {
    /// 参与单位不足，无法开始战斗。
    #[error("insufficient participants: required={required}, actual={actual}")]
    InsufficientParticipants { required: usize, actual: usize },
    
    /// 不是该单位的回合。
    #[error("it is not this unit's turn")]
    NotYourTurn,
    
    // ... 其他变体
}

impl RuleFailure for CombatFailure {
    fn code(&self) -> &'static str {
        match self {
            Self::InsufficientParticipants { .. } => "COMBAT_INSUFFICIENT_PARTICIPANTS",
            Self::NotYourTurn => "COMBAT_NOT_YOUR_TURN",
            Self::NoActionRemaining => "COMBAT_NO_ACTION",
            Self::UnitDead => "COMBAT_UNIT_DEAD",
        }
    }
}
```

**关键点**：
- **不派发** `Event`
- 实现 `RuleFailure` trait（提供统一错误码）

---

## 6. 第三层：基础设施层 — 特殊错误处理

基础设施层有一些特殊的错误处理模式。

### 6.1 Localization 错误（不派生 Event）

`src/infra/localization/error.rs` 定义了 `LocError`，它**不派生 Event**，因为本地化错误是基础设施层内部错误，不需要作为 Bevy 事件传播：

```rust
use std::error::Error;
use std::fmt;

/// Localization 错误类型
#[derive(Debug, Clone)]
pub enum LocError {
    /// Key 在重试所有 fallback locale 后仍未找到
    KeyNotFound {
        key: String,
        locale: LocaleId,
        fallbacks_attempted: Vec<LocaleId>,
    },
    
    /// 参数不匹配
    MissingParameter {
        key: String,
        missing: Vec<String>,
        provided: Vec<String>,
    },
    
    // ... 其他变体
}

impl fmt::Display for LocError { ... }
impl Error for LocError {}
```

**关键点**：
- 手动实现 `Display` 和 `Error`（不使用 thiserror）
- **不派生** `Event`
- 零依赖层，仅使用 Rust 标准库

### 6.2 Pipeline 错误（管线系统）

管线系统有自己的错误处理机制：

```rust
/// 管线执行结果。
#[derive(Debug, Clone, PartialEq)]
pub enum StepResult {
    /// 成功
    Success,
    /// 失败带领域错误上下文
    Failure(ErrorContext<String>),
    /// 跳过
    Skipped,
}

/// Pipeline 领域错误。
#[derive(Debug, Clone, PartialEq)]
pub enum PipelineError {
    /// 阶段未找到
    StageNotFound(String),
    /// 步骤执行失败
    StepFailed {
        stage: String,
        step: String,
        detail: String,
    },
    // ... 其他变体
}
```

**关键点**：
- `StepResult::Failure` 使用 `ErrorContext<String>` 包装错误
- `PipelineError` 是管线系统自身的错误（不是业务错误）

### 6.3 Save 错误（存档系统）

存档系统也使用 `ErrorContext` 包装错误：

```rust
pub struct SaveError {
    pub error_context: ErrorContext<String>,
}
```

---

## 7. 数据流全景：一条错误的诞生与处理

以「玩家尝试在不是自己回合时行动」为例：

```
Step 1: 领域代码检查业务规则
─────────────────────────────
  combat_system.rs:
    fn execute_action(entity: Entity, action: &Action) -> Result<(), CombatFailure> {
        // 检查是否是该单位的回合
        if current_turn != entity {
            return Err(CombatFailure::NotYourTurn);  // 返回业务规则失败
        }
        // ...
        Ok(())
    }

Step 2: 调用方处理 Failure
─────────────────────────────
  action_handler.rs:
    match execute_action(entity, action) {
        Ok(()) => { /* 执行成功 */ }
        Err(failure) => {
            // 通过 RuleFailure trait 获取统一错误码
            let code = failure.code();  // "COMBAT_NOT_YOUR_TURN"
            
            // 根据错误码决定如何处理
            match code {
                "COMBAT_NOT_YOUR_TURN" => {
                    // 显示 UI 提示：「现在不是你的回合」
                    ui.show_message("现在不是你的回合");
                }
                "COMBAT_NO_ACTION" => {
                    ui.show_message("没有行动点了");
                }
                // ... 其他错误码
            }
        }
    }

Step 3: 如果是程序错误，使用 ErrorContext 包装
─────────────────────────────
  combat_system.rs:
    fn load_combat_config() -> Result<CombatConfig, ErrorContext<CombatError>> {
        let config = read_config_file()
            .domain("combat")  // 添加领域标签
            .with_context("combat", "加载战斗配置")?;  // 添加上下文
        
        Ok(config)
    }

Step 4: 程序错误被上层捕获
─────────────────────────────
  app_plugin.rs:
    match load_combat_config() {
        Ok(config) => { /* 使用配置 */ }
        Err(error_context) => {
            // 输出: [combat] Failed to read config file (加载战斗配置)
            error!("{}", error_context);
            
            // 可能 panic 或返回错误给用户
        }
    }
```

---

## 8. 实战：如何添加新的错误类型

假设你要为「召唤系统」添加新的错误类型。

### 第一步：确定是 Error 还是 Failure

- 如果是**程序错误**（不应该发生）→ 添加到 `SummonError`
- 如果是**业务规则失败**（正常结果）→ 添加到 `SummonFailure`

### 第二步：添加 Error 变体（如果需要）

```rust
// src/core/domains/summon/error.rs
use bevy::prelude::*;
use thiserror::Error;

/// 召唤系统错误。
#[derive(Debug, Clone, PartialEq, Event, Error)]
pub enum SummonError {
    // 已有变体...
    
    /// 召唤模板损坏。
    #[error("summon template corrupted: {template_id}")]
    TemplateCorrupted { template_id: String },
}
```

### 第三步：添加 Failure 变体（如果需要）

```rust
// src/core/domains/summon/failure.rs
use crate::shared::traits::RuleFailure;
use thiserror::Error;

/// 召唤系统业务规则失败。
#[derive(Debug, Clone, PartialEq, Error)]
pub enum SummonFailure {
    // 已有变体...
    
    /// 召唤槽位已满。
    #[error("summon slots full: max={max_slots}")]
    SlotsFull { max_slots: u32 },
}

impl RuleFailure for SummonFailure {
    fn code(&self) -> &'static str {
        match self {
            // 已有匹配...
            Self::SlotsFull { .. } => "SUMMON_SLOTS_FULL",
        }
    }
}
```

### 第四步：在领域代码中使用

```rust
// src/core/domains/summon/systems.rs
fn create_summon(
    owner: Entity,
    template: &SummonTemplate,
) -> Result<Entity, SummonFailure> {
    // 检查槽位
    let current_count = count_summons(owner);
    let max_slots = get_max_summon_slots(owner);
    
    if current_count >= max_slots {
        return Err(SummonFailure::SlotsFull { max_slots });  // 业务规则失败
    }
    
    // ...
    Ok(summon_entity)
}
```

### 检查清单

- [ ] 确定错误类型（Error vs Failure）
- [ ] Error 变体派发了 `Event`
- [ ] Failure 变体实现了 `RuleFailure` trait
- [ ] 错误码命名遵循 `{域前缀}_{失败类型}` 格式
- [ ] 在领域代码中正确使用 `Err()` 返回

---

## 9. 现状盘点：已经做了什么，还缺什么

### 已实现

| 组件 | 状态 | 说明 |
|------|------|------|
| ErrorContext<E> 包装器 | ✅ 完整 | 带领域标签的错误包装 |
| ContextExt trait | ✅ 完整 | 为 Result 提供 `.domain()` / `.with_context()` |
| RuleFailure trait | ✅ 完整 | 为 Failure 提供统一错误码接口 |
| 15 个领域的 Error 枚举 | ✅ 完整 | combat/crafting/inventory 等全部定义 |
| 15 个领域的 Failure 枚举 | ✅ 完整 | 实现 RuleFailure trait |
| Pipeline 错误处理 | ✅ 完整 | StepResult + PipelineError |
| Localization 错误 | ✅ 完整 | LocError（零依赖） |

### 阶段 2 迁移状态（ADR-051）— ✅ 全部完成

根据 ADR-051，阶段 2 迁移已全部完成：

| 域 | 程序错误数 | Failure 数 | 说明 |
|----|-----------|-----------|------|
| Combat | 5 | 4 | NotCombatParticipant, CombatNotStarted, CombatAlreadyEnded, EmptyTurnOrder, DamageAlreadyResolved |
| Inventory | 1 | 8 | ItemNotFound |
| Spell | 1 | 9 | SpellDefNotFound |
| CampRest | 2 | 4 | InterruptedTimeout, InvalidPhase |
| Progression | 0 | 7 | 已删除 error.rs（无程序错误） |
| Crafting | 0 | 7 | 已删除 error.rs（无程序错误） |
| Economy | 1 | 6 | ItemNotFound |
| Faction | 2 | 3 | FactionNotFound, RelationAsymmetry |
| Narrative | 2 | 4 | DialogueNodeNotFound, DialogueTreeHasCycle |
| Party | 2 | 7 | MemberNotFound, BondDefNotFound |
| Quest | 2 | 6 | QuestNotFound, ObjectiveNotFound |
| Reaction | 1 | 5 | SpecialNotRegistered |
| Summon | 1 | 5 | TemplateNotFound |
| Terrain | 3 | 7 | OutOfBounds, InvalidHazardDefinition, TileNotFound |

### 设计决策说明

| 组件 | 决策 | 说明 |
|------|------|------|
| ErrorContext/ContextExt | 🧊 冻结 6 个月 | ADR-051 决策：冻结至 2026-12-19，之后若仍无人使用则删除 |
| 统一错误日志 | ✅ 已有方案 | Error 通过 tracing 记录（参考 logging-overview.md），Failure 通过 `code()` 记录 |
| 错误恢复策略 | ✅ 不需要 | Failure 是正常业务结果，不需要恢复；Pipeline 已有 Retry 策略处理程序错误 |

---

## 10. 规则速查：该做什么和不该做什么

### ✅ 允许的

| 场景 | 做法 |
|------|------|
| 程序错误（不应该发生） | 定义在 `*Error` 枚举中，通过 `Err()` 传播 |
| 业务规则失败（正常结果） | 定义在 `*Failure` 枚举中，通过 `Result<T, Failure>` 返回 |
| 添加错误上下文 | 使用 `.domain()` 或 `.with_context()` |
| 获取统一错误码 | Failure 实现 `RuleFailure` trait |
| 基础设施层错误 | 不派发 Event，手动实现 Display |

### ❌ 禁止的

| 场景 | 为什么禁止 |
|------|-----------|
| Failure 派发 Event | Failure 是正常结果，不需要事件机制 |
| Error 不实现 RuleFailure | Error 是程序错误，不需要统一错误码 |
| 业务规则失败用 Error | 混淆语义，破坏错误处理架构 |
| 程序错误用 Failure | 混淆语义，破坏错误处理架构 |
| 在 Error 中写业务逻辑 | Error 只定义错误类型，不处理业务 |

---

## 参考文档

| 文档 | 内容 |
|------|------|
| `docs/00-governance/ai-constitution-complete.md` | 项目总宪法 |
| `docs/00-governance/coding-rules.md` §错误处理 | 错误处理编码规范 |
| `docs/01-architecture/40-cross-cutting/ADR-051-error-failure-separation.md` | Error/Failure 分离架构决策 |
| `src/shared/error/mod.rs` | ErrorContext<E> + ContextExt 定义 |
| `src/shared/traits/mod.rs` | RuleFailure trait 定义 |
| `src/core/domains/combat/error.rs` | 典型 Error 枚举示例 |
| `src/core/domains/combat/failure.rs` | 典型 Failure 枚举示例 |
| `src/core/capabilities/runtime/pipeline/foundation/types.rs` | Pipeline 错误处理 |
| `src/infra/localization/error.rs` | 基础设施层错误示例 |
