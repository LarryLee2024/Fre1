---
id: 01-architecture.40-cross-cutting.ADR-052
title: "ADR-052: 日志架构（领域事件驱动 + LogCode + 结构化日志）"
status: Proposed
owner: architect
created: 2026-06-25
tags:
  - architecture
  - logging
  - observability
  - infrastructure
---

# ADR-052: 日志架构（领域事件驱动 + LogCode + 结构化日志）

## 状态

Accepted

## 背景

宪法 §11.1-11.5 已定义日志架构原则（领域事件驱动、结构化日志、Observer 在基础设施层），但代码层面未实现：
- `src/infra/logging/` 不存在
- `src/shared/diagnostics/` 不存在
- 189+ 处 domain 层直接 `info!`/`warn!` 违反宪法 §11.4
- 无 LogCode、无 DiagnosticContext、无日志风暴保护

50 万行级项目需要：LogCode 编码体系、CorrelationId 链路、DiagnosticContext 自动携带上下文、日志风暴保护、Span 链路。

## 引用的领域规则

- `docs/00-governance/ai-constitution-complete.md` §11.1-11.5 — 日志宪法
- `docs/00-governance/coding-rules.md` §14 — 日志规范
- `docs/02-domain/` — 各域事件消费表（日志为下游消费者）

## 决策

### 核心原则

```
领域层不写日志
领域层发 Event

Logging 属于 Infrastructure
通过 Observer 监听 Domain Event 生成日志

统一使用 tracing
统一结构化日志
统一 Span 链路
```

### 日志分级（宪法 §11.2 扩展）

| 级别 | 用途 | 示例 |
|------|------|------|
| ERROR | 程序异常，预算 = 0，出现即修 | `entity_not_found` |
| WARN | 异常但可恢复 | `missing_localization`、`config_fallback` |
| INFO | 核心业务事件边界，用于 Replay | `battle_started`、`skill_cast` |
| DEBUG | 开发调试，发布版关闭 | `damage_calculated`、`buff_stacking` |
| TRACE | 极细粒度，仅专项调试 | `tag_query`、`attribute_resolve` |

### LogCode 编码体系

统一编码，替代文本搜索。格式：`{域前缀}{三位编号}`

```
域前缀分配：
BAT — Combat          TAC — Tactical         TER — Terrain
ABL — Ability         EFF — Effect           TAG — Tag
MOD — Modifier        AGG — Aggregator       TRG — Trigger
SPR — Spell           RCT — Reaction         QST — Quest
PRG — Progression     INV — Inventory        ECO — Economy
CRF — Crafting        FAC — Faction          PRY — Party
CNR — CampRest        NAR — Narrative        SUM — Summon
CNT — Content (infra) SAV — Save (infra)     RPL — Replay (infra)
```

使用示例：
```rust
info!(
    code = ?LogCode::BAT001,
    battle_id = ?e.battle_id,
    participants = e.participant_count,
    "battle_started"
);
```

### CorrelationId 链路

战斗调试神器，替代时间戳：

```
Battle#1 → Turn#8 → Action#12
```

```rust
pub struct BattleId(pub u64);
pub struct TurnId(pub u32);
pub struct ActionId(pub u64);
```

日志输出：
```json
{"battle":"battle_1001","turn":"turn_8","action":"action_991","event":"damage_applied"}
```

### DiagnosticContext

自动携带上下文，避免每个日志手写：

```rust
pub struct DiagnosticContext {
    pub battle_id: Option<BattleId>,
    pub turn_id: Option<TurnId>,
    pub action_id: Option<ActionId>,
    pub entity: Option<Entity>,
}
```

便捷方法：
```rust
impl DiagnosticContext {
    pub fn info(&self, code: LogCode, event: &str) { ... }
    pub fn warn(&self, code: LogCode, event: &str) { ... }
}
```

### 日志风暴保护

Bevy ECS 特别容易出问题（1000 实体 × 60FPS = 百万日志）：

```rust
// 禁止
for entity in query.iter() {
    warn!("missing buff");  // ❌ 1000 条
}

// 正确
warn_once!(LogCode::EFF003, "missing_buff");  // ✅ 只输出一次
```

实现：`OnceGuard` 基于 `AtomicBool`，每个调用点只触发一次。

### Span 链路

