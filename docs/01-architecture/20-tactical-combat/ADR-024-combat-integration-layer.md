---
id: 01-architecture.ADR-024
title: ADR-024 — Combat Domain integration/ Module
status: proposed
owner: architect
created: 2026-06-18
updated: 2026-06-18
supersedes: none
---

# ADR-024: Combat Domain integration/ Module Design

## Status

**Accepted** — 依赖 ADR-021（Turn State Machine）和 `docs/01-architecture/README.md` §6.2（Business Domain 标准结构），本架构决策自 2026-06-18 起生效。

## 背景

架构总纲 §6.2 规定每个 Business Domain 必须有一个 `integration/` 模块作为与 Capabilities 交互的唯一入口（Facade + SystemParam 模式），并明确禁止 Systems 直接 import Capabilities 组件类型进行字段访问。

当前 Combat 域违反此规则：`effect_tick_system.rs` 直接 import `ActiveEffectContainer`、`tick_durations`、`expire_effects` 等 Effect 能力层的内部类型与函数。随着 Combat 域后续接入更多 Capabilities（Execution、Targeting、Condition、Ability、Event、Trigger），若无 integration 层约束，直接 imports 将指数级增长，导致：

1. **架构边界腐化** — Capabilities 内部重构时需同步修改 Combat 域代码
2. **AI 误用风险** — 下游 System 可直接操作 Capabilities 内部字段，绕过业务语义校验
3. **测试维护成本** — 集成测试需要理解 Capabilities 内部类型而非 Combat 域视图类型

Tactical 域已建立完整的 integration 层模式（`tactical/integration/movement/`），本 ADR 将该模式标准化应用于 Combat 域。

## 引用的架构规则

- `docs/01-architecture/README.md` §6.2 — Business Domain 标准结构（integration/ 模块强制要求）
- `docs/01-architecture/README.md` §6.2 — 禁止 Systems 直接 import Capabilities 组件类型
- `docs/01-architecture/README.md` §3.3 — Capabilities/Domains 双轴依赖方向（Domain → Capabilities 单向）
- `docs/00-governance/ai-constitution-complete.md` §3.2 — Effect Pipeline 唯一入口规则
- `docs/02-domain/domains/combat_domain.md` §7 — Combat 域与 Capabilities 的对齐校验

## 决策

### 1. 创建 `src/core/domains/combat/integration/` 模块

遵循 Tactical 域已建立的 integration 架构模式，Combat 域创建自己的 Anti-Corruption Layer。

#### 文件结构

```
src/core/domains/combat/integration/
  ├── mod.rs              # re-export 所有子模块
  └── effect/             # 效果能力（Effect Capability）集成
      ├── mod.rs          # 模块入口 + re-export
      ├── facade.rs       # 业务语义 API（唯一调用 Effect 内部的地方）
      ├── types.rs        # Combat 视图类型（替代裸 Capabilities 类型）
      └── system_param.rs # Bevy SystemParam（封装 Effect 查询依赖）
```

#### 未来子模块预留

```
src/core/domains/combat/integration/
  └── execution/          # 执行能力（Execution Capability）— 计划 Phase C-2
  └── targeting/          # 目标选择（Targeting Capability）— 计划 Phase D
  └── condition/          # 条件检查（Condition Capability）— 计划 Phase D
  └── event/              # 事件桥接（Event Capability）— 预留
```

子模块按"需要时再创建"原则添加，不为未明确的未来需求提前设计。

### 2. Effect 集成模块详细设计

#### 2.1 facade.rs — 业务语义 API

Facade 层是唯一直接访问 `ActiveEffectContainer`、`tick_durations`、`expire_effects` 等 Effect 内部类型/函数的地方。Systems 和 Rules 通过 facade 函数交互，永远不直接 import Effect 能力层。

