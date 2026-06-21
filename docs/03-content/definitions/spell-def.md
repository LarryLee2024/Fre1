---
id: 03-content.definitions.spell-def
title: SpellDef — Spell Content Def 定义
status: draft
owner: content-architect
created: 2026-06-21
updated: 2026-06-21
---

# SpellDef — Spell Content Def 定义

> **Content Layer**: L1 Capability (Spell 域) | **领域规则**: `docs/02-domain/domains/spell_domain.md` | **数据 Schema**: `docs/04-data/domains/spell_schema.md` | **插件代码**: `src/content/plugins/spell_plugin.rs`

---

## 1. Overview

SpellDef 是 L1 Capability 层 Spell 域的 **Def 定义**——它是一个轻量包装层，将法术的魔法上下文（学派、环阶、组件、专注、升环）附加到对应的 AbilityDef 上。

### 核心定位

- **AbilityDef 的魔法包装**：SpellDef 不定义技能逻辑（消耗、冷却、目标选择、效果链）——这些由引用的 AbilityDef 提供。SpellDef 只描述"这是一个什么法术"
- **Spell 域的业务入口**：Spell 域系统通过 SpellDef → AbilityDef 的双层结构管理施法流程。SpellDef 提供施法前的约束检查（组件、法术位、专注），AbilityDef 提供激活后的效果执行
- **非组合终端**：SpellDef 不是组合终端——它不编排其他 Def，只引用 AbilityDef。AbilityDef 才是真正的组合终端

### 关键设计原则

- **SpellDef 不重复定义技能机制**：消耗、冷却、目标选择、效果链由 AbilityDef 定义。SpellDef 只定义魔法特定的上下文字段
- **Ability 是执行核心**：法术施放的最终效果执行通过 AbilityDef 的激活流程完成，不新建"施法执行管线"
- **Spell 域负责法术特有规则**：法术位检查、组件检查、专注管理、豁免检定——这些是 Spell 域的运行时逻辑，不写入 Def 定义
- **升环复用 Effect 引用**：升环施法时，不同环级映射到不同的 Effect 引用，统一通过 Effect 管线执行
- **双层校验**：SpellDef 本身校验法术特有字段；AbilityDef 的校验保持独立。SpellDef 加载时额外校验与引用 AbilityDef 的一致性

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `spell_domain.md` | 法术环阶体系、施法组件、专注规则、升环施法、豁免规则 |
| `spell_schema.md` | SpellDef、SpellSlotPool、Spellbook、Concentration 的数据结构 |
| `ability-def.md` | 本 Def 的 `ability_id` 字段引用的 AbilityDef 类型 |
| `effect-def.md` | 本 Def 的 `upcast_effects` 中引用的 EffectDefId 类型 |
| `tag-def.md` | 本 Def 的 `tags` 字段和 `school` 映射的标签类型 |
| `trigger-def.md` | 本 Def 施法可能引用的 Reaction 类型 Trigger |

### 依赖关系全景

```
SpellDef (AbilityDef 包装)
  ├──→ AbilityDef   (ability_id——底层能力引擎)
  │     └──→ (全 L1 Def 依赖链)
  │
  ├──→ EffectDef    (upcast_effects 升环效果引用)
  │     └──→ ExecutionDef, ModifierDef, StackingDef, CueDef (间接)
  │
  └──→ TagDef       (tags, school → TagId 映射)

SpellDef 不放宽 L1 层的引用限制：
  不引用 L0+（通过 AbilityDef → Effect → Modifier 间接）
  不引用 L2+（Entity/Gameplay/World 层）
```

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;
use std::collections::HashMap;

