---
id: foundation.event-history-architecture.v1
title: Event History Architecture Deep Dive — 事件历史架构详述
status: stable
owner: data-architect
created: 2026-06-21
updated: 2026-06-21
layer: runtime
replay-safe: true
---

# Event History Architecture — 事件历史架构详述

> **总纲引用**: `docs/04-data/README.md` §7 — Replay 架构（Event History 作为 Replay 互补系统）
> **ADR 引用**: `docs/01-architecture/40-cross-cutting/ADR-059-event-history.md`（架构决策）
> **领域 Schema**: `docs/04-data/capabilities/event_schema.md` §11-13（EventStore Schema 定义）
> **本文档是事件历史系统的深度展开**，覆盖 EventStore 数据结构、层所有权、持久化策略、Replay 集成和 Schema 演化。

---

## 1. Event History Store Schema

### 1.1 完整数据流

```
[Domain System] ──publish──→ [EventBus]
                                  │
                    ┌─────────────┼─────────────┐
                    │             │             │
                    ▼             ▼             ▼
            [Observer A]   [Observer B]   [EventStore]
                                              │
                                              ▼
                                      RingBuffer<StoredEvent>
                                              │
                                              ▼
                                    ┌─────────────────┐
                                    │   Query API      │
                                    │ by_domain        │
                                    │ by_log_code      │
                                    │ by_frame_range   │
                                    │ by_correlation   │
                                    │ recent / get     │
                                    └─────────────────┘
```

### 1.2 StoredEvent 结构

Event History 存储的最小单位是 `StoredEvent`。它是 `GameplayEvent` 的**持久化视图**，不是序列化副本——它丢弃了 EventBus 分发所需的元数据（`EventEnvelope`、`DispatchState`、`priority`），保留了对事后分析有用的结构化数据。

```rust
/// 存储在 EventStore 中的历史事件记录。
///
/// 每个 StoredEvent 从 GameplayEvent + ObservableEvent 转换而来。
/// 转换发生在 EventService.publish() 内部，作为事件分发流程的附加 sink。
struct StoredEvent {
    /// 单调递增的事件序号（全局唯一，从 0 开始）。
    event_id: u64,

    /// 事件创建时的帧号（帧计数器，与 ReplayFrame 帧号对齐）。
    timestamp: u64,

    /// 事件编码，关联 LogCode 枚举（如 BAT007、EFF001）。
    log_code: LogCode,

    /// 路由域——事件所属的业务域（如 Combat、Progression）。
    domain: Domain,

    /// 关联标识（可选），用于串联一次完整行为中的所有事件。
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
    /// 存储为 String→String 的键值对，确保跨版本兼容。
    fields: HashMap<String, String>,
}
```

**字段来源映射**：

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

### 1.3 EventStore 结构

```rust
/// 事件存储——环形缓冲区实现。
///
/// - 定长 RingBuffer，容量在初始化时指定（默认 1000）
/// - 追加写入，超过容量时覆盖最旧记录
/// - 只读查询，不支持修改或删除已存储的事件
///
/// 所有权归属 core/capabilities/runtime/event/，与 EventBus 同域。
/// EventService 在事件分发后自动写入，写入失败不影响分发主路径。
struct EventStore {
    /// 环形缓冲区，存储定量的 StoredEvent。
    buffer: RingBuffer<StoredEvent>,

    /// 最大容量（初始化时设定，运行时不可变）。
    capacity: usize,
}
```

### 1.4 序列化格式

EventStore 默认使用 **MessagePack** 作为内存序列化格式，原因如下：

| 特性 | MessagePack | JSON | bincode |
|------|------------|------|---------|
| 紧凑度 | 高（~200-500 bytes/条） | 低（~500-1500 bytes/条） | 最高 |
| Schema 灵活 | 高（可选字段支持） | 高 | 低（严格顺序） |
| 跨语言 | 支持 | 支持 | 有限 |
| 人类可读 | 否 | 是 | 否 |
| 运行时性能 | 快 | 中 | 最快 |

MessagePack 在紧凑度和字段灵活性之间取得平衡，适合运行时 RingBuffer 存储和未来 SQLite BLOB 持久化。