```rust
// ─── 读操作 ───

/// 构建实体的效果视图。
/// 封装对 ActiveEffectContainer 的只读查询。
pub fn build_effect_view(container: &ActiveEffectContainer) -> EffectView { ... }

/// 检查实体是否有活跃效果。
pub fn has_active_effects(container: &ActiveEffectContainer) -> bool { ... }

/// 检查实体是否有指定 def_id 的效果在生效。
pub fn has_active_effect_by_def(container: &ActiveEffectContainer, def_id: &str) -> bool { ... }

// ─── 写操作（Combat 域唯一允许调用 tick_durations / expire_effects 的地方） ───

/// 推进单次效果 Tick（1 回合）。
/// 封装 tick_durations + 事件收集。
pub fn tick_all_effects(
    container: &mut ActiveEffectContainer,
    current_turn: u64,
) -> EffectTickOutcome { ... }

/// 清理到期的 Expiring 效果。
/// 封装 expire_effects。
pub fn expire_all_effects(
    container: &mut ActiveEffectContainer,
) -> Vec<String> { ... }

/// 推进+清理一步到位（合并 Debt-D9-002 优化）。
pub fn tick_and_expire(
    container: &mut ActiveEffectContainer,
    current_turn: u64,
) -> EffectTickOutcome { ... }
```

#### 2.2 types.rs — Combat 视图类型

```rust
/// 效果 Tick 执行结果（替代直接暴露 TickResult）。
#[derive(Debug, Clone)]
pub struct EffectTickOutcome {
    pub ticked: Vec<String>,       // 触发了周期 Tick 的 effect instance IDs
    pub expired: Vec<String>,      // 本轮转为 Expiring 的 effect instance IDs
    pub error_count: usize,        // Tick 过程中出现的错误数
}

/// 实体的效果视图（查询用）。
#[derive(Debug, Clone)]
pub struct EffectView {
    pub total_effects: usize,
    pub active_effects: Vec<EffectSummary>,
}

/// 单个效果的摘要。
#[derive(Debug, Clone)]
pub struct EffectSummary {
    pub instance_id: String,
    pub def_id: String,
    pub remaining_turns: i64,
    pub stage: String,
    pub paused: bool,
}
```

#### 2.3 system_param.rs — Bevy SystemParam

```rust
/// 效果 Tick 查询参数 — 封装所有 Effect Capabilities 依赖。
#[derive(SystemParam)]
pub struct EffectTickParam<'w, 's> {
    pub turn_queue: Res<'w, TurnQueue>,
    pub container_query: Query<'w, 's, &'static mut ActiveEffectContainer>,
}

impl EffectTickParam<'w, 's> {
    /// 对所有实体执行 tick + expire 一步到位。
    pub fn tick_all(&mut self, commands: &mut Commands) -> Vec<EffectTickOutcome> { ... }
}
```

### 3. effect_tick_system 重构方案

重构 `on_turn_end_tick_effects` Observer 使用 integration 层：

- **重构前**: 直接 import `ActiveEffectContainer`, `tick_durations`, `expire_effects`, `EffectTicked`
- **重构后**: 仅 import `EffectTickParam` + `EffectTicked`（事件类型仍需直接 import）

```rust
pub(crate) fn on_turn_end_tick_effects(
    _trigger: On<'_, '_, OnTurnEnd>,
    mut commands: Commands,
    mut effect_tick: EffectTickParam,
) {
    let outcomes = effect_tick.tick_all(&mut commands);
    // 日志输出（outcomes 替代裸 result 处理）
}
```

**收益**: Observer 文件从 71 行缩减至 ~20 行，capabilities imports 从 3 个降至 0 个（EffectTicked 保留），双重迭代问题（Debt-D9-002）在 `tick_and_expire` 中自然解决。

### 4. 效果发射事件处理

`EffectTicked` 事件是 Effect Capability 公开的事件类型，不在 integration 的"禁止直接 import"范围内。Observer 仍可直接 import `EffectTicked` 用于 `commands.trigger()`。Facade 的 `tick_all` 方法返回 `EffectTickOutcome` 供 Observer 决定如何发射事件，但具体事件触发（commands.trigger）保留在 Observer 层面——这属于"事件发布"而非"字段访问"。

### 5. 与现有代码的兼容过渡

1. 新建 `integration/effect/` 模块（本 ADR）
2. 保留现有 `effect_tick_system.rs`，内部改为调用 facade 函数
3. 后续 `@refactor-guardian` 验证 integration 层覆盖后，再将旧 imports 清理标记为已解决
4. 后续子模块（execution/ 等）不需要迁移——直接基于 integration 层开发

## Module Design

