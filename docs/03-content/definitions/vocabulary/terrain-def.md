---
id: 03-content.definitions.vocabulary.terrain-def
title: TerrainDef — Terrain Content Def 定义
status: draft
owner: content-architect
created: 2026-06-22
updated: 2026-06-22
---

# TerrainDef — Terrain Content Def 定义

> **Content Layer**: L0 Vocabulary | **领域规则**: `docs/02-domain/domains/terrain_domain.md` | **数据 Schema**: `docs/04-data/domains/terrain_schema.md` | **插件代码**: `src/content/plugins/vocabulary_plugin.rs`

---

## 1. Overview

TerrainDef 定义了游戏世界中的一种**基础地形类型**——每类地形（平原、森林、山地、水域等）的通行属性、战术加成和表现元数据。

地形是 SRPG 战斗地图的基础构成元素，影响所有上层系统：
- **移动系统**（L1/L2）：通过 `move_cost` 和 `flags` 决定单位是否/如何通过
- **战斗系统**（L3）：通过 `defense_bonus` / `avoid_bonus` / `concealment` 影响攻防计算
- **AI 系统**（L3）：通过 `move_cost` / `concealment` 影响路径偏好和目标选择
- **渲染系统**（Infra）：通过 `color_hex` / `tile_material_key` 驱动视觉表现

### TerrainDef 与旧原型的关键差异

旧原型 `docs/99-history/ai_ignore_this_dir/ai_ignore_this_dir/content/terrains/plain.ron` 的字段在本设计中的去向：

