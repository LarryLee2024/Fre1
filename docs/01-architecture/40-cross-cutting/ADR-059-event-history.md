---
id: 01-architecture.ADR-059
title: ADR-059 — Event History Architecture
status: draft
owner: content-architect
created: 2026-06-21
updated: 2026-06-21
supersedes: none
---

# ADR-059: Event History Architecture

## 状态

**Draft** — 依赖 ADR-041（Replay Determinism）、ADR-052（Logging Architecture）、ObservableEvent trait 和 EventBus Schema（`docs/04-data/capabilities/event_schema.md`）。

## 背景

当前 Domain 事件通过 EventBus 分发后即被丢弃——事件是 fire-and-forget 的纯运行时通信机制（见 `event_schema.md` §8 Save Compatibility：事件不参与存档持久化）。这导致以下能力缺口：

- **Undo/回滚**：没有事件历史记录，无法知道"发生了什么"，更无法撤销
- **QA 分析**：Bug 复现后，QA 需要手动插入日志才能定位异常事件——已有的 LogCode 日志是文本结构，缺少跨事件的结构化关联查询
- **AI 行为分析**：AI 决策的后果事件不可追溯，无法对 AI 行为进行事后定量评估
- **调试体验**：开发者在调试时需要查询"上一帧发生了什么事件"，当前只能靠 tracing 日志文本搜索

### 现有基础设施

项目已有与 Event History 相关的能力：

- **Replay 系统**（ADR-041）记录输入命令（`RecordedCommand`、`ReplayFrame`）——回答了"输入是什么"
- **ObservableEvent trait**（`src/shared/diagnostics/observable.rs`）提供了 `DOMAIN`、`CODE`、`record_fields()`  契约——每个领域事件已具备结构化的可观测性
- **LogCode 枚举**（`src/shared/diagnostics/log_code.rs`）编码了所有事件类型（100+ 事件编码）
- **Domain 枚举**（`src/shared/diagnostics/domain.rs`）定义了 24 个路由域
- **CorrelationId**（`src/shared/diagnostics/correlation.rs`）提供了 BattleId → TurnId → ActionId 的关联标识层级

EventBus 自身也定义了事件的分发状态机（Created → Queued → Dispatching → Consumed），但缺少"事件持久化"这一步骤。

## 决策

### 1. Event History 与 Replay 分离

**Event History 不是 Replay。** 两者是独立但互补的系统，服务于不同的目的。

| 维度 | Replay | Event History |
|------|--------|---------------|
| 录什么 | 输入命令（玩家的操作、AI 决策、RNG 种子） | 输出事件（Domain Event 的结构化快照） |
| 为什么录 | 确定性验证、Bug 复现 | 事后分析、调试、QA、AI 分析 |
| 完整性要求 | 每帧全部，缺一不可 | 环形缓冲区，容量溢出丢弃旧记录 |
| 因果关系 | 原因 | 结果 |

两者结合可构建完整因果链：Replay 还原输入 → Command 驱动逻辑 → Event 产生输出 → EventStore 记录输出。

### 2. EventStore 是 Observer 的附加 Sink

EventStore **不是** EventBus 的新实现，也不是对现有事件分发机制的修改。它是 EventService 在事件分发过程中的一个附加 sink。

```
EventBus.publish(event):
  1. 校验 event_tag + source_entity
  2. EventStore.append(StoredEvent.from(event))   ← 新增，非阻塞写入
  3. 按 EventPriority 匹配订阅者
  4. 逐个投递到订阅者
```

写入 EventStore 位于分发路径上，但必须满足：
- **非阻塞**：写入是轻量结构体构建 + RingBuffer push
- **不影响分发**：写入失败（理论上不会发生）不中断事件分发
- **异步友好**：未来可切换为异步批量写入

### 3. 存储模型：In-Memory RingBuffer

EventStore 的核心存储是一个定长 RingBuffer：

```rust
struct EventStore {
    buffer: RingBuffer<StoredEvent>,
    capacity: usize,       // 默认 1000，可通过配置调整
    next_id: u64,          // 单调递增的事件序号
}
```

| 属性 | 值 |
|------|-----|
| 默认容量 | 1000 条 |
| 单条记录大小 | ~200-500 字节（受 `fields: HashMap` 影响） |
| 总内存占用 | ~500KB（默认 1000 条） |
| 溢出策略 | 覆盖最旧记录（RingBuffer 语义） |
| 线程安全 | 单线程 ECS，无需同步原语 |

容量可通过 `EventStoreConfig` 配置：

```rust
struct EventStoreConfig {
    pub capacity: usize,           // 默认 1000
    pub enabled: bool,             // 默认 true — 运行时开关
}
```