tracing 最大价值，自动生成调用链：

```rust
#[instrument(skip_all, fields(battle_id, turn_id))]
fn execute_turn(...) { }
```

输出：
```
battle
 └─ turn
     └─ ability
         └─ effect
```

### 领域事件 → 日志的四路消费

```
Domain Event
    │
    ├── Logger      （人看 — tracing 日志）
    ├── Metrics     （统计 — counter!/gauge!）
    ├── Replay      （回放 — 命令录制）
    └── Snapshot    （复现 — 状态快照）
```

## Module Design

```
src/shared/diagnostics/           # L0: 日志基础设施类型
├── mod.rs
├── log_code.rs                   # LogCode 枚举
├── log_category.rs               # LogCategory 分类
├── correlation.rs                # CorrelationId（BattleId, TurnId, ActionId）
└── context.rs                    # DiagnosticContext

src/infra/logging/                # L2: 日志基础设施实现
├── mod.rs
├── plugin.rs                     # LoggingPlugin
├── observers/
│   ├── mod.rs
│   ├── battle_logger.rs          # 监听战斗事件 → INFO 日志
│   ├── ability_logger.rs         # 监听技能事件 → INFO 日志
│   ├── effect_logger.rs          # 监听效果事件 → INFO 日志
│   ├── spell_logger.rs           # 监听法术事件 → INFO 日志
│   ├── quest_logger.rs           # 监听任务事件 → INFO 日志
│   ├── content_logger.rs         # 监听内容加载 → WARN 日志
│   └── turn_logger.rs            # 监听回合事件 → INFO 日志
├── rate_limit/
│   ├── mod.rs
│   └── once_guard.rs             # warn_once!/error_once! 实现
└── sinks/                        # 预留（console/file/telemetry）
    └── mod.rs
```

## Communication Design

- **Hook**: 不适用
- **Trigger**: 领域事件通过 `commands.trigger()` 发射
- **Observer**: Logging observers 通过 `app.add_observer()` 订阅领域事件
- **Message**: 不适用
- **Query API**: 不适用

## 边界定义

- 允许：infra/logging/ 监听所有领域事件
- 允许：shared/diagnostics/ 被任何层引用（纯类型）
- 允许：infra 层保留基础设施自身的 `info!`/`error!`（如"存档成功"）
- 禁止：domain 层直接调用 `info!`/`warn!` 输出业务事件
- 禁止：domain 层 import infra/logging/ 的任何类型
- 禁止：LogObserver 包含业务逻辑

## Forbidden（禁止事项）

- 🟥 domain 层直接 `info!`/`warn!` 输出业务事件（必须走领域事件链路）
- 🟥 LogObserver 包含业务逻辑（只负责日志输出）
- 🟥 日志使用 `format!` 拼接字符串（必须结构化字段）
- 🟥 循环/迭代器内部输出 INFO 级别日志
- 🟥 Release 版本每帧系统输出 INFO/DEBUG
- 🟥 ERROR 预算非零（出现 error! 就是 Bug）
- 🟥 LogCode 使用文本搜索替代编码（必须用 `code = ?LogCode::XXX`）

## Definition / Instance Design

- Definition（不可变配置）：LogCode 枚举、域前缀映射
- Instance（运行时状态）：DiagnosticContext（Resource）、OnceGuard（Local）

## 后果

### 正面
- 日志格式统一，AI 可搜索 LogCode
- CorrelationId 支持战斗全链路调试
- DiagnosticContext 减少重复代码
- 日志风暴保护防止性能问题
- 四路消费（Logger/Metrics/Replay/Snapshot）解耦

### 负面
- 需要为每个域事件定义 LogCode（初始工作量）
- 领域事件需要携带足够上下文供日志使用
- infra/logging/ 依赖所有域的事件类型（通过 Cargo.toml 依赖）

## 替代方案

### 方案 A：领域层直接 `info!`（已拒绝）
- 违反宪法 §11.4
- 日志格式不统一
- 无法做四路消费

### 方案 B：日志 = 回放（已拒绝）
- 日志和回放混在一起
- 无法独立演进
- 性能互相影响

### 方案 C：本文案（采纳）
- 领域事件驱动
- LogCode 编码
- 四路消费解耦
- 符合 50 万行级项目标准
