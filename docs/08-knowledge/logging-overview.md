---
id: 08-knowledge.logging-overview
title: 日志系统深度解析 — 从宪法到代码
status: draft
owner: architect
created: 2026-06-19
tags:
  - knowledge
  - logging
  - observability
  - tracing
---

# 日志系统深度解析

> 目标读者：新加入项目的开发者，或其他想理解日志系统全貌的人。
> 读完本文，你会知道日志是怎么设计出来的、代码放在哪、一条日志从产生到你看到它经历了什么。

---

## 目录

1. [核心思想：为什么日志要这么设计？](#1-核心思想为什么日志要这么设计)
2. [整体架构全景图](#2-整体架构全景图)
3. [第一层（L0）：共享类型层 — 日志的「方言」](#3-第一层-l0共享类型层--日志的方言)
4. [第二层（L2）：基础设施实现层 — 日志的「记者」](#4-第二层-l2基础设施实现层--日志的记者)
5. [数据流全景：一条日志的诞生](#5-数据流全景一条日志的诞生)
6. [实战：如何添加一条新日志](#6-实战如何添加一条新日志)
7. [现状盘点：已经做了什么，还缺什么](#7-现状盘点已经做了什么还缺什么)
8. [规则速查：该做什么和不该做什么](#8-规则速查该做什么和不该做什么)

---

## 1. 核心思想：为什么日志要这么设计？

### 1.1 最大的原则：领域层不写日志

传统游戏项目的日志长这样：

```rust
// ❌ 传统做法：在业务逻辑里直接写日志
fn execute_damage(mut health: ResMut<Health>, damage: u32) {
    health.current = health.current.saturating_sub(damage);
    info!("造成了 {} 点伤害", damage);  // 日志和业务逻辑混一起
}
```

看起来没问题，但在 Fre 这种 50 万行级别的项目里会出现三个问题：

1. **日志格式不统一** — 每个人写的格式不一样，搜日志像大海捞针
2. **日志和回放耦合** — 回放系统也需要知道「造成了伤害」这件事，但日志文本没法用来恢复游戏状态，必须重新定义事件
3. **日志风暴** — 如果循环里不小心写了个 `info!`，1000 个实体瞬间输出 1000 行，日志文件爆炸

所以 Fre 的做法是反过来：**领域层只产生事件，不写日志。**

```rust
// ✅ Fre 的做法：领域层发事件
commands.trigger(DamageDealt {
    target: enemy,
    amount: damage,
    source: player,
});

// 日志？那是基础设施层的事，领域层不管
```

这种做法叫做**领域事件驱动日志**，记录在宪法 §11.1–11.5 和 ADR-052 中。

### 1.2 日志的四个下游消费者

一个领域事件产生后，可以被**四路消费**：

```
领域事件
  │
  ├── Logger    → 人看的日志（tracing::info! 输出到控制台）
  ├── Metrics   → 统计数据（战斗次数、胜率等）
  ├── Replay    → 回放录制（保存命令序列用于回放）
  └── Snapshot  → 状态快照（存档等）
```

日志只是其中一路。这个设计保证了：**领域事件是一切事实的唯一源头**。

### 1.3 三层架构中的日志位置

Fre 的代码分为三层，日志系统跨越其中两层：

```
┌─ L0: Shared（原子层）─────────────────────────────────┐
│  shared/diagnostics/                                    │
│  ├── log_code.rs         ← LogCode 枚举（日志类型编码）   │
│  ├── log_category.rs     ← LogCategory 分类              │
│  ├── correlation.rs      ← CorrelationId（关联链路）      │
│  └── context.rs          ← DiagnosticContext（诊断上下文） │
│  这些是纯类型定义，只有数据结构，没有行为逻辑              │
├─ L1: Core（领域规则层）─────────────────────────────────│
│  这里绝对不能有日志代码（新项目还有历史遗留违规，正在清理中） │
├─ L2: Infra（技术实现层）────────────────────────────────│
│  infra/logging/                                          │
│  ├── plugin.rs           ← LoggingPlugin 注册 Observer   │
│  ├── observers/           ← 领域事件的日志监听器          │
│  ├── rate_limit/          ← 日志风暴保护                 │
│  └── sinks/               ← 日志输出后端（预留）          │
└─────────────────────────────────────────────────────────┘
```

---

## 2. 整体架构全景图

```
                                                    ┌─ 四条红线 ──────────────────────────┐
                                                    │ 领域层禁止 info!() 直接写业务事件     │
                                                    │ 禁止循环内 INFO 级别日志              │
                                                    │ ERROR 出现即 Bug                     │
                                                    │ 所有 INFO 必须带 LogCode              │
                                                    └──────────────────────────────────────┘

┌─────── 领域代码 (L1 Core) ───────┐
│                                  │
│  commands.trigger(TurnStarted    │
│    { unit: entity })             │──────────┐
│                                  │          │
└──────────────────────────────────┘          │  Bevy 调度
                                              ▼
┌─────── LoggingPlugin (L2 Infra) ─────────────────────────────────────┐
│                                                                     │
│  7 个 Observer 模块 × 共 18 个事件监听器                               │
│  (battle / turn / spell / ability / effect / quest / content)       │
│                                                                     │
│  ┌─ Observable（以 turn_logger 为例）────────────────────────────┐    │
│  │  fn on_turn_started(trigger: On<TurnStarted>) {              │    │
│  │      let e = trigger.event();                                │    │
│  │      info!(                                                  │    │
│  │          code = ?LogCode::BAT005,    ← 编码                 │    │
│  │          event = "turn_started",     ← 事件名               │    │
│  │          unit = ?e.unit,             ← 结构化字段           │    │
│  │          "turn_started"             ← 人类可读消息          │    │
│  │      );                                                     │    │
│  │  }                                                           │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                                                     │
└───────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────── tracing (Bevy 内置 LogPlugin) ──────┐
│                                            │
│  tracing::info! → tracing-subscriber       │
│                    ↓                       │
│              控制台输出                      │
│              (stderr, 带颜色和时间戳)         │
└────────────────────────────────────────────┘
```

---

## 3. 第一层（L0）：共享类型层 — 日志的「方言」

在 `src/shared/diagnostics/` 下，定义了四种核心类型，它们是日志的「方言」，所有层都能引用。

### 3.1 LogCode — 日志类型编码

LogCode 是**每个日志事件的唯一身份证**。格式是 `{域前缀}{3位数字}`。

```
BAT001 → 战斗开始     SPR001 → 法术施放     PRG001 → 获得经验
BAT002 → 战斗结束     EFF001 → 效果施加     SAV001 → 存档创建
```

为什么不用文本搜索？因为：
- 文本搜索：搜 "damage" 能搜到几百条，不知道哪个是哪个
- LogCode：搜 `BAT007` 只有这一个事件，AI 也能理解

目前定义了 20 个域名、约 150 个编码。每个 LogCode 有两个方法：

```rust
LogCode::BAT001.code()          // 返回 "BAT001"
LogCode::BAT001.description()   // 返回 "战斗开始"
```

### 3.2 LogCategory — 日志分类

五个大分类，用来对日志做聚合和过滤：

| 分类 | 包含的域 | 举例 |
|------|---------|------|
| Battle | BAT, TAC, TER, RCT | 战斗开始、移动完成、陷阱触发 |
| Ability | ABL, SPR, TRG | 技能激活、法术施放、条件触发 |
| Effect | EFF, TAG, MOD, AGG | 效果施加、标签变更、属性聚合 |
| Content | QST, PRG, INV, ECO, CRF, FAC, PRY, CNR, NAR, SUM | 任务、经验、背包、交易、声望 |
| Infra | CNT, SAV, RPL | 存档、回放、内容加载 |

每个 LogCode 可以通过 `.category()` 查询自己属于哪个分类。

### 3.3 CorrelationId — 日志关联链路

想象你在调试一场战斗，日志里有几百行。怎么知道哪些日志属于「第 3 回合 第 5 次行动」？

CorrelationId 就是为此设计的。层级关系：

```
Battle(42)
  └── Turn { battle_id: 42, round: 1, turn_index: 3 }
       └── Action { turn_id: ..., sequence: 5 }
```

三种级别：
- **BattleId** = `u64`，整场战斗一个 ID
- **TurnId** = `(battle_id, round, turn_index)`，一轮中的一次回合
- **ActionId** = `(turn_id, sequence)`，一次具体行动

通过 `enum CorrelationId` 你可以用统一的类型表示三者。

### 3.4 DiagnosticContext — 诊断上下文

一个 Builder 模式的结构体，用来携带日志的附加上下文：

```rust
let ctx = DiagnosticContext::default()
    .with_correlation(CorrelationId::Battle(42))
    .with_entity(entity)
    .with_tag("combat");

// Display 输出: [Battle(42)] Entity(3)
```

目前这个结构体已经定义好了，但在实际 Observer 中还没有被广泛使用——当前 Observer 选择直接把字段传给 `info!()` 的结构化参数。

---

## 4. 第二层（L2）：基础设施实现层 — 日志的「记者」

在 `src/infra/logging/` 下，是真正的日志运行时实现。

### 4.1 LoggingPlugin

入口文件 `plugin.rs`。内容极其简单——它只做一件事：**注册 Observer**。

```rust
impl Plugin for LoggingPlugin {
    fn build(&self, app: &mut App) {
        // battle / turn / spell
        app.add_observer(battle_logger::on_battle_started)
            .add_observer(battle_logger::on_battle_ended);
        app.add_observer(turn_logger::on_turn_started)
            .add_observer(turn_logger::on_turn_ended);
        app.add_observer(spell_logger::on_spell_cast_result);

        // ability (4 events)
        app.add_observer(ability_logger::on_ability_activated)
            .add_observer(ability_logger::on_ability_completed)
            .add_observer(ability_logger::on_ability_cancelled)
            .add_observer(ability_logger::on_ability_cooldown_started);

        // effect (4 events)
        app.add_observer(effect_logger::on_effect_applied)
            .add_observer(effect_logger::on_effect_removed)
            .add_observer(effect_logger::on_effect_ticked)
            .add_observer(effect_logger::on_effect_immunity);

        // quest (5 events)
        app.add_observer(quest_logger::on_quest_accepted)
            .add_observer(quest_logger::on_objective_completed)
            .add_observer(quest_logger::on_quest_turned_in)
            .add_observer(quest_logger::on_quest_failed)
            .add_observer(quest_logger::on_quest_progress_updated);

        // content (1 event)
        app.add_observer(content_logger::on_definition_reloaded);
    }
}
```

当前版本注册了 **7 个 Observer 模块、共 18 个事件监听器**。启动时会输出 `[LoggingPlugin] initialized (18 observers registered)`。

这个 Plugin 在 `app/app_plugin.rs` 的 Phase 8 被注册（Infrastructure 层）。

### 4.2 observers/ — 各领域的日志监听器

每个 Observer 文件对应一组领域事件。目前实现了 **7 个 Observer 模块、共 18 个事件监听器**：

**battle_logger.rs**
- 监听 `BattleStarted` → `BAT001`，目前不记录额外字段
- 监听 `BattleEnded` → `BAT002`，记录 `victory` 字段

**turn_logger.rs**
- 监听 `TurnStarted` → `BAT005`，记录 `unit` 字段
- 监听 `TurnEnded` → `BAT006`，记录 `unit` 字段

**spell_logger.rs**
- 监听 `SpellCastResult` → `SPR001`，记录 `caster`、`spell_id`、`result` 字段

**ability_logger.rs**（新增）
- 监听 `AbilityActivated` → `ABL001`，记录 `entity`、`spec_id`、`context_desc`
- 监听 `AbilityCompleted` → `ABL002`，记录 `entity`、`spec_id`、`result`
- 监听 `AbilityCancelled` → `ABL003`，记录 `entity`、`spec_id`、`reason`
- 监听 `AbilityCooldownStarted` → `ABL004`，记录 `entity`、`spec_id`、`duration`、`shared_group`

**effect_logger.rs**（新增）
- 监听 `EffectApplied` → `EFF001`，记录 `instance_id`、`def_id`、`target`、`duration_type`
- 监听 `EffectRemoved` → `EFF002`，记录 `instance_id`、`def_id`、`target`、`reason`
- 监听 `EffectTicked` → `EFF003`（使用 `debug!` 级别），记录 `instance_id`、`tick_number`
- 监听 `EffectImmunityTriggered` → `EFF004`（使用 `warn!` 级别），记录 `def_id`、`target`、`immune_tag`

**quest_logger.rs**（新增）
- 监听 `QuestAccepted` → `QST001`，记录 `entity`、`quest_id`
- 监听 `ObjectiveCompleted` → `QST002`，记录 `entity`、`quest_id`、`objective_id`
- 监听 `QuestTurnedIn` → `QST003`，记录 `entity`、`quest_id`
- 监听 `QuestFailed` → `QST004`（使用 `warn!` 级别），记录 `entity`、`quest_id`、`fail_reason`
- 监听 `QuestProgressUpdated` → `QST005`，记录 `entity`、`quest_id`、`old_progress`、`new_progress`、`target`

**content_logger.rs**（新增）
- 监听 `OnDefinitionReloaded` → `CNT001`，记录 `bucket_name`、`version`、`changed_ids` 数量

`observers/mod.rs` 现在导出 7 个模块：`ability_logger`、`battle_logger`、`content_logger`、`effect_logger`、`quest_logger`、`spell_logger`、`turn_logger`。

所有 Observer 都长一个模样：

```rust
pub(crate) fn on_xxx(trigger: On<SomeEvent>) {
    let event = trigger.event();
    info!(
        code = ?LogCode::XXX,
        event = "event_name",
        field1 = ?event.field1,
        field2 = ?event.field2,
        "human_readable_message"
    );
}
```

关键要求：
- `code` 字段必须传 LogCode
- `event` 字段必须和事件名一致
- 所有数据必须是结构化字段，不能用 `format!` 拼到消息字符串里

### 4.3 rate_limit/ — 日志风暴保护

Bevy ECS 很容易踩坑：一个系统遍历 1000 个实体，每个实体打一条 `warn!`。瞬间 1000 行日志。

解决方案是 `OnceGuard`（基于 `AtomicBool` 的一次性守卫）：

```rust
// 定义在 static 变量中
static GUARD: OnceGuard = OnceGuard::new();

// 使用宏：第一次输出，之后静默
warn_once!(GUARD, "实体 {} 缺少 Buff 组件", entity);
```

实现原理很简单——`AtomicBool` 加 `compare_exchange`，第一次调用改成 `true` 并返回 `true`，后续返回 `false`。

配套提供了两个宏：`warn_once!` 和 `error_once!`。

### 4.4 sinks/ — 日志输出后端（预留）

目前是一个空模块。日志输出的实际后端由 Bevy 的 `DefaultPlugins` 中的 `LogPlugin` 处理（默认输出到 stderr）。未来可以在这里实现文件输出、遥测上报等功能。

### 4.5 旁支：Pipeline ExecutionLog

还有一个独立于 tracing 的「日志」系统：`PipelineContext.execution_log`（定义在 `src/core/capabilities/runtime/pipeline/foundation/types.rs`）。

这是一个 `Vec<ExecutionLogEntry>`，记录管线执行过程中每个阶段的步骤和结果（成功/失败/跳过）。它不输出到控制台，而是保存在内存中供调试和回放使用。

配套的 `ExecutionLogHook`（`src/infra/pipeline/hooks.rs`）会在每个步骤结束时以 `trace!` 级别输出到 tracing，用于开发调试。

### 4.6 旁支：UI CombatLog

还有一个单独的战斗日志 UI 系统（旧代码，在 `docs/ai_ignore_this_dir/` 中），它通过 `CombatLog` Resource 存储多色文本片段，驱动 UI 面板显示。这个属于**表现层日志**，不是**基础设施日志**，两者职责不同。

---

## 5. 数据流全景：一条日志的诞生

以「角色施法」为例，一条日志的完整旅程：

```
Step 1: 领域代码产生事件
─────────────────────────
  spell_system.rs:
    commands.trigger(SpellCastResult {
        caster: player_entity,
        spell_id: fireball_id,
        result: CastSuccess { target, damage },
    });

Step 2: Bevy 调度 Observer
─────────────────────────
  LoggingPlugin 注册了 spell_logger::on_spell_cast_result
  Bevy 在执行完 spell_system 后，自动调用匹配的 Observer

Step 3: Observer 生成结构化日志
─────────────────────────
  spell_logger.rs:
    info!(
        code = ?LogCode::SPR001,      // "SPR001"
        event = "spell_cast",          // 事件名
        caster = ?event.caster,        // Entity(7)
        spell_id = ?event.spell_id,    // SpellId("fireball")
        result = ?event.result,        // CastSuccess(...)
        "spell_cast"                   // 消息
    );

Step 4: tracing 框架处理
─────────────────────────
  tracing::info! → tracing-subscriber（Bevy 默认初始化）
      ↓
  格式化输出:
  2026-06-19T10:30:00.123456Z  INFO
    code=SPR001 event=spell_cast caster=Entity(7)
    spell_id=SpellId("fireball") result=CastSuccess
    : spell_cast

Step 5: 你看到了
─────────────────────────
  在终端里看到带颜色的日志行
```

这就是为什么领域层不需要写日志——它只需要发事件，剩下的由基础设施层自动完成。

---

## 6. 实战：如何添加一条新日志

假设你想为「铸造系统」添加日志，成品铸造成功时输出一条日志。

### 第一步：确认 LogCode

如果 CRF003（铸造完成）已经在 `log_code.rs` 中定义了，直接使用。如果没有，加一个：

```rust
// src/shared/diagnostics/log_code.rs
pub enum LogCode {
    // ...
    /// 铸造完成
    CRF003,
    // ...
}

impl LogCode {
    pub fn code(&self) -> &'static str {
        match self {
            // ...
            Self::CRF003 => "CRF003",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            // ...
            Self::CRF003 => "铸造完成",
        }
    }
}
```

### 第二步：确认 LogCategory 映射

在 `log_category.rs` 中确认 `CRF003` 被正确归类到 `Content`。LogCode 枚举的 category 映射是一个大的 match，把 `CRF003` 加到 `Content` 分支里。

### 第三步：创建或更新 Observer

如果已经有 `crafting_logger.rs`，在里面加函数。没有则新建：

```rust
// src/infra/logging/observers/crafting_logger.rs
use bevy::prelude::*;
use crate::core::domains::crafting::events::ItemCrafted;
use crate::shared::diagnostics::LogCode;

pub(crate) fn on_item_crafted(trigger: On<ItemCrafted>) {
    let event = trigger.event();
    info!(
        code = ?LogCode::CRF003,
        event = "item_crafted",
        crafter = ?event.crafter,
        item_id = ?event.item_id,
        quality = ?event.quality,
        "item_crafted"
    );
}
```

### 第四步：在 observers/mod.rs 中注册 + 在 LoggingPlugin 中注册

```rust
// src/infra/logging/observers/mod.rs
pub(crate) mod crafting_logger;

// src/infra/logging/plugin.rs
app.add_observer(crafting_logger::on_item_crafted);
```

### 第五步：在领域代码中触发事件

```rust
// 铸造系统里
commands.trigger(ItemCrafted {
    crafter: player,
    item_id: sword_id,
    quality: Quality::Masterwork,
});
```

完成。领域代码只需发事件，日志自动产生。

### 检查清单

- [ ] LogCode 在 `log_code.rs` 中已定义（code + description）
- [ ] LogCategory 映射已更新（如果需要新的分类规则）
- [ ] Observer 函数遵循 `info!(code = ?LogCode::XXX, event = "...", ...)` 格式
- [ ] Observer 在 `LoggingPlugin` 中注册
- [ ] 领域事件（`ItemCrafted`）包含日志所需的全部字段
- [ ] 没有在 Observer 中写任何业务逻辑

---

## 7. 现状盘点：已经做了什么，还缺什么

### 已实现

| 组件 | 状态 | 说明 |
|------|------|------|
| LogCode 枚举 | ✅ 完整 | 150+ 个编码覆盖 20 个域 |
| LogCategory 映射 | ✅ 完整 | 五大分类映射全部 LogCode |
| CorrelationId | ✅ 完整 | BattleId / TurnId / ActionId 三级链路 |
| DiagnosticContext | ✅ 已定义 | Builder 模式，带 Display 格式化 |
| OnceGuard + 宏 | ✅ 完整 | warn_once! / error_once! |
| battle_logger | ✅ 已注册 | 监听 BattleStarted / BattleEnded |
| turn_logger | ✅ 已注册 | 监听 TurnStarted / TurnEnded |
| spell_logger | ✅ 已注册 | 监听 SpellCastResult |
| ability_logger | ✅ 已注册 | 监听 4 个技能事件（ABL001-004） |
| effect_logger | ✅ 已注册 | 监听 4 个效果事件（EFF001-004） |
| quest_logger | ✅ 已注册 | 监听 5 个任务事件（QST001-005） |
| content_logger | ✅ 已注册 | 监听内容热重载（CNT001） |
| LoggingPlugin | ✅ 已注册 | 18 个 Observer 在 app_plugin.rs Phase 8 |
| ExecutionLogHook | ✅ 已实现 | Pipeline 执行日志 trace 输出 |

### 计划中但未实现

| 组件 | 状态 | 说明 |
|------|------|------|
| sinks (文件输出) | ❌ 未实现 | `src/infra/logging/sinks/` 仍是空模块 |
| DiagnosticContext 实际使用 | ❌ 未接入 | Observer 目前直接传结构化字段给 `info!()`，未使用 `DiagnosticContext` 类型 |
| tracing::instrument span | ❌ 未接入 | ADR-052 规划的链路追踪 |
| Metrics 消费端 | ❌ 未接入 | 领域事件 → 统计数据（counter/gauge）的链路 |
| domain 层违规清理 | ⚠️ 持续进行中 | 仍有 13 个 core 层文件（分布在 progression/inventory/faction/tactical/terrain/combat/narrative）存在直接 `tracing::info!`/`warn!`/`trace!` 调用，需要逐步改为事件驱动 |

---

## 8. 规则速查：该做什么和不该做什么

### ✅ 允许的

| 场景 | 做法 |
|------|------|
| 领域代码记录业务事件 | 发领域事件 `commands.trigger(...)` |
| 基础设施层的初始化/完成通知 | 直接 `tracing::info!("[XxxPlugin] initialized")` |
| 调试时观察变量 | 用 `tracing::trace!` 或 `tracing::debug!` |
| 循环中的异常检查 | 用 `warn_once!` / `error_once!` 宏 |
| 添加新日志 | 创建新 Observer，在 LoggingPlugin 注册 |

### ❌ 禁止的

| 场景 | 为什么禁止 |
|------|-----------|
| 领域层直接 `info!()` 写业务事件 | 违反宪法 §11.4，破坏四路消费架构 |
| 循环/迭代器内 `info!()` | 日志风暴，性能灾难 |
| Release 版本每帧 `info!()` | 日志爆炸，掩盖真正的问题 |
| 非结构化日志 `info!("角色{}发技能", x)` | 没法用 AI/工具搜索，违反结构化要求 |
| INFO 日志不带 `code = ?LogCode::XXX` | 无法编码化管理 |
| Observer 中写业务逻辑 | Observer 只负责输出日志，不负责判断 |
| ERROR 级别日志 | ERROR = Bug，出现即修，预算为零 |
| 使用 `println!` / `dbg!` | 必须用 tracing 统一管理 |

---

## 参考文档

| 文档 | 内容 |
|------|------|
| `docs/00-governance/ai-constitution-complete.md` §11 | 可观测性宪法（最高准则） |
| `docs/00-governance/coding-rules.md` §14 | 日志编码规范 |
| `docs/01-architecture/40-cross-cutting/ADR-052-logging-architecture.md` | 日志架构决策记录 |
| `src/shared/diagnostics/mod.rs` | 共享日志类型定义 |
| `src/infra/logging/plugin.rs` | LoggingPlugin 入口 |
| `src/infra/logging/observers/` | 各领域日志 Observer |
| `src/infra/logging/rate_limit/once_guard.rs` | 日志风暴保护实现 |
