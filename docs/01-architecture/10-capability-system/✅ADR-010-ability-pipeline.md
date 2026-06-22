---
id: 01-architecture.ADR-010
title: ADR-010 — Ability → Effect Pipeline Architecture
status: approved
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-010: Ability → Effect 管线架构

## 状态

**Approved** — 依赖 ADR-000（Feature Module Map）和 `docs/04-data/capabilities/ability_schema.md`，本架构决策正式生效。

## 背景

Ability（技能/能力）是游戏的"动词"——角色用它来产生效果。Ability 本身不包含行为逻辑（Data Law 004），它只描述 Cost/Cooldown/Targeting/Effects 的集合。Effect 是唯一的业务执行入口（Data Law 005）。需要一个清晰的管线将这个链路串联起来。

## 引用的领域规则与数据架构

- `docs/02-domain/capabilities/ability_domain.md` — Ability 领域规则
- `docs/02-domain/capabilities/effect_domain.md` — Effect 领域规则
- `docs/02-domain/capabilities/execution_domain.md` — Execution 领域规则
- `docs/02-domain/capabilities/targeting_domain.md` — Targeting 领域规则
- `docs/04-data/capabilities/ability_schema.md` — Ability Schema
- `docs/04-data/capabilities/effect_schema.md` — Effect Schema
- `docs/04-data/capabilities/execution_schema.md` — Execution Schema
- `.trae/rules/SRPG专项规则.md` §五 — 技能五阶段固定流程

## 决策

### 1. Ability Execution Pipeline 六阶段

所有 Ability 的执行严格遵循以下六阶段管线，禁止跳过或乱序：

```
Phase 1: Validate    ── 目标校验、条件检查、冷却检查
       │
       ▼
Phase 2: Pre-Cost   ── 扣消耗（SP/MP/Items）
       │
       ▼
Phase 3: Targeting  ── 确定最终目标集合
       │
       ▼
Phase 4: Execute    ── 生成 Effect 列表
       │
       ▼
Phase 5: Resolve    ── Effect 逐一执行（可能触发反应链）
       │
       ▼
Phase 6: Post-Cost  ── 后置消耗处理（冷却开始、充能消耗）
       │
       ▼
    Complete
```

### 2. Pipeline 阶段定义

```rust
/// Ability 执行管线 — 每个阶段一个独立 Event
#[derive(Event)]
pub enum AbilityPipelinePhase {
    /// Phase 1: 目标校验
    Validate {
        ability: Entity,       // AbilitySpec Entity
        caster: Entity,
        raw_target: TargetingIntent,
    },
    /// Phase 2: 前置消耗
    PreCost {
        ability: Entity,
        caster: Entity,
        targets: TargetSet,
    },
    /// Phase 3: 目标选择（确定最终目标）
    Targeting {
        ability: Entity,
        caster: Entity,
        candidates: TargetSet,
    },
    /// Phase 4: 效果生成（Ability → Effects）
    Execute {
        ability: Entity,
        caster: Entity,
        targets: TargetSet,
        effects: Vec<EffectInstance>,
    },
    /// Phase 5: 效果结算
    Resolve {
        ability: Entity,
        caster: Entity,
        targets: TargetSet,
        effects: Vec<EffectInstance>,
    },
    /// Phase 6: 后置消耗
    PostCost {
        ability: Entity,
        caster: Entity,
        result: AbilityResult,
    },
}
```

### 3. Data Flow: Ability → Effect

```
AbilityDef (Asset)
  │
  ├── cost: CostDef           # 消耗定义
  ├── cooldown: CooldownDef   # 冷却定义
  ├── targeting: TargetingDef # 目标选择定义
  └── effects: Vec<EffectDef> # Effect 列表（唯一业务行为）
         │
         ▼
AbilitySpec (Component, 运行时)
  │
  ├── ability_def_id: AbilityDefId
  ├── level: u8
  ├── cooldown_remaining: u8
  └── snapshot: AbilitySnapshot   # 施法时的快照值
         │
         ▼
EffectInstance (运行时, 每 Effect 一个 Entity)
  │
  ├── effect_def: EffectDefId
  ├── source: Entity          # 施法者
  ├── target: Entity          # 目标
  ├── snapshot: EffectSnapshot # 快照（含属性值、暴击率等）
  └── state: EffectState
         │
         ▼
    Effect Resolution
      ├── Modifier (属性修改)
      ├── Cue (表现信号)
      └── Trigger (连锁事件)
```

### 4. Pipeline 各阶段 System 职责

| 阶段 | System | 输入 | 输出 |
|------|--------|------|------|
| Validate | `validate_ability_system` | `AbilityCastEvent` | `ValidateResult` |
| PreCost | `pre_cost_system` | `ValidateResult` | `CostResult` |
| Targeting | `targeting_system` | `CostResult` | `TargetSet` |
| Execute | `execute_ability_system` | `TargetSet` + `AbilitySpec` | `Vec<EffectInstance>` |
| Resolve | `resolve_effect_system` | `EffectInstance` | `EffectResult` |
| PostCost | `post_cost_system` | `EffectResult` | `AbilityCompleteEvent` |

