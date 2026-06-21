---
id: 03-content.definitions.world.map-def
title: MapDef — Map Content Def 定义（MapAsset）
status: draft
owner: content-architect
created: 2026-06-22
updated: 2026-06-22
---

# MapDef — Map Content Def 定义（MapAsset）

> **Content Layer**: L4 World | **架构决策**: `docs/01-architecture/40-cross-cutting/ADR-065-map-content-pipeline.md` | **数据 Schema**: `docs/04-data/infrastructure/map-asset-schema.md`（待创建） | **插件代码**: `src/infra/map/asset.rs` + `src/content/content_plugin.rs`

---

## 1. Overview

MapDef（在代码中对应 `MapAsset` 类型）是 L4 World 层的**运行时地图数据格式**——Importer 从 Tiled TMX 转换而来的最终产物，是游戏直接消耗的地图资产。

### MapDef 的定位：运行时数据，非 TMX 映射

MapDef 不是 Tiled TMX 的 1:1 映射，而是针对运行时消费优化的数据结构：

```
Tiled (TMX)                ← 编辑格式：内容团队使用 Tiled 编辑
    │
    ▼  [Importer — 构建时]
MapAsset (RON)             ← 运行时格式：Importer 产物，游戏直接消费
    │
    ▼  [Content Pipeline — 启动时]
ECS State (GridMap + Ent)  ← 实例化：GridMap + Marker Entities + Renderer
```

MapDef 是 **Importer 的输出产物**，而不是手写的 RON 配置。通常不手动编写 MapDef RON 文件，而是通过 `tools/map_importer/` 工具从 TMX 自动生成。

### 设计原则

1. **Tile 只存 TerrainId**：每个 Tile 只存 `terrain_id: TerrainId`，Gameplay 数值（move_cost、defense_bonus）从 Config Registry 中的 TerrainDef 查询
2. **Object Layer 是一等公民**：对象层中的 MapObject 是运行时 ECS Entity 的模板定义，非直接实例
3. **L4 到 L3 的软引用**：SpawnPoints 通过 `spawn_group_id` 引用 L3 SpawnGroupDef，不直接引用具体单位
4. **NavigationMask 是预计算产物**：通行性导航掩码由 Importer 在构建时预计算，非运行时生成
5. **稳定 GUID**：所有 MapObject 和 SpawnPoint 携带 Importer 生成的稳定 GUID，用于跨存档追溯

### 与已有架构的对接

```
MapDef (MapAsset)
  │
  ├── terrain_grid  → GridMap (Resource)            ← Tactical Domain
  ├── object_layers → TileMarker + Domain Entities ← 实例化策略
  ├── spawn_points  → Encounter System              ← Combat Domain
  ├── regions       → MapRegion (未来 RegionQuery)   ← 预留
  └── navigation_mask → MovementSystem (加速结构)     ← Tactical Domain
```

### 跨文档引用

| 文档 | 内容 |
|------|------|
| `ADR-065-map-content-pipeline.md` | 完整地图管线架构（Importer、渲染、场景生命周期） |
| `ADR-022-grid-terrain-faction.md` | GridMap 架构、TileData packed 格式 |
| `tactical_domain.md` | 网格移动规则、寻路 API |
| `terrain_domain.md` | 地形类型、通行性、遮蔽度 |
| `terrain-def.md` | 本 Def 依赖的 L0 TerrainDef |
| `encounter-def.md` | L3 EncounterDef（通过 spawn_point 间接关联） |
| `spawn-group-def.md` | L3 SpawnGroupDef（本 Def 的 spawn_points 引用目标） |
| `content-layering.md` | L4 World 层约束（可引用 L0-L3，禁止被 L3 引用） |

---

## 2. Def 结构定义

