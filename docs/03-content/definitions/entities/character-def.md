---
id: 03-content.definitions.entities.character-def
title: CharacterDef — Character Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# CharacterDef — Character Content Def 定义

> **Content Layer**: L2 Entity | **领域规则**: `docs/02-domain/domains/party_domain.md` | **数据 Schema**: `docs/04-data/domains/party_schema.md` | **插件代码**: `src/content/plugins/character_plugin.rs`

---

## 1. Overview

CharacterDef 定义了可加入队伍的可操作角色模板——包括角色的基础属性、天赋技能、装备配置、阵营归属和元数据。CharacterDef 是"角色模板"而非角色实例：玩家队伍中的每个具体角色（如"Lv.5 的战士盖伦"）是运行时由此模板实例化的结果。

### 关键设计原则

- **模板而非实例**：CharacterDef 定义角色的先天属性（种族基础值、天生技能、阵营），不包含运行时状态（当前 HP、等级、经验值、装备实例）
- **Forward Reference 抑制**：`class_id` 字段引用 L3 ProgressionDef，但 L3 尚未定义。该字段在 L2 加载时被记录为 LazyRef，在 L3 就绪后二次解析
- **CreatureBase 模式**：CharacterDef 与 MonsterDef 共享 CreatureBase 字段集合——基础属性、天生 Ability/Buff、阵营、标签
- **装备槽位定义**：CharacterDef 本身定义角色可用的装备槽位集合（如人类默认 11 个 D&D 5e 槽位），但槽位中的具体装备实例由运行时管理

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `party_domain.md` | 队伍结构、羁绊系统、入场/退场规则 |
| `party_schema.md` | CharacterDef 的完整字段结构、EquipmentSlotConfig 定义 |
| `inventory_domain.md` | 物品实例的管理，CharacterDef 引用的需求 |
| `attribute-def.md` | 本 Def 的 `base_attributes` 键引用的 AttributeDef |
| `ability-def.md` | 本 Def 的 `innate_abilities` 引用的 AbilityDef |
| `buff-def.md` | 本 Def 的 `innate_buffs` 引用的 BuffDef |
| `effect-def.md` | 本 Def 的 `camp_rest_bonuses` 引用的 EffectDef |
| `trigger-def.md` | 本 Def 的 `innate_triggers` 引用的 TriggerDef |
| `condition-def.md` | 本 Def 的 `restrictions` 引用的 ConditionDef |
| `tag-def.md` | 本 Def 的 `tags` 引用的 TagDef |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// CreatureBase — 生物类 Def 的共享字段基座。
///
/// CharacterDef 和 MonsterDef 各自内联嵌入这些字段。
/// 这不是独立 Def，而是 Schema 层面的复用模式。
struct CreatureBase {
    // ── 统一标识字段 ──
    /// 全局唯一 ID（CharacterDef 前缀: `chr_`）
    pub id: CharacterId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号（用于未来迁移兼容）
    pub schema_version: u32,

    // ── 基础属性 ──
    /// 基础属性值列表（引用 L0 AttributeDef，值为初始值）
    pub base_attributes: Vec<(AttributeId, f32)>,

    /// 天生的技能列表（引用 L1 AbilityDef，角色出生即拥有）
    pub innate_abilities: Vec<AbilityId>,

    /// 天生的常驻 Buff 列表（引用 L1 BuffDef，角色出生即携带）
    pub innate_buffs: Vec<BuffId>,

    /// 天生的 Trigger 列表（引用 L1 TriggerDef，角色特质）
    pub innate_triggers: Vec<TriggerId>,

    // ── 阵营与分类 ──
    /// 阵营（引用 L0 FactionDef，可选）
    pub faction: Option<FactionId>,

    /// 标签列表（引用 L0 TagDef，种族/职业/体型等分类）
    pub tags: Vec<TagId>,

    // ── 表现资源 ──
    /// 肖像 Key（UI 头像）
    pub portrait_key: Option<String>,
    /// 模型 Key（3D 模型或 Spine 动画的资源路径）
    pub model_key: Option<String>,
}

