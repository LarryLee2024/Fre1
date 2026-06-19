---
id: 01-architecture.ADR-002
title: ADR-002 — ECS Four-Tier Communication Strategy
status: approved
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-002: ECS 四级通信机制选型

## 状态

**Approved** — 通信机制架构选型已锁定。Observer 性能阈值由 @feature-developer 在 Spike 中验证确认，不阻塞后续实现。

## 背景

Bevy 0.19+ 提供多级通信原语：Hook（Component 生命周期）、Trigger（实体事件链）、Observer（状态变化响应）、Event/Message（全局广播）。错误的选择会导致：
- Observer 滥用 → 高频系统性能退化
- Event 滥用 → 全局耦合、调试困难
- Hook 承载复杂逻辑 → 生命周期管理混乱
- 本可用 Changed Filter 解决的场景却用了全套 Event + Observer

## 引用的领域规则与数据架构

- `.trae/rules/ECS规则.md` — 四级通信机制详细定义
- `.trae/rules/架构规则.md` — 跨模块交互规范
- `docs/04-data/README.md` — Data Law 005（Effect 唯一入口），Data Law 012（域间禁止直接数据引用）

## 决策

### 1. 通信机制决策矩阵

| 场景 | 推荐机制 | 不推荐 | 理由 |
|------|---------|--------|------|
| Component 添加/移除的轻量副作用 | **Hook** (`#[component(on_add, on_remove)]`) | Observer | Hook 声明式、零运行时开销；但不可承载复杂逻辑 |
| 同 Feature 内多段响应链 | **Trigger** (`commands.trigger()`) | Message | Trigger 轻量，天然绑定触发 Entity，Observer 在同一 World 内响应 |
| Component 值变化 → 表现更新 | **Changed Filter** | Observer | Changed 零分配，适合 UI 刷新、视觉同步 |
| Component 变化 → 其他逻辑响应 | **Observer** (`on_event`) | Hook | Observer 可以承载逻辑，适合血量变化触发 Buff 检测 |
| 跨 Feature 业务通知 | **Message** (`EventWriter/EventReader`) | 直接类型引用 | `events.rs` 声明 → 发送方不知接收方，完全解耦 |
| 领域事件（业务事实源） | **Message + WhiteList** | Trigger | 领域事件需要跨域可达、可录制、可回放 |
| 全局状态流转 | **State** (`States`/`SubStates`) | 手写枚举 | 官方 State 支持 OnEnter/OnExit，自带调度隔离 |
| 高频属性刷新（每帧） | **System + Changed** | Observer | Observer 在高频场景有调度开销 |
| 战斗事件链（伤害→护盾→吸血） | **Trigger** | 嵌套 Message | Trigger 绑定实体链，Observer 按顺序消费 |
| 指挥官命令 | **Command pattern**（独立 struct） | Event | Command 可录制、可 undo、可网络同步 |

### 2. 选择指南流程图

```
Component 生命周期事件？
├─ 是 → 纯轻量副作用？ → Hook
├─ 是 → 需要复杂逻辑？ → Observer
└─ 否
     │
     ▼
同 Feature 内事件链？
├─ 是 → 需要绑定触发 Entity？ → Trigger
├─ 是 → 纯状态查询？ → Changed Filter
└─ 否
     │
     ▼
跨 Feature 通知？
├─ 是 → 业务事实/需要录制？ → Message (WhiteList)
├─ 是 → 只需当前帧？ → Event with Reader
└─ 否
     │
     ▼
    直接 System 查询即可
```

### 3. 事件白名单管理

所有跨 Feature 的 Event 必须在 `events.rs` 中声明为白名单事件：

```rust
/// events.rs — Domain Events Whitelist
/// 所有在此登记的 Event 被认为是"业务事实源"，
/// 日志、回放、UI 均为消费者。
pub enum DomainEvent {
    /// 战斗结算完成
    CombatResolved {
        attacker: Entity,
        defender: Entity,
        result: CombatResult,
    },
    /// 单位死亡
    UnitDied { entity: Entity, killer: Option<Entity> },
    /// 回合阶段切换
    TurnPhaseChanged { old: TurnPhase, new: TurnPhase },
    /// 任务进度更新
    QuestProgressed { quest_id: QuestId, delta: u32 },
    /// ...
}
```

