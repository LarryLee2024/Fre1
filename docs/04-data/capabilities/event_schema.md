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

> **领域归属**: Capabilities — 逻辑骨架层 | **依赖 Schema**: GameplayContext, Tag | **定义依据**: `docs/02-domain/capabilities/event_domain.md`

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

- **延迟事件**: 支持定时/条件触发的延迟事件
- **事件追踪**: 事件流转的跟踪日志（用于性能分析和 bug 排查）
- **事件过滤链**: 支持在事件投递前经过过滤链（如静默区域阻止事件触发）

---

## 11. EventStore Schema — 事件历史存储

> 本文档 §10 最初提及"事件历史缓冲区"。本节正式定义 EventStore Schema，是 Event History 系统的数据层。


EventStore 是 EventBus 的附加 sink，记录所有通过 EventBus 分发的 `GameplayEvent` 的结构化快照。EventService 在将事件分发给订阅者的同时，将事件写入 EventStore。

### 11.1 StoredEvent

```rust
/// 存储在 EventStore 中的历史事件记录。
struct StoredEvent {
    /// 单调递增的事件序号（全局唯一）。
    event_id: u64,

    /// 事件创建时的帧号（用于时序分析和回放对齐）。
    timestamp: u64,

    /// 事件编码，关联 LogCode 枚举（如 BAT007、EFF001）。
    log_code: LogCode,

    /// 路由域——事件所属的业务域。
    domain: Domain,

    /// 关联标识（可选），用于串联一次完整战斗行为中的所有日志。
    /// 层级关系: BattleId → TurnId → ActionId
    correlation_id: Option<CorrelationId>,

    /// 事件类型标签（用于路由匹配，与 EventTag 对应）。
    event_tag: EventTag,

    /// 来源实体 ID（可选）。
    source_entity: Option<EntityId>,

    /// 目标实体 ID（可选）。
    target_entity: Option<EntityId>,

    /// 事件数值载荷（如伤害量 25、治疗量 10）。
    magnitude: Option<f32>,

    /// 结构化字段——从 ObservableEvent::record_fields() 收集的动态字段。
    fields: HashMap<String, String>,
}
```

### 11.2 EventStore

```rust
/// 事件存储——环形缓冲区实现。
///
/// - 定长 RingBuffer，容量在初始化时指定（默认 1000）
/// - 追加写入，超过容量时覆盖最旧记录
/// - 只读查询，不支持修改或删除已存储的事件
///
/// 所有权归属 `core/capabilities/runtime/event/`，与 EventBus 同域。
/// EventService 在事件分发后自动写入。
struct EventStore {
    /// 环形缓冲区。
    buffer: RingBuffer<StoredEvent>,

    /// 最大容量（不可变）。
    capacity: usize,
}
```

### 11.3 EventService 集成

```
[发布者 Domain]                  [EventBus]                     [事件历史]
      │                              │                              │
      │── GameplayEvent ──────────→  │                              │
      │                              │── EventStore.append(event) → │
      │                              │     (写入历史，异步非阻塞)     │
      │                              │                              │
      │                              │── (按 EventTag 匹配订阅者) → │
      │                              │── ...分发到订阅者...          │
```

EventService 在 `EventBus.publish()` 的调用链中增加一步：在事件入队分发之前，先将事件的快照写入 EventStore。写入必须是异步且非阻塞的——EventStore 作为观察者式 sink，不应影响事件分发主路径的延迟。

### 11.4 与 ObservableEvent 的关系

EventStore 利用现有的 `ObservableEvent` trait 提取事件的结构化字段：

```rust
// EventStore 内部实现示意（非代码规范，仅说明流程）：
fn record(&mut self, event: &impl ObservableEvent, metadata: EventMetadata) {
    let mut collector = FieldCollector::default();
    event.record_fields(&mut collector);

    let stored = StoredEvent {
        log_code: event.log_code(),
        domain: ObservableEvent::DOMAIN,       // const 泛型参数
        fields: collector.fields().iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
        // ... 其他字段从 metadata 提取
    };
    self.buffer.push(stored);
}
```

---

## 12. Query Interface — 事件历史查询

EventStore 提供基于索引的只读查询接口，用于调试工具、QA 回放分析、AI 行为分析等场景。

### 12.1 查询功能

```rust
/// 事件历史查询接口。
impl EventStore {
    /// 按关联标识查询（Battle、Turn、Action）。
    fn by_correlation(&self, id: CorrelationId) -> Vec<StoredEvent>;

    /// 按路由域查询（如 Combat、Progression）。
    fn by_domain(&self, domain: Domain) -> Vec<StoredEvent>;

    /// 按事件编码查询（如 BAT007）。
    fn by_log_code(&self, code: LogCode) -> Vec<StoredEvent>;

    /// 按事件标签查询（如 event.combat.damage_dealt）。
    fn by_event_tag(&self, tag: &EventTag) -> Vec<StoredEvent>;

    /// 按帧号范围查询。
    fn by_frame_range(&self, start: u64, end: u64) -> Vec<StoredEvent>;

    /// 获取最近的 N 条记录。
    fn recent(&self, n: usize) -> Vec<StoredEvent>;

    /// 获取单条记录。
    fn get(&self, event_id: u64) -> Option<StoredEvent>;
}
```

### 12.2 查询约束

| 规则 | 说明 |
|------|------|
| 只读 | 查询不修改 EventStore，所有查询方法返回不可变引用或克隆 |
| 按条件过滤 | 所有查询均为过滤操作——在 RingBuffer 中遍历匹配记录 |
| 容量溢出 | 超过 RingBuffer 容量的最旧记录自动丢弃，不可恢复 |
| O(n) 复杂度 | 基于 RingBuffer 的全量过滤，非索引查询。大数据量场景建议未来迁移到 SQLite |