### 4. StoredEvent Schema

```rust
/// 存储在 EventStore 中的历史事件记录。
///
/// 对应 `event_schema.md` §11.1 的完整定义。
/// 每个 StoredEvent 从 GameplayEvent + ObservableEvent 转换而来。
struct StoredEvent {
    /// 单调递增的事件序号（全局唯一）。
    event_id: u64,

    /// 事件创建时的帧号（帧计数器 Replay 帧号对齐）。
    timestamp: u64,

    /// 事件编码，关联 LogCode 枚举（如 BAT007、EFF001）。
    log_code: LogCode,

    /// 路由域——事件所属的业务域。
    domain: Domain,

    /// 关联标识（可选），用于串联一次完整战斗行为中的所有日志。
    /// 层级关系: BattleId → TurnId → ActionId
    correlation_id: Option<CorrelationId>,

    /// 事件类型标签（与 EventTag 对应，如 "event.combat.damage_dealt"）。
    event_tag: EventTag,

    /// 来源实体 ID（可选）。
    source_entity: Option<Entity>,

    /// 目标实体 ID（可选）。
    target_entity: Option<Entity>,

    /// 事件数值载荷（如伤害量 25、治疗量 10）。
    magnitude: Option<f32>,

    /// 结构化字段——从 ObservableEvent::record_fields() 收集的动态字段。
    fields: HashMap<String, String>,
}
```

**StoredEvent 是 GameplayEvent 的持久化视图**，不是 GameplayEvent 的序列化——它丢弃了 EventBus 分发所需的元数据（`EventEnvelope`、`DispatchState`、`priority`），保留了对事后分析有用的结构化数据。

### 5. 字段来源映射

| StoredEvent 字段 | 来源 |
|-----------------|------|
| `event_id` | EventStore 自增计数器 |
| `timestamp` | 帧计数器 `FrameCounter.0` |
| `log_code` | `Event::log_code()`（ObservableEvent trait） |
| `domain` | `Event::DOMAIN`（ObservableEvent trait） |
| `correlation_id` | GameplayEvent 创建时传入（由发布者设置，可选） |
| `event_tag` | GameplayEvent.event_tag |
| `source_entity` | GameplayEvent.source_entity |
| `target_entity` | GameplayEvent.target_entity |
| `magnitude` | GameplayEvent.magnitude |
| `fields` | `ObservableEvent::record_fields(collector)` 收集的动态字段 |

### 6. Query Interface

EventStore 提供基于索引的只读查询接口。所有查询为 RingBuffer 遍历过滤（O(n)）：

```rust
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

    /// 当前存储的事件总数。
    fn len(&self) -> usize;

    /// 是否为空。
    fn is_empty(&self) -> bool;
}
```

### 7. 所有权归属

```
src/core/capabilities/runtime/event/
├── foundation/
│   ├── event_store.rs     ← EventStore, StoredEvent 定义
│   └── ...                ← 现有 EventBus 类型
├── mechanism/
│   ├── event_service.rs   ← 注入 EventStore 的 EventService 实现
│   └── ...                ← 现有 EventBus 实现
└── ...
```

- **EventStore** 位于 `core/capabilities/runtime/event/`，与 EventBus 同域
- **StoredEvent** 是 foundation 类型（纯数据定义，零依赖）
- **EventService** 在 mechanism 层注入 EventStore 写入逻辑

EventStore 不位于 `infra/replay/` 或单独的 `infra/event_history/`，因为它不是技术实现细节——Event Store 是 Event 能力领域的自然延伸。

### 8. Future: SQLite 持久化

EventStore 的 RingBuffer 实现满足阶段 1 需求。阶段 2（远期）可引入可选的 SQLite 持久化 backend：

- 提供 `EventStorageBackend` trait，RingBuffer 和 SQLite 各实现一个 variant
- 运行时通过配置切换 backend
- SQLite 持久化支持跨会话历史保留，用于离线分析和 AI 训练
- SQLite 支持索引加速查询（按 correlation_id、domain 建索引）

阶段 2 暂不实现，仅保持 `EventStore` 结构对 backend 抽象的开放性。

### 9. 生命周期与 Save 系统

EventStore 是纯运行时设施，不参与 Save 持久化：

| 场景 | EventStore 行为 |
|------|----------------|
| 新游戏开始 | 清空 RingBuffer |
| 加载存档 | 清空 RingBuffer（历史属于上一场游戏） |
| Replay 回放 | 正常写入——EventStore 记录回放过程中产生的事件 |
| 保存存档 | 不写 —— EventStore 不进入存档文件 |

