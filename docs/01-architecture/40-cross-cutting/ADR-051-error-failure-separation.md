---
id: 01-architecture.40-cross-cutting.ADR-051
title: "ADR-051: Error/Failure 分离架构（规则失败与程序错误严格区分）"
status: Accepted
owner: architect
created: 2026-06-19
updated: 2026-06-19
tags:
  - architecture
  - error-handling
  - failure
  - rule-failure
  - cross-cutting
  - error-separation
---

# ADR-051: Error/Failure 分离架构

## 状态

**Accepted**

## 背景

项目已完成错误处理架构审计，发现以下关键差距：

### 问题 1：规则失败与程序错误混合

`.trae/rules/错误规则.md` 要求严格区分两类不同性质的"错误"：

| 类别 | 本质 | 语义 | 示例 |
|------|------|------|------|
| **规则失败 (Rule Failure)** | 业务规则不满足，属于正常业务结果 | 不是 Bug，不是异常，是预期中的业务分支 | 背包满、MP 不足、装备条件不满足 |
| **程序错误 (Program Error)** | 系统异常，属于程序缺陷或环境问题 | 不应该发生的异常情况 | 配置不存在、ID 无效、IO 失败 |

但审计发现所有 domain 的 `*Error` 枚举同时包含两类变体：

```
CombatError (combat/error.rs):
  ├── ✅ 程序错误: NotCombatParticipant, CombatNotStarted, CombatAlreadyEnded, EmptyTurnOrder, DamageAlreadyResolved
  └── ❌ 规则失败 (应属于 CombatFailure): InsufficientParticipants, NotYourTurn, NoActionRemaining, UnitDead

InventoryError (inventory/error.rs):
  ├── ✅ 程序错误: ItemNotFound
  └── ❌ 规则失败 (应属于 InventoryFailure): InventoryFull, ExceedsWeightLimit, EquipConditionNotMet, SlotOccupied, InsufficientQuantity, ItemNotUsable, UniqueItemLimit, TwoHandedWeaponConflict

SpellError (spell/error.rs):
  ├── ✅ 程序错误: SpellDefNotFound
  └── ❌ 规则失败 (应属于 SpellFailure): InsufficientSlots, SpellNotKnown, SpellNotPrepared, Silenced, etc.
```

### 问题 2：重复变体间的冗余

当前 `*Error` 中的 Failure 变体带有 `"规则失败: "` 前缀标注（如 `InventoryError::InventoryFull` 的 error 字符串以 `"规则失败: "` 开头），这是临时的人肉标注，未从类型系统层面解决区分问题。

### 问题 3：Err 是程序错误的通道

将规则失败通过 `Err` 返回会污染调用方的错误处理语义：
- 调用方无法区分"这是预期中的业务结果"还是"系统出了异常"
- 错误传播链上的 `?` 操作符会无差别中断流程
- 回放/日志系统无法区分"正常业务分支"和"系统异常"

### Phase 1 已落地的实现

Phase 1 已在代码中落地了以下内容：

| 文件 | 内容 |
|------|------|
| `src/shared/traits/mod.rs` | `RuleFailure` trait（`fn code(&self) -> &'static str`） |
| `src/core/domains/combat/failure.rs` | `CombatFailure` 枚举（4 个变体） |
| `src/core/domains/inventory/failure.rs` | `InventoryFailure` 枚举（8 个变体） |
| `src/core/domains/spell/failure.rs` | `SpellFailure` 枚举（9 个变体） |

### ErrorContext / ContextExt 已冻结

`src/shared/error/mod.rs` 中的 `ErrorContext<E>` / `ContextExt` 当前被 **0 个 domain 使用**，已决策冻结 6 个月（至 2026-12-19），之后若仍无人使用则删除。

## 引用的领域规则

- `.trae/rules/错误规则.md` — 核心错误处理规则（失败分类标准、RuleFailure 模式、绝对禁令）
- `.trae/rules/审查规则.md` — 审查规则 Error handling 检查项（7 条）
- `docs/00-governance/ai-constitution-complete.md` §8 — 错误处理原则
- `docs/01-architecture/00-foundation/ADR-045-module-visibility-strategy.md` — 模块可见性（failure 模块 `pub(crate)` 依据）
- `src/shared/traits/mod.rs` — `RuleFailure` trait 实际代码
- `src/core/domains/combat/failure.rs` — 参考实现（CombatFailure）

## 决策