```rust
use bevy_asset::Asset;
use bevy_reflect::TypePath;
use serde::Deserialize;

/// 运行时地图资产——Importer 从 Tiled TMX 转换而来。
///
/// MapDef（代码名 MapAsset）是 L4 World 层的运行时地图数据。
/// 它不可变、版本可控、不包含业务逻辑。
///
/// 详见 ADR-065 §3 MapAsset 结构定义。
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct MapAsset {
    // ── 地图元数据 ──
    pub metadata: MapMetadata,

    // ── 核心数据 ──
    /// 地形网格数据（核心数据，定义地形布局）
    pub terrain_grid: TerrainGrid,

    /// 对象层列表（一等公民）
    pub object_layers: Vec<ObjectLayer>,

    /// 出生点列表（单位生成位置，引用 L3 SpawnGroupDef）
    pub spawn_points: Vec<SpawnPoint>,

    /// 区域/命名范围（预留：未来 Region 系统的数据基础）
    pub regions: Vec<MapRegion>,

    /// 通行性导航掩码（Importer 预计算，运行时加速）
    pub navigation_mask: NavigationMask,
}
```

### MapMetadata

```rust
/// 地图元数据——标识和尺寸信息
#[derive(Deserialize, Clone, Debug)]
pub struct MapMetadata {
    // ── 统一标识字段 ──
    /// 地图 Def ID（全局唯一，ID 前缀: `map:`）
    ///
    /// 此 ID 被场景系统用于指定加载哪个地图，也被 L4 的
    /// RegionDef/NarrativeArcDef 等 Def 引用。
    pub id: MapId,

    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,

    /// Schema 版本号
    pub schema_version: u32,

    // ── 网格尺寸 ──
    /// 网格宽度（格子数）
    pub width: u32,
    /// 网格高度（格子数）
    pub height: u32,
    /// 网格布局类型
    pub layout: GridLayout,

    // ── 像素尺寸（Importer 从 TMX 元数据复制） ──
    /// Tiled 原始像素宽度（用于坐标转换参考）
    pub pixel_width: u32,
    /// Tiled 原始像素高度（用于坐标转换参考）
    pub pixel_height: u32,
    /// 每格像素宽度（Importer 从 TMX tileset 配置读取）
    pub tile_width: u32,
    /// 每格像素高度（Importer 从 TMX tileset 配置读取）
    pub tile_height: u32,
}
```

### TerrainGrid & TileEntry

```rust
/// 地形网格——地图的核心数据结构
///
/// 按行优先布局，总格子数 = width * height。
/// 运行时由 MapLoader 转换为 GridMap Resource。
#[derive(Deserialize, Clone, Debug)]
pub struct TerrainGrid {
    /// 宽度（格数）
    pub width: u32,
    /// 高度（格数）
    pub height: u32,
    /// 按行优先排列的 Tile 数据
    /// 索引公式: index = y * width + x
    pub tiles: Vec<TileEntry>,
}

/// 每个 Tile 的运行时数据
///
/// 🟥 禁止存储 Gameplay 数值（move_cost/defense_bonus/avoid_bonus）。
/// 这些数值在 TerrainDef Registry 中，Tile 只存 TerrainId。
/// 详见 ADR-065 §4 Tile → Config 映射策略。
#[derive(Deserialize, Clone, Debug)]
pub struct TileEntry {
    /// 地形类型 ID（唯一标识，查 Registry 可得完整 TerrainDef）
    ///
    /// L0 TerrainDef 的运行时引用。消费方通过此 ID 从 DefRegistry<TerrainDef>
    /// 获取 move_cost、defense_bonus 等 Gameplay 数值。
    pub terrain_id: TerrainId,

    /// 高度（0-255，TileData 已预留给定字段）
    ///
    /// 0 = 地面层，1-254 = 高地层，255 = 保留（天空/不可达）
    /// 由 Importer 从 TMX Tile 的 "height" Custom Property 填充。
    pub height: u8,

    /// 位标记：通行/可飞行/可建造/阻挡视线
    ///
    /// Importer 从 TerrainDef.flags 转换而来。
    /// 运行时直接查询此字段，不经过 TerrainDef Registry。
    pub flags: TileFlags,

    /// 旋转（0-3，90 度递增）
    ///
    /// Tiled Tile 的旋转属性，用于渲染时 Sprite 旋转。
    /// 不影响 Gameplay。
    pub rotation: u8,

    /// Tiled 原始 Tile ID（仅用于调试追溯，运行时可不加载）
    ///
    /// 在 debug 构建中用于回溯某个 Tile 在 TMX 中的原始索引。
    /// release 构建中此字段不存在。
    pub gid: Option<u32>,
}
```