### 1.5 RingBuffer 内存占用估算

| 配置 | 每条大小 | 容量 | 总内存 |
|------|---------|------|--------|
| 默认 | ~300 bytes | 1,000 | ~300 KB |
| 大容量 | ~300 bytes | 10,000 | ~3 MB |
| 调试模式 | ~500 bytes | 100,000 | ~50 MB |

单条 StoredEvent 的大小取决于 `fields: HashMap` 中动态字段的数量。典型战斗事件（DamageDealt）约 5-10 个字段，平均 300 bytes。含大量上下文的事件（如技能结算）可能达到 800 bytes。

---

## 2. 层所有权

### 2.1 事件分层体系

事件历史系统中的事件按来源分为三个层级，每个层级有不同的事件类型特征和生命周期规则：

```
Layer 1: Capability Events（能力层事件）
  ├── Effect 生命周期事件（effect.applied, effect.expired）
  ├── Modifier 变更事件（modifier.added, modifier.removed）
  ├── Stacking 变更事件（stack.changed, stack.max_reached）
  └── Trigger 触发事件（trigger.activated, trigger.cooldown_ready）

Layer 2: Domain Events（业务层事件）
  ├── Combat 事件（combat.damage_dealt, combat.unit_died, combat.turn_ended）
  ├── Spell 事件（spell.cast, spell.resolved, spell.concentration_broken）
  ├── Progression 事件（progression.level_up, progression.talent_changed）
  ├── Quest 事件（quest.accepted, quest.completed, quest.objective_updated）
  ├── Inventory 事件（item.used, item.equipped, item.looted）
  └── 其他 Domain 事件...

Layer 3: Infra Events（基础设施层事件）
  ├── Replay 事件（replay.start, replay.end, replay.desync）
  ├── Save 事件（save.created, save.loaded, save.migrated）
  └── Pipeline 事件（pipeline.step_started, pipeline.step_completed）
```

### 2.2 各层事件的所有权规则

| 层级 | 定义位置 | 路由域 | Replay 参与 | EventStore 记录 | 备注 |
|------|---------|--------|-------------|----------------|------|
| Capability Events | `core/capabilities/*/event/` | 对应能力域 | ✅ 间接 | ✅ 记录 | 通过 EventBus 分发，被 Domain 消费 |
| Domain Events | `core/domains/*/events/` | 对应业务域 | ✅ 核心 | ✅ 记录 | 跨域共享事件定义在 `core/events.rs` |
| Infra Events | `infra/*/events/` | `Infrastructure` | ❌ 不参与 | 🟡 可选 | 仅用于运维和调试，非业务路径 |

**核心规则**：
- Capability 和 Domain 事件**强制**记录到 EventStore（由 EventService 自动完成）
- Infra 事件**可选择**记录（通过 `EventStoreConfig` 控制 `record_infra_events: bool`），默认关闭
- 禁止 Capability 事件直接引用 Domain 特有的数据类型（保持通用性）
- 禁止 Infra 事件通过 EventBus 影响业务逻辑（Infra 事件仅供观察和日志）

### 2.3 层间事件流转

```
[Combat Domain] ──publish──→  EventBus
     │                          │
     │                          ├──→ EventStore.append (自动)
     │                          │       │
     │                          │       ├── StoredEvent { domain: Domain::Combat, ... }
     │                          │       └── fields: { "damage": "25", "defense": "10" }
     │                          │
     │                          ├──→ [Terrain Domain]  (Observer: surface decay)
     │                          │
     │                          └──→ [Effect Capability] (Observer: stack consumption)
```

Shared Cross-Domain Events（`core/events.rs`）的特殊地位：它们既是 Domain Event（由 Combat 发射），也是跨域通知（由多个 Domain 消费）。EventStore 记录时以原始发射 Domain 为准。

### 2.4 事件类型分类对照