这延续了 `event_schema.md` §8 的决策：事件是纯运行时通信机制。如有跨会话历史需求，通过 `EventStorageBackend` 的 SQLite 实现解决。

---

## 模块设计

```
src/core/capabilities/runtime/event/
├── foundation/
│   ├── mod.rs              ← 现有
│   ├── gameplay_event.rs   ← 现有
│   ├── event_tag.rs        ← 现有
│   ├── event_store.rs      ← NEW: EventStore, StoredEvent, EventStoreConfig
│   └── ...
├── mechanism/
│   ├── mod.rs              ← 现有
│   ├── event_bus.rs        ← 现有
│   ├── event_service.rs    ← 修改: EventService 注入 EventStore 引用
│   └── ...
└── ...
```

---

## 通信设计

| 通信 | 机制 | 方向 |
|------|------|------|
| EventStore 写入 | `EventStore.append()` （EventService 内部调用） | EventService → EventStore |
| 事件历史查询 | `EventStore.by_*()` 方法 | 调试工具/QA → EventStore |
| 容量配置 | `EventStoreConfig` Resource | App 启动时 → EventStore |
| EventStore 清除 | `EventStore.clear()` | 新游戏/加载存档 → EventStore |

---

## 边界定义

### 允许
- EventStore 作为 EventBus 分发的附加 sink 写入
- 通过 Query Interface 查询历史事件
- 配置 RingBuffer 容量（运行时调整）
- Replay 模式下正常写入（不改变 Replay 确定性）

### 禁止
- EventStore 写入失败影响事件分发主路径
- 修改已存储的事件记录（只追加不可变）
- EventStore 暴露可变引用给外部系统
- 依赖 EventStore 实现 Undo 的核心逻辑（EventStore 是分析工具，不是事务日志）

---

## 后果

### 正面
- 填补"事后分析"的能力空白——QA 和开发者可以直接查询事件历史
- 利用现有的 ObservableEvent 基础设施，零额外事件定义工作量
- 与 Replay 系统互补而非冲突，形成完整因果链
- 轻量内存存储，对性能影响极小（~500KB 默认）
- 查询接口为后续 Undo 系统和 AI 分析提供数据结构基础

### 负面
- 增加约 500KB-2MB 的运行时内存占用（取决于容量配置）
- 每发布一个事件增加一次 `HashMap` 构建和 `RingBuffer.push` 操作的延迟（微秒级）
- RingBuffer 查询为 O(n) 全量遍历，大数据量场景不适合
- 容量溢出后旧记录不可恢复（RingBuffer 语义）

### 中立
- EventStore 的引入不改变现有 Observer 模式——EventStore 是新增的 Observer sink
- 不是 Save 系统的组成部分——事件历史不跨会话保留

---

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 将事件历史附加到 Replay 文件 | Replay 记录输入命令，事件是输出。将输出附加到输入文件违反单一职责——Replay 文件应该只记录确定输入。另外，事件历史不是确定性的（依赖游戏逻辑版本），不应绑定到 Replay |
| 仅依赖 tracing 日志文本搜索 | 文本日志缺少结构化字段和查询能力——无法按 `correlation_id` 或 `domain` 精确过滤。且日志级别过滤可能跳过事件 |
| 在 Save 文件中持久化事件历史 | 事件是纯运行时机制（`event_schema.md` §8 决策）。Save 不应包含中间事件数据 |
| 用数据库替代 RingBuffer（阶段 1） | 1000 条运行时事件无需数据库。O(n) 查询在此量级下完全够用（微秒级）。SQLite 为阶段 2 可选 |

---

## 评审要点

- [ ] EventStore 写入是否在 EventBus 主路径上？是否满足"非阻塞、不影响分发"的要求？
- [ ] StoredEvent 字段是否覆盖了 QA/调试/AI 分析的核心需求？是否需要更多字段（如事件处理耗时）？
- [ ] RingBuffer 默认容量 1000 是否合适？游戏一局战斗约产生多少事件？
- [ ] Query Interface 是否满足典型场景？是否需要更多查询维度（按 entity、按 magnitude 范围）？
- [ ] EventStore 的所有权归属是否正确？是否应为 `infra/` 层而非 `core/capabilities/runtime/event/`？
- [ ] 与 ObservableEvent 的集成是否会产生性能瓶颈？（`record_fields()` 的 `HashMap` 分配）
- [ ] 是否需要为 EventStore 添加独立的事件（如 `EventStored`、`EventStoreOverflow`）用于监控？
- [ ] Stage 2 SQLite 的 `EventStorageBackend` trait 是否应现在预留接口，还是延迟到需要时再引入？
