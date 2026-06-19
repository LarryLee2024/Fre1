---
id: 03-content.definitions.gameplay.difficulty-def
title: DifficultyDef — Difficulty Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# DifficultyDef — Difficulty Content Def 定义

> **Content Layer**: L3 Gameplay | **领域规则**: `docs/02-domain/domains/combat_domain.md` | **数据 Schema**: `docs/04-data/domains/combat_schema.md` | **插件代码**: `src/content/plugins/difficulty_plugin.rs`

---

## 1. Overview

DifficultyDef 定义了全局难度配置——伤害倍率、AI 强度、资源系数和模式限制。DifficultyDef 是 L3 层依赖最少的 Def 类型（仅依赖 L0 TagDef），作为一个纯数值配置层供 Combat 和其他领域系统使用。

### 关键设计原则

- **纯数值配置**：DifficultyDef 不引用任何 L1 或 L2 Def，仅使用纯数值字段和 L0 TagDef 分类。这使得难度配置完全解耦于具体游戏内容
- **乘法而非加法**：所有倍率以乘法形式应用（如"敌人造成伤害 × 1.5"），避免加法修正带来的边缘情况
- **Encounter 覆盖**：单个 EncounterDef 可通过 `difficulty_override` 覆盖全局难度，实现"Boss 战强制 Hard"等场景
- **限制模式**：`restrictions` 字段定义该难度下的硬性限制（如永久死亡、禁止存档），由 Game Mode 领域读取执行

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `combat_domain.md` | 战斗平衡、难度设计 |
| `combat_schema.md` | DifficultyDef 完整字段结构、AIIntelligence 定义 |
| `tag-def.md` | 本 Def 的 `tags` 引用的 TagDef |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 难度配置定义——全局数值倍率和限制选项。
///
/// DifficultyDef 是 Content Asset，经 Load → Deserialize → Validate → Register → Freeze
/// 管线后进入 DefRegistry<DifficultyDef>，运行时只读。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct DifficultyDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID（DifficultyDef 前缀: `diff_`）
    pub id: DifficultyId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key，如"简单模式：专注于故事"）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 伤害倍率 ──
    /// 敌人造成的伤害倍率（1.0 = 标准）
    pub damage_dealt_multiplier: f32,
    /// 玩家承受的伤害倍率（1.0 = 标准）
    pub damage_taken_multiplier: f32,

    // ── 敌人属性倍率 ──
    /// 敌人数量倍率（1.0 = 标准，2.0 = 双倍敌人）
    pub enemy_count_multiplier: f32,
    /// 敌人生命值倍率（1.0 = 标准）
    pub enemy_hp_multiplier: f32,

    // ── 资源倍率 ──
    /// 经验值倍率（1.0 = 标准）
    pub xp_multiplier: f32,
    /// 金币倍率（1.0 = 标准）
    pub gold_multiplier: f32,
    /// 掉落倍率（1.0 = 标准，影响掉落数量和概率）
    pub loot_multiplier: f32,

    // ── AI 强度 ──
    /// AI 智能等级（影响 Combat 领域 AI 决策质量）
    pub ai_intelligence: AIIntelligence,

    // ── 限制选项 ──
    /// 难度限制（可选）
    pub restrictions: Option<DifficultyRestrictions>,

    // ── 元数据 ──
    /// 标签列表（引用 L0 TagDef，用于分类过滤）
    pub tags: Vec<TagId>,
    /// 难度排序（数字越小越简单，用于 UI 排序）
    pub sort_order: u32,
}
```

### 内嵌数据结构

```rust
/// AI 智能等级——影响敌人 AI 的决策质量
#[derive(Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AIIntelligence {
    /// 低智能：只会攻击最近目标
    Dumb,
    /// 基础智能：会攻击弱点、使用基础技能
    Basic,
    /// 战术智能：会集火、走位、使用组合技
    Tactical,
    /// 自适应智能：会针对玩家策略调整行为
    Adaptive,
}

