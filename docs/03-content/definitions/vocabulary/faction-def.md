---
id: 03-content.definitions.vocabulary.faction-def
title: FactionDef — Faction Content Def 定义
status: draft
owner: content-architect
created: 2026-06-20
updated: 2026-06-20
---

# FactionDef — Faction Content Def 定义

> **Content Layer**: L0 Vocabulary | **领域规则**: `docs/02-domain/domains/faction_domain.md` | **数据 Schema**: `docs/04-data/domains/faction_schema.md` | **插件代码**: `src/content/plugins/vocabulary_plugin.rs`

---

## 1. Overview

FactionDef 定义了游戏中的一个**阵营**——阵营是实体（角色、怪物、NPC）的群体归属标识，决定了实体之间的初始关系和交互倾向。

阵营系统是游戏世界的社会基础：
- **玩家阵营**：玩家控制的角色所属
- **敌对阵营**：与玩家敌对的怪物、敌人
- **中立阵营**：不主动敌对的 NPC、生物
- **特殊阵营**：临时同盟、第三方势力

### FactionDef 的定位

FactionDef 只定义阵营的身份标识和默认态度。**阵营间的关系矩阵**（谁对谁友好/敌对/中立）不由 FactionDef 定义——因为它涉及 L0-to-L0 引用（A 阵营引用 B 阵营），违反 L0 同层引用规则。关系矩阵由更高层（L3 Gameplay）定义。

### 关系矩阵的分离

| 概念 | 归属层 | 说明 |
|------|--------|------|
| `FactionDef` | L0 Vocabulary | 阵营身份：ID、名称、默认态度、颜色 |
| `FactionRelationshipMatrix` | L3 Gameplay | 阵营间关系：FactionA → FactionB = Hostile |
| `FactionReputation` | L3 Gameplay | 个体声望：玩家在阵营中的声望值 |

这种分离的好处：
- 可以在不新增阵营的情况下改变阵营关系（L3 配置变更）
- Mod 可以在不修改基础 Def 的情况下定义新的阵营关系
- 运行时声望变化只影响 L3 数据，不影响 L0 Def

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `faction_domain.md` | 阵营定义规则、默认态度、关系计算、声望系统 |
| `faction_schema.md` | FactionRelationship、ReputationLevel 的数据结构 |
| `character-def.md` | 本 Def 被 CharacterDef.faction 引用 |
| `monster-def.md` | 本 Def 被 MonsterDef.faction 引用 |
| `summon-def.md` | 本 Def 被 SummonDef.faction_override 引用 |
| `quest-def.md` | 本 Def 被 QuestDef 关联的阵营条件引用 |
| `encounter-def.md` | 本 Def 被 EncounterDef 的阵营配置引用 |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 阵营定义——游戏中实体的群体归属标识。
///
/// FactionDef 只定义阵营的身份标识和默认态度。
/// 阵营间关系矩阵由 L3 Gameplay 层定义，以遵守
/// L0 同层引用禁止规则（FactionDef 不可引用其他 FactionDef）。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct FactionDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID
    pub id: FactionId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 阵营属性 ──
    /// 默认态度——当未定义特定关系时使用的基线态度
    ///
    /// 这是"第一印象"——如果你不知道对方是什么阵营，默认怎么对待？
    /// - Friendly: 默认友好（玩家阵营之间）
    /// - Neutral: 默认中立（大多数中立 NPC）
    /// - Hostile: 默认敌对（怪物阵营）
    pub default_attitude: FactionAttitude,

    // ── 表现资源 ──
    /// 图标 Key（用于 UI 中的阵营标识）
    pub icon_key: Option<String>,

    /// 阵营颜色（十六进制 RGB，用于 UI 着色）
    ///
    /// 示例: "#FF0000" = 红色（敌对）、"#00FF00" = 绿色（友好）
    /// 格式: `#RRGGBB` 或 `#RRGGBBAA`
    pub color_hex: Option<String>,
}
```

### 内嵌枚举

```rust
/// 阵营的默认态度——当无特定关系定义时的基线态度
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum FactionAttitude {
    /// 友好——默认视为盟友
    ///
    /// 玩家阵营之间使用。不会自动攻击，可共享信息/资源。
    Friendly,
    /// 中立——默认互不干涉
    ///
    /// 大多数中立 NPC 使用。不会主动攻击，但也不会帮助。
    /// 受到攻击后可能变为临时敌对。
    Neutral,
    /// 敌对——默认视为敌人
    ///
    /// 怪物阵营使用。主动攻击，不可交涉（除非特殊机制）。
    Hostile,
}
```

### 字段说明

- **`default_attitude`**: 决定了实体在"不知道对方阵营详细信息"时的基线行为。不等于最终态度——最终态度由 L3 的关系矩阵叠加声望值计算得出
- **`color_hex`**: UI 颜色标识。用于阵营名称着色、小地图标记、名称板颜色等。这是 L0 层提供的 UI 元数据，不是业务逻辑的一部分

### 为什么没有`relationship_overrides`字段

常见的 FactionDef 设计会包含一个关系矩阵：
```rust
// 禁止 —— 违反 L0 同层引用规则
relationship_overrides: Vec<(FactionId, FactionAttitude)>,
```

这个字段涉及 FactionId-to-FactionId 引用（L0-to-L0），违反内容分层规则。关系矩阵必须上移至 L3 Gameplay 层，与声望系统和外交系统放在一起：

```rust
// L3 Gameplay — 阵营关系矩阵定义
pub struct FactionRelationshipMatrix {
    // 每对阵营的关系定义
    pub relationships: Vec<(FactionId, FactionId, FactionAttitude)>,
}
```

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// 按 ID 查找 FactionDef
pub fn get_faction_def(
    id: &FactionId,
    registry: &DefRegistry<FactionDef>,
) -> Option<&FactionDef> {
    registry.get(id)
}

/// 按默认态度过滤 FactionDef
pub fn get_faction_defs_by_attitude(
    attitude: FactionAttitude,
    registry: &DefRegistry<FactionDef>,
) -> Vec<&FactionDef> {
    registry.iter()
        .filter(|def| def.default_attitude == attitude)
        .collect()
}
```