/// Spell 定义——AbilityDef 的魔法上下文包装。
///
/// SpellDef 将法术的魔法特性（学派、环阶、组件、专注、升环）
/// 附加到引用的 AbilityDef 上。施法时通过 ability_id 查找 AbilityDef，
/// 复用其生命周期管理和效果执行。
///
/// 经 Load → Deserialize → Validate → Register → Freeze
/// 管线后进入 DefRegistry<SpellDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct SpellDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID（前缀: "spl:"）
    pub id: SpellId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号（用于未来迁移兼容）
    pub schema_version: u32,

    // ── Ability 引用 ──
    /// 引用的 AbilityDef ID——法术的完整能力定义
    ///
    /// 法术的消耗、冷却、目标选择、效果链由该 AbilityDef 定义。
    /// 施法时 Spell 域先进行法术特有检查（组件/法术位/专注），
    /// 通过后委托给 AbilityDef 执行技能生命周期。
    pub ability_id: AbilityId,

    // ── 法术分类 ──
    /// 法术学派（防护/咒法/预言/附魔/塑能/幻术/死灵/变化）
    pub school: MagicSchool,

    /// 法术环阶（0 = 戏法, 1-9 = 法术环阶）
    pub spell_level: SpellLevel,

    // ── 施法组件 ──
    /// 施法组件需求（语言/姿势/材料）
    pub components: SpellComponents,

    /// 施法时间类型
    pub casting_time: CastingTime,

    /// 法术射程
    pub range: SpellRange,

    /// 法术持续时间
    pub duration: SpellDuration,

    // ── 法术机制 ──
    /// 是否需要专注
    ///
    /// true = 专注法术，施法者受伤时需进行体质豁免维持专注。
    /// 同一时间只能维持一个专注法术。
    pub requires_concentration: bool,

    /// 豁免类型（如不需要豁免则为 None）
    ///
    /// 目标抵抗法术效果时进行的属性检定类型。
    /// 法术豁免 DC = 8 + 熟练加值 + 施法属性调整值 + 其他加值。
    pub saving_throw: Option<AbilityType>,

    /// 是否可升环施法
    ///
    /// true = 消耗更高环级的法术位来施放此法术，获得增强效果。
    /// upcast_effects 提供各环级的额外效果定义。
    pub can_upcast: bool,

    /// 升环效果映射（环级 → 额外效果引用）
    ///
    /// 升环施法时，除了 AbilityDef 效果链中的基础效果外，
    /// 额外应用该环级对应的 EffectDef 列表。
    /// 从 spell_level + 1 开始，每级映射需连续不中断。
    pub upcast_effects: Vec<UpcastLevel>,

    // ── 学习条件（可选） ──
    /// 法术学习条件
    pub learn_requirements: Option<SpellLearnRequirements>,

    // ── 元数据 ──
    /// 法术标签（用于分类过滤和法术免疫检查）
    pub tags: Vec<TagId>,

    /// 法术元数据
    pub metadata: SpellMetadata,
}

// ═══════════════════════════════════════════
// 内嵌数据结构
// ═══════════════════════════════════════════

/// 法术学派
#[derive(Deserialize, Clone, Debug)]
pub enum MagicSchool {
    /// 防护学派（护甲、结界、防护）
    Abjuration,
    /// 咒法学派（召唤、传送到、造物）
    Conjuration,
    /// 预言学派（侦测、探知、预知）
    Divination,
    /// 附魔学派（魅惑、胁迫、心智控制）
    Enchantment,
    /// 塑能学派（元素能量、力场、光）
    Evocation,
    /// 幻术学派（幻影、隐形、虚假感知）
    Illusion,
    /// 死灵学派（亡灵、生命吸取、腐朽）
    Necromancy,
    /// 变化学派（变形、强化、改变物理特性）
    Transmutation,
}

/// 法术环阶
#[derive(Deserialize, Clone, Debug)]
pub enum SpellLevel {
    /// 0 环——戏法，不消耗法术位
    Cantrip,
    /// 1 环
    Level1,
    /// 2 环
    Level2,
    /// 3 环——关键分水岭，强度显著提升
    Level3,
    Level4,
    Level5,
    Level6,
    Level7,
    Level8,
    /// 9 环——最高环阶法术
    Level9,
}

/// 施法组件需求
#[derive(Deserialize, Clone, Debug)]
pub struct SpellComponents {
    /// 语言成分（V）：必须能说话才能施法。沉默状态禁止施法
    pub verbal: bool,

    /// 姿势成分（S）：必须能自由活动一只手。束缚状态禁止施法
    pub somatic: bool,

    /// 材料成分（M）：需要特定的施法材料或法器
    pub material: Option<MaterialComponent>,
}

/// 材料成分
#[derive(Deserialize, Clone, Debug)]
pub struct MaterialComponent {
    /// 材料描述（本地化 Key）
    pub description_key: LocalizationKey,
    /// 材料是否被消耗
    pub consumed: bool,
    /// 材料是否有金币价值要求（None = 无要求）
    pub cost_gold: Option<u32>,
}

/// 施法时间
#[derive(Deserialize, Clone, Debug)]
pub enum CastingTime {
    /// 1 个标准动作
    Action,
    /// 1 个附赠动作
    BonusAction,
    /// 反应（在特定时机触发，如护盾术）
    Reaction,
    /// 长施法时间（以分钟为单位）
    Longer { minutes: u32 },
}

/// 法术射程
#[derive(Deserialize, Clone, Debug)]
pub enum SpellRange {
    /// 自身
    Self_,
    /// 触碰
    Touch,
    /// 远程（基础射程 + 可选最大射程）
    Ranged { base: u32, max: Option<u32> },
    /// 半径（以自身或某点为中心）
    Radius { center: RangeCenter, radius: u32 },
    /// 锥形
    Cone { length: u32 },
    /// 线形
    Line { length: u32, width: u32 },
    /// 无限
    Unlimited,
    /// 特殊（由 AbilityDef 的 targeting 处理）
    Special,
}

