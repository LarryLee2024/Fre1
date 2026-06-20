---
id: 08-knowledge.logging-overview
title: 日志系统深度解析 — 从宪法到代码
status: draft
owner: architect
created: 2026-06-19
updated: 2026-06-20（吸收 6 点最佳实践 + 实现 telemetry::emit）
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
│  ├── plugin.rs           ← LoggingPlugin (56 Observers) │
│  ├── observers/           ← 20 个日志监听器模块          │
│  ├── metrics/             ← 全局度量计数 + 定期汇总      │
│  ├── rate_limit/          ← 日志风暴保护                 │
│  └── sinks/               ← 文件写入 (JSON, 可轮转)     │
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
│  20 个 Observer 模块 × 共 56 个事件监听器                              │
│  (覆盖全部 15 个业务领域 + 内容基础设施)                                 │
│                                                                     │
│  同时初始化 MetricsCollector Resource + metrics_flush_system         │
│                                                                     │
│  ┌─ Observable（以 progression_logger 为例）────────────────────┐    │
│  │  #[tracing::instrument(skip_all, fields(code = ?PRG002))]   │    │
│  │  fn on_level_up(trigger: On<LevelUp>) {                     │    │
│  │      telemetry::emit(LogCode::PRG002);    ← 统一观测入口    │    │
│  │      let e = trigger.event();                               │    │
│  │      info!(                                                 │    │
│  │          code = ?LogCode::PRG002,   ← 编码                 │    │
│  │          event = "level_up",        ← 事件名               │    │
│  │          entity = ?e.entity,        ← 结构化字段           │    │
│  │          old = e.old_level,                                 │    │
│  │          new = e.new_level,                                 │    │
│  │          "level_up"                ← 人类可读消息          │    │
│  │      );                                                     │    │
│  │  }                                                           │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                                                     │
│  额外输出后端：                                                       │
│  ┌─ sinks/file_sink.rs ───────────────────────────────────────┐    │
│  │  FileSink: 可轮转的 JSON 文件日志输出器                      │    │
│  │  配置: dir/prefix/max_bytes/max_files/enabled               │    │
│  └──────────────────────────────────────────────────────────────┘    │
└───────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────── tracing + MetricsCollector + FileSink ────┐
│                                                  │
│  tracing::info! → tracing-subscriber              │
│                    ↓  控制台输出 (stderr)          │
│                                                  │
│  MetricsCollector → 每 60 帧 DEBUG 摘要           │
│                                                  │
│  FileSink → JSON 文件 (logs/game.jsonl)          │
└──────────────────────────────────────────────────┘
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

目前定义了 20 个域名、约 150 个编码。每个 LogCode 有四个核心方法：

```rust
LogCode::BAT001.code()          // 返回 "BAT001"
LogCode::BAT001.description()   // 返回 "战斗开始"
LogCode::BAT001.event_name()    // 返回 "battle_started" — 机器可读事件名（英文 snake_case）
LogCode::BAT001.target()        // 返回 "domain.combat" — 对应的 tracing target
```

其中 `event_name()` 和 `target()` 是新增方法，目的是让 LogCode 成为**单一事实源**：
- `event` 字符串从 LogCode 派生，不需要在 Observer 中手动写 `event = "level_up"`
- `target` 从 LogCode 域前缀映射为 `domain.xxx` 层级格式
- 后续通过 `telemetry::emit(LogCode::XXX)` 自动完成所有派生

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

入口文件 `plugin.rs`。它做两件事：**初始化 MetricsCollector** 和 **注册 56 个 Observer**。

```rust
impl Plugin for LoggingPlugin {
    fn build(&self, app: &mut App) {
        // ── 初始化度量收集器 ──
        app.init_resource::<MetricsCollector>();
        app.add_systems(Update, metrics::metrics_flush_system);

        // ── 注册日志 Observer（20 模块 × 56 事件）──
        // BAT  战斗 (2)   TAC  战术 (2)   TER  地形 (4)
        // SPR  法术 (1)   RCT  反应 (5)   ABL  技能 (4)
        // EFF  效果 (4)   QST  任务 (5)   PRG  成长 (6)
        // INV  背包 (5)   ECO  经济 (3)   CRF  制作 (2)
        // FAC  阵营 (4)   PRY  队伍 (5)   CNR  营地 (5)
        // NAR  叙事 (5)   SUM  召唤 (4)   CNT  内容 (1)

        app.add_observer(battle_logger::on_battle_started) ...
        // ... 完整列表见源文件
    }
}
```