### ObjectLayer & MapObject

```rust
/// 对象层——地图上的一层对象定义
///
/// 对象层是 MapDef 的一等公民。
/// 运行时由 ObjectInstantiator 根据 class 映射策略实例化 ECS Entity。
#[derive(Deserialize, Clone, Debug)]
pub struct ObjectLayer {
    /// Tiled 原始层 ID（仅调试用）
    pub id: u32,
    /// 层名称
    pub name: String,
    /// 透明度
    pub opacity: f32,
    /// 是否可见
    pub visible: bool,
    /// 层像素偏移
    pub offset_x: i32,
    pub offset_y: i32,
    /// 本层所有对象（稳定 GUID）
    pub objects: Vec<MapObject>,
}

/// 地图对象——运行时生成 ECS Entity 的模板定义
///
/// Object 是定义而非实例。实例化策略由 Domain 系统决定。
#[derive(Deserialize, Clone, Debug)]
pub struct MapObject {
    /// 稳定 GUID（Importer 内容哈希生成）
    pub guid: MapObjectGuid,
    /// Tiled 原始 ID（仅调试用）
    pub tiled_id: u32,
    /// 对象名称
    pub name: String,
    /// 对象类型/Custom Class（Tiled Class 名）——实例化的依据
    pub class: String,
    /// 网格位置（Importer 从像素坐标转换）
    pub position: GridPos,
    /// 尺寸（格）
    pub width: u32,
    pub height: u32,
    /// 旋转角度
    pub rotation: f32,
    /// 自定义属性映射
    pub properties: PropertyMap,
    /// 形状（碰撞/区域判定）
    pub shape: ObjectShape,
}

/// 稳定 GUID——全局唯一、跨存档稳定
///
/// 由 Importer 使用内容哈希生成，不依赖 Tiled ID。
/// GUID 生成: hash(map_id, layer_name, object_class, tile_x, tile_y)
pub struct MapObjectGuid(u64);

/// 属性映射——泛型键值对容器
///
/// 消费方是运行时 Domain 系统（InteractionSystem、HazardSystem 等）。
/// 不承载核心 Gameplay 数值——仅用于标记和配置覆盖。
#[derive(Deserialize, Clone, Debug)]
pub struct PropertyMap {
    pub entries: HashMap<String, PropertyValue>,
}

#[derive(Deserialize, Clone, Debug)]
pub enum PropertyValue {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Color([f32; 4]),
    File(String),
}

/// 对象形状——用于碰撞/区域判定
#[derive(Deserialize, Clone, Debug)]
pub enum ObjectShape {
    Point,
    Rectangle { width: u32, height: u32 },
    Ellipse { width: u32, height: u32 },
    Polygon { points: Vec<(f32, f32)> },
    Polyline { points: Vec<(f32, f32)> },
}
```

### Class 映射策略

MapObject 的 `class: String` 是运行时实例化的唯一依据。以下 class 映射由 Domain 系统实现，属于运行时策略而非 MapDef 定义：

| Class | 实例化目标 | 负责 Domain | 消费的 Property |
|-------|-----------|-------------|----------------|
| `SpawnPoint` | → SpawnPoint 结构（见下节） | Combat Domain | faction, facing, spawn_group_id |
| `Chest` | → [Interaction] Chest Entity | Interaction Domain | event_id, locked, key_id |
| `Door` | → [Interaction] Door Entity | Interaction Domain | locked, key_id, auto_close |
| `Hazard` | → HazardZone | Terrain Domain | hazard_id, damage, trigger_condition |
| `Trigger` | → [TriggerZone] Trigger Entity | Narrative Domain | event_id, scene_id, one_shot |
| `Region` | → MapRegion（见下节） | —（v1 仅数据存储） | — |
| `Decor` | → Visual Entity Only | MapRenderer | sprite, animation, scale |

