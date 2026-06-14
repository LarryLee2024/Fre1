---
id: history.archive.adr.ADR-006-domain-events-audit-trail
title: ADR-006-domain-events-audit-trail
status: archived
owner: architect
created: 2026-06-14
updated: 2026-06-14
---

# ADR-006: 领域事件白名单与审计轨迹架构

## 状态

Accepted

## 背景

当前代码中领域事件管理存在不规范的情况：
- 部分事件为临时副作用随意创建（违反 §2.2.6）
- 缺乏统一的事件白名单管理
- 战斗审计轨迹不完整，难以支持 Battle Replay

需要建立领域事件白名单与审计轨迹架构，实现：
1. 所有领域事件纳入统一白名单管理
2. 核心战斗流程生成结构化审计轨迹
3. 支持 Battle Replay、录像、Bug 复现

## 引用的领域规则

- `docs/AI开发宪法.md` §2.2.5 — 领域事件是唯一业务事实源
- `docs/AI开发宪法.md` §2.2.6 — 领域事件白名单管理
- `docs/AI开发宪法.md` §14.10.1 — 事件统一事实源
- `docs/AI开发宪法.md` §14.10.2 — 事件白名单
- `docs/AI开发宪法.md` §14.10.3 — 审计轨迹（Audit Trail）

## 决策

采用「领域事件白名单 + 审计轨迹」架构：

### 事件白名单管理

所有正式领域事件必须收录在白名单文档中，新增事件必须先更新白名单。

核心事件分类：

| 类别 | 事件 | 用途 |
|------|------|------|
| **战斗** | BattleStarted, BattleEnded | 战斗生命周期 |
| **回合** | TurnStarted, TurnEnded | 回合生命周期 |
| **单位** | UnitMoved, UnitAttacked, UnitDamaged, UnitDied | 单位行为 |
| **Buff** | BuffApplied, BuffRemoved, BuffExpired | Buff 生命周期 |
| **技能** | SkillCastStarted, SkillCastFinished | 技能释放 |
| **任务** | QuestAccepted, QuestCompleted | 任务进度 |

### 审计轨迹架构

```
业务代码 → DomainEvent → AuditTrail
                            ↓
                    ┌───────┴───────┐
                    │ BattleReplay  │
                    │ BattleLogUI   │
                    │ Achievement   │
                    │ QuestSystem   │
                    └───────────────┘
```

审计轨迹数据结构：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub entity: Option<Entity>,
    pub data: serde_json::Value,
    pub metadata: AuditMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditMetadata {
    pub turn_number: u32,
    pub phase: String,
    pub source: String,
}
```

核心原则：
1. **事件是唯一事实源**：日志、回放、UI、成就共用同一套事件
2. **白名单管理**：新增事件必须先更新白名单
3. **审计轨迹**：核心战斗流程必须生成结构化审计数据
4. **下游复用**：一次事件触发，全链路复用

## Module Design

```
src/
└── infrastructure/
    └── audit/
        ├── mod.rs              # 模块入口，注册 AuditPlugin
        ├── trail.rs            # 审计轨迹收集器
        ├── event.rs            # 审计事件数据结构
        ├── whitelist.rs        # 事件白名单管理
        └── replay.rs           # Battle Replay 支持（可选）
```

### 职责划分

| 文件 | 职责 |
|------|------|
| `mod.rs` | 统一导出所有审计类型 |
| `trail.rs` | 审计轨迹收集器，监听 DomainEvent |
| `event.rs` | 审计事件数据结构定义 |
| `whitelist.rs` | 事件白名单管理，校验事件合法性 |
| `replay.rs` | Battle Replay 支持（可选，后期实现） |

## Communication Design

### Message（跨 Feature 广播）
所有 DomainEvent 通过 Message 广播，审计轨迹收集器监听。

### Observer（局部响应）
不涉及。

### Hook（组件固有行为）
不涉及。

### 函数调用
审计轨迹收集器内部调用序列化函数。

## 边界定义

### 允许
- 审计轨迹收集器监听所有 DomainEvent
- 审计轨迹支持序列化/反序列化
- 事件白名单管理新增事件校验

### 禁止
- 🟥 禁止为临时副作用随意新增领域事件
- 🟥 禁止业务代码直接调用审计轨迹 API
- 🟥 禁止在审计轨迹中包含敏感信息
- 🟥 禁止审计轨迹影响业务逻辑执行

## Forbidden（禁止事项）

- 🟥 禁止：为临时副作用随意新增领域事件 — 理由：控制事件数量爆炸，保证领域事件的权威性与一致性（§2.2.6）
- 🟥 禁止：业务代码直接调用审计轨迹 API — 理由：审计轨迹是事件的下游消费者，不是业务代码主动调用的功能
- 🟥 禁止：在审计轨迹中包含敏感信息 — 理由：审计轨迹可能被持久化或传输
- 🟥 禁止：审计轨迹影响业务逻辑执行 — 理由：审计轨迹是观察者，不是执行者

## Definition / Instance Design

### Definition（不可变配置）
- 事件白名单文档（docs/domain/event_whitelist.md）

### Instance（运行时状态）
- `AuditTrail`：审计轨迹收集器，存储事件列表
- `AuditEvent`：单个审计事件实例

## 后果

### 正面
1. **统一管理**：所有领域事件纳入白名单，防止事件膨胀
2. **可追溯**：审计轨迹支持 Battle Replay、录像、Bug 复现
3. **下游复用**：日志、回放、UI、成就共用同一套事件
4. **可扩展**：新增下游只需监听审计轨迹

### 负面
1. **迁移成本**：现有临时事件需要逐步迁移到白名单
2. **代码量增加**：需要定义审计事件数据结构和收集器
3. **性能开销**：审计轨迹收集需要额外的序列化开销

## 替代方案

### 方案1：不管理事件白名单
优点：零迁移成本
缺点：事件数量爆炸，难以维护
**结论：否决** — 违反 §2.2.6

### 方案2：使用 tracing 作为审计轨迹
优点：利用 tracing 生态
缺点：tracing 是文本日志，不支持结构化查询和回放
**结论：否决** — 不适合 Battle Replay 场景

### 方案3：每个下游独立收集事件
优点：松耦合
缺点：重复收集，资源浪费
**结论：否决** — 违反事件统一事实源原则

## 架构合规性检查

- [x] 符合 ECS 约束（Entity=ID, Component=数据, System=行为）
- [x] 符合 Feature First 原则（infrastructure/audit/ 作为独立模块）
- [x] 符合领域事件白名单管理原则
- [x] 符合审计轨迹架构要求
- [x] 没有创建禁止的模块（components.rs/systems.rs/utils.rs）
- [x] 所有禁止事项已明确列出
- [x] 已检查 docs/AI开发宪法.md §2.2.5, §14.10
