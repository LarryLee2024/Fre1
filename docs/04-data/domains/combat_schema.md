---
id: domains.combat.schema.v1
title: Combat Schema — 战斗数据架构
status: draft
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: instance, persistence
replay-safe: true
---

# Combat Schema — 战斗数据架构

> **领域归属**: Domains — 战斗核心层 | **依赖 Schema**: Tactical, Terrain, Effect, Event, Faction, Execution | **定义依据**: `docs/02-domain/combat_domain.md`

---

## 1. Schema Design

### 1.1 CombatState（Instance 层）

```rust
/// 战斗宏观状态机。
struct CombatState {
    /// 当前战斗阶段
    phase: CombatPhase,

    /// 当前轮数（从 1 开始）
    round_number: u32,

    /// 战斗唯一标识
    combat_id: CombatId,
}

enum CombatPhase {
    Preparation,  // 战前准备（编队、先攻检定）
    InProgress,   // 战斗中（回合循环）
    Resolution,   // 战斗结算（经验、战利品）
    Ended,        // 战斗结束（不可逆）
}
```

### 1.2 TurnOrder（Instance 层）

```rust
/// 先攻排序队列。战斗开始后不变，除非有特殊能力显式声明重排。
struct TurnOrder {
    /// 按先攻值从高到低排列的参与者列表
    entries: Vec<InitiativeEntry>,

    /// 当前行动单位在 entries 中的索引
    current_index: usize,
}

struct InitiativeEntry {
    /// 参与者实体
    participant: EntityId,

    /// 先攻值
    initiative: InitiativeValue,

    /// 是否已在本轮行动过
    has_acted: bool,
}

struct InitiativeValue {
    /// 基础先攻值（d20 + 敏捷调整 + 其他加值）
    base: i32,

    /// 平局时敏捷属性高者优先，仍平则掷骰
    tiebreaker: i32,
}
```

### 1.3 CombatParticipant（Instance 层）

```rust
/// 战斗参与者标记与运行时数据。
struct CombatParticipant {
    /// 所属阵营
    faction_id: FactionDefId,

    /// 当前 HP
    hp: i32,

    /// 最大 HP
    max_hp: i32,

    /// 临时 HP
    temp_hp: i32,

    /// 护甲等级（AC）
    armor_class: i32,

    /// 是否存活
    is_alive: bool,
}
```

### 1.4 ActionPoints（Instance 层）

```rust
/// 单位在当前回合的行动资源。每轮重置。
struct ActionPoints {
    /// 标准动作
    standard_action: ActionState,

    /// 附赠动作
    bonus_action: ActionState,

    /// 反应槽（回合外使用，每轮最多 1 次）
    reaction: ReactionState,

    /// 行动力（移动距离）
    movement: MovementState,
}

enum ActionState {
    Available,    // 可用
    Used,         // 已使用
}

enum ReactionState {
    Available,    // 可用
    Used,         // 本轮回合外已使用
}
```

### 1.5 CombatIntent（Instance 层 — 瞬时攻击意图）

```rust
/// 攻击意图——伤害结算的唯一入口。
/// 瞬时结构，在结算流程中创建、传递、消费，不持久化。
struct CombatIntent {
    /// 攻击者
    attacker: EntityId,

    /// 目标
    target: EntityId,

    /// 攻击方式（引用 AbilityDefId）
    ability_id: Option<AbilityDefId>,

    /// 攻击骰结果
    attack_roll: i32,

    /// 是否为暴击（自然 20）
    is_critical: bool,

    /// 伤害骰结果
    damage_roll: i32,

    /// 上下文（地形、战术修正等）
    context: CombatContext,
}

struct CombatContext {
    flanking: FlankingState,
    cover: CoverState,
    highground: HighgroundState,
}
```

### 1.6 DamageResult（Instance 层 — 瞬时结算结果）

```rust
/// 伤害结算结果。瞬时结构，用于事件广播。
struct DamageResult {
    /// 原始伤害
    raw_damage: i32,

    /// 减伤/抗性减免后
    mitigated: i32,

    /// 最终实际伤害
    final_damage: i32,

    /// 命中结果
    hit_result: HitResult,

    /// 计算明细（调试/UI 显示）
    breakdown: Vec<DamageBreakdownItem>,
}

enum HitResult {
    Miss,
    Hit,
    CriticalHit,
    Immune,
    Intercepted,  // 被反应拦截
}
```

### 1.7 VictoryConditionDef（Definition 层）

