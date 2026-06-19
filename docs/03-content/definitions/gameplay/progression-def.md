---
id: 03-content.definitions.gameplay.progression-def
title: ProgressionDef — Progression Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# ProgressionDef — Progression Content Def 定义

> **Content Layer**: L3 Gameplay | **领域规则**: `docs/02-domain/domains/progression_domain.md` | **数据 Schema**: `docs/04-data/domains/progression_schema.md` | **插件代码**: `src/content/plugins/progression_plugin.rs`

---

## 1. Overview

ProgressionDef 定义了职业/等级成长模板——等级经验表、职业特性、子职选项、多职业要求。ProgressionDef 是 L2 CharacterDef 的 `class_id` 字段**前向引用**的目标 Def 类型，是 L3 加载管线中需优先处理的类型之一。

### 关键设计原则

- **职业而非角色**：ProgressionDef 定义职业的成长曲线（如"战士职业 1-20 级"），而非角色的成长实例。一个角色引用一个 ProgressionDef 作为其职业模板
- **Forward Reference 接收者**：L2 CharacterDef 的 `class_id` 字段指向 ProgressionDef。L3 加载完成后必须二次校验所有 CharacterDef 的 class_id 引用
- **L1 依赖**：职业特性通过引用 L1 AbilityDef/BuffDef/TriggerDef/ModifierDef 实现能力解锁，不将技能逻辑硬编码到 ProgressionDef 中
- **多职业支持**：通过 `multiclass_requirements` 定义兼职条件，Progression 领域的运行时管理多职业组合

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `progression_domain.md` | 等级成长体系、经验表、多职业规则、熟练加值 |
| `progression_schema.md` | ProgressionDef 完整字段结构、LevelDef、ClassFeature 定义 |
| `attribute-def.md` | 本 Def 的 `primary_attributes`、`saving_throw_proficiencies`、`multiclass_requirements` 引用的 AttributeDef |
| `ability-def.md` | 本 Def 的 `levels[].features[].AbilityRef` 引用的 AbilityDef |
| `buff-def.md` | 本 Def 的 `levels[].features[].BuffRef` 引用的 BuffDef |
| `trigger-def.md` | 本 Def 的 `levels[].features[].TriggerRef` 引用的 TriggerDef |
| `modifier-def.md` | 本 Def 的 `levels[].features[].ModifierRef` 引用的 ModifierDef |
| `condition-def.md` | 本 Def 的 `levels[].features[].FeatureChoice` 和 `multiclass_requirements` 引用的 ConditionDef |
| `tag-def.md` | 本 Def 的 `tags` 引用的 TagDef |
| `faction-def.md` | 本 Def 的 `class_type` 可选引用（阵营限定职业） |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 职业成长模板定义——描述一个职业从 1 级到最高级的完整成长曲线。
///
/// ProgressionDef 是 Content Asset，经 Load → Deserialize → Validate → Register → Freeze
/// 管线后进入 DefRegistry<ProgressionDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct ProgressionDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID（ProgressionDef 前缀: `prog_`）
    pub id: ProgressionId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 职业分类 ──
    /// 职业类型
    pub class_type: ClassType,

    // ── 基础属性 ──
    /// 生命骰（d6/d8/d10/d12）
    pub hit_die: u8,
    /// 主要属性（引用 L0 AttributeDef，等级提升时影响属性分配）
    pub primary_attributes: Vec<AttributeId>,
    /// 豁免熟练属性（引用 L0 AttributeDef）
    pub saving_throw_proficiencies: Vec<AttributeId>,

    // ── 等级定义 ──
    /// 每个等级的详细定义（1 级到最高级）
    pub levels: Vec<LevelDef>,

    // ── 子职 ──
    /// 选择子职的等级（0 = 无子职）
    pub subclass_level: u32,
    /// 子职选项列表（可选）
    pub subclass_options: Option<Vec<SubclassOption>>,

    // ── 多职业 ──
    /// 兼职该职业所需的属性条件
    pub multiclass_requirements: Vec<MulticlassRequirement>,

    // ── 元数据 ──
    /// 标签列表（引用 L0 TagDef）
    pub tags: Vec<TagId>,
    /// 职业图标 Key
    pub icon_key: Option<String>,
}