/// 范围中心
#[derive(Deserialize, Clone, Debug)]
pub enum RangeCenter {
    /// 施法者自身
    Self_,
    /// 空间中的指定点
    Point,
}

/// 法术持续时间
#[derive(Deserialize, Clone, Debug)]
pub enum SpellDuration {
    /// 瞬时
    Instant,
    /// 专注（最多持续 max_turns 回合）
    Concentration { max_turns: u32 },
    /// 计时（固定回合数）
    Timed { turns: u32 },
    /// 永久
    Permanent,
}

/// 升环效果——高环施法时获得的额外效果
#[derive(Deserialize, Clone, Debug)]
pub struct UpcastLevel {
    /// 目标环级
    pub level: SpellLevel,
    /// 升环描述（本地化 Key）
    pub description_key: LocalizationKey,
    /// 升环新增/替换的 EffectDef 引用列表
    ///
    /// 这些效果在 AbilityDef.effect_chain 的基础上附加执行。
    /// 如果 upcast_replace 为 true，则替换同名效果而非附加。
    pub additional_effects: Vec<EffectId>,
    /// 是否替换基础效果（true = 替换，false = 附加）
    pub upcast_replace: bool,
    /// 升环时的额外消耗源和量（如"每升一环多消耗 1 法术位"的特殊规则）
    pub extra_cost_multiplier: Option<f32>,
}

/// 法术学习条件
#[derive(Deserialize, Clone, Debug)]
pub struct SpellLearnRequirements {
    /// 所需最低施法者等级
    pub min_caster_level: Option<u8>,
    /// 所需施法属性最低值
    pub min_spellcasting_ability: Option<AttributeId>,
    /// 所需前置法术（必须已习得的法术）
    pub prerequisite_spells: Option<Vec<SpellId>>,
    /// 所需职业标签
    pub required_class: Option<Vec<TagId>>,
    /// 学习消耗（金币）
    pub learn_cost_gold: Option<u32>,
}

/// 法术元数据
#[derive(Deserialize, Clone, Debug)]
pub struct SpellMetadata {
    /// 是否在法术书中显示
    pub visible: bool,
    /// 图标资源 Key
    pub icon_key: Option<String>,
    /// 施法动画 Key
    pub cast_animation: Option<String>,
    /// 法术书分类排序权重
    pub sort_order: u32,
    /// 是否为仪式法术（可仪式施法、不消耗法术位但增加施法时间）
    pub ritual: bool,
}

/// 外部引入的类型
// AbilityId — AbilityDef
// EffectId — EffectDef
// TagId — TagDef
// AttributeId — AttributeDef
// SpellId — 自引用
// SpellLevel — 本文件定义
// MagicSchool — 本文件定义
```

### 字段说明

- **`ability_id`**: 引用的 AbilityDef ID。这是 SpellDef 和 AbilityDef 之间的桥梁——SpellDef 不定义技能机制，通过此引用委派给 AbilityDef。验证规则确保引用的 AbilityDef 已注册且 AbilityType 兼容（施法类法术使用 Active 类型，受专注类法术使用 Concentration 激活）
- **`school`**: 法术学派决定法术的分类归属和可能受影响的专长/抗性。使用枚举类型而非 TagId，因为学派是法术的固有分类维度，数量固定为 8 个
- **`spell_level`**: 法术环阶决定消耗的法术位环级。0 环（Cantrip）不消耗法术位，1-9 环消耗对应环级的法术位。升环施法时使用更高环级的法术位
- **`components`**: 施法组件需求。至少一个组件必须为 true。戏法通常只需语言/姿势，高环法术常需要贵重材料
- **`range` 与 AbilityDef.targeting 的关系**: SpellDef.range 描述"法术规则书中的射程"，用于 Spell 域的施法范围检查（如"目标是否在法术射程内"）。AbilityDef.targeting.range 描述"实际目标选择范围"。两者在施法流程中先后使用——先检查 SpellDef.range，再执行 AbilityDef.targeting
- **`requires_concentration`**: 专注法术的核心标识。专注法术施放后占用专注槽，施法者受伤时需进行体质豁免维持
- **`can_upcast` + `upcast_effects`**: 升环施法机制。upcast_effects 的环级必须连续递增，从 spell_level + 1 开始填充
- **`casting_time`**: 覆盖 AbilityDef 的 ActivationType。Spell 域的施法时间专注处理具有法术特色的时间规则（如仪式施法增加 10 分钟）

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// SpellDef 注册插件
pub struct SpellDefPlugin;

impl Plugin for SpellDefPlugin {
    fn build(&self, app: &mut App) {
        // 1. 注册 Asset 类型
        app.register_asset::<SpellDef>();

        // 2. 注册 AssetLoader
        app.init_asset_loader::<RonAssetLoader<SpellDef>>();

        // 3. 创建 DefRegistry 资源
        app.insert_resource(DefRegistry::<SpellDef>::new());

        // 4. 注册加载/校验/注册管线
        app.add_systems(
            PreUpdate,
            load_spell_defs
                .run_if(resource_changed::<Assets<SpellDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 SpellDef
pub fn get_spell_def(id: &SpellId, registry: &DefRegistry<SpellDef>) -> Option<&SpellDef> {
    registry.get(id)
}

/// 按法术学派过滤
pub fn get_spells_by_school(
    school: MagicSchool,
    registry: &DefRegistry<SpellDef>,
) -> Vec<&SpellDef> {
    registry.iter()
        .filter(|def| matches!(&def.school, s if std::mem::discriminant(s) == std::mem::discriminant(&school)))
        .collect()
}

/// 按法术环阶过滤
pub fn get_spells_by_level(
    level: SpellLevel,
    registry: &DefRegistry<SpellDef>,
) -> Vec<&SpellDef> {
    registry.iter()
        .filter(|def| matches!(&def.spell_level, l if std::mem::discriminant(l) == std::mem::discriminant(&level)))
        .collect()
}
```