未识别 Class → 发出警告并跳过实例化。

### SpawnPoint

```rust
/// 出生点——单位生成位置
///
/// spawn_point 通过 spawn_group_id 引用 L3 SpawnGroupDef，
/// 遵循 L4 → L3 的合法引用方向。
///
/// SpawnPoint 不直接引用具体单位（MonsterDef），而是引用
/// SpawnGroupDef 作为"单位模板组"。这使平衡性调整可以
/// 在 SpawnGroupDef 中完成，无需修改地图文件。
#[derive(Deserialize, Clone, Debug)]
pub struct SpawnPoint {
    /// 稳定 GUID
    pub guid: MapObjectGuid,

    /// 生成组 ID（引用 L3 SpawnGroupDef）
    ///
    /// 软引用——SpawnGroupDef 中定义了具体引用的 MonsterDef 和数量。
    /// Content Pipeline 验证此 ID 在 DefRegistry<SpawnGroupDef> 中存在。
    pub spawn_group_id: SpawnGroupId,

    /// 网格位置
    pub position: GridPos,

    /// 阵营（引用 L0 FactionDef）
    ///
    /// 覆盖 SpawnGroupDef 中定义的阵营。
    /// None = 使用 SpawnGroupDef 指定的阵营。
    pub faction: Option<FactionId>,

    /// 朝向
    pub facing: HexDirection,

    /// 额外属性（实现级别，用于存储队伍 ID、初始状态等）
    pub properties: PropertyMap,
}
```

### MapRegion

```rust
/// 区域——地图上的命名 Tile 集合
///
/// v1 仅做数据存储，不提供运行时查询 API。
/// 数据基础为未来 Region 系统（区域触发、AI 区域感知等）预留。
#[derive(Deserialize, Clone, Debug)]
pub struct MapRegion {
    /// 区域标识符（字符串 ID，不要求全局唯一）
    pub id: String,
    /// 显示名称（本地化 Key）
    pub name_key: LocalizationKey,
    /// 包含的网格位置集合
    pub tiles: Vec<GridPos>,
    /// 区域属性
    pub properties: PropertyMap,
}
```

### NavigationMask（Importer 预计算）

```rust
/// 通行性导航掩码——Importer 在构建时预计算
///
/// 每个 Tile 一个 byte，bitfield 表示不同移动类型的通行性。
/// 运行时作为 GridMap 寻路的加速结构，GridMap 本身仍是通行性的权威数据源。
///
/// 🟥 此字段由 Importer 生成，不得手动编写。
/// 🟥 运行时 GridMap 的数据变更需同步更新此掩码（v1 不做同步，仅用于初始加速）。
#[derive(Deserialize, Clone, Debug)]
pub struct NavigationMask {
    pub width: u32,
    pub height: u32,
    /// 每个 Tile 一个 byte，bitfield 表示不同移动类型
    pub data: Vec<u8>,
}
```

### 网格布局

```rust
/// 网格布局类型——与 Tactical Domain 的 GridLayout 一致
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum GridLayout {
    /// 四向网格（简单）
    Square,
    /// 六边形，奇数列偏移
    HexRowOdd,
    /// 六边形，偶数列偏移
    HexRowEven,
    /// 六边形，奇数行偏移
    HexColOdd,
    /// 六边形，偶数列偏移
    HexColEven,
}
```

### Importer 生成字段一览

以下字段由 `tools/map_importer/` 生成，内容创作者**不应手动编写**：

| 字段 | 归属 | 生成方式 |
|------|------|----------|
| `MapObject.guid` | ObjectLayer | 内容哈希（map_id + layer + class + position） |
| `SpawnPoint.guid` | SpawnPoints | 同上 |
| `TileEntry.flags` | TerrainGrid | 从 TerrainDef.flags 转换 |
| `TileEntry.position`（通过 GridPos 构造） | 所有引用 Tile 的结构 | TMX 像素坐标 → GridPos 转换 |
| `NavigationMask` | 顶级字段 | 从 TerrainGrid + TerrainDef.flags 预计算 |
| `MapMetadata.pixel_width/height` | Metadata | 从 TMX map 元数据复制 |
| `MapMetadata.tile_width/height` | Metadata | 从 TMX tileset 配置读取 |
| `TileEntry.gid` | TerrainGrid | TMX 原始 GID（仅 debug 构建） |

