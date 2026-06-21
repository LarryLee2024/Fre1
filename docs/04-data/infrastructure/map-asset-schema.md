---
id: infrastructure.map-asset.schema.v1
title: MapAsset Schema — 运行时地图数据架构
status: draft
owner: data-architect
created: 2026-06-22
updated: 2026-06-22
layer: definition
replay-safe: true
supersedes: none
---

# MapAsset Schema — 运行时地图数据架构

> **领域归属**: Infrastructure — Map | **依赖 Schema**: Tactical (GridPos, TileData), Terrain (TerrainId, TileFlags) | **定义依据**: `docs/01-architecture/40-cross-cutting/ADR-065-map-content-pipeline.md`, `docs/03-content/definitions/world/map-def.md`

---

## 1. Domain Ownership

| 数据类别 | 归属层 | 说明 |
|----------|--------|------|
| `MapAsset` | Definition | 运行时地图资产（Bevy Asset），Importer 产物，不可变 |
| `MapMetadata` | Definition | 地图元数据嵌套结构 |
| `TerrainGrid` | Definition | 地形网格数据（String terrain_id 格式，非 packed） |
| `TileEntry` | Definition | 单个 Tile 数据（terrain_id 为 String 引用） |
| `ObjectLayer` | Definition | 对象层，一等公民 |
| `MapObject` | Definition | 地图对象定义，ECS Entity 模板 |
| `PropertyMap` | Definition | 泛型键值对容器 |
| `MapObjectGuid` | Definition | 稳定 GUID（Importer 使用内容哈希生成） |
| `SpawnPoint` | Definition | 出生点定义（引用 L3 SpawnGroupDef） |
| `MapRegion` | Definition | 区域/命名范围（v1 仅数据存储） |
| `NavigationMask` | Definition | 预计算通行性导航掩码（Importer 生成） |

### 数据流归属

MapAsset 是运行时地图数据格式。它属于 Definition 层的一个特例——虽然是运行时加载的数据，但它不可变、版本可控、不包含业务逻辑。与标准 Definition 的关键区别：

- **不是 DefRegistry 的成员**：MapAsset 是 Bevy Asset，通过 `AssetServer::load::<MapAsset>()` 加载
- **场景级生命周期**：在场景 `OnEnter` → `OnExit` 时加载/卸载，而非应用全生命周期常驻
- **位置访问模式**：通过 `GridPos → TileData` 位置随机访问，非 ID 查找

### 两个 Tile 表示

系统中有两个不同的 Tile 数据结构，分别服务于不同阶段：

| 数据结构 | 所在层 | 位置 | TerrainId 格式 | 用途 |
|----------|--------|------|---------------|------|
| `TileEntry` | Definition | `MapAsset.terrain_grid.tiles` | `TerrainId` (String, e.g. `"ter:plain"`) | RON 文件格式，人类可读 |
| `TileData` | Instance | `GridMap.tiles` (packed u32) | `u16` (packed 高位 16bit) | 运行时内存格式，性能优先 |

转换发生在 MapLoader（`src/infra/map/loader.rs`）：

```
TileEntry (MapAsset, RON)           TileData (GridMap, packed u32)
  ├── terrain_id: "ter:plain"  ──→    ├── terrain_def_id: u16 (Registry 索引)
  ├── height: 0                 ──→    ├── height: u8
  ├── flags: TileFlags(0x03)    ──→    ├── flags: TileFlags (u8)
  └── rotation: 0               ──→    └── (不存储)
```

---

## 2. Schema Design

### 2.1 MapAsset（顶级结构）

```rust
use bevy::asset::Asset;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

/// 运行时地图资产——Importer 从 Tiled TMX 转换而来。
///
/// MapAsset 是 L4 World 层的运行时地图数据。
/// 不可变、版本可控、不包含业务逻辑。
/// 不是 DefRegistry 成员——通过 Bevy AssetServer 加载。
///
/// 🟥 禁止运行时修改。
/// 🟥 禁止承载业务逻辑（寻路、战斗结算等）。
///
/// 详见 ADR-065 §3。
#[derive(Asset, TypePath, Deserialize, Serialize, Clone, Debug)]
pub struct MapAsset {
    /// 地图元数据（标识、尺寸、布局）
    pub metadata: MapMetadata,

    /// 地形网格数据（核心数据）
    pub terrain_grid: TerrainGrid,

    /// 对象层列表（一等公民）
    #[serde(default)]
    pub object_layers: Vec<ObjectLayer>,

    /// 出生点列表
    #[serde(default)]
    pub spawn_points: Vec<SpawnPoint>,

    /// 区域/命名网格范围集合
    #[serde(default)]
    pub regions: Vec<MapRegion>,

    /// 通行性导航掩码（Importer 预计算，运行时只读）
    pub navigation_mask: NavigationMask,
}
```

