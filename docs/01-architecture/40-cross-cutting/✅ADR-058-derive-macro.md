---
id: 01-architecture.40-cross-cutting.ADR-058
title: "ADR-058: Derive宏+Trait组合模式（消除RuleFailure/DefinitionType样板代码）"
status: Accepted
owner: architect
created: 2026-06-21
updated: 2026-06-21
tags:
  - architecture
  - derive-macro
  - trait
  - code-generation
  - boilerplate
  - cross-cutting
---

# ADR-058: Derive宏+Trait组合模式

## 状态

**Accepted** — 已被架构委员会接受。

## 背景

项目中有大量类型需要为同一 trait 重复实现机械性的样板代码：

| Trait | 手动impl数量 | 代码性质 |
|-------|-------------|---------|
| `RuleFailure` | 17+ | 每个变体返回固定 `code()` 值 |
| `DefinitionType` | 15+ | 每个类型注册到全局 Registry |
| `DomainEvent` | 10+ | 同时实现 `Observable` + `Replayable` + `Auditable` |

这些实现逻辑高度相似、不包含业务判断，完全符合"宏只做重复结构"原则，是 derive 宏的合法使用场景。

## 决策

### 核心原则

当同一 trait 需要为 **10+ 类型**手动实现且逻辑高度相似时，必须创建 derive 宏自动生成，禁止继续手动重复。

### 业务价值优先级

按消除样板代码量和影响面排序：

1. **`#[derive(DomainEvent)]`** — 最高优先级。自动生成三个 trait impl（Observable + Replayable + Auditable），且事件是项目中新增最频繁的类型之一
2. **`#[derive(RuleFailure)]`** — 高优先级。每个 failure 变体只需标注 `#[code = "INV001"]` 属性，宏自动生成 `code()` 方法
3. **`#[derive(DefinitionType)]`** — 中优先级。自动生成 Registry 注册代码，与 `#[derive(Reflect)]` 协同工作

### 设计与约束

#### 与 Reflect 自动注册协同

derive 宏内部通过 `#[derive(Reflect)]` + `#[reflect(...)]` 属性组合，在生成 Trait impl 的同时完成 Bevy Reflect 类型注册：

```rust
// derive_macro 展开后等价于：
impl DomainEvent for MyEvent {
    fn event_name(&self) -> &'static str { "my_event" }
    fn replay(&self) -> ReplayAction { /* ... */ }
}
// 同时自动添加 Reflect 注册：
app.register_type::<MyEvent>();
```

两套注册逻辑统一在同一个 derive 宏中完成，禁止在插件中额外手动注册。

#### 边界条件

- 🟥 Derive 宏只生成**结构性样板代码**（Trait impl 的机械重复），禁止包含任何业务判断
- 🟩 必须通过 `cargo expand` 可查看展开结果，并在宏文档中说明展开内容
- 🟩 属性参数（如 `#[code = "INV001"]`、`#[event_name = "my_event"]`）必须是编译期常量
- 🟥 禁止通过 derive 宏参数传递闭包、函数指针等可执行逻辑
- 🟩 生成代码必须通过项目的 Clippy lint 检查（`#![allow(...)]` 按需添加，不污染外层）

## Module Design

### 宏 crate 结构

新增独立 proc-macro crate（建议名称 `fre-macros`），与主 crate 分开编译：

```
fre-macros/
├── Cargo.toml                    # [lib] proc-macro = true
├── src/
│   ├── lib.rs                    # 模块入口 + 公共辅助函数
│   ├── domain_event.rs           # #[derive(DomainEvent)]
│   ├── rule_failure.rs           # #[derive(RuleFailure)]
│   └── definition_type.rs        # #[derive(DefinitionType)]
```

### 各宏展开行为

| 宏 | 自动生成内容 | 属性参数 |
|----|-------------|---------|
| `#[derive(DomainEvent)]` | `DomainEvent` impl + `Observable` impl + `Replayable` impl + `Auditable` impl | `#[event_code = "XXX"]`, `#[event_name = "xxx"]` |
| `#[derive(RuleFailure)]` | `RuleFailure::code()` 方法 | `#[code = "DOMAIN_REASON"]` （枚举变体级别） |
| `#[derive(DefinitionType)]` | `DefinitionType::BUCKET_NAME` + `EXTENSION` + Registry 注册 | `#[bucket = "xxx"]`, `#[extension = "ron"]` |