### DefRegistry 提供的能力

- `registry.get(id: &SpellId) -> Option<&SpellDef>` — 按 ID 精确查找
- `registry.iter() -> impl Iterator<Item = &SpellDef>` — 遍历所有 Def
- `registry.count() -> usize` — 获取总数
- `registry.contains(id: &SpellId) -> bool` — 判断是否存在
- `registry.dependencies(id: &SpellId) -> Vec<DefDependency>` — 获取依赖关系（跨层引用解析）
- `registry.freeze()` — 冻结注册表（加载完成后调用，禁止后续变更）

### 跨层引用解析

SpellDef 引用 AbilityDef + EffectDef，解析时递归展开：

```
SpellDef 注册时需解析：
  ├── Tier 1（直接引用）
  │     ability_id                    → DefRegistry<AbilityDef>
  │     upcast_effects[].additional_effects[] → DefRegistry<EffectDef>
  │     tags                          → DefRegistry<TagDef>
  │     learn_requirements.min_spellcasting_ability → DefRegistry<AttributeDef>
  │     learn_requirements.required_class → DefRegistry<TagDef>
  │
  └── Tier 2（通过 AbilityDef 间接引用，递归解析）
        AbilityDef:
          ├── activation_conditions → DefRegistry<ConditionDef>
          ├── triggers              → DefRegistry<TriggerDef>
          ├── effect_chain[].effect_def_id → DefRegistry<EffectDef>
          │     └── (EffectDef 的递归引用链)
          └── stacking              → DefRegistry<StackingDef>
```

### 加载顺序

SpellDefPlugin 必须在以下插件之后加载：

```rust
fn build_content_pipeline(app: &mut App) {
    // L0 Vocabulary
    app.add_plugins(TagDefPlugin);
    app.add_plugins(AttributeDefPlugin);

    // L1 Capability（先注册所有依赖）
    app.add_plugins(ConditionDefPlugin);
    app.add_plugins(ExecutionDefPlugin);
    app.add_plugins(CueDefPlugin);
    app.add_plugins(ModifierDefPlugin);
    app.add_plugins(EffectDefPlugin);
    app.add_plugins(TriggerDefPlugin);
    app.add_plugins(TargetingDefPlugin);
    app.add_plugins(StackingDefPlugin);

    // AbilityDef 作为 L1 组合终端
    app.add_plugins(AbilityDefPlugin);

    // SpellDef 引用 AbilityDef，在 AbilityDef 之后注册
    app.add_plugins(SpellDefPlugin);
}
```

### 注册生命周期