| 旧字段 | 去向 | 理由 |
|--------|------|------|
| `name: "草"` | → `name_key: LocalizationKey` | Localization First（宪法 §22） |
| `char_code: Some('P')` | **已移除** | TMX 使用 GID 索引，不依赖字符映射 |
| `move_cost: 1` | 保留，类型改为 `f32` | 支持非整数消耗（如沼泽 1.5） |
| `defense_bonus: 0` | 保留 | 升格为 `i32` |
| `color: (0.56, 0.73, 0.35)` | 保留 | 改为 `color_hex: Option<String>`（`#RRGGBB`） |
| `passable: true` | → `TerrainFlags` 位标记 | 支持更细粒度的通行控制 |

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `terrain_domain.md` | 地形类型、通行性、遮蔽度、表面变化规则 |
| `terrain_schema.md` | TileProperties、TerrainType、Passability、SurfaceType 数据 Schema |
| `tactical_domain.md` | GridMap 移动规则、寻路如何消费 move_cost |
| `tactical_schema.md` | TileData（packed u32）、TileFlags 位标记 |
| `combat_domain.md` | 地形防御加成、遮蔽度在战斗计算中的应用 |
| `content-content-layering.md` | L0 Vocabulary 层约束（无跨 Def 引用） |
| `docs/01-architecture/20-tactical-combat/ADR-022-grid-terrain-faction.md` | GridMap + TerrainDef + Faction 系统设计 |
| `docs/01-architecture/40-cross-cutting/ADR-065-map-content-pipeline.md` | Tile → Config 映射策略（Tile 只存 TerrainId） |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 地形类型定义——游戏中一种基础地形的通行属性、战术加成和表现元数据。
///
/// TerrainDef 是 L0 Vocabulary Def，禁止引用任何其他 Def（包括同层 TagDef）。
/// 地形类型的 Gameplay 数值在 TerrainDef 中定义，Tile 只存储 TerrainId 引用。
///
/// TerrainDef 的数据流向：
///   TerrainDef (Config Registry) → TileEntry.terrain_id → GridMap.TileData
///
/// 详见 ADR-065 §4 Tile → Config 映射策略。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct TerrainDef {
    // ── 统一标识字段 ──
    /// 全局唯一 ID（TerrainDef 前缀: `ter:`）
    pub id: TerrainId,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 描述文本（本地化 Key）
    pub description_key: LocalizationKey,
    /// Schema 版本号
    pub schema_version: u32,

    // ── 通行属性 ──
    /// 移动消耗（格数），0.0 = 不可通行
    ///
    /// 该值乘以单位的移动力得到在该地形上的实际消耗。
    /// 示例：move_cost: 1.0（平地），2.0（森林/沼泽），0.0（水域/障碍）。
    /// f32 类型支持非整数消耗（如 1.5），更精确的消耗计算
    /// 由寻路系统使用 GridMap + TileData 查询后计算。
    pub move_cost: f32,

    /// 飞行消耗（可选），None = 与 move_cost 相同
    ///
    /// 飞行单位移动时使用此值而非 move_cost。
    /// 例如：山地 move_cost: 0.0（步行不可达），fly_cost: 2.0（飞行消耗正常）。
    pub fly_cost: Option<f32>,

    // ── 战术属性 ──
    /// 防御加成（被攻击时额外防御值）
    ///
    /// 典型值：森林 +2，高地 +1，平地为 0。
    /// 与 ADR-022 TileFlags 无关——这是 Gameplay 数值，不在 TileData 中存储。
    pub defense_bonus: i32,

    /// 闪避加成（站在该地形上获得的额外闪避率）
    ///
    /// 典型值：丛林 +10，掩体 +15，平地为 0。
    pub avoid_bonus: i32,

    /// 基础遮蔽度——未站在该地形上时对目标的遮蔽程度
    pub concealment: Concealment,

    /// 通行标记位集合
    ///
    /// 决定该地形的基本通行属性。这些标记在 Map Importer 阶段
    /// 被转换为 TileData.flags（详见 ADR-022），运行时不再查询 TerrainDef。
    pub flags: TerrainFlags,

    // ── 表现资源 ──
    /// 地形颜色（十六进制 RGB，用于小地图/编辑器预览）
    ///
    /// 示例: "#8FBA59"（plain 绿）、"#5A8C3A"（forest 深绿）
    /// 格式: `#RRGGBB`
    pub color_hex: Option<String>,

    /// 地形瓦片材质 Key（用于渲染系统查找对应 Sprite/TileSet 资源）
    ///
    /// 该值由渲染系统用于索引 TileSet 中的 Sprite，不包含渲染逻辑。
    pub tile_material_key: Option<String>,
}
```

### 内嵌枚举

```rust
/// 地形遮蔽度——影响单位在该地形上的被侦测/瞄准难度。
///
/// 由 Targeting 系统和 Combat 系统的命中计算消费。
/// 此枚举是 TerrainDef 的内联定义，与 terrain_schema.md 中的
/// Concealment 枚举结构一致但独立——Content 层不需要依赖 Schema 层。
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Concealment {
    /// 无遮蔽——完全可见，无命中修正
    None,
    /// 半遮蔽——隐蔽 -2 命中修正
    /// 典型地形：丛林、烟雾
    Half,
    /// 全遮蔽——不可见，无法作为目标
    /// 典型地形：全遮蔽掩体后方（通常配合 blocking 标记）
    Full,
}
```

```rust
/// 地形通行标记集合——决定该地形的基本通行方式。
///
/// 这些标记在运行时被转换为 TileData.flags（ADR-022 定义的 packed bits），
/// 通过 GridMap 公开给寻路和单位移动系统。
#[derive(Deserialize, Clone, Debug)]
pub struct TerrainFlags {
    /// 可行走——地面单位可通过
    ///
    /// false = 步行不可通行（如水域、岩浆、悬崖）
    pub passable: bool,

    /// 可飞行——飞行单位可通过
    ///
    /// false = 飞行不可通行（如室内天花板、结界）
    /// true 但 passable=false = 仅飞行单位可通过
    pub flyable: bool,

    /// 可建造——可在该地形上建造/放置设施
    ///
    /// 用于陷阱部署、召唤物放置、建筑类技能。
    pub buildable: bool,