---

## 3. Registry 模式

MapAsset 是 Bevy Asset，通过 AssetServer 加载，不进入 `DefRegistry<T>`：

```rust
use bevy::asset::Asset;
use bevy::prelude::*;

/// MapAsset 注册在 Infra 层的 MapPlugin 中
pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        // 注册 Asset 类型
        app.init_asset::<MapAsset>();

        // 注册场景生命周期系统
        app.add_systems(OnEnter(GameState::TacticalMap), load_map);
        app.add_systems(OnExit(GameState::TacticalMap), cleanup_map);
        app.add_systems(OnEnter(GameState::Combat), load_map);
        app.add_systems(OnExit(GameState::Combat), cleanup_map);

        // 注册 MapLoader（MapAsset → GridMap + ECS State）
        app.add_plugins(MapRendererPlugin);
    }
}

/// Content Plugin 中注册 MapAsset 的加载和 Hot-reload
impl Plugin for ContentPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<MapAsset>()
           .add_observer(on_map_asset_added);
    }
}
```

### 为什么 MapAsset 不使用 DefRegistry？

MapAsset 与典型的 Content Def 有以下差异：

| 维度 | Content Def（TagDef, FactionDef 等） | MapAsset |
|------|--------------------------------------|----------|
| 数据规模 | 小巧（几十到几百个 Def 在一个 RON 文件中） | 庞大（100x100 网格 + 对象层，单个文件可能数 MB） |
| 查询模式 | 按 ID 频繁查询 | 按位置随机访问（GridPos → TileData） |
| 生命周期 | 全局唯一，应用全生命周期常驻 | 地图级，在场景 OnEnter/OnExit 时加载/卸载 |
| 存储结构 | HashMap 式注册表 | 连续数组（Tile 数据为 packed u32） |
| 消费方式 | 通过 `registry.get(id)` 查询 | 通过 `GridMap.get_tile(pos)` 查询 |
| 创建方式 | 手写 RON / 编辑器工具 | Importer 从 TMX 自动生成 |

MapAsset 作为 Bevy Asset 直接管理，不进入 DefRegistry。它的查询接口由 `GridMap` Resource（Tactical Domain）和 `MapQuery` Facade（Infra 层）提供。

### 内容资产目录位置

MapAsset 的 RON 资产位于 L4 目录 `assets/config/04_world/maps/`：

```
assets/config/04_world/
├── maps/
│   ├── map_dragon_peak.ron          ← MapAsset（Importer 输出）
│   ├── map_dark_forest.ron
│   └── map_training_grounds.ron
├── regions.ron                       ← RegionDef（待定义）
├── scenes.ron                        ← SceneDef（待定义）
└── cutscenes.ron                     ← CutsceneDef（待定义）
```

与 Content Def 不同，MapAsset 遵循**单文件单地图**原则（每张地图一个独立的 RON 文件），而非单文件多 Def。原因是地图文件可能很大（包含整个 terrain_grid），不适合放在集合文件中。

---

## 4. 校验规则

### 4.1 Schema 级校验（Importer 构建时执行）

| # | 规则 | 说明 |
|---|------|------|
| V1 | 尺寸一致性 | `terrain_grid.width * terrain_grid.height == terrain_grid.tiles.len()` |
| V2 | 无空洞 | 所有 grid 位置有有效 TileEntry |
| V3 | GUID 唯一性 | 所有 ObjectLayer 中无重复 GUID |
| V4 | 尺寸合法性 | width > 0, height > 0, 且 ≤ 最大限制（v1: 256x256） |
| V5 | `metadata.id` 非空 | MapId 不能为空字符串 |
| V6 | `metadata.id` 格式合法 | 必须匹配 `^map:[a-z][a-z0-9_]+$`（如 `map:dragon_peak`） |
| V7 | `schema_version` 兼容 | 当前支持的版本为 1 |

