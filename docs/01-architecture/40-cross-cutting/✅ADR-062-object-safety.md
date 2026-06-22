---
id: 01-architecture.40-cross-cutting.ADR-062
title: "ADR-062: Object Safety 分层策略（热路径泛型/冷路径 dyn）"
status: Accepted
owner: architect
created: 2026-06-21
updated: 2026-06-21
tags:
  - architecture
  - object-safety
  - generics
  - cross-cutting
---

# ADR-062: Object Safety 分层策略（热路径泛型/冷路径 dyn）

## 状态

**Accepted** — 已被架构委员会接受。

## 背景

项目代码中存在两类对 trait object 的不同需求：

1. **热路径**：战斗执行、属性计算、Effect 应用等高频调用路径。这些路径需要极致性能，应使用泛型静态分发（monomorphization），避免动态分发的 vtable 间接调用开销和 inline 屏障。
2. **冷路径**：编辑器、Mod 系统、诊断工具等低频调用路径。这些路径更关注灵活性和可扩展性，允许使用 `dyn Trait` 动态分发。

当前代码中存在以下 Object Safety 相关的问题：

- 部分 trait（如 `StrongId`）被标记为 object-safe，但实际使用中从未通过 `dyn` 调用
- `PipelineHook` 为保持 Object Safety 限制了方法签名，导致热路径中无法使用泛型参数传递上下文
- 架构规则规定"Registry + Trait Object 替代 match"，但未明确该规则的作用域，导致冷路径和热路径使用同一模式

需要正式制定 Object Safety 分层策略，明确哪些 trait 必须保持 object safe、哪些允许非 object safe，以及不同路径下的 trait 设计标准。

## 引用的领域规则

- `docs/01-architecture/40-cross-cutting/ADR-057-sealed-trait.md` — 密封策略影响 trait 的 object safety 设计，框架级密封 trait 需按热/冷路径分类确定 object safety 要求
- `docs/01-architecture/40-cross-cutting/ADR-058-derive-macro.md` — Derive 宏自动注册与 Reflect 协同，宏生成的 trait impl 需遵循 object safety 分层策略
- `docs/01-architecture/40-cross-cutting/ADR-013-registry-hotreload.md` — Registry + Trait Object 模式，需明确其在热路径和冷路径中的适用范围
- `.trae/rules/架构规则.md` — 模块间依赖规则和 trait 设计规范

## 决策

### 1. 分层标准

| 路径 | 分发策略 | 适用场景 | 性能要求 |
|------|---------|---------|---------|
| **热路径** | 泛型静态分发（`impl Trait` / 泛型参数） | 战斗执行、属性计算、Effect handler、Condition checker | 极致性能，零开销抽象 |
| **冷路径** | 动态分发（`dyn Trait`） | 编辑器、Mod 系统、诊断工具、Inspector | 灵活优先，允许小幅性能损失 |

### 2. Object Safety Trait 设计原则

**热路径 trait**（非 object safe 允许）：
- 允许关联类型、泛型方法、`Self: Sized` 约束
- 不要求 `dyn` 兼容
- 优先使用泛型参数而非 `Box<dyn Trait>`

**冷路径 trait**（必须 object safe）：
- 禁止关联类型（除非满足 object safety 约束）
- 禁止泛型方法
- 禁止 `Self: Sized` 约束
- 禁止按值接收 `self`
- 方法返回类型必须是 object safe 的

**双路径 trait**（热路径和冷路径都需要）：
- 定义时使用 `where Self: Sized` 约束热路径方法
- 或拆分为两个 trait（一个泛型版本、一个 `dyn` 版本）
- 当热路径和冷路径方法签名差异过大时，优先拆分 trait

### 3. 现有 Trait 分类

