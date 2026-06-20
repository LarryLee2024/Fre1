---
id: 01-architecture.40-cross-cutting.ADR-061
title: "ADR-061: Typestate模式在Content Pipeline中的应用"
status: Accepted
owner: architect
created: 2026-06-21
updated: 2026-06-21
tags:
  - architecture
  - typestate
  - compile-time
  - pipeline
  - content-loading
  - cross-cutting
---

# ADR-061: Typestate模式在Content Pipeline中的应用

## 状态

**Accepted** — 已被架构委员会接受。

## 背景

项目中 Content Pipeline 的 Def 加载流程（ADR-047）分为五个阶段：Discovery → Loading → Validation → Registration → Notification。当前所有阶段之间的状态安全均在运行时通过断言或校验函数检查，编译期无法防止以下错误：

1. 未经验证的 Def 直接注册到 Registry
2. 跳过校验阶段的非法状态转换
3. 已冻结的 Def 被意外修改

随着 Def 类型增加到 19 个（AbilityDef、EffectDef、SpellDef 等），运行时检查的覆盖面和维护成本同步增长。

同时，项目宪法的§16.5 "三次才抽象" 原则对抽象时机有严格约束。Typestate 属于编译期类型安全机制，与运行时重复逻辑的抽象策略正交，不受该条款限制。

## 决策

### 1. 引入 Typestate 到 Content Pipeline

在 Def 加载流程中引入 Typestate 状态链，将状态编码到类型系统中：

```
DefBuilder<Unvalidated> → DefBuilder<Validated> → DefFrozen
```

| 状态 | 含义 | 允许操作 |
|------|------|----------|
| `Unvalidated` | 原始 RON 数据已加载但未校验 | 反序列化、格式转换 |
| `Validated` | 校验通过但未冻结 | ID/引用/数值校验 |
| `Frozen` | 已冻结的不可变 Definition | 注册到 Registry |

### 2. 编译期保证

- 只有 `DefFrozen` 类型暴露注册到 Registry 的方法
- `DefBuilder<Unvalidated>` → `DefFrozen` 的直接转换在编译期被阻止（必须经过 `Validated`）
- 状态转换函数是零开销抽象，运行时无额外成本

### 3. 与现有 Pipeline 的关系

- ADR-047 的 5 个阶段逻辑不变，Typestate 仅在类型层面约束阶段间的数据流
- Discovery → Loading 产生 `DefBuilder<Unvalidated>`
- Validation 产生 `DefBuilder<Validated>`
- Registration 只接受 `DefFrozen`
- Notification 阶段无需 Typestate，由事件系统驱动

### 4. 状态转换示意

```rust
// Phase 1-2: Discovery + Loading
let unvalidated: DefBuilder<Unvalidated> = DefBuilder::load(raw_data)?;

// Phase 3: Validation (返回 Result)
let validated: DefBuilder<Validated> = unvalidated.validate(ctx)?;

// Phase 4: Freeze + Register
let frozen: DefFrozen = validated.freeze();
registry.register(frozen);  // 只有 DefFrozen 可注册

// 编译期禁止：直接从未验证跳转到注册
// ❌ registry.register(unvalidated);  // 编译错误
// ❌ registry.register(unvalidated.freeze());  // freeze() 只在 Validated 上可用
```

## 边界

- 🟩 仅限 Pipeline/Builder 模式
- 🟩 仅适用于 Content Pipeline 的 Def 加载流程
- 🟩 与宪法§16.5「三次才抽象」正交——Typestate 是编译期类型安全，不受运行时抽象规则限制

## Forbidden

- 🟥 禁止 Typestate 化运行时状态机（运行时状态机用 enum 表示，不要用泛型参数）
- 🟥 禁止为 Typestate 引入复杂泛型体操（如多参泛型、GAT、泛型关联类型）
- 🟥 禁止在热路径使用 Typestate（Typestate 用于初始化阶段，不在帧循环中）
- 🟥 禁止将 Typestate 扩展到 Pipeline 引擎的运行时阶段（ADR-044 的 C3 Runtime Pipeline 使用 enum + Hook 模式，不改动）

## 理由

1. **编译期安全**：非法状态转换在编译期被阻止，消除一类运行时错误
2. **零开销**：Typestate 是纯编译期抽象，无运行时成本
3. **自文档化**：类型签名明确表达当前状态和允许的操作
4. **渐进可采**：仅在 Content Pipeline 引入，不影响其它系统
5. **符合 Rust 生态实践**：Typestate 是 Rust 中成熟的 Builder/Pipeline 模式

## 不引入 Typestate 的替代方案

| 方案 | 缺点 |
|------|------|
| 运行时标志位（`validated: bool`） | 运行时断言失败才报错，非编译期保证 |
| enum `DefState { Unvalidated, Validated, Frozen }` | match 分支完整性依赖开发纪律，无法阻止非法转换后调用 |
| 纯文档约束 | 零编译期保证，依赖人工审查 |

## 影响

### 正面

- Def 注册的安全性从运行时提升到编译期
- Pipeline 状态的合法性由类型系统证明
- 新增 Def 类型时，状态转换路径清晰

### 负面

- 引入少量泛型参数和类型标记
- 提升编译期复杂度（类型检查多一个步骤）
- 需要团队理解 Typestate 概念

### 风险与缓解

| 风险 | 缓解 |
|------|------|
| 类型系统复杂度失控 | Forbidden 条款禁止复杂泛型体操，仅使用空标记类型 |
| 团队不熟悉 Typestate | 体积极小（3 个标记类型 + 1 个 Builder），学习成本低 |
| 引入后无法回退 | Pipeline 实现保持模块化，Typestate 可隔离剥离 |

## 相关文档

- `ADR-047` — Content Loading Pipeline（Def 加载的五阶段流程）
- `ADR-044` — Pipeline Engine Architecture（C3 Runtime 管线引擎）
- `.trae/rules/架构规则.md` §Typestate 模式适用场景
- `.trae/rules/测试规范.md` §Typestate 测试
- `docs/04-data/infrastructure/pipeline_schema.md` §6 Typestate 编译期状态保证
