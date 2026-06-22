---
id: 01-architecture.40-cross-cutting.ADR-057
title: "ADR-057: 关键Trait密封性策略（框架级trait必须Sealed，扩展点trait保持开放）"
status: Accepted
owner: architect
created: 2026-06-21
updated: 2026-06-21
tags:
  - architecture
  - sealed-trait
  - trait-pattern
  - cross-cutting
  - api-stability
---

# ADR-057: 关键Trait密封性策略

## 状态

**Accepted** — 已被架构委员会接受。

## 背景

项目中存在两类公开 trait：

1. **框架级 trait**：定义为整个项目的核心契约，如 `StrongId`、`RuleFailure`、`PipelineHook`、`ObservableEvent`、`DefinitionType`。这些 trait 的不变量必须被严格保护。
2. **扩展点 trait**：定义为允许外部自定义行为的扩展接口，如 `EffectHandler`、`ConditionChecker`、`DamageFormula`。这些 trait 需要保持开放以支持 Mod 和自定义内容。

当前所有 trait 均处于完全开放状态，无法在编译期防止外部（尤其是 Mod 层或第三方代码）意外实现框架级 trait，这可能导致不变量被破坏、运行时行为异常或安全漏洞。

### 宪法交叉验证：DefinitionType 密封状态

宪法（`docs/00-governance/ai-constitution-complete.md` 第 1289 行）列出的框架级密封 trait 包括：`StrongId`、`RuleFailure`、`PipelineHook`、`ObservableEvent` 四项。本 ADR 额外列出 `DefinitionType`，理由如下：

- **已在代码中实际密封**：`src/content/loading/definition_type.rs` 中的 `DefinitionType` trait 已要求 `sealed::Sealed` supertrait，代码领先于宪法记载。
- **密封必要性**：`DefinitionType` 是内容加载管线的核心契约——它决定了哪些类型可以作为配置 Def 被资产系统加载、校验和注册。如果外部代码意外实现此 trait，可能绕过内容验证管线或引入非预期的配置加载行为。
- **已有 15+ 领域内实现**：所有核心 Def 类型（`SpellDef`、`AbilityDef`、`EffectDef`、`CueDef`、`RuleDef` 等）均已实现此 trait，其注册流程依赖 `BUCKET_NAME` 和 `EXTENSION` 常量，密封确保这些常量不会被外部实现篡改。

**建议**：更新宪法密封清单，将 `DefinitionType` 加入框架级密封 trait 列表。

## 决策

### 1. 框架级 Trait 使用 Sealed 模式

框架级公开 trait 必须使用 Sealed Trait 模式，防止外部实现破坏不变量。

### 2. 扩展点 Trait 保持开放

明确标识为扩展点的 trait 不进行密封，允许 Mod 层和第三方实现。

### 3. Sealed 模式标准实现

所有密封 trait 遵循统一实现方式：

```rust
// 在 trait 定义模块中添加：
pub(crate) mod sealed {
    /// Sealed trait — 防止外部实现。
    /// 仅 crate 内部类型可实现此 trait。
    pub trait Sealed {}
}

// Trait 定义要求 sealed::Sealed 作为 supertrait：
pub trait MyFrameworkTrait: sealed::Sealed + OtherBounds {
    // ...
}

// 为每个现有实现者添加 Sealed impl：
impl sealed::Sealed for MyType {}
```

关键设计要点：
- `sealed` 模块为 `pub(crate)`，确保只有 crate 内部可以访问
- `Sealed` trait 虽然是 `pub`，但由于模块可见性限制，外部 crate 无法实现
- `sealed::Sealed` 作为 supertrait 约束，外部类型无法满足

## Module Design

### 密封清单

以下 trait 必须使用 Sealed 模式：

| Trait | 所在模块 | 理由 |
|-------|----------|------|
| `StrongId` | `shared/ids/foundation/strong_id.rs` | ID 系统统一接口，保证所有 ID 类型行为一致 |
| `RuleFailure` | `shared/traits/mod.rs` | 规则失败标记，保证失败码格式统一 |
| `PipelineHook` | `core/capabilities/runtime/pipeline/hooks.rs` | 管线生命周期回调，禁止外部 Hook 篡改 |
| `ObservableEvent` | `shared/diagnostics/observable.rs` | 可观测事件契约，保证事件码和字段协议 |
| `DefinitionType` | `content/loading/definition_type.rs` | 配置定义接口，保证注册和校验流程正确 |

### 不密封清单

以下 trait 保持开放，不添加密封约束：