### 决策 1：语义边界 — Error vs Failure 的严格划分

**规则失败 (Failure)** 是业务规则不满足的结果，是正常业务分支：
- 特征：业务规则检查后返回，属于预期中的流程分支
- 标准：**如果业务规则有明确的"if-else"分支，else 分支中的限制性结果 = Failure**
- 判定指南：问"用户看到这个会认为是 Bug 吗？" — 如果答案是不认为是 Bug（如"背包已满"、"MP 不足"），则是 Failure；如果答案是觉得出 Bug 了（如"配置加载失败"），则是 Error
- 示例：背包满、法术位不足、装备条件不满足、非当前回合、负重超限

**程序错误 (Error)** 是系统异常，属于程序缺陷或环境问题：
- 特征：系统状态异常或外部资源不可用，调用方无法通过业务逻辑修复
- 标准：**如果问题不是业务规则限制导致的，而是系统/环境/配置问题 = Error**
- 示例：配置不存在、ID 无效、实体未找到、IO 失败、反序列化失败

**边界模糊案例处理规则**：
- 当某个条件同时涉及规则判断和系统状态时，若系统状态异常才是根本原因，归为 Error；若业务规则限制是根本原因，归为 Failure
- 例如："装备不存在"（ItemNotFound）= Error（数据问题）；"装备条件不满足"（EquipConditionNotMet）= Failure（规则限制）

### 决策 2：RuleFailure trait 职责边界

```rust
pub trait RuleFailure: std::fmt::Debug + Send + Sync + 'static {
    fn code(&self) -> &'static str;
}
```

**Trait 只提供 `code()`，不提供 `message()`**：
- `code()`：机器可读的错误码，格式 `"{DOMAIN}_{REASON}"`（全大写，下划线分隔），用于日志索引、UI 本地化 key、回放标识
- `message()` 不在 trait 内：人类可读消息由各 domain 通过 `thiserror::Error` 的 `#[error("...")]` 派生实现
- 理由：message 是领域语义的（不同国家语言、不同 UI 展示需求），code 是跨领域统一的（日志聚合、监控、回放确定性）

**Trait 不包含 `Display` bound**：
- Display 通过 `thiserror::Error` 自动派生，不放入 trait
- Failure 值可能被日志记录（通过 `Debug`），但不需要 Display 用于用户展示（UI 层通过 code 查本地化表）

**Trait bounds 选择理由**：
- `Debug`：日志记录和调试
- `Send + Sync`：跨线程传递（ECS System 中的错误传播）
- `'static`：类型擦除支持（`Box<dyn RuleFailure>`）
- 不包含 `Clone + PartialEq`：不是所有 failure 都需要相等比较，各 domain 按需 derive

### 决策 3：Failure 值的流通路径

**不走 `Err`，不走 `Event`，走直接返回值。**

```
规则检查函数
    │
    ├── 满足规则 → Ok(result_value)
    │
    └── 不满足规则 → Err(Failure)
                      │
                      ├── 调用方接收 Failure 值
                      ├── 判断业务分支（match / if-let）
                      ├── 可记录日志（通过 Debug + code）
                      └── 可转换为 UI 反馈（通过 code 查本地化表）
```

流通规则：
1. **不走 `Err` 通道**：Failure 虽名为 `Err(Failure)` 或通过专用结果枚举返回，但语义上不是异常，调用方应通过 `match` 或 `if-let` 处理
2. **不走 ECS Event 通道**：Failure 是同步判断结果，不是需要广播的领域事件。Event 用于"发生了某事"，Failure 用于"某件事能不能做"
3. **Return 路径**：Failure 值通过函数的正常返回值返回给直接调用方
4. **不跨系统传播**：Failure 在产生它的 system/function 边界内处理，不跨越 ECS System 边界传播（不同 system 间通过 Event 通信，而 Failure 不是 Event）

返回模式选择：

| 场景 | 推荐模式 | 示例 |
|------|---------|------|
| 仅失败路径需传值 | `Result<T, Failure>` | `fn try_equip(item: Entity) -> Result<(), EquipFailure>` |
| 成功/失败均需返回值 | 枚举包含成功 + 失败变体 | `enum TryEquipResult { Success, SlotOccupied { slot }, ... }` |
| 多层嵌套规则检查 | `Result<T, Failure>` + 早返回 | `let _ = check_slot()?; let _ = check_weight()?;` |

### 决策 4：模块设计 — failure.rs 与 error.rs 并列