### 5. 管线编排策略

使用 Bevy System 的显式排序表达管线阶段：

```rust
// plugin.rs
fn build(&self, app: &mut App) {
    app.add_event::<AbilityCastEvent>()
       .add_systems(Update, (
            validate_ability_system,
            pre_cost_system,
            targeting_system,
            execute_ability_system,
            resolve_effect_system,
            post_cost_system,
       ).chain())  // ⬅ 使用 .chain() 强制串行执行
       .observe(on_ability_complete);
}
```

> `.chain()` 确保六个 System 按顺序执行，不需要手写状态机。如果某个阶段需要跳过多帧（如目标选择的 UI 等待），则在对应阶段插入 `run_if` 条件。

## Module Design

本管线横跨多个 Feature 模块，但管线编排归属 `execution/` 模块：

```
src/core/capabilities/ability/
  ├── components.rs      — AbilitySpec, AbilityInstance
  ├── systems/
  │   ├── validate.rs    — 校验系统
  │   └── pre_cost.rs    — 前置消耗系统
  └── events.rs          — AbilityCastEvent, AbilityCompleteEvent

src/core/capabilities/targeting/
  └── systems.rs         — targeting_system

src/core/capabilities/execution/
  ├── systems.rs         — execute_ability_system (Ability → Effect 转换)
  └── pipelines.rs       — Pipeline 编排（chain 顺序）

src/core/capabilities/effect/
  ├── components.rs      — EffectInstance, ActiveEffect
  ├── systems/
  │   └── resolve.rs     — resolve_effect_system
  └── events.rs          — EffectResult
```

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| Ability 激活 → Pipeline 启动 | Message (`AbilityCastEvent`) | 外部 → execution |
| 阶段间数据传递 | System Parameter + Local | execution 内部 |
| 阶段完成 → 下一阶段 | `.chain()` 隐式排序 | execution 内部 |
| Effect 执行完成 → 外部通知 | Message (`AbilityCompleteEvent`) | execution → 外部 |
| Effect → Modifier | `commands.trigger()` | effect → modifier（同 Entity） |
| Effect → Cue | `commands.trigger()` （CueSignal） | effect → cue |

## 边界定义

### 允许
- 外部 Feature 通过 `AbilityCastEvent` 发起能力执行
- Effect 执行时通过 Trigger 触发 Modifier/Cue
- 管线阶段读取 `AbilitySpec` 的 snapshot 字段

### 🟥 禁止
- Ability 跳过 pipeline 直接执行 Effect
- Effect 修改 `AbilityDef`（静态数据不可变）
- Targeting 阶段修改游戏状态（只读查询）
- Pipeline 中抛出的 Context 跨越到下一帧（管线必须单帧完成或显式暂停）

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| Ability 直接调用 Effect Applier | 必须经过 Execution Pipeline |
| Effect 绕过 Modifier Pipeline | Effect → Modifier 必须通过 Trigger |
| AbilityDef 运行时修改 | Definition 不可变 |
| 纯读阶段写入状态 | 违反 CQRS Lite 原则（SRPG §8.1） |
| 超过 3 层嵌套 Trigger 链 | 递归深度限制 |

## Definition / Instance Design

- **Definition**: `AbilityDef`, `EffectDef`, `CostDef`, `CooldownDef`, `TargetingDef`
- **Spec**: `AbilitySpec` (Component, Entity-bound)
- **Instance**: `EffectInstance` (Component), `ActiveEffect` (Component)
- **Persistence**: 存档时保存 `AbilitySpec`（含 snapshot）和 `ActiveEffect` 列表

## 后果

### 正面
- 六阶段流程与 SRPG 专项规则完全对齐
- `.chain()` 让管线顺序声明式可读
- Ability/Effect 完全分离，符合 Data Law
- 每阶段可独立测试

### 负面
- 六阶段管线对简单技能（如普攻）可能显得冗余
- 跨多个 Feature（ability/targeting/execution/effect）导致调试时需要在多个目录间跳转

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 单一大 System 处理所有阶段 | 违反单一职责，不可测试 |
| State Machine + OnEnter 驱动阶段 | 过度设计，`.chain()` 已覆盖简单顺序场景 |
| Behavior Tree | 当前复杂度不需要，将来 Mod/Scripting 需求时再考虑 |

## 评审要点

- [ ] PostCost 在 Resolve 之后是否合理？（冷却应该在效果完全结算后开始）
- [ ] 是否需要 Preview 模式（只计算不执行）？SRPG §8.1 要求预览/执行分离
- [ ] `.chain()` 是否足够表达所有阶段？当需要跨帧等待 UI 输入时怎么办？