### 4.2 语义级校验（Importer 构建时执行）

| # | 规则 | 说明 |
|---|------|------|
| V8 | TerrainId 有效性 | 每个 `TileEntry.terrain_id` 在 TerrainDef Registry 中存在 |
| V9 | SpawnGroupId 有效性 | 每个 `SpawnPoint.spawn_group_id` 在 DefRegistry<SpawnGroupDef> 中存在（警告而非报错——允许地图引用后续加载的 Def） |
| V10 | 高度连续性 | 相邻 Tile 高度差 ≤ 3（最大坡度，警告） |
| V11 | 必要属性 | Object 的必需 Property 存在（如 Trigger 需要 event_id，警告） |
| V12 | 导航一致性 | NavigationMask 与 terrain_id 的通行性一致（警告） |

### 4.3 运行时校验（MapLoader，仅 debug 模式）

| # | 规则 | 说明 |
|---|------|------|
| V13 | 快速完整性 | width * height == tiles.len() |
| V14 | SpawnGroupId 二次校验 | 所有 SpawnPoint 的 spawn_group_id 在 L3 已注册 |
| V15 | GUID 无冲突 | 实例化时检测 GUID 冲突 |

### 4.4 L4 层间校验

| # | 规则 | 说明 |
|---|------|------|
| V16 | 跨层引用合规 | MapDef 引用 L0 合法（TerrainId, FactionId）、L3 合法（SpawnGroupId）、禁止引用 L4 自身以外的同层类型（当前仅此一个 L4 Def 类型） |
| V17 | 无 Gameplay 数值嵌入 | TileEntry 不包含 move_cost/defense_bonus 等 Gameplay 数值——这些必须通过 TerrainDef Registry 查询 |

---

## 5. RON 示例（MapAsset）

以下示例展示 Importer 输出的 MapAsset RON 格式。内容创作者通常不手写此文件——它由 `tools/map_importer/` 从 TMX 自动生成。