每个 domain 的标准结构增加 `failure.rs` 文件：

```
domains/<domain>/
├── mod.rs           # pub(crate) mod failure;
├── failure.rs       # 规则失败枚举（pub(crate)）
├── error.rs         # 程序错误枚举（pub(crate)）
├── plugin.rs        # 唯一对外入口
├── components.rs    # ECS Components
├── systems/         # 业务系统
├── events.rs        # 领域事件
├── rules/           # 纯业务规则
└── integration/     # Anti-Corruption Layer
```

**failure 模块可见性**：`pub(crate)` — 遵循 ADR-045，域内共享，对外不暴露。

**mod.rs 导出方式**：

```rust
// mod.rs — 推荐
pub(crate) mod error;
pub(crate) mod failure;

// 不推荐使用 `pub use error::*; pub use failure::*;`
// failure 和 error 不在 domain 的 pub API 中暴露
```

**命名约定**：
- 文件名：`failure.rs`（与 `error.rs` 对称）
- 枚举名：`<Domain>Failure`（如 `CombatFailure`, `InventoryFailure`, `SpellFailure`）
- 错误码格式：`"{DOMAIN}_{REASON}"`（全大写，下划线分隔）

### 决策 5：代码迁移策略

当前 `*Error` 中同时包含 Error 变体和 Failure 变体。迁移分两阶段进行：

**阶段 1（✅ 已完成 — Phase 1）**：
- 创建 `failure.rs` 文件，定义独立的 `*Failure` 枚举
- 实现 `RuleFailure` trait，提供 `code()`
- 保留 `*Error` 中的重复变体不动（不破坏现有代码）
- 新代码优先使用 Failure 类型

**阶段 2（✅ 已完成 — Phase 2, 2026-06-20）**：
- 所有 15 个 domain 已完成 Error/Failure 分离迁移
- 所有 `*Error` 中的重复 Failure 变体已删除
- 所有 `"规则失败: "` 前缀标注已清理
- 所有 domain 已确认调用方正确处理 Failure 值（不走 Err 链传播）

> **注意**：上述阶段 2 迁移覆盖 Domain 层（combat、inventory、spell 等 15 个域）。能力层（Capability）的错误类型（AbilityError、EffectError 等）不在本次迁移范围内，作为已知架构缺口追踪。

## Module Design

### 模块文件组织

```text
src/core/domains/<domain>/
├── mod.rs                     # pub(crate) mod error; pub(crate) mod failure;
├── error.rs                   # [ADR-051] *Error 枚举 — 仅程序错误变体
├── failure.rs                 # [ADR-051] *Failure 枚举 — 仅规则失败变体，impl RuleFailure
├── plugin.rs                  # pub — Plugin 实现
├── components.rs              # ECS Components
├── systems/                   # 业务系统
├── events.rs                  # 领域事件
├── rules/                     # 纯业务规则
└── integration/               # Anti-Corruption Layer
```

### failure.rs 标准结构

```rust
//! 规则失败 — <Domain> 域业务规则不满足结果。
//!
//! 与 `<Domain>Error`（程序错误）不同，这些是正常业务结果，不应通过 `Err` 返回。
//! 详见 ADR-051

use crate::shared::traits::RuleFailure;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum <Domain>Failure {
    /// <变体说明，包括上下文参数>
    #[error("<human readable message: {param}>")]
    VariantName { /* contextual params */ },
    // ...
}

impl RuleFailure for <Domain>Failure {
    fn code(&self) -> &'static str {
        match self {
            Self::VariantName { .. } => "<DOMAIN>_<REASON>",
            // ...
        }
    }
}
```

### 已实现的 failure 模块

| Domain | 文件 | 变体数 | 错误码前缀 |
|--------|------|--------|-----------|
| Combat | `src/core/domains/combat/failure.rs` | 4 | `COMBAT_` |
| Inventory | `src/core/domains/inventory/failure.rs` | 8 | `INVENTORY_` |
| Spell | `src/core/domains/spell/failure.rs` | 9 | `SPELL_` |
| CampRest | `src/core/domains/camp_rest/failure.rs` | 6 | `CAMPREST_` |
| Crafting | `src/core/domains/crafting/failure.rs` | 7 | `CRAFTING_` |
| Economy | `src/core/domains/economy/failure.rs` | 6 | `ECONOMY_` |
| Faction | `src/core/domains/faction/failure.rs` | 3 | `FACTION_` |
| Narrative | `src/core/domains/narrative/failure.rs` | 4 | `NARRATIVE_` |
| Party | `src/core/domains/party/failure.rs` | 7 | `PARTY_` |
| Progression | `src/core/domains/progression/failure.rs` | 7 | `PROGRESSION_` |
| Quest | `src/core/domains/quest/failure.rs` | 6 | `QUEST_` |
| Reaction | `src/core/domains/reaction/failure.rs` | 5 | `REACTION_` |
| Summon | `src/core/domains/summon/failure.rs` | 5 | `SUMMON_` |
| Tactical | `src/core/domains/tactical/failure.rs` | 6 | `TACTICAL_` |
| Terrain | `src/core/domains/terrain/failure.rs` | 7 | `TERRAIN_` |