当前版本注册了 **20 个 Observer 模块、共 56 个事件监听器**。启动时会输出 `[LoggingPlugin] initialized (Metrics + 56 observers)`。

这个 Plugin 在 `app/app_plugin.rs` 的 Phase 8 被注册（Infrastructure 层）。

### 4.2 observers/ — 各领域的日志监听器

目前实现了 **20 个 Observer 模块、共 56 个事件监听器**，覆盖全部 15 个业务领域 + 内容基础设施。按 LogCode 域前缀分组：

**BAT — Combat（战斗）：battle_logger.rs + turn_logger.rs**
- `BattleStarted` → `BAT001`，监听全局战斗开始
- `BattleEnded` → `BAT002`，记录 `victory`
- `TurnStarted` → `BAT005`，记录 `unit`
- `TurnEnded` → `BAT006`，记录 `unit`

**TAC — Tactical（战术）：tactical_logger.rs**
- `UnitMoved` → `TAC001`，记录 `entity`、`from`、`to`、`remaining_mp`
- `PositionChanged` → `TAC005`，记录 `entity`、`new_pos`

**TER — Terrain（地形）：terrain_logger.rs**
- `TileEntered` → `TER001`，记录 `entity`、`tile`
- `SurfaceChanged` → `TER002`，记录 `tile`、`old_surface`、`new_surface`
- `HazardTriggered` → `TER003`，记录 `entity`、`hazard`
- `TerrainEffectApplied` → `TER004`，记录 `entity`、`effect`

**SPR — Spell（法术）：spell_logger.rs**
- `SpellCastResult` → `SPR001`，记录 `caster`、`spell_id`、`result`

**RCT — Reaction（反应）：reaction_logger.rs**
- `ReactionTriggered` → `RCT001`，记录 `entity`、`reaction`
- `ReactionExecuted` → `RCT002`，记录 `entity`、`result`
- `ReactionDeclined` → `RCT003`，记录 `entity`、`reason`
- `OpportunityAttack` → `RCT004`，记录 `attacker`、`target`
- `Counterspell` → `RCT005`，记录 `caster`、`target_spell`

**ABL — Ability（技能）：ability_logger.rs**
- `AbilityActivated` → `ABL001`，记录 `entity`、`spec_id`
- `AbilityCompleted` → `ABL002`，记录 `entity`、`spec_id`、`result`
- `AbilityCancelled` → `ABL003`，记录 `entity`、`spec_id`、`reason`
- `AbilityCooldownStarted` → `ABL004`，记录 `entity`、`spec_id`、`duration`、`shared_group`

> ⚠️ **注意**：`ABL001` 原有一个 `context_desc` 字段，因高基数风险（自然语言描述会导致日志聚合系统 label 爆炸）已被移除。请改用 `spec_id` + `LocalizationKey` 的组合来提供上下文信息。

**EFF — Effect（效果）：effect_logger.rs**
- `EffectApplied` → `EFF001`，记录 `instance_id`、`def_id`、`target`
- `EffectRemoved` → `EFF002`，记录 `instance_id`、`def_id`、`reason`
- `EffectTicked` → `EFF003`（使用 `debug!`），记录 `instance_id`、`tick_number`
- `EffectImmunity` → `EFF004`（使用 `warn!`），记录 `def_id`、`immune_tag`

**QST — Quest（任务）：quest_logger.rs**
- `QuestAccepted` → `QST001`，记录 `entity`、`quest_id`
- `ObjectiveCompleted` → `QST002`，记录 `entity`、`quest_id`、`objective_id`
- `QuestTurnedIn` → `QST003`，记录 `entity`、`quest_id`
- `QuestFailed` → `QST004`（使用 `warn!`），记录 `entity`、`quest_id`、`fail_reason`
- `QuestProgressUpdated` → `QST005`，记录 `entity`、`quest_id`、`old`、`new`、`target`

**PRG — Progression（成长）：progression_logger.rs**
- `ExperienceGained` → `PRG001`，记录 `entity`、`amount`、`source`、`level`
- `LevelUp` → `PRG002`，记录 `entity`、`old`、`new`
- `TalentUnlocked` → `PRG003`，记录 `entity`、`talent_id`
- `SubclassChosen` → `PRG004`，记录 `entity`、`subclass_id`
- `ASICompleted` → `PRG005`，记录 `entity`、`level`、`choices`
- `ClassGained` → `PRG006`，记录 `entity`、`class_id`、`level`

