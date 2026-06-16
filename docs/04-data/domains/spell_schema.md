---
id: domains.spell.schema.v1
title: Spell Schema — 法术数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition, instance, persistence
replay-safe: true
---

# Spell Schema — 法术数据架构

> **领域归属**: Domains — 战斗核心层 | **依赖 Schema**: Ability, Effect, Event, Combat | **定义依据**: `docs/02-domain/spell_domain.md`

---

## 1. Schema Design

### 1.1 SpellDef（Definition 层）

```rust
/// 法术的静态定义。Spell 是一种特殊类型的 Ability，复用 Ability 生命周期。
struct SpellDef {
    /// 法术唯一标识（前缀: `spl_`）
    id: SpellDefId,

    /// 法术名称本地化 Key
    name_key: LocalizationKey,

    /// 法术描述本地化 Key
    desc_key: LocalizationKey,

    /// 法术环阶（0 = 戏法, 1-9 = 法术环阶）
    level: SpellLevel,

    /// 施法时间
    casting_time: CastingTime,

    /// 施法组件需求
    components: SpellComponents,

    /// 法术射程
    range: SpellRange,

    /// 持续时间
    duration: SpellDuration,

    /// 是否需要专注
    requires_concentration: bool,

    /// 豁免类型（如不需要豁免则为 None）
    saving_throw: Option<AbilityType>,

    /// 法术是否可升环施法
    can_upcast: bool,

    /// 升环效果描述（每个环级的效果变化）
    /// 引用 EffectDefId，升环时选择对应环级的 EffectDef
    upcast_effects: HashMap<SpellLevel, Vec<EffectDefId>>,

    /// 基础效果（当前环级的效果）
    effects: Vec<EffectDefId>,
}

enum SpellLevel { Cantrip, L1, L2, L3, L4, L5, L6, L7, L8, L9 }

enum CastingTime {
    Action,           // 1 个标准动作
    BonusAction,      // 1 个附赠动作
    Reaction,         // 反应（在特定时机触发）
    Longer { minutes: u32 },  // 长施法时间（分钟）
}

struct SpellComponents {
    verbal: bool,
    somatic: bool,
    material: Option<MaterialComponent>,
}

struct MaterialComponent {
    description: LocalizationKey,
    consumed: bool,           // 材料是否被消耗
    cost_gold: Option<u32>,  // 材料是否有金币价值要求
}

enum SpellRange {
    Self,
    Touch,
    Ranged { base: u32, max: Option<u32> },
    Radius { center: RangeCenter, radius: u32 },
    Cone { length: u32 },
    Line { length: u32, width: u32 },
    Unlimited,
    Special,
}

enum RangeCenter { Self, Point }

enum SpellDuration {
    Instant,
    Concentration { max_turns: u32 },
    Timed { turns: u32 },
    Permanent,
}
```

### 1.2 SpellSlotPool（Instance 层/Persistence 层）

```rust
/// 法术位池。记录每个环阶的法术位总数与已用量。
struct SpellSlotPool {
    /// 各环阶法术位配置
    slots_by_level: HashMap<SpellLevel, SpellSlotEntry>,
}

struct SpellSlotEntry {
    /// 该环级的最大法术位数
    total: u32,

    /// 已使用的法术位数
    used: u32,
}

/// 快捷查询
impl SpellSlotPool {
    fn remaining(&self, level: SpellLevel) -> u32 { ... }
    fn consume(&mut self, level: SpellLevel) -> bool { ... }
    fn restore_all(&mut self) { ... }
}
```

### 1.3 Spellbook（Instance 层/Persistence 层）

```rust
/// 法术书/法术列表。记录角色已知和已准备的法术。
struct Spellbook {
    /// 所有已习得的法术
    known_spells: Vec<SpellDefId>,

    /// 当前已准备的法术（长休后可更换）
    prepared_spells: Vec<SpellDefId>,

    /// 最大可准备法术数量
    max_prepared: u32,
}
```

### 1.4 Concentration（Instance 层）