**`#[serde(default)]` 策略**：`object_layers`、`spawn_points`、`regions` 允许为空集合。一张没有交互对象的地图（纯演示/测试地图）是合法的。`terrain_grid` 和 `navigation_mask` 不使用 default——地图必须有网格数据。

### 2.2 MapMetadata

```rust
/// 地图元数据——标识和尺寸信息。
///
/// 所有像素字段由 Importer 从 TMX 元数据复制。
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MapMetadata {
    // ── 统一标识 ──

    /// 地图 Def ID（全局唯一，前缀: `map:`）
    ///
    /// 格式：`^map:[a-z][a-z0-9_]+$`
    /// 示例：`"map:dragon_peak"`, `"map:dark_forest"`
    ///
    /// 此 ID 被场景系统用于指定加载哪个地图。
    pub id: MapId,

    /// 显示名称本地化 Key
    pub name_key: LocalizationKey,

    /// Schema 版本号（当前版本: 1）
    pub schema_version: u32,

    // ── 网格尺寸 ──

    /// 网格宽度（格子数），> 0
    pub width: u32,

    /// 网格高度（格子数），> 0
    pub height: u32,

    /// 网格布局类型
    pub layout: GridLayout,

    // ── 像素尺寸（Importer 从 TMX 元数据复制，用于坐标转换） ──

    /// Tiled 原始像素宽度（pixel_width = tile_width * width，若为方形网格）
    pub pixel_width: u32,

    /// Tiled 原始像素高度（pixel_height = tile_height * height）
    pub pixel_height: u32,

    /// 每格像素宽度（从 TMX tileset 配置读取）
    pub tile_width: u32,

    /// 每格像素高度
    pub tile_height: u32,
}
```

**MapId 类型**：`MapId` 是 `String` 的 newtype 包装，在 `src/shared/ids/types.rs` 中定义。ID 前缀使用 `map:`，符合 L4 World 层的命名规范。

**schema_version 的使用**：Importer 在输出时设置 `schema_version: 1`。未来的格式变更通过版本号判断是否需要迁移。运行时 MapLoader 检查版本兼容性——不兼容的版本在 debug 模式下发出警告。

### 2.3 TerrainGrid & TileEntry

```rust
/// 地形网格——地图的核心数据结构。
///
/// 按行优先布局。总格子数 = width * height。
/// 索引公式: index = y * width + x
///
/// 🟥 禁止在此结构中存储 Gameplay 数值。
///     所有数值（move_cost, defense_bonus 等）从 TerrainDef Registry 查询。
///
/// 与已有 GridMap 的关系：
///   TerrainGrid (RON, String terrain_id)
///       → MapLoader 转换
///   GridMap.tiles (packed u32, u16 terrain_def_id)
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TerrainGrid {
    /// 宽度（格数），必须与 MapMetadata.width 一致
    pub width: u32,

    /// 高度（格数），必须与 MapMetadata.height 一致
    pub height: u32,

    /// 按行优先排列的 Tile 数据
    /// 索引公式: index = y * width + x
    pub tiles: Vec<TileEntry>,
}

/// 单个 Tile 的运行时数据。
///
/// 以人类可读的 String terrain_id 存储，而非 packed u32。
/// 优点：RON 文件可直接阅读、diff、review。
/// 缺点：比 packed 格式占用更多空间——但 MapAsset 仅用于加载，
/// 加载完成后压缩为 TileData (packed u32)，此内存开销不持续。
///
/// 🟥 禁止在此结构中存储 Gameplay 数值。
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct TileEntry {
    /// 地形类型 ID（String，如 `"ter:plain"`, `"ter:forest"`）
    ///
    /// 运行时通过此 ID 从 DefRegistry<TerrainDef> 查询 terrain 的
    /// move_cost、defense_bonus、concealment 等所有 Gameplay 数值。
    /// 详见 ADR-065 §4 Tile → Config 映射策略。
    pub terrain_id: TerrainId,

    /// 高度（0-255）
    ///
    /// 0 = 地面层
    /// 1-254 = 高地层
    /// 255 = 保留（天空/不可达）
    ///
    /// v1 寻路系统不区分高度，v2 加入高度影响。
    /// 视觉效果系统使用此值进行高度偏移渲染。
    pub height: u8,

    /// 位标记——PASSABLE, FLYABLE, BUILDABLE, BLOCKS_SIGHT
    ///
    /// 🟡 冗余字段：理论上从 terrain_id → TerrainDef.flags 可推导。
    ///     复制到 TileEntry 是为了：
    ///     1) 运行时 GridMap 寻路不需查 TerrainDef Registry
    ///     2) 支持运行时 TerrainDef 修改后旧地图仍用旧 flags
    ///
    /// Importer 从 TerrainDef.flags 转换并写入此字段。
    pub flags: TileFlags,

    /// 旋转（0-3，90 度递增）
    ///
    /// 仅影响渲染——Sprite 旋转。
    /// 不影响 Gameplay。
    pub rotation: u8,

    /// Tiled 原始 GID（仅用于调试追溯，运行时可不加载）
    ///
    /// debug 构建: 保留此字段，用于回溯某个 Tile 在 TMX 中的原始索引。
    /// release 构建: 此字段可通过 serde(skip) 移除以节约文件体积。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gid: Option<u32>,
}
```