```
SpellDefPlugin::build
  │
  ├── SpellDef 从 assets/config/01_capabilities/spells.ron 加载
  │
  ├── Deserialize (ron::from_str)
  │     └── 校验: RON 语法正确性、枚举合法性（SpellLevel/MagicSchool/CastingTime 等）
  │
  ├── Validate
  │     ├── ID 唯一性检查
  │     ├── 字段级校验（环阶/组件/射程/专注等）
  │     ├── 引用存在性检查（ability_id → AbilityDef, additional_effects → EffectDef）
  │     ├── AbilityDef 兼容性检查（AbilityType 与法术特性一致）
  │     ├── 升环连续性检查
  │     ├── 组件有效检查（至少一个组件为 true）
  │     ├── 专注一致性检查
  │     └── 层间规则校验
  │
  ├── Register (注入 DefRegistry<SpellDef>)
  │
  └── Freeze (管线完成后不可变)
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | SpellId 不能为空字符串，格式应为 "spl:xxx" |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1，不兼容版本拒绝加载 |
| V3 | `spell_level` 合法 | 必须匹配 SpellLevel 的已知变体 |
| V4 | `school` 合法 | 必须匹配 MagicSchool 的已知变体 |
| V5 | `components` 至少一个为 true | 语言/姿势/材料至少一项为 true |
| V6 | `components.material` 非空时 `description_key` 非空 | 材料成分必须有描述 |
| V7 | `requires_concentration` = true 时 `duration` 必须为 `Concentration` 或 `Permanent` | 专注法术的持续时间类型必须匹配 |
| V8 | `can_upcast` = true 时 `upcast_effects` 非空 | 可升环法术必须有升环效果定义 |
| V9 | `upcast_effects` 中环级连续递增 | 从 `spell_level + 1` 开始，中间不能跳过 |
| V10 | `upcast_effects` 中环级不重复 | 同一环级不能有多个定义 |
| V11 | `upcast_effects[].level` > `spell_level` | 升环目标环级必须高于基础环级 |
| V12 | `learn_requirements.min_caster_level` >= 1（如果设置） | 最低施法者等级至少为 1 |
| V13 | `spell_level` 为 Cantrip 时 `can_upcast` 必须为 false | 戏法不能升环施法 |
| V14 | `casting_time` 为 Reaction 时 `spell_level` <= 5 | 反应法术通常不高于 5 环 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V15 | `ability_id` 已注册 | 在 DefRegistry<AbilityDef> 中存在 |
| V16 | `upcast_effects[].additional_effects` 中的每个 EffectId 已注册 | 在 DefRegistry<EffectDef> 中存在 |
| V17 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V18 | `learn_requirements.min_spellcasting_ability`（如果设置）中的 AttributeId 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V19 | `learn_requirements.required_class`（如果设置）中的 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V20 | `learn_requirements.prerequisite_spells`（如果设置）中的 SpellId 已注册 | 在 DefRegistry<SpellDef> 中存在 |

### 4.3 AbilityDef 兼容性校验

| # | 规则 | 说明 |
|---|------|------|
| V21 | `ability_id` 引用的 AbilityDef 不能是 Passive 类型 | 法术必须是可激活的技能。常驻能力不应建模为法术 |
| V22 | `ability_id` 引用的 AbilityDef 的 `ability_type` 与 `casting_time` 一致 | 瞬发法术使用 Instant 激活，需施法时间的法术使用 CastTime/Concentration |
| V23 | `requires_concentration` = true 时 AbilityDef 的 `ability_type` 必须使用 Concentration 激活 | 专注法术的激活方式必须匹配 |
| V24 | SpellDef 不放宽 L1 层引用限制 | 引用的 AbilityDef 和 EffectDef 均不得引用 L2+ Def |
| V25 | `spell_level` 为 Cantrip 时 AbilityDef 的 `costs` 应为空 | 戏法不应有消耗（法术位之外的材料消耗除外） |

### 4.4 升环连续性校验

| # | 规则 | 说明 |
|---|------|------|
| V26 | `upcast_effects` 按 `level` 升序排列 | 升环效果列表必须有序 |
| V27 | `upcast_effects` 的环级范围不超出 1-9 | 不能升环到 9 环以上 |
| V28 | `upcast_effects` 中 `upcast_replace` 为 true 时必须有 `additional_effects` | 替换模式不能为空 |
| V29 | 同一 SpellDef 不能在同一环级同时存在可升环和不可升环冲突 | `can_upcast` 一致性 |
| V30 | `extra_cost_multiplier`（如果设置）必须 > 0.0 | 消耗倍率必须为正数 |

### 4.5 依赖图校验

| # | 规则 | 说明 |
|---|------|------|
| V31 | SpellDef 不得通过 `prerequisite_spells` 形成循环 | A 需要 B 前置，B 需要 A 前置，不允许 |
| V32 | SpellDef 引用的 AbilityDef 在依赖图中不形成循环 | AbilityDef 依赖图递归校验无循环 |
| V33 | SpellDef 不得引用任何 L2+ Def | L1 层能力不可引用 Entity/Gameplay/World 层内容 |

### 4.6 与 AbilityDef 的冗余/冲突校验

| # | 规则 | 说明 |
|---|------|------|
| V34 | SpellDef 和 AbilityDef 的 `name_key` 不应冲突 | 同一法术的两个层次不应使用不同的名称 Key（可相同） |
| V35 | SpellDef 的锁定范围不应宽于 AbilityDef | 如果 SpellDef.range = Touch，AbilityDef.targeting 不应要求 > Touch 的距离 |
| V36 | 升环效果的 EffectDef 不重复于 AbilityDef 基础效果链 | 升环效果应提供"额外"而非"重复"的效果 |

### 4.7 内联结构校验

| # | 规则 | 说明 |
|---|------|------|
| V37 | `SpellComponents` 全 false 时警告 | 无任何组件的法术需要明确确认（属于特殊情况） |
| V38 | `material.consumed` = true 时 `cost_gold` 不应为 None | 消耗性材料通常具有金币价值 |
| V39 | `SpellRange::Ranged` 时 `base > 0` | 远程法术的基础射程必须为正 |
| V40 | `SpellRange::Radius` 时 `radius > 0` | 半径必须为正 |
| V41 | `SpellRange::Cone` 时 `length > 0` | 锥形长度必须为正 |
| V42 | `SpellRange::Line` 时 `length > 0 && width > 0` | 线形长度和宽度必须为正 |
| V43 | `CastingTime::Longer` 时 `minutes >= 1` | 长施法时间至少 1 分钟 |
| V44 | `metadata.sort_order` 范围 | 0-1000，默认 500 |
| V45 | `SpellDuration::Concentration` 时 `max_turns` >= 1 | 专注最大回合数至少为 1 |
| V46 | `SpellDuration::Timed` 时 `turns` >= 1 | 计时持续时间至少为 1 回合 |

---

## 5. RON 示例

### 示例：火球术（3 环 / 塑能 / AOE 火焰伤害）

```ron
// SpellDef 示例：火球术
//
// 3 环塑能法术，以施法者指定点为中心产生 20 英尺半径的火焰爆炸。
// 依赖的 AbilityDef: abl:fireball（定义消耗、冷却、目标选择、效果链）
// 依赖的 EffectDef: eff:fireball_explosion、eff:fireball_upcast_4 等
//
// 火球术的 RON 文件与 abl:fireball 的 AbilityDef RON 文件互补：
// - abl:fireball 定义"这个火球技能怎么执行"
// - 本文件定义"这个火球法术有什么魔法属性"

