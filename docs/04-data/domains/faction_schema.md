---
id: domains.faction.schema.v1
title: Faction Schema — 阵营关系数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-20
layer: definition, instance, persistence
replay-safe: true
---

# Faction Schema — 阵营关系数据架构

> **领域归属**: Domains — 叙事/社交层 | **依赖 Schema**: Event | **定义依据**: `docs/02-domain/domains/faction_domain.md`

---

## 1. Schema Design

### 1.1 FactionDef（Definition 层）

```rust
/// 阵营/势力的静态定义。运行时只读。
struct FactionDef {
    /// 阵营唯一标识（前缀: `fct_`）
    id: FactionId,

    /// 阵营名称本地化 Key
    name_key: LocalizationKey,

    /// 阵营描述本地化 Key
    description_key: LocalizationKey,

    /// 阵营类型
    faction_type: FactionType,

    /// 默认态度——当未定义特定关系时使用的基线态度
    default_attitude: FactionAttitude,

    /// 是否为关键/剧情阵营（禁止声望降至敌对阈值以下）
    is_critical: bool,
}

enum FactionType {
    Player,       // 玩家方
    Enemy,        // 敌方势力
    Neutral,      // 中立势力
    Quest,        // 任务相关特定势力
    Temporary,    // 临时同盟（剧情中短暂结盟）
}

/// 默认态度——当无特定关系定义时的基线态度
///
/// 来源：Content 层设计（`docs/03-content/definitions/vocabulary/faction-def.md`）
/// Schema 原 `base_relations` 违反 L0 同层引用规则，已移除。
enum FactionAttitude {
    Friendly,     // 默认友好（玩家阵营之间）
    Neutral,      // 默认中立（大多数中立 NPC）
    Hostile,      // 默认敌对（怪物阵营）
}
```

### 1.2 FactionRelation（L3 Gameplay 层）

```rust
/// 两个阵营之间的固有关系。运行时罕见变更，修改需要外交事件。
///
/// 注意：此枚举不再嵌入 FactionDef（L0），因为 L0 禁止同层引用。
/// FactionRelation 仅在 L3 Gameplay 层的 FactionRelationshipMatrix 中使用。
/// FactionDef 通过 `default_attitude`（FactionAttitude）声明基线态度，
/// 具体阵营间关系矩阵由 L3 层定义。
/// 来源：`docs/03-content/definitions/vocabulary/faction-def.md` §6
enum FactionRelation {
    Ally,     // 盟友 — 默认友好
    Neutral,  // 中立 — 无特殊关系
    Hostile,  // 敌对 — 默认敌对
    War,      // 战争 — 全面战争，无差别攻击
}
```

### 1.3 FactionMembership（Instance 层）

```rust
/// 角色（Entity）所属的阵营列表。一个角色可以属于多个阵营。
struct FactionMembership {
    /// 所属阵营 ID 列表
    factions: Vec<FactionId>,

    /// 主要阵营（用于 UI 显示、默认关系判定）
    primary_faction: FactionId,
}
```

### 1.4 Reputation（Instance 层/Persistence 层）

```rust
/// 角色在某个阵营中的声望值。
/// 取值 [-100, +100]，持久化保存。
struct Reputation {
    /// 目标阵营
    faction_id: FactionId,

    /// 当前声望值
    value: i32,

    /// 变更历史（用于调试和回放校验，最多保留最近 N 条）
    change_log: Vec<ReputationChange>,
}

struct ReputationChange {
    delta: i32,
    reason: ReputationReason,
    timestamp: GameTime,
}

enum ReputationReason {
    KillMember,         // 击杀敌对阵营成员
    QuestComplete,      // 完成任务
    DialogueChoice,     // 对话选择
    Theft,              // 偷窃行为
    Gift,               // 赠送礼物
    Betrayal,           // 背叛行为
    StoryEvent,         // 剧情事件
}

/// 声望等级分段映射（概念枚举，等级由数值范围决定）
enum ReputationLevel {
    Hated,    // -100 ~ -51: 主动攻击，不交易，对话不可用
    Hostile,  // -50  ~ -11: 可攻击，交易 ×2，对话受限
    Neutral,  // -10  ~ +9:  不主动攻击，标准交易，标准对话
    Friendly, // +10  ~ +49: 不攻击，交易 -10%，额外对话
    Honored,  // +50  ~ +89: 不攻击，交易 -20%，特殊任务
    Revered,  // +90  ~ +100: 不攻击，交易 -30%，专属装备/任务
}
```

