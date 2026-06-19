---
id: 03-content.definitions.entities.summon-def
title: SummonDef — Summon Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# SummonDef — Summon Content Def 定义

> **Content Layer**: L2 Entity | **领域规则**: `docs/02-domain/domains/summon_domain.md` | **数据 Schema**: `docs/04-data/domains/summon_schema.md` | **插件代码**: `src/content/plugins/summon_plugin.rs`

---

## 1. Overview

SummonDef 定义了召唤物模板——通过技能/法术/能力召唤到战场上的临时实体。SummonDef 不嵌入 CreatureBase 或 ItemBase，因为召唤物的生命周期和行为与角色/怪物有本质区别。

### 关键设计原则

- **召唤物 ≠ 角色**：召唤物没有装备系统、没有职业、没有经验值，也没有永久的队伍归属。它们是临时实体
- **召唤物 ≠ 怪物**：召唤物由玩家控制（或遵循 SummonAI），不受 Encounter 系统管理，消灭不提供经验值
- **能力通过 AbilityDef**：召唤物的技能引用 L1 AbilityDef，与角色/怪物共享同一套能力系统
- **持续时间独立定义**：SummonDef 直接定义持续时间规则（固定回合数/专注维持/永久），不依赖 L1 EffectDef 的 duration 机制
- **AI 行为模式**：召唤物 AI 与 MonsterDef 的 AI 不同——召唤物 AI 侧重于"跟随召唤者指令"，而非"自主战斗"

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `summon_domain.md` | 召唤物分类、持续时间规则、消亡触发条件 |
| `summon_schema.md` | SummonDef 完整字段结构、SummonDuration、SummonAIBehavior 定义 |
| `attribute-def.md` | 本 Def 的 `base_attributes` 键引用的 AttributeDef |
| `ability-def.md` | 本 Def 的 `abilities` 引用的 AbilityDef |
| `buff-def.md` | 本 Def 的 `spawn_buffs` 引用的 BuffDef |
| `effect-def.md` | 本 Def 的 `on_summon_effect` 和 `on_dismiss_effect` 引用的 EffectDef |
| `condition-def.md` | 本 Def 的 `dismissal_conditions` 引用的 ConditionDef |
| `tag-def.md` | 本 Def 的 `tags` 引用的 TagDef |
| `faction-def.md` | 本 Def 的 `faction_override` 引用的 FactionDef |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 召唤物模板定义——描述一个可被召唤到战场上的临时实体。
///
/// SummonDef 是 Content Asset，经 Load → Deserialize → Validate → Register → Freeze
/// 管线后进入 DefRegistry<SummonDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct SummonDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID（SummonDef 前缀: `sum_`）
    pub id: SummonId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 基础属性 ──
    /// 召唤物基础属性（引用 L0 AttributeDef）
    ///
    /// 这些值通常基于召唤者的属性和法术等级计算，但 SummonDef 提供基础模板值。
    pub base_attributes: Vec<(AttributeId, f32)>,

    /// 召唤物拥有的技能（引用 L1 AbilityDef）
    ///
    /// 这些技能在召唤物存在期间可用，消失后自动注销。
    pub abilities: Vec<AbilityId>,

    /// 召唤物出生时获得 Buff（引用 L1 BuffDef）
    ///
    /// 如"亡灵召唤物获得暗影抗性"、"元素召唤物获得元素亲和"
    pub spawn_buffs: Vec<BuffId>,

    // ── 召唤与消失效果 ──
    /// 召唤出场时触发的效果（引用 L1 EffectDef，可选）
    pub on_summon_effect: Option<EffectId>,

    /// 召唤物消失/死亡时触发的效果（引用 L1 EffectDef，可选）
    pub on_dismiss_effect: Option<EffectId>,

    // ── 阵营 ──
    /// 阵营覆写——覆盖召唤者阵营（可选）
    ///
    /// None = 继承召唤者阵营。Some = 使用指定阵营（如"敌对召唤物使用敌对阵营"）
    pub faction_override: Option<FactionId>,

    // ── 标签 ──
    /// 召唤物标签（引用 L0 TagDef）
    pub tags: Vec<TagId>,

    // ── 行为 ──
    /// 召唤物 AI 行为模式
    pub ai_behavior: SummonAIBehavior,

    // ── 持续时间 ──
    /// 召唤持续时间规则
    pub duration: SummonDuration,

    /// 消失触发条件（引用 L1 ConditionDef）
    ///
    /// 额外触发条件——满足时提前消失。如"召唤者失去专注"、"召唤者离开战场"
    pub dismissal_conditions: Option<Vec<ConditionId>>,

    // ── 战场占位 ──
    /// 召唤物在战场上占用的网格数
    ///
    /// 1 = 标准 1x1，2 = 大型 2x2（如召唤元素），以此类推
    pub grid_occupation: u32,

    // ── 资源限制 ──
    /// 同一召唤者可同时拥有的此召唤物最大数量
    pub max_per_summoner: u32,

    // ── 表现资源 ──
    /// 图标 Key
    pub icon_key: Option<String>,
    /// 模型 Key
    pub model_key: Option<String>,
}
```

### 内嵌数据结构

```rust
/// 召唤持续时间类型
#[derive(Deserialize, Clone, Debug)]
pub enum SummonDuration {
    /// 固定回合数
    FixedTurns(u32),
    /// 召唤者保持专注则持续（中断专注则消失）
    Concentration,
    /// 永久存在（直到被驱逐或死亡）
    Permanent,
    /// 条件驱动（绑定到某个 ConditionDef）
    ConditionDriven(ConditionId),
}