(
    id: "spl:fireball",
    name_key: "spell.spl_fireball.name",
    description_key: "spell.spl_fireball.desc",
    schema_version: 1,

    // 引用 abl:fireball 作为底层能力引擎
    ability_id: "abl:fireball",

    // 3 环塑能法术
    school: Evocation,
    spell_level: Level3,

    // 施法组件：语言 + 姿势 + 材料（蝙蝠粪和硫磺）
    components: (
        verbal: true,
        somatic: true,
        material: Some((
            description_key: "spell.spl_fireball.material",
            consumed: false,
            cost_gold: None,
        )),
    ),

    // 施法时间：1 个标准动作
    casting_time: Action,

    // 射程：150 英尺
    range: Ranged(
        base: 150,
        max: None,
    ),

    // 持续时间：瞬时
    duration: Instant,

    // 不需要专注
    requires_concentration: false,

    // 敏捷豁免（减半伤害）
    saving_throw: Some(Dexterity),

    // 可升环施法（3 环以上每升一环 +2d6 伤害）
    can_upcast: true,
    upcast_effects: [
        (
            level: Level4,
            description_key: "spell.spl_fireball.upcast.4",
            additional_effects: ["eff:fireball_upcast_d6"],
            upcast_replace: false,
            extra_cost_multiplier: None,
        ),
        (
            level: Level5,
            description_key: "spell.spl_fireball.upcast.5",
            additional_effects: ["eff:fireball_upcast_2d6"],
            upcast_replace: false,
            extra_cost_multiplier: None,
        ),
    ],

    // 学习条件：5 级施法者
    learn_requirements: Some((
        min_caster_level: Some(5),
        min_spellcasting_ability: None,
        prerequisite_spells: Some(["spl:burning_hands"]),
        required_class: Some(["tag:class_wizard", "tag:class_sorcerer"]),
        learn_cost_gold: Some(500),
    )),

    // 标签
    tags: ["tag:damage_type_fire", "tag:school_evocation", "tag:spell_level_3"],

    // 元数据
    metadata: (
        visible: true,
        icon_key: Some("icon_spell_fireball"),
        cast_animation: Some("anim_cast_fireball"),
        sort_order: 300,
        ritual: false,
    ),
)
```

### 示例：法师之手（0 环戏法 / 咒法）

```ron
// SpellDef 示例：法师之手
//
// 0 环咒法戏法，创造一个幽灵手进行简单交互（无消耗、可随意施放）。
// 戏法没有消耗、没有升环、没有法术位要求。
(
    id: "spl:mage_hand",
    name_key: "spell.spl_mage_hand.name",
    description_key: "spell.spl_mage_hand.desc",
    schema_version: 1,

    // 依赖的 AbilityDef 定义技能的具体效果（创建可操控的召唤物）
    ability_id: "abl:mage_hand",

    // 0 环咒法戏法
    school: Conjuration,
    spell_level: Cantrip,

    // 施法组件：语言 + 姿势
    components: (
        verbal: true,
        somatic: true,
        material: None,
    ),

    // 施法时间：1 个标准动作
    casting_time: Action,

    // 射程：30 英尺
    range: Ranged(
        base: 30,
        max: None,
    ),

    // 持续时间：1 分钟
    duration: Timed(
        turns: 10,
    ),

    // 不需要专注
    requires_concentration: false,

    // 无豁免
    saving_throw: None,

    // 戏法不能升环
    can_upcast: false,
    upcast_effects: [],

    // 学习条件：有施法能力的职业均可习得
    learn_requirements: None,

    tags: ["tag:school_conjuration", "tag:cantrip", "tag:utility"],

    metadata: (
        visible: true,
        icon_key: Some("icon_spell_mage_hand"),
        cast_animation: Some("anim_cast_mage_hand"),
        sort_order: 100,
        ritual: false,
    ),
)
```

### 示例：护盾术（1 环 / 防护 / 反应施法）

```ron
// SpellDef 示例：护盾术
//
// 1 环防护法术，反应施法——被攻击时瞬间提升 AC +5 直到下回合。
// CastingTime::Reaction 表明这是一个反应法术，由 Trigger 触发。
(
    id: "spl:shield",
    name_key: "spell.spl_shield.name",
    description_key: "spell.spl_shield.desc",
    schema_version: 1,

    ability_id: "abl:shield",

    school: Abjuration,
    spell_level: Level1,

    // 施法组件：语言 + 姿势
    components: (
        verbal: true,
        somatic: true,
        material: None,
    ),

    // 反应施法（被攻击时触发）
    casting_time: Reaction,

    // 射程：自身
    range: Self_,

    // 持续 1 回合
    duration: Timed(
        turns: 1,
    ),

    // 不需要专注（瞬时防御法术）
    requires_concentration: false,

    // 无豁免（对自身生效）
    saving_throw: None,

    can_upcast: false,
    upcast_effects: [],

    tags: ["tag:school_abjuration", "tag:spell_level_1", "tag:reaction"],

    metadata: (
        visible: true,
        icon_key: Some("icon_spell_shield"),
        cast_animation: Some("anim_cast_shield"),
        sort_order: 150,
        ritual: false,
    ),
)
```

---

## 6. 与 AbilityDef 的关系

SpellDef 和 AbilityDef 之间是**引用包装**关系，不是继承关系：

```
SpellDef (法术定义)
  │
  ├── 法术特有数据: school, spell_level, components, casting_time,
  │                 range, duration, requires_concentration,
  │                 saving_throw, can_upcast, upcast_effects
  │
  └── ability_id ──→ AbilityDef (能力定义)
                        │
                        ├── costs, cooldown
                        ├── targeting
                        ├── activation_conditions
                        ├── effect_chain
                        ├── level_scaling
                        └── restrictions
