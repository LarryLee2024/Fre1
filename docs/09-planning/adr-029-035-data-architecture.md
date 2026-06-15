---
id: 09-planning.adr-029-035-data-architecture
title: ADR-029~035 数据架构设计
status: draft
owner: data-architect
created: 2026-06-15
updated: 2026-06-15
tags:
  - data
  - schema
  - registry
  - migration
---

# ADR-029~035 数据架构设计

设计者：@data-architect
基线：ADR-029~035 + Linglan 13 领域数据模型 + 现有数据架构规范
策略：**Zero compatibility, scorched-earth**

---

## 1. ID 系统设计

### 1.1 `define_id!` 宏接口

基于 `docs/01-architecture/ids-design.md` 的 Strong ID 模式，定义统一的 ID 生成宏：

```rust
// src/shared/ids/define_id.rs

/// 生成强类型 ID 的宏。
///
/// 为每个领域生成：
/// - 一个 newtype struct（如 `AbilityId`）
/// - `Display`、`FromStr`、`Deref<Target=str>`、`Serialize`、`Deserialize`、`Reflect`、`Hash`、`Eq`、`Ord`、`Clone`、`Copy`、`Debug`
///
/// # 参数
/// - `$vis`：可见性（pub / pub(crate)）
/// - `$name`：类型名（如 `AbilityId`）
/// - `$prefix`：Display 前缀（如 "ability" → Display 为 "ability:s_1001"）
/// - `$reflect`：是否生成 Bevy Reflect 实现（true/false）
macro_rules! define_id {
    (
        $(#[$outer:meta])*
        $vis:vis $name:ident,
        prefix: $prefix:literal,
        reflect: $reflect:ident,
    ) => { ... };
}
```

**生成的 impl 签名**：

| Trait | 实现 |
|-------|------|
| `Display` | `write!(f, "{}:{}", prefix, self.0)` |
| `FromStr` | 拆分 `:` 前后，验证 prefix 匹配 |
| `Deref<Target=str>` | `&self.0` |
| `Serialize` / `Deserialize` | 序列化为完整字符串 `"prefix:id_value"` |
| `Reflect`（条件） | `#[derive(Reflect)]` + `#[reflect(Hash)]` |

### 1.2 全部 22+ ID 类型清单

按领域分组，每个 ID 使用 `define_id!` 生成：

| 领域 | ID 类型 | macro prefix | Reflect | 来源 ADR |
|------|---------|-------------|---------|----------|
| **Attribute** | `AttributeId` | `"attr"` | true | ADR-031 |
| **Tag** | `TagId` | `"tag"` | true | ADR-031 |
| **Modifier** | `ModifierId` | `"mod"` | true | ADR-032 |
| **Effect** | `EffectId` | `"eff"` | true | ADR-032 |
| **Stacking** | `StackingId` | `"stack"` | true | ADR-032 |
| **Execution** | `ExecutionId` | `"exec"` | true | ADR-032 |
| **Ability** | `AbilityId` | `"ability"` | true | ADR-033 |
| **Trigger** | `TriggerId` | `"trig"` | true | ADR-033 |
| **Targeting** | `TargetingId` | `"tgt"` | true | ADR-033 |
| **Cue** | `CueId` | `"cue"` | true | ADR-034 |
| **Character** | `CharacterId` | `"char"` | true | — |
| **Unit** | `UnitId` | `"unit"` | true | — |
| **Equipment** | `EquipmentId` | `"equip"` | true | — |
| **Item** | `ItemId` | `"item"` | true | — |
| **Terrain** | `TerrainId` | `"terrain"` | false | — |
| **Class** | `ClassId` | `"class"` | true | — |
| **Race** | `RaceId` | `"race"` | false | — |
| **Trait** | `TraitId` | `"trait"` | true | — |
| **AI Behavior** | `AiBehaviorId` | `"ai"` | false | — |
| **Campaign** | `CampaignId` | `"camp"` | false | — |
| **Stage** | `StageId` | `"stage"` | false | — |
| **Faction** | `FactionId` | `"faction"` | false | — |

**注意**：`BuffId` 和 `SkillId` **不需要**单独定义——两者分别被 Effect 和 Ability 替代。

### 1.3 目录结构

```
src/shared/ids/
├── mod.rs         # pub mod 导出 + 重新导出
├── define_id.rs   # define_id! 宏定义
├── attribute.rs   # define_id!(pub AttributeId, prefix: "attr", reflect: true)
├── tag.rs
├── modifier.rs
├── effect.rs
├── stacking.rs
├── execution.rs
├── ability.rs
├── trigger.rs
├── targeting.rs
├── cue.rs
├── character.rs
├── unit.rs
├── equipment.rs
├── item.rs
├── terrain.rs
├── class.rs
├── race.rs
├── trait_def.rs
├── ai_behavior.rs
├── campaign.rs
├── stage.rs
└── faction.rs
```