### shared/traits 中的 RuleFailure trait

```rust
// src/shared/traits/mod.rs
/// 规则失败标记 trait。
///
/// 业务规则不满足是正常结果（非 Err），与程序错误严格区分。
/// 每个 domain 独立定义各自的 `*Failure` 枚举并实现此 trait。
pub trait RuleFailure: std::fmt::Debug + Send + Sync + 'static {
    /// 返回机器可读的规则失败码。
    /// 格式：`"{DOMAIN}_{REASON}"`，如 `"COMBAT_NOT_YOUR_TURN"`。
    fn code(&self) -> &'static str;
}
```

### domain/mod.rs 导出模式

```rust
// mod.rs 中 failure 模块的可见性
pub(crate) mod failure;  // [ADR-045] [ADR-051] — 域内共享，不对外暴露

// 不公开 re-export failure 类型
// 外部通过 integration/ 中的 facade 方法间接使用 Failure 值
```

## Communication Design

**Failure 值不经过 ECS 通信通道**（Hook/Trigger/Observer/Message 均不适用）。

| 机制 | 适用性 | 理由 |
|------|--------|------|
| **Hook** | ❌ 不适用 | Hook 是 Component 生命周期行为，Failure 不是 Component |
| **Trigger** | ❌ 不适用 | Trigger 是 Entity 绑定的事件链，Failure 不是事件 |
| **Observer** | ❌ 不适用 | Observer 响应 Event，Failure 不是 Event |
| **Message** | ❌ 不适用 | Message 是跨域广播，Failure 是同步判断结果 |
| **Return Value** | ✅ **唯一路径** | Failure 作为函数返回值，由直接调用方处理 |

### Failure 值处理模式

```
┌─────────────────────────────────────────────────────────┐
│  调用方（System / Service / Function）                    │
│                                                         │
│  let result = try_apply_rule(entity, context);           │
│  match result {                                         │
│      Ok(value) => { /* 正常流程 */ }                     │
│      Err(failure) => {                                   │
│          // 1. 记录日志（通过 Debug + code）               │
│          info!(failure.code, "rule rejected: {failure:?}");│
│          // 2. 提供 UI 反馈（通过 code 查本地化表）         │
│          ui.show_localized(failure.code());              │
│          // 3. 不向上传播为 Err                           │
│          // 4. 切换业务分支                               │
│          return; 或 continue_with_alternative();          │
│      }                                                   │
│  }                                                       │
└─────────────────────────────────────────────────────────┘
```

### 限制传播范围

- Failure 值**不跨越 ECS System 边界**：如果 system A 检查规则产生 Failure，应当在 system A 内部处理完毕，不通过 Event 发送到 system B
- Failure 值**仅存在于函数调用栈内**：产生 → 处理 → 销毁，不在 ECS World 中持久化
- 日志记录是 Failure 唯一的"跨系统"影响（通过 `tracing::info!` 记录到日志文件）

## 边界定义

### 允许

- 每个 domain 定义独立的 `*Failure` 枚举，与 `*Error` 并列
- Failure 值通过函数返回值传递，不走 Err 异常链
- 调用方通过 `match` / `if-let` 处理 Failure 值
- Failure 值通过 `Debug` + `code()` 记录日志
- 使用 `thiserror::Error` 自动派生 Display
- 新 domain 在创建时同时创建 `error.rs` 和 `failure.rs`

### 禁止（永久有效）

- 禁止在 `*Error` 中新增 Failure 语义的变体 — Failure 变体必须放入 `failure.rs`
- 禁止将 Failure 值通过 ECS Event 发送
- 禁止将 Failure 值作为跨域通信的载荷
- 禁止将 Failure 值持久化到存档或回放记录
- 禁止在 `RuleFailure` trait 中添加 `message()` 或 Display bound
- 禁止在 shared 层定义全局 `GameResult<T, E>`（domain 各自选择返回模式）