    /// 阻挡视线——该地形阻挡射击线/视野
    ///
    /// 用于遮挡类地形（墙壁、大型障碍物）。
    /// 注意：此标记决定"通过该地形的视线是否被阻挡"，
    /// 与 concealment 不同（concealment 决定"站在该地形上的单位的可见度"）。
    pub blocks_sight: bool,
}
```

### 字段说明

- **`move_cost`**: 基础通行成本，0.0 = 完全不可通行。与 `flags.passable` 同步——passable=false 时寻路系统直接拒绝，不读取 move_cost。这是两个互补的通行控制层级：flags 是二值通过/拒绝，move_cost 是连续成本值
- **`fly_cost`**: 飞行单位独立成本。设计意图：某些地形对地面单位困难但飞行单位容易（山地），或反之（室内低矮通道）。None = 使用 move_cost（退化同值）
- **`defense_bonus` / `avoid_bonus`**: 纯粹的 Gameplay 数值，仅在战斗计算时消费。这两个值不存储在 TileData 中——TerrainDef Registry 是唯一数据源
- **`concealment`**: 影响 Targeting 视野计算、Combat 命中计算。与 `flags.blocks_sight` 配合使用：blocks_sight 是"通过性遮挡"，concealment 是"站位遮挡"
- **`color_hex`**: 主要服务于 L0-L2 层的快速预览/调试用途（编辑器 UI、小地图、调试渲染层）。正式地图渲染使用 `tile_material_key` 引用的 TileSet 资源
- **`tile_material_key`**: 渲染系统的资源索引字符串，不包含渲染逻辑。该字符串由 Infra 层的渲染系统映射到具体 Bevy 材质/纹理资源。格式和解析规则由 `docs/06-ui/` 定义

### 与旧 TerrainDefRon 的关键差异

| 维度 | 旧原型 `TerrainDefRon` | 新 `TerrainDef` (L0 Content) |
|------|-----------------------|-------------------------------|
| `id` | `"plain"`（无前缀） | `"ter:plain"`（L0 统一前缀格式） |
| `name` | 硬编码中文 `"草"` | `name_key: LocalizationKey` |
| `char_code` | `Some('P')` | **已移除**（TMX 用 GID） |
| `color` | `(f32, f32, f32)` | `color_hex: Option<String>`（标准化） |
| `passable` | `bool` | 合并到 `TerrainFlags { passable, flyable, buildable, blocks_sight }` |
| `move_cost` | `u32`（0=不可通行） | `f32`（0.0=不可通行） |
| 遮蔽度 | 无 | 新增 `concealment: Concealment` |
| 闪避加成 | 无 | 新增 `avoid_bonus: i32` |
| 飞行成本 | 无 | 新增 `fly_cost: Option<f32>` |
| 材质引用 | 无 | 新增 `tile_material_key: Option<String>` |
| 注册方式 | 硬编码 `register_defaults()` + 文件扫描 | Content Pipeline（Load → Deserialize → Validate → Register → Freeze） |

---

## 3. Registry 模式

```rust
use crate::infra::registry::DefRegistry;

/// TerrainDef 随 L0 Vocabulary Plugin 统一加载
pub struct VocabularyPlugin;

impl Plugin for VocabularyPlugin {
    fn build(&self, app: &mut App) {
        // TerrainDef 注册在 L0 阶段
        app.register_asset::<TerrainDef>();
        app.insert_resource(DefRegistry::<TerrainDef>::new());
        // 与其他 L0 Def 一起批量加载
    }
}

/// 按通行性过滤 TerrainDef
pub fn get_terrains_by_passability(
    registry: &DefRegistry<TerrainDef>,
) -> Vec<&TerrainDef> {
    registry.iter()
        .filter(|def| def.flags.passable)
        .collect()
}
```

### 注册生命周期

```
Phase 1 (L0): TerrainDef 从 assets/config/00_vocabulary/terrains.ron 加载
    │
    ├── Load        → AssetServer 加载 RON → Deserialize
    ├── Validate    → ID 唯一性、字段合法性、枚举有效性
    ├── Register    → DefRegistry<TerrainDef>.insert()
    └── Freeze      → 运行时只读
          │
          ▼  [L1+ 加载完成后]
    Importer 验证阶段（构建时工具）：
          └── validate_terrain_ids() → 检查所有 TMX 中引用的 TerrainId 在 Registry 中存在