---

## 2. Registry 架构

### 2.1 RegistryLoader trait

```rust
/// 注册表加载器：从 RON 加载数据到 Registry。
///
/// 每个 Registry 实现一个 RegistryLoader，支持全量加载和增量热重载。
pub trait RegistryLoader: Send + Sync + 'static {
    /// 加载到的 Registry 类型
    type Registry: 'static;

    /// 定义类型（RON 反序列化的中间表示）
    type Def: DeserializeOwned + Send + Sync;

    /// 全量加载：从 RON 构建 Registry
    fn load(&self, def: Self::Def) -> Result<Self::Registry, RegistryLoadError>;

    /// 热重载：从新 RON 增量更新 Registry（可选，默认调用 load）
    fn reload(&self, old: &Self::Registry, def: Self::Def) -> Result<Self::Registry, RegistryLoadError> {
        self.load(def) // 默认：全量替换
    }
}
```

### 2.2 Registry trait

```rust
/// 运行时 Registry：提供只读查找接口。
pub trait Registry: Send + Sync + 'static {
    type Key: std::fmt::Display + ?Sized;
    type Value: 'static;

    fn get(&self, key: &Self::Key) -> Option<&Self::Value>;
    fn contains(&self, key: &Self::Key) -> bool { self.get(key).is_some() }
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool { self.len() == 0 }
    fn iter(&self) -> Box<dyn Iterator<Item = (&Self::Key, &Self::Value)> + '_>;
}
```

### 2.3 Registry 初始化顺序（DAG）

```
Layer 1 (独立，零依赖)
├── TagRegistry        — 标签定义，无外部依赖
├── AttributeRegistry  — 属性定义，无外部依赖
├── TerrainRegistry    — 地形定义，无外部依赖
├── FactionRegistry    — 阵营定义，无外部依赖
├── ClassRegistry      — 职业定义，无外部依赖
├── RaceRegistry       — 种族定义，无外部依赖

Layer 2 (依赖 Tag)
├── ModifierRegistry   — 依赖 Tag（条件型 Modifier 需要 Tag 引用）
├── TargetingRegistry  — 依赖 Tag（筛选条件需要 Tag 引用）

Layer 3 (依赖 Tag + Modifier)
├── ExecutionRegistry  — 依赖 Tag + Modifier（算式上下文引用）
├── StackingRegistry   — 依赖 Tag（互斥规则引用）
├── CueRegistry        — 无外部依赖（纯表现事件）

Layer 4 (依赖 Effect + Tag + Targeting + Execution + Stacking)
├── EffectRegistry     — 依赖 Tag + Execution + Stacking + Cue

Layer 5 (依赖 Effect + Tag + Targeting)
├── AbilityRegistry    — 依赖 Effect + Targeting + Tag
├── TriggerRegistry    — 依赖 Effect + Tag

Layer 6 (依赖 Ability 等)
├── CharacterRegistry  — 依赖 Ability + Trait + Class + Equipment + Race
├── EquipmentRegistry  — 依赖 Modifier + Tag + Trait
├── ItemRegistry       — 依赖 Modifier + Tag
├── TraitRegistry      — 依赖 Modifier + Tag + Trigger

Layer 7 (场景/关卡依赖所有)
├── CampaignRegistry   — 依赖 Stage
├── StageRegistry      — 依赖 Character + Terrain + Faction
├── AiBehaviorRegistry — 依赖 Tag + Ability
```

### 2.4 热重载策略

| Registry 类型 | 热重载支持 | 策略 |
|---------------|-----------|------|
| TagRegistry | ✅ 支持 | 全量替换，运行时校验未使用标签 |
| AttributeRegistry | ✅ 支持 | 全量替换 |
| TerrainRegistry | ✅ 支持 | 全量替换 |
| ModifierRegistry | ❌ 战斗锁定 | `AppState::InGame` 时锁定 |
| EffectRegistry | ❌ 战斗锁定 | 同上 |
| AbilityRegistry | ❌ 战斗锁定 | 同上 |
| CharacterRegistry | ❌ 战斗锁定 | 同上 |
| ExecutionRegistry | ✅ 支持 | 纯计算逻辑，无状态 |
| CueRegistry | ✅ 支持 | 纯表现定义 |
| StackingRegistry | ✅ 支持 | 纯规则定义 |
| TriggerRegistry | ❌ 战斗锁定 | 同上 |
| TargetingRegistry | ✅ 支持 | 纯规则定义 |
| EquipmentRegistry | ❌ 战斗锁定 | 同上 |

