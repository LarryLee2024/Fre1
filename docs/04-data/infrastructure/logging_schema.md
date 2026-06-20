---
id: infrastructure.logging.schema.v1
title: Logging Schema — 日志系统数据架构
status: stable
owner: data-architect
created: 2026-06-19
updated: 2026-06-19
layer: infrastructure
---

# Logging Schema — 日志系统数据架构

> **领域归属**: Infrastructure — C3 Runtime | **依赖 Schema**: 全部 Schema | **定义依据**: `docs/00-governance/Fre项目架构设计.md`, Data Law 010

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `LogEntry` | Infrastructure | 单条日志记录 |
| `LogCode` | Infrastructure | 日志类型编码（按域分组） |
| `LogCategory` | Infrastructure | 日志分类（5 大类） |
| `DiagnosticContext` | Infrastructure | 诊断上下文（关联 ID） |
| `CorrelationId` | Infrastructure | 关联标识（BattleId/TurnId/ActionId） |

---

## 2. Problem

日志系统需要解决：
- 战斗/技能/效果等核心流程的确定性记录
- 跨域事件的关联追踪（一次战斗 → 多个回合 → 多次伤害 → 多个效果）
- 诊断与调试时的快速定位
- Replay 系统的日志辅助（日志 ≠ 回放，日志是补充信息）

---

## 3. Schema Design

### 3.1 LogCode 枚举（按域分组）