| Trait | 所在模块 | 理由 |
|-------|----------|------|
| `EffectHandler` | `core/capabilities/effect/...` | Mod 需要自定义效果处理逻辑 |
| `ConditionChecker` | `core/capabilities/condition/...` | Mod 需要自定义条件判断逻辑 |
| `DamageFormula` | `core/capabilities/execution/...` | Mod 需要自定义伤害公式 |
| 扩展点 trait | 各处 | 预留未来 Mod 支持 |

## Communication Design

本决策不涉及 ECS 通信机制（Hook/Trigger/Observer/Message）。Sealed Trait 模式是编译期契约保护机制，与运行时通信无关。

## 边界定义

### 允许
- 框架级公开 trait 应用 Sealed 模式，防止外部实现
- 扩展点 trait 保持开放，允许 Mod 层和第三方实现
- 在 trait 定义模块内嵌入 `pub(crate) mod sealed { pub trait Sealed {} }`
- 为每个现有实现者添加 `impl sealed::Sealed for T {}`

### 禁止
- 禁止对扩展点 trait 添加密封约束
- 禁止在 `sealed` 模块的 `Sealed` trait 中添加方法
- 禁止将 sealed 模块声明为 `pub`（必须是 `pub(crate)`）
- 禁止公开 re-export `Sealed` trait（保护不可实现性不因 re-export 被破坏）

## Forbidden（禁止事项）

- **🟥 禁止对扩展点 trait 应用 Sealed 模式** — EffectHandler、ConditionChecker、DamageFormula 等必须保持开放以支持 Mod 扩展
- **🟥 禁止在 sealed 模块中添加非空方法** — `Sealed` trait 必须保持空 trait 状态，仅用作实现约束标记
- **🟥 禁止将 sealed 模块对外公开** — `pub(crate)` 是密封生效的前提，模块可见性一旦提升则密封失效
- **🟥 禁止在公开 API 中暴露 `sealed::Sealed`** — 即使模块是 `pub(crate)`，也不应通过 `pub use` 或 re-export 泄露 Sealed trait
- **🟥 禁止新增框架级 trait 时跳过密封** — 所有新增框架级 trait 必须默认应用 Sealed 模式
- **🟥 禁止在宪法密封清单之外新增密封 trait 而不更新宪法** — ADR-057 是密封策略的权威记录，所有密封决策必须在 ADR 和宪法中对齐

## Definition / Instance Design

本决策不涉及 Definition/Instance 分离。Sealed trait 是编译期契约机制，不产生运行时数据：

| 元素 | 适用性 | 说明 |
|------|--------|------|
| **Definition** | 不适用 | Sealed 是 trait 定义时的可见性策略，非配置数据 |
| **Instance** | 不适用 | Sealed 不影响运行时 Component 或 Instance 数据 |
| **sealed::Sealed trait** | 编译期标记 trait | 空 trait，仅用作 supertrait 约束，零运行时开销 |

## 后果

### 正面
- 框架级 trait 的不变量在编译期得到保护
- 清晰的"密封 vs 开放"区分，降低使用者心智负担
- 统一的 sealed 实现模式，降低维护成本
- DefinitionType 密封已在代码层落地（`src/content/loading/definition_type.rs`），此 ADR 正式追认

### 负面
- 为所有现有实现添加 `impl sealed::Sealed for T {}` 样板代码
- Mod 层如需扩展 sealed trait 需要修改核心代码（但框架级 trait 本来就不应被 Mod 实现）
- 宏生成的类型需要在宏中添加 sealed impl（增加宏复杂度）

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 不密封，靠文档约定 | 无法编译期强制，容易遗漏 |
| 将 trait 放入私有模块再 re-export | 增加使用复杂度，不如 sealed supertrait 模式直接 |
| 使用 `#[doc(hidden)]` 标记 | 仅隐藏文档，不阻止实现 |
| 将框架代码独立为单独 crate | 架构成本过高，当前单体仓库阶段不适用 |

## 评审要点

- [ ] 密封与开放的区分是否合理？是否有 trait 被错误归类？
- [ ] sealed 模式的标准实现是否已纳入架构规则和 AI 宪法？
- [ ] 未来新增框架级 trait 时，是否会自动应用 sealed 模式？
- [ ] Mod 框架引入后，扩展点 trait 清单是否需要调整？
- [ ] 宪法密封清单是否需要更新以包含 DefinitionType？

## 参考

- [Rust Sealed Trait Pattern](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits)
- 宪法 §16.3 Sealed Trait 条款（第 1289 行）
- `src/content/loading/definition_type.rs` — DefinitionType 的 sealed 实现（参考基准）
- 架构规则 §架构模式补充