---

## 3. RON Schema 设计

### 3.1 Attribute Schema

```ron
// content/attributes/attributes.ron
(
    attributes: [
        // 核心属性 (Core)
        (id: "phys_atk",  name_key: "attr.a_001.name", category: Core,    default: 10.0, min: 0.0,  max: 99999.0),
        (id: "magic_atk", name_key: "attr.a_002.name", category: Core,    default: 10.0, min: 0.0,  max: 99999.0),
        (id: "phys_def",  name_key: "attr.a_003.name", category: Core,    default: 5.0,  min: 0.0,  max: 99999.0),
        (id: "magic_def", name_key: "attr.a_004.name", category: Core,    default: 5.0,  min: 0.0,  max: 99999.0),
        (id: "max_hp",    name_key: "attr.a_005.name", category: Core,    default: 100.0, min: 1.0,  max: 99999.0),
        // 次级属性 (Secondary)
        (id: "crit_rate",   name_key: "attr.a_006.name", category: Secondary, default: 0.05, min: 0.0,   max: 0.95),
        (id: "crit_dmg",    name_key: "attr.a_007.name", category: Secondary, default: 1.5,  min: 1.5,   max: 5.0),
        (id: "move_range",  name_key: "attr.a_008.name", category: Secondary, default: 3.0,  min: 1.0,   max: 99.0),
        (id: "atk_range",   name_key: "attr.a_009.name", category: Secondary, default: 1.0,  min: 1.0,   max: 99.0),
        (id: "hit_rate",    name_key: "attr.a_010.name", category: Secondary, default: 1.0,  min: 0.0,   max: 1.0),
        (id: "dodge_rate",  name_key: "attr.a_011.name", category: Secondary, default: 0.05, min: 0.0,   max: 0.8),
    ],
    boundaries: (
        max_damage_reduction: 0.9,      // 减伤上限 90%
        rounding: Floor,                 // 取整方式
        min_final_damage: 1,             // 伤害下限（真实伤害可到 0）
    ),
)
```

### 3.2 Tag Schema

```ron
// content/tags/tags.ron
(
    // 5 分类：Elemental / Status / Class / Equipment / Mechanism
    tags: [
        // ===== Elemental (伤害/元素类型) =====
        (id: "dmg_physical",  category: Elemental, name_key: Some("tag.t_001.name"), priority_weight: 0, dispellable: false, reflectable: true),
        (id: "dmg_magical",   category: Elemental, name_key: Some("tag.t_002.name"), priority_weight: 0, dispellable: false, reflectable: true),
        (id: "dmg_pierce",    category: Elemental, name_key: Some("tag.t_003.name"), priority_weight: 0, dispellable: false, reflectable: false),
        (id: "dmg_true",      category: Elemental, name_key: Some("tag.t_004.name"), priority_weight: 0, dispellable: false, reflectable: false),
        (id: "fire",          category: Elemental, name_key: Some("tag.t_005.name"), priority_weight: 0, dispellable: false, reflectable: true),
        (id: "ice",           category: Elemental, name_key: Some("tag.t_006.name"), priority_weight: 0, dispellable: false, reflectable: true),

        // ===== Status (状态) =====
        (id: "buff",           category: Status, name_key: Some("tag.t_010.name"), priority_weight: 0,  dispellable: true,  reflectable: false),
        (id: "debuff",         category: Status, name_key: Some("tag.t_011.name"), priority_weight: 0,  dispellable: true,  reflectable: false),
        (id: "special_state",  category: Status, name_key: Some("tag.t_012.name"), priority_weight: 0,  dispellable: false, reflectable: false),
        (id: "control_soft",   category: Status, name_key: Some("tag.t_013.name"), priority_weight: 1,  dispellable: true,  reflectable: false),
        (id: "control_hard",   category: Status, name_key: Some("tag.t_014.name"), priority_weight: 2,  dispellable: true,  reflectable: false),
        (id: "control_full",   category: Status, name_key: Some("tag.t_015.name"), priority_weight: 3,  dispellable: true,  reflectable: false),

        // ===== Class (职业/身份/阵营) =====
        (id: "warrior",   category: Class, name_key: Some("tag.t_020.name"), priority_weight: 0, dispellable: false, reflectable: false),
        (id: "mage",      category: Class, name_key: Some("tag.t_021.name"), priority_weight: 0, dispellable: false, reflectable: false),
        (id: "archer",    category: Class, name_key: Some("tag.t_022.name"), priority_weight: 0, dispellable: false, reflectable: false),
        (id: "ally",      category: Class, name_key: Some("tag.t_023.name"), priority_weight: 0, dispellable: false, reflectable: false),
        (id: "enemy",     category: Class, name_key: Some("tag.t_024.name"), priority_weight: 0, dispellable: false, reflectable: false),
        (id: "boss",      category: Class, name_key: Some("tag.t_025.name"), priority_weight: 0, dispellable: false, reflectable: false),
        (id: "summon",    category: Class, name_key: Some("tag.t_026.name"), priority_weight: 0, dispellable: false, reflectable: false),

        // ===== Equipment (装备/物品) =====
        (id: "sword",         category: Equipment, name_key: Some("tag.t_030.name"), priority_weight: 0, dispellable: false, reflectable: false),
        (id: "staff",         category: Equipment, name_key: Some("tag.t_031.name"), priority_weight: 0, dispellable: false, reflectable: false),
        (id: "heavy_armor",   category: Equipment, name_key: Some("tag.t_032.name"), priority_weight: 0, dispellable: false, reflectable: false),
        (id: "accessory",     category: Equipment, name_key: Some("tag.t_033.name"), priority_weight: 0, dispellable: false, reflectable: false),

        // ===== Mechanism (机制) =====
        (id: "flying",          category: Mechanism, name_key: None, priority_weight: 0,  dispellable: false, reflectable: false),
        (id: "grounded",        category: Mechanism, name_key: None, priority_weight: 0,  dispellable: false, reflectable: false),
        (id: "dispellable",     category: Mechanism, name_key: None, priority_weight: 0,  dispellable: false, reflectable: false),
        (id: "undispellable",   category: Mechanism, name_key: None, priority_weight: 0,  dispellable: false, reflectable: false),
        (id: "reflectable",     category: Mechanism, name_key: None, priority_weight: 0,  dispellable: false, reflectable: false),
        (id: "invincible",      category: Mechanism, name_key: Some("tag.t_040.name"), priority_weight: 99, dispellable: false, reflectable: false),
        (id: "untargetable",    category: Mechanism, name_key: Some("tag.t_041.name"), priority_weight: 98, dispellable: false, reflectable: false),
    ],
    mutual_exclusions: [
        (tag_a: "flying",       tag_b: "grounded"),
        (tag_a: "dmg_physical", tag_b: "dmg_magical"),
        (tag_a: "control_full", tag_b: "control_hard"),
        (tag_a: "control_full", tag_b: "control_soft"),
        (tag_a: "invincible",   tag_b: "dmg_physical"),
        (tag_a: "invincible",   tag_b: "dmg_magical"),
        (tag_a: "invincible",   tag_b: "dmg_pierce"),
        (tag_a: "invincible",   tag_b: "dmg_true"),
    ],
)
```