| 分类 | 示例 EventTag | 层 | EventStore 记录 |
|------|-------------|-----|-----------------|
| 战斗核心 | `event.combat.damage_dealt` | Domain | ✅ |
| 回合管理 | `event.turn.started`, `event.turn.ended` | Domain/Shared | ✅ |
| 法术 | `event.spell.cast`, `event.spell.resolved` | Domain | ✅ |
| 效果 | `event.effect.applied`, `event.effect.expired` | Capability | ✅ |
| 堆叠 | `event.stack.changed` | Capability | ✅ |
| 物品 | `event.item.used`, `event.item.equipped` | Domain | ✅ |
| 任务 | `event.quest.accepted`, `event.quest.completed` | Domain | ✅ |
| 战斗开始/结束 | `event.battle.started`, `event.battle.ended` | Domain/Shared | ✅ |
| Replay 状态 | `event.replay.desync` | Infra | 🟡 选录 |
| 存档操作 | `event.save.created` | Infra | 🟡 选录 |

---

## 3. 持久化策略

### 3.1 两阶段存储模型

EventStore 采用**两阶段存储策略**：

| 阶段 | 存储后端 | 容量 | 持久化 | 生命周期 | 状态 |
|------|---------|------|--------|---------|------|
| Stage 1: In-Memory | RingBuffer | 1,000 - 10,000 条 | 不持久化 | 会话级 | ✅ 已实现 |
| Stage 2: Optional SQLite | SQLite (BLOB) | 无上限 | 跨会话 | 按需配置 | 🔮 未来 |

### 3.2 Stage 1: In-Memory RingBuffer（当前实现）

默认实现，无外部依赖：

```rust
impl EventStore {
    /// 追加一条事件记录。
    /// 超过 capacity 时自动覆盖最旧记录。
    fn append(&mut self, event: StoredEvent) {
        self.buffer.push(event);
    }

    /// 清空所有记录（在新游戏开始或加载存档时调用）。
    fn clear(&mut self) {
        self.buffer.clear();
    }
}
```

**裁剪策略**：RingBuffer 语义——容量达到上限时，最旧的记录被新记录覆盖。无需额外的裁剪任务。

**触发清空的场景**：

| 场景 | 清空时机 | 原因 |
|------|---------|------|
| 新游戏开始 | 调用 `EventStore.clear()` | 新会话不应保留旧会话的事件 |
| 加载存档 | 调用 `EventStore.clear()` | 存档加载后事件流应重新开始记录 |
| Replay 回放 | **不清空** — 覆盖写入 | 回放过程中产生的事件正常记录 |
| 场景切换 | **不清空** | 连续游戏内的场景切换不应丢失已有事件历史 |

### 3.3 Stage 2: SQLite 持久化（未来规划）

通过 `EventStorageBackend` trait 实现可切换的存储后端：

```rust
/// 事件存储后端抽象接口。
/// RingBuffer 和 SQLite 各实现一个 variant。
trait EventStorageBackend {
    /// 追加事件记录。
    fn append(&mut self, event: StoredEvent) -> Result<(), StorageError>;

    /// 按条件查询。
    fn query(&self, filter: &EventFilter) -> Result<Vec<StoredEvent>, StorageError>;

    /// 清空存储。
    fn clear(&mut self) -> Result<(), StorageError>;

    /// 当前存储量。
    fn len(&self) -> usize;
}
```

**SQLite 表结构**（未来实现）：

```sql
CREATE TABLE event_history (
    event_id    INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp   INTEGER NOT NULL,         -- 帧号
    log_code    TEXT NOT NULL,             -- LogCode 字符串表示
    domain      TEXT NOT NULL,             -- Domain 字符串表示
    correlation_id TEXT,                   -- 关联标识（如 "battle_123/turn_5/action_2"）
    event_tag   TEXT NOT NULL,             -- 事件标签路径
    source_entity TEXT,                    -- 来源实体 ID
    target_entity TEXT,                    -- 目标实体 ID
    magnitude   REAL,                      -- 数值载荷
    fields      BLOB,                      -- MessagePack 编码的动态字段
    recorded_at INTEGER DEFAULT (strftime('%s', 'now'))  -- 记录时间戳（用于运维）
);

-- 查询加速索引
CREATE INDEX idx_event_history_domain ON event_history(domain);
CREATE INDEX idx_event_history_log_code ON event_history(log_code);
CREATE INDEX idx_event_history_correlation ON event_history(correlation_id);
CREATE INDEX idx_event_history_timestamp ON event_history(timestamp);
```

