---
id: 03-content.definitions.gameplay.spawn-group-def
title: SpawnGroupDef — SpawnGroup Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# SpawnGroupDef — SpawnGroup Content Def 定义

> **Content Layer**: L3 Gameplay | **领域规则**: `docs/02-domain/domains/combat_domain.md` | **数据 Schema**: `docs/04-data/domains/combat_schema.md` | **插件代码**: `src/content/plugins/spawn_group_plugin.rs`

---

## 1. Overview

SpawnGroupDef 定义了可复用的怪物编队模板——哪些怪物类型、数量、阵型、刷新规则。SpawnGroupDef 是 EncounterDef 的"构建块"：一个 EncounterDef 引用多个 SpawnGroupDef 组成完整遭遇战。

### 关键设计原则

- **可复用性**：SpawnGroupDef 是 EncounterDef 间的共享模板。例如 "goblin_ambushers" SpawnGroup 可被"森林伏击"和"洞穴突袭"两个 EncounterDef 引用
- **Encounter 无关**：SpawnGroupDef 不引用 EncounterDef。引用方向是单向的：EncounterDef → SpawnGroupDef
- **熵权重**：条目权重决定组内各怪物类型的相对出现概率（如"80% 普通哥布林 + 20% 哥布林弓箭手"）
- **装备覆盖**：`equipment_override` 允许 SpawnGroupDef 为特定怪物配置预设装备（如"所有哥布林弓箭手装备生锈短弓"）

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `combat_domain.md` | 怪物行为、生成规则、阵型概念 |
| `combat_schema.md` | SpawnGroupDef 完整字段结构、FormationType 定义 |
| `condition-def.md` | 本 Def 的 `spawn_conditions` 和 `respawn_rules` 引用的 ConditionDef |
| `monster-def.md` | 本 Def 的 `monster_entries` 引用的 MonsterDef |
| `equipment-def.md` | 本 Def 的 `monster_entries[].equipment_override` 引用的 EquipmentDef |
| `tag-def.md` | 本 Def 的 `tags` 引用的 TagDef |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 怪物生成组定义——描述一组可复用的怪物编队模板。
///
/// SpawnGroupDef 是 Content Asset，经 Load → Deserialize → Validate → Register → Freeze
/// 管线后进入 DefRegistry<SpawnGroupDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct SpawnGroupDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID（SpawnGroupDef 前缀: `spawn_`）
    pub id: SpawnGroupId,
    /// 显示名称（可选，仅用于调试/策划参考）
    pub name_key: Option<LocalizationKey>,
    /// 描述文本（可选）
    pub description_key: Option<LocalizationKey>,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 怪物条目 ──
    /// 组内的怪物列表（权重决定分布比例）
    pub monster_entries: Vec<SpawnMonsterEntry>,

    // ── 生成条件 ──
    /// 生成条件（引用 L1 ConditionDef，可选）
    pub spawn_conditions: Option<Vec<ConditionId>>,

    // ── 阵型 ──
    /// 编队阵型
    pub formation: FormationType,

    // ── 刷新规则 ──
    /// 重生/刷新规则（可选，不设置 = 一次性生成）
    pub respawn_rules: Option<RespawnRules>,

    // ── 元数据 ──
    /// 标签列表（引用 L0 TagDef，用于分类过滤）
    pub tags: Vec<TagId>,
}

/// 生成组怪物条目
#[derive(Deserialize, Clone, Debug)]
pub struct SpawnMonsterEntry {
    /// 怪物类型（引用 MonsterDef）
    pub monster_id: MonsterId,
    /// 该怪物在组内的数量范围（(最小, 最大)）
    pub quantity: (u32, u32),
    /// 等级覆盖（可选，覆盖 MonsterDef 的默认等级换算）
    pub min_level: Option<u32>,
    /// 装备覆盖（可选，替换 MonsterDef 的默认装备）
    pub equipment_override: Option<Vec<EquipmentId>>,
    /// 相对权重（高权重则更大概率生成此怪物而非同组其他）
    pub weight: u32,
}

/// 编队阵型枚举
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum FormationType {
    /// 松散阵型
    Loose,
    /// 密集阵型
    Tight,
    /// 横排阵型
    Line,
    /// 先锋阵型（前排肉盾 + 后排输出）
    Vanguard,
    /// 环绕阵型（围绕目标）
    Surround,
    /// 随机阵型
    Random,
    /// 自定义（Mod 扩展用）
    Custom(String),
}

