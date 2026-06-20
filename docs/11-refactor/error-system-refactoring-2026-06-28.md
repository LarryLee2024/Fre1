# 错误处理系统激进重构计划

> 发现于 2026-06-28 全面错误处理架构评审
> 基于 ADR-051、.trae/rules/错误规则.md、docs/08-knowledge/error-handling-overview.md、docs/00-governance/ai-constitution-complete.md
> 优先级：P0（绝对禁止违规）→ P1（治理规则违规）→ P2（Error/Failure 分离）→ P3（错误质量）

---

## 评审发现的全部问题

### 文档问题

| ID | 严重度 | 描述 | 文件 |
|----|--------|------|------|
| D01 | P0 | Phase 2 状态与实际不符：标注 TBD 但代码已全部完成 | `docs/01-architecture/40-cross-cutting/ADR-051-error-failure-separation.md` |
| D02 | P1 | overview.md 说 "全部完成" 但 ADR-051 说 "TBD" — 两文档矛盾 | `docs/08-knowledge/error-handling-overview.md` vs `ADR-051` |
| D03 | P2 | 能力层（AbilityError、EffectError 等）不受 Error/Failure 分离规则覆盖 | ADR-051、错误规则.md、error-handling-overview.md 均未提及 |
| D04 | P3 | 旧 docs/02-domain 文档引用的错误类型可能已在 Phase 2 变更 | `docs/02-domain/domains/*.md` |

### 代码问题

| ID | 严重度 | 描述 | 位置 |
|----|--------|------|------|
| C01 | P0 | `pub use failure::*;` 公开暴露 Failure 类型，违反 ADR-051 红线 | `src/core/domains/combat/mod.rs:30` |
| C02 | P0 | `pub use failure::*;` + 完全无可见性注释 | `src/core/domains/camp_rest/mod.rs` |
| C03 | P0 | `pub use error::*;` 公开暴露 Error 类型，违反 ADR-045 可见性策略 | `src/core/domains/combat/mod.rs:28` |
| C04 | P0 | `pub use error::*;` 同上 | `src/core/domains/faction/mod.rs:23` |
| C05 | P0 | `pub use error::*;` 同上 | `src/core/domains/terrain/mod.rs:26` |
| C06 | P0 | 同上 + `pub use components::*;` + 无 ADR-045 注释 | `src/core/domains/camp_rest/mod.rs` |
| C07 | P1 | 手动 Display + Error impl，违反 thiserror 强制要求 | `src/content/loading/errors.rs` |
| C08 | P1 | 手动 Display + Error impl | `src/infra/localization/error.rs` |
| C09 | P1 | 手动 Display + Error impl | `src/core/capabilities/runtime/pipeline/foundation/types.rs` (PipelineError) |
| C10 | P1 | 手动 Display + Error impl | `src/core/capabilities/stacking/foundation/types.rs` (StackingError) |
| C11 | P2 | `InsufficientCost` 是业务规则失败，不应在 Error 中 | `src/core/capabilities/ability/foundation/types.rs` |
| C12 | P2 | `ConditionFailed` 是业务规则失败 | `src/core/capabilities/ability/foundation/types.rs` |
| C13 | P2 | `OnCooldown` 是业务规则失败 | `src/core/capabilities/ability/foundation/types.rs` |
| C14 | P2 | `SlotLimitReached` 是业务规则失败 | `src/core/capabilities/effect/foundation/types.rs` |
| C15 | P2 | `ConditionNotMet` 是业务规则失败 | `src/core/capabilities/effect/foundation/types.rs` |
| C16 | P3 | 使用位置 String 替代结构化字段 | `AbilityError::SpecNotFound(String)`、`EffectError::MissingSource(String)` 等 7 处 |
| C17 | P3 | 泛型 Runtime 变体 — 混淆所有运行时错误 | `AbilityError::Runtime(String)`、`EffectError::Runtime(String)`、`StackingError::Runtime(String)` |
| C18 | P4 | Capability 错误不派生 Event — 无法被 Observer 监听 | `AbilityError`、`EffectError`、`StackingError` 等 |
| C19 | P4 | `ErrorContext<E>` / `ContextExt` 被 0 个 domain 使用 — 冻结 6 个月到期 | `src/shared/error/mod.rs` |