```

### TileFlags 的构建时机

`TerrainDef.flags`（Content 层布尔结构）到 `TileData.flags`（Tactical 层 u8 bitmask）的转换发生在 Map Importer 阶段：

```
TerrainDef.flags (Content L0)
  { passable: true, flyable: true, buildable: false, blocks_sight: false }
    │
    ▼  [Map Importer — 构建时]
TileData.flags (Tactical 运行时)
  TileFlags(0b0000_0011)  // PASSABLE | FLYABLE
    │
    ▼  [运行时]
GridMap.get_tile(pos) → TileData.flags().is_passable()
```

这个转换是 Importer 的职责，不是 Content Pipeline 的职责。运行时 GridMap 直接读取 TileData.flags，不再回查 TerrainDef Registry——这是性能优化（TileData 是 packed u32，单次内存读取）。

---

## 4. 校验规则

### 4.1 字段级校验

| # | 规则 | 说明 |
|---|------|------|
| V1 | `id` 非空 | TerrainId 不能为空字符串 |
| V2 | `id` 格式合法 | 必须匹配 `^ter:[a-z][a-z0-9_]+$`（如 `ter:plain`、`ter:deep_water`） |
| V3 | `schema_version` 兼容 | 当前支持的版本为 1 |
| V4 | `name_key` 非空 | 地形必须有显示名称 |
| V5 | `description_key` 非空 | 地形必须有描述文本 |
| V6 | `move_cost` >= 0.0 | 移动消耗不能为负数；0.0 = 不可通行 |
| V7 | `defense_bonus` 合理范围 | 建议 -10 到 +10，超出范围发出警告 |
| V8 | `avoid_bonus` 合理范围 | 建议 0 到 +30，超出范围发出警告 |
| V9 | `concealment` 为有效枚举值 | 必须是 Concealment 的三个变体之一 |
| V10 | `color_hex` 格式合法（若设置） | 必须匹配 `^#[0-9A-Fa-f]{6}$`（如 `#8FBA59`） |

### 4.2 一致性校验

| # | 规则 | 说明 |
|---|------|------|
| V11 | `move_cost` 与 `flags.passable` 一致 | passable=false 时 move_cost 应为 0.0；passable=true 时 move_cost 应 > 0.0 |
| V12 | `fly_cost` 一致性（若设置） | fly_cost 应 >= 0.0；0.0 = 飞行不可通行 |
| V13 | `blocks_sight` 与 `concealment` 逻辑一致 | blocks_sight=true 且 concealment=None 是可疑的（阻挡视线却无遮蔽），发出警告 |

### 4.3 无跨 Def 引用校验（L0 约束）

TerrainDef 是 L0 Def，禁止引用任何其他 Def：

| # | 规则 | 说明 |
|---|------|------|
| V14 | TerrainDef 不包含任何 Def ID 字段 | 字段只允许原始类型和枚举 |
| V15 | TerrainDef 不引用 TagDef | 不包含 `tags: Vec<TagId>` |

### 4.4 语义校验

| # | 规则 | 说明 |
|---|------|------|
| V16 | `tile_material_key` 建议有值 | 渲染系统需要材质引用才能正确渲染地形（仅警告） |
| V17 | 地形类型定义完备性 | 一个地图配置中引用的所有 TerrainId 必须在 Registry 中存在（由 Importer 在构建时校验） |

---

## 5. RON 示例

