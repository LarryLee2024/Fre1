---
id: capabilities.event.schema.v1
title: Event Schema — 事件通信数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: runtime
replay-safe: true
---

# Event Schema — 事件通信数据架构

> **领域归属**: Capabilities — 逻辑骨架层 | **依赖 Schema**: GameplayContext, Tag | **定义依据**: `docs/02-domain/event_domain.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `GameplayEvent` | Runtime | 统一事件结构（Tag + 上下文 + 数值载荷） |
| `EventTag` | Definition | 事件类型标签（用于路由和订阅匹配） |
| `EventSubscription` | Definition | 事件订阅关系 |

---

## 2. Problem

Event 是 Domain 间通信的「唯一通道」——所有跨领域通知必须经过 EventBus。Schema 必须解决：
- 统一事件结构（EventTag + GameplayContext + 数值载荷）
- 事件路由（基于 EventTag 匹配订阅者）
- 事件分发优先级（订阅者的执行顺序）
- 事件载荷中的核心数值

---

## 3. Schema Design

### 3.1 GameplayEvent（Runtime 层）

```rust
/// 统一的事件结构。
/// 所有跨 Domain 通信使用此结构，禁止定义独立的事件类型。
struct GameplayEvent {
    /// 事件唯一标识
    event_id: EventId,

    /// 事件类型标签（用于路由匹配）
    event_tag: EventTag,

    /// 事件数值载荷（如伤害量 25、治疗量 10、层数 3）
    magnitude: Option<f32>,

    /// 来源实体
    source_entity: EntityId,

    /// 目标实体（可选）
    target_entity: Option<EntityId>,

    /// 完整上下文（可选，重量级载荷）
    context: Option<GameplayContextData>,

    /// 创建帧号
    created_at_frame: u64,

    /// 事件优先级（影响分发顺序）
    priority: EventPriority,
}
```

### 3.2 EventTag（Definition 层）

```rust
/// 事件类型标签——使用标签层级体系，支持模糊匹配订阅。
/// 格式: "event.<category>.<action>"
///
/// 示例层级:
///   event.combat.damage_dealt
///   event.combat.heal_dealt
///   event.combat.unit_died
///   event.turn.started
///   event.turn.ended
///   event.spell.cast
///   event.spell.resolved
///   event.spell.concentration_broken
///   event.item.equipped
///   event.item.used
///   event.progression.level_up
///   event.quest.accepted
///   event.quest.completed
struct EventTag {
    /// 标签路径（如 "event.combat.damage_dealt"）
    path: String,

    /// 分类
    category: EventCategory,
}

enum EventCategory {
    Combat,
    Turn,
    Spell,
    Item,
    Progression,
    Quest,
    Dialogue,
    Movement,
    Terrain,
    Faction,
    Party,
    Custom(String),
}
```

### 3.3 EventPriority（Definition 层）

```rust
/// 事件分发优先级（影响订阅者的接收顺序）
#[repr(u8)]
enum EventPriority {
    /// 最高优先级——最先收到事件
    Critical = 0,
    /// 高优先级
    High = 25,
    /// 普通优先级（默认）
    Normal = 50,
    /// 低优先级
    Low = 75,
    /// 最后收到
    Last = 100,
}
```

### 3.4 EventSubscription（Definition 层）

```rust
/// 事件订阅关系。
struct EventSubscription {
    /// 订阅者标识
    subscriber_id: String,

    /// 关注的事件标签（支持通配符: event.turn.* 匹配所有 turn 事件）
    event_tag_pattern: String,

    /// 订阅者优先级（同事件下的接收顺序）
    priority: EventPriority,

    /// 是否只消费一次（fire-and-forget）
    one_shot: bool,

    /// 过滤条件（可选，只有满足条件的事件才投递）
    filter: Option<Condition>,
}
```

### 3.5 EventEnvelope（Runtime 层 — 分发时包装）

```rust
/// 事件在 EventBus 内部分发时的包装结构。
struct EventEnvelope {
    /// 原始事件
    event: GameplayEvent,

    /// 分发状态
    state: DispatchState,

    /// 已投递的订阅者列表
    delivered_to: Vec<String>,

    /// 失败的订阅者列表
    failed_to: Vec<(String, String)>, // (subscriber_id, error_message)
}