---

## 重构阶段

### Phase 0: 文档急修（P0-P1 文档问题）

**目的**：消除文档与代码之间的矛盾。

| 任务 | 文件 | 操作 |
|------|------|------|
| 0.1 | ADR-051 | Phase 2 状态从 "TBD (Phase 2)" → "✅ 已完成 (2026-06-20)" |
| 0.2 | ADR-051 | 更新迁移优先级表为 ALL DONE，删除 "迁移禁止" 中过时的条款 |
| 0.3 | ADR-051 | 更新 "阶段 2 的迁移优先级" → "阶段 2 已完成，无待迁移项" |
| 0.4 | ADR-051 | 在 "Not covered" 节新增：能力层（Capability）错误不在 Error/Failure 分离范围内，作为已知设计缺口追踪 |
| 0.5 | error-handling-overview.md | §9 补充 Capability 错误状态（未分离，追踪中） |

**验证**：ADR-051 不再提到 Phase 2 TBD。

---

### Phase 1: 修复 P0 违规（mod.rs 可见性）

**目的**：消除所有 "绝对禁止" 级别的架构违规。

**任务 1.1: combat/mod.rs**

```rust
// 从:
pub(crate) mod failure;
pub use failure::*;
pub use error::*;

// 改为:
pub(crate) mod failure;
pub(crate) mod error;
// 删除: pub use failure::*;
// 删除: pub use error::*;
```

**任务 1.2: camp_rest/mod.rs**

```rust
// 从:
mod error;
pub use error::*;
mod failure;
pub use components::*;

// 改为:
// [ADR-045] pub(crate) — 领域错误定义，crate 内共享
pub(crate) mod error;
// [ADR-045] pub(crate) — 业务规则失败定义，crate 内共享
pub(crate) mod failure;
// 组件保持 private
mod components;
// 删除: pub use error::*;
// 删除: pub use components::*;
```

**任务 1.3: faction/mod.rs**

```rust
// 从:
pub use error::*;
// 改为: 删除该行，error 已经是 pub(crate)
```

**任务 1.4: terrain/mod.rs**

```rust
// 从:
pub use error::*;
// 改为: 删除该行
```

**验证**：
- `cargo build` 通过（可能需要修复外部对 `pub use error::*` 的依赖）
- `cargo nextest run` 通过
- 运行 `grep -rn "pub use.*error::\*" src/core/domains/` 确认 0 处
- 运行 `grep -rn "pub use.*failure::\*" src/core/domains/` 确认 0 处

---

### Phase 2: 错误类型从 types.rs 提取为独立文件（结构性重构）

**目的**：消除能力层 14 个模块中错误类型嵌入 types.rs 的结构性债，建立与 Domain 层一致的 "错误有专属文件" 模式。

**问题**：所有 capability 的 error 枚举都定义在 `foundation/types.rs` 中，与 TargetType、TargetShape、StackingType 等纯类型定义混在一起。这违反单一职责 + 一致性原则。

**目标结构**：

```
# 每个 capability:
foundation/
├── types.rs    ← ONLY 纯数据结构定义（枚举、结构体、值对象）
├── error.rs    ← 程序错误枚举（从 types.rs 提取）
├── failure.rs  ← 业务规则失败枚举（部分新建，部分提取）
├── mod.rs      ← 更新 re-export
└── ...
```

**迁移判断矩阵**（哪些需要 error.rs 提取，哪些需要 failure.rs 新建）：

