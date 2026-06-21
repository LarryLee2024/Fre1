---
name: event-history-architecture
description: Event History 架构文档和 EventStore Schema — RingBuffer+StoredEvent，不持久化，Stage 2 SQLite 可选
metadata:
  type: reference
---

Schema: `docs/04-data/foundation/event_history_architecture.md`
- EventHistory/Replay 分离互补：EventStore 记录输出事件 (StoredEvent)，Replay 记录输入命令 (Command)
- StoredEvent 从 GameplayEvent + ObservableEvent 转换，丢弃 EventBus 分发元数据
- 默认 1000 条 RingBuffer，约 300KB 内存，写入非阻塞
- 三层事件归属：Capability Events（能力层）、Domain Events（业务层）、Infra Events（基础设施层）
- Infra 事件默认不记录 (record_infra_events: false)
- StoredEvent 不参与存档持久化（延续 event_schema.md §8 决策）
- `fields: HashMap<String, String>` 作为主要向前兼容机制——新增字段通过新增键值对
- LogCode/Domain 枚举使用 `#[serde(other)]` 兜底 variant 支持 enum 扩展
- 与 ADR-049 共享事件集成：TurnEnded/TurnStarted/BattleStarted/BattleEnded 作为时间轴标记

**Why:** 填补"事后分析"能力空白——QA/开发者可直接查询事件历史，与 Replay 形成完整因果链。