**flags 冗余性说明**：`TileEntry.flags` 是从 `TerrainDef.flags` 复制过来的冗余字段。这是有意设计：

- **运行时性能**：GridMap 寻路每次需要检查 PASSABLE/FLYABLE，通过 packed `TileData.flags` 单次内存读取即可完成，不需查 DefRegistry
- **空间换时间**：每个 TileEntry 额外 1 byte（flags）+ 1 byte（rotation），以 100x100 地图计约 20KB 的 RON 文件额外开销——可接受
- **一致性保证**：Importer 在输出时验证 flags 与 terrain_id 对应 TerrainDef 一致

### 2.4 ObjectLayer & MapObject

```rust
/// 对象层——地图上的一层对象定义。
///
/// 对象层是 MapAsset 的一等公民。
/// 运行时由 ObjectInstantiator 根据 class 映射策略实例化 ECS Entity。
///
/// 设计哲学（ADR-065 §5）：
/// - Object 是定义而非实例
/// - class String 是实例化的唯一依据
/// - 运行时 System 决定是否/何时实例化
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ObjectLayer {
    /// Tiled 原始层 ID（仅用于调试追溯）
    pub id: u32,

    /// 层名称
    pub name: String,

    /// 透明度
    pub opacity: f32,

    /// 是否可见
    pub visible: bool,

    /// 层像素偏移 X
    pub offset_x: i32,

    /// 层像素偏移 Y
    pub offset_y: i32,

    /// 本层所有对象（携带稳定 GUID）
    pub objects: Vec<MapObject>,
}

/// 地图对象——运行时 ECS Entity 的模板定义。
///
/// Object 不是 Entity。实例化由 Domain System 通过 class 类型决定。
///
/// 稳定 GUID 保证跨存档/跨场景的身份追溯。
///
/// 🟥 Object 不直接实例化为 Entity。
/// 🟥 Object 不包含 Gameplay 数值逻辑。
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MapObject {
    /// 稳定 GUID（Importer 内容哈希生成）
    pub guid: MapObjectGuid,

    /// Tiled 原始 ID（仅用于调试追溯）
    pub tiled_id: u32,

    /// 对象名称（Tiled 中的 name 字段）
    pub name: String,

    /// 对象类型/Custom Class（Tiled Class 名）
    ///
    /// 这是运行时实例化的唯一依据。Domain System 根据此值决定：
    /// - "Chest" → InteractionSystem 实例化 Chest Entity
    /// - "Hazard" → TerrainDomain 实例化 HazardZone
    /// - "Decor" → MapRenderer 实例化 Visual Entity
    ///
    /// 见 MapDef 文档的 Class 映射策略表。
    pub class: String,

    /// 网格位置（Importer 从像素坐标转换）
    ///
    /// GridPos { x: i32, y: i32, layer: i8 }
    /// layer 通常为 0，多层地图时可使用
    pub position: GridPos,

    /// 尺寸（格）
    pub width: u32,

    /// 尺寸（格）
    pub height: u32,

    /// 旋转角度（度）
    pub rotation: f32,

    /// 自定义属性映射——泛型键值对容器
    pub properties: PropertyMap,

    /// 形状（用于碰撞/区域判定）
    pub shape: ObjectShape,
}

/// 地图对象稳定 GUID——全局唯一、跨存档稳定。
///
/// 由 Importer 使用确定性内容哈希生成。
/// GUID 生成算法:
///   hash(map_id, layer_name, object_class, tile_x, tile_y)
/// 使用 SipHash-2-4 确保相同输入总是产生相同输出。
///
/// GUID 不依赖 Tiled 内部 ID——Tiled ID 在重新编辑地图时可能变化。
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MapObjectGuid(pub u64);
```