/// 难度限制——硬性规则约束
#[derive(Deserialize, Clone, Debug)]
pub struct DifficultyRestrictions {
    /// 禁止在地牢中存档
    pub disable_save_in_dungeon: bool,
    /// 永久死亡（角色死亡即删除）
    pub permadeath: bool,
    /// 禁止在非安全区休息
    pub disable_resting_in_dungeon: bool,
    /// 禁止交易（无法使用商店）
    pub disable_trading: bool,
    /// 禁止重新投点（roll 点结果不可重来）
    pub disable_reroll: bool,
}
```

### 字段说明

- **所有倍率字段**: 默认 1.0 = 标准难度。`damage_dealt_multiplier: 2.0` = 敌人造成双倍伤害。`enemy_count_multiplier: 1.5` = 遭遇战生成 1.5 倍敌人（向下取整）
- **`ai_intelligence`**: 不直接修改 AI 逻辑，而是作为参数传递给 Combat 领域的 AI 决策系统。AI 系统根据此参数调整搜索深度、评估策略和决策质量
- **`restrictions`**: 限定列表（非全部列举）。未来可扩展。每个限制是二元选项（启用/禁用），不由 ConditionDef 表达——限制是直接规则，不涉及复杂条件
- **`sort_order`**: UI 排序用。如简单=0, 普通=1, 困难=2, 噩梦=3。数值越大难度越高

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// DifficultyDef 注册插件
pub struct DifficultyDefPlugin;

impl Plugin for DifficultyDefPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset::<DifficultyDef>();
        app.init_asset_loader::<RonAssetLoader<DifficultyDef>>();
        app.insert_resource(DefRegistry::<DifficultyDef>::new());
        app.add_systems(
            PreUpdate,
            load_difficulty_defs
                .run_if(resource_changed::<Assets<DifficultyDef>>())
                .in_set(ContentPipeline::ValidateAndRegister),
        );
    }
}

/// 获取默认难度（sort_order 最低的配置）
pub fn get_default_difficulty(
    registry: &DefRegistry<DifficultyDef>,
) -> Option<&DifficultyDef> {
    registry.iter().min_by_key(|def| def.sort_order)
}

/// 按标签过滤难度
pub fn get_difficulties_by_tag(
    tag_id: &TagId,
    registry: &DefRegistry<DifficultyDef>,
) -> Vec<&DifficultyDef> {
    registry.iter()
        .filter(|def| def.tags.iter().any(|t| t == tag_id))
        .collect()
}
```

### 注册生命周期