### 3.3 Modifier Schema

```ron
// content/modifiers/modifiers.ron
(
    modifiers: [
        // 攻击提升 20%（加算百分比）
        (id: "atk_up_20",    target_attr: "phys_atk",  operation: AddPercent, value: 0.2,  stacking_rule: "additive_same_name_max", source_type: Buff),
        // 降防 40%（乘算）
        (id: "def_down_40",  target_attr: "phys_def",  operation: MulPercent, value: 0.4,  stacking_rule: "multiplicative",        source_type: Buff),
        // 防御 +50 固定值
        (id: "def_up_50",    target_attr: "phys_def",  operation: Add,        value: 50.0, stacking_rule: "additive",             source_type: Equipment),
        // 暴击率 +15%（加算百分比）
        (id: "crit_up_15",   target_attr: "crit_rate", operation: AddPercent, value: 0.15, stacking_rule: "additive",             source_type: Buff),
        // 常驻攻击 +10%（装备）
        (id: "equip_atk_10", target_attr: "phys_atk",  operation: AddPercent, value: 0.10, stacking_rule: "additive",             source_type: Equipment),
    ],
)
```

**ModifierOp 枚举**：

```rust
pub enum ModifierOp {
    Add(f32),         // 固定加法：base + value
    AddPercent(f32),  // 百分比加法：base × (1 + Σvalue)
    MulPercent(f32),  // 乘算：base × Π(1 - value)
}
```

### 3.4 Execution Schema

```ron
// content/executions/executions.ron
(
    executions: [
        // 物理伤害计算
        (id: "phys_damage_calc", execution_type: Damage,
         formula: AttackMinusDefense,
         params: (
            damage_type: "dmg_physical",
            can_crit: true,
            atk_attr: "phys_atk",
            def_attr: "phys_def",
         )),
        // 魔法伤害计算
        (id: "magic_damage_calc", execution_type: Damage,
         formula: AttackMinusDefense,
         params: (
            damage_type: "dmg_magical",
            can_crit: true,
            atk_attr: "magic_atk",
            def_attr: "magic_def",
         )),
        // 固定数值治疗
        (id: "heal_fixed", execution_type: Heal,
         formula: FixedValue,
         params: (heal_type: Fixed, can_crit: false)),
        // 物理护盾
        (id: "shield_physical", execution_type: Shield,
         formula: FixedValue,
         params: (shield_type: Physical, can_regen: false)),
    ],
)
```