| Capability | 当前错误 | 提取 error.rs | 新建 failure.rs | 难度 |
|-----------|---------|--------------|----------------|------|
| ability | `AbilityError` (10 var) | ✅ 提取 | ✅ `InsufficientCost`/`ConditionFailed`/`OnCooldown` | 中 |
| effect | `EffectError` (10 var) | ✅ 提取 | ✅ `SlotLimitReached`/`ConditionNotMet` | 中 |
| stacking | `StackingError` (3 var) | ✅ 提取 | ❌ 无需（无业务规则失败） | 低 |
| targeting | `TargetingError` (6 var) | ✅ 提取 | ❌ 无需 | 低 |
| execution | `ExecutionError` (6 var) | ✅ 提取 | ❌ 无需 | 低 |
| spec | `SpecError` (5 var) | ✅ 提取 | ❌ 无需 | 低 |
| cue | `CueError` | ✅ 提取 | ❌ 无需 | 低 |
| gameplay_context | `ContextBuildError` | ✅ 提取 | ❌ 无需 | 低 |
| pipeline | `PipelineError` (4 var) | ✅ 提取 | ❌ 无需 | 低 |
| command | `CommandError` (3 var) | ✅ 提取 | ❌ 无需 | 低 |
| registry | `RegistryError` (5 var) | ✅ 提取 | ❌ 无需 | 低 |
| replay | `ReplayError` (6 var) | ✅ 提取 | ❌ 无需 | 低 |
| scheduler | `SchedulerError` (4 var) | ✅ 提取 | ❌ 无需 | 低 |
| aggregator | `PipelineError` (2 var) | ✅ 提取 | ❌ 无需 | 低 |
| attribute | `AttributeRegistrationError` | ✅ 提取 | ❌ 无需 | 低 |
| modifier | `ModifierValidationError` | ✅ 提取 | ❌ 无需 | 低 |
| tag | `TagRegistrationError` | ✅ 提取 | ❌ 无需 | 低 |

**任务 2.x：通用提取模式**

每提取一个 capability，执行以下操作：

1. 在 `foundation/types.rs` 中找到 `pub enum *Error` 块，剪切到新建 `foundation/error.rs`
2. 给 error.rs 加上 proper docs + imports
3. 更新 `foundation/mod.rs` 的 re-exports（从 `pub use types::*;` 恢复原有精确 re-export，增加 `pub(crate) mod error;` 或相应可见性）
4. 更新所有 `use ...::types::*Error` → `use ...::error::*Error`

```rust
// types.rs 提取前：
pub enum AbilityError {
    NotReady { ... },
    ConditionFailed { ... },
    // ...
}

// types.rs 提取后：纯类型定义
// (AbilityError 被完全移除)

// error.rs（新建）：
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum AbilityError {
    #[error("ability '{spec_id}' not ready to activate (current state: {current_state:?})")]
    NotReady { current_state: AbilityState, spec_id: String },
    // ... 只保留程序错误变体（Phase 4 再做 Error/Failure 分离）
}

// mod.rs 更新前：
pub use types::*;

// mod.rs 更新后：
pub(crate) mod error;
pub use types::*;  // 保持 types 的 re-export 不变
// error 不 pub use，按 ADR-045 保持 pub(crate)
```

**任务 2.A: ability foundation** — 提取 AbilityError

**任务 2.B: effect foundation** — 提取 EffectError

**任务 2.C: stacking, targeting, execution, spec, cue, gameplay_context** — 6 个提取

**任务 2.D: pipeline, command, registry, replay, scheduler** — 5 个 runtime 提取

**任务 2.E: aggregator, attribute, modifier, tag** — 4 个提取

**验证**：
- `cargo build` 通过
- `cargo nextest run` 通过
- 每个 capability 的 `foundation/types.rs` 不再包含任何错误枚举
- 运行 `grep -rn "pub enum.*Error" src/core/capabilities/*/foundation/types.rs` 确认 0 处

---

### Phase 3: 修复 P1 违规（thiserror 迁移）

**目的**：消除所有 "禁止手写 Display impl" 违规。

**任务 3.1: content/loading/errors.rs**