```ron
// TerrainDef 示例 — 平原
(
    id: "ter:plain",
    name_key: "terrain.ter_plain.name",
    description_key: "terrain.ter_plain.desc",
    schema_version: 1,

    move_cost: 1.0,
    fly_cost: None,

    defense_bonus: 0,
    avoid_bonus: 0,
    concealment: None,
    flags: (
        passable: true,
        flyable: true,
        buildable: true,
        blocks_sight: false,
    ),

    color_hex: Some("#8FBA59"),
    tile_material_key: Some("tiles/terrain/plain"),
)
```

```ron
// TerrainDef 示例 — 森林
(
    id: "ter:forest",
    name_key: "terrain.ter_forest.name",
    description_key: "terrain.ter_forest.desc",
    schema_version: 1,

    move_cost: 2.0,
    fly_cost: None,

    defense_bonus: 2,
    avoid_bonus: 10,
    concealment: Half,
    flags: (
        passable: true,
        flyable: true,
        buildable: false,
        blocks_sight: false,
    ),

    color_hex: Some("#5A8C3A"),
    tile_material_key: Some("tiles/terrain/forest"),
)
```

```ron
// TerrainDef 示例 — 水域（飞行单位可通过）
(
    id: "ter:deep_water",
    name_key: "terrain.ter_deep_water.name",
    description_key: "terrain.ter_deep_water.desc",
    schema_version: 1,

    move_cost: 0.0,
    fly_cost: Some(1.5),

    defense_bonus: 0,
    avoid_bonus: 0,
    concealment: None,
    flags: (
        passable: false,
        flyable: true,
        buildable: false,
        blocks_sight: false,
    ),

    color_hex: Some("#3A7BD5"),
    tile_material_key: Some("tiles/terrain/water"),
)
```

```ron
// TerrainDef 示例 — 墙壁（阻挡视线，不可通行）
(
    id: "ter:wall",
    name_key: "terrain.ter_wall.name",
    description_key: "terrain.ter_wall.desc",
    schema_version: 1,

    move_cost: 0.0,
    fly_cost: Some(3.0),

    defense_bonus: 0,
    avoid_bonus: 15,
    concealment: Full,
    flags: (
        passable: false,
        flyable: true,
        buildable: false,
        blocks_sight: true,
    ),

    color_hex: Some("#8B7355"),
    tile_material_key: Some("tiles/terrain/wall"),
)
```

---

## 6. 与 Tactical Domain TileFlags 的映射关系

TerrainDef 的 `flags: TerrainFlags` 与 Tactical Domain 的 `TileFlags`（`src/core/domains/tactical/resources.rs`）存在逻辑映射，但两者是独立类型：

| TerrainDef.flags (L0 Content) | TileFlags (Tactical 运行时) | 映射规则 |
|-------------------------------|----------------------------|---------|
| `passable: true` | `TileFlags::PASSABLE (0b0000_0001)` | Importer 在构建 TMX → MapAsset 时执行转换 |
| `flyable: true` | `TileFlags::FLYABLE (0b0000_0010)` | 同上 |
| `buildable: true` | `TileFlags::BUILDABLE (0b0000_0100)` | 同上 |
| `blocks_sight: true` | `TileFlags::BLOCKS_SIGHT (0b0000_1000)` | 同上 |

### 为什么有两个类型？

| 维度 | `TerrainFlags` (L0 Content) | `TileFlags` (Tactical 运行时) |
|------|-----------------------------|-------------------------------|
| 位置 | TerrainDef 定义 | `TileData.packed`（u32 中的低 8 位） |
| 格式 | `struct { bool, bool, bool, bool }` | `struct TileFlags(pub u8)` — 位掩码 |
| 使用者 | 内容作者（可读性优先） | 寻路系统（性能优先） |
| 变更时机 | 内容管线加载时，不变 | 运行时不可变（从 TileData 提取） |
| 存储位置 | Config Registry | GridMap Resource |

**内容作者层面**：内容创作者通过 RON 中的 `flags: (passable: true, flyable: true, ...)` 控制地形通行性——不需要理解位掩码。

