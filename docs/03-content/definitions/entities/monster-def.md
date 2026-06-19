---
id: 03-content.definitions.entities.monster-def
title: MonsterDef — Monster Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# MonsterDef — Monster Content Def 定义

> **Content Layer**: L2 Entity | **领域规则**: `docs/02-domain/domains/combat_domain.md` (Encounter 部分) | **数据 Schema**: `docs/04-data/domains/combat_schema.md` | **插件代码**: `src/content/plugins/monster_plugin.rs`

---

## 1. Overview

MonsterDef 定义了战斗中出现的非玩家实体模板（敌人、中立生物、BOSS）。MonsterDef 与 CharacterDef 共享 CreatureBase 字段结构（基础属性、天生 Ability/Buff、阵营、标签），但在装备、职业、战利品和行为方面存在本质差异。

### 关键设计原则

- **CreatureBase 共享**：MonsterDef 使用与 CharacterDef 相同的 CreatureBase 字段集合，确保生物类 Def 的一致性
- **简化装备模型**：怪物通常没有多槽位装备系统。MonsterDef 用 `equipment_override` 字段简化表示——要么无装备，要么直接引用少量 EquipmentDef
- **LootTable Forward Reference**：`loot_table` 引用 L3 LootTableDef，标记为 Forward Reference
- **AI 行为提示**：`ai_behavior_hints` 字段不定义 AI 逻辑，只给出行为倾向提示（攻击优先级、技能使用倾向、移动策略），实际 AI 决策由 Combat 领域的 AI 系统执行

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `combat_domain.md` | Encounter 组成、怪物行为模式、难度评估 |
| `combat_schema.md` | MonsterDef 完整字段结构、AIBehaviorHints 定义 |
| `inventory_schema.md` | LootTableDef 的前向引用 |
| `attribute-def.md` | 本 Def 的 `base_attributes` 键引用的 AttributeDef |
| `ability-def.md` | 本 Def 的 `innate_abilities` 引用的 AbilityDef |
| `buff-def.md` | 本 Def 的 `innate_buffs` 引用的 BuffDef |
| `trigger-def.md` | 本 Def 的 `innate_triggers` 引用的 TriggerDef |
| `condition-def.md` | 本 Def 的 `spawn_conditions` 引用的 ConditionDef |
| `tag-def.md` | 本 Def 的 `tags` 引用的 TagDef |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 怪物模板定义——描述一个非玩家战斗实体的静态属性。
///
/// MonsterDef 是 Content Asset，经 Load → Deserialize → Validate → Register → Freeze
/// 管线后进入 DefRegistry<MonsterDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct MonsterDef {
    // ── 嵌入 CreatureBase ──
    /// 全局唯一 ID（MonsterDef 前缀: `mob_`）
    pub id: MonsterId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号（用于未来迁移兼容）
    pub schema_version: u32,

    /// 基础属性值列表（引用 L0 AttributeDef，值为初始值）
    pub base_attributes: Vec<(AttributeId, f32)>,

    /// 天生技能列表（引用 L1 AbilityDef）
    pub innate_abilities: Vec<AbilityId>,

    /// 天生的常驻 Buff 列表（引用 L1 BuffDef）
    pub innate_buffs: Vec<BuffId>,

    /// 天生的 Trigger 列表（引用 L1 TriggerDef）
    pub innate_triggers: Vec<TriggerId>,

    /// 阵营（引用 L0 FactionDef，可选）
    pub faction: Option<FactionId>,

    /// 标签列表（引用 L0 TagDef，种族/体型/元素属性等）
    pub tags: Vec<TagId>,

    /// 肖像 Key（UI 头像）
    pub portrait_key: Option<String>,

    /// 模型 Key（3D 模型或 Spine 动画的资源路径）
    pub model_key: Option<String>,

    // ── 怪物特有字段 ──

    // ── 难度与奖励 ──
    /// 难度等级（Challenge Rating，用于 Encounter 难度平衡）
    pub difficulty_rating: u32,

    /// 击败后获得的经验值奖励
    pub xp_reward: u32,

    // ── 战利品（Forward Reference 到 L3） ──
    /// 战利品表引用（Forward Reference 到 L3 LootTableDef）
    ///
    /// 此项为 L3 层的 Forward Reference。加载 L2 时若 L3 尚未就绪，
    /// 此字段被记录为 LazyRef<MonsterId, LootTableId>，在 L3 加载后二次解析。
    pub loot_table: Option<LootTableId>,

    // ── 装备（简化模型） ──
    /// 怪物可装备的装备 ID 列表（可选，简化多槽位模型）
    ///
    /// 与 CharacterDef 的多槽位系统不同，MonsterDef 采用扁平的装备列表。
    /// 怪物要么没有装备（大部分），要么有少量预设装备（Boss/人形怪）。
    pub equipment_override: Option<Vec<EquipmentId>>,

    // ── AI 行为 ──
    /// AI 行为提示——定义怪物的战斗行为倾向
    ///
    /// 注意：这仅定义提示/倾向，AI 的具体决策逻辑属于 Combat 领域。
    pub ai_behavior_hints: AIBehaviorHints,

    // ── 移动与战斗 ──
    /// 基础移动范围（格数）
    pub movement_range: u32,

    // ── 生成条件 ──
    /// 怪物生成条件（引用 L1 ConditionDef，如"仅夜间生成"、"仅特定地形"）
    pub spawn_conditions: Option<Vec<ConditionId>>,

    // ── 规模与占位 ──
    /// 体型占位（标准 1x1、大型 2x2、超大型 3x3 等）
    pub size_occupation: GridOccupation,
}
```

### 内嵌数据结构

```rust
/// AI 行为提示——不定义 AI 逻辑，只给出倾向性提示
#[derive(Deserialize, Clone, Debug)]
pub struct AIBehaviorHints {
    /// 战斗风格（攻击性/防守性/策略性/逃跑倾向）
    pub combat_style: CombatStyle,
    /// 目标选择优先级（最近/最弱/最危险/特定标签优先）
    pub target_priority: TargetPriority,
    /// 技能使用策略（按冷却/按情况/随机）
    pub ability_usage: AbilityUsageStrategy,
    /// 移动策略（冲锋/迂回/保持距离/固守）
    pub movement_strategy: MovementStrategy,
    /// 是否在特定条件下呼叫增援
    pub can_call_reinforcements: bool,
    /// 自定义行为标签（扩展用）
    pub custom_behaviors: Option<Vec<TagId>>,
}