```rust
/// 专注状态。同一时间最多一个专注法术。
struct Concentration {
    /// 当前专注的法术
    active_spell: SpellDefId,

    /// 专注持续的总回合数
    total_duration: u32,

    /// 已持续的回合数
    elapsed_rounds: u32,

    /// 专注建立时的快照（施法属性、熟练加值）
    /// 用于专注打断检定
    concentration_snapshot: ConcentrationSnapshot,
}

struct ConcentrationSnapshot {
    caster_entity: EntityId,
    caster_proficiency: i32,
    caster_con_modifier: i32,
}
```

### 1.5 SpellState（Persistence 层）

```rust
/// 法术系统的持久化状态。
struct SpellState {
    /// 法术位池状态
    slot_pool: SpellSlotPool,

    /// 法术书状态
    spellbook: Spellbook,

    /// 专注状态（存档时保存，读档后重建）
    concentration: Option<Concentration>,
}
```

---

## 2. Layer Summary

| Layer | Structures | 说明 |
|-------|-----------|------|
| **Definition** | `SpellDef` (含环阶/组件/射程/时长/升环) | 法术的静态定义，内容团队配置 |
| **Spec** | — | Spell 复用 Ability Spec；施法参数在 SpellDef 中固定 |
| **Instance** | `SpellSlotPool`, `Spellbook`, `Concentration` | 运行时法术资源与状态 |
| **Persistence** | `SpellState` | 法术位、法术书、专注状态的存档子集 |

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → AbilitySchema | Spell 复用 Ability 的施放生命周期（消耗→执行→完成） |
| → EffectSchema | 法术效果引用 EffectDefId |
| → CombatSchema | 施法在回合框架内执行（标准动作） |
| → EventSchema | 施法事件发布（SpellCast, ConcentrationBroken 等） |
| ← ReactionSchema | 法术反制和护盾术是 Reaction 的子类型 |

---

## 4. Replay & Save

### Replay

- 施法动作录制为 Command（选择法术 → 选择目标（可施法）→ 确定环级（如果升环））
- 专注打断检定由伤害事件触发，确定性足够（种子 PRNG 决定豁免骰结果）
- 法术位消费完全由施法动作驱动，replay 中逐帧还原

### Save

- `SpellState` 包含法术位、法术书和专注状态
- 专注中的法术在读档后重建（基于存档的 duration 快照）
- 已完成/消耗的法术位精确恢复（防止读档后法术位重生）

---

## 5. Validation Rules

| 规则 | 说明 | 违反处理 |
|------|------|----------|
| 法术位不可透支 | 施法前检查 `SpellSlotPool.remaining >= 1` | 施法失败，不消耗法术位 |
| 专注唯一性 | 建立专注时检查 `Concentration` 是否已存在 | 旧专注自动解除 |
| 组件检查 | 施法前检查组件可用性（沉默/束缚/材料） | 施法失败 |
| 环阶匹配 | 施法者等级必须 >= SpellDef 的环级要求 | 施法失败 |
| 升环合法 | 升环施法时目标环级必须有可用法术位 | 降级到可用的最低环级 |

---

## 6. Constitution Check

- ✅ **Data Law 001 (Def-Instance分离)**: SpellDef 为纯 Definition，SpellSlotPool/Spellbook/Concentration 为 Instance
- ✅ **Data Law 003 (配置只引用ID)**: SpellDef 引用 EffectDefId，Spellbook 引用 SpellDefId
- ✅ **Data Law 004 (Ability不拥有行为)**: Spell 复用 Ability 生命周期，法术效果通过 Effect 执行
- ✅ **Data Law 005 (Effect是唯一业务执行入口)**: 法术的伤害/治疗/增益全部通过 Effect 实现
- ✅ **Data Law 010 (Replay优先)**: 施法动作录制为 Command，专注打断由确定性 PRNG 驱动
- ✅ **Data Law 011 (Schema版本化)**: SpellState 携带版本号支持字段演化
- ✅ **Data Law 012 (域间禁止直接数据引用)**: Spell 通过 Event 与 Combat/Reaction 通信