将 `ConfigError` 和 `ValidationError` 从手动 Display/Error 迁移为 `#[derive(thiserror::Error)]`。

```rust
// 从:
#[derive(Debug, Clone)]
pub enum ConfigError {
    FileReadError { path: PathBuf, reason: String },
    // ...
}
impl std::fmt::Display for ConfigError { /* 30+ lines */ }
impl std::error::Error for ConfigError {}

// 改为:
#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read {path}: {reason}")]
    FileReadError { path: PathBuf, reason: String },
    // ...
}
// 删除: impl Display for ConfigError 块
// 删除: impl Error for ConfigError 块
```

`ValidationError` 同理。

**任务 3.2: infra/localization/error.rs**

```rust
// 从:
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum LocError { /* 4 variants */ }
impl fmt::Display for LocError { /* 40+ lines */ }
impl Error for LocError {}

// 改为:
#[derive(Debug, Clone, thiserror::Error)]
pub enum LocError {
    #[error("Key '{key}' not found in locale '{locale}' (fallbacks attempted: {fallbacks_attempted:?})")]
    KeyNotFound { key: String, locale: LocaleId, fallbacks_attempted: Vec<LocaleId> },
    // ...
}
// 删除: impl Display + impl Error 块
```

**任务 3.3: pipeline/foundation/error.rs**（提取后）
**任务 3.4: stacking/foundation/error.rs**（提取后）

将提取后的错误也使用 thiserror（当前 Phase 2 已提取，需确保使用 thiserror）。

**验证**：
- `cargo build` 通过
- `cargo nextest run` 通过
- 确认 `content/`、`infra/localization/`、`pipeline/`、`stacking/` 下无手动 Error impl

---

### Phase 4: Error/Failure 分离 — 能力层（语义重构）

**目的**：在结构性提取（Phase 2）之后，将能力层中的业务规则失败变体迁移到独立的 `*Failure` 枚举。

**设计决策**：能力层不是 Domain，不遵循 domain 的 error.rs/failure.rs 并列模式。改为在 capability 的 foundation 层新建 `failure.rs`。

**任务 4.1: AbilityFailure 分离**

从 `AbilityError` 中移出（在 error.rs 中完成提取后）：
- `InsufficientCost` → `AbilityFailure::InsufficientCost`
- `ConditionFailed` → `AbilityFailure::ConditionFailed`
- `OnCooldown` → `AbilityFailure::OnCooldown`

新建 `src/core/capabilities/ability/foundation/failure.rs`：

```rust
//! 规则失败 — Ability 能力层业务规则不满足结果。
//!
//! 与 `AbilityError`（程序错误）不同，这些是正常业务结果。
//! 详见 ADR-051

use crate::shared::traits::RuleFailure;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum AbilityFailure {
    #[error("condition check failed: {reason}")]
    ConditionFailed { reason: String },
    #[error("insufficient '{resource}': required {required}, available {available}")]
    InsufficientCost { resource: String, required: f32, available: f32 },
    #[error("ability '{spec_id}' on cooldown ({remaining_turns} turns remaining)")]
    OnCooldown { spec_id: String, remaining_turns: u32 },
}

impl RuleFailure for AbilityFailure {
    fn code(&self) -> &'static str {
        match self {
            Self::ConditionFailed { .. } => "ABILITY_CONDITION_FAILED",
            Self::InsufficientCost { .. } => "ABILITY_INSUFFICIENT_COST",
            Self::OnCooldown { .. } => "ABILITY_ON_COOLDOWN",
        }
    }
}
```

更新 `ability/foundation/mod.rs`：`pub(crate) mod failure;`

更新调用方：从 `Err(AbilityError::InsufficientCost {..})` → `Err(AbilityFailure::InsufficientCost {..})`

**任务 4.2: EffectFailure 分离**

从 `EffectError` 中移出：
- `SlotLimitReached` → `EffectFailure::SlotLimitReached`
- `ConditionNotMet` → `EffectFailure::ConditionNotMet`

