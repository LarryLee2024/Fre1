# ADR-049: 跨域共享事件模式

## 状态
✅ Accepted

## 背景
综合评审发现 `terrain/systems/surface_system.rs` 直接 import `combat::OnTurnEnd`，违反 §3.5.2 Domain 间禁止直接依赖规则。terrain 域需要在回合结束时递减表面覆盖持续回合数，但不应直接引用 combat 域的事件类型。

## 引用的领域规则
- `docs/02-domain/domains/terrain_domain.md` — 地形表面恢复规则
- `docs/02-domain/domains/combat_domain.md` — 回合生命周期事件

## 决策
在 `src/core/events.rs` 中定义跨域共享事件，供多个 Domain 订阅，避免 Domain 间直接依赖。

## Module Design
```
src/core/events.rs       # 跨域共享事件定义
src/core/mod.rs          # 导出 events 模块
src/core/core_plugin.rs  # 注册共享事件
```

## Communication Design
- **Hook**: 不适用
- **Trigger**: 共享事件通过 `commands.trigger()` 发射
- **Observer**: 各 Domain 通过 `app.add_observer()` 订阅共享事件
- **Message**: 不适用（共享事件替代跨域 Message）
- **Query API**: 不适用

## 边界定义
- 允许：任何 Domain 发射共享事件
- 允许：任何 Domain 订阅共享事件
- 禁止：Domain 间直接 import 对方的事件类型
- 禁止：共享事件携带过多数据（应只携带 Entity ID + 最小上下文）

## Forbidden（禁止事项）
- 🟥 Domain A 直接 import Domain B 的事件类型
- 🟥 共享事件包含业务逻辑
- 🟥 共享事件替代 Query API 的读操作

## Definition / Instance Design
- Definition（不可变配置）：不适用
- Instance（运行时状态）：共享事件作为 ECS Event 触发

## 实现的共享事件

| 事件 | 发射方 | 消费方 | 用途 |
|------|--------|--------|------|
| `TurnEnded` | combat pipeline | terrain (表面恢复) | 回合结束通知 |
| `TurnStarted` | combat pipeline | 待定 | 回合开始通知 |
| `BattleStarted` | combat | 待定 | 战斗开始通知 |
| `BattleEnded` | combat | 待定 | 战斗结束通知 |

## 后果
### 正面
- 消除跨域直接依赖
- 符合 Data Law 012（域间禁止直接数据引用）
- 新 Domain 可安全订阅回合事件

### 负面
- 需要在 `core/events.rs` 中维护共享事件列表
- 共享事件增加了一层间接性

## 替代方案
1. **将 OnTurnEnd 下沉到 capabilities/event/**: 可行，但 OnTurnEnd 携带 combat 特有的 `unit: Entity` 语义，不属于通用机制
2. **通过 integration/ 暴露 Query API**: 可行，但 terrain 需要的是事件驱动（回合结束时触发），不是轮询查询

## 文件状态
| 文件 | 状态 | 负责人 | 完成日期 |
|------|------|--------|----------|
| `ADR-049-shared-cross-domain-events.md` | ✅ stable | architect | 2026-06-19 |

## 后续更新

### D2-6: 共享事件作为 Event History 种子数据

本 ADR 定义的共享事件（`TurnEnded`、`TurnStarted`、`BattleStarted`、`BattleEnded`）已被确认为 Event History（详见 `docs/01-architecture/40-cross-cutting/ADR-059-event-history.md`）的种子数据：

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
                          EventHistoryStore
```

**架构含义**：
- Event History 的 replay 回放能力依赖这些共享事件作为时间轴标记
- 新增共享事件时应同时评估其对 Event History 的时间轴意义
- 共享事件的参数字段应包含 Event History 索引所需的最小上下文（entity ID + frame number）

详见 `docs/04-data/foundation/event_history_architecture.md`。
