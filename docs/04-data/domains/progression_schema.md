---
id: domains.progression.schema.v1
title: Progression Schema — 成长养成数据架构
status: draft
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition, instance, persistence
replay-safe: false
---

# Progression Schema — 成长养成数据架构

> **领域归属**: Domains — 成长养成层 | **依赖 Schema**: Attribute, Modifier, Event, Condition, Ability | **定义依据**: `docs/02-domain/progression_domain.md`

---

## 1. Schema Design

### 1.1 LevelProgression（Definition 层）

```rust
/// 等级成长配置表。定义各等级的经验阈值和熟练加值。
struct LevelProgression {
    /// 等级上限（D&D 5e = 20）
    max_level: u32,

    /// 各等级所需的累计经验值 [level_1_threshold, level_2_threshold, ...]
    /// 索引 = 等级-1，例如 exp_thresholds[0] = 1→2 所需经验
    exp_thresholds: Vec<u32>,

    /// 各等级的熟练加值 [1-4: +2, 5-8: +3, ...]
    proficiency_by_level: Vec<i32>,

    /// ASI 触发等级
    asi_levels: Vec<u32>,
}
```

### 1.2 Experience（Instance 层/Persistence 层）

```rust
/// 角色的经验值数据。
struct Experience {
    /// 当前总经验值
    total_xp: u64,

    /// 是否为满级（等级 >= 20）
    is_max_level: bool,
}

/// 经验值来源记录（用于回放校验和调试）
struct XpSource {
    amount: u32,
    reason: XpReason,
    timestamp: GameTime,
}

enum XpReason {
    Combat,         // 战斗胜利
    Quest,          // 任务完成
    Discovery,      // 探索发现
    Dialogue,       // 对话奖励
    Special,        // 特殊事件
}
```

### 1.3 LevelComponent（Instance 层）

```rust
/// 角色的等级数据。
struct LevelComponent {
    /// 当前总等级（1-20）
    total_level: u32,

    /// 各职业的等级分布（多职业）
    class_levels: Vec<ClassLevelEntry>,
}

struct ClassLevelEntry {
    /// 职业 ID
    class_id: ClassDefId,

    /// 在该职业上的等级
    level: u32,
}
```

### 1.4 TalentTree（Instance 层/Persistence 层）

```rust
/// 天赋树状态。记录天赋解锁情况。
struct TalentTree {
    /// 已解锁的天赋列表
    unlocked_talents: Vec<TalentDefId>,

    /// 可用的天赋点数（尚未分配的）
    available_points: u32,
}
```

### 1.5 SubclassChoice（Instance 层/Persistence 层）

```rust
/// 子职选择记录。
struct SubclassChoice {
    /// 职业 → 所选子职的映射
    choices: HashMap<ClassDefId, SubclassDefId>,
}
```

### 1.6 ASIState（Instance 层 — 瞬时）

```rust
/// ASI 选择状态。在升级到 ASI 等级时创建，选择完成后销毁。
struct ASIState {
    /// 触发 ASI 的等级
    level: u32,

    /// 已做的选择
    choices: Vec<ASIChoice>,

    /// 可选属性列表
    available_attributes: Vec<AttributeId>,
}

enum ASIChoice {
    /// 单一属性 +2
    SingleAttribute(AttributeId),
    /// 两项属性各 +1
    TwoAttributes(AttributeId, AttributeId),
    /// 选择一个专长
    Feat(FeatDefId),
}
```

### 1.7 ProgressionState（Persistence 层）

```rust
/// 成长系统的持久化状态。
struct ProgressionState {
    /// 经验值
    experience: Experience,

    /// 等级数据
    level: LevelComponent,

    /// 天赋解锁状态
    talents: TalentTree,

    /// 子职选择
    subclasses: SubclassChoice,
}
```

---

## 2. Layer Summary

| Layer | Structures | 说明 |
|-------|-----------|------|
| **Definition** | `LevelProgression` | 等级表、熟练加值、ASI 时机为静态配置 |
| **Spec** | — | Progression 无 Spec 层；等级规则由代码实现 |
| **Instance** | `Experience`, `LevelComponent`, `TalentTree`, `SubclassChoice`, `ASIState` | 成长运行时状态；ASIState 为瞬时 |
| **Persistence** | `ProgressionState` | 经验/等级/天赋/子职持久化 |

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → AttributeSchema | ASI 修改 BaseValue，触发 Aggregator 重算 |
| → ModifierSchema | 天赋/专长注册被动 Modifier |
| → EventSchema | 成长事件发布（LevelUp, ExperienceGained 等） |
| → ConditionSchema | 天赋前置条件/升级条件检查 |
| → AbilitySchema | 升级注册新 Ability |
| ← CombatSchema | 战斗结束发放经验 |
| ← QuestSchema | 任务完成发放经验 |

---

## 4. Replay & Save

### Replay

- 经验获取和升级 **不需要 replay 确定性** — 成长是玩家进程数据，不参与战斗回放
- 标记 `replay-safe: false` — 成长数据通过 Save/Load 恢复，不在 Replay 中重演

### Save

- `ProgressionState` 完整持久化（经验、等级、天赋、子职）
- LevelProgression 配置表从 Definition 层加载，不存入存档
- ASIState 不持久化（瞬时结构，ASI 选择完成后立即写入 Attribute）

---

## 5. Validation Rules

| 规则 | 说明 | 违反处理 |
|------|------|----------|
| 等级上限 | total_level <= max_level (20) | 经验继续累计但不再升级 |
| 经验只增不减 | XP 不可被任何系统消耗（除升级自动扣除） | Schema 校验拒绝 |
| 天赋前置链完整 | 解锁前检查等级/属性/前置天赋 | 解锁失败 |
| 子职唯一性 | 同一职业只能选择一个子职，不可更改 | 选择拒绝 |
| ASI 不可跳过 | 到达 ASI 等级必须分配属性提升或专长 | 阻塞其他流程 |

---

## 6. Constitution Check

- ✅ **Data Law 001 (Def-Instance分离)**: LevelProgression 为 Definition，Experience/Level 为 Instance
- ✅ **Data Law 002 (Rule-Content分离)**: 等级经验表为配置内容，升级规则为代码逻辑
- ✅ **Data Law 003 (配置只引用ID)**: TalentTree 引用 TalentDefId，SubclassChoice 引用 SubclassDefId
- ✅ **Data Law 005 (Effect是唯一业务执行入口)**: 天赋/专长的效果通过 Modifier 和 Ability 实现
- ✅ **Data Law 011 (Schema版本化)**: ProgressionState 携带版本号，exp_thresholds 表可扩展
- ✅ **Data Law 012 (域间禁止直接数据引用)**: Progression 通过 Event 与 Combat/Quest 通信