```

### 职责划分

| 职责 | 归属 | 理由 |
|------|------|------|
| 法术的魔法分类（学派/环阶） | SpellDef | 仅法术有，非法术能力不需要 |
| 消耗（法术位等价物） | AbilityDef.costs | 消耗统一通过 Attribute + Effect 机制处理 |
| 冷却（施法频率） | AbilityDef.cooldown | 冷却统一通过 Tag + Effect 机制处理 |
| 效果执行（伤害/治疗/Buff） | AbilityDef.effect_chain | 效果统一通过 Effect 管线，法术不另造执行链 |
| 升环增强 | SpellDef.upcast_effects | 仅法术有升环机制，非法术能力不需要 |
| 组件检查（V/S/M） | SpellDef.components | 仅法术有组件概念 |
| 专注管理 | SpellDef.requires_concentration | 专注是法术特有机制 |
| 豁免检定 | SpellDef.saving_throw | 豁免是法术/效果特有的对抗机制 |
| 目标选择 | AbilityDef.targeting | 目标选择是通用能力流程 |

### 运行时协作流程

```
施法请求
  │
  ├── Spell 域
  │     ├── 1. 检查法术位 (SpellSlotPool)
  │     ├── 2. 检查组件 (SpellComponents)
  │     ├── 3. 检查专注 (Concentration)
  │     ├── 4. 检查施法时间 → 占用动作
  │     └── 5. 计算豁免 DC
  │
  ├── 委托给 Ability 域
  │     ├── 6. AbilityDef.activation_conditions 检查
  │     ├── 7. AbilityDef.costs 消耗
  │     ├── 8. AbilityDef.targeting 目标选择 (或使用法术预选目标)
  │     └── 9. AbilityDef.effect_chain 执行
  │
  └── Spell 域收尾
        ├── 10. 消耗法术位 (SpellSlotPool.consume)
        ├── 11. 建立专注 (如需要)
        └── 12. 发布 SpellCast 事件