**INV — Inventory（背包）：inventory_logger.rs**
- `ItemAcquired` → `INV001`，记录 `entity`、`item_id`、`quantity`
- `ItemUsed` → `INV002`，记录 `entity`、`item_id`、`target`
- `EquipmentChanged` → `INV003`，记录 `entity`、`slot`、`old_item`、`new_item`
- `ItemRemoved` → `INV004`，记录 `entity`、`item_id`、`reason`
- `LootGenerated` → `INV005`，记录 `entity`、`items`、`gold`

**ECO — Economy（经济）：economy_logger.rs**
- `TransactionCompleted` → `ECO001`，记录 `entity`、`item`、`price`
- `PriceChanged` → `ECO002`，记录 `shop`、`item`、`old`、`new`
- `CurrencyChanged` → `ECO003`，记录 `entity`、`amount`、`reason`

**CRF — Crafting（制作）：crafting_logger.rs**
- `ItemCrafted` → `CRF003`，记录 `crafter`、`recipe`、`quality`
- `CraftingFailed` → `CRF004`（使用 `warn!`），记录 `crafter`、`recipe`、`reason`

**FAC — Faction（阵营）：faction_logger.rs**
- `ReputationChanged` → `FAC001`，记录 `entity`、`faction`、`old`、`new`
- `FactionRelationChanged` → `FAC002`，记录 `faction_a`、`faction_b`、`new_relation`
- `ReputationLevelUp` → `FAC003`，记录 `entity`、`faction`、`level`
- `RelationshipEvaluated` → `FAC004`，记录 `entity_a`、`entity_b`、`result`

**PRY — Party（队伍）：party_logger.rs**
- `MemberJoined` → `PRY001`，记录 `party`、`member`
- `MemberRemoved` → `PRY002`，记录 `party`、`member`、`reason`
- `MemberSwapped` → `PRY003`，记录 `party`、`out`、`in`
- `BondActivated` → `PRY004`，记录 `member_a`、`member_b`
- `BondDeactivated` → `PRY005`，记录 `member_a`、`member_b`

**CNR — CampRest（营地）：camp_rest_logger.rs**
- `ShortRestCompleted` → `CNR001`，记录 `entity`、`hd_spent`
- `LongRestStarted` → `CNR002`，记录 `entity`
- `LongRestCompleted` → `CNR003`，记录 `entity`、`recovered_hp`
- `LongRestInterrupted` → `CNR004`（使用 `warn!`），记录 `entity`、`reason`
- `CampEventTriggered` → `CNR005`，记录 `entity`、`event`

**NAR — Narrative（叙事）：narrative_logger.rs**
- `DialogueStarted` → `NAR001`，记录 `entity`、`dialogue_id`
- `ChoiceMade` → `NAR002`，记录 `entity`、`choice_id`
- `StoryFlagSet` → `NAR003`，记录 `flag`、`value`
- `CutsceneStarted` → `NAR004`，记录 `cutscene_id`
- `CutsceneEnded` → `NAR005`，记录 `cutscene_id`

**SUM — Summon（召唤）：summon_logger.rs**
- `SummonCreated` → `SUM001`，记录 `owner`、`template`
- `SummonExpired` → `SUM002`，记录 `entity`、`reason`
- `SummonCommand` → `SUM003`，记录 `entity`、`command`
- `SummonSlotChanged` → `SUM004`，记录 `owner`、`used`、`max`

**CNT — Content（内容基础设施）：content_logger.rs**
- `OnDefinitionReloaded` → `CNT001`，记录 `bucket_name`、`version`、`changed_ids` 数量

Observer 的典型模板（以 progression_logger 为例）：

```rust
#[tracing::instrument(skip_all, target = "domain.progression", fields(
    code = ?LogCode::PRG002,
    event = "level_up",
))]
pub(crate) fn on_level_up(trigger: On<LevelUp>) {
    // 1. 统一可观测性入口（日志 + 度量 + trace）
    telemetry::emit(LogCode::PRG002);

    // 2. 取事件数据
    let event = trigger.event();

    // 3. 输出 — 只放变量字段，不放 span 已覆盖的不变量
    info!(
        target = "domain.progression",
        entity = ?event.entity,
        old = event.old_level,
        new = event.new_level,
        "角色升级",
    );
}
```