```ron
(
    metadata: (
        id: "map:dragon_peak",
        name_key: "map.map_dragon_peak.name",
        schema_version: 1,

        width: 30,
        height: 20,
        layout: HexRowOdd,

        pixel_width: 1920,
        pixel_height: 1280,
        tile_width: 64,
        tile_height: 64,
    ),

    terrain_grid: (
        width: 30,
        height: 20,
        tiles: [
            // ⚠ 此处省略 600 个 TileEntry——实际文件由 Importer 生成
            // 每个 TileEntry 格式：
            // (terrain_id: "ter:plain", height: 0, flags: TileFlags(PASSABLE|FLYABLE), rotation: 0, gid: Some(1)),
            // ...
        ],
    ),

    object_layers: [
        (
            id: 1,
            name: "interactables",
            opacity: 1.0,
            visible: true,
            offset_x: 0,
            offset_y: 0,
            objects: [
                (
                    guid: 0x4A1B2C3D,
                    tiled_id: 101,
                    name: "chest_01",
                    class: "Chest",
                    position: (5, 8),
                    width: 1,
                    height: 1,
                    rotation: 0.0,
                    properties: (
                        entries: {
                            "event_id": String("evt:dragon_peak_chest_01"),
                            "locked": Bool(true),
                            "key_id": String("item:dragon_key"),
                        },
                    ),
                    shape: Rectangle(width: 1, height: 1),
                ),
                (
                    guid: 0x5C2D3E4F,
                    tiled_id: 102,
                    name: "door_main",
                    class: "Door",
                    position: (12, 3),
                    width: 1,
                    height: 1,
                    rotation: 0.0,
                    properties: (
                        entries: {
                            "locked": Bool(false),
                        },
                    ),
                    shape: Point,
                ),
            ],
        ),
        (
            id: 2,
            name: "hazards",
            opacity: 0.8,
            visible: false,
            offset_x: 0,
            offset_y: 0,
            objects: [
                (
                    guid: 0x6D3E4F5A,
                    tiled_id: 201,
                    name: "poison_trap_01",
                    class: "Hazard",
                    position: (8, 12),
                    width: 2,
                    height: 2,
                    rotation: 0.0,
                    properties: (
                        entries: {
                            "hazard_id": String("haz:poison_swamp"),
                            "trigger_condition": String("on_enter"),
                        },
                    ),
                    shape: Square,
                ),
            ],
        ),
    ],

    spawn_points: [
        (
            guid: 0xE4F5A6B7,
            spawn_group_id: "spawn:dragon_cultists",
            position: (25, 10),
            faction: Some("faction:enemy"),
            facing: West,
            properties: (
                entries: {},
            ),
        ),
        (
            guid: 0xF5A6B7C8,
            spawn_group_id: "spawn:dragon_elder",
            position: (15, 10),
            faction: Some("faction:enemy"),
            facing: South,
            properties: (
                entries: {
                    "is_boss": Bool(true),
                    "phase_trigger": String("evt:dragon_intro"),
                },
            ),
        ),
    ],

    regions: [
        (
            id: "dragon_throne_room",
            name_key: "region.dragon_throne_room.name",
            tiles: [
                (13, 5), (14, 5), (15, 5), (16, 5),
                (13, 6), (14, 6), (15, 6), (16, 6),
                (13, 7), (14, 7), (15, 7), (16, 7),
            ],
            properties: (
                entries: {
                    "encounter_id": String("enc:dragon_peak_boss"),
                },
            ),
        ),
        (
            id: "safe_zone",
            name_key: "region.safe_zone.name",
            tiles: [
                (2, 2), (2, 3), (3, 2), (3, 3),
            ],
            properties: (
                entries: {
                    "no_combat": Bool(true),
                },
            ),
        ),
    ],

    navigation_mask: (
        width: 30,
        height: 20,
        // ⚠ 600 bytes 的通行性位掩码，由 Importer 预计算
        // 此处省略数据——实际文件中此字段为完整 Vec<u8>
        data: [],
    ),
)
```

---

## 6. 与 L3 EncounterDef 的软引用模式

MapDef 与 L3 EncounterDef 通过两种方式建立关联，均遵循 L4 → L3 的合法引用方向：

### 方式一：SpawnPoint 引用 SpawnGroupDef

```
MapDef (L4)
  └── spawn_points: [
        (guid: ..., spawn_group_id: "spawn:dragon_cultists", position: (25, 10)),
        (guid: ..., spawn_group_id: "spawn:dragon_elder",   position: (15, 10)),
      ]
        │
        ▼
DefRegistry<SpawnGroupDef> (L3)
  ├── "spawn:dragon_cultists" → { monster_defs: [...], count: 2, ... }
  └── "spawn:dragon_elder"    → { monster_defs: [...], count: 1, ... }
```

MapDef 不直接引用 EncounterDef——通过 SpawnGroupDef 间接关联单位数据。Content Pipeline 在 L3 加载完成后二次校验所有 SpawnGroupId 引用。

### 方式二：Region Property 关联 EncounterDef（内容约定）

Region 的 PropertyMap 可以包含 `"encounter_id"` key，用于标记特定区域触发哪个 Encounter：

```ron
regions: [
    (
        id: "boss_arena",
        tiles: [...],
        properties: (
            entries: {
                "encounter_id": String("enc:dragon_peak_boss"),
            },
        ),
    ),
]
```

这种关联是内容层面的约定（非结构性引用），不属于 Content Pipeline 的校验范围。运行时由 Domain System 读取 Property 并手动触发对应 Encounter。

### 为什么不直接引用 EncounterDef？

ADR-065 和 Content Layering 规则禁止 L3 引用 L4（EncounterDef 不可引用 MapDef），但允许 L4 引用 L3（MapDef 引用 EncounterDef）。两种间接模式的设计意图：