```rust
/// 日志类型编码，按域分组，三位数字递增。
/// 格式: {域前缀}{3位数字}
enum LogCode {
    // ─── BAT — Combat（战斗）───
    BAT001, // battle_started          战斗开始
    BAT002, // battle_ended            战斗结束
    BAT003, // round_started           回合开始
    BAT004, // round_ended             回合结束
    BAT005, // turn_begin              单位回合开始
    BAT006, // turn_end                单位回合结束
    BAT007, // damage_dealt            伤害结算完成
    BAT008, // unit_died               单位死亡
    BAT009, // initiative_rolled       先攻检定完成
    BAT010, // victory_condition_met   胜负条件满足

    // ─── TAC — Tactical（战术/网格）───
    TAC001, // unit_moved              单位移动完成
    TAC002, // flanking_detected       夹击判定完成
    TAC003, // backstab_detected       背刺判定完成
    TAC004, // cover_evaluated         掩体判定完成
    TAC005, // position_changed        单位位置变更

    // ─── TER — Terrain（地形）───
    TER001, // tile_entered            单位进入格子
    TER002, // surface_changed         格子表面变化
    TER003, // hazard_triggered        陷阱触发
    TER004, // terrain_effect_applied  地形效果施加

    // ─── ABL — Ability（技能）───
    ABL001, // ability_activated       技能成功激活
    ABL002, // ability_completed       技能执行完毕
    ABL003, // ability_cancelled       技能被取消/打断
    ABL004, // ability_cooldown_start  冷却开始

    // ─── EFF — Effect（效果）───
    EFF001, // effect_applied          效果成功施加
    EFF002, // effect_removed          效果移除
    EFF003, // effect_ticked           周期效果 Tick
    EFF004, // effect_immunity         效果因免疫被阻止
    EFF005, // execution_completed     执行计算完成
    EFF006, // execution_failed        执行计算失败
    EFF007, // custom_execution_registered 自定义执行注册
    EFF008, // stack_overflow          堆叠达到上限触发溢出

    // ─── TAG — Tag（标签）───
    TAG001, // tag_added               标签授予实体
    TAG002, // tag_removed             标签从实体移除
    TAG003, // tag_hierarchy_changed   标签层级变更
    TAG004, // tag_query_evaluated     标签查询评估完成

    // ─── MOD — Modifier（修改器）───
    MOD001, // modifier_applied        修改器注册到容器
    MOD002, // modifier_removed        修改器从容器移除
    MOD003, // modifier_suppressed     修改器被高优先级抑制
    MOD004, // modifier_stale          检测到过期修改器

    // ─── AGG — Aggregator（聚合计算）───
    AGG001, // aggregation_complete    属性聚合计算完成
    AGG002, // aggregate_dirty         属性被标记需要重算
    AGG003, // snapshot_created        快照拍摄完成
    AGG004, // pipeline_cycle_detected 检测到聚合闭环

    // ─── TRG — Trigger（触发器）───
    TRG001, // trigger_fired           触发条件满足
    TRG002, // trigger_registered      触发器注册
    TRG003, // trigger_removed         触发器移除
    TRG004, // trigger_suppressed      触发器因频率限制被抑制

    // ─── SPR — Spell（法术）───
    SPR001, // spell_cast              法术施放完成
    SPR002, // spell_slot_changed      法术位数量变化
    SPR003, // concentration_broken    专注打断
    SPR004, // save_result             豁免检定结果

    // ─── RCT — Reaction（反应/援护）───
    RCT001, // reaction_triggered      反应满足触发条件
    RCT002, // reaction_executed       反应执行完毕
    RCT003, // reaction_declined       单位选择不使用反应
    RCT004, // opportunity_attack      机会攻击执行完毕
    RCT005, // counterspell_executed   法术反制执行完毕

    // ─── QST — Quest（任务）───
    QST001, // quest_accepted          任务被接受
    QST002, // objective_completed     单个目标完成
    QST003, // quest_turned_in         任务交付完成
    QST004, // quest_failed            任务失败
    QST005, // quest_progress_updated  任务进度变化

    // ─── PRG — Progression（成长养成）───
    PRG001, // experience_gained       角色获得经验
    PRG002, // level_up                角色升级
    PRG003, // talent_unlocked         天赋解锁
    PRG004, // subclass_chosen         子职选择
    PRG005, // asi_completed           属性提升完成
    PRG006, // class_gained            获得新职业等级

    // ─── INV — Inventory（背包/物品）───
    INV001, // item_acquired           物品进入背包
    INV002, // item_used               消耗品使用完成
    INV003, // equipment_changed       装备穿戴/卸下
    INV004, // item_removed            物品从背包移除
    INV005, // loot_generated          战利品生成

    // ─── ECO — Economy（经济/交易）───
    ECO001, // transaction_completed   交易完成
    ECO002, // price_changed           商店价格变化
    ECO003, // currency_changed        角色货币变化

    // ─── CRF — Crafting（制作）───
    CRF001, // recipe_learned          配方习得
    CRF002, // crafting_started        制作开始
    CRF003, // crafting_completed      制作完成
    CRF004, // crafting_failed         制作失败

    // ─── FAC — Faction（阵营）───
    FAC001, // reputation_changed      角色声望变化
    FAC002, // faction_relation_changed 阵营关系变化
    FAC003, // reputation_level_up     声望等级提升
    FAC004, // relationship_evaluated  关系判定完成

    // ─── PRY — Party（队伍）───
    PRY001, // member_joined           新成员加入队伍
    PRY002, // member_removed          成员离开队伍
    PRY003, // member_swapped          战斗中换人
    PRY004, // bond_activated          羁绊激活
    PRY005, // bond_deactivated        羁绊解除

    // ─── CNR — CampRest（营地休息）───
    CNR001, // short_rest_completed    短休完成
    CNR002, // long_rest_started       长休开始
    CNR003, // long_rest_completed     长休完成
    CNR004, // long_rest_interrupted   长休被中断
    CNR005, // camp_event_triggered    营地事件触发

    // ─── NAR — Narrative（叙事）───
    NAR001, // dialogue_started        对话开始
    NAR002, // choice_made             玩家选择分支
    NAR003, // story_flag_set          故事标记设置
    NAR004, // cutscene_started        过场动画开始
    NAR005, // cutscene_ended          过场动画结束

    // ─── SUM — Summon（召唤）───
    SUM001, // summon_created          召唤物被创建
    SUM002, // summon_expired          召唤物消失
    SUM003, // summon_command          召唤物接受指令
    SUM004, // summon_slot_changed     召唤占用变化

    // ─── CNT — Content（内容基础设施）───
    CNT001, // content_loaded          内容加载完成
    CNT002, // content_validation_failed 内容校验失败
    CNT003, // registry_registered     注册中心注册
    CNT004, // context_created         上下文构建完成
    CNT005, // context_consumed        上下文生命周期结束
    CNT006, // context_cycle_detected  溯源链检测到循环
    CNT007, // context_validation_failed 上下文构建校验失败
    CNT008, // spec_granted            Spec 授予到实体
    CNT009, // spec_removed            Spec 从实体移除
    CNT010, // spec_level_changed      Spec 等级变更
    CNT011, // spec_snapshot_taken     EffectSpec 快照
    CNT012, // cue_triggered           Cue 触发条件满足
    CNT013, // cue_suppressed          Cue 被禁用/跳过
    CNT014, // condition_passed        条件评估通过
    CNT015, // condition_failed        条件评估不通过
    CNT016, // immunity_triggered      免疫条件生效
    CNT017, // condition_subscribed    条件进入订阅状态
    CNT018, // target_selected         目标选择完成
    CNT019, // target_changed          目标选择被修改
    CNT020, // no_valid_target         无合法目标
    CNT021, // target_validated        单个目标通过校验
    CNT022, // execution_completed     执行计算完成
    CNT023, // execution_failed        执行计算失败
    CNT024, // custom_execution_registered 自定义执行注册
    CNT025, // event_published         事件被发布到 EventBus
    CNT026, // event_delivered         事件成功投递到订阅者
    CNT027, // event_delivery_failed   事件投递到订阅者失败
    CNT028, // event_cycle_detected    检测到事件循环触发

    // ─── SAV — Save（存档基础设施）───
    SAV001, // save_created            存档创建
    SAV002, // save_loaded             存档加载
    SAV003, // save_deleted            存档删除

    // ─── RPL — Replay（回放基础设施）───
    RPL001, // replay_started          回放开始
    RPL002, // replay_frame_recorded   回放帧录制
    RPL003, // replay_checksum_mismatch 回放校验不一致
}
```