新建 `src/core/capabilities/effect/foundation/failure.rs`。

更新 `effect/foundation/mod.rs`：`pub(crate) mod failure;`

**任务 4.3: 更新能力层测试**

所有测试中引用旧 `AbilityError::InsufficientCost` / `EffectError::SlotLimitReached` 的地方迁移到 `*Failure`。

**验证**：
- `cargo build` 通过
- `cargo nextest run` 通过
- `AbilityError` 不再包含业务规则失败变体
- `EffectError` 不再包含业务规则失败变体

---

### Phase 5: 错误质量提升 — 结构化字段

**目的**：消除所有位置 String 参数，改为命名结构字段。

**任务 4.1: AbilityError 结构化**

```rust
// 从:
SpecNotFound(String),
// 改为:
SpecNotFound { spec_id: String },
```

影响：调用方 `.map_err(|e| AbilityError::SpecNotFound(e.to_string()))` → `AbilityError::SpecNotFound { spec_id: id.into() }`

**任务 4.2: EffectError 结构化**

```rust
// 从:
MissingSource(String),
MissingTarget(String),
ConditionNotMet(String),  // (如 Phase 3 未删除)
EffectNotFound(String),
InvalidPeriod(String),

// 改为:
MissingSource { detail: String },
MissingTarget { detail: String },
EffectNotFound { effect_id: String },
InvalidPeriod { reason: String },
```

**任务 4.3: PipelineError 结构化**

```rust
// 从:
StageNotFound(String),
Aborted(String),
MissingContext(String),

// 改为:
StageNotFound { stage: String },
Aborted { reason: String },
MissingContext { key: String },
```

**任务 4.4: StackingError 结构化**

```rust
// 从:
InvalidConfig(String),
// 改为:
InvalidConfig { reason: String },
```

**验证**：
- `cargo build` 通过
- `cargo nextest run` 通过
- 检查 `grep -rn "Error::[A-Z][a-z]*(String" src/` 确认 0 处

---

### Phase 5: 消除泛型 Runtime 变体

**目的**：用具体语义变体替代泛型 `Runtime(String)` 兜底。

**任务 5.1: AbilityError::Runtime 替代**

检查所有 `AbilityError::Runtime(...)` 的使用处，为每个唯一模式创建具体变体：
- 常见的如 IO 错误 → `StorageError { detail: String }`
- 并发冲突 → `ConcurrentModification { detail: String }`
- 其余无法分类的 → 保留 `Runtime(String)` 但有注释说明剩余用途

**任务 5.2: EffectError::Runtime 替代**

同理，检查所有使用处。预期替代变体：
- Effect pipeline 执行异常 → `ExecutionFailed { stage: String, detail: String }`
- 剩余无法归类 → 保留但有注释

**任务 5.3: StackingError::Runtime 替代**

Stacking 层的 Runtime 通常来自 Modifier 应用失败 → `ApplicationFailed { detail: String }`

**验证**：
- `cargo build` 通过
- `cargo nextest run` 通过
- 每个保留的 Runtime 变体有注释说明何时使用

---

### Phase 6: Capability 错误 Event 派生

**目的**：为 Capability 错误添加 `Event` 派生，使其能被 Observer 监听。

> **设计考虑**：Domain 错误派生 Event 是因为它们需要被基础设施层（日志、告警）监听。Capability 错误的 Event 需求类似，但引入 Event 意味着增加 `bevy::prelude::*` 依赖。此阶段可单独评估必要性。

**任务 6.1**: 评估哪些 Capability 错误需要 Event 派生
- `AbilityError` — 高：技能激活失败需要被 combat domain 监听
- `EffectError` — 高：效果应用失败需要被 buff/debuff 系统监听
- `StackingError` — 中：堆叠失败通常是配置问题
- `PipelineError` — 低：管线内部错误
- `ExecutionError` — 中：执行器错误需要被调用方感知

**任务 6.2**: 为选定的 Error 枚举添加 `Event` 派生