### 过渡期规则（历史 — Phase 2 已于 2026-06-20 完成）

Phase 2 迁移期间（2026-06-19 → 2026-06-20）的过渡规则现已过期。迁移完成后：
- 所有 `*Error` 中的重复 Failure 变体已删除
- 所有 `"规则失败: "` 前缀标注已清理
- 所有 domain 已确认调用方正确处理 Failure 值

> 当前不存在待迁移的重复变体。如有新增 domain，应在创建时直接遵循 Error/Failure 分离。

## Forbidden（禁止事项）

### 绝对禁止

- 🟥 **禁止将规则失败 (Failure) 通过 `Err` 当作程序错误传播** — Failure 是正常的业务结果，不是异常
- 🟥 **禁止在 `RuleFailure` trait 中添加 `message()` 方法** — Display 由各 domain 通过 `thiserror` 自行实现，message 不属跨域契约
- 🟥 **禁止将 Failure 值通过 ECS Event/Message 跨系统广播** — Failure 是同步判断结果，不是事件
- 🟥 **禁止在 shared 层定义全局 `AppError`、`anyhow::Error`、`Box<dyn Error>`** — 分 domain 独立定义
- 🟥 **禁止在 domain 的 `*Error` 中新增 Failure 语义变体** — 新增 Failure 请放到 `failure.rs` 下

### ⚠️ 需要警惕

- ⚠️ Failure 值不要持久化到存档/回放 — 回放记录的是"发生了什么事"，不是"什么事做不了"
- ⚠️ 不要在多个 system 间传递同一个 Failure 值 — 在产生的 system 内处理完毕
- ⚠️ 不要在 `mod.rs` 中 `pub use failure::*` 公开 Failure 类型 — failure 模块是 `pub(crate)` 的域内实现

### 迁移禁止

- 🟥 **禁止在阶段 2 完成前删除 `*Error` 中的 Failure 变体** — 保证过渡期代码兼容
- 🟥 **禁止一次性迁移所有 Failure 变体** — 逐个 system/function 迁移，确保可测试

## Definition / Instance Design

本决策不涉及 Definition/Instance 分离：

| 元素 | 设计 | 说明 |
|------|------|------|
| **Definition** | 不适用 | Failure 定义是代码枚举，非配置数据 |
| **Instance** | 不适用 | Failure 值是运行时产生的临时值，非持久化状态 |
| **RuleFailure trait** | shared/traits 中的公共 trait | 提供统一的 code() 契约 |
| **`*Failure` 枚举** | 各 domain 独立定义 | 遵循 ADR-045 的 `pub(crate)` 可见性 |

## 后果

### 正面

- **语义清晰**：Failure 和 Error 在类型层面严格区分，调用方无需猜测"这到底是异常还是正常业务分支"
- **无害流通**：Failure 不走 Err 传播链，避免 `?` 操作符无差别中断流程
- **领域自治**：每个 domain 自主决定 Failure 类型和 code，shared 层只提供最小契约
- **日志友好**：统一的 `code()` 格式使日志聚合、监控告警、UI 本地化均可用同一 key
- **迁移路径明确**：两阶段迁移方案保证过渡期代码兼容，不阻塞现有开发

### 负面

- **文件数量增加**：每个 domain 增加一个 `failure.rs` 文件
- **过渡期重复**：阶段 2 完成前，`*Error` 和 `*Failure` 存在重复变体，增加维护成本
- **学习成本**：开发者需要理解 Failure vs Error 的语义差异并正确选择
- **调用方处理负担**：Failure 值需要调用方显式处理（match/if-let），不能通过 `?` 自动传播

## 替代方案

### 方案 A：Failure 也通过 Err 返回（当前混合状态）

保留现有混合模式，不分离 Failure 和 Error。

- **优点**：无需迁移，零改动
- **缺点**：调用方无法区分"预期内的业务分支"和"系统异常"，所有错误视为异常
- **结论**：❌ 拒绝 — 混合状态是问题根源

### 方案 B：Failure 通过 ECS Event 传播

将 Failure 值封装为 ECS Event 发送。

