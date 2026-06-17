---
id: domains.camp_rest.schema.v1
title: CampRest Schema — 营地/休息数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: instance, persistence
replay-safe: false
---

# CampRest Schema — 营地/休息数据架构

> **领域归属**: Domains — 成长养成层 | **依赖 Schema**: Party, Event, Progression, Spell | **定义依据**: `docs/02-domain/domains/camp_rest_domain.md`

---

## 1. Schema Design

### 1.1 RestState（Instance 层）

```rust
/// 休息状态机。标记队伍当前的休息阶段。
struct RestState {
    /// 休息类型
    rest_type: Option<RestType>,

    /// 当前阶段
    phase: RestPhase,

    /// 长休中断累计时间（分钟）
    interrupt_duration: u32,

    /// 上次长休完成的时间戳（用于 24 小时限制检查）
    last_long_rest: Option<GameTime>,
}

enum RestType {
    ShortRest,
    LongRest,
}

enum RestPhase {
    Idle,
    Resting,        // 短休进行中 / 长休睡眠阶段
    LightActivity,  // 长休轻活动阶段
    Complete,       // 已完成
    Failed,         // 长休失败（中断超 1h）
}
```

### 1.2 HitDicePool（Instance 层/Persistence 层）

```rust
/// 生命骰池。短休时消耗以恢复 HP。
struct HitDicePool {
    /// 当前可用的生命骰数量
    current: u32,

    /// 最大生命骰数（等于角色等级）
    max: u32,

    /// 生命骰类型（按职业，如 d6/d8/d10/d12）
    dice_type: DiceType,
}

enum DiceType { D6, D8, D10, D12 }
```

### 1.3 CampNPC（Instance 层/Persistence 层）

```rust
/// 营地 NPC 状态。
struct CampNPC {
    /// NPC 实体 ID
    entity_id: EntityId,

    /// 当前是否在营地中
    is_at_camp: bool,

    /// 可用对话选项列表（由 Narrative 领域提供）
    available_dialogues: Vec<DialogueDefId>,
}
```

### 1.4 CampEventDef（Definition 层）

```rust
/// 营地事件模板定义。
struct CampEventDef {
    /// 营地事件唯一标识（前缀: `cmp_`）
    id: CampEventId,

    /// 事件标题本地化 Key
    title_key: LocalizationKey,

    /// 事件描述本地化 Key
    desc_key: LocalizationKey,

    /// 触发条件
    trigger_conditions: Vec<ConditionDefId>,

    /// 事件类型
    event_type: CampEventType,

    /// 事件优先级（剧情 > 角色 > 随机）
    priority: u32,

    /// 触发后的效果（ModifierDefId / 剧情推进标记）
    effects: CampEventEffects,
}

enum CampEventType {
    Story,        // 剧情推进
    Character,    // 角色个人事件
    Random,       // 随机遭遇
    Rest,         // 纯休息（无事件）
}

struct CampEventEffects {
    /// 应用的 Modifier
    modifiers: Vec<ModifierDefId>,

    /// 推进的剧情标记
    story_flags: Vec<String>,

    /// 声望变化
    reputation_changes: Vec<(FactionDefId, i32)>,
}
```

### 1.5 CampRestState（Persistence 层）

```rust
/// 营地/休息系统的持久化状态。
struct CampRestState {
    /// 生命骰池状态
    hit_dice: HitDicePool,

    /// 上次长休的 GameTime（用于 24 小时限制）
    last_long_rest: Option<GameTime>,

    /// 营地 NPC 状态（哪些 NPC 当前在营地）
    camp_npcs: Vec<CampNPC>,

    /// 已触发的营地事件记录（防止重复触发）
    triggered_events: Vec<CampEventId>,
}
```

---

## 2. Layer Summary

| Layer | Structures | 说明 |
|-------|-----------|------|
| **Definition** | `CampEventDef` | 营地事件模板为静态配置 |
| **Spec** | — | CampRest 无 Spec 层 |
| **Instance** | `RestState`, `HitDicePool`, `CampNPC` | 休息流程和生命骰的运行时状态 |
| **Persistence** | `CampRestState` | 生命骰、长休计时、营地状态持久化 |

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → PartySchema | 长休时可调整队伍配置 |
| → EventSchema | 休息事件发布（ShortRestCompleted, LongRestCompleted 等） |
| → ProgressionSchema | 生命骰恢复量依赖角色等级 |
| → SpellSchema | 长休恢复法术位 |
| → ConditionSchema | 营地事件触发条件 |
| ← CombatSchema | 战斗结束后可进入休息状态 |

---

## 4. Replay & Save

### Replay

- 标记 `replay-safe: false` — 休息是进程管理行为，不参与战斗回放

### Save

- `CampRestState` 持久化：生命骰数量、上次长休时间、营地 NPC 状态
- `RestState` 中的 `interrupt_duration` 和 `phase` 不持久化（瞬时状态，读档后从 Idle 开始）
- CampEventDef 从配置加载

---

## 5. Validation Rules

| 规则 | 说明 | 违反处理 |
|------|------|----------|
| 长休 24h 限制 | 两次长休间隔 >= 24 小时游戏内时间 | 长休请求失败 |
| 非战斗状态 | 休息只能在非战斗状态进行 | 休息请求失败 |
| 安全区域 | 长休需要安全区域（营地/避难所） | 长休请求失败 |
| 生命骰上限 | hit_dice.current <= hit_dice.max (= 角色等级) | 超过时 clamp |
| 生命骰恢复量 | 长休恢复后 hit_dice.current <= ceil(等级/2) | 超过时 clamp |

---

## 6. Constitution Check

- ✅ **Data Law 001 (Def-Instance分离)**: CampEventDef 为 Definition，RestState/HitDicePool 为 Instance
- ✅ **Data Law 002 (Rule-Content分离)**: 休息规则（24h 限制、中断 1h 规则）为代码逻辑
- ✅ **Data Law 003 (配置只引用ID)**: CampEventDef 引用 ConditionDefId/ModifierDefId
- ✅ **Data Law 011 (Schema版本化)**: CampRestState 携带版本号
- ✅ **Data Law 012 (域间禁止直接数据引用)**: CampRest 通过 Event 与 Spell/Combat/Party 通信