/// 职业类型枚举
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ClassType {
    /// 标准职业（如战士、法师、盗贼）
    Standard,
    /// 进阶职业（需要前置条件）
    Prestige,
    /// 种族限定职业（如"龙裔术士"）
    Racial,
    /// NPC 专用职业（不可供玩家选择）
    NPC,
    /// 自定义（Mod 扩展用）
    Custom(String),
}

/// 等级定义——每级的详细配置
#[derive(Deserialize, Clone, Debug)]
pub struct LevelDef {
    /// 等级编号（从 1 开始）
    pub level: u32,
    /// 达到此级所需的累积经验值
    pub xp_required: u64,
    /// 此等级时的熟练加值
    pub proficiency_bonus: u32,
    /// 升级时获得的职业特性列表
    pub features: Vec<ClassFeature>,
    /// 是否可获得属性值提升（ASI）
    pub asi: bool,
}

/// 职业特性——升级时解锁的能力/被动/资源
#[derive(Deserialize, Clone, Debug)]
pub struct ClassFeature {
    /// 特性名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 特性描述（本地化 Key）
    pub description_key: LocalizationKey,
    /// 特性类型
    pub feature_type: FeatureType,
    /// 该特性授予的能力（引用 L1 AbilityDef）
    pub granted_abilities: Option<Vec<AbilityId>>,
    /// 该特性授予的常驻 Buff（引用 L1 BuffDef）
    pub granted_buffs: Option<Vec<BuffId>>,
    /// 该特性授予的 Trigger（引用 L1 TriggerDef）
    pub granted_triggers: Option<Vec<TriggerId>>,
    /// 该特性授予的 Modifier（引用 L1 ModifierDef）
    pub granted_modifiers: Option<Vec<ModifierId>>,
    /// 选择选项（如"从以下 3 个专长中选择一个"）
    pub choices: Option<Vec<FeatureChoice>>,
}

/// 特性类型枚举
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FeatureType {
    /// 被动特性（自动生效）
    Passive,
    /// 主动能力（需要玩家激活）
    Active,
    /// 资源（如法术位、怒气、能量）
    Resource,
    /// 子职选择点
    SubclassChoice,
    /// 属性值提升（ASI）
    ASI,
}

/// 特性选择——升级时从多个选项中选择一个
#[derive(Deserialize, Clone, Debug)]
pub struct FeatureChoice {
    /// 选择类型
    pub choice_type: ChoiceType,
    /// 可选选项列表
    pub options: Vec<FeatureOption>,
    /// 最多可选数量（1 = 单选，>1 = 多选）
    pub max_selections: u32,
}

/// 选择类型
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ChoiceType {
    /// 选择能力（从多个 AbilityId 选一）
    Ability,
    /// 选择 Buff（从多个 BuffId 选一）
    Buff,
    /// 选择子职
    Subclass,
    /// 选择熟练项（引用 TagDef 表示的技能类型）
    Proficiency,
    /// 选择法术
    Spell,
}

/// 特性选项
#[derive(Deserialize, Clone, Debug)]
pub struct FeatureOption {
    /// 选项名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 选项描述（本地化 Key）
    pub description_key: LocalizationKey,
    /// 该选项授予的具体能力/效果
    pub grants: FeatureGrants,
}

/// 特性授予——选项选中后实际授予的内容
#[derive(Deserialize, Clone, Debug)]
pub enum FeatureGrants {
    /// 授予能力（引用 AbilityDef）
    AbilityRef(AbilityId),
    /// 授予常驻 Buff（引用 BuffDef）
    BuffRef(BuffId),
    /// 授予 Trigger（引用 TriggerDef）
    TriggerRef(TriggerId),
    /// 授予属性修正（引用 ModifierDef）
    ModifierRef(ModifierId),
    /// 子职选择标记
    SubclassChoice,
    /// 熟练项（引用 TagDef 表示的技能类型）
    SkillProficiency(TagId),
    /// 自定义条件（引用 ConditionDef）
    Custom(ConditionId),
}