/// 角色模板定义——描述一个可操作角色的静态属性。
///
/// CharacterDef 是 Content Asset，经 Load → Deserialize → Validate → Register → Freeze
/// 管线后进入 DefRegistry<CharacterDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct CharacterDef {
    // ── 嵌入 CreatureBase ──
    pub id: CharacterId,
    pub name_key: LocalizationKey,
    pub description_key: LocalizationKey,
    pub schema_version: u32,
    pub base_attributes: Vec<(AttributeId, f32)>,
    pub innate_abilities: Vec<AbilityId>,
    pub innate_buffs: Vec<BuffId>,
    pub innate_triggers: Vec<TriggerId>,
    pub faction: Option<FactionId>,
    pub tags: Vec<TagId>,
    pub portrait_key: Option<String>,
    pub model_key: Option<String>,

    // ── 角色特有字段 ──

    /// 职业/等级模板引用（Forward Reference 到 L3 ProgressionDef）
    ///
    /// 此项为 L3 层的 Forward Reference。加载 L2 时若 L3 尚未就绪，
    /// 此字段被记录为 LazyRef<CharacterId, ProgressionId>，在 L3 加载后二次解析。
    /// 当值 None 时角色为无职业模板（例如 NPC 角色、临时角色）。
    pub class_id: Option<ProgressionId>,

    /// 初始等级（仅当 class_id 有值时有效）
    pub starting_level: Option<u32>,

    // ── 装备槽位配置 ──
    /// 装备槽位配置——角色可装备的槽位类型列表
    ///
    /// 标准人类角色使用 Inventory Domain 定义的 11 个 D&D 5e 装备槽位。
    /// 非人形角色（如马、魔像）可以自定义槽位集合。
    pub equipment_slots: EquipmentSlotConfig,

    /// 出生时默认装备的装备 ID 列表（可选）
    pub starting_equipment: Option<Vec<EquipmentId>>,

    // ── 移动与战斗 ──
    /// 基础移动范围（格数）
    pub movement_range: u32,

    // ── 非战斗系统 ──
    /// 篝火休息时的额外效果（引用 L1 EffectDef）
    pub camp_rest_bonuses: Option<Vec<EffectId>>,

    // ── 使用限制 ──
    /// 角色使用限制（引用 L1 ConditionDef，如"仅限主线第3章后"）
    pub restrictions: Option<Vec<ConditionId>>,
}
```

### 内嵌数据结构

```rust
/// 装备槽位配置
#[derive(Deserialize, Clone, Debug)]
pub struct EquipmentSlotConfig {
    /// 槽位类型列表（顺序决定 UI 显示顺序）
    pub slots: Vec<EquipmentSlotType>,
    /// 是否禁用某些槽位（黑名单模式）
    pub disabled_slots: Option<Vec<EquipmentSlotType>>,
}

/// 装备槽位类型枚举（基于 D&D 5e/BG3 装备体系）
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum EquipmentSlotType {
    MainHand,
    OffHand,
    Helmet,
    Armor,
    Gloves,
    Boots,
    Cloak,
    Ring1,
    Ring2,
    Amulet,
    Special,
    /// 自定义槽位（Mod 扩展用）
    Custom(String),
}
```

### 字段说明

- **`class_id` / `starting_level`**: 两者成对出现。`class_id` 为 None 时角色无职业模板（视为基础单位）。当 L3 ProgressionDef 尚未定义时，此字段暂缓解析
- **`equipment_slots`**: 定义了角色"能够装备哪些槽位"，而非"当前装备了什么"。运行时装备状态属于 Inventory 领域的 ItemInstance
- **`starting_equipment`**: 引用 EquipmentDef，表示角色创建时自动获得的装备。与 `starting_level` 配合可实现"Lv.1 新兵装备"和"Lv.5 老兵装备"的差异化
- **`camp_rest_bonuses`**: 角色在篝火休息时可获得的额外增益，例如"精灵在休息时恢复额外 1d4 HP"
- **`restrictions`**: 角色使用限制，例如"仅限女角色"、"仅限主线第 3 章后解锁"等

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// CharacterDef 注册插件
pub struct CharacterDefPlugin;

impl Plugin for CharacterDefPlugin {
    fn build(&self, app: &mut App) {
        // 1. 注册 Asset 类型
        app.register_asset::<CharacterDef>();

        // 2. 注册 AssetLoader
        app.init_asset_loader::<RonAssetLoader<CharacterDef>>();

        // 3. 创建 DefRegistry 资源
        app.insert_resource(DefRegistry::<CharacterDef>::new());

        // 4. 注册加载/校验/注册管线
        app.add_systems(
            PreUpdate,
            load_character_defs
                .run_if(resource_changed::<Assets<CharacterDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 CharacterDef
pub fn get_character_def(
    character_id: &CharacterId,
    registry: &DefRegistry<CharacterDef>,
) -> Option<&CharacterDef> {
    registry.get(character_id)
}

/// 按标签过滤 CharacterDef
pub fn get_character_defs_by_tag(
    tag_id: &TagId,
    registry: &DefRegistry<CharacterDef>,
) -> Vec<&CharacterDef> {
    registry.iter().filter(|def| def.tags.iter().any(|t| t == tag_id)).collect()
}

/// 按阵营过滤 CharacterDef
pub fn get_character_defs_by_faction(
    faction_id: &FactionId,
    registry: &DefRegistry<CharacterDef>,
) -> Vec<&CharacterDef> {
    registry.iter().filter(|def| def.faction.as_ref() == Some(faction_id)).collect()
}
```