/// 召唤物 AI 行为模式
#[derive(Deserialize, Clone, Debug)]
pub enum SummonAIBehavior {
    /// 被动跟随——不自主行动，跟随召唤者的指令
    Passive,
    /// 防御守护——保护召唤者，攻击靠近的敌人
    Guardian,
    /// 自主攻击——主动搜索并攻击敌人
    Aggressive,
    /// 策略自主——基于 Minimax 或其他策略自主决策
    Tactical,
    /// 预定义脚本——按固定的行为剧本执行
    Scripted(Vec<SummonScriptedStep>),
    /// 自定义（Mod 扩展用）
    Custom(String),
}

/// 脚本化的召唤物行为步骤
#[derive(Deserialize, Clone, Debug)]
pub struct SummonScriptedStep {
    pub condition: Option<ConditionId>,
    pub ability_id: AbilityId,
    pub target_preference: SummonTargetPreference,
    pub priority: u8,
}

/// 召唤物目标偏好
#[derive(Deserialize, Clone, Debug)]
pub enum SummonTargetPreference {
    SummonersTarget,
    NearestEnemy,
    WeakestEnemy,
    MostDangerous,
    SameAsSummoner,
}

/// 召唤物来源分类枚举
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum SummonSourceCategory {
    Spell,
    Ability,
    Illusion,
    Permanent,
    Item,
    Custom(String),
}
```

### 字段说明

- **`base_attributes`**: 提供召唤物的基础模板值。实际召唤时，这些值可能根据召唤者的属性和法术等级进行缩放。缩放规则由 ExecutionDef 或 AbilityDef 定义
- **`faction_override`**: None 时召唤物与召唤者同阵营。这在大多数情况下是合理的（玩家的召唤物是友方）。Some 时可用于"召唤敌对生物"或"召唤中立生物"
- **`duration`**: 核心生命周期控制。Concentration 类型在召唤者受伤时需做专注检定（规则见 summon_domain.md），Permanent 类型用于永久的宠物/随从
- **`dismissal_conditions`**: 额外提前消失条件。"召唤者离开战场"是最常见的——战斗结束时所有非永久召唤物自动消失
- **`max_per_summoner`**: 数量限制。1 = 唯一的召唤物（如 Ranger 的动物伙伴），3 = 亡灵法师可同时召唤 3 个骷髅，以此类推

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// SummonDef 注册插件
pub struct SummonDefPlugin;

impl Plugin for SummonDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<SummonDef>();
        app.init_asset_loader::<RonAssetLoader<SummonDef>>();
        app.insert_resource(DefRegistry::<SummonDef>::new());
        app.add_systems(
            PreUpdate,
            load_summon_defs
                .run_if(resource_changed::<Assets<SummonDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 SummonDef
pub fn get_summon_def(
    summon_id: &SummonId,
    registry: &DefRegistry<SummonDef>,
) -> Option<&SummonDef> {
    registry.get(summon_id)
}

/// 按来源分类过滤 SummonDef（L3 领域系统使用）
pub fn get_summon_defs_by_source(
    source: &str,
    _other_criteria: &[TagId],
    _registry: &DefRegistry<SummonDef>,
) -> Vec<&SummonDef> {
    // 基于标签和分类过滤（聚合查询）
    unimplemented!("L3 调用方使用 tag + category 过滤")
}
```