### 3.5 Effect Schema

```ron
// content/effects/effects.ron
(
    effects: [
        // ==== 伤害效果 ====
        (id: "phys_damage",      effect_type: Damage,
         execution: "phys_damage_calc",
         cue: Some("hit_physical"),
         tags_required: [], tags_forbidden: ["invincible"]),
        (id: "fire_damage",      effect_type: Damage,
         execution: "magic_damage_calc",
         cue: Some("hit_fire"),
         tags_required: [], tags_forbidden: ["invincible"]),

        // ==== 治疗效果 ====
        (id: "heal_small",       effect_type: Heal,
         execution: "heal_fixed",
         cue: Some("heal_light")),

        // ==== 护盾效果 ====
        (id: "shield_phys_100",  effect_type: ApplyShield,
         execution: "shield_physical",
         cue: Some("shield_on"),
         duration: Some((duration_type: Turns, value: 3, tick_timing: TurnEnd))),

        // ==== Buff 效果（持续效果） ====
        (id: "poison",           effect_type: ApplyBuff,
         name_key: Some("buff.b_001.name"),
         desc_key: Some("buff.b_001.desc"),
         duration: Some((duration_type: Turns, value: 3, tick_timing: ActionEnd)),
         stacking: Some("stack_independent"),
         max_stack: 9,
         modifiers: ["def_down_40"],
         tick_effects: ["poison_tick_damage"],
         cue: Some("poison_applied"),
         tags_applied: ["debuff"],
         dispellable: true),

        // ==== 驱散效果 ====
        (id: "dispel_all_debuff", effect_type: Dispel,
         dispel_type: DebuffOnly,
         dispel_count: 3,
         cue: Some("dispel_effect")),

        // ==== 位移效果 ====
        (id: "knockback_2",     effect_type: Displacement,
         displacement: (displacement_type: Forced, distance: 2,
                        can_cross_obstacle: false, wall_damage_pct: Some(0.1)),
         cue: Some("knockback")),

        // ==== 召唤效果 ====
        (id: "summon_wolf",     effect_type: Summon,
         summon_template: "wolf",
         inherit_ratio: 0.6,
         max_count: 3,
         duration: Some((duration_type: Turns, value: 5, tick_timing: TurnEnd)),
         cue: Some("summon_effect")),

        // ==== 死亡效果（内部链路，不可直接配置） ====
        (id: "kill_entity",     effect_type: Kill,
         cue: Some("death_effect")),
    ],
)
```

**EffectType 枚举**（8 种）：

```rust
pub enum EffectType {
    Damage(DamageEffect),
    Heal(HealEffect),
    ApplyBuff(BuffEffect),     // 持续效果 = Effect + Duration + Modifiers
    Dispel(DispelEffect),
    Displacement(DisplacementEffect),
    ApplyShield(ShieldEffect),
    Summon(SummonEffect),
    Kill,
}
```

### 3.6 Stacking Schema

```ron
// content/stacking/stacking.ron
(
    stacking_rules: [
        // 同名取最大层数（如中毒）
        (id: "additive_same_name_max",   stack_type: AdditiveSameNameMax,
         max_stack: 9, refresh_duration: true, decay_on_tick: false),
        // 同名叠加层数（如属性Buff）
        (id: "additive_same_name",       stack_type: AdditiveSameName,
         max_stack: 5, refresh_duration: true, decay_on_tick: false),
        // 独立共存（不同来源各自计时）
        (id: "stack_independent",        stack_type: Independent,
         max_stack: 1, refresh_duration: false, decay_on_tick: false),
        // 强覆盖（新覆盖旧）
        (id: "override_new",             stack_type: OverrideNew,
         max_stack: 1, refresh_duration: false, decay_on_tick: false),
        // 弱覆盖（旧持续，新不进）
        (id: "override_old",             stack_type: OverrideOld,
         max_stack: 1, refresh_duration: false, decay_on_tick: false),
        // 乘算叠加（如减伤）
        (id: "multiplicative",           stack_type: Multiplicative,
         max_stack: 99, refresh_duration: false, decay_on_tick: false),
        // 无法叠加（仅存在/不存在）
        (id: "binary",                   stack_type: Binary,
         max_stack: 1, refresh_duration: true, decay_on_tick: false),
        // 衰减型（触发后减半层数）
        (id: "decay_on_trigger",         stack_type: DecayOnTrigger,
         max_stack: 9, refresh_duration: false, decay_on_tick: true),
    ],
)
```