**为什么 `info!()` 里没有 `code` 和 `event`？** 因为它们已经在 `#[instrument]` 的 span 字段里了。Span 负责不变量（本次调用共用的固定值），Event 只放变量（本次调用独有的动态数据）。重复写它们不仅冗余，还会导致未来修改时只改了一处、另一处忘记改的维护问题。

> 注意：`#[instrument]` 的 `target` 只作用于 span 层。`info!()` 内部仍需显式传递 `target` 参数，因为 tracing 的 event 不会继承父 span 的 target。这是当前模式的已知冗余，后续通过 `telemetry::emit` 统一封装后可消除。

每个 Observer 包含三个要素：
1. **`#[tracing::instrument]`** — 自动生成 span，携带 `code` 和 `event` 等不变字段
2. **`telemetry::emit(LogCode::XXX)`** — 统一可观测性入口（当前包装 metrics::record，未来扩展 trace/audit）
3. **`info!()` / `warn!()` / `debug!()`** — 输出结构化日志（只放动态变量字段）

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

### 4.4 sinks/ — 日志输出后端（文件写入）

现已实现 `file_sink.rs`，提供可轮转的 JSON 文件日志输出器：

```rust
let config = FileSinkConfig {
    dir: PathBuf::from("logs"),
    prefix: "game".to_string(),
    max_bytes: 10 * 1024 * 1024,   // 10MB 后轮转
    max_files: 5,                    // 保留最多 5 个轮转文件
    enabled: cfg!(debug_assertions), // 默认仅 debug 模式启用
};
let sink = FileSink::new(config);
sink.write(r#"{"level":"INFO","code":"PRG002","event":"level_up"}"#);
```

目前 Observer 中未自动调用 `FileSink`（因为它需要接入 tracing-subscriber 的 Layer，而非从 Observer 手动调用）。配套的 `format_json()` 函数可以将结构化字段格式化为 JSON 字符串。控制台输出仍由 Bevy 的 `DefaultPlugins.LogPlugin` 处理。

### 4.5 metrics/ — 事件度量统计（新实现）

对应 ADR-052 中「领域事件 → 日志的四路消费」的 Metrics 消费端。**这是一个新的子模块，完整实现了基于 LogCode 的度量计数和定期汇总。**

**全局计数器**：使用 `LazyLock<Mutex<HashMap<LogCode, u64>>>`，Observer 中通过 `telemetry::emit(LogCode::XXX)` 调用，无需 World 访问。

**MetricsCollector Resource**：每帧从全局计数器 drain 增量，每 60 帧通过 `metrics_flush_system` 输出一次 DEBUG 摘要——按域前缀（`BAT`、`PRG` 等）聚合，显示事件类型数和总量。

```rust
// 每 60 帧控制台输出类似：
// DEBUG frame=120 delta_events=47 total_events=284
//   detail="BAT:3x12 ABL:2x8 EFF:4x15 QST:2x7 CRF:1x5"
```

这个设计保证了：即使每秒发生数千次效果 Tick，Metrics 也不会产生大量日志，只会每 60 帧汇总一行。

### 4.6 旁支：Pipeline ExecutionLog

还有一个独立于 tracing 的「日志」系统：`PipelineContext.execution_log`（定义在 `src/core/capabilities/runtime/pipeline/foundation/types.rs`）。

这是一个 `Vec<ExecutionLogEntry>`，记录管线执行过程中每个阶段的步骤和结果（成功/失败/跳过）。它不输出到控制台，而是保存在内存中供调试和回放使用。

配套的 `ExecutionLogHook`（`src/infra/pipeline/hooks.rs`）会在每个步骤结束时以 `trace!` 级别输出到 tracing，用于开发调试。

### 4.7 旁支：UI CombatLog

还有一个单独的战斗日志 UI 系统（旧代码，在 `docs/ai_ignore_this_dir/` 中），它通过 `CombatLog` Resource 存储多色文本片段，驱动 UI 面板显示。这个属于**表现层日志**，不是**基础设施日志**，两者职责不同。

---

## 5. 数据流全景：一条日志的诞生

以「角色升级」为例，一条日志的完整旅程（当前最新模式）：