/// 战斗风格枚举
#[derive(Deserialize, Clone, Debug)]
pub enum CombatStyle {
    Aggressive,
    Defensive,
    Tactical,
    Cautious,
    Retreating,
    Custom(String),
}

/// 目标优先级枚举
#[derive(Deserialize, Clone, Debug)]
pub enum TargetPriority {
    Nearest,
    Weakest,
    MostDangerous,
    Specific(TagId),
    Random,
}

/// 技能使用策略枚举
#[derive(Deserialize, Clone, Debug)]
pub enum AbilityUsageStrategy {
    OnCooldown,
    Situational,
    Random,
    Scripted(Vec<ScriptedAbilityStep>),
}

/// 移动策略枚举
#[derive(Deserialize, Clone, Debug)]
pub enum MovementStrategy {
    Charge,
    Flank,
    KeepDistance,
    HoldPosition,
    Patrol,
    Random,
    Custom(String),
}

/// 脚本化的技能使用步骤（用于 Boss 战）
#[derive(Deserialize, Clone, Debug)]
pub struct ScriptedAbilityStep {
    pub ability_id: AbilityId,
    pub trigger_condition: Option<ConditionId>,
    pub priority: u8,
    pub phase: Option<u32>,
}

/// 网格占位枚举
#[derive(Deserialize, Clone, Debug)]
pub enum GridOccupation {
    Small,    // 1x1
    Medium,   // 1x1 (标准)
    Large,    // 2x2
    Huge,     // 3x3
    Gargantuan, // 4x4
    Custom(u32, u32),
}
```

### 字段说明

- **`difficulty_rating`**: 挑战等级（CR），用于 Encounter 系统计算战斗难度。CR 0-30，参考 D&D 5e CR 体系
- **`ai_behavior_hints`**: 只定义倾向，不定义具体 AI 逻辑。AI 系统读取这些 hint 作为决策输入参数，但最终决策逻辑仍由 AI 系统自身完成
- **`equipment_override`**: 与 CharacterDef 的多槽位系统不同，怪物采用扁平列表。人形怪（如 Goblin）可能有简单装备，而野兽（如 Wolf）通常无装备
- **`size_occupation`**: 定义怪物在战术地图上占用的网格数量，影响碰撞检测和移动

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// MonsterDef 注册插件
pub struct MonsterDefPlugin;

impl Plugin for MonsterDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<MonsterDef>();
        app.init_asset_loader::<RonAssetLoader<MonsterDef>>();
        app.insert_resource(DefRegistry::<MonsterDef>::new());
        app.add_systems(
            PreUpdate,
            load_monster_defs
                .run_if(resource_changed::<Assets<MonsterDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按 ID 查找 MonsterDef
pub fn get_monster_def(
    monster_id: &MonsterId,
    registry: &DefRegistry<MonsterDef>,
) -> Option<&MonsterDef> {
    registry.get(monster_id)
}

/// 按标签过滤 MonsterDef
pub fn get_monster_defs_by_tag(
    tag_id: &TagId,
    registry: &DefRegistry<MonsterDef>,
) -> Vec<&MonsterDef> {
    registry.iter()
        .filter(|def| def.tags.iter().any(|t| t == tag_id))
        .collect()
}

/// 按难度范围过滤 MonsterDef
pub fn get_monster_defs_by_cr_range(
    min_cr: u32,
    max_cr: u32,
    registry: &DefRegistry<MonsterDef>,
) -> Vec<&MonsterDef> {
    registry.iter()
        .filter(|def| def.difficulty_rating >= min_cr && def.difficulty_rating <= max_cr)
        .collect()
}
```