1. **解耦 Encounter 与地图位置**：EncounterDef 保持地图无关性，可被多个 MapDef 复用同一个 Encounter 配置
2. **平衡性调整不改地图**：改单位类型/数量只需要改 SpawnGroupDef，不需要改 MapAsset
3. **内容团队职责分离**：关卡设计师管理 MapDef（位置布局），系统设计师管理 EncounterDef（战斗配置）

---

## 7. 场景生命周期

MapDef 的加载和卸载由场景生命周期管理（符合 ADR-050 游戏状态机架构）：

```
OnEnter(TacticalMap / Combat)
  │
  ├── 1. 场景数据通道读取 map_asset_id（来自 MapLoadParams Resource）
  ├── 2. AssetServer::load::<MapAsset>(path) 获取 Handle
  ├── 3. 解析 TerrainGrid → GridMap Resource
  ├── 4. 创建 TileMarker Entity（交互式 Tile 实体化）
  ├── 5. 按 class 策略实例化 ObjectLayer → ECS Entity
  ├── 6. 初始化 MapRenderer（生成 Tile Sprite Entity）
  ├── 7. 标记所有 Entity 为 SceneRoot 的子级
  └── 8. 触发 MapLoaded Event

Update
  ├── Domain Systems (tactical, terrain, combat) 消费 GridMap
  └── MapRenderer (tick animations, update overlays)

OnExit(TacticalMap / Combat)
  └── cleanup_scene System → Despawn SceneRoot 所有子级
```

MapLoader 职责仅限于 "MapDef → ECS State" 的转换，**不包含业务逻辑**（单位生成、战斗初始化均由对应 Domain 系统处理）。详见 ADR-065 §8。

---

## 8. 设计说明

### MapDef 为什么是 L4 World 层？

四条判定标准全部满足 L4 归属：

1. **具体世界呈现**：MapDef 描述了"某张具体地图"（dragon_peak、dark_forest）的布局，而非抽象的规则或实体
2. **变更最频繁的层**：地图是内容创作中迭代最多的资产，需要独立版本控制
3. **引用所有下层**：MapDef 引用 L0 TerrainId/FactionId、L3 SpawnGroupId
4. **被叙事内容引用**：RegionDef、NarrativeArcDef、SceneDef 通过 MapId 引用具体地图

### MapDef vs MapAsset 命名说明

文档中 MapDef 指内容定义层面，代码中对应的 Rust 类型名称为 `MapAsset`（Bevy Asset）。两者是同一概念的不同视角：MapDef 强调它是 L4 的一类 Content Def，MapAsset 强调它是 Bevy Asset 系统中的一个 Asset 类型。

相似的模式：EncounterDef 对应 `EncounterDef` 在代码中也是 Asset 类型。MapAsset 命名不同的原因是它不由 Content Pipeline 的 `DefRegistry` 管理，而是作为 Bevy Asset 直接加载。

### NavigationMask 的预计算策略

NavigationMask 采用 Importer 预计算而不是运行时动态生成的决策理由：

- **构建时开销 vs 运行时开销**：SRPG 地图的通行性在运行时极少改变（除非有地形破坏/建造），预计算在构建时一次性完成，运行时零开销
- **一致性保证**：NavigationMask 与 terrain_id 的通行性在 Importer 阶段同步校验，避免运行时不一致
- **加速结构**：寻路系统使用 NavigationMask 作为快速过滤层，GridMap 仍是通行性的权威数据源

v1 不做 NavigationMask 的运行时同步——如果 terrain_id 在运行时被修改，NavigationMask 不会自动更新。这是有意设计，因为 v1 不支持运行时地形修改（ADR-065 §13 明确排除）。

### Object Layer 的未来扩展

当前 MapDef 的对象层设计是通用的（class + PropertyMap），支持未来在不修改 MapDef 格式的情况下扩展：

- 新增 Class 类型：只需在 ObjectInstantiator 中添加新的 class → Entity 映射
- 新增 Property：不需要修改 MapDef 结构，新系统读取 PropertyMap 中对应的 key
- 嵌套 Object Group：Tiled 支持 Group Layer，可在未来 MapDef 版本中增加 children 字段

---

*本文档由 @content-architect 维护。*