**验证**：Bevy Observer 可以 `observe(|ev: Trigger<AbilityError>| { ... })`

---

### Phase 7: Domain 文档审计

**目的**：确保 `docs/02-domain/domains/*.md` 中引用的错误类型与代码一致。

**任务 7.1**: 对照每个 domain 的 error.rs + failure.rs，检查其 domain 文档中的错误引用

重点检查：
- combat_domain.md — 是否引用已删除的旧变体
- spell_domain.md — 同上
- inventory_domain.md — 同上（InventoryError 只有 1 个变体）
- 所有其他 domain

**任务 7.2**: 修复过时的错误引用

**验证**：`grep -rn "CombatError::InsufficientParticipants\|InventoryError::InventoryFull\|SpellError::Silenced" docs/02-domain/` 确认 0 处

---

### Phase 8: 工具脚本与最终验证

**目的**：创建检查脚本防止回归，验证全系统一致性。

**任务 8.1**: 创建 `tools/check-error-invariants.sh`

检查项：
1. 禁止 `pub use error::*;` — 确认 mod.rs 不使用通配符导出 error
2. 禁止 `pub use failure::*;` — 同上的 failure 版本
3. thiserror 使用检查 — 所有 `pub enum *Error` 和 `pub enum *Failure` 必须用 thiserror
4. 禁止裸 `String` 作为错误变体载荷 — 必须使用命名结构字段
5. 无 Runtime 变体（除非有注释）— 跟踪 Runtime 兜底
6. 所有 Failure 实现 RuleFailure — 使用 grep 确认

**任务 8.2**: ErrorContext 清理决策

根据 ADR-051 冻结期（至 2026-12-19）：
- 如还有 6 个月：在当前 refactoring 文档中标记清理计划
- 如已到 2026-12-19：删除 `src/shared/error/mod.rs` 中的 `ErrorContext` + `ContextExt`

**任务 8.3**: 最终验证

```bash
cargo build
cargo nextest run
cargo clippy -- -D warnings
tools/check-error-invariants.sh --ci
```

---

## 优先级汇总

| 阶段 | 任务 | 级别 | 估计文件变更 | 风险 |
|------|------|------|-------------|------|
| 0 | 文档急修 | P0 | 2 文档 | 低 — 纯文档变更 |
| 1 | mod.rs 可见性修正 | P0 | 4 文件 | 中 — 可能破坏外部引用 |
| 2 | 错误类型从 types.rs 提取到独立文件 | P1 | 14+ 模块 × 3 文件 | 中 — 影响 import 路径 |
| 3 | thiserror 迁移 | P1 | 4-6 文件 | 低 — 机械替换 |
| 4 | Capability Error/Failure 分离 | P2 | 6+ 文件 | 中 — 改变 API 契约 |
| 5 | 结构化字段 | P3 | 8+ 文件 | 中 — 影响调用方 |
| 6 | 消除 Runtime 泛型 | P3 | 3 文件 | 中 — 需审查每个使用点 |
| 7 | Event 派生 | P4 | 5+ 文件 | 低 — 添加 derive |
| 7b | Domain 文档审计 | P3 | ~15 文档 | 低 — 纯文档变更 |
| 8 | 工具脚本 + 验证 | P4 | 1 脚本 | 低 |

## 执行建议

1. **Phase 0 + 1 并行执行**（均 P0，无依赖关系）
2. **Phase 2 是最大变更**（14 个模块 × 3 步操作），建议分批执行：先提取 ability/effect，再批量提取其余
3. **Phase 3 在 Phase 2 之后**，因为提取后的新 error.rs 才能做 thiserror 迁移
4. **Phase 4 需要最多人工审查**（改变 API 契约，影响所有调用方）
5. **Phase 5 可半自动化**（grep 找所有使用处批量替换）
6. **Phase 6 每处 Runtime 使用点需要单独审查**（不能批量）
7. **Phase 7 + 7b + 8 可并行**（互相独立）
