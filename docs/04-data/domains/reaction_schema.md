---
id: domains.reaction.schema.v1
title: Reaction Schema — 反应/援护数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: instance
replay-safe: true
---

# Reaction Schema — 反应/援护数据架构

> **领域归属**: Domains — 战斗核心层 | **依赖 Schema**: Combat, Tactical, Event | **定义依据**: `docs/02-domain/reaction_domain.md`

---

## 1. Schema Design

### 1.1 ReactionState（Instance 层）

```rust
/// 单位的反应槽位状态。每回合重置。
struct ReactionState {
    /// 当前回合是否已使用反应
    used: bool,

    /// 允许的额外反应次数（特殊能力/专长提供，默认 0）
    extra_reactions: u32,

    /// 本回合已使用的额外反应次数
    extra_used: u32,
}

impl ReactionState {
    fn can_react(&self) -> bool {
        !self.used || self.extra_used < self.extra_reactions
    }
    fn consume(&mut self) { ... }
    fn reset(&mut self) { ... }
}
```

### 1.2 ReactionQueue（Instance 层 — 瞬时）

```rust
/// 反应队列。当多个反应同时触发时，按优先级排队执行。
/// 瞬时结构，在一帧内创建、消费、销毁。
struct ReactionQueue {
    /// 排队中的反应条目
    entries: Vec<ReactionEntry>,

    /// 当前正在处理的反应索引
    current_index: usize,
}

struct ReactionEntry {
    /// 触发者
    reactor: EntityId,

    /// 反应类型
    reaction_type: ReactionType,

    /// 触发事件上下文
    trigger: ReactionTrigger,

    /// 优先级（防御 > 进攻，同类型按先攻）
    priority: u32,

    /// 状态
    status: ReactionEntryStatus,
}

enum ReactionType {
    OpportunityAttack,
    Counterspell,
    Shield,
    Guardian,
    Special { custom_id: String },
}

enum ReactionEntryStatus {
    Pending,      // 等待决策
    Accepted,     // 玩家/AI 选择使用
    Declined,     // 玩家/AI 放弃使用
    Executed,     // 已执行
    Cancelled,    // 因前置反应影响而取消（如第一个机会攻击命中后后续取消）
}

enum ReactionTrigger {
    /// 单位离开威胁区
    LeaveThreatRange { mover: EntityId, from: GridPosition, to: GridPosition },
    /// 敌方施法
    EnemySpellCast { caster: EntityId, spell_id: SpellDefId },
    /// 被攻击命中前
    BeforeHit { attacker: EntityId, target: EntityId, attack_roll: i32 },
    /// 相邻友方被攻击
    AdjacentAllyHit { ally: EntityId, attacker: EntityId },
    /// 自定义触发
    Custom { event_type: String, data: Vec<u8> },
}
```

### 1.3 OpportunityAttackData（Instance 层 — 瞬时）

```rust
/// 机会攻击的触发与执行数据。瞬时结构。
struct OpportunityAttackData {
    /// 攻击者（威胁单位）
    attacker: EntityId,

    /// 目标（离开威胁区的单位）
    target: EntityId,

    /// 触发位置
    from_position: GridPosition,

    /// 攻击结果（由 Combat 领域填充）
    result: Option<AttackResult>,
}

enum AttackResult {
    Hit { damage: i32 },
    Miss,
    CriticalHit { damage: i32 },
}
```

### 1.4 CounterspellData（Instance 层 — 瞬时）

```rust
/// 法术反制的触发与判定数据。瞬时结构。
struct CounterspellData {
    /// 反制者
    counterer: EntityId,

    /// 被反制的法术
    target_spell: SpellDefId,

    /// 被反制法术的环级
    target_level: SpellLevel,

    /// 反制使用的环级
    counter_level: SpellLevel,

    /// 判定结果（自动成功 / 需要检定 / 失败）
    verdict: CounterspellVerdict,
}

enum CounterspellVerdict {
    AutoSuccess,
    CheckRequired { dc: u32, roll: Option<i32> },
    Failed,
}
```

### 1.5 GuardianData（Instance 层 — 瞬时）

```rust
/// 援护格挡的触发与执行数据。瞬时结构。
struct GuardianData {
    /// 援护者
    guardian: EntityId,

    /// 被援护的目标
    target: EntityId,

    /// 攻击者
    attacker: EntityId,

    /// 转移的伤害量
    transferred_damage: i32,

    /// 援护者位置的格子坐标（用于距离校验）
    guardian_position: GridPosition,
}
```

---

## 2. Layer Summary

| Layer | Structures | 说明 |
|-------|-----------|------|
| **Definition** | — | Reaction 无 Definition 层；反应类型由枚举定义，规则由代码实现 |
| **Spec** | — | Reaction 无 Spec 层；触发条件由代码检测，不通过配置 |
| **Instance** | `ReactionState` | 持久化：反应槽位状态（是否已使用） |
| **Instance (瞬时)** | `ReactionQueue`, `OpportunityAttackData`, `CounterspellData`, `GuardianData` | 瞬时：在执行一帧内创建消费，不跨帧持久化 |

> **说明**: Reaction 是纯运行时机制。反应类型（机会攻击/法术反制/护盾/援护）由代码定义，
> 特殊反应通过注册回调实现。没有 Definition 和 Spec 层的独立数据。

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → CombatSchema | 伤害结算复用 CombatIntent 入口 |
| → TacticalSchema | 机会攻击依赖威胁区判定、位置数据 |
| → EventSchema | 反应触发/执行事件发布 |
| → SpellSchema | 法术反制引用 SpellDefId |
| ← CombatSchema | 战斗流程预留反应插入点（伤害结算前、移动中） |

---

## 4. Replay & Save

### Replay

- 反应决策录制为 Command（"是否使用机会攻击？" → Yes/No）
- 瞬时的 ReactionQueue/OpportunityAttackData/CounterspellData 不持久化，回放时重新生成
- 反应槽位的消耗/重置完全由回合事件驱动（TurnBegin 时 reset = Available）

### Save

- Only `ReactionState.used` needs persistence (per-saved-entity, as part of a larger combat snapshot)
- All one-frame transient structures (ReactionQueue, opportunity attacks, counterspell data) are ephemeral

---

## 5. Validation Rules

| 规则 | 说明 | 违反处理 |
|------|------|----------|
| 单回合反应上限 | `ReactionState.can_react()` 为 false 时不触发 | 跳过反应触发检测 |
| 回合外触发 | 反应不在己方回合主动触发（除非法术声明可己方回合施放） | 过滤触发条件 |
| 机会攻击触发条件 | 仅主动离开威胁区触发（不包括强制移动/传送/推开） | 过滤非法触发源 |
| 援护距离 | 援护者必须在目标的相邻格 | 距离检查不通过则取消援护 |
| 法术反制环级 | 反制环级 < 目标环级时需施法属性检定 | 自动进入 CheckRequired 流程 |

---

## 6. Constitution Check

- ✅ **Data Law 001 (Def-Instance分离)**: Reaction 无 Def 层；ReactionState 为 Instance 层
- ✅ **Data Law 005 (Effect是唯一业务执行入口)**: 反应的效果（伤害/护盾 Buff）通过 Effect 领域执行
- ✅ **Data Law 010 (Replay优先)**: 反应决策录制为 Command，所有瞬时结构在回放时重建
- ✅ **Data Law 012 (域间禁止直接数据引用)**: Reaction 通过 Event 与 Combat/Spell/Tactical 通信