**运行时层面**：Map Importer 将 `TerrainDef.flags` 转换为 `TileData.flags`（packed u8 bitmask），运行时寻路系统通过 `GridMap.get_tile(pos).flags()` 直接读取，不经过 Config Registry 查询，确保寻路性能。

### 运行时映射流程

```
TerrainDef Registry (Config)
  │
  ├── TerrainDef { id: "ter:plain", flags: { passable: true, flyable: true, ... } }
  │
  ▼  [Map Importer — 构建时]
MapAsset {
    terrain_grid: TerrainGrid {
        tiles: [
            TileEntry { terrain_id: "ter:plain", flags: TileFlags(PASSABLE|FLYABLE), ... },
            ...
        ]
    }
}
  │
  ▼  [MapLoader — 启动时]
GridMap (Resource)
  └── TileData { packed: u32(terrain_id | height | flags) }
        └── .flags() → TileFlags(u8) → .is_passable() → bool
```

---

## 7. 内容资产目录位置

TerrainDef 的 RON 资产位于 L0 目录 `assets/config/00_vocabulary/`：

```
assets/config/00_vocabulary/
├── tags.ron                  ← TagDef 集合
├── attributes.ron            ← AttributeDef 集合
├── damage_types.ron          ← DamageTypeDef 集合
├── factions.ron              ← FactionDef 集合
├── elements.ron              ← ElementDef 集合
├── status_categories.ron     ← StatusCategoryDef 集合
└── terrains.ron              ← TerrainDef 集合（新增）
```

遵循**单文件多 Def**原则：所有 TerrainDef 放在一个 `terrains.ron` 文件中。超过 2000 行或 50 个 Def 时可拆分为 `terrains/` 子目录。

---

## 8. 设计说明

### 为什么 TerrainDef 属于 L0 而非 L4？

TerrainDef 定义的是基础词汇（"什么是平原、什么是森林、通行成本多少"），而非具体世界中的地图配置。类比 ElementDef 定义"什么是火元素"，TerrainDef 定义"什么是森林地形"。具体地图中"哪些格子是森林"属于 L4 MapDef 的编排职责。

分层归属的核心判定标准：**是否被多个上层层引用**。TerrainDef 被 L1（移动成本查询）、L2（实体放置检查）、L3（战斗加成计算）、L4（地图渲染）全部引用，符合 L0 Vocabulary 的多层引用模式。

### 旧原型 char_code 的移除理由

旧原型使用 `char_code: Option<char>` 在文本网格中表示地形（如 `"MPPPM"` 中的 'P'=Plain、'M'=Mountain）。这个设计来源于**文本网格**格式（LevelConfigDef.terrain_grid: Vec<String>）。

在 Tiled + Importer 管线中，TMX 使用 GID（Global Tile ID）而非字符码来索引 TileSet。Importer 通过 `tmx_parser` 读取 GID 并映射到 TerrainId，不需要 char_code 作为中间层。因此 TerrainDef 不再包含 char_code 字段。

### concealment vs blocks_sight 的区别

这两个概念处理不同层面的可见性：

```
concealment: None   → 站在地形上的单位完全可见
concealment: Half   → 站在地形上的单位 -2 命中（如丛林）
concealment: Full   → 站在地形上的单位不可被瞄准（如全遮蔽掩体）

flags.blocks_sight: false → 视线可以通过该地形
flags.blocks_sight: true  → 视线被该地形阻挡（不能"看穿"）
```

一个地形可以 blocks_sight=true（阻挡视线穿过）同时 concealment=None（站在其上不提供遮蔽），反之亦然。

### 为什么 SurfaceType 不在 TerrainDef 中

地形表面的运行时变化（Normal→Ice→Burning→Normal）属于 Instance 层的数据，不是 Definition 层的属性。SurfaceType 和 SurfaceOverride 由 `terrain_schema.md`（04-data）管理，TerrainDef 只定义基础地形类型。Runtime 表面变化通过 Terrain Domain 的 `tile_properties.surface` 和 `tile_properties.original_surface` 追踪。

---

*本文档由 @content-architect 维护。*