### 注册生命周期

```
MonsterDefPlugin::build
  │
  ├── MonsterDef 从 assets/config/02_entities/monsters.ron 加载
  │
  ├── Deserialize → Validate → Register → Freeze
  │
  └── Validate 具体规则：
        ├── ID 唯一性
        ├── L0-L1 引用存在性检查（同 CharacterDef）
        ├── difficulty_rating 范围 0-30
        ├── movement_range 范围 1-99
        ├── xp_reward 合理性检查
        ├── ai_behavior_hints 字段完整性
        ├── loot_table 标记为 Forward Reference
        └── spawn_conditions 引用存在性检查
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | MonsterId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `base_attributes` 非空 | 怪物必须有至少一项基础属性 |
| V4 | `difficulty_rating` 范围 | 0-30，默认 1 |
| V5 | `xp_reward` >= 0 | 经验值奖励必须非负 |
| V6 | `movement_range` 范围 | 1-99 |
| V7 | `portrait_key` 和 `model_key` 引用存在 | 资源文件存在性检查 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V8 | `base_attributes` 中的每个 AttributeId 已注册 | 在 DefRegistry<AttributeDef> 中存在 |
| V9 | `innate_abilities` 中的每个 AbilityId 已注册 | 在 DefRegistry<AbilityDef> 中存在 |
| V10 | `innate_buffs` 中的每个 BuffId 已注册 | 在 DefRegistry<BuffDef> 中存在 |
| V11 | `innate_triggers` 中的每个 TriggerId 已注册 | 在 DefRegistry<TriggerDef> 中存在 |
| V12 | `faction`（如果设置）已注册 | 在 DefRegistry<FactionDef> 中存在 |
| V13 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |
| V14 | `equipment_override` 中的每个 EquipmentId（如果设置）已注册 | 在 DefRegistry<EquipmentDef> 中存在 |
| V15 | `spawn_conditions` 中的每个 ConditionId（如果设置）已注册 | 在 DefRegistry<ConditionDef> 中存在 |

### 4.3 Forward Reference 校验

| # | 规则 | 说明 |
|---|------|------|
| V16 | `loot_table`（如果设置）在 L2 加载时只记录不校验 | 标记为 Forward Reference |
| V17 | L3 加载完成后触发二次校验 | 验证 LootTableDef 存在 |

### 4.4 层间依赖校验

| # | 规则 | 说明 |
|---|------|------|
| V18 | MonsterDef 不得引用任何 L3 Gameplay Def（除 `loot_table` 白名单） | 层间依赖方向规则 |
| V19 | MonsterDef 不得引用任何 L4 World Def | 同上 |
| V20 | MonsterDef 之间不得互相引用 | 无循环依赖风险 |

---

## 5. RON 示例

```ron
(
    // ── CreatureBase ──
    id: "mob:goblin_raider",
    name_key: "monster.mob_goblin_raider.name",
    description_key: "monster.mob_goblin_raider.desc",
    schema_version: 1,

    base_attributes: [
        ("attr:strength", 10.0),
        ("attr:dexterity", 14.0),
        ("attr:constitution", 12.0),
        ("attr:intelligence", 8.0),
        ("attr:wisdom", 10.0),
        ("attr:charisma", 6.0),
        ("attr:max_hp", 12.0),
        ("attr:initiative", 2.0),
        ("attr:armor_class", 14.0),
    ],

    innate_abilities: [
        "ability:melee_attack_scimitar",
        "ability:goblin_nimble_escape",
    ],

    innate_buffs: [],
    innate_triggers: [],

    faction: Some("faction:hostile"),
    tags: ["tag:goblinoid", "tag:humanoid", "tag:small_size"],

    portrait_key: Some("portraits/monsters/goblin_raider.png"),
    model_key: Some("models/monsters/goblin_raider.glb"),

    // ── 怪物特有 ──
    difficulty_rating: 1,
    xp_reward: 50,

    loot_table: Some("loot:goblin_raider_drops"),

    equipment_override: Some([
        "equip:scimitar_rusty",
        "equip:leather_armor_worn",
    ]),

    ai_behavior_hints: (
        combat_style: Aggressive,
        target_priority: Weakest,
        ability_usage: OnCooldown,
        movement_strategy: Flank,
        can_call_reinforcements: true,
        custom_behaviors: None,
    ),

    movement_range: 6,

    spawn_conditions: Some([
        "cond:is_nighttime",
        "cond:biome_forest",
    ]),

    size_occupation: Medium,
)
```

---

## 6. 与 CharacterDef 的关系

| 对比维度 | CharacterDef | MonsterDef |
|----------|-------------|------------|
| 核心用途 | 可操作角色 | 战斗中的敌人/中立生物 |
| 装备系统 | 多槽位装备系统 (`equipment_slots`) | 扁平装备列表 (`equipment_override`) |
| 职业系统 | `class_id` + `starting_level` (L3 ProgressionDef) | 无（用 `difficulty_rating`） |
| 战利品 | 无 | `loot_table` (L3 LootTableDef) |
| AI 行为 | 玩家直接控制 | `ai_behavior_hints` |
| 经验值 | 获得经验 | 提供经验 (`xp_reward`) |
| 生成条件 | 无（固定队伍成员） | `spawn_conditions` |
| 体型占位 | 默认 Medium | 可自定义 (`size_occupation`) |
| 阵营 | 通常为 Player 阵营 | 通常为 Hostile/Neutral |

---

*本文档由 @content-architect 维护。*