### 12.3 典型查询场景

| 场景 | 查询参数 | 用途 |
|------|---------|------|
| 调试单次技能 | `correlation_id = ActionId(turn_id, seq)` | 追踪一次技能释放导致的所有效果 |
| QA 日志分析 | `by_frame_range(120, 180)` | 分析指定帧范围内的所有事件 |
| UI 战斗回放 | `by_correlation(CorrelationId::Battle(battle_id))` | 重建一局战斗的事件序列 |
| AI 行为分析 | `by_domain(Domain::Combat)` | 统计战斗事件分布和频率 |
| 定量分析 | `by_log_code(LogCode::BAT007)` | 统计伤害事件的总次数 |

---

## 13. Event History 与 Replay 的边界

Event History 和 Replay 是两个独立但互补的系统。下表明确二者的职责边界。

| 维度 | Replay（回放） | Event History（事件历史） |
|------|---------------|--------------------------|
| **录制的数据** | 输入命令（`RecordedCommand`）：玩家操作、AI 决策、RNG 种子 | 输出事件（`StoredEvent`）：`GameplayEvent` 的结构化快照 |
| **核心目的** | 确定性验证——同一输入 + 同一种子 = 同一结果 | 可观测性——记录"发生了什么事"供事后分析 |
| **数据完整性** | 必须完整记录每一帧，缺失即破坏确定性 | 允许容量溢出丢弃旧记录（RingBuffer 语义） |
| **持久化** | 序列化到 `.replay` 文件，可离线存储和回放 | 运行时内存，未来可扩展到 SQLite 持久化 |
| **数据流向** | `ReplayLog` → `ReplayPlayer` → 模拟输入 → 游戏逻辑 | `GameplayEvent` → `EventService` → `EventStore` |
| **使用者** | CI/CD 回归测试、官方回放质控 | 开发调试、QA 分析、UI 历史面板、AI 行为分析 |
| **版本依赖** | 强版本绑定——回放版本需匹配游戏版本 | 无版本绑定——事件 Schema 数据层向前兼容 |
| **因果关系** | 原因（输入） | 结果（输出事件） |
| **存储模型** | 全量逐帧序列 | 环形缓冲区（容量上限自动覆写） |

### 13.1 互补关系

```
      ┌─────────────────────────────────────────────────────────┐
      │                    游戏运行过程                          │
      │                                                         │
      │  [用户输入] ──→ [Command] ──→ [游戏逻辑] ──→ [Domain Events] │
      │       ↑                          ↓                     │
      │   Replay录                    EventStore              │
      │   (输入命令)                   (输出事件)                │
      │                                                         │
      │  QA 工作流:                                              │
      │  1. Replay 还原输入 → 复现 Bug                          │
      │  2. EventStore 查询输出 → 定位异常事件                    │
      │  3. 两者结合 → 完整因果链分析                             │
      └─────────────────────────────────────────────────────────┘
```

Replay 回答"输入是什么"；Event History 回答"输出了什么"。两者结合可构建完整的因果链（Replay → Command → EventStore → 结构化数据），显著提升 QA 和 Debug 效率。

### 13.2 配合使用场景

| 场景 | Replay 角色 | Event History 角色 |
|------|------------|-------------------|
| CI 回归测试 | 执行 Replay 验证确定性 | 断言关键事件（BAT007）的期望值和顺序 |
| Bug 复现 | 加载 Replay 还原输入序列 | 查询 EventStore 定位异常事件的精确帧和参数 |
| 战斗分析 | 不参与 | 按 BattleId 查询完整事件序列，用于定量分析 |
| Undo 系统 | 不参与（记录原始输入） | 记录事件序列，作为 Undo 的基础数据来源 |
| AI 学习 | 不参与（输入序列不包含中间状态） | 输出事件序列携带结构化状态，可用于训练 |

---

## 14. Risks (更新)

新增 EventStore 引入以下风险：

| 风险 | 影响 | 缓解 |
|------|------|------|
| 内存占用 | RingBuffer 存储 1000 条事件记录，每条约 200-500 字节 | 默认容量 1000（~500KB），可通过配置调整 |
| 写入延迟 | 每次事件发布多一步写入操作 | 写入为轻量结构体构建 + `Vec::push`；未来可异步批量写入 |
| 查询性能 | RingBuffer 全量遍历 O(n) | 默认 1000 条查询在微秒级；大数据量建议迁移 SQLite |
| 不完整历史 | 容量溢出后旧记录丢失 | 预期行为——定长 RingBuffer 语义，非持久化存储 |

---

## 15. Migration Strategy (更新)

| 版本 | 变更 | 迁移策略 |
|------|------|----------|
| v1 | 初始版本 | — |
| v2 | 新增 EventCategory | 新增 enum variant |
| v3 (未来) | 新增 EventStore (Event History) | EventService 增加 EventStore 写入步骤。可选功能——新项目默认启用，存量项目按需开启 |
| v4 (远期) | 新增 SQLite 持久化选项 | 可选的 Storage backend，运行时配置；RingBuffer 仍为默认实现 |

---

## 16. Constitution Check (更新)

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| Domain 间仅通过事件通信 | ✅ | EventBus 是域间唯一通信通道 |
| Event 与 Trigger 职责分离 | ✅ | Event 是通知，Trigger 是条件→技能映射 |
| Replay First | ✅ | 事件顺序和内容确定；EventStore 不改变 Replay 确定性 |
| 可观测性 | ✅ | EventStore 是 ObservableEvent 的附加 sink，利用现有日志基础设施 |