### 2.5 PropertyMap & PropertyValue

```rust
/// 属性映射——泛型键值对容器。
///
/// 消费方是运行时 Domain System（InteractionSystem, HazardSystem 等）。
///
/// 🟥 不承载核心 Gameplay 数值。
///     适合标记和配置覆盖（event_id, locked, key_id 等）。
///     核心数值归 TerrainDef / EncounterDef / SpawnGroupDef 管理。
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PropertyMap {
    /// 键值对集合
    pub entries: HashMap<String, PropertyValue>,
}

/// 属性值类型——支持 Tiled 的所有原生 Property 类型。
///
/// 🟡 与 Tiled 支持的 Property 类型一致：
///   String, int, float, bool, color, file
#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum PropertyValue {
    /// 字符串值
    String(String),
    /// 整数值
    Int(i32),
    /// 浮点值
    Float(f32),
    /// 布尔值
    Bool(bool),
    /// 颜色值（RGBA，每个分量 0.0-1.0）
    Color([f32; 4]),
    /// 文件路径（Tiled 的 File 类型）
    File(String),
}

/// 对象形状——用于碰撞/区域判定。
///
/// Tiled 对象支持的所有形状类型。
/// Point 和 Rectangle 会被 Importer 从像素坐标→网格坐标转换，
/// 其他复杂形状保持原始像素坐标。
#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum ObjectShape {
    /// 点（无尺寸）
    Point,
    /// 矩形
    Rectangle {
        /// 宽度（Importer 转换为格数）
        width: u32,
        /// 高度（Importer 转换为格数）
        height: u32,
    },
    /// 椭圆
    Ellipse {
        /// 宽度
        width: u32,
        /// 高度
        height: u32,
    },
    /// 多边形
    Polygon {
        /// 顶点列表（像素坐标，相对于对象原点）
        points: Vec<(f32, f32)>,
    },
    /// 折线
    Polyline {
        /// 顶点列表（像素坐标，相对于对象原点）
        points: Vec<(f32, f32)>,
    },
}
```

### 2.6 SpawnPoint

```rust
/// 出生点——单位生成位置。
///
/// 通过 spawn_group_id 引用 L3 SpawnGroupDef，遵循 L4 → L3 的合法引用方向。
/// 不直接引用具体单位——平衡性调整改 SpawnGroupDef，不改地图文件。
///
/// 软引用策略（ADR-065 §4、MapDef §6）：
///   1. 解耦 Encounter 与地图位置
///   2. 平衡性调整不改地图
///   3. 内容团队职责分离
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct SpawnPoint {
    /// 稳定 GUID
    pub guid: MapObjectGuid,

    /// 生成组 ID（引用 L3 SpawnGroupDef）
    ///
    /// 格式: `"spawn:cultists"`, `"spawn:boss_group"`
    /// Content Pipeline 在 L3 加载完成后校验此 ID 的存在性。
    pub spawn_group_id: SpawnGroupId,

    /// 网格位置
    pub position: GridPos,

    /// 阵营（引用 L0 FactionDef）
    ///
    /// Some = 覆盖 SpawnGroupDef 中定义的阵营
    /// None = 使用 SpawnGroupDef 指定的阵营
    pub faction: Option<FactionId>,

    /// 朝向
    pub facing: HexDirection,

    /// 额外属性
    ///
    /// 常见用途：队伍 ID、初始状态、相位触发事件 ID
    #[serde(default)]
    pub properties: PropertyMap,
}
```

### 2.7 MapRegion

```rust
/// 区域——地图上的命名 Tile 集合。
///
/// v1 仅做数据存储，不提供运行时 Region 查询 API。
/// 数据基础为未来 Region 系统（区域触发、AI 区域感知等）预留。
///
/// v1 不做的功能（ADR-065 §12）：
/// - 运行时 region.contains(pos) 查询 API
/// - Region 触发的事件
/// - AI 区域感知
/// - Region 嵌套与层次结构
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct MapRegion {
    /// 区域标识符（字符串 ID，同一地图内唯一）
    pub id: String,

    /// 显示名称本地化 Key
    pub name_key: LocalizationKey,

    /// 包含的网格位置集合
    pub tiles: Vec<GridPos>,

    /// 区域属性
    ///
    /// 常见用法：
    ///   "encounter_id": String("enc:dragon_peak_boss")
    ///   "no_combat": Bool(true)
    ///   "hazard_id": String("haz:poison_swamp")
    #[serde(default)]
    pub properties: PropertyMap,
}
```