### 3.1.1 LogCode 扩展方法（事件名自动派生）

LogCode 是事件唯一标识，所有关联属性应从 LogCode 本身派生，不需要外部维护映射表：

```rust
impl LogCode {
    /// 返回机器可读的事件名（snake_case），用于 `event` 结构化字段。
    /// 例如：PRG002 → "level_up", BAT001 → "battle_started"
    /// 这是 event 字段的 Single Source of Truth。
    pub fn event_name(&self) -> &'static str { ... }

    /// 返回日志分类（按业务领域划分五大类）
    pub fn category(&self) -> LogCategory { ... }

    /// 返回日志 target（按 `domain.module.submodule` 层级格式）
    /// 例如：PRG002 → "domain.progression", BAT001 → "domain.combat"
    pub fn target(&self) -> &'static str { ... }
}
```

**设计原则**：代码中 `#[instrument(fields(event = "level_up"))]` 中的 `event` 字符串应与 `LogCode::PRG002.event_name()` 保持一致。后续通过 `telemetry::emit(LogCode::PRG002)` 自动派生，消除手动维护双份事件名的冗余。

### 3.2 LogCategory 分类

```rust
/// 日志分类，按业务领域划分五大类。
enum LogCategory {
    /// 战斗相关：战斗开始/结束、回合流转、伤害结算、击杀、反应触发
    Battle,

    /// 技能/效果相关：技能激活、效果施加/移除/堆叠、触发器、修改器
    Ability,

    /// 效果/标签/修改器相关：效果生命周期、标签变更、属性聚合
    Effect,

    /// 内容/数据相关：任务进度、经验获取、背包变化、交易、声望
    Content,

    /// 基础设施相关：存档、回放、内容加载、系统事件
    Infra,
}
```

**分类映射规则**：

| LogCode 前缀 | LogCategory | 说明 |
|---------------|-------------|------|
| BAT / TAC / TER / RCT | Battle | 战斗核心流程 |
| ABL / SPR / TRG | Ability | 技能激活与触发 |
| EFF / TAG / MOD / AGG | Effect | 效果/标签/修改器/属性聚合 |
| QST / PRG / INV / ECO / CRF / FAC / PRY / CNR / NAR / SUM | Content | 内容/数据 |
| CNT / SAV / RPL | Infra | 基础设施 |

### 3.3 DiagnosticContext 数据结构

```rust
/// 诊断上下文，用于关联同一战斗/回合/行动中的多条日志。
struct DiagnosticContext {
    /// 关联 ID（战斗/回合/行动）
    correlation: CorrelationId,

    /// 实体标识（哪个单位）
    entity_id: Option<EntityId>,

    /// 帧号（确定性时间点）
    frame_number: Option<u64>,

    /// 回合号
    turn_number: Option<u32>,

    /// 回合号
    round_number: Option<u32>,

    /// 额外标签（用于过滤/搜索）
    tags: Vec<String>,
}
```