**StackType 枚举**（8 种）：

```rust
pub enum StackType {
    AdditiveSameNameMax,  // 同名取最大
    AdditiveSameName,     // 同名叠加层数
    Independent,          // 独立共存
    OverrideNew,          // 新覆盖旧
    OverrideOld,          // 旧保留
    Multiplicative,       // 乘算
    Binary,               // 二值
    DecayOnTrigger,       // 触发衰减
}
```

### 3.7 Ability Schema

```ron
// content/abilities/abilities.ron
(
    abilities: [
        // 普通攻击
        (id: "normal_attack",
         name_key: "skill.s_1000.name",
         desc_key: "skill.s_1000.desc",
         ability_type: NormalAttack,
         cost: (ap: 1, cp: 0),
         cooldown: None,
         range: 1,
         targeting: "single_enemy",
         effects: ["phys_damage"],
         tags_required: [],
         tags_forbidden: ["control_full"],
         special_rules: []),

        // 火球术
        (id: "fireball",
         name_key: "skill.s_1001.name",
         desc_key: "skill.s_1001.desc",
         ability_type: ActiveSkill,
         cost: (ap: 1, cp: 30),
         cooldown: Some(3),
         range: 3,
         targeting: "single_enemy",
         effects: ["fire_damage", "apply_burn"],
         tags_required: [],
         tags_forbidden: ["silenced", "control_full"],
         special_rules: [(can_cast_after_move: true)]),
    ],
)
```

### 3.8 Trigger Schema

```ron
// content/triggers/triggers.ron
(
    triggers: [
        // 反击
        (id: "counter_attack",
         event_type: TakeDamage,
         condition: Some((weapon_type_eq: Melee)),
         effect: "phys_damage",
         priority: 100,
         max_trigger_per_turn: Some(99),
         max_trigger_per_battle: None,
         chain_depth: 0),

        // 援护
        (id: "bodyguard",
         event_type: AllyTakeDamage,
         condition: Some((distance_le: 1)),
         effect: "redirect_damage",
         priority: 150,
         max_trigger_per_turn: Some(3),
         max_trigger_per_battle: None,
         chain_depth: 1),

        // 追击
        (id: "follow_up",
         event_type: AllyDealDamage,
         condition: Some((target_hp_pct_le: 0.5)),
         effect: "follow_up_damage",
         priority: 50,
         max_trigger_per_turn: Some(1),
         max_trigger_per_battle: None,
         chain_depth: 1),
    ],
)
```

### 3.9 Targeting Schema

```ron
// content/targetings/targetings.ron
(
    targetings: [
        (id: "single_enemy",   target_type: SingleEnemy, range: 1,
         filters: [(faction: "enemy")]),
        (id: "single_ally",    target_type: SingleAlly,  range: 3,
         filters: [(faction: "ally")]),
        (id: "self",           target_type: Self,        range: 0,
         filters: []),
        (id: "aoe_enemy_cross", target_type: AoEEnemy,   range: 3,
         aoe_shape: Some(Cross(radius: 1)),
         filters: [(faction: "enemy")]),
        (id: "aoe_enemy_circle", target_type: AoEEnemy,  range: 3,
         aoe_shape: Some(Circle(radius: 2)),
         filters: [(faction: "enemy")]),
        (id: "aoe_ally",       target_type: AoEAlly,     range: 2,
         aoe_shape: Some(Circle(radius: 2)),
         filters: [(faction: "ally")]),
        (id: "all_enemy",      target_type: AllEnemy,    range: 99,
         filters: [(faction: "enemy")]),
        (id: "directional_line", target_type: DirectionalLine, range: 4,
         aoe_shape: Some(Line(width: 1)),
         filters: [(faction: "enemy")]),
    ],
)
```

### 3.10 Cue Schema

