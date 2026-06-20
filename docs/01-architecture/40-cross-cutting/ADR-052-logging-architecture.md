---
id: 01-architecture.40-cross-cutting.ADR-052
title: "ADR-052: 日志架构（领域事件驱动 + LogCode + 结构化日志）"
status: Accepted
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

### Target 层级规范

所有日志 `target` 必须遵循 `domain.module.submodule` 层级格式：

| target | 模块 | 说明 |
|--------|------|------|
| `domain.combat` | 战斗核心流程 | BAT 系列 |
| `domain.ability.activation` | 技能激活 | ABL001–ABL004 |
| `domain.effect` | 效果系统 | EFF 系列 |
| `domain.tactical.turn` | 回合流转 | TAC 系列 |
| `domain.tactical.movement` | 单位移动 | TAC001–TAC005 |
| `domain.terrain` | 地形效果 | TER 系列 |
| `domain.spell` | 法术系统 | SPR 系列 |
| `domain.reaction` | 反应/援护 | RCT 系列 |
| `domain.progression` | 成长体系 | PRG 系列 |
| `domain.inventory` | 背包物品 | INV 系列 |
| `domain.economy` | 经济交易 | ECO 系列 |
| `domain.crafting` | 制作系统 | CRF 系列 |
| `domain.faction` | 阵营关系 | FAC 系列 |
| `domain.party` | 队伍管理 | PRY 系列 |
| `domain.camp_rest` | 营地休息 | CNR 系列 |
| `domain.narrative` | 叙事对话 | NAR 系列 |
| `domain.quest` | 任务系统 | QST 系列 |
| `domain.summon` | 召唤系统 | SUM 系列 |
| `infra.save` | 存档 | SAV 系列 |
| `infra.content` | 内容加载 | CNT 系列 |
| `infra.replay` | 回放 | RPL 系列 |

`#[instrument]` 用法示例：`#[tracing::instrument(skip_all, target = "domain.ability.activation", fields(...))]`

> 注意：`#[instrument]` 的 `target` 只影响 span 层。`info!()` 内部仍需显式传递 `target` 参数，因为 tracing 的 event 不会从父 span 继承 target。后续通过 `telemetry::emit` 统一封装。

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

使用示例（span 放不变量，event 放变量）：
```rust
#[tracing::instrument(skip_all, target = "domain.combat", fields(
    code = ?LogCode::BAT001,
    event = "battle_started",
))]
fn on_battle_started(trigger: On<BattleStarted>) {
    let e = trigger.event();
    metrics::record(LogCode::BAT001);
    info!(
        target = "domain.combat",
        battle_id = ?e.battle_id,
        participants = e.participant_count,
        "战斗开始",
    );
}
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

### Observer 字段分离规范（Span 不变量 vs Event 变量）

`#[instrument]` 的 `fields()` 和 `info!()` 的参数有严格分工，禁止重复：

| 位置 | 内容 | 示例 |
|------|------|------|
| `#[instrument(fields(...))]` | 不变量：本次调用中所有日志共用的固定值 | `code = ?LogCode::PRG002`, `event = "level_up"` |
| `info!()` / `warn!()` | 变量：仅本次调用独有的动态数据 | `entity`, `old`, `new`, `amount` |

```rust
// ✅ 正确：span 放不变量，event 只放变量
#[tracing::instrument(skip_all, target = "domain.progression", fields(
    code = ?LogCode::PRG002,
    event = "level_up",
))]
fn on_level_up(trigger: On<LevelUp>) {
    metrics::record(LogCode::PRG002);
    let e = trigger.event();
    info!(
        target = "domain.progression",
        entity = ?e.entity,
        old = e.old_level,
        new = e.new_level,
        "角色升级",
    );
}

// ❌ 错误：code/event 重复出现在 info!() 中（已在 span 中）
info!(
    code = ?LogCode::PRG002,     // ❌ 重复
    event = "level_up",          // ❌ 重复
    entity = ?e.entity,
    ...
);
```

注意：`#[instrument]` 的 `target` 只作用于 span 层。`info!()` 内部仍需显式传递 `target` 参数，因为 tracing 的 event 不会继承父 span 的 target。后续通过 `telemetry::emit` 统一封装后可解决此问题。

### 结构化字段低基数要求

所有结构化字段必须使用 ID 类型（`entity_id`、`spec_id`、`item_id`），**禁止使用自然语言文本**：

```rust
// ❌ 高基数：context_desc 是自然语言，每个调用值都不同
info!(context_desc = "caster's level 3 fireball");

// ✅ 低基数：使用 ID
info!(spec_id = ?spec_id, entity = ?entity);
```

`context_desc` 等自由文本字段会导致日志聚合分析系统（如 Loki、Elasticsearch）的基数爆炸，长期存在会压垮存储和查询。应改用 `context_id` + `LocalizationKey` 方案。

### 字段语言规范

- `event` 字段值（结构化字段）**必须使用英文**（`"level_up"`、`"battle_started"`）——结构化日志的消费者是机器（日志聚合系统、AI 搜索），英文保证可移植性
- `message`（字符串消息）可以用中文或英文——消费者是人，以开发者阅读效率为主
- `LogCode::description()` 可以用中文（例如 `"角色升级"`）

### telemetry::emit — 未来统一入口

当前 Observer 存在三要素重复模式：`#[instrument]` span + `metrics::record()` + `info!()`，其中 target 需要在两处重复指定。后续引入 `telemetry::emit()` 统一封装：

```rust
// 未来模式
#[instrument(skip_all, fields(code = ?LogCode::PRG002))]
fn on_level_up(trigger: On<LevelUp>) {
    let e = trigger.event();
    telemetry::emit(LogCode::PRG002, e);
}

// telemetry::emit 内部实现：
// 1. metrics::record(LogCode::PRG002)
// 2. 自动识别 target = "domain.progression"（从 LogCode 域前缀映射）
// 3. output!(LogCode::PRG002, entity = ?e.entity, ...)
// 4. span 已携带 code/event，无需重复
```

收益：
- 消除 target 两处重复指定
- 自动统一 metrics 和 log 调用
- `event` 字符串从 LogCode 自动派生，消除"LogCode 和 event 是同一件事的两种表达"的冗余

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
- 🟥 在 `info!()` 中重复 `#[instrument(fields(...))]` 已覆盖的 `code`/`event` 字段（span 负责不变量，event 只放变量）
- 🟥 `event` 字段值使用中文（结构化日志是机器消费的，必须英文）
- 🟥 使用 `context_desc` 等自然语言文本作为结构化字段（高基数风险，必须改用 ID）

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