```
Step 1: 领域代码产生事件
─────────────────────────
  progression_system.rs:
    commands.trigger(LevelUp {
        entity: player,
        old_level: 4,
        new_level: 5,
        class_id: ...,
        is_asi_level: false,
    });

Step 2: Bevy 调度 Observer
─────────────────────────
  LoggingPlugin 注册了 progression_logger::on_level_up
  Bevy 在触发 LevelUp 事件后自动调用该 Observer

Step 3: Observer 生成 span + 度量 + 结构化日志（2026-06 最新模式）
─────────────────────────
  progression_logger.rs:
    #[tracing::instrument(skip_all, target = "domain.progression", fields(
        code = ?LogCode::PRG002,
        event = "level_up",
    ))]
    fn on_level_up(trigger: On<LevelUp>) {
        telemetry::emit(LogCode::PRG002);           // ← 统一入口（metrics + trace）
        let e = trigger.event();
        info!(                                      // ← 只放变量
            target = "domain.progression",
            entity = ?e.entity,
            old = e.old_level,
            new = e.new_level,
            "角色升级",
        );
    }

Step 4: 三路输出
─────────────────────────
  ┌─ tracing::info! → tracing-subscriber
  │     ↓
  │   控制台标准错误输出:
  │   2026-06-19T10:30:00.123456Z  INFO
  │     code=PRG002 event=level_up entity=Entity(3)
  │     old=4 new=5: level_up
  │
  ├─ #[instrument] span → 自动嵌套追踪
  │     ↓
  │   当有父 span（如战斗 span）时自动形成调用链:
  │   battle → turn → ability → level_up
  │
  └─ telemetry::emit(PRG002) → MetricsCollector
        ↓
      每 60 帧聚合输出:
      DEBUG frame=120 delta_events=47 detail="PRG:3x12 ..."

Step 5: 你看到了
─────────────────────────
  控制台有带颜色的日志行，必要时可以在 logs/game.jsonl 找到 JSON 格式的持久化文件。
```

这就是为什么领域层不需要写日志——它只需要发事件，剩下的由基础设施层自动完成三路输出（tracing + span 链路 + 度量统计）。

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
use crate::infra::logging::metrics;
use crate::shared::diagnostics::LogCode;