### 注册生命周期

```
Load (factions.ron) → Deserialize → Validate → Register (DefRegistry<FactionDef>) → Freeze
```

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | FactionId 不能为空字符串 |
| V2 | `id` 格式合法 | 必须匹配 `^faction:[a-z][a-z0-9_]+$`（如 `faction:player`、`faction:enemy`） |
| V3 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V4 | `name_key` 非空 | 阵营必须有显示名称 |
| V5 | `description_key` 非空 | 阵营必须有描述文本 |
| V6 | `default_attitude` 为有效枚举值 | 必须是 FactionAttitude 的三个变体之一 |
| V7 | `color_hex` 格式合法（若设置） | 必须匹配 `^#[0-9A-Fa-f]{6}([0-9A-Fa-f]{2})?$`（如 `#FF0000`、`#00FF00AA`） |

### 4.2 无跨 Def 引用校验（L0 约束）

FactionDef 是 L0 Def，禁止引用任何其他 Def（包括其他 FactionDef）。关系矩阵在 L3 Gameplay 层定义。

---

## 5. RON 示例

```ron
// FactionDef 示例 — 玩家阵营
(
    id: "faction:player",
    name_key: "faction.player.name",
    description_key: "faction.player.desc",
    schema_version: 1,

    default_attitude: Friendly,

    icon_key: Some("icons/factions/player.png"),
    color_hex: Some("#4488FF"),
)
```

```ron
// FactionDef 示例 — 敌对阵营
(
    id: "faction:enemy",
    name_key: "faction.enemy.name",
    description_key: "faction.enemy.desc",
    schema_version: 1,

    default_attitude: Hostile,

    icon_key: Some("icons/factions/enemy.png"),
    color_hex: Some("#FF4444"),
)
```

```ron
// FactionDef 示例 — 中立阵营
(
    id: "faction:neutral",
    name_key: "faction.neutral.name",
    description_key: "faction.neutral.desc",
    schema_version: 1,

    default_attitude: Neutral,

    icon_key: Some("icons/factions/neutral.png"),
    color_hex: Some("#CCCCCC"),
)
```

---

## 6. 设计说明

### 阵营关系矩阵的时序

FactionDef 注册后，L3 Gameplay 层的阵营关系矩阵在以下时机加载：

```
Phase 1: L0 加载 → FactionDef 全部注册
Phase 2: L1/L2 加载（跳过关系矩阵）
Phase 3: L3 加载 → FactionRelationshipMatrix 注册
Phase 4: 关系矩阵校验
     ├── 每个 FactionId 引用在 DefRegistry<FactionDef> 中存在
     ├── 矩阵对称性检查（若 A 对 B 友好，B 对 A 也应该是友好）
     └── 默认态度覆盖完整性检查
Phase 5: 关系矩阵冻结 → 战斗系统可用
```

这意味着：
- 在 L3 加载完成前，阵营关系不可用（fallback 到 `default_attitude`）
- 运行时只使用 L3 的关系矩阵，不直接查询 FactionDef 的 `default_attitude`
- `default_attitude` 只作为 L3 关系矩阵未定义时的基线

### 阵营 vs 标签（FactionId vs TagId）

| 维度 | FactionDef | TagDef |
|------|-----------|--------|
| 语义 | 群体归属 | 通用分类 |
| 运行时用途 | 敌我判断、声望、外交 | 分类、过滤、条件匹配 |
| 关系表达 | L3 关系矩阵 | TagHierarchy（层级继承） |
| 数量级 | 几十（最多几十个阵营） | 数百（最多 200+ 标签） |
| UI 表现 | 颜色、图标、名称着色 | 通常无 UI 表现 |

FactionDef 和 TagDef 在 L0 层**无引用关系**。Content Pipeline 不保证 `faction:player` 有对应的 `tag:faction_player` 标签。需要这种映射时，在 Data Schema 层或 L3 层定义映射表。

---

*本文档由 @content-architect 维护。*