enum DispatchState {
    /// 已入队等待分发
    Queued,
    /// 正在分发给订阅者
    Dispatching,
    /// 所有订阅者已处理完毕
    Consumed,
    /// 分发过程中出现不可恢复错误
    Failed,
}
```

### 3.6 EventConfig（Definition 层 — 配置格式）

```yaml
# RON 配置示例 — 事件订阅配置
EventSubscriptionConfig:
  subscriptions:
    # Combat 领域订阅伤害事件
    - subscriber_id: "domain.combat"
      event_tag_pattern: "event.combat.*"
      priority: Normal

    # Spell 领域订阅施法事件
    - subscriber_id: "domain.spell"
      event_tag_pattern: "event.spell.*"
      priority: Normal

    # Quest 领域订阅击杀事件
    - subscriber_id: "domain.quest"
      event_tag_pattern: "event.combat.unit_died"
      priority: Low
      filter:
        TagRequirement:
          mode: HasAny
          target_tags: ["tag_000080"]   # CombatState.InCombat

    # UI 订阅所有事件（仅调试模式）
    - subscriber_id: "ui.debug"
      event_tag_pattern: "*"
      priority: Last
      one_shot: false
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 可热重载 | 备注 |
|----------|-------|--------|----------|------|
| `EventTag` | Definition | 代码枚举 | 否 | 事件类型 |
| `EventSubscription` | Definition | 代码注册 | 是 | 启动时注册 |
| `GameplayEvent` | Runtime | 否 | 否 | 瞬时事件 |
| `EventEnvelope` | Runtime | 否 | 否 | 分发包装 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → GameplayContextSchema | 事件携带 GameplayContext |
| 依赖 | → TagSchema | EventTag 使用标签体系 |
| 被依赖 | ← 所有 Domain | 所有 Domain 通过 Event 通信 |
| 被依赖 | ← TriggerSchema | Trigger 监听 Event 作为触发源 |

---

## 6. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | EventTag 格式正确 | 启动注册 | path 以 event. 开头并至少包含 3 段 |
| V2 | 订阅者不重复 | 启动注册 | 无重复 subscriber_id + event_tag_pattern |
| V3 | source_entity 存在 | 事件创建 | source_entity 必须有效 |
| V4 | 事件不可突变 | 分发中 | 投递过程中 EventEnvelope 的 event 不可变 |
| V5 | 事件不被多个系统消费 | 分发 | 每事件每个订阅者只收到一次 |

---

## 7. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| 事件发布 | 🟩 完全确定 | 由确定性行为触发（DamageDealt、TurnStarted 等） |
| 事件订阅分发 | 🟩 完全确定 | 基于 EventTag 的匹配确定 |
| 事件顺序 | 🟩 确定 | 按 EventPriority + 创建帧号排序 |

Event 系统是 Replay 的基础设施——回放时事件必须按录制时的顺序和内容精确重放。

---

## 8. Save Compatibility

事件是纯运行时通信机制，不参与存档持久化。

---

## 9. Migration Strategy

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2（未来） | 新增 EventCategory | 新增枚举 variant |

---

## 10. Future Extension

- **事件历史缓冲区**: 保留最近 N 个事件用于调试和回放
- **延迟事件**: 支持定时/条件触发的延迟事件
- **事件追踪**: 事件流转的跟踪日志（用于性能分析和 bug 排查）
- **事件过滤链**: 支持在事件投递前经过过滤链（如静默区域阻止事件触发）

---

## 11. Risks

| 风险 | 影响 | 缓解 |
|------|------|------|
| 事件风暴 | 连锁事件导致无限循环 | ContextChain 循环检测 + 每帧事件上限 |
| 事件丢失 | 订阅者处理失败导致事件未消费 | 失败记录 + 可恢复重试（最多 3 次） |
| 订阅者死锁 | A 系统等待 B 系统处理事件，B 等待 A | 强制事件处理为同步 + 非阻塞 |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| Domain 间仅通过事件通信 | ✅ | EventBus 是域间唯一通信通道 |
| Event 与 Trigger 职责分离 | ✅ | Event 是通知，Trigger 是条件→技能映射 |
| Replay First | ✅ | 事件顺序和内容确定 |