### 2.8 NavigationMask

```rust
/// 通行性导航掩码——Importer 在构建时预计算。
///
/// 每个 Tile 一个 byte，bitfield 表示不同移动类型的通行性。
///
/// 寻路系统使用流程：
///   1. NavigationMask 作为快速过滤层（单 byte 读取）
///   2. 通过过滤的 Tile 再从 GridMap 获取完整 TileData
///   3. GridMap 始终是通行性的权威数据源
///
/// 🟥 此字段由 Importer 生成，内容创作者不应手动编写。
/// 🟥 v1 不做 NavigationMask 的运行时同步。如果 terrain_id
///     在运行时被修改，NavigationMask 不会自动更新——因为 v1
///     不支持运行时地形修改（ADR-065 §13 明确排除）。
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct NavigationMask {
    /// 掩码宽度（格数），必须与 MapMetadata.width 一致
    pub width: u32,

    /// 掩码高度（格数），必须与 MapMetadata.height 一致
    pub height: u32,

    /// 每个 Tile 一个 byte，bitfield 表示不同移动类型的通行性
    ///
    /// 位定义：
    ///   位 0: WALK — 地面单位可通行 (0x01)
    ///   位 1: FLY — 飞行单位可通行 (0x02)
    ///   位 2: SWIM — 游泳单位可通行 (0x04)
    ///   位 3-7: 保留
    ///
    /// 生成规则（Importer）：
    ///   WALK = TerrainDef.flags.passable
    ///   FLY = TerrainDef.flags.flyable
    ///   SWIM = terrain_id == "ter:water" 或自定义规则
    pub data: Vec<u8>,
}
```

### 2.9 网格布局枚举

```rust
/// 网格布局类型——与 Tactical Domain 的 GridLayout 定义一致。
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Hash)]
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

### 2.10 TileFlags（Definition 层副本）

```rust
/// Tile 通行标记——MapAsset 层使用的标记结构。
///
/// ⚠️ 此类型与 Tactical Domain 的 TileFlags（resources.rs）在逻辑上一致，
/// 但物理上是独立类型：
///   - MapAsset.TileFlags: serde 序列化版本（RON 中可见）
///   - Tactical.TileFlags: packed u8 位操作版本（运行时性能优化）
///
/// 两者通过 From/TryFrom 转换关联。
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TileFlags(pub u8);