### 1.5 RelationshipState（Instance 层 — 瞬时计算）

```rust
/// 角色与某阵营/角色的综合关系状态。瞬时计算，不持久化。
struct RelationshipState {
    /// 基础阵营关系
    base_relation: FactionRelation,

    /// 声望修正后的等级
    reputation_level: Option<ReputationLevel>,

    /// 最终综合状态
    final_state: RelationOutcome,
}

enum RelationOutcome {
    Ally,        // 盟友（FactionRelation=Ally 且声望未降至敌对）
    Friendly,    // 友好（FactionRelation=Neutral/Friendly 且声望>=Friendly）
    Neutral,     // 中立（无明显倾向）
    Hostile,     // 敌对（FactionRelation=Hostile 或声望<=Hostile）
    War,         // 战争（FactionRelation=War）
}
```

### 1.6 FactionState（Persistence 层）

```rust
/// 阵营关系持久化状态。
struct FactionState {
    /// 所有角色的声望数据
    reputations: Vec<(EntityId, Reputation)>,

    /// 阵营间关系变更记录（如果有运行时外交事件）
    relation_overrides: Vec<(FactionId, FactionId, FactionRelation)>,
}
```

---

## 2. Layer Summary

| Layer | Structures | 说明 |
|-------|-----------|------|
| **Definition** | `FactionDef`, `FactionAttitude`, `ReputationLevel` | 阵营定义、默认态度、声望等级为静态配置 |
| **Spec** | — | Faction 无 Spec 层；声望阈值映射为纯规则（代码） |
| **Instance** | `FactionMembership`, `Reputation`, `RelationshipState` | 角色归属与声望运行时数据；RelationshipState 为瞬时计算 |
| **Persistence** | `FactionState` | 声望值和运行时关系变更持久化 |

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → EventSchema | 声望变更发布 ReputationChanged 事件 |
| ← NarrativeSchema | 对话分支消费声望数据做条件过滤 |
| ← EconomySchema | 价格折扣消费声望等级 |
| ← CombatSchema | 敌对判定消费 RelationshipState |
| ← QuestSchema | 任务条件可能依赖声望阈值 |

---

## 4. Replay & Save

### Replay

- 声望变更通过 `ReputationChange` 录制（原因 + delta），回放时重算
- 外交事件（FactionRelation 变更）作为独立 Command 录制
- 确定性：声望值 clamp 到 [-100, +100] 确保边界一致

### Save

- `FactionState` 持久化所有实体的声望数据
- relation_overrides 只保存运行时发生变更的关系，基线态度（default_attitude）从 FactionDef 读取
- change_log 可选持久化（调试用），生产环境可限制最近 10 条或完全丢弃

---

## 5. Validation Rules

| 规则 | 说明 | 违反处理 |
|------|------|----------|
| 声望范围 | Reputation.value 必须在 [-100, 100] | clamp 到边界，记录警告 |
| 等级连续性 | 声望等级不可跳过中间等级（如 Neutral → Honored） | 运行时断言 |
| 默认态度有效 | FactionDef.default_attitude 必须是 FactionAttitude 的合法变体 | Schema 校验拒绝 |
| 关键角色保护 | is_critical=true 的阵营声望不能降至 Hostile 以下 | 运行时断言 |
| 变更有因 | ReputationChange 必须携带 reason | Schema 校验拒绝 |

---

## 6. Constitution Check

- ✅ **Data Law 001 (Def-Instance分离)**: FactionDef 为纯 Definition，Reputation 为 Instance，FactionState 为 Persistence
- ✅ **Data Law 002 (Rule-Content分离)**: 声望阈值映射（-100~-51 → Hated 等）属于代码规则，不嵌入配置
- ✅ **Data Law 003 (配置只引用ID)**: FactionMembership 和 Reputation 引用 FactionId
- ✅ **Data Law 010 (Replay优先)**: 声望变更由有因事件驱动，回放时逐条重放 ReputationChange
- ✅ **Data Law 011 (Schema版本化)**: FactionState 携带版本号，change_log 字段可演化为可选
- ✅ **Data Law 012 (域间禁止直接数据引用)**: Faction 通过 Event 对外发布声望/关系变更，消费方通过 Event 订阅