**迁移策略**：从 RingBuffer 到 SQLite 的迁移是**配置变更**，不是数据迁移。两个后端的数据不互通——切换到 SQLite 后 RingBuffer 中的旧记录不迁移到 SQLite。

### 3.4 事件历史与 Save 系统的关系

EventStore **不参与存档持久化**。这是 `event_schema.md` §8 的延续决策：

| 场景 | EventStore 行为 | 原因 |
|------|----------------|------|
| 保存存档 | 不清空、不写入存档文件 | 事件是纯运行时机制，不应进入持久化状态 |
| 加载存档 | 清空 RingBuffer | 新会话从零开始记录 |
| 新游戏开始 | 清空 RingBuffer | 同上 |
| 游戏关闭 | 数据自然丢失 | 非持久化存储 |

如果有跨会话保留事件历史的需求（如长期 AI 训练数据），通过 Stage 2 SQLite 后端解决——SQLite 文件由开发者/QA 按需保留，不是存档系统的组成部分。

---

## 4. Replay 兼容性

### 4.1 Event History 与 Replay 的关系

Event History 和 Replay 是两个独立但互补的系统。下表重申并深化 `event_schema.md` §13 的边界定义：

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

### 4.2 互补数据流

```
┌─────────────────────────────────────────────────────────────┐
│                    游戏运行过程                              │
│                                                             │
│  [用户输入] ──→ [Command] ──→ [游戏逻辑] ──→ [Domain Events] │
│       ↑                          ↓                         │
│   Replay 录制                 EventStore                   │
│   (输入命令)                   (输出事件)                    │
│                                                             │
│  QA 工作流:                                                  │
│  1. Replay 还原输入 → 复现 Bug                              │
│  2. EventStore 查询输出 → 定位异常事件                       │
│  3. 两者结合 → 完整因果链分析                                │
└─────────────────────────────────────────────────────────────┘
```

### 4.3 EventStore 在 Replay 模式下的行为

在 Replay 回放模式下，EventStore **正常写入**，不改变现有行为：

| Replay 阶段 | EventStore 行为 | 说明 |
|------------|----------------|------|
| Replay 开始 | 保留已有记录，不清空 | Replay 前后的事件流可对比 |
| Replay 执行 | 正常写入每个事件 | 回放产生的全部事件记录到 RingBuffer |
| Replay 结束 | 保留记录 | QA 可在回放后查询事件历史 |
| Replay desync | 正常写入，同时记录 `event.replay.desync` | desync 前后的事件序列是重要的调试数据 |

EventStore 的写入**不改变 Replay 确定性**，原因如下：

1. **写入是纯附加操作**：`RingBuffer.push()` 不改变系统状态，不影响游戏逻辑
2. **写入在分发路径的只读侧**：EventService 在分发事件给订阅者的同时写入 EventStore，订阅者不受写入操作影响
3. **EventStore 不是事件源**：EventStore 的查询接口是只读的，没有任何系统依赖 EventStore 的内容做决策
4. **Replay SyncCheckpoint 不包含 EventStore**：SyncCheckpoint 的哈希计算不包含 `EventStore` 中存储的任何数据

### 4.4 确定性保证复核

| 要求 | 满足情况 | 说明 |
|------|---------|------|
| RNG 种子确定 | ✅ 不依赖 | EventStore 不涉及随机数 |
| 帧计数器对齐 | ✅ 保证 | `StoredEvent.timestamp` 使用帧计数器，与 ReplayFrame 对齐 |
| 非外部状态依赖 | ✅ 保证 | EventStore 不读取文件系统、网络或系统时钟 |
| 写入不影响主路径 | ✅ 保证 | 写入失败不传播，不中断事件分发 |
| 查询不影响状态 | ✅ 保证 | 所有查询接口返回不可变引用或克隆 |

### 4.5 共享事件作为时间轴标记

ADR-049 §D2-6 定义了 Event History 利用共享事件作为时间轴标记：