```ron
// content/cues/cues.ron
(
    cues: [
        // 伤害类
        (id: "hit_physical",  cue_type: Hit,   params: (vfx: "vfx_hit_phys",  sfx: "sfx_hit_phys")),
        (id: "hit_fire",      cue_type: Hit,   params: (vfx: "vfx_hit_fire",  sfx: "sfx_hit_fire")),
        (id: "hit_magic",     cue_type: Hit,   params: (vfx: "vfx_hit_magic", sfx: "sfx_hit_magic")),
        // 治疗类
        (id: "heal_light",    cue_type: Heal,  params: (vfx: "vfx_heal",      sfx: "sfx_heal")),
        (id: "heal_crit",     cue_type: Heal,  params: (vfx: "vfx_heal_crit", sfx: "sfx_heal_crit")),
        // Buff 类
        (id: "poison_applied", cue_type: BuffApplied, params: (vfx: "vfx_poison", sfx: "sfx_poison")),
        (id: "shield_on",     cue_type: Shield, params: (vfx: "vfx_shield",   sfx: "sfx_shield")),
        (id: "dispel_effect", cue_type: Dispel, params: (vfx: "vfx_dispel",   sfx: "sfx_dispel")),
        // 位移类
        (id: "knockback",     cue_type: Knockback, params: (vfx: "vfx_push",   sfx: "sfx_push")),
        (id: "summon_effect", cue_type: Summon,    params: (vfx: "vfx_summon", sfx: "sfx_summon")),
        // 死亡/特殊
        (id: "death_effect",  cue_type: Death,   params: (vfx: "vfx_death",   sfx: "sfx_death")),
        (id: "crit_hit",      cue_type: Crit,    params: (vfx: "vfx_crit",    sfx: "sfx_crit")),
    ],
)
```

**CueType 枚举**（12 种）：

```rust
pub enum CueType {
    Hit, Heal, Crit, Shield, BuffApplied, BuffRemoved,
    Dispel, Knockback, Summon, Death, LevelUp, Special,
}
```

---

## 4. Content 目录结构

```
content/
├── attributes/
│   └── attributes.ron              # 属性定义（11 项）
├── tags/
│   └── tags.ron                    # 标签定义（5 类 ~30 个）
├── modifiers/
│   └── modifiers.ron               # Modifier 定义
├── executions/
│   └── executions.ron              # Execution 算式定义
├── effects/
│   └── effects.ron                 # Effect 定义（8 类型）
├── stackings/
│   └── stackings.ron               # 堆叠策略定义（8 种）
├── abilities/
│   ├── abilities.ron               # Ability 定义
│   └── normal_attacks.ron          # 普攻统一管理
├── triggers/
│   └── triggers.ron                # Trigger 定义（5 大类事件）
├── targetings/
│   └── targetings.ron              # Targeting 定义（7 种类型）
├── cues/
│   └── cues.ron                    # Cue 定义（12 种）
├── characters/
│   ├── classes.ron                 # 职业定义
│   ├── races.ron                   # 种族定义
│   ├── traits.ron                  # 特质定义
│   └── units/                      # 单位模板（按关卡/类型分组）
│       ├── player_units.ron
│       └── enemy_units.ron
├── equipments/
│   └── equipments.ron              # 装备定义
├── items/
│   └── items.ron                   # 物品定义
├── terrains/
│   └── terrains.ron                # 地形定义
├── ai/
│   └── ai_behaviors.ron            # AI 行为定义
├── stages/
│   ├── stage_001.ron               # 关卡配置（每关一个文件）
│   ├── stage_002.ron
│   └── ...
├── campaigns/
│   └── campaigns.ron               # 战役编排
└── global_config.ron               # 全局数值边界配置
```

---

## 5. 数据迁移规则

### 5.1 Schema 演化策略

```
小版本（+0.1）：新增可选字段（Option<T>），旧 RON 无需修改
大版本（+1.0）：删除字段 / 类型变更 / 必填字段新增，需要迁移

规则：
- RON 中所有 struct 必须标记 #[non_exhaustive]
- 新字段必须为 Option<T> 或 #[serde(default)]
- 删除字段必须经过 3 步：deprecate(版本N) → warn(版本N+1) → remove(版本N+2)
```

### 5.2 三步删除规则

| 步骤 | 版本 N | 版本 N+1 | 版本 N+2 |
|------|--------|----------|----------|
| 代码 | 标记 `#[deprecated]` | 加载时输出 WARN 日志 | 删除字段 |
| RON | 字段保留，含义不变 | 字段保留，标注"即将删除" | 字段可删除 |
| 兼容 | 完全兼容 | 加载兼容，运行时不再使用 | 不再加载 |

### 5.3 版本标记

```ron
// 每个 RON 文件头部包含版本号
// 顶层 struct 示例：
(
    _schema_version: "1.0",  // 数据架构版本
    attributes: [...],
)
```

版本比较规则：
- RON 文件版本 ≤ 运行时支持版本 → 加载
- RON 文件版本 > 运行时支持版本 → WARN 日志 + 尝试加载
- RON 文件大版本不匹配 → ERROR 日志 + 跳过

---

## 6. Save/Replay 兼容

### 6.1 Save 格式

存档只保存 **Instance** 数据，**Definition** 数据从 RON 重建：