### 3.4 CorrelationId 类型定义

```rust
/// 关联标识，用于串联一次完整战斗行为中的所有日志。
enum CorrelationId {
    /// 战斗级关联：同一场战斗的所有日志
    Battle(BattleId),

    /// 回合级关联：同一轮/回合的所有日志
    Turn(TurnId),

    /// 行动级关联：同一次行动（技能/攻击/物品使用）的所有日志
    Action(ActionId),
}

/// 战斗唯一标识（UUID 或自增 ID）
type BattleId = u64;

/// 回合标识：(BattleId, RoundNumber, TurnIndex)
#[derive(Clone, Copy)]
struct TurnId {
    battle_id: BattleId,
    round: u32,
    turn_index: u32,
}

/// 行动标识：(TurnId, ActionSequence)
#[derive(Clone, Copy)]
struct ActionId {
    turn_id: TurnId,
    sequence: u32,
}
```

### 3.5 LogEntry 数据结构

```rust
/// 单条日志记录。
struct LogEntry {
    /// 日志类型编码
    code: LogCode,

    /// 日志分类
    category: LogCategory,

    /// 诊断上下文（关联 ID + 实体 + 时间点）
    context: DiagnosticContext,

    /// 日志消息（人类可读描述）
    message: String,

    /// 结构化数据（用于 JSON 输出）
    data: HashMap<String, LogValue>,

    /// 日志级别
    level: LogLevel,
}

enum LogLevel {
    /// 调试信息（开发期）
    Debug,
    /// 一般信息（正常流程）
    Info,
    /// 警告（异常但可恢复）
    Warn,
    /// 错误（异常且影响流程）
    Error,
}

/// 日志值类型（支持多种数据类型）
enum LogValue {
    Bool(bool),
    I64(i64),
    F64(f64),
    String(String),
    EntityId(EntityId),
    Array(Vec<LogValue>),
}
```

---

## 4. Layer Analysis

| 数据结构 | Layer | 持久化 | 说明 |
|----------|-------|--------|------|
| `LogCode` | Infrastructure | 否（编译期枚举） | 日志类型编码 |
| `LogCategory` | Infrastructure | 否（编译期枚举） | 日志分类 |
| `DiagnosticContext` | Infrastructure | 否 | 诊断上下文 |
| `CorrelationId` | Infrastructure | 否 | 关联标识 |
| `LogEntry` | Infrastructure | 可选（文件/输出流） | 单条日志记录 |

---

## 5. Dependency Analysis

| 依赖方向 | 依赖 Schema | 说明 |
|----------|------------|------|
| 依赖 | → CombatSchema | 战斗级关联 ID |
| 依赖 | → TurnSchema | 回合级关联 ID |
| 依赖 | → ActionSchema | 行动级关联 ID |
| 被依赖 | ← 全部 Domain | 所有域通过日志记录关键事件 |

---

## 6. 日志输出格式规范

### 6.1 JSON 格式（结构化日志）

```json
{
  "code": "BAT007",
  "category": "Battle",
  "level": "Info",
  "message": "Damage dealt: 15 fire damage from Entity(3) to Entity(7)",
  "context": {
    "correlation": { "Battle": 42 },
    "entity_id": { "Some": 3 },
    "frame_number": { "Some": 1234 },
    "turn_number": { "Some": 2 },
    "round_number": { "Some": 1 },
    "tags": ["combat", "damage", "fire"]
  },
  "data": {
    "attacker": { "EntityId": 3 },
    "target": { "EntityId": 7 },
    "damage_type": { "String": "fire" },
    "raw_damage": { "I64": 20 },
    "final_damage": { "I64": 15 },
    "is_critical": { "Bool": false },
    "hit_result": { "String": "Hit" }
  }
}
```

### 6.2 人类可读格式（控制台/调试工具）

```
[2026-06-19T10:30:45.123Z] [INFO] [BAT007] Battle(42) Turn(1,2) Frame(1234)
  Damage dealt: 15 fire damage from Entity(3) to Entity(7)
  raw=20 final=15 critical=false result=Hit
```

格式说明：
```
[时间戳] [级别] [LogCode] [关联ID] [消息]
  [结构化数据（缩进显示）]
```