/// 重生/刷新规则
#[derive(Deserialize, Clone, Debug)]
pub struct RespawnRules {
    /// 是否启用重生
    pub respawn_enabled: bool,
    /// 重生延迟（游戏内小时）
    pub respawn_delay: u32,
    /// 最大重生次数（0 = 无限）
    pub max_respawns: u32,
    /// 重生条件（引用 L1 ConditionDef，可选）
    pub respawn_condition: Option<ConditionId>,
}
```

### 字段说明

- **`monster_entries`**: 组内怪物类型列表。每个条目定义一种怪物及其数量范围。最终生成时，在数量范围内随机取值
- **`quantity: (min, max)`**: 二元组表示数量范围。如 `(2, 4)` = 随机生成 2-4 个该怪物。LootTable 风格的范围表达
- **`weight`**: 组内相对权重。多怪物组中决定每种怪物的比例。如 Goblin: weight 80, GoblinArcher: weight 20 → 4:1 比例
- **`formation`**: 阵型是生成时的初始站位提示。精确站位由 EncounterDef 的 `position_hint` 和 L4 MapDef 共同决定
- **`respawn_rules`**: 重生规则仅适用于非战斗状态下的地图怪物（野怪刷新）。Boss 战一般禁用重生

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// SpawnGroupDef 注册插件
pub struct SpawnGroupDefPlugin;

impl Plugin for SpawnGroupDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<SpawnGroupDef>();
        app.init_asset_loader::<RonAssetLoader<SpawnGroupDef>>();
        app.insert_resource(DefRegistry::<SpawnGroupDef>::new());
        app.add_systems(
            PreUpdate,
            load_spawn_group_defs
                .run_if(resource_changed::<Assets<SpawnGroupDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 按阵型类型过滤 SpawnGroupDef
pub fn get_spawn_groups_by_formation(
    formation: FormationType,
    registry: &DefRegistry<SpawnGroupDef>,
) -> Vec<&SpawnGroupDef> {
    registry.iter()
        .filter(|def| def.formation == formation)
        .collect()
}
```

### 注册生命周期

```
SpawnGroupDefPlugin::build
  │
  ├── SpawnGroupDef 从 assets/config/03_gameplay/spawn_groups.ron 加载
  │
  ├── Deserialize → Validate → Register → Freeze
  │
  └── Validate 具体规则：
        ├── ID 唯一性
        ├── L0 (TagId) 引用存在性
        ├── L1 (ConditionId) 引用存在性
        ├── L2 (MonsterId, EquipmentId) 引用存在性
        ├── monster_entries 非空（至少 1 个）
        ├── quantity.min <= quantity.max
        ├── weight >= 1
        ├── L4 禁止引用检查
        └── respawn_rules.respawn_delay >= 1（如果 enabled）
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | SpawnGroupId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `monster_entries` 非空 | 生成组必须有至少一个怪物条目 |
| V4 | `quantity.0` <= `quantity.1` | 数量范围必须有效 |
| V5 | `weight` >= 1 | 权重至少为 1 |
| V6 | `formation` 合法 | 必须匹配 FormationType 的已知变体 |
| V7 | `respawn_rules.respawn_delay` >= 1（如果启用） | 重生延迟至少为 1 小时 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V8 | `monster_entries` 中的每个 MonsterId 已注册 | 在 DefRegistry<MonsterDef> 中存在 |
| V9 | `monster_entries` 中的每个 EquipmentId（equipment_override 中，如果设置）已注册 | 在 DefRegistry<EquipmentDef> 中存在 |
| V10 | `spawn_conditions` 中的每个 ConditionId（如果设置）已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V11 | `respawn_rules.respawn_condition`（如果设置）已注册 | 在 DefRegistry<ConditionDef> 中存在 |
| V12 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |

### 4.3 层间依赖校验

| # | 规则 | 说明 |
|---|------|------|
| V13 | SpawnGroupDef 不得引用任何 L4 World Def | 层间依赖方向规则 |

### 4.4 语义校验

| # | 规则 | 说明 |
|---|------|------|
| V14 | 同组中不推荐重复的 MonsterId | 多个相同 MonsterId 的条目可合并为更大的 quantity 范围 |
| V15 | `respawn_enabled = true` 时应有 respawn_delay | 启重生必须设置延迟时间 |
| V16 | `equipment_override` 不应与 MonsterDef 的 `equipment_override` 冲突 | 两者同时设置时的合并规则需由 Combat 领域定义 |

---

## 5. RON 示例

```ron
(
    id: "spawn:goblin_ambushers",
    name_key: Some("spawn_group.spawn_goblin_ambushers.name"),
    description_key: None,
    schema_version: 1,

    monster_entries: [
        (
            monster_id: "mob:goblin_raider",
            quantity: (2, 4),
            min_level: None,
            equipment_override: None,
            weight: 80,
        ),
        (
            monster_id: "mob:goblin_archer",
            quantity: (1, 2),
            min_level: None,
            equipment_override: Some(["equip:shortbow_rusty"]),
            weight: 20,
        ),
    ],

    spawn_conditions: Some([
        "cond:is_nighttime",
    ]),

    formation: Flanking,

    respawn_rules: Some((
        respawn_enabled: true,
        respawn_delay: 24,
        max_respawns: 3,
        respawn_condition: None,
    )),

    tags: ["tag:goblinoid", "tag:low_level", "tag:forest"],
)
```

---

## 6. 与其他 L3 Def 的关系

| L3 Def | SpawnGroupDef 的关系 |
|--------|---------------------|
| EncounterDef | EncounterDef 引用 SpawnGroupDef 构建遭遇战。引用方向是单向的（EncounterDef → SpawnGroupDef） |
| QuestDef | 无直接引用关系。QuestDef 的 Kill 目标引用 MonsterDef，而非 SpawnGroupDef |

**SpawnGroupDef 的复用模式**：

```
SpawnGroupDef "goblin_ambushers"  ← 可复用的模板
    │
    ├── 被 EncounterDef "forest_ambush" 引用 (count: 2, Flanking)
    ├── 被 EncounterDef "cave_raid" 引用 (count: 1, Vanguard)
    └── 被 EncounterDef "night_watch" 引用 (count: 3, Surround)
```

---

*本文档由 @content-architect 维护。*