```ron
// save_data.ron (简化示意)
(
    _save_version: "1.0",
    _timestamp: "...",
    battle_state: (
        turn_number: 5,
        turn_phase: PlayerPhase,
    ),
    units: [
        (
            unit_id: "player_warrior",
            position: (x: 3, y: 5),
            current_hp: 75,
            core_attrs: {
                "phys_atk": 50,
                "phys_def": 25,
                "max_hp": 100,
            },
            secondary_attrs: {
                "crit_rate": 0.05,
                "move_range": 3,
            },
            ability_cooldowns: {
                "fireball": 2,
            },
            active_effects: [
                (effect_id: "poison", remaining_duration: 2, current_stack: 3),
            ],
        ),
    ],
)
```

**关键原则**：
- 存档引用 Definition ID，不嵌入 Definition 内容
- 加载时从 Registry 重建 Definition 引用
- 如果 Definition 不存在 → 丢弃该 ID 并输出 WARN

### 6.2 Replay 格式

基于 `docs/04-data/ll/10_Pipeline_Replay_ll.md` 的 Command Stream 模型：

```ron
// replay_stream.ron
(
    _replay_version: "1.0",
    _seed: 123456789,
    _initial_state_hash: "0x...",
    events: [
        (turn: 1, sequence: 0, event: TurnStart),
        (turn: 1, sequence: 1, event: UseAbility(actor: "unit_1", ability: "normal_attack", target: "enemy_1")),
        (turn: 1, sequence: 2, event: UseAbility(actor: "unit_1", ability: "fireball", target: "enemy_2")),
        (turn: 1, sequence: 3, event: TurnEnd),
        (turn: 2, sequence: 0, event: TurnStart),
        // ...
    ],
    end_state_hash: "0x...",
)
```

**确定性保证**：
- 每个 ReplayEvent 记录 `(turn, sequence, actor, ability_id, target, seed_snapshot)`
- Replay 时从同一初始状态 + 事件序列 + RNG 种子 → 必然相同结果
- Replay 消费 Command Stream（Track A），不消费 Audit Trail（Track B）

---

## 7. 数据校验规则

### 7.1 Schema 验证（Level 1）

| 校验项 | 时机 | 失败处理 |
|--------|------|----------|
| RON 反序列化成功 | 文件加载 | ERROR + 跳过文件 |
| 所有必填字段非空 | 反序列化后 | ERROR + 跳过 |
| 枚举值在允许范围内 | 反序列化后 | ERROR + 跳过 |
| 数值在 min/max 范围内 | 注册时 | WARN + clamp |

### 7.2 引用完整性验证（Level 2）

| 校验项 | 涉及领域 | 失败处理 |
|--------|---------|----------|
| Effect.execution 引用有效 ExecutionId | Effect → Execution | ERROR + 注册失败 |
| Effect.modifiers 引用有效 ModifierId | Effect → Modifier | ERROR + 注册失败 |
| Effect.stacking 引用有效 StackingId | Effect → Stacking | ERROR + 注册失败 |
| Effect.tags_applied 引用有效 TagId | Effect → Tag | ERROR + 注册失败 |
| Ability.effects 引用有效 EffectId | Ability → Effect | ERROR + 注册失败 |
| Ability.targeting 引用有效 TargetingId | Ability → Targeting | ERROR + 注册失败 |
| Ability.tags_required 引用有效 TagId | Ability → Tag | ERROR + 注册失败 |
| Trigger.effect 引用有效 EffectId | Trigger → Effect | ERROR + 注册失败 |
| Trigger.condition 引用有效 TagId | Trigger → Tag | WARN + condition 失效 |
| Modifier.target_attr 引用有效 AttributeId | Modifier → Attribute | ERROR + 注册失败 |

### 7.3 跨域一致性验证

| 校验项 | 失败处理 |
|--------|----------|
| 所有 ID 全局唯一（不跨类型） | ERROR + 列出冲突 |
| 引用的 Registry 初始化顺序正确 | ERROR + 输出 DAG 顺序 |
| Tag 互斥表无自引用 | WARN + 自动修正 |
| Attribute min < max | ERROR + 修正 |

---

## 8. 依赖于当前实现的决策

| 待确认项 | 当前建议 | 最终决定者 |
|----------|---------|-----------|
| `define_id!` 宏的 `Reflect` 默认值 | 默认 true，领域标记 false 的可以不生成 | @feature-developer |
| Registry 使用 `HashMap<String, T>` 还是 `HashMap<IdType, T>` | `HashMap<IdType, T>` 以利用强类型 | @feature-developer |
| Effect 的 `tick_effects` 是否也走 Effect 引用 | 走 EffectId 引用（递归检查） | @data-architect（实现时确认） |
| Cue 的 vfx/sfx ID 格式 | 用字符串标识，由表现层映射到资产路径 | @feature-developer |
| RON 文件：一个文件 vs 一个目录多个文件 | 量小的（<50 项）用单文件；单位模板等多文件的用目录 | @data-architect |