/// 子职选项
#[derive(Deserialize, Clone, Debug)]
pub struct SubclassOption {
    /// 子职 ID（同层引用 ProgressionDef 自身？或内部定义？）
    /// 设计决策：子职使用独立的 ProgressionDef（共享主职业的等级表）
    pub subclass_id: ProgressionId,
    /// 子职名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 子职描述（本地化 Key）
    pub description_key: LocalizationKey,
}

/// 多职业要求——兼职此职业的最低属性条件
#[derive(Deserialize, Clone, Debug)]
pub struct MulticlassRequirement {
    /// 属性 ID（引用 L0 AttributeDef）
    pub attribute_id: AttributeId,
    /// 最低属性值
    pub minimum_value: u32,
}
```

### 字段说明

- **`hit_die`**: 生命骰面数。d6=6, d8=8, d10=10, d12=12。影响每级生命值成长
- **`levels`**: 等级列表从 1 到最高级（通常是 20）。`xp_required` 是累积值而非单级所需。缺失的等级（如只有 1, 5, 10, 15, 20）无效——必须是连续的
- **`subclass_level`**: 子职选择等级。0 = 该职业无子职。子职在主职业的 ProgressionDef 上叠加额外特性
- **`subclass_options`**: 子职使用独立的 ProgressionDef 定义。子职 ProgressionDef 的 `id` 通过 `subclass_id` 引用。子职等级表是主职业的子集
- **`asi`**: 每级布尔标记。`true` = 此级可获得属性值提升。ASI 的具体分配由 Progression 领域运行时管理
- **`multiclass_requirements`**: 空列表 = 该职业不可兼职。D&D 5e 规则中所有标准职业至少有一个属性要求

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// ProgressionDef 注册插件
pub struct ProgressionDefPlugin;

impl Plugin for ProgressionDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<ProgressionDef>();
        app.init_asset_loader::<RonAssetLoader<ProgressionDef>>();
        app.insert_resource(DefRegistry::<ProgressionDef>::new());
        app.add_systems(
            PreUpdate,
            load_progression_defs
                .run_if(resource_changed::<Assets<ProgressionDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按职业类型过滤 ProgressionDef
pub fn get_classes_by_type(
    class_type: ClassType,
    registry: &DefRegistry<ProgressionDef>,
) -> Vec<&ProgressionDef> {
    registry.iter()
        .filter(|def| def.class_type == class_type)
        .collect()
}

/// 获取职业的最大等级
pub fn get_max_level(
    progression_id: &ProgressionId,
    registry: &DefRegistry<ProgressionDef>,
) -> Option<u32> {
    registry.get(progression_id)
        .and_then(|def| def.levels.last().map(|lv| lv.level))
}
```

### 注册生命周期

