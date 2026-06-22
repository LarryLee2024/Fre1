---
id: 01-architecture.40-cross-cutting.ADR-060
title: "ADR-060: Extension Trait for Bevy Types（EntityCommandsExt/QueryExt）"
status: Accepted
owner: architect
created: 2026-06-21
updated: 2026-06-21
tags:
  - architecture
  - extension-trait
  - entity-commands
  - cross-cutting
---

# ADR-060: Extension Trait for Bevy Types（EntityCommandsExt/QueryExt）

## 状态

**Accepted** — 已被架构委员会接受。

## 背景

项目中散落大量 free 函数（`add_buff(commands, ...)`, `heal(commands, ...)`），代码可读性差。这些 free 函数本质上是特定类型的"方法"，但因为 Rust 没有鸭子类型，它们被迫以 `fn foo(commands, ...)` 的形式散落在各处。

与此同时，项目已有 1 个 Extension Trait 实践（`ContextExt` in `shared/error`），证明该模式在项目中可行且有效。

## 决策

采用 Extension Trait 模式，为 Bevy 核心 ECS 类型添加领域操作扩展方法，将散落的 free 函数包装为方法语法糖：

1. **`EntityCommandsExt`** -- 为 `EntityCommands` 提供 `.add_buff()`, `.heal()`, `.kill()` 等方法
2. **`QueryExt`** -- 为 `Query` 提供 `.alive()`, `.hostile_to()`, `.in_range()` 等过滤方法

扩展方法内部调用 `integration/` 层的 Facade 函数，不直接操作 Capabilities 内部类型。Extension Trait 是"方法语法糖"——不封装业务逻辑，不替代领域函数。

## 模块设计

### 文件路径与结构

扩展 trait 定义位于 `src/core/domains/combat/integration/ext/`：

```
ext/
├── mod.rs                    # 导出两个 trait
├── entity_commands_ext.rs    # EntityCommandsExt 实现
└── query_ext.rs              # QueryExt 实现
```

### EntityCommandsExt

```rust
pub trait EntityCommandsExt {
    fn add_buff(&mut self, buff_id: SpecId);
    fn heal(&mut self, amount: u32);
    fn kill(&mut self);
}
```

### QueryExt

```rust
pub trait QueryExt<'w, 's, D: 'static, F: 'static> {
    fn alive(&self) -> impl Iterator<Item = Entity>;
    fn hostile_to(&self, faction: &str) -> impl Iterator<Item = Entity>;
}
```

### 命名约定

- 使用 `*Ext` 后缀（与现有 `ContextExt` 保持一致）
- `EntityCommands` -> `EntityCommandsExt`
- `Query` -> `QueryExt`

## 通信设计

本决策不直接引入 ECS 通信机制（Hook/Trigger/Observer/Message）。Extension Trait 的方法通过以下路径通信：

- 扩展方法内部委托给 `integration/` 层的 Facade 函数
- Facade 函数内部使用 Trigger/Observer/Message 完成跨域通信
- 扩展方法本身不发起 ECS 事件，仅作为入口语法糖，通信路径对调用方透明

## 边界定义

### 允许

- Extension Trait 作为方法语法糖，包装对领域函数/Facade 的调用
- 扩展方法内部调用 `integration/` 层的 Facade 函数，不直接操作 Capabilities
- 使用 `*Ext` 后缀命名约定，与 `ContextExt` 保持一致
- 渐进式引入：现有 free 函数可逐步包装为扩展方法，无需一次性迁移
- 返回 `Iterator<Item = Entity>` 等与 Bevy 链式操作兼容的类型

### 禁止

- 禁止在扩展方法中实现业务规则
- 禁止替代领域函数（Facade 函数仍然是主入口）
- 禁止直接操作 Capabilities 内部类型

## 禁止事项（Forbidden）

- **🟥 禁止在扩展方法中包含业务逻辑** — 扩展方法是方法语法糖，内部必须委托给 `integration/` Facade 函数或领域函数，禁止在扩展方法中实现条件分支、规则判断等业务语义
- **🟥 禁止扩展方法替代领域函数** — Facade 函数是主入口，扩展方法仅提供语法糖包装。领域函数保持独立可测试，所有通过扩展方法可执行的操作也必须能直接通过领域函数完成
- **🟥 禁止扩展方法直接操作 Capabilities 内部类型** — 扩展方法必须通过 `integration/` Facade 访问下层的 Capabilities，禁止直接引用 `capabilities/` 内部的 Component 或 Resource
- **🟥 禁止为核心库类型添加领域无关的扩展** — Extension Trait 只添加与项目领域操作相关的扩展方法，禁止添加通用工具方法（如数学运算、字符串处理等应放在 `shared/` 层）
- **🟥 禁止扩展方法持有或管理状态** — 扩展方法必须是纯语法糖，零状态，所有操作最终委托给底层领域函数
- **🟥 禁止扩展方法产生无法从领域函数复现的副作用** — 确保所有通过扩展方法执行的操作，也可以直接通过调用等价领域函数完成

## 定义 / 实例设计

本决策不涉及 Definition/Instance 分离。Extension Trait 是方法语法糖机制，不产生运行时数据：

| 元素 | 适用性 | 说明 |
|------|--------|------|
| **Definition** | 不适用 | 扩展方法是 trait 定义和 impl，非配置数据 |
| **Instance** | 不适用 | 扩展方法不引入新的运行时 Component 或 Instance 数据结构 |
| **Trait + impl** | 编译期代码组织模式 | 零运行时开销，方法调用经编译器单态化后与直接函数调用等价 |

## 影响评估

### 正面

1. 提升 ECS 操作流畅度：`commands.add_buff(id)` 比 `add_buff(&mut commands, id)` 更自然
2. 与 Rust 生态一致：Extension Trait 是 Rust 的标准实践
3. 与项目现有模式一致：复用 `ContextExt` 的 `*Ext` 命名约定
4. 渐进式引入：现有 free 函数可逐步包装为扩展方法，无需一次性迁移

### 负面

1. 需要额外导入 trait 才能使用方法
2. 过度使用可能导致"隐式魔法"——需通过规范约束

### 中性

1. 第一阶段为桩实现，后续逐步接入实际 Facade

## 替代方案

| 方案 | 说明 | 结论 |
|------|------|------|
| 保持 free 函数现状 | 继续使用 `add_buff(commands, ...)` 散落写法 | 拒绝 — 可读性差 |
| 使用 Newtype 包装 Bevy 类型 | 为 `EntityCommands` 创建包装类型 | 拒绝 — 失去与 Bevy 原生 API 的兼容性 |
| 单个 Mega-Trait 统一所有扩展 | 将 EntityCommandsExt 和 QueryExt 合并为一个 trait | 拒绝 — 违反单一职责，降低可组合性 |

## 参考

- ADR-057: Sealed Trait
- ADR-058: Derive Macro
- BEVY RFC #64: Extension traits patterns
- 现有实践：`ContextExt` in `src/shared/error/mod.rs`