```
src/core/domains/combat/
  ├── integration/           ← 新增（Anti-Corruption Layer）
  │   ├── mod.rs
  │   └── effect/
  │       ├── mod.rs
  │       ├── facade.rs
  │       ├── types.rs
  │       └── system_param.rs
  ├── plugin.rs              — 无变更
  ├── components.rs          — 无变更
  ├── events.rs              — 无变更
  ├── api.rs                 — 无变更
  ├── systems/
  │   ├── mod.rs             — 无变更
  │   ├── turn_systems.rs    — 无变更
  │   └── effect_tick_system.rs — 内部重构（调用 facade + SystemParam）
  └── tests/                 — 测试无变更（已覆盖 integration 逻辑）
```

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| OnTurnEnd → Effect tick | Observer (existing) | Combat domain → Effect capability (via integration) |
| Effect tick → EffectTicked | Trigger (existing) | Effect capability → Combat + external observers |
| Integration facade | Pure function call | combat/integration → effect/mechanism/lifecycle |
| Integration SystemParam | Bevy SystemParam | combat/systems → combat/integration |

## 边界定义

### 允许
- Systems 通过 `integration::effect::EffectTickParam` 访问 Effect 数据
- Systems 通过 `integration::effect::facade::*` 函数调用 Effect 能力
- Systems 直接 import Effect 公开事件类型（如 `EffectTicked`）

### 🟥 禁止
- Systems 直接 import `ActiveEffectContainer`、`tick_durations`、`expire_effects` 等 Capabilities 内部类型/函数
- Systems 在 integration layer 之外创建、修改 Capabilities 组件（如直接 `container.effects.push(...)`）
- Systems 调用 Capabilities 函数时未经过 facade 的校验流程
- Capabilities 层 import Combat 域的任何类型（保持依赖方向单向）

## Definition / Instance Design

本 ADR 不涉及新的 Definition/Instance 层。integration 层是纯 Facade 模式，无新增数据定义。

- **Definition**: 无变更（Effect 的 Definition 由 Effect Capability 管理）
- **Instance**: `EffectTickParam` (SystemParam, 运行时组合), `EffectTickOutcome` (瞬时值对象)
- **Persistence**: 无影响（Effect 持久化通过 ActiveEffectContainer 的 Save 系统处理）

## 后果

### 正面
- **架构合规** — 解决 Debt-D9-001，满足架构总纲 §6.2 的 integration/ 强制要求
- **边界强化** — 所有 Capabilities 访问经过单一入口，重构时只需修改 facade.rs
- **AI 安全** — 降低 AI 误用风险（直接 import Capabilities 内部类型在 review 时可被明确禁止）
- **双重迭代修复** — `tick_and_expire` 一次迭代完成，自然解决 Debt-D9-002
- **模式复用** — Tactical 域的 integration 模式被标准化复用，降低团队认知成本

### 负面
- **额外抽象层** — 对于当前仅 71 行的 effect_tick_system 有一定的"过度封装"感，但考虑到 Combat 域未来接入 5-10 个 Capabilities 子模块，抽象收益递增
- **文件数量增加** — 新增 4 个文件（mod + facade + types + system_param）

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 不创建 integration 层，保持现状 | 违反架构总纲 §6.2，已知 Debt-D9-001 待修复，未来 imports 失控 |
| 仅创建 facade.rs 不拆分子模块 | 当 5+ Capabilities 接入后，单一 facade.rs 膨胀为 God File，违反单一职责 |
| 在 combat/ 根目录创建 integration.rs | 违反"按能力域拆分"原则，与 tactical/integration 模式不一致 |
| 直接修改 AI 开发宪法解除禁止 | integrity 要求 Domains 用 integration/ 是架构原则，非 AI 约束问题 |

## 评审要点

- [ ] 是否所有 Capabilities imports 都应禁止直接 import？事件类型（EffectTicked）是否属于例外？
- [ ] `EffectTickParam.tick_all()` 是否应该自动发射 `EffectTicked` 事件，还是留在 Observer 中手动触发？
- [ ] 当前的 `tick_all` + `expire_all` 拆分是否有必要保留，还是直接合并为 `tick_and_expire`？
- [ ] types.rs 的 `EffectView` / `EffectSummary` 在初版是否过度设计？是否仅保留 `EffectTickOutcome` 即可？
- [ ] 未来 submodule 的预留文件名是否需要现在创建空文件？