```
ProgressionDefPlugin::build
  │
  ├── ProgressionDef 从 assets/config/03_gameplay/progression.ron 加载
  │
  ├── Deserialize → Validate → Register → Freeze
  │
  └── Validate 具体规则：
        ├── ID 唯一性
        ├── L0 (TagId, AttributeId) 引用存在性
        ├── L1 (AbilityId, BuffId, TriggerId, ModifierId, ConditionId) 引用存在性
        ├── L3 同层引用（subclass_options.subclass_id）存在性
        ├── levels 非空（至少 1 级）
        ├── levels 必须从 1 开始连续递增
        ├── xp_required 必须递增
        ├── hit_die 合法值（6, 8, 10, 12）
        ├── subclass_level 在 levels 范围内
        ├── L4 禁止引用检查
        └── 子职循环引用检测（subclass_id 不得指向自身或其子职指向自身）
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | ProgressionId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `levels` 非空 | 职业至少定义 1 级 |
| V4 | `levels` 必须从 1 开始连续递增 | 无跳跃等级 |
| V5 | `levels` 中的 `xp_required` 必须严格递增 | 每级经验需求递增 |
| V6 | `hit_die` 合法 | 只能为 6, 8, 10, 12 |
| V7 | `subclass_level` 在 levels 范围 | 子职选择等级不能超过职业最高级 |
| V8 | `class_type` 合法 | 必须匹配 ClassType 的已知变体 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V9 | `primary_attributes` 中的每个 AttributeId 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V10 | `saving_throw_proficiencies` 中的每个 AttributeId 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V11 | `levels[].features[].granted_abilities` 中的每个 AbilityId 已注册 | 在 DefRegistry<AbilityDef> 中存在 |
| V12 | `levels[].features[].granted_buffs` 中的每个 BuffId 已注册 | 在 DefRegistry<BuffDef> 中存在 |
| V13 | `levels[].features[].granted_triggers` 中的每个 TriggerId 已注册 | 在 DefRegistry<TriggerDef> 中存在 |
| V14 | `levels[].features[].granted_modifiers` 中的每个 ModifierId 已注册 | 在 DefRegistry<ModifierDef> 中存在 |
| V15 | `multiclass_requirements` 中的每个 AttributeId 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V16 | `subclass_options` 中的每个 ProgressionId（子职引用）已注册 | 在 DefRegistry<ProgressionDef> 中存在 |
| V17 | `features[].choices[].options[].grants` 中的引用均已注册 | 在对应 DefRegistry 中存在 |
| V18 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |

### 4.3 循环引用检测

| # | 规则 | 说明 |
|---|------|------|
| V19 | `subclass_id` 不得引用自身 | ProgressionDef A 的 subclass_id 不能是 A |
| V20 | 子职链不得形成循环 | A→B→A 的链式引用不允许 |

### 4.4 层间依赖校验

| # | 规则 | 说明 |
|---|------|------|
| V21 | ProgressionDef 不得引用任何 L4 World Def | 层间依赖方向规则 |

### 4.5 Forward Reference 校验（接收来自 L2 的引用）

| # | 规则 | 说明 |
|---|------|------|
| V22 | L3 加载完成后，二次校验所有 CharacterDef 的 `class_id` 引用 | 验证每个引用的 ProgressionId 在 DefRegistry<ProgressionDef> 中存在 |

### 4.6 语义校验

| # | 规则 | 说明 |
|---|------|------|
| V23 | `primary_attributes` 非空 | 职业必须声明至少 2 个主要属性 |
| V24 | `saving_throw_proficiencies` 非空 | 职业必须声明豁免熟练项 |
| V25 | ASI 等级与传统规则一致 | 建议 4, 8, 12, 16, 19 级设 asi: true |
| V26 | 熟练加值合理 | levels[0].proficiency_bonus 应为 2（1 级熟练 +2） |

---

## 5. RON 示例

```ron
(
    id: "prog:fighter",
    name_key: "progression.prog_fighter.name",
    description_key: "progression.prog_fighter.desc",
    schema_version: 1,

    class_type: Standard,

    hit_die: 10,
    primary_attributes: ["attr:strength", "attr:dexterity", "attr:constitution"],
    saving_throw_proficiencies: ["attr:strength", "attr:constitution"],

    levels: [
        (
            level: 1,
            xp_required: 0,
            proficiency_bonus: 2,
            features: [
                (
                    name_key: "progression.prog_fighter.feature.fighting_style.name",
                    description_key: "progression.prog_fighter.feature.fighting_style.desc",
                    feature_type: Active,
                    granted_abilities: None,
                    granted_buffs: None,
                    granted_triggers: None,
                    granted_modifiers: None,
                    choices: Some([
                        (
                            choice_type: Ability,
                            options: [
                                (name_key: "...", description_key: "...", grants: ModifierRef("mod:fighting_style_defense")),
                                (name_key: "...", description_key: "...", grants: ModifierRef("mod:fighting_style_great_weapon")),
                                (name_key: "...", description_key: "...", grants: ModifierRef("mod:fighting_style_dueling")),
                            ],
                            max_selections: 1,
                        ),
                    ]),
                ),
                (
                    name_key: "progression.prog_fighter.feature.second_wind.name",
                    description_key: "progression.prog_fighter.feature.second_wind.desc",
                    feature_type: Active,
                    granted_abilities: Some(["ability:fighter_second_wind"]),
                    granted_buffs: None,
                    granted_triggers: None,
                    granted_modifiers: None,
                    choices: None,
                ),
            ],
            asi: false,
        ),
        (
            level: 2,
            xp_required: 300,
            proficiency_bonus: 2,
            features: [
                (
                    name_key: "progression.prog_fighter.feature.action_surge.name",
                    description_key: "progression.prog_fighter.feature.action_surge.desc",
                    feature_type: Active,
                    granted_abilities: Some(["ability:fighter_action_surge"]),
                    granted_buffs: None,
                    granted_triggers: None,
                    granted_modifiers: None,
                    choices: None,
                ),
            ],
            asi: false,
        ),
        (
            level: 3,
            xp_required: 900,
            proficiency_bonus: 2,
            features: [
                (
                    name_key: "progression.prog_fighter.feature.subclass.name",
                    description_key: "progression.prog_fighter.feature.subclass.desc",
                    feature_type: SubclassChoice,
                    granted_abilities: None,
                    granted_buffs: None,
                    granted_triggers: None,
                    granted_modifiers: None,
                    choices: Some([
                        (
                            choice_type: Subclass,
                            options: [
                                (name_key: "...", description_key: "...", grants: SubclassChoice),
                                (name_key: "...", description_key: "...", grants: SubclassChoice),
                            ],
                            max_selections: 1,
                        ),
                    ]),
                ),
            ],
            asi: false,
        ),
        (
            level: 4,
            xp_required: 2700,
            proficiency_bonus: 2,
            features: [],
            asi: true,
        ),
    ],

    subclass_level: 3,
    subclass_options: Some([
        (
            subclass_id: "prog:fighter_champion",
            name_key: "progression.prog_fighter_champion.name",
            description_key: "progression.prog_fighter_champion.desc",
        ),
        (
            subclass_id: "prog:fighter_battlemaster",
            name_key: "progression.prog_fighter_battlemaster.name",
            description_key: "progression.prog_fighter_battlemaster.desc",
        ),
    ]),

    multiclass_requirements: [
        (attribute_id: "attr:strength", minimum_value: 13),
        (attribute_id: "attr:dexterity", minimum_value: 13),
    ],

    tags: ["tag:martial_class", "tag:warrior", "tag:player_class"],
    icon_key: Some("icons/classes/fighter.png"),
)
```

---

## 6. 与其他 L3 Def 的关系

| L3 Def | ProgressionDef 的关系 |
|--------|---------------------|
| QuestDef | 无直接引用。Progression 领域监听 QuestCompleted 事件发放经验 |
| EncounterDef | 无直接引用。Progression 领域监听 CombatEnded 事件发放战斗经验 |
| DifficultyDef | 无直接引用。Progression 不涉及难度系统 |

**L2-L3 Forward Reference 解析流程**：

```
Phase 3: Load L3 (Gameplay)
  │
  ├── 1. Load ProgressionDef（优先加载——被 L2 前向引用）
  │
  ├── 2. Load other L3 Defs
  │
  └── 3. 二次校验：遍历所有 CharacterDef，解析 class_id 引用
        ├── 每个 ProgressionId → 在 DefRegistry<ProgressionDef> 中查找
        └── 未找到 → ContentError::UnresolvedForwardRef
```

**子职模式**：

子职使用独立的 ProgressionDef 定义。主职业的 `subclass_level` 标记子职选择发生的等级，`subclass_options` 列出可选的子职。子职 ProgressionDef 与主职业共享等级表（主职业定义基础特性，子职定义额外特性）：

```
主职业: "prog:fighter" (levels 1-20, 基础特性)
  └── 子职: "prog:fighter_champion" (levels 3-20, 冠军之路特性)
  └── 子职: "prog:fighter_battlemaster" (levels 3-20, 战术大师特性)
```

子职 ProgressionDef 不是独立职业——它不能脱离主职业存在。子职的等级从 3 开始（因为 1-2 级是主职业基础），它的特性和主职业特性叠加。

---

*本文档由 @content-architect 维护。*