### DefRegistry 提供的能力

- `registry.get(id: &CharacterId) -> Option<&CharacterDef>` — 按 ID 精确查找
- `registry.iter() -> impl Iterator<Item = &CharacterDef>` — 遍历所有 Def
- `registry.count() -> usize` — 获取总数
- `registry.contains(id: &CharacterId) -> bool` — 判断是否存在
- `registry.dependencies(id: &CharacterId) -> Vec<DefDependency>` — 获取依赖关系
- `registry.freeze()` — 冻结注册表（加载完成后调用，禁止后续变更）

### 注册生命周期

```
CharacterDefPlugin::build
  │
  ├── CharacterDef 从 assets/config/02_entities/characters.ron 加载
  │
  ├── Deserialize (ron::from_str)
  │     └── 校验: RON 语法正确性、枚举合法性
  │
  ├── Validate
  │     ├── ID 唯一性检查
  │     ├── 引用存在性检查（L0-L1 Def 引用）
  │     ├── 字段合法性检查（base_attributes 非空, movement_range <= 最大, ...）
  │     ├── 依赖图循环检查
  │     ├── Forward Reference 标记（class_id 被记录但暂不校验）
  │     └── EquipmentSlotType 有效性（Custom 类型需注册）
  │
  ├── Register（注入 DefRegistry<CharacterDef>）
  │
  └── Freeze（管线完成后不可变）
```

### Forward Reference 二次校验