- **优点**：异步解耦，支持跨 system 响应
- **缺点**：Failure 是同步判断结果，事件化增加不必要的异步复杂度；Event 有消费顺序问题，无法保证在产生 system 的同一帧内处理
- **结论**：❌ 拒绝 — 用 Event 模拟函数调用（违反四级通信规范）

### 方案 C：通过 enum Result<(), Failure> 统一返回

所有规则检查函数返回 `Result<(), Failure>`。

- **优点**：统一模式，简单明确
- **缺点**：某些场景下调用方更希望得到专用的结果枚举而非通用 Result
- **结论**：✅ **采用为推荐模式** — 作为默认推荐，但不强制（允许 domain 选择专用结果枚举）

### 方案 D：向 RuleFailure trait 添加 message()

在 trait 中添加 `fn message(&self) -> &'static str` 或 Display bound。

- **优点**：trait 提供完整契约（code + message）
- **缺点**：message 是领域特定语义，不应由 shared 层统一约束；domain 通过 `thiserror` 派生 Display 更灵活
- **结论**：❌ 拒绝 — message 是领域职责，不是跨域契约

## 验证清单

- [x] 符合 DDD 三层 + 横切四层架构（Failure 在 domain 内部，不走跨域通道）
- [x] 符合 ADR-045 可见性策略（failure 模块 `pub(crate)`）
- [x] 符合 `.trae/rules/错误规则.md` 的失败分类标准
- [x] `RuleFailure` trait 仅含 `code()`，无 `message()` / Display bound
- [x] Failure 值通过函数返回值传递，不走 Err / Event
- [x] `*Error` 枚举中不再新增 Failure 语义变体
- [x] 所有 domain 的 Phase 2 迁移已完成（重复变体已删除、标注已清理）
- [x] Forbidden 列表已明确列出所有禁止行为
- [x] 与现有 ADR（045、046、049）无冲突

## 文件状态

| 文件 | 状态 | 负责人 | 完成日期 |
|------|------|--------|----------|
| `ADR-051-error-failure-separation.md` | ✅ accepted | architect | 2026-06-19 |
| `src/shared/traits/mod.rs` (RuleFailure trait) | ✅ stable | feature-developer | 2026-06-19 |
| 全部 15 个 domain 的 `failure.rs` | ✅ stable | feature-developer | 2026-06-20 |
| 全部 15 个 domain 的 `*Error` 重复变体清理 | ✅ complete | feature-developer | 2026-06-20 |

> **已知缺口**：能力层（Capability）错误类型（AbilityError、EffectError 等）尚未进行 Error/Failure 分离，原因见上文"Note"。该缺口由 `docs/11-refactor/error-system-refactoring-2026-06-28.md` 追踪。

## 后续更新

### D2-7: RuleFailure.code() 与 Explain trait calc_breakdown 的协同

`RuleFailure::code()`（`src/shared/traits/mod.rs`）与 Explain trait 的 `calc_breakdown()` 方法在以下场景产生协同效应：

**场景：Modifier 聚合失败的可解释性**

当 Modifier 聚合计算（通过 `aggregator` capability 的 `calc_breakdown()`）产生一个 Failure 结果时（如 `CombatFailure::InsufficientParticipants`），两种机制的协同流程如下：

```
calc_breakdown() 返回 BreakdownResult
    │
    ├── 成功 → Vec<BreakdownEntry>（正常聚合链路）
    │
    └── 失败 → 伴随对应的 RuleFailure 值
                  │
                  ├── code() → "COMBAT_INSUFFICIENT_PARTICIPANTS"
                  │              → 用于 UI 本地化 key
                  │              → 用于日志索引
                  │              → 用于回放标识
                  │
                  └── [error("...")] → "at least 2 participants required, got {count}"
                                         → 用于开发者调试
                                         → 用于 BreakdownEntry 的 human_readable 字段
```

**具体用例**：当 `CombatFailure::NotYourTurn` 作为 Modifier 聚合的"禁止执行"原因返回时，UI 层通过 `code()` 获取本地化 key 显示"不是你的回合"，同时 Breakdown UI 通过 `calc_breakdown()` 的返回值展示完整的 Modifier 应用链路（包括被拒绝的 Modifier 及其拒绝原因）。

**API 边界**：
- `RuleFailure::code()` — 跨域统一的机器可读标识符（用于日志、本地化、回放）
- `calc_breakdown()` — 域内的聚合明细展示（用于调试 UI、Modifier 可视化）
- 两者不直接互相调用，但共享相同的 Failure 值作为数据源