## Communication Design

本决策不涉及 ECS 通信机制（Hook/Trigger/Observer/Message）。Derive 宏是编译时代码生成工具，不产生运行时通信。

## 边界定义

### 允许
- proc-macro crate 独立于主 crate，减少增量编译影响
- 宏属性参数用 `#[...]` 标注在类型或变体上
- `cargo expand` 可审查展开结果
- 宏内部调用 `#[derive(Reflect)]` 协同生成注册代码

### 禁止
- 禁止在宏中嵌入任何业务逻辑判断
- 禁止通过宏参数传递闭包或函数指针
- 禁止宏生成违反项目 Clippy lint 规则的代码（内部 `#[allow(...)]` 可接受）
- 禁止宏与手动注册混用（所有注册逻辑统一在宏中完成）
- 禁止宏生成的代码污染外层命名空间

## Forbidden（禁止事项）

- **🟥 禁止在 derive 宏中包含任何业务逻辑** — 宏只做机械性的样板代码生成（code() 映射、注册语句），禁止包含条件分支、规则判断等业务语义
- **🟥 禁止通过宏参数传递可执行逻辑** — 属性参数必须是编译期常量（字符串、整数、标识符），禁止传递闭包、函数指针、表达式
- **🟥 禁止宏与手动注册混用** — 一旦类型使用了 `#[derive(DomainEvent)]`，相关注册必须完全由宏完成，禁止在 plugin.rs 或其他位置再手动调用 `app.register_type::<T>()`
- **🟥 禁止 proc-macro crate 依赖主 crate** — proc-macro crate 必须保持独立，可依赖 `syn`、`quote`、`proc-macro2` 等基础设施，但禁止反向依赖主 crate 的任何类型
- **🟥 禁止为 < 10 个实现者的 trait 创建 derive 宏** — 避免过度抽象，"三次才抽象"原则适用于宏创建
- **🟥 禁止宏生成的代码产生 Clippy 警告** — 生成代码必须通过 `#![warn(clippy::all)]` 检查，如需豁免使用内部 `#[allow(...)]`，不污染外层

## Definition / Instance Design

本决策不涉及 Definition/Instance 分离。Derive 宏是编译期代码生成工具：

| 元素 | 适用性 | 说明 |
|------|--------|------|
| **Definition** | 不适用 | 宏生成的是 Trait impl 代码，非运行时配置数据 |
| **Instance** | 不适用 | 宏不影响运行时 Component 或 Instance 数据结构 |
| **proc-macro crate** | 独立的编译期依赖 | `Cargo.toml` 中声明 `[lib] proc-macro = true`，仅编译期需要 |

## 后果

### 正面
- 新增一个 `DomainEvent` 类型从 3 处手动 impl + 2 处注册 = 5 处修改减少到 1 个 `#[derive(DomainEvent)]` + 0 处注册
- 消除 `RuleFailure` 手动 impl 中常见的 copy-paste 遗漏错误
- `cargo expand` 输出可审查，透明可追溯

### 负面
- 需要一个独立的 proc-macro crate（`fre-macros`），增加构建时间约 1-2 秒
- proc-macro 调试较为困难，需要 `cargo expand` 辅助

## 替代方案

| 方案 | 缺点 | 结论 |
|------|------|------|
| 继续手动 impl | 重复代码多、新增类型易遗漏、Review 负担重 | 拒绝 |
| 用 `macro_rules!` 声明式宏 | 参数表达能力有限，复杂 trait impl 难以表达 | 仅适合简单场景 |
| 用 `build.rs` 代码生成 | 生成逻辑与源码分离、IDE 支持差 | 拒绝 |
| 用 `#[proc_macro_derive]` | 需要独立 proc-macro crate，构建复杂度增加 | 接受 — 收益大于成本 |

## 相关

- 约束来源：宪法 §16.5 抽象与宏使用规范
- 协同工作：`#[derive(Reflect)]` 类型注册
- 相关 ADR：ADR-054 (Bevy 0.19 迁移，Observer + 注册机制)