```rust
/// 胜负判定条件定义。可配置，支持组合条件。
struct VictoryConditionDef {
    /// 胜利条件列表（全部满足即胜利）
    win_conditions: Vec<WinCondition>,

    /// 失败条件列表（任一满足即失败）
    lose_conditions: Vec<LoseCondition>,
}

enum WinCondition {
    EliminateAll { factions: Vec<FactionDefId> },
    SurviveRounds { count: u32 },
    ProtectTarget { target_id: EntityId },
    OccupyPoints { points: Vec<GridPosition>, duration: u32 },
    BossKill { boss_id: EntityId },
    Custom { condition_id: ConditionDefId },
}

enum LoseCondition {
    PartyWipe,
    TimerExpiry { max_rounds: u32 },
    TargetDied { target_id: EntityId },
    PointLost { points: Vec<GridPosition> },
}
```

### 1.8 CombatSnapshot（Persistence 层）

```rust
/// 战斗状态持久化快照。用于存档/读档。
struct CombatSnapshot {
    /// 战斗阶段
    phase: CombatPhase,

    /// 当前轮数
    round_number: u32,

    /// 先攻排序快照
    turn_order: Vec<InitiativeEntry>,

    /// 所有参与者的战斗状态
    participants: Vec<CombatParticipant>,

    /// 各单位行动资源状态
    action_states: Vec<(EntityId, ActionPoints)>,
}
```

---

## 2. Layer Summary

| Layer | Structures | 说明 |
|-------|-----------|------|
| **Definition** | `VictoryConditionDef` | 胜负条件为可配置的静态定义 |
| **Spec** | — | Combat 无 Spec 层；战斗行为通过 Ability/Effect Spec 表达 |
| **Instance** | `CombatState`, `TurnOrder`, `CombatParticipant`, `ActionPoints`, `CombatIntent`, `DamageResult` | 战斗流程的运行时状态；Intent 和 Result 为瞬时结构 |
| **Persistence** | `CombatSnapshot` | 战斗中存档/读档所需的战斗状态子集 |

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → TacticalSchema | 夹击、掩体、高地判定数据 |
| → TerrainSchema | 地形通行性、遮蔽度、地形效果 |
| → EffectSchema | 持续效果 Tick 推进 |
| → ExecutionSchema | 伤害/治疗数值计算 |
| → EventSchema | 战斗事件发布（CombatStarted, TurnBegin, DamageDealt 等） |
| → FactionSchema | 阵营敌对关系判定 |
| ← SpellSchema | 施法时机管理 |
| ← ReactionSchema | 回合外反应插入点 |

---

## 4. Replay & Save

### Replay

- 所有玩家输入和 AI 决策录制为 `Command`（移动、攻击、施法、结束回合）
- 先攻检定结果在战斗开始时录制，回放时使用相同的随机种子
- 伤害骰/攻击骰结果由确定性 PRNG 驱动
- `CombatIntent` 作为攻击的唯一入口确保回放时结算路径一致

### Save

- `CombatSnapshot` 保存战斗中存档所需的最小状态集
- 不持久化 `CombatIntent` 和 `DamageResult`（瞬时结构，读档后重新生成）
- 战斗结束后清除所有 Combat 相关组件（CombatParticipant, TurnOrder, ActionPoints）

---

## 5. Validation Rules

| 规则 | 说明 | 违反处理 |
|------|------|----------|
| CombatIntent 唯一入口 | 所有伤害必须通过 CombatIntent | 运行时断言失败 |
| 回合严格交替 | 单位必须按先攻顺序依次行动 | 运行时断言失败 |
| 一回合一个行动态单位 | 同一时刻最多一个单位处于行动中 | 运行时断言失败 |
| 先攻排序不变性 | 战斗开始后 TurnOrder 不修改（除非特殊能力声明） | 运行时断言失败 |
| 战斗结束不可逆 | CombatState 进入 Ended 后不可重新激活 | Schema 校验拒绝 |
| 伤害单次结算 | 同一 CombatIntent 只能结算一次 | 运行时断言失败 |

---

## 6. Constitution Check

- ✅ **Data Law 001 (Def-Instance分离)**: VictoryConditionDef 为 Definition，CombatState/TurnOrder 为 Instance，CombatSnapshot 为 Persistence
- ✅ **Data Law 004 (Ability不拥有行为)**: Combat 不拥有技能行为——通过 Ability 领域执行
- ✅ **Data Law 005 (Effect是唯一业务执行入口)**: 持续效果和地形效果通过 Effect 领域推进
- ✅ **Data Law 010 (Replay优先)**: CombatIntent 唯一攻击入口确保回放确定性；先攻/伤害骰由确定性 PRNG 驱动
- ✅ **Data Law 011 (Schema版本化)**: CombatSnapshot 携带版本号，支持字段演化
- ✅ **Data Law 012 (域间禁止直接数据引用)**: Combat 通过 Event 与 Spell/Tactical/Reaction 通信