### 6.3 输出目标

| 输出目标 | 格式 | 用途 |
|----------|------|------|
| 控制台 | 人类可读 | 开发调试 |
| 日志文件 | JSON | 运行时诊断 |
| Replay 辅助 | JSON（仅关键日志） | 回放验证 |

---

## 7. Validation Rules

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| V1 | LogCode 唯一性 | 日志记录时 | 同一 LogCode 在同一 CorrelationId 下不重复（除非是周期性事件如 EffectTicked） |
| V2 | CorrelationId 非空 | 日志记录时 | 必须有有效的 CorrelationId（Battle/Turn/Action 之一） |
| V3 | Level 合理性 | 日志记录时 | Error 级别日志必须附带错误详情 |
| V4 | 数据完整性 | 日志输出时 | JSON 格式必须包含 code/category/context/message 四个必填字段 |
| V5 | 字段低基数 | Schema 设计时 | 所有结构化字段必须使用 ID 类型（entity_id、spec_id、item_id），禁止使用 context_desc 等自然语言文本字段。高基数字段会压垮日志聚合系统（Loki/Elasticsearch） |
| V6 | event 字段语言 | 日志记录时 | `event` 字段值必须使用英文（`"level_up"`、`"battle_started"`），结构化日志是机器消费的，禁止使用中文 |

---

## 8. 日志级别使用规范

| 级别 | 使用场景 | 示例 |
|------|----------|------|
| Debug | 开发调试信息（仅 dev-tools 模式输出） | 标签查询评估结果、修改器抑制详情 |
| Info | 正常业务流程记录 | 战斗开始/结束、技能激活、效果施加、伤害结算 |
| Warn | 异常但可恢复的情况 | 堆叠溢出、属性值被 clamp、效果移除失败 |
| Error | 异常且影响流程的情况 | 内容校验失败、回放校验不一致、注册中心注册失败 |

---

## 9. 字段语言与基数规范

### 9.1 语言规范

| 字段 | 语言要求 | 原因 |
|------|----------|------|
| `code` (LogCode) | N/A（枚举） | 纯编码，无语言问题 |
| `event` | **必须英文**（snake_case） | 机器消费（日志聚合/AI搜索），需统一可移植 |
| `message`（字符串消息） | 中文/英文皆可 | 人消费，以开发者阅读效率为准 |
| `LogCode::description()` | 中文/英文皆可 | 人消费，以开发者理解为准 |

### 9.2 基数规范

| 字段类型 | 允许 | 禁止 | 原因 |
|----------|------|------|------|
| ID 字段 | `entity_id`, `spec_id`, `item_id` | — | 低基数，适合聚合 |
| 枚举字段 | `event`, `damage_type` | — | 有限取值集合 |
| 自然语言 | — | `context_desc`, `detail` 等 | 高基数，压垮聚合存储 |

> **经验教训**：`context_desc` 等自由文本字段在 Loki/Elasticsearch 中会指数级增加 label 基数，导致查询性能骤降和存储爆炸。必须使用 ID + LocalizationKey 替代。

---

## 9. Constitution Check

| 宪法条款 | 合规 | 说明 |
|----------|------|------|
| Replay First (P0) | ✅ | 日志是 Replay 的补充信息，不替代 ReplayLog |
| Logic / Presentation Separation (P0) | ✅ | 日志只记录数据，不触发 UI 行为 |
| 确定性 | ✅ | 日志记录基于确定性事件，不引入随机源 |
| Domain 间无直接依赖 | ✅ | 日志是 Infrastructure 层，被所有域引用但不引用任何域 |

---

## 10. Future Extension

- **`telemetry::emit` 统一入口**：当前 Observer 存在三要素重复（`#[instrument]` + `metrics::record()` + `info!()`），target 需要在两处重复指定。后续引入 `telemetry::emit(LogCode::XXX, ...)` 统一封装日志 + 度量 + trace，消除重复模式。
- **日志聚合分析**：基于 LogCode 统计技能使用率、伤害分布、效果触发频率
- **战斗重放辅助**：将关键日志（BAT/ABL/EFF）与 ReplayFrame 关联
- **性能监控**：统计各 LogCode 的记录频率和延迟
- **告警系统**：Error 级别日志触发实时告警