| 共享事件 | Event History 用途 | 记录时机 |
|---------|-------------------|---------|
| `BattleStarted` | 战斗开始标记，关联参战单位快照 | Replay 帧序列第一帧 |
| `TurnEnded` | 回合边界标记，用于分段式重放 | 每回合结束时 |
| `TurnStarted` | 回合开始标记，用于 UI 轮播同步 | 每回合开始时 |
| `BattleEnded` | 战斗结果标记，关联胜负判定 | 战斗结算时 |

**数据流**：

```
Combat Pipeline → commands.trigger(TurnEnded)
                     │
                     ├──→ Terrain Domain (surface decay)
                     │
                     └──→ Event History Recorder (事件流记录)
                              │
                              ▼
                          EventStore
```

共享事件的参数字段包含 Event History 索引所需的最小上下文（entity ID + frame number）。

---

## 5. Schema 演化

### 5.1 版本历史

| 版本 | 变更 | 日期 | 向后兼容 |
|------|------|------|---------|
| v1 | 初始版本（RingBuffer + StoredEvent） | 2026-06-21 | — |
| v2 (未来) | 新增 `EventStorageBackend` trait | TBD | ✅ 兼容（新增 trait，现有 RingBuffer 实现不变） |
| v3 (未来) | SQLite 持久化选项 | TBD | ✅ 兼容（配置切换，不影响 RingBuffer） |

### 5.2 StoredEvent Schema 兼容性规则

`StoredEvent` 的 `fields: HashMap<String, String>` 是主要的向前兼容机制：

- **新增字段**：通过新增 `fields` 键值对实现，不修改结构体字段
- **废弃字段**：保留原有键名，标记为 `deprecated: "true"`，不在新版本中移除
- **字段类型变更**：通过新增键名实现（如 `damage_i32` → `damage_f64`），旧键名保留

**结构化字段的兼容策略**：

```rust
// v1: 原始字段
fields: {
    "damage": "25",
    "defense": "10",
    "crit": "true",
}

// v2: 新增字段
fields: {
    "damage": "25",
    "damage_details": "{\"base\":20,\"bonus\":5,\"type\":\"fire\"}",  // 新增，JSON 子结构
    "defense": "10",
    "crit": "true",
    "mitigation": "0.5",  // 新增
}

// v3: 废弃字段（不删除，保留以支持旧查询）
fields: {
    "damage": "25",            // 仍写入（保持兼容）
    "damage_f64": "25.7",      // 新增更精确的版本
    "defense": "10",
    "crit": "true",
    "mitigation": "0.5",
    "legacy_value": "deprecated: true",  // 标记废弃
}
```

### 5.3 EventTag 演化

`EventTag` 使用层级路径格式（`event.<category>.<action>`），支持模糊匹配和新增：

```rust
// v1: 初始事件标签
event.combat.damage_dealt
event.combat.heal_dealt
event.turn.started
event.turn.ended

// v2: 新增事件标签（不影响已有订阅者）
event.combat.damage_dealt
event.combat.damage_mitigated     // 新增
event.combat.heal_dealt
event.combat.heal_over_time       // 新增
event.turn.started
event.turn.ended
event.turn.skipped                // 新增
```

**关键词**：
- 新增事件标签**不影响**现有订阅者（订阅 `event.combat.*` 的 Observer 自动收到新标签的事件）
- 删除事件标签**影响**现有订阅者（按标签过滤的查询不再返回结果）
- 重命名事件标签**破坏**兼容性（应为旧标签添加 `deprecated` 标记，新增标签替代，过渡期后移除）

### 5.4 LogCode 枚举演化

`LogCode` 枚举（定义在 `src/shared/diagnostics/log_code.rs`）的变更是编入索引的：

| 操作 | 兼容性 | 规则 |
|------|--------|------|
| 新增 variant | ✅ 前向兼容 | 新代码新增，旧代码反序列化时使用 `Unknown(u32)` 兜底 |
| 废弃 variant | ✅ 保持兼容 | variant 保留，标记 `#[deprecated]`，不在新代码中使用 |
| 删除 variant | ❌ 破坏兼容 | 禁止删除——废弃 variant 保留到下个大版本 |
| 重命名 variant | ❌ 破坏兼容 | 禁止重命名——使用别名或新增替代 |

### 5.5 Domain 枚举演化

`Domain` 枚举（定义在 `src/shared/diagnostics/domain.rs`）的演化规则与 LogCode 相同：