| Trait | 当前状态 | 分类 | 行动 |
|-------|---------|------|------|
| `PipelineHook` | Object safe | 双路径 | 保持 object safe。热路径通过泛型 wrapper 使用，冷路径通过 `Box<dyn PipelineHook>` 使用 |
| `StrongId` | Object safe | 热路径（但非 object safe 不影响） | 允许非 object safe。`StrongId` 仅在编译期使用，从未在运行时通过 `dyn` 调用 |
| `EffectHandler` | Object safe | 热路径 | 允许非 object safe。热路径使用泛型静态分发 |
| `ConditionChecker` | Object safe | 热路径 | 允许非 object safe。热路径使用泛型静态分发 |
| `DamageFormula` | Object safe | 热路径 | 允许非 object safe。热路径使用泛型静态分发 |
| `RuleFailure` | Object safe | 双路径（错误收集） | 保持 object safe。错误在冷热路径均需收集，struct 本身满足 object safe |
| `DomainEvent` | Object safe | 双路径 | 保持 object safe。事件需要在热路径发送、冷路径回放和诊断 |
| `ObservableEvent` | Object safe | 冷路径 | 保持 object safe。仅用于诊断和 Observable 系统 |

### 4. Registry + Trait Object 的作用域限定

架构规则"Registry + Trait Object 替代 match"（详见 ADR-013）限定在冷路径：

- **冷路径**：使用 `Registry<Box<dyn Trait>>` 模式，替代大型 match 表达式
- **热路径**：使用泛型注册表 `Registry<T>` 或编译期枚举分发，保持零开销抽象

### 5. PipelineHook Object Safety 策略

`PipelineHook` trait 保持 object safe，以支持两种使用方式：

```rust
// 冷路径：通过 dyn 动态注册
let hook: Box<dyn PipelineHook> = Box::new(MyHook);
pipeline.register_hook(hook);

// 热路径：通过泛型直接使用
fn run_pipeline<H: PipelineHook>(hook: H) { ... }
```

PipelineHook 的 object safe 约束：
- 允许 `&self` 和 `&mut self` 方法
- 允许返回 `Box<dyn Any>` 或 `()`
- 禁止泛型方法
- 禁止按值消费 `self`

### 6. StrongId Object Safety 策略

`StrongId` 允许非 object safe，因为该 trait 仅在编译期用于类型约束和值构造，代码库中无任何 `dyn StrongId` 使用场景。允许其释放 `Sized` 约束或关联类型等非 object safe 特性，以提供更灵活的 ID 类型定义。

## Module Design

本决策不产生新的模块，是跨模块的 trait 设计规范。需更新以下文件：

| 文件 | 更新内容 |
|------|---------|
| `.trae/rules/架构规则.md` | 新增 Object Safety 分层规则 |
| `docs/01-architecture/40-cross-cutting/ADR-013-registry-hotreload.md` | 补充 Registry + Trait Object 场景限定 |

### 代码审查锚点

审查代码时检查以下要点：
- 热路径中是否存在不必要的 `Box<dyn Trait>` 包装
- 冷路径 trait 是否满足 object safety 要求
- 双路径 trait 是否通过 `where Self: Sized` 或 trait 拆分实现
- `Registry + dyn Trait` 是否在冷路径范围内使用

## Communication Design

本决策不涉及 ECS 通信机制（Hook/Trigger/Observer/Message）。Object Safety 是 trait 定义和分发策略的约束，与运行时通信机制无关。

## 边界定义

### 允许
- 热路径 trait 允许不满足 object safety
- 冷路径 trait 必须满足 object safety
- 双路径 trait 使用 `where Self: Sized` 分发不同路径的方法
- `PipelineHook` 保持 object safe，同时支持泛型和 `dyn` 使用
- `StrongId` 允许非 object safe（实际无 dyn 使用场景）

### 禁止
- 禁止在热路径中使用 `Box<dyn Trait>` 包装（除非经过性能评测验证需要）
- 禁止冷路径 trait 包含非 object safe 方法
- 禁止 `Registry + dyn Trait` 模式扩展到热路径
- 禁止为满足 object safety 在热路径 trait 中引入堆分配或装箱

