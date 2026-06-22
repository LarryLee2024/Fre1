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
| `content` | 内容加载 | CNT 系列 |
| `infra.replay` | 回放 | RPL 系列 |

`#[instrument]` 用法示例：`#[tracing::instrument(skip_all, target = "domain.ability.activation", fields(...))]`

target 是 Domain 的职责，不由 LogCode 承担。每个 Domain 枚举变体决定自己的 target 字符串（详见 `src/shared/diagnostics/domain.rs`）。

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
    emit_info!(
        LogCode::BAT001,
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
    let e = trigger.event();
    emit_info!(
        LogCode::PRG002,
        entity = ?e.entity,
        old = e.old_level,
        new = e.new_level,
        "角色升级",
    );
}

// ❌ 错误：code/event 重复出现在 emit_info! 中（已在 span 中）
emit_info!(
    LogCode::PRG002,
    code = ?LogCode::PRG002,     // ❌ 重复
    event = "level_up",          // ❌ 重复
    entity = ?e.entity,
    ...
);
```

注意：`emit_info!`/`emit_warn!`/`emit_debug!` 宏内部不指定 target——target 继承自 `#[instrument]` span。
LogCode 只负责事件编码，不负责路由（见 Domain::target）。

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

### Observability Facade — 观测入口的 L1→L2 演进

#### L1（当前）：emit_info! 宏

Observer 使用 `emit_info!`/`emit_warn!`/`emit_debug!` 宏作为统一入口：

```rust
#[instrument(skip_all, target = "domain.progression", fields(code = ?LogCode::PRG002))]
fn on_level_up(trigger: On<LevelUp>) {
    let e = trigger.event();
    emit_info!(LogCode::PRG002, entity = ?e.entity, old = e.old_level, "角色升级");
}
```

宏内部自动完成：
1. `telemetry::record(LogCode)` — 记录度量
2. `tracing::info!(fields...)` — 输出结构化日志（target 从 span 继承）

收益：
- Observer 不需要手动调用 `metrics::record`
- Observer 不需要手动指定 `target`（由 span 承载）
- `event` 字符串从 LogCode 自动派生

#### L2（未来）：record_event 统一分发

未来将提供 `telemetry::record_event(&event)` 作为唯一入口：

```rust
#[instrument(skip_all, target = Domain::Progression.target())]
fn on_level_up(trigger: On<LevelUp>) {
    telemetry::record_event(trigger.event());
}
```

`record_event` 内部会将 `ObservableEvent` 统一分发到所有 sink：
```
record_event(event)
    ├── LoggerSink    → tracing 结构化日志
    ├── MetricSink    → 全局计数 + 定期汇总
    ├── ReplaySink    → 录制完整事件对象
    ├── AuditSink     → 审计轨迹持久化
    └── AnalyticsSink → 运行时统计分析
```

当前 `record_event` 仅实现 MetricSink 部分，其余为预留扩展点。

### emit_info! 使用限制

`emit_info!`/`emit_warn!`/`emit_debug!` 宏只能在以下位置使用：

| 允许使用 | 禁止使用 |
|---------|---------|
| Observer 层（infra/logging/observers/） | Domain Service |
| Adapter 层（infra/ 下其他适配器） | Domain Model |
| Infrastructure 层初始化日志 | Ability Executor |
| | Pipeline 内部 |
| | 任何领域逻辑代码 |

DDD 原则：领域代码不知道 Observability 的存在，领域只产生事件。

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

> **Note**: LogCode/LogCategory types are physically located in `shared/diagnostics/` but logically owned by the Infrastructure logging system.

src/infra/logging/                # L2: 日志基础设施实现
├── mod.rs
├── plugin.rs                     # LoggingPlugin
├── observers/
│   ├── mod.rs
│   ├── ability_logger.rs       # 技能事件 → INFO
│   ├── battle_logger.rs        # 战斗生命周期 → INFO
│   ├── camp_rest_logger.rs     # 营地休息事件 → INFO
│   ├── content_logger.rs       # 内容加载事件 → INFO
│   ├── crafting_logger.rs      # 制作/附魔事件 → INFO
│   ├── economy_logger.rs       # 交易/货币事件 → INFO
│   ├── effect_logger.rs        # 效果生命周期 → INFO/WARN
│   ├── faction_logger.rs       # 声望/阵营事件 → INFO
│   ├── inventory_logger.rs     # 物品/装备事件 → INFO
│   ├── narrative_logger.rs     # 对话/剧情事件 → INFO
│   ├── party_logger.rs         # 队伍管理事件 → INFO
│   ├── progression_logger.rs   # 经验/升级事件 → INFO
│   ├── quest_logger.rs         # 任务生命周期 → INFO/WARN
│   ├── reaction_logger.rs      # 反应/援护事件 → INFO
│   ├── spell_logger.rs         # 法术事件 → INFO
│   ├── summon_logger.rs        # 召唤物事件 → INFO
│   ├── tactical_logger.rs      # 战术移动事件 → INFO
│   ├── terrain_logger.rs       # 地形效果事件 → INFO
│   └── turn_logger.rs          # 回合流转事件 → INFO
├── rate_limit/
│   ├── mod.rs
│   └── once_guard.rs             # warn_once!/error_once! 实现
└── sinks/                        # 预留（console/file/telemetry）
    └── mod.rs
```

### ObservableEvent 契约（2026-06-28 新增，2026-06-30 扩展 const DOMAIN/CODE）

从 `docs/ai_ignore_this_dir/14可观测.md` 提炼的 Single Source of Observability 原则：

1. **ObservableEvent trait** — 位于 `shared/diagnostics/observable.rs`，领域事件实现此 trait 后
   可观测系统可以自动提取事件的结构化字段，无需每个 Observer 手动展开事件字段
2. **Observability Facade** — `infra/logging/telemetry.rs` 提供 `emit_info!`/`emit_warn!` 宏，
   Observer 不再需要手动指定 target 和调用 metrics::record
3. **Domain 路由分离** — `shared/diagnostics/domain.rs` 定义 Domain 枚举，LogCode 不再承担
   路由职责。`ObservableEvent::DOMAIN` 决定 tracing target，`ObservableEvent::CODE` 决定事件编码
4. **业务代码零感知** — Observer 是基础设施代码，领域代码只产生领域事件，不知道日志/指标的存在

```rust
// ObservableEvent trait（当前版本）
pub trait ObservableEvent: Debug + Send + Sync + 'static {
    const DOMAIN: Domain;          // 路由域 → 决定 tracing target
    const CODE: LogCode;           // 事件编码 → 唯一标识

    fn log_code(&self) -> LogCode; // 默认返回 Self::CODE，支持运行时覆盖
    fn record_fields(&self, _collector: &mut FieldCollector) {}
}
```

## Communication Design

- **Hook**: 不适用
- **Trigger**: 领域事件通过 `commands.trigger()` 发射
- **Observer**: Logging observers 通过 `app.add_observer()` 订阅领域事件（共 73 个 observer）
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

- 🟥 domain 层直接 `info!`/`warn!`/`emit_info!`/`emit_warn!` 输出业务事件（必须走领域事件链路，emit 宏只能在 Observer/Infra 层使用）
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