```rust
enum Domain {
    // ── v1 稳定 variant ──
    Combat,
    Tactical,
    Terrain,
    Faction,
    Spell,
    Reaction,
    Progression,
    Inventory,
    Party,
    CampRest,
    Narrative,
    Quest,
    Economy,
    Crafting,
    Summon,
    Infrastructure,

    // ── v2 新增 variant ──
    // Social,              // 未来可能的社交系统

    // ── 兜底 ──
    #[serde(other)]
    Unknown(String),
}
```

新增 `Domain` variant 时，`StoredEvent` 的 `domain` 字段自动支持新值，无需修改 EventStore 的存储结构。

---

## 6. EventStoreConfig

EventStore 的行为通过全局 Resource 配置：

```rust
/// EventStore 运行时配置。
struct EventStoreConfig {
    /// RingBuffer 容量（默认 1000）。
    pub capacity: usize,

    /// 是否启用 EventStore（默认 true）。
    /// 禁用时 EventService 跳过 EventStore 写入。
    pub enabled: bool,

    /// 是否记录 Infra 层事件（默认 false）。
    /// Infra 事件包括 replay.*、save.* 等运维事件。
    pub record_infra_events: bool,

    /// 事件采样率（0.0 - 1.0，默认 1.0）。
    /// 1.0 = 记录所有事件，0.5 = 随机记录 50%。
    /// 高负载场景可通过降低采样率控制内存使用。
    pub sample_rate: f32,
}
```

**配置变更兼容性**：

| 字段 | 变更类型 | 兼容性 |
|------|---------|--------|
| `capacity` | 仅初始化时读取 | 运行时修改不生效 |
| `enabled` | 运行时切换 | 启用时重新开始记录，历史不恢复 |
| `record_infra_events` | 运行时切换 | 立即生效 |
| `sample_rate` | 运行时切换 | 立即生效，未来 feature |

---

## 7. 完整性校验

### 7.1 运行时校验

| 校验 | 触发时机 | 行为 |
|------|---------|------|
| `event_id` 单调递增 | 每次写入 | 断言：`new_id == last_id + 1`（调试模式 panic） |
| `timestamp` 非递减 | 每次写入 | 断言：`new_ts >= last_ts`（调试模式 panic） |
| RingBuffer 一致性 | 定期快照 | 检查 buffer 的连续性 |

### 7.2 查询输入校验

```rust
impl EventStore {
    /// 按帧号范围查询。start 必须 ≤ end。
    fn by_frame_range(&self, start: u64, end: u64) -> Result<Vec<StoredEvent>, QueryError> {
        if start > end {
            return Err(QueryError::InvalidRange { start, end });
        }
        Ok(self.buffer.iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end)
            .collect())
    }
}
```

---

## 8. 所有权与模块边界

### 8.1 文件分布

```
src/core/capabilities/runtime/event/
├── foundation/
│   ├── mod.rs                # 现有
│   ├── gameplay_event.rs     # 现有 — GameplayEvent, EventTag
│   ├── event_tag.rs          # 现有 — EventTag 定义
│   └── event_store.rs        # NEW — EventStore, StoredEvent, EventStoreConfig
├── mechanism/
│   ├── mod.rs                # 现有
│   ├── event_bus.rs          # 现有 — EventBus 实现
│   ├── event_service.rs      # 修改 — 注入 EventStore 写入
│   └── ...
└── tests/
    └── ...
```

### 8.2 所有权边界

| 组件 | 所属层 | 所有者 | 说明 |
|------|-------|--------|------|
| `EventStore` | foundation | data-architect | 纯数据结构定义 |
| `StoredEvent` | foundation | data-architect | 纯数据结构定义 |
| `EventStoreConfig` | foundation | data-architect | 配置定义 |
| `EventStorageBackend` | mechanism (future) | data-architect + feature-developer | 存储后端抽象 |
| EventStore 写入逻辑 | mechanism | feature-developer | EventService 集成 |
| Query API 实现 | mechanism | feature-developer | 遍历过滤逻辑 |
| EventStore 测试 | tests | test-guardian | 回放确定性测试 |

---

## 9. 与现有系统的集成点

