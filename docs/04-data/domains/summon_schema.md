---
id: domains.summon.schema.v1
title: Summon Schema — 召唤数据架构
status: stable
owner: data-architect
created: 2026-06-16
updated: 2026-06-16
layer: definition, instance
replay-safe: true
---

# Summon Schema — 召唤数据架构

> **领域归属**: Domains — 经济系统层 | **依赖 Schema**: Effect, Combat, Tactical, Event | **定义依据**: `docs/02-domain/summon_domain.md`

---

## 1. Schema Design

### 1.1 SummonTemplateDef（Definition 层）

```rust
/// 召唤物模板定义。运行时只读。
struct SummonTemplateDef {
    /// 召唤物模板唯一标识（前缀: `sum_`）
    id: SummonTemplateId,

    /// 召唤物名称本地化 Key
    name_key: LocalizationKey,

    /// 基础属性模板（引用 AttributeDefId → 基础值）
    base_attributes: HashMap<AttributeDefId, f32>,

    /// 拥有的 Tag 列表
    tags: Vec<TagDefId>,

    /// 拥有的 Ability 列表
    abilities: Vec<AbilityDefId>,

    /// 默认 Modifier
    modifiers: Vec<ModifierDefId>,

    /// 占用的格子数
    grid_size: GridSize,

    /// AI 模式
    default_ai_mode: SummonAIMode,

    /// 召唤消耗的资源
    summon_cost: SummonCost,
}

enum GridSize { Small, Medium, Large, Huge }

enum SummonAIMode {
    /// 自主行动（AI 自动决策）
    Autonomous,
    /// 跟随召唤者行动
    Follow,
    /// 守卫当前位置
    Guard,
    /// 防御模式（不主动攻击）
    Defensive,
}

struct SummonCost {
    /// 消耗的 AbilityDefId（召唤作为一个特殊的 Ability）
    ability_id: Option<AbilityDefId>,
    /// 消耗的法术位环级（法术召唤）
    spell_level: Option<SpellLevel>,
    /// 是否消耗专注
    requires_concentration: bool,
}
```

### 1.2 SummonBond（Instance 层）

```rust
/// 召唤物与召唤者的绑定关系。
struct SummonBond {
    /// 召唤者实体 ID
    caster: EntityId,

    /// 使用的模板 ID
    template_id: SummonTemplateId,

    /// 当前的 AI 模式
    ai_mode: SummonAIMode,

    /// 召唤时间（创建时的 GameTime）
    summoned_at: GameTime,
}

/// 召唤者的召唤槽位管理
struct SummonSlotManager {
    /// 当前活跃的召唤物列表
    active_summons: Vec<EntityId>,

    /// 最大可同时存在的召唤物数量
    max_slots: u32,
}
```

### 1.3 SummonDuration（Instance 层 — 通过 Effect 管理）

```rust
/// 召唤持续时间——通过 Effect(Duration) 管理。
/// 召唤物的存续时间不由独立的 SummonDuration 组件管理，
/// 而是通过 Effect 领域的能力系统表达。
///
/// 此结构仅为存档时的快照，用于读档后重建。
struct SummonDurationSnapshot {
    /// 持续时间类型
    duration_type: SummonDurationType,

    /// 剩余回合数（仅对 Timed/Concentration 类型有效）
    remaining_turns: Option<u32>,
}

enum SummonDurationType {
    /// 专注维持（由 Spell 领域的 Concentration 管理）
    Concentration,
    /// 固定回合数
    Timed { max_turns: u32 },
    /// 永久（直到被击杀或主动解散）
    Permanent,
}
```

---

## 2. Layer Summary

| Layer | Structures | 说明 |
|-------|-----------|------|
| **Definition** | `SummonTemplateDef` | 召唤物模板的静态配置 |
| **Spec** | — | Summon 无 Spec 层；召唤能力通过 Ability Spec 表达 |
| **Instance** | `SummonBond`, `SummonSlotManager`, `SummonDurationSnapshot` | 召唤运行时绑定和槽位管理；持续时间由 Effect 管理 |
| **Persistence** | — | Summon 状态随 CombatSnapshot 持久化（召唤物作为 CombatParticipant） |

---

## 3. Dependency Analysis

| 依赖 | 说明 |
|------|------|
| → EffectSchema | 召唤持续时间通过 Effect(Duration) 管理 |
| → CombatSchema | 召唤物是 CombatParticipant |
| → TacticalSchema | 召唤物占用网格位置 |
| → EventSchema | 召唤事件发布（SummonCreated, SummonExpired） |
| → SpellSchema | 专注类召唤依赖 Concentration 管理 |
| → AbilitySchema | 召唤能力通过 Ability 表达 |

---

## 4. Replay & Save

### Replay

- 召唤动作录制为 Command（召唤者执行召唤 Ability → 创建召唤物 Entity）
- 召唤物消失由 Effect(Duration) 到期/专注打断/被击杀驱动——均为确定性事件
- 标记 `replay-safe: true`

### Save

- 召唤物状态随战斗存档（CombatSnapshot）一起保存
- 召唤物作为 CombatParticipant 序列化，附带 SummonBond 信息
- SummonTemplateDef 从配置加载

---

## 5. Validation Rules

| 规则 | 说明 | 违反处理 |
|------|------|----------|
| 召唤者生死约束 | 召唤者死亡时所有召唤物立即消失 | UnitDied 事件触发级联移除 |
| 专注召唤唯一性 | 一个施法者只能维持一个专注召唤 | Spell Concentration 规则 |
| 模板一致性 | 召唤物必须基于 SummonTemplateDef 创建 | 创建时断言 |
| 占位不冲突 | 召唤位置必须可通行且无单位 | 创建失败 |
| 禁止嵌套召唤 | 召唤物不可再产生召唤物 | 运行时断言 |

---

## 6. Constitution Check

- ✅ **Data Law 001 (Def-Instance分离)**: SummonTemplateDef 为 Definition，SummonBond/SummonSlotManager 为 Instance
- ✅ **Data Law 005 (Effect是唯一业务执行入口)**: 召唤物持续时间通过 Effect(Duration) 管理
- ✅ **Data Law 010 (Replay优先)**: 召唤/消失由确定性事件驱动；召唤动作录制为 Command
- ✅ **Data Law 011 (Schema版本化)**: SummonDurationSnapshot 携带版本号
- ✅ **Data Law 012 (域间禁止直接数据引用)**: Summon 通过 Event 与 Combat/Tactical/Effect 通信