// span 放不变量（code + event），info! 只放变量
#[tracing::instrument(skip_all, target = "domain.crafting", fields(
    code = ?LogCode::CRF003,
    event = "item_crafted",
))]
pub(crate) fn on_item_crafted(trigger: On<ItemCrafted>) {
    telemetry::emit(LogCode::CRF003);
    let event = trigger.event();
    info!(
        target = "domain.crafting",
        crafter = ?event.crafter,
        item_id = ?event.item_id,
        quality = ?event.quality,
        "物品铸造完成",
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

- [ ] LogCode 在 `log_code.rs` 中已定义（code + description + event_name + target）
- [ ] LogCategory 映射已更新（如果需要新的分类规则）
- [ ] Observer 函数标注了 `#[tracing::instrument(skip_all, target="domain.xxx", fields(...))]`
- [ ] Observer 函数调用 `telemetry::emit(LogCode::XXX)`（统一入口，而非直接调用 metrics::record）
- [ ] `info!()` 中**没有**重复 `code`/`event` 字段（它们已在 span 中）
- [ ] `info!()` 中只放变量字段（entity, old, new, amount 等动态数据）
- [ ] `event` 字段值是英文（`"item_crafted"`），消息可用中文（`"物品铸造完成"`）
- [ ] 所有结构化字段都是 ID 类型（`entity_id`、`spec_id`），没有 `context_desc` 等自然语言文本
- [ ] Observer 在 `observers/mod.rs` 中注册 (`pub(crate) mod xxx_logger`)
- [ ] Observer 在 `LoggingPlugin` 中注册 (`app.add_observer(...)`)
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
| Observer 覆盖（20 模块） | ✅ 完整 | 15 个业务领域 + 内容基础设施全部覆盖 |
| LoggingPlugin | ✅ 已注册 | 56 个 Observer 在 app_plugin.rs Phase 8 |
| MetricsCollector | ✅ 已实现 | 全局计数器 + Bevy Resource + 每 60 帧 DEBUG 摘要 |
| sinks/FileSink | ✅ 已实现 | 可轮转的 JSON 文件日志输出器（logs/game.jsonl） |
| tracing::instrument span | ✅ 已接入 | 17/20 个 Observer 模块已标注 `#[instrument]` |
| ExecutionLogHook | ✅ 已实现 | Pipeline 执行日志 trace 输出 |

### 计划中但未实现

| 组件 | 状态 | 说明 |
|------|------|------|
| DiagnosticContext 实际使用 | ❌ 未接入 | 类型已定义但 Observer 仍直接传结构化字段给 `info!()`，未使用 `DiagnosticContext` 来包裹上下文 |
| 旧版 Observer 升级 | ✅ 已完成 | 全部 20 个 observer 已统一模式：`#[instrument]` + `telemetry::emit` + `info!()` |
| Metrics 持久化 | ❌ 未实现 | MetricsCollector 当前仅输出 DEBUG 日志，未持久化到文件或对接外部监控 |
| domain 层违规清理 | ⚠️ 持续进行中 | 仍有 13 个 core 层文件（progression/inventory/faction/tactical/terrain/combat/narrative）存在直接 `tracing::info!`/`warn!`/`trace!` 调用 |
| **span-field-only 模式改造** | ✅ 已完成 | 所有 Observer 已消除 `info!()` 中的 `code`/`event` 重复字段，改为 span 放不变量、event 放变量 |
| **`context_desc` 清理** | ✅ 已完成 | `ability_logger` 中的 `context_desc` 已移除，`ReactionType::log_name()` 已用于避免高基数 |
| **`telemetry::emit` 统一入口** | ✅ 已实现 | `telemetry::emit(LogCode)` 已创建为 metrics::record 的封装，所有 Observer 已迁移。后续可扩展为 `emit(LogCode, fields...)` 消除 target 两处重复 |

---

## 8. 本次同步吸收的 6 点最佳实践

以下 6 条最佳实践是 2026-06-20 跨文档同步后吸收的结论，涵盖 Observer 编写规范、LogCode 职责、字段管控和未来进化方向：

| # | 实践 | 核心理念 | 关键约束 |
|---|------|---------|---------|
| 1 | **Span-Event 字段分离** | `#[instrument(fields(...))]` 放不变量（`code`、`event`），`info!()` 放变量（`entity`、`amount`） | 禁止在 `info!()` 中重复 span 已有的字段 |
| 2 | **event 值用英文** | `event = "level_up"` 而非 `event = "升级"`；结构化日志是给机器消费的 | 英文小写 + 下划线，禁止中文 |
| 3 | **LogCode 作为 SSoT** | `LogCode::XXX.event_name()` 和 `LogCode::XXX.target()` 派生所有元数据，Observer 不硬编码 | Observer 只引用 `LogCode::XXX`，不写字面量 `"level_up"` |
| 4 | **target 分层格式** | `layer.domain`（如 `domain.progression`、`infra.logging`）；Layer 取 `app/core/infra/content` 缩写 | I/O / 渲染 / AI / 持久化用 `infra`，领域逻辑用 `domain` |
| 5 | **禁止高基数字段** | 自然语言描述（`context_desc`）导致日志聚合系统 label 爆炸 | 用 `spec_id` + `LocalizationKey` 替代，字段值必须可枚举 |
| 6 | **`telemetry::emit` 统一入口** | 用 `telemetry::emit(LogCode)` 封装观测入口，Observer 不再直接调用 `metrics::record` | 已实现基础版，未来可扩展为 `emit(LogCode, fields...)` |

这些实践已在 `docs/01-architecture/40-cross-cutting/ADR-052.md`、`logging_schema.md` 和宪法 §11 中落笔，这里用通俗语言归纳，方便日常速查。

---

## 9. 规则速查：该做什么和不该做什么

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
| 在 `info!()` 中重复 span 的 `code`/`event` 字段 | 冗余维护，改一处忘另一处 |
| `event` 字段值用中文（如 `"技能激活"`） | 结构化日志是机器消费的，必须英文 |
| 使用 `context_desc` 等自然语言文本作为结构化字段 | 高基数，压垮日志聚合系统 |

---

## 参考文档

| 文档 | 内容 |
|------|------|
| `docs/00-governance/ai-constitution-complete.md` §11.1-11.6 | 可观测性宪法（最高准则，含 Observer 实现规范） |
| `docs/00-governance/coding-rules.md` §14 | 日志编码规范 |
| `docs/01-architecture/40-cross-cutting/ADR-052-logging-architecture.md` | 日志架构决策记录 |
| `src/shared/diagnostics/mod.rs` | 共享日志类型定义 |
| `src/infra/logging/plugin.rs` | LoggingPlugin 入口（56 Observers + Metrics） |
| `src/infra/logging/observers/` | 20 个领域日志 Observer 模块 |
| `src/infra/logging/metrics/mod.rs` | MetricsCollector 度量统计实现 |
| `src/infra/logging/sinks/file_sink.rs` | FileSink 文件日志输出器 |
| `src/infra/logging/rate_limit/once_guard.rs` | 日志风暴保护实现 |