| 系统 | 集成方式 | 变更范围 |
|------|---------|---------|
| EventBus | EventService.publish() 内部调用 EventStore.append() | 修改 EventService（追加一步） |
| ObservableEvent | 通过 `record_fields()` 收集结构化字段 | 无变更（现有 trait） |
| LogCode | StoredEvent.log_code 字段 | 无变更（现有枚举） |
| Domain | StoredEvent.domain 字段 | 无变更（现有枚举） |
| CorrelationId | StoredEvent.correlation_id 字段 | 无变更（现有类型） |
| Replay | EventStore 在 Replay 模式下正常写入 | 无变更（EventStore 透明） |
| Save | EventStore 不参与存档 | 无变更 |
| UI Debug Panel | 通过 Query API 在 UI 中展示事件历史 | 新增 UI Widget（presentation 层） |

---

## 10. Future Extension

| 扩展 | 阶段 | 说明 |
|------|------|------|
| **SQLite 持久化** | Stage 2 | 可选的持久化 backend，通过 `EventStorageBackend` trait 切换 |
| **事件导出** | Stage 2 | 将 EventStore 内容导出为 JSON/CSV，用于离线分析和 AI 训练 |
| **事件回滚（Undo）** | Stage 3 | 基于事件历史的操作撤销——EventStore 提供"发生了什么"，Undo 系统决定"如何回滚" |
| **实时事件流推送** | Stage 3 | 通过 WebSocket 将事件流推送到外部监控工具 |
| **事件采样策略** | Stage 2 | 支持智能采样（重要事件全录，低价值事件按比例采样） |
| **事件索引** | Stage 2 (SQLite) | 按 domain、log_code、correlation_id 建索引，加速查询 |
| **事件聚合统计** | Stage 3 | 在 EventStore 上构建聚合层——按类别统计事件频率、总量，供平衡性分析 |

---

## 11. Risks

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|---------|
| RingBuffer 覆盖重要调试数据 | QA 无法回溯关键事件 | 中 | 默认容量 1000；提供容量配置；Stage 2 SQLite 提供无限存储 |
| EventStore 写入影响帧率 | 性能下降 | 低 | 写入为 `Vec::push` 量级（微秒）；写入失败不阻塞；采样率可配置 |
| `fields: HashMap` 内存碎片 | 堆内存膨胀 | 中 | 每条 StoredEvent 约 300 bytes；1000 条共 ~300 KB；无可观影响 |
| 事件定义者忘记实现 ObservableEvent | 字段缺失 | 低 | 编译期 trait 检查；`record_fields()` 默认实现至少提供基本字段 |
| LogCode/Domain 枚举不匹配 | 查询结果不准确 | 低 | 枚举定义集中管理；使用 `#[serde(other)]` 兜底；测试覆盖匹配检查 |

---

## 12. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| Def-Instance 强制分离 (DL001) | ✅ | EventStore/StoredEvent 是纯 Runtime 结构，不混用 Definition/Spec/Persistence 职责 |
| Rule-Content 强制分离 (DL002) | ✅ | EventStore 只记录事件，不含业务逻辑 |
| 配置只引用 ID (DL003) | ✅ | EventTag 使用标签体系，不嵌入 Definition 内容 |
| Effect 是唯一业务入口 (DL005) | ✅ | EventStore 记录 Effect 产生的事件，不绕过 Effect 修改状态 |
| Replay 优先 (DL010) | ✅ | EventStore 是只读附加 sink，不改变 Replay 确定性 |
| Schema 版本化 (DL011) | ✅ | StoredEvent 通过 `fields: HashMap` 提供前向兼容；LogCode/Domain 枚举使用 `#[serde(other)]` 兜底 |
| 域间禁止直接数据引用 (DL012) | ✅ | EventStore 通过 Domain 枚举标识事件来源，不引用特定 Domain 的数据结构 |
| 用户可见文本使用 LocalizationKey (DL013) | ✅ | 事件历史查询结果中的文本字段使用 LocalizationKey 引用 |
| 表现必须经过 Cue (DL009) | ✅ | EventStore 不直接触发 UI——UI 通过 Query API 读取事件历史后，经过 Cue 更新显示 |