impl TileFlags {
    /// 可行走（位 0）
    pub const PASSABLE: Self = Self(0b0000_0001);
    /// 可飞行（位 1）
    pub const FLYABLE: Self = Self(0b0000_0010);
    /// 可建造（位 2）
    pub const BUILDABLE: Self = Self(0b0000_0100);
    /// 阻挡视线（位 3）
    pub const BLOCKS_SIGHT: Self = Self(0b0000_1000);
}
```

### 2.11 HexDirection（SpawnPoint 使用）

```rust
/// 六边形方向枚举——用于 SpawnPoint.facing。
///
/// 与 Tactical Domain 的 HexDirection 逻辑一致。
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum HexDirection {
    North,
    NorthEast,
    SouthEast,
    South,
    SouthWest,
    NorthWest,
    // 方形网格下的别名
    East,
    West,
}
```

---

## 3. Layer Analysis

| 数据结构 | Layer | 持久化 | 运行时修改 | 说明 |
|----------|-------|--------|-----------|------|
| `MapAsset` | Definition | 文件（RON） | 禁止 | 整个 MapAsset 不可变 |
| `MapMetadata` | Definition | 内嵌于 MapAsset | 禁止 | 加载后只读 |
| `TerrainGrid` | Definition | 内嵌于 MapAsset | 禁止 | 加载后转换为 GridMap（Instance） |
| `TileEntry` | Definition | 内嵌于 TerrainGrid | 禁止 | — |
| `ObjectLayer` | Definition | 内嵌于 MapAsset | 禁止 | Object 是定义，实例化在 Instance 层 |
| `MapObject` | Definition | 内嵌于 ObjectLayer | 禁止 | — |
| `PropertyMap` | Definition | 内嵌于 Object/SpawnPoint/Region | 禁止 | 定义层属性，实例化后复制 |
| `MapObjectGuid` | Definition | 内嵌于 MapObject/SpawnPoint | 禁止 | 不可变标识符 |
| `SpawnPoint` | Definition | 内嵌于 MapAsset | 禁止 | — |
| `MapRegion` | Definition | 内嵌于 MapAsset | 禁止 | v1 仅数据存储 |
| `NavigationMask` | Definition | 内嵌于 MapAsset | 禁止 | 预计算产物 |

MapAsset 属于 Definition 层，其所有字段运行时不可变。GridMap（Instance 层）从 MapAsset 构建，之后独立运行。

### MapAsset 不属于 Spec 层的原因

Spec 层的定义是 "Definition → Instance 的桥梁，携带运行时选择与快照"。MapAsset 没有选择/快照语义——它只是定义层数据的序列化格式。MapAsset 本身不被 Spec 引用，也没有动态配置槽位。

---

## 4. Dependency Analysis

| 依赖方向 | 依赖 Schema/类型 | 说明 |
|----------|-----------------|------|
| 依赖 | `TerrainId`（Shared IDs） | TileEntry.terrain_id 的 ID 类型 |
| 依赖 | `TileFlags`（Tactical Domain） | 逻辑映射：MapAsset TileFlags ↔ Tactical TileFlags |
| 依赖 | `GridPos`（Tactical Domain） | 位置/区域使用的坐标类型 |
| 依赖 | `GridLayout`（Tactical Domain） | 网格布局枚举 |
| 依赖 | `LocalizationKey`（Shared） | name_key 字段 |
| 依赖 | `FactionId`（Shared IDs） | SpawnPoint.faction |
| 依赖 | `SpawnGroupId`（Shared IDs） | SpawnPoint.spawn_group_id |
| 被依赖 | → `GridMap`（Tactical Instance） | MapLoader 将 TerrainGrid → GridMap |
| 被依赖 | → ObjectInstantiator（Infra） | 将 MapObject → ECS Entity |
| 被依赖 | → EncounterSystem（Combat Domain） | 读取 SpawnPoint 进行单位生成 |

---

## 5. Validation Rules

### 5.1 Schema 级校验（RON 反序列化时自动检查）

| # | 规则 | 说明 | 失败处理 |
|---|------|------|----------|
| V1 | `metadata.width * metadata.height == terrain_grid.tiles.len()` | Tile 数量与网格尺寸一致 | 反序列化失败 |
| V2 | `terrain_grid.width == metadata.width` | TerrainGrid 宽度与元数据一致 | 反序列化失败 |
| V3 | `terrain_grid.height == metadata.height` | TerrainGrid 高度与元数据一致 | 反序列化失败 |
| V4 | `navigation_mask.width == metadata.width` | NavigationMask 宽度与元数据一致 | 反序列化失败 |
| V5 | `navigation_mask.height == metadata.height` | NavigationMask 高度与元数据一致 | 反序列化失败 |
| V6 | `navigation_mask.data.len() == width * height` | NavigationMask 数据量与网格一致 | 反序列化失败 |

### 5.2 语义级校验（Importer 输出前 / MapLoader debug 模式）

| # | 规则 | 检查内容 | 失败处理 |
|---|------|----------|----------|
| V7 | TerrainId 有效性 | 每个 `TileEntry.terrain_id` 在 TerrainDef Registry 中存在 | 报错 + 停止 |
| V8 | GUID 唯一性 | 所有 ObjectLayer 中无重复 GUID | 报错 + 停止 |
| V9 | SpawnGroupId 引用完整性 | 每个 `SpawnPoint.spawn_group_id` 在 DefRegistry<SpawnGroupDef> 中存在 | 警告 + 继续（允许引用后续加载的 Def） |
| V10 | 网格完整性 | 所有 grid 位置有有效 TileEntry（无空洞） | 报错 + 停止 |
| V11 | 尺寸合法性 | width > 0, height > 0, 且 ≤ 256x256（v1 上限） | 报错 + 停止 |
| V12 | `metadata.id` 非空 | MapId 不能为空字符串 | 报错 + 停止 |
| V13 | `metadata.id` 格式合法 | 必须匹配 `^map:[a-z][a-z0-9_]+$` | 报错 + 停止 |
| V14 | `schema_version` 兼容 | 当前支持的版本为 1 | 警告 + 继续（未来可扩展） |
| V15 | 高度连续性 | 相邻 Tile 高度差 ≤ 3（最大坡度） | 警告 + 继续 |
| V16 | 必要属性 | Object 的必需 Property 存在（如 Trigger 需要 event_id） | 警告 + 继续 |
| V17 | 导航一致性 | navigation_mask 与 terrain_id 的通行性一致 | 警告 + 继续 |
| V18 | 跨层引用合规 | MapAsset 仅引用 L0（TerrainId, FactionId）和 L3（SpawnGroupId） | 运行时断言 |
| V19 | 无 Gameplay 数值嵌入 | TileEntry 不包含 move_cost/defense_bonus 等数值 | 运行时断言 |

---

## 6. Replay Compatibility

| 场景 | 兼容性 | 说明 |
|------|--------|------|
| MapAsset 加载阶段 | 🟩 完全确定 | RON 文件 → AssetServer → Deserialize → GridMap，无随机性 |
| GUID 生成 | 🟩 完全确定 | 内容哈希，相同输入永远相同输出 |
| 通行性判定 | 🟩 完全确定 | 基于 TerrainDef.flags（配置）。TerrainDef 在 Content Pipeline 冻结后只读 |
| NavigationMask | 🟩 完全确定 | 预计算，无运行时随机性 |
| 网格布局 | 🟩 完全确定 | GridLayout 枚举确定 |
| Object→Entity 实例化 | 🟩 完全确定 | class 映射策略确定 |
| 高度系统 | 🟩 完全确定 | height: u8 确定值 |

**确定性保证**：MapAsset 是 Definition 层数据，所有字段在加载完成后不可变。不依赖时间、系统 RNG、或外部状态。因此 MapAsset 本身是 replay 安全的。GridMap（Instance 层）从 MapAsset 确定性地构建。

**关键点**：Replay 录制的是场景入口时的 MapLoadParams（`{ map_asset_id: String }`），而不是 MapAsset 本身。回放时通过相同的 MapAsset RON 文件重新加载，确保地图状态一致。

---

## 7. Save Compatibility

MapAsset 本身就是存档加载所需的数据源。存档中保存的是地图标识，而不是地图数据本身：

```rust
/// 存档中关于地图的部分——只保存 ID，不保存 Tile 数据。
#[derive(Serialize, Deserialize, Clone)]
pub struct MapSaveData {
    /// 地图 Asset ID（用于重新加载 MapAsset）
    pub map_asset_id: String,