**规则**：
- 🟩 白名单事件必须通过 `info!(event = ?evt)` 输出结构化日志
- 🟩 白名单事件优先通过 Event 发送，禁用 Trigger（Trigger 绑定 Entity 不足以表达跨域事件）
- 🟥 禁止为临时副作用随意新增白名单事件

### 4. 通信安全策略

```rust
// 同 Feature 内部：使用 Trigger（轻量，绑定实体）
fn on_combat_hit(
    trigger: Trigger<HitEvent>,
    mut commands: Commands,
) {
    // 护盾响应 ← Observer
    // 吸血响应 ← Observer
    // 反击触发 ← Observer
}

// 跨 Feature：使用 Event（完全解耦）
fn on_unit_died(
    mut events: EventReader<DomainEvent>,
    mut quest_events: EventWriter<QuestCheckEvent>,
) {
    for event in events.read() {
        if let DomainEvent::UnitDied { entity, .. } = event {
            quest_events.send(QuestCheckEvent::new(*entity));
        }
    }
}

// 纯状态读取：使用 Changed Filter（零分配）
fn update_hp_bar(
    query: Query<&Health, Changed<Health>>,
    mut ui_query: Query<&mut UiStyle, With<HpBar>>,
) {
    // ...
}
```

## Module Design

通信相关的模块设计：

```
每个 Feature 的 events.rs
  └── 白名单事件声明 + DomainEvent 枚举（如果持有）

src/event/  (Layer 2)
  └── 全局 Event Bus 管理（如有需要）
```

## Communication Design

本 ADR 本身就是通信设计。

## 边界定义

### 允许
- 同 Feature 内任意使用 Trigger + Observer
- 跨 Feature 使用 Event（白名单登记）
- Feature 内部 System 之间使用直接函数调用（非事件化）
- 表现层使用 Changed Filter 监听业务状态

### 🟥 禁止
- 将普通函数调用事件化（如本可用 `push()` 解决的却发 Event）
- 高频循环中使用 Observer（性能退化）
- 跨 Feature 使用 Trigger（Trigger 是 Feature 内机制）
- Observers 中存在递归无保护（必须设深度限制）
- 业务代码直接操作 UI Components

## Forbidden

| 禁止行为 | 理由 | 替代方案 |
|---------|------|---------|
| 用 Message 模拟同 Feature 函数调用 | 全局耦合，性能浪费 | Trigger 或直接调用 |
| 用 Observer 做每帧更新 | Observer 调度开销大 | `Changed` Filter + System |
| 白名单事件不记日志 | 违反可观测性规范 | `info!(event = ?evt)` |
| 事件无递归深度限制 | 可能无限循环 | 设置 `MAX_OBSERVER_DEPTH` 常量 |
| 跨 Feature 用 Trigger | Trigger 绑定触发 Entity 不适配跨域场景 | Event |

## Definition / Instance Design

无直接数据结构产出。Event 类型属 Instance 层。

## 后果

### 正面
- 四种机制的适用场景清晰，开发者有据可依
- Trigger + Observer 替代了手写事件链的样板代码
- Changed Filter 覆盖了 80% 的"状态变化→响应"场景
- 白名单事件统一管理，日志/回放/UI 天然对齐

### 负面
- Trigger + Observer 是 Bevy 0.19 的新机制，团队需要学习
- 过量使用 Observer 可能导致隐式调度依赖难以追踪
- 白名单管理初期增加少量额外工作

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 全部使用 Event | 全局耦合，性能差，丢失 Entity 绑定信息 |
| 全部使用 Trigger + Observer | Trigger 绑定 Entity 不适用于跨域广播 |
| 全部使用 Changed Filter | 无法表达事件链和顺序依赖 |
| 自建 Event Bus | 重复造轮子，Bevy 原生机制已覆盖所有场景 |

## 评审要点

- [ ] 四种机制是否覆盖了所有预期的通信场景？
- [ ] 白名单登记流程是否足够简单（避免开发者绕过）？
- [ ] Observer 的递归深度限制经验值 10 是否合理？
- [ ] Changed Filter 是否可以用于 UI 层监听业务状态变化？