```
DifficultyDefPlugin::build
  │
  ├── DifficultyDef 从 assets/config/03_gameplay/difficulties.ron 加载
  │
  ├── Deserialize → Validate → Register → Freeze
  │
  └── Validate 具体规则：
        ├── ID 唯一性
        ├── L0 (TagId) 引用存在性
        ├── 所有倍率字段在合理范围（0.1-10.0）
        ├── sort_order 不重复（每个难度 unique 排序值）
        ├── ai_intelligence 枚举合法性
        ├── L4 禁止引用检查（本 Def 无跨 L0 引用，但需保持检查惯例）
        └── restrictions 各字段合法
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | DifficultyId 不能为空字符串 |
| V2 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V3 | `damage_dealt_multiplier` 范围 | 0.1-10.0 |
| V4 | `damage_taken_multiplier` 范围 | 0.1-10.0 |
| V5 | `enemy_count_multiplier` 范围 | 0.5-5.0 |
| V6 | `enemy_hp_multiplier` 范围 | 0.1-10.0 |
| V7 | `xp_multiplier` 范围 | 0.0-10.0（0 = 无经验） |
| V8 | `gold_multiplier` 范围 | 0.0-10.0 |
| V9 | `loot_multiplier` 范围 | 0.0-10.0 |
| V10 | `ai_intelligence` 合法 | 必须匹配 AIIntelligence 的已知变体 |
| V11 | `sort_order` 不重复 | 多个 DifficultyDef 的 sort_order 必须唯一 |

### 4.2 跨 Def 引用校验

| # | 规则 | 说明 |
|---|------|------|
| V12 | `tags` 中的每个 TagId 已注册 | 在 DefRegistry<TagDef> 中存在 |

### 4.3 层间依赖校验

| # | 规则 | 说明 |
|---|------|------|
| V13 | DifficultyDef 不得引用任何 L4 World Def | 层间依赖方向规则 |

### 4.4 语义校验

| # | 规则 | 说明 |
|---|------|------|
| V14 | sort_order 合理排序 | 简单 < 普通 < 困难 < 噩梦 的排序值应依次递增 |
| V15 | 至少有一个 DifficultyDef 未启用 `restrictions.permadeath` | 确保有"安全"难度选项 |
| V16 | 难度名称约定 | `diff_easy`, `diff_normal`, `diff_hard`, `diff_nightmare` 为保留 ID |

---

## 5. RON 示例

```ron
(
    id: "diff:normal",
    name_key: "difficulty.diff_normal.name",
    description_key: "difficulty.diff_normal.desc",
    schema_version: 1,

    damage_dealt_multiplier: 1.0,
    damage_taken_multiplier: 1.0,

    enemy_count_multiplier: 1.0,
    enemy_hp_multiplier: 1.0,

    xp_multiplier: 1.0,
    gold_multiplier: 1.0,
    loot_multiplier: 1.0,

    ai_intelligence: Basic,

    restrictions: None,

    tags: ["tag:standard_difficulty"],
    sort_order: 1,
)
```

```ron
(
    id: "diff:hard",
    name_key: "difficulty.diff_hard.name",
    description_key: "difficulty.diff_hard.desc",
    schema_version: 1,

    damage_dealt_multiplier: 1.5,
    damage_taken_multiplier: 1.0,

    enemy_count_multiplier: 1.5,
    enemy_hp_multiplier: 1.5,

    xp_multiplier: 1.2,
    gold_multiplier: 0.8,
    loot_multiplier: 0.8,

    ai_intelligence: Tactical,

    restrictions: Some((
        disable_save_in_dungeon: false,
        permadeath: false,
        disable_resting_in_dungeon: true,
        disable_trading: false,
        disable_reroll: false,
    )),

    tags: ["tag:hard_difficulty"],
    sort_order: 2,
)
```

```ron
(
    id: "diff:nightmare",
    name_key: "difficulty.diff_nightmare.name",
    description_key: "difficulty.diff_nightmare.desc",
    schema_version: 1,

    damage_dealt_multiplier: 2.0,
    damage_taken_multiplier: 1.0,

    enemy_count_multiplier: 2.0,
    enemy_hp_multiplier: 2.0,

    xp_multiplier: 1.5,
    gold_multiplier: 0.5,
    loot_multiplier: 0.5,

    ai_intelligence: Adaptive,

    restrictions: Some((
        disable_save_in_dungeon: true,
        permadeath: true,
        disable_resting_in_dungeon: true,
        disable_trading: true,
        disable_reroll: true,
    )),

    tags: ["tag:nightmare_difficulty"],
    sort_order: 3,
)
```

---

## 6. 与其他 L3 Def 的关系

| L3 Def | DifficultyDef 的关系 |
|--------|---------------------|
| EncounterDef | EncounterDef 通过 `difficulty_override` 选择引用 DifficultyDef。不设置 = 使用全局默认难度 |
| QuestDef | 无直接引用。Quest 的奖励不受难度倍率影响（职责分离） |
| ProgressionDef | 无直接依赖。但经验倍率 (`xp_multiplier`) 间接影响 Progression 成长速度 |

**难度应用流程**（简要）：

```
Combat 领域计算伤害
  │
  ├── 1. 查询全局活跃的 DifficultyDef（玩家选择的难度）
  │
  ├── 2. 如果当前 Encounter 有 difficulty_override → 使用覆盖难度
  │
  ├── 3. 应用倍率：
  │     ├── 敌人攻击力 × damage_dealt_multiplier
  │     ├── 敌人 HP × enemy_hp_multiplier（生成时计算）
  │     ├── 玩家受到的伤害 × damage_taken_multiplier
  │     └── 敌人数量 × enemy_count_multiplier（生成时计算）
  │
  └── 4. 应用限制：
        ├── restrictions 影响 Save/Rest/Trade 系统行为
        └── permadeath 影响 Party 领域角色死亡处理
```

---

*本文档由 @content-architect 维护。*