    /// 存档时版本号（用于校验）
    pub schema_version: u32,
}
```

### 为什么 MapAsset 本身不参与存档？

| 理由 | 说明 |
|------|------|
| **Single Source of Truth** | 地图数据在 MapAsset RON 文件中。存档时保存完整的地图数据意味着存在两份，引发一致性问题 |
| **存档体积优化** | 50x50 地图约 10KB RON → 存档中只需 2 个字段 |
| **内容更新兼容** | 旧存档在更新后的地图中加载——地图 BUG 修复后，读旧存档即获得修复后的地图 |
| **不需要恢复** | 地图数据是静态的，战斗不修改 MapAsset（即使有地形修改 v1 也不做） |

**例外**：如果未来引入运行时地图修改（地形破坏等），受影响的 Tile 数据需要在 `TerrainState`（terrain_schema.md 定义）中存档，作为 MapSaveData 的补充。这属于 Terrain Domain 的存档职责，不在 MapAsset Schema 中处理。

---

## 8. Migration Strategy

### 版本历史

| 版本 | 变更 | 迁移 |
|------|------|------|
| 1 | 初始版本 | — |

### 未来可能变更

| 变更类型 | 示例 | 兼容方式 |
|----------|------|----------|
| 新增字段 | `environment_tags: Vec<String>` | `#[serde(default)]` 新字段 |
| 新增 TileEntry 字段 | `biome_id: Option<BiomeId>` | `#[serde(default)]` 可选字段 |
| NavigationMask 位扩展 | 新增 SWIM 位 | 解码逻辑兼容旧位定义 |
| Region 扩展 | `children: Vec<MapRegion>` | 区域嵌套到后期再添加 |

### 迁移策略

- **schema_version 自增**：每次 MapAsset 结构变更时递增
- **新增字段使用 `#[serde(default)]`**：旧文件加载时不报错
- **废弃字段使用 `#[serde(skip_deserializing)]`**：保留序列化输出兼容
- **移除字段使用版本判断**：MapLoader 检查 schema_version，执行对应版本的处理逻辑

---

## 9. Future Extension