```

### 对比总结

| 对比维度 | AbilityDef | SpellDef |
|----------|-----------|----------|
| 本质 | 通用能力编排容器 | AbilityDef 的魔法上下文包装 |
| 是否引用其他 Def | 是——引用所有 L1 Def | 是——引用 AbilityDef + EffectDef |
| 是否可独立注册 | 是 | 是（但必须引用已注册的 AbilityDef） |
| 是否 L1 组合终端 | 是 | 否（AbilityDef 才是组合终端） |
| 运行时实例 | AbilityInstance（ECS Component） | 无（施法遵循 AbilityInstance 生命周期） |
| 存储位置 | `abilities.ron` | `spells.ron` |
| ID 前缀 | `abl:` | `spl:` |

---

## 7. 设计决策记录

| # | 决策 | 理由 |
|---|------|------|
| D1 | **SpellDef 引用 AbilityDef 而非继承/内联** | 职责分离——SpellDef 只描述"魔法属性"，AbilityDef 描述"技能机制"。非法术能力（战技、天赋）不需要 SpellDef |
| D2 | **MagicSchool 使用枚举而非 TagId** | 法术学派是 8 个固定的固有分类，枚举提供编译期穷尽检查和模式匹配，比 TagId 更强类型安全 |
| D3 | **SpellLevel 使用枚举而非 u8** | 环阶是离散的 10 个级别（0-9），枚举明确禁止非法值（如 10 环、-1 环）并提供语义化命名 |
| D4 | **range 和 duration 在 SpellDef 中独立定义** | 法术规则书的射程/时长描述（如"150 英尺""持续 1 分钟"）与 AbilityDef 的网格/回合数值可能不同。两者并存用于不同的校验上下文 |
| D5 | **upcast_effects 使用 Vec<UpcastLevel> 而非 HashMap<SpellLevel, ...>** | 照顾 RON 配置的序列化——RON 的枚举 Key 序列化较复杂，列表形式更自然。验证规则强制环级连续递增 |
| D6 | **不创建 SpellInstance ECS Component** | 施法效果复用 AbilityInstance 生命周期。Spell 域只管理法术位、专注、法术书等"法术特有状态"，不管理"技能执行状态" |
| D7 | **没有将 saves (豁免) 建模为自包含的 Schema** | 豁免使用已有的 AbilityType 枚举（Strength/Dexterity/Constitution/Intelligence/Wisdom/Charisma），不另建数据类型。相关领域规则在 spell_domain.md 中定义 |
| D8 | **SpellId 使用前缀 "spl:"** | 统一 Def ID 命名规范，与 AbilityDef（abl:）、EffectDef（eff:）等其他 Def ID 格式一致 |

---

## 8. 与 AbilityDef 的协作示例

### 数据流示例：火球术施法

```
玩家/AI 选择施放"火球术"
  │
  ├── 1. Spell 域通过 "spl:fireball" 查 DefRegistry<SpellDef>
  │      ├── spell_level = Level3 → 需要 3 环法术位
  │      ├── components = V+S+M → 检查沉默/束缚/材料
  │      ├── casting_time = Action → 消耗标准动作
  │      ├── requires_concentration = false → 无需专注检查
  │      ├── saving_throw = Some(Dexterity) → 准备计算豁免 DC
  │      └── ability_id = "abl:fireball" → 获取底层能力定义
  │
  ├── 2. Spell 域检查法术位 (SpellSlotPool)
  │      └── Level3 有可用法术位 → 继续
  │
  ├── 3. 委托给 Ability 域
  │      ├── 获取 DefRegistry<AbilityDef> 中的 "abl:fireball"
  │      ├── 检查 activation_conditions
  │      ├── 消耗 costs（30 MP、1 个法术位（Spell 域消耗））
  │      ├── 执行 targeting（Enemy, Area(radius=2), range=8）
  │      └── 执行 effect_chain（火焰爆发伤害 → 可能的灼烧 DOT）
  │
  └── 4. Spell 域收尾
         ├── 发布 SpellCast 事件
         ├── 消耗法术位（如果升环则消耗目标环级法术位）
         └── 更新 UI（法术位变化、施法动画）
```