```rust
/// L2-L3 Forward Reference 二次校验系统。
/// 在 L3 加载完成后触发，重新校验所有 CharacterDef 的 class_id。
pub fn validate_forward_references(
    character_registry: Res<DefRegistry<CharacterDef>>,
    progression_registry: Res<DefRegistry<ProgressionDef>>,
    mut diagnostics: ResMut<ContentDiagnostics>,
) {
    for character in character_registry.iter() {
        if let Some(ref class_id) = character.class_id {
            if !progression_registry.contains(class_id) {
                diagnostics.report_error(
                    ContentError::UnresolvedForwardRef {
                        def_type: "CharacterDef",
                        def_id: character.id.clone(),
                        field: "class_id",
                        target_id: class_id.clone(),
                        target_layer: "L3 ProgressionDef",
                    }
                );
            }
        }
    }
}
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | CharacterId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1，不兼容版本拒绝加载 |
| V3 | `base_attributes` 非空 | 角色必须有至少一项基础属性 |
| V4 | `base_attributes` 无重复 | 同一个 AttributeId 不能出现两次 |
| V5 | `movement_range` 范围 | 1-99，默认 6 |
| V6 | `starting_level` >= 1（如果设置） | 初始等级至少为 1 |
| V7 | `portrait_key` 和 `model_key` 引用存在 | 引用的资源文件在 assets 中存在 |
| V8 | `equipment_slots.slots` 非空 | 角色必须至少有一个装备槽位 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V9 | `base_attributes` 中的每个 AttributeId 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V10 | `innate_abilities` 中的每个 AbilityId 已注册 | 在 DefRegistry<AbilityDef> 中存在 |
| V11 | `innate_buffs` 中的每个 BuffId 已注册 | 在 DefRegistry<BuffDef> 中存在 |
| V12 | `innate_triggers` 中的每个 TriggerId 已注册 | 在 DefRegistry<TriggerDef> 中存在 |
| V13 | `faction`（如果设置）已注册 | 在 DefRegistry<FactionDef> 中存在 |
| V14 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V15 | `starting_equipment` 中的每个 EquipmentId（如果设置）已注册 | 在 DefRegistry<EquipmentDef> 中存在 |
| V16 | `camp_rest_bonuses` 中的每个 EffectId（如果设置）已注册 | 在 DefRegistry<EffectDef> 中存在 |
| V17 | `restrictions` 中的每个 ConditionId（如果设置）已注册 | 在 DefRegistry<ConditionDef> 中存在 |

### 4.3 Forward Reference 校验

| # | 规则 | 说明 |
|---|------|------|
| V18 | `class_id`（如果设置）在 L2 加载时只记录不校验 | 标记为 Forward Reference，不阻塞 L2 加载 |
| V19 | L3 加载完成后触发二次校验 | 如上 `validate_forward_references` 系统 |

### 4.4 层间依赖校验

| # | 规则 | 说明 |
|---|------|------|
| V20 | CharacterDef 不得引用任何 L3 Gameplay Def | 除 `class_id`（Forward Reference 白名单）外 |
| V21 | CharacterDef 不得引用任何 L4 World Def | L2 → L4 的引用永远不允许 |
| V22 | CharacterDef 依赖图不得形成循环 | CharacterDef 之间禁止循环引用 |

### 4.5 语义校验

| # | 规则 | 说明 |
|---|------|------|
| V23 | `class_id` 和 `starting_level` 必须同时设置或同时不设置 | 职业和等级成对出现 |
| V24 | 双手武器槽位冲突警告 | 若角色有 MainHand 和 OffHand 槽位，标记双手武器兼容性 |
| V25 | 标签一致性检查 | 如 `tag:humanoid` 和 `tag:beast` 不应同时存在 |

---

## 5. RON 示例

```ron
(
    // ── CreatureBase ──
    id: "chr:fighter_mark",
    name_key: "character.chr_fighter_mark.name",
    description_key: "character.chr_fighter_mark.desc",
    schema_version: 1,

    base_attributes: [
        ("attr:strength", 16.0),
        ("attr:dexterity", 14.0),
        ("attr:constitution", 15.0),
        ("attr:intelligence", 10.0),
        ("attr:wisdom", 12.0),
        ("attr:charisma", 8.0),
        ("attr:max_hp", 55.0),
        ("attr:initiative", 2.0),
    ],

    innate_abilities: [
        "ability:second_wind",
        "ability:action_surge",
        "ability:melee_attack",
    ],

    innate_buffs: [
        "buff:fighting_style_defense",
    ],

    innate_triggers: [
        "trig:on_damage_taken_rally",
    ],

    faction: Some("faction:players"),
    tags: ["tag:humanoid", "tag:human", "tag:warrior", "tag:medium_size"],

    portrait_key: Some("portraits/human_male_fighter_01.png"),
    model_key: Some("models/characters/human_fighter.glb"),

    // ── CharacterDef 特有 ──
    class_id: Some("prog:fighter"),
    starting_level: Some(1),

    equipment_slots: (
        slots: [MainHand, OffHand, Helmet, Armor, Gloves, Boots, Cloak, Ring1, Ring2, Amulet],
        disabled_slots: None,
    ),

    starting_equipment: Some([
        "equip:longsword_iron",
        "equip:shield_wooden",
        "equip:chainmail",
    ]),

    movement_range: 6,

    camp_rest_bonuses: Some([
        "eff:rest_hp_recovery",
    ]),

    restrictions: None,
)
```

---

## 6. 与 MonsterDef 的关系

CharacterDef 和 MonsterDef 共享相同的 CreatureBase 字段结构。区别在于：

| 对比维度 | CharacterDef | MonsterDef |
|----------|-------------|------------|
| 核心用途 | 可操作角色 | 战斗中的敌人/中立生物 |
| 装备系统 | 多槽位装备系统 | 无或单槽位 |
| 职业系统 | 引用 L3 ProgressionDef | 无职业，用 DifficultyRating 表示强度 |
| 战利品 | 无（不可掉落） | 引用 L3 LootTableDef |
| AI 行为 | 玩家直接控制 | AI 行为提示定义 |
| 经验值 | 获得经验 | 提供经验（XP Reward） |
| 阵营 | 通常为玩家阵营 | 敌对/中立/可招募 |
| 使用角色 | Party 领域驱动 | Combat/Encounter 领域驱动 |

---

*本文档由 @content-architect 维护。*