### 注册生命周期

```
SummonDefPlugin::build
  │
  ├── SummonDef 从 assets/config/02_entities/summons.ron 加载
  │
  ├── Deserialize (ron::from_str)
  │
  ├── Validate
  │     ├── ID 唯一性
  │     ├── 字段范围检查
  │     ├── L0-L1 引用存在性检查
  │     ├── duration 字段合法性
  │     ├── ai_behavior 完整性
  │     ├── grid_occupation >= 1
  │     ├── max_per_summoner >= 1
  │     └── 层间依赖检查
  │
  ├── Register（注入 DefRegistry<SummonDef>）
  │
  └── Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | SummonId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `base_attributes` 非空 | 召唤物必须有至少一项基础属性 |
| V4 | `grid_occupation` >= 1 | 至少占用 1 格 |
| V5 | `max_per_summoner` >= 1 | 单召唤者最多可拥有的数量至少为 1 |
| V6 | `duration` 参数合法 | FixedTurns(t) ⇒ t >= 1，Concentration/Permanent 无需参数 |
| V7 | `icon_key` 和 `model_key` 引用存在 | 资源文件存在性检查 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V8 | `base_attributes` 中的每个 AttributeId 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V9 | `abilities` 中的每个 AbilityId 已注册 | 在 DefRegistry<AbilityDef> 中存在 |
| V10 | `spawn_buffs` 中的每个 BuffId 已注册 | 在 DefRegistry<BuffDef> 中存在 |
| V11 | `on_summon_effect`（如果设置）已注册 | 在 DefRegistry<EffectDef> 中存在 |
| V12 | `on_dismiss_effect`（如果设置）已注册 | 在 DefRegistry<EffectDef> 中存在 |
| V13 | `faction_override`（如果设置）已注册 | 在 DefRegistry<FactionDef> 中存在 |
| V14 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V15 | `dismissal_conditions` 中的每个 ConditionId（如果设置）已注册 | 在 DefRegistry<ConditionDef> 中存在 |

### 4.3 层间依赖校验

| # | 规则 | 说明 |
|---|------|------|
| V16 | SummonDef 不得引用任何 L3 Gameplay Def | 层间依赖方向规则 |
| V17 | SummonDef 不得引用任何 L4 World Def | 同上 |

### 4.4 语义校验

| # | 规则 | 说明 |
|---|------|------|
| V18 | `on_summon_effect` 的 duration 应为 Instant | 召唤出场效果应为瞬时 |
| V19 | `on_dismiss_effect` 的 duration 应为 Instant | 消失效果应为瞬时 |
| V20 | Concentration 类型的召唤物应有关联的 ConditionDef | 专注中断的条件应通过 dismissal_conditions 表达 |
| V21 | `abilities` 中的技能不应有冷却时间相关技能 | 召唤物技能冷却由 Summon 领域管理，与 AbilityDef 的冷却系统协调 |

---

## 5. RON 示例

```ron
(
    id: "sum:skeleton_warrior",
    name_key: "summon.sum_skeleton_warrior.name",
    description_key: "summon.sum_skeleton_warrior.desc",
    schema_version: 1,

    base_attributes: [
        ("attr:strength", 12.0),
        ("attr:dexterity", 10.0),
        ("attr:constitution", 14.0),
        ("attr:intelligence", 6.0),
        ("attr:wisdom", 8.0),
        ("attr:charisma", 4.0),
        ("attr:max_hp", 18.0),
        ("attr:armor_class", 12.0),
        ("attr:initiative", 0.0),
    ],

    abilities: [
        "ability:melee_attack_bone_sword",
    ],

    spawn_buffs: [
        "buff:undead_nature",
    ],

    on_summon_effect: Some("eff:skeleton_rise"),
    on_dismiss_effect: Some("eff:skeleton_crumble"),

    faction_override: None,

    tags: ["tag:undead", "tag:skeleton", "tag:summoned", "tag:medium_size"],

    ai_behavior: Guardian,

    duration: Concentration,

    dismissal_conditions: Some([
        "cond:summoner_unconscious",
        "cond:combat_ended",
    ]),

    grid_occupation: 1,

    max_per_summoner: 3,

    icon_key: Some("icons/summons/skeleton_warrior.png"),
    model_key: Some("models/summons/skeleton_warrior.glb"),
)
```

```ron
(
    id: "sum:fire_elemental",
    name_key: "summon.sum_fire_elemental.name",
    description_key: "summon.sum_fire_elemental.desc",
    schema_version: 1,

    base_attributes: [
        ("attr:strength", 16.0),
        ("attr:dexterity", 12.0),
        ("attr:constitution", 18.0),
        ("attr:intelligence", 8.0),
        ("attr:wisdom", 10.0),
        ("attr:charisma", 6.0),
        ("attr:max_hp", 45.0),
        ("attr:armor_class", 15.0),
        ("attr:initiative", 1.0),
    ],

    abilities: [
        "ability:melee_attack_fire_slam",
        "ability:fire_aura_passive",
    ],

    spawn_buffs: [
        "buff:fire_immunity",
        "buff:elemental_nature",
    ],

    on_summon_effect: Some("eff:fire_elemental_erupt"),
    on_dismiss_effect: Some("eff:fire_elemental_disperse"),

    faction_override: None,

    tags: ["tag:elemental", "tag:fire", "tag:summoned", "tag:large_size"],

    ai_behavior: Aggressive,

    duration: FixedTurns(5),

    dismissal_conditions: Some([
        "cond:combat_ended",
    ]),

    grid_occupation: 2,

    max_per_summoner: 1,

    icon_key: Some("icons/summons/fire_elemental.png"),
    model_key: Some("models/summons/fire_elemental.glb"),
)
```

---

## 6. 与 MonsterDef / CharacterDef 的关系

| 对比维度 | CharacterDef | MonsterDef | SummonDef |
|----------|-------------|------------|-----------|
| 核心用途 | 可操作角色模板 | 敌人模板 | 召唤物模板 |
| CreatureBase | 有（全量） | 有（全量） | 无（独立结构） |
| 装备系统 | 多槽位 | 扁平列表 | 无 |
| 职业系统 | L3 ProgressionDef | 无（CR） | 无 |
| 战利品 | 无 | L3 LootTableDef | 无 |
| 经验值 | 获得经验 | 提供经验 | 无 |
| 持续时间 | 永久 | 永久（直到死亡） | 有限（SummonDuration） |
| AI 行为 | 玩家控制 | AIBehaviorHints | SummonAIBehavior |
| 阵营 | 通常 Player | 通常 Hostile | 继承召唤者 |
| 数量限制 | 队伍上限 | Encounter 限制 | max_per_summoner |
| 技能来源 | 职业/等级 | 天生 | 模板定义 |

---

*本文档由 @content-architect 维护。*