## Forbidden（禁止事项）

- **🟥 禁止在热路径 trait 中强制保持 object safe** — 热路径（战斗执行、属性计算）优先性能，允许关联类型、泛型方法、`Self: Sized` 约束，不得因 object safety 要求牺牲性能
- **🟥 禁止在冷路径 trait 中引入非 object safe 方法** — 冷路径（编辑器、Mod、工具）trait 必须满足 object safety，禁止泛型方法和关联类型，确保 `dyn` 分发的兼容性和灵活性
- **🟥 禁止 Registry + dyn Trait 模式扩展到热路径** — "Registry + Trait Object 替代 match"限定在冷路径，热路径使用泛型注册表 `Registry<T>` 或编译期枚举分发，禁止混用
- **🟥 禁止为满足 object safety 装箱热路径类型** — 不得为满足 object safety 要求而将热路径类型包装到 `Box<dyn Trait>` 或 `Rc<RefCell<dyn Trait>>` 中，这引入不必要的堆分配和运行时开销
- **🟥 禁止双路径 trait 使用 `Box<dyn Any>` 类型擦除替代泛型参数** — 类型擦除降低类型安全，仅在跨 FFI 边界时作为最后手段，不得在正常的 trait 设计中使用
- **🟥 禁止在框架级 trait 中同时包含 object safe 和非 object safe 方法而不加 `where Self: Sized` 区分** — 混用两种签名导致调用者心智负担，必须显式使用 `where Self: Sized` 标记非 object safe 方法

## Definition / Instance Design

本决策不涉及 Definition/Instance 分离。Object Safety 是 trait 定义时的分发策略约束：

| 元素 | 适用性 | 说明 |
|------|--------|------|
| **Definition** | 不适用 | Object Safety 是 trait 设计期的约束，非配置数据 |
| **Instance** | 不适用 | Object Safety 不影响运行时 Component 或 Instance 数据结构 |
| **热路径/冷路径标注** | 代码注释 + 文档 | 每个 trait 需在文档注释中标注所属路径分类 |

## 后果

### 正面
- 热路径获得极致性能：泛型静态分发无 vtable 开销、inline 友好
- 冷路径获得灵活性：`dyn Trait` 动态分发支持运行时多态
- 清晰的"热 vs 冷"区分降低 trait 设计的认知负载
- PipelineHook 保持 object safe 满足冷热两种场景需求
- Registry + Trait Object 限域在冷路径，消除热路径性能隐患

### 负面
- 双路径 trait 需要额外设计（`where Self: Sized` 或 trait 拆分）
- 分类不准确可能导致性能回退或灵活性损失
- 需持续审查确保分类与实际使用一致

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 所有 trait 保持 object safe | 热路径类型引入装箱性能损失，违反零开销抽象原则 |
| 所有 trait 不要求 object safe | 冷路径无法使用 Registry + dyn Trait 模式，编辑器/Mod 系统复杂度上升 |
| 仅通过文档约定热/冷路径 | 无法自动审查，依赖人工纪律，易退化 |
| enum 替代 trait object 做静态分发 | 丧失扩展性，Mod 系统无法注册新类型 |

## 参考

- [Rust Reference — Object Safety](https://doc.rust-lang.org/reference/items/traits.html#object-safety)
- [Rust API Guidelines — Object Safety](https://rust-lang.github.io/api-guidelines/future-proofing.html)
- `docs/01-architecture/40-cross-cutting/ADR-013-registry-hotreload.md` — Registry + Trait Object 场景
- `docs/01-architecture/40-cross-cutting/ADR-057-sealed-trait.md` — 密封策略对 trait 设计的约束
- `docs/01-architecture/40-cross-cutting/ADR-058-derive-macro.md` — Derive 宏与 Reflect 协同