| 扩展点 | 预留方式 | 触发条件 |
|--------|----------|----------|
| 环境标签 | `MapMetadata` 可新增 `environment_tags: Vec<String>` | 需要按环境分类/过滤地图时 |
| 多层地形 | `TileEntry` 已有 `layer` 概念（GridPos.layer），但未用 | 需要立体地图/多层区域时 |
| 多 TileSet | `TileEntry` 可扩展 `tileset_id` 字段 | 单地图使用多种 TileSet （如室内+室外）时 |
| Region 运行时查询 | `MapRegion` 结构已是查询所需的数据基础 | v2 Region 系统开始时 |
| 动态导航更新 | `NavigationMask` 标记 "不可更新"（v1 只读） | 需要运行时地形修改时 |
| Fog of War | 不依赖 MapAsset 结构，由独立系统处理 | 战术需求明确时 |
| 动画 Tile | `TileEntry` 可扩展 `animation_id: Option<AnimationId>` | 需要动态地形效果时 |
| Object Group 嵌套 | Tiled 支持 Group Layer，未来可在 `ObjectLayer` 增加 `children: Vec<ObjectLayer>` | 需要对象层嵌套层次时 |

---

## 10. Risks

| 风险 | 级别 | 影响 | 缓解措施 |
|------|------|------|----------|
| TerrainGrid 文件体积过大 | 低 | 100x100 地图约 6MB RON | 1) 运行时立即转换为 packed GridMap<br>2) 考虑 RON 编码/压缩优化<br>3) schema_version 升级时可引入压缩格式 |
| TileEntry.flags 与 TerrainDef.flags 不一致 | 中 | 导入时未被发现会导致寻路使用错误 flags | Importer 校验 V17 检查一致性。若 TerrainDef 更新，需重新导入所有引用该 terrain_id 的地图 |
| GUID 冲突 | 低 | 罕见，内容哈希碰撞概率极低 | 在 Importer 中校验 GUID 唯一性（V8）。支持 64 位基本不可能碰撞 |
| TMX 重新导出时对象属性丢失 | 中 | Tiled 在某些操作下会清空 Custom Property | 内容团队需要遵循 Tiled 工作流规范：编辑后检查 properties |
| NavigationMask 与 TerrainDef 解耦 | 低 | 修改 TerrainDef 的通行性后 NavigationMask 不更新 | ADR-065 §13 已排除运行时地形修改。TerrainDef 修改后需要重新导入所有相关地图 |

---

## 11. Constitution Check

| 条款 | 合规 | 说明 |
|------|------|------|
| DL001 Def-Instance 分离 | ✅ | MapAsset = Definition（不可变）, GridMap = Instance（运行时），严格分离 |
| DL002 Rule-Content 分离 | ✅ | MapAsset 只存储数据 ID（terrain_id, spawn_group_id），不含业务规则 |
| DL003 Config IDs Only | ✅ | TileEntry 只存 TerrainId，所有数值通过 Registry 查询 |
| DL004 Ability 不拥有行为 | ✅ | 不涉及 |
| DL005 Effect 是唯一业务入口 | ✅ | 不涉及 |
| DL006 Modifier 只改变数值 | ✅ | 不涉及 |
| DL007 Duration 属于 Effect | ✅ | 不涉及 |
| DL008 所有 Stacking 归 Stacking | ✅ | 不涉及 |
| DL009 表现必须通过 Cue | ✅ | 不涉及 |
| DL010 Replay 优先 | ✅ | MapAsset 完全确定，ID + 预计算掩码无随机性 |
| DL011 Schema 版本化 | ✅ | schema_version 字段 + #[serde(default)] 兼容 |
| DL012 域间禁止直接引用 | ✅ | MapAsset 在 Infra 层，不直接引用任何 Domain 数据 |
| DL013 Localization First | ✅ | name_key 使用 LocalizationKey |
| Archetype 爆炸防护 | ✅ | GridMap 是 Resource（非 Entity 集合），不创建 Archetype |
| Forbidden: Tile 存储数值 | ✅ | TileEntry 只存 terrain_id，不包含 move_cost/defense_bonus |
| Forbidden: 运行时修改 | ✅ | MapAsset 不可变，加载后不修改 |
| Forbidden: Importer 依赖 Core | ✅ | 类型定义共享，Importer 不引入 Core 层依赖 |
| Bevy 0.19: trigger()+Observer | ✅ | 通信使用 MapLoaded Event + Observer，不使用 EventWriter/Reader |

---

*本文档由 @data-architect 维护。*
