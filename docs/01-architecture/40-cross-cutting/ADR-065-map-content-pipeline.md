---
id: 01-architecture.ADR-065
title: ADR-065 — Map Content Pipeline Architecture
status: proposed
owner: architect
created: 2026-06-22
updated: 2026-06-22
supersedes: none
---

# ADR-065: Map 内容管线架构

## 状态

**Proposed** — 依赖 ADR-022（网格/地形/阵营架构）、ADR-047（Content 加载管线）、ADR-050（游戏状态机与场景架构），本架构决策正式生效。

## 背景

项目中已有完整的 Tactical Domain（GridMap/GridPos/TileData/BFS 寻路/移动规则）和 Terrain Domain（TerrainDef/SurfaceType/通行性/地形效果），但地图数据完全靠硬编码（test_battle 的 6x6 占位网格），没有任何从 Tiled 编辑器到游戏运行时的内容管线。

本文档定义了从 Tiled 编辑器到运行时 MapAsset 的完整内容管线架构，涵盖 13 个关键决策点。

### 当前状态

| 模块 | 路径 | 状态 |
|------|------|------|
| Tactical Domain | `src/core/domains/tactical/` | GridPos, GridMap(Resource), TileData(packed), BFS 寻路, 移动规则 |
| Terrain Domain | `src/core/domains/terrain/` | TerrainDef, SurfaceType, TileProperties, Hazard, 地形效果系统 |
| ADR-022 | `docs/01-architecture/20-tactical-combat/ADR-022-grid-terrain-faction.md` | 网格/地形/阵营架构 |
| 寻路 | `src/core/domains/tactical/rules/movement.rs` | A* 寻路，使用 GridMap(Resource) + TileData |
| Map Content Pipeline 规划 | `docs/09-planning/map-content-pipeline-plan.md` | 行动方案（当前 sprint） |

### 缺口

1. 无 Tiled → Importer → MapAsset 管线
2. 无 MapAsset 运行时格式定义
3. 无 MapRenderer（当前 test_battle 使用占位彩块）
4. 无地图场景生命周期（地图加载/卸载）
5. TerrainDef RON 配置未通过 Content Pipeline 加载
6. 无 Importer 验证（GUID 唯一性、TerrainId 有效性）

## 引用的领域规则与数据架构

- `docs/02-domain/domains/tactical_domain.md` — Tactical 领域规则（GridPos/移动/夹击/高地）
- `docs/02-domain/domains/terrain_domain.md` — Terrain 领域规则（Tile 属性/表面变化/陷阱/Hazard）
- `docs/04-data/domains/tactical_schema.md` — GridMap/TileData Schema
- `docs/04-data/domains/terrain_schema.md` — TerrainDef/SurfaceType/HazardZone Schema
- `docs/03-content/content-layering.md` — 5 层分层体系，L4 World 层含 MapDef
- `docs/03-content/definitions/README.md` — L0-L3 Def 类型索引，L4 待补充 MapDef
- `docs/01-architecture/20-tactical-combat/ADR-022-grid-terrain-faction.md` — 网格/地形/阵营架构（本 ADR 补充而非取代）
- `docs/01-architecture/40-cross-cutting/ADR-047-content-loading-pipeline.md` — Content 加载管线
- `docs/01-architecture/00-foundation/ADR-050-game-state-machine.md` — 场景生命周期
- `docs/04-data/README.md` — Data Laws（尤其 Def-Instance 分离、Rule-Content 分离）

## 决策

### 1. Tiled 定位：内容生产工具，运行时格式独立

**决策**：Tiled 仅作为内容生产工具使用。地图数据的生命周期划分为三层：

```
Tiled (TMX)           ← 编辑格式：内容团队使用 Tiled 编辑
    │
    ▼  [Importer — 构建时]
MapAsset (RON)        ← 运行时格式：Importer 产物，游戏直接消费
    │
    ▼  [Content Pipeline — 启动时]
ECS State             ← 实例化：GridMap + Marker Entities + Renderer
```

- TMX 是 Tiled 的原生保存格式，仅存在于开发和内容制作阶段
- MapAsset RON 是游戏直接加载的运行时格式，版本可控、可 diff、可 review
- 游戏二进制永不包含 TMX 解析代码
- 编辑器可以更换（未来可切到其他工具），MapAsset 格式不变

**对 ADR-022 的补充**：ADR-022 定义了 GridMap 的内存格式和查询 API，本 ADR 定义 GridMap 的数据来源（MapAsset.TerrainGrid → GridMap）。

### 2. Importer 管线设计：构建时转换

**决策**：Importer 是一个独立的 Rust 工具 crate，位于 `tools/map_importer/`，在构建时运行。

```
tools/map_importer/
├── Cargo.toml          # 独立 crate，依赖 MapAsset 共享类型
├── src/
│   ├── main.rs          # CLI 入口：importer --input map.tmx --output map.ron
│   ├── tmx_parser.rs    # TMX XML 解析（使用 quick-xml 或 tiled crate）
│   ├── tiled_types.rs   # TMX 内部数据结构（TileLayer, ObjectGroup 等）
│   ├── converter.rs     # TMX → MapAsset 转换逻辑
│   ├── guid_gen.rs      # 稳定 GUID 生成（内容哈希）
│   ├── nav_builder.rs   # 通行性导航掩码生成
│   ├── validator.rs     # 地图验证（GUID/ID/引用完整性）
│   └── tests/           # 单元/集成测试
```

**执行时机**：
- 开发期：内容团队编辑 TMX 后手动 `cargo run --bin importer -- -i map.tmx -o map.ron`
- CI 中：可在 CI 中作为验证步骤运行（检查所有 TMX 是否可成功转换为 MapAsset）
- 构建时：不通过 build.rs 自动执行（避免构建过慢），而是通过 Makefile/Justfile 编排

**Importer 不依赖游戏核心逻辑**：Importer 只依赖 MapAsset 类型定义（共享的类型 crate 或通过 infra/map 的 pub 类型），不引入 Core 层依赖。确保 Importer 的编译和运行与游戏解耦。

### 3. MapAsset 结构定义

**决策**：MapAsset 是 Bevy Asset（`#[derive(Asset, TypePath)]`），存储在 `assets/config/04_world/maps/` 目录。

```rust
/// MapAsset — 运行时地图资产（Importer 产物）
/// L4 World 层，不可变。定义与实例分离（Data Law 001）。
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct MapAsset {
    /// 地图元数据
    pub metadata: MapMetadata,
    /// 地形网格数据（核心数据）
    pub terrain_grid: TerrainGrid,
    /// 对象层列表（一等公民）
    pub object_layers: Vec<ObjectLayer>,
    /// 出生点列表
    pub spawn_points: Vec<SpawnPoint>,
    /// 区域/命名范围
    pub regions: Vec<MapRegion>,
    /// 通行性导航掩码（Importer 预计算）
    pub navigation_mask: NavigationMask,
}

/// 地图元数据
pub struct MapMetadata {
    /// 地图 Def ID（关联 L4 MapDef）
    pub id: String,
    /// 显示名称 LocalizationKey
    pub name_key: LocalizationKey,
    /// 网格宽度（格子数）
    pub width: u32,
    /// 网格高度（格子数）
    pub height: u32,
    /// 网格布局类型
    pub layout: GridLayout,
    /// Tiled 原始像素尺寸（用于坐标转换）
    pub pixel_width: u32,
    pub pixel_height: u32,
    /// 每格像素尺寸
    pub tile_width: u32,
    pub tile_height: u32,
}

/// 地形网格 — 核心数据
pub struct TerrainGrid {
    pub width: u32,
    pub height: u32,
    /// 按行优先排列的 Tile 数据
    pub tiles: Vec<TileEntry>,
}

/// 每个 Tile 的运行时数据
pub struct TileEntry {
    /// 地形 ID（唯一标识，查 Registry 可得完整 TerrainDef）
    pub terrain_id: TerrainDefId,
    /// 高度（0-255，TileData 已预留给定字段）
    pub height: u8,
    /// 位标记：PASSABLE, FLYABLE, BUILDABLE, BLOCKS_SIGHT
    pub flags: TileFlags,
    /// 旋转（0-3，90 度递增）
    pub rotation: u8,
    /// Tiled 原始 Tile ID（仅用于调试追溯，运行时可不加载）
    pub gid: Option<u32>,
}

/// 对象层 — 一等公民
pub struct ObjectLayer {
    /// 层 ID（Tiled 原始 ID，仅用于调试追溯）
    pub id: u32,
    /// 层名称
    pub name: String,
    /// 透明度
    pub opacity: f32,
    /// 是否可见
    pub visible: bool,
    /// 层偏移（像素）
    pub offset_x: i32,
    pub offset_y: i32,
    /// 本层所有对象（稳定 GUID）
    pub objects: Vec<MapObject>,
}

/// 地图对象 — 运行时生成 ECS Entity 的模板
pub struct MapObject {
    /// 稳定 GUID（Importer 生成，用于跨存档/跨场景追溯）
    pub guid: MapObjectGuid,
    /// Tiled 原始 ID（仅用于调试追溯和 TMX 回查）
    pub tiled_id: u32,
    /// 对象名称
    pub name: String,
    /// 对象类型/Custom Class（Tiled Class 名）
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
    /// 形状（用于碰撞/区域判定）
    pub shape: ObjectShape,
}

/// 属性映射 — 泛型属性容器
pub struct PropertyMap {
    pub entries: HashMap<String, PropertyValue>,
}

pub enum PropertyValue {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Color([f32; 4]),
    File(String),
}

pub enum ObjectShape {
    Point,
    Rectangle { width: u32, height: u32 },
    Ellipse { width: u32, height: u32 },
    Polygon { points: Vec<(f32, f32)> },
    Polyline { points: Vec<(f32, f32)> },
}

/// 出生点 — 单位放置位置
pub struct SpawnPoint {
    pub guid: MapObjectGuid,
    /// 生成组 ID（存 ID 不存具体单位——平衡性调整不改地图）
    pub spawn_group_id: SpawnGroupId,
    /// 网格位置
    pub position: GridPos,
    /// 阵营
    pub faction: FactionDefId,
    /// 朝向
    pub facing: HexDirection,
    /// 额外属性
    pub properties: PropertyMap,
}

/// 区域 — 地图上的命名范围
pub struct MapRegion {
    /// 区域标识符
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 包含的网格位置
    pub tiles: Vec<GridPos>,
    /// 区域属性
    pub properties: PropertyMap,
}

/// 通行性导航掩码 — Importer 预计算
pub struct NavigationMask {
    pub width: u32,
    pub height: u32,
    /// 每个 Tile 一个 byte，bitfield 表示不同移动类型的通行性
    pub data: Vec<u8>,
}
```

**MapObjectGuid 类型**：

```rust
/// 地图对象稳定 GUID — 全局唯一、跨存档稳定
/// 由 Importer 使用内容哈希生成，不依赖 Tiled ID
pub struct MapObjectGuid(u64);
```

GUID 生成策略：`hash(map_id, layer_name, object_class, tile_x, tile_y)` 使用确定性哈希（如 SipHash-2-4），确保相同 TMX 输入始终产生相同 GUID。

### 4. Tile → Config 映射策略

**决策**：Tile 不承载 Gameplay 数值，数值来自 Config Registry。

```
TileEntry.terrain_id = ter_000001 (TerrainDefId)
        │
        ▼  [运行时查找]
TerrainDef (Config Registry)
  ├── movement_cost: 1.0      ← 移动消耗
  ├── defense_bonus: 0.0       ← 防御加成
  ├── avoid_bonus: 0.0         ← 闪避加成
  ├── flags: Passable           ← 通行性
  └── tags: ["terrain:plain"]   ← Tag 系统连接
```

**遵循的宪法原则**：
- **Data Law 001（Def-Instance 分离）**：TileEntry.terrain_id = Definition 引用，所有可变状态在 Instance 层
- **Data Law 002（Rule-Content 分离）**：地形 Gameplay 数值在 TerrainDef 中，不在 Tile 中
- **Data Law 003（配置只引用 ID）**：TileEntry 只存 TerrainDefId，不内嵌 TerrainDef 字段

**优点**：
- 修改地形平衡 → 改 TerrainDef RON，不改 TMX
- 新增地形类型 → 新增 TerrainDef + 在 TMX 中使用新 Tile，不改代码
- 地图文件只存 ID，体积小，无重复数据
- 与 ADR-022 中 "TerrainDef.tags → TagSystem" 设计一致

### 5. Object Layer：一等公民

**决策**：Object Layer 是 MapAsset 的一等公民，对象具有稳定 GUID。

**Object Layer 的设计哲学**（继承历史经验）：
- Object Layer 不像 Tile Layer 那样是纯数据（terrain_id 决定一切）——Object 包含业务语义
- 一个 Object = 一个未来可能实例化为 ECS Entity 的模板
- Object 不直接映射 Entity——Object 是定义，运行时 System 决定是否/何时实例化
- 实例化策略：

```
MapObject (MapAsset 中的定义，不可变)
  │
  ├── class: "SpawnPoint"    → SpawnPoint → SpawnSystem → Unit Entity
  ├── class: "Chest"         → [Interaction] → Chest Entity (含 Inventory)
  ├── class: "Door"          → [Interaction] → Door Entity (含 DoorState)
  ├── class: "Hazard"        → HazardZone → HazardSystem
  ├── class: "Trigger"         → [TriggerZone] → Trigger Entity (含 event_id)
  ├── class: "Region"           → MapRegion → RegionQuery（未来）
  └── class: "Decor"             → Visual Entity Only（无 Gameplay）
```

**类名映射**：Tiled 的 Custom Class（或 Type 字段）是实例化的唯一依据。内容团队在 Tiled 中设置对象的 Class 属性，MapAsset 记录 class: String，运行时系统根据 class 决定如何处理。

**GUID 稳定性保证**：
- GUID 由 Importer 通过内容哈希生成，不依赖 Tiled 内部 ID
- GUID 在同一地图内唯一，跨地图不保证唯一（通常用 map_id + guid 组成全局唯一键）
- GUID 在 存档/读档 中保持稳定，用于追溯对象身份

### 6. Property 映射规则

**决策**：Tiled Property → PropertyMap（通用键值对）→ 运行时领域系统按需查询。

```
Tiled Property (编辑时)
  string: "event_id" = "evt:chest_open"
  int:    "difficulty" = 3
  bool:   "locked" = true
        │
        ▼  [Importer 转换]
PropertyMap (MapAsset)
  entries:
    "event_id" → String("evt:chest_open")
    "difficulty" → Int(3)
    "locked" → Bool(true)
        │
        ▼  [运行时查询]
Domain System (e.g., InteractionDomain)
  map_object.properties.get("event_id")
  map_object.properties.get("locked")
```

**Property 的消费方**：
- **SpawnSystem** 消费 spawn_point.properties（队伍 ID、初始状态）
- **InteractionSystem** 消费 MapObject.properties（event_id、locked、key_id）
- **HazardSystem** 消费 MapObject.properties（hazard_id、damage、trigger_condition）
- **NarrativeSystem** 消费 MapObject.properties（dialogue_id、scene_id）

**Property 不承载核心 Gameplay 数值**：PropertyMap 适合「标记」和「配置覆盖」，不适合承载核心数值。核心数值归 TerrainDef/EncounterDef/SpawnGroupDef 管理。

### 7. Map 渲染架构

**决策**：自研 MapRenderer，位于 `src/infra/map/renderer/`，不使用 `bevy_ecs_tilemap`。

```
MapRenderer
├── Tile Sprite Layer       ← 基础地形瓦片渲染（Material2d 批处理）
├── Overlay Layer           ← 移动范围高亮、AOE 范围、技能指示器
├── Grid Overlay            ← 网格线（调试用，可选）
├── Cursor Layer            ← 鼠标悬浮/选择高亮
└── Camera Integration      ← 坐标转换 + 镜头联动 (infra/camera/)
```

**渲染策略**：
- **基础瓦片层**：使用 Bevy `Material2d` 或自定义 `Mesh2d` 将 Tile Sprite 批处理渲染。每个 TileEntry 根据 terrain_id 查找对应的 Sprite（从 TileSet 中索引）。支持旋转 (rotation: u8)。
- **高亮层**：使用单独的渲染 pass，半透明 overlay。移动范围=A* 可达位置集合，AOE=圆形/锥形/线性范围。由 Domain 系统计算范围 → 通过 Resource 或 Component 传递给 Renderer。
- **坐标转换**：GridPos ↔ WorldPos 转换通过 GridMap 的 `grid_to_world()` / `world_to_grid()` 方法。Camera 系统提供屏幕空间 ↔ 世界空间的转换。

**遵循的架构原则**：
- MapRenderer 在 Infra 层，不包含业务逻辑
- 渲染所需的数据（移动范围、AOE 范围）由 Domain 层计算后通过 ECS 组件传递
- 不直接 Query 领域数据用于渲染——通过 Overlay 组件解耦

### 8. 地图场景生命周期

**决策**：地图加载通过 ADR-050 的场景生命周期管理。

```
┌──────────────────────────────────────────────────────────┐
│                   Scene Lifecycle                         │
│                                                          │
│  OnEnter(TacticalMap / Combat)                           │
│  ┌────────────────────────────────────────────────────┐  │
│  │ MapLoader System                                   │  │
│  │  1. 从场景数据通道读取 map_asset_id                │  │
│  │  2. 通过 AssetServer 获取 MapAsset Handle          │  │
│  │  3. 解析 MapAsset → GridMap Resource                │  │
│  │  4. 创建 TileMarker Entity（交互式 Tile 实体化）    │  │
│  │  5. 处理 ObjectLayer（按 class 策略实例化）        │  │
│  │  6. 初始化 MapRenderer（生成 Tile Sprite 实体）    │  │
│  │  7. 标记所有实体为 SceneRoot 的子级                │  │
│  │  8. 发射 MapLoaded 事件                             │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  Update                                                   │
│  ┌────────────────────────────────────────────────────┐  │
│  │ Domain Systems (tactical, terrain, combat)         │  │
│  │ MapRenderer (tick animations, update overlays)     │  │
│  └────────────────────────────────────────────────────┘  │
│                                                          │
│  OnExit(TacticalMap / Combat)                            │
│  ┌────────────────────────────────────────────────────┐  │
│  │ cleanup_scene System                               │  │
│  │  Despawn all SceneRoot 实体 → 地图资源全释放       │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

**场景数据通道**（符合 ADR-050 的数据通道模式）：
```rust
/// 场景入口参数 — OnEnter(Combat/TacticalMap) 时消费
#[derive(Resource)]
pub struct MapLoadParams {
    pub map_asset_id: String,       // MapAsset 的 AssetPath
    pub battle_config: Option<BattleConfig>,  // 战斗额外配置
}
```

**MapLoader System 职责**：
- 仅做 "MapAsset → ECS State" 的转换，不包含业务逻辑
- 不检查单位放置合理性、不触发战斗初始化
- 这些业务逻辑由 CombatPlugin 的 OnEnter 系统处理

### 9. 与 Tactical/Terrain Domain 的对接

**决策**：MapAsset 的数据流向已有 Domain，不绕过已有架构。

```
MapAsset
  │
  ├── terrain_grid
  │     │
  │     ▼
  │   GridMap (Resource)                    ← Tactical Domain 已有
  │     ├── get_tile(pos) → &TileData       ← 已有 API
  │     ├── neighbors(pos) → Vec<GridPos>    ← 已有 API
  │     ├── find_path(...) → Option<Path>    ← 已有 API
  │     └── world_to_grid / grid_to_world    ← 已有 API
  │
  ├── object_layers
  │     │
  │     ├── class: "Chest" → TileMarker + ChestState    ← 实例化新 Entity
  │     ├── class: "Door"  → TileMarker + DoorState     ← 实例化新 Entity
  │     └── class: "Decor" → TileMarker + VisualBundle  ← 实例化新 Entity
  │
  ├── spawn_points
  │     │
  │     ▼
  │   Encounter System (Combat Domain)    ← SpawnUnit 决策
  │     └── spawn_group_id → SpawnGroupDef → 单位列表
  │
  ├── regions → MapRegion (Resource)       ← 未来：RegionQuery API
  │
  └── navigation_mask
        │
        ▼
      MovementSystem (Tactical Domain)     ← 可选的加速结构
        └── GridMap 仍是权威数据源，navigation_mask 仅用于快速过滤
```

**对接要点**：
- TileEntry.terrain_id → TileData.terrain_def_id（类型一致，直接映射）
- TileEntry.height → TileData.height（已预留字段，直接填充）
- TileEntry.flags → TileFlags（已有类型，直接映射）
- GridMap 从 MapAsset.TerrainGrid 构建，后续运行时不回写 MapAsset
- TileMarker 组件（ADR-022）在交互式 Tile 实体化时添加

### 10. 高度系统

**决策**：TileData.height: u8 已在 ADR-022 预留，Importer 从 Tiled 的 Terrain 高度数据填充。

- Tiled 中通过 Tile 的 Custom Property "height"（或通过 Terrain 的 elevation 属性）设置高度
- Importer 读取该值填入 TileEntry.height
- 运行时 TileData.height 由 GridMap 构建时从 TileEntry 复制
- 高度用于地形视觉效果（Shader 高度偏移）和 Gameplay（视野/攻击/寻路）
- 寻路系统在 v1 中不区分高度（视为平地），v2 加入高度影响

**高度范围**：0-255（u8），0=地面层，1-254=高地，255=保留（天空/不可达）

### 11. Importer 验证

**决策**：Importer 在生成 MapAsset 时执行以下验证，验证失败时拒绝输出。

| 验证项 | 检查内容 | 失败处理 |
|--------|----------|----------|
| GUID 唯一性 | 所有 ObjectLayer 中无重复 GUID | 报错 + 停止 |
| TerrainId 有效性 | 每个 TileEntry.terrain_id 在 TerrainDef Registry 中存在 | 报错 + 停止 |
| 引用完整性 | SpawnPoint.spawn_group_id 在 L3 SpawnGroupDef 中存在 | 警告 + 继续 |
| 高度连续性 | 相邻 Tile 高度差 ≤ 3（最大坡度） | 警告 + 继续 |
| 网格完整性 | 所有 grid 位置有有效 TileEntry（无空洞） | 报错 + 停止 |
| 尺寸一致性 | width/height 与 tiles.len() 一致 | 报错 + 停止 |
| 必要属性 | Object 的必需 Property 存在（如 Trigger 需要 event_id） | 警告 + 继续 |
| 导航一致性 | navigation_mask 与 terrain_id 的通行性一致 | 警告 + 继续 |

**验证阶段**：
1. **Schema 级验证**：MapAsset RON 结构正确性（Bevy Asset 反序列化时自动检查）
2. **语义级验证**：上述表格中的验证项（Importer 输出前检查）
3. **运行时验证**：MapLoader 加载 MapAsset 时做快速完整性检查（仅 debug 模式）

### 12. Region/Zone 系统

**决策**：MapAsset 存储 Region 数据，v1 不做 Region 运行时查询系统。

```
MapRegion {
    id: "danger_zone",
    name: "危险区域",
    tiles: [GridPos(5,3), GridPos(5,4), GridPos(6,3), GridPos(6,4)],
    properties: { "hazard_id": "haz:poison_swamp" }
}
```

**v1 能力**：
- 数据存储：MapAsset.regions 存储命名的 Tile 集合和属性
- 静态查询：系统可通过遍历 regions 查询某个 Tile 属于哪些 Region
- Hazard 区域：由 Terrain Domain 的 HazardZone 处理（已有架构），不依赖 Region 系统

**不做**（v1）：
- 运行时 Region 查询 API（`region.contains(pos)` 可以手动实现，但无统一查询入口）
- Region 触发的事件（"单位进入 Region A → 触发事件"）
- AI 区域感知（"Region A 是战略要地，AI 倾向占领"）
- Region 嵌套与层次结构

**接口预留**：MapAsset 的 regions 字段为未来 Region 系统提供数据基础，无需改动 MapAsset 格式即可升级。

### 13. 当前不做范围

以下功能明确排除在 v1 管线之外，防止范围蔓延：

| 功能 | 不做理由 | 触发条件 |
|------|----------|----------|
| **World Map（世界地图）** | 需要全局地图切换/导航系统，当前无叙事需求 | 叙事系统需要多地图切换时 |
| **Fog of War（战争迷雾）** | 需要视野/可见性系统，复杂度高 | 战术需求明确时 |
| **Dynamic Map（动态地图）** | 运行时地形修改/破坏，涉及 Effect 管线集成 | 技能系统需要地形交互时 |
| **Runtime Editing** | 由玩家建设/修改地形，涉及存档 | 有明确建设系统需求时 |
| **Chunk Streaming** | SRPG 地图有限（通常 ≤ 100×100），无需分块 | 大地图超过 256×256 时 |
| **Multi-layer terrain** | 多层瓦片叠加渲染，当前单层即可 | 地形视觉效果需要时 |
| **TMX 热重载** | TMX 仅在编辑期存在，运行时不加载 TMX | 开发期调试需要时（可加 dev feature） |

## Module Design

### 新增模块结构

```
src/infra/map/                     # 地图管线基础设施
├── mod.rs                         # pub mod 声明
├── plugin.rs                      # MapPlugin（Asset 类型注册 + Loader System 注册）
├── asset.rs                       # MapAsset 类型定义（Asset, TypePath）
├── types.rs                       # 辅助类型：MapObjectGuid, PropertyMap, ObjectShape 等
├── loader.rs                      # MapAsset → GridMap + ECS State 的加载逻辑
├── events.rs                      # MapLoaded, MapUnloaded 事件
├── integration/                   # 跨域访问层（供 Domain 查询地图数据）
│   ├── mod.rs
│   ├── facade.rs                  # MapQuery 业务语义 API
│   └── query_api.rs               # Tile/Region/Object 查询
├── renderer/                      # MapRenderer
│   ├── mod.rs
│   ├── plugin.rs                  # MapRendererPlugin
│   ├── tile_layer.rs             # 基础瓦片层渲染
│   ├── overlay_layer.rs          # 高亮层（移动范围/AOE/指示器）
│   ├── grid_overlay.rs           # 网格线（调试）
│   └── materials/                # 自定义 Material
│       ├── mod.rs
│       └── tile_material.rs
├── systems/                       # Map 生命周期 Systems
│   ├── mod.rs
│   ├── map_loader_system.rs      # OnEnter → 加载地图
│   ├── map_cleanup_system.rs     # OnExit → 清除地图
│   └── object_instantiator.rs    # MapObject → ECS Entity
└── tests/                         # 测试
    ├── mod.rs
    ├── unit/
    │   ├── mod.rs
    │   ├── map_asset_test.rs
    │   └── property_map_test.rs
    └── integration/
        ├── mod.rs
        └── map_loader_test.rs

tools/map_importer/                # Importer 独立工具
├── Cargo.toml
└── src/
    ├── main.rs                    # CLI 入口
    ├── tmx_parser.rs              # TMX 解析
    ├── converter.rs               # TMX → MapAsset 转换
    ├── guid_gen.rs                # 稳定 GUID 生成
    ├── nav_builder.rs             # 通行性导航掩码
    ├── validator.rs               # 地图验证
    └── tests/
        ├── integration/
        └── fixtures/              # 测试用 TMX 文件
```

### Plugin 注册顺序

MapPlugin 属于 Infra 层，注册在 Phase 8（基础设施层），紧接 Save/Input Plugin 之后：

```rust
// Phase 8: Infrastructure (L2)
.add_plugins(infra::registry::RegistryPlugin)
.add_plugins(infra::pipeline::PipelinePlugin)
.add_plugins(infra::replay::ReplayPlugin)
.add_plugins(infra::save::SavePlugin)
.add_plugins(infra::input::InputPlugin)
.add_plugins(infra::map::MapPlugin)          // ← 新增
.add_plugins(infra::localization::LocalizationPlugin)
```

MapRendererPlugin 同样在 Phase 8 注册（作为 MapPlugin 的子 Plugin）：

```rust
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        // 1. 注册 Asset 类型
        app.init_asset::<MapAsset>();

        // 2. 注册场景生命周期系统
        app.add_systems(OnEnter(GameState::TacticalMap), load_map);
        app.add_systems(OnExit(GameState::TacticalMap), cleanup_map);
        app.add_systems(OnEnter(GameState::Combat), load_map);
        app.add_systems(OnExit(GameState::Combat), cleanup_map);

        // 3. 注册 Renderer（独立子 Plugin）
        app.add_plugins(MapRendererPlugin);
    }
}
```

### Content Plugin 变更

`src/content/content_plugin.rs` 需注册 MapAsset 的 Asset 加载和 Hot-reload Observer：

```rust
// 在 ContentPlugin::build() 中追加
app.init_asset::<MapAsset>()
   .add_observer(on_map_asset_added);
```

### Infra 层模块导出

`src/infra/mod.rs` 新增：

```rust
pub mod map;
```

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| TMX → MapAsset | Importer（构建时进程） | 文件系统 → 构建期 |
| MapAsset 加载 | AssetServer::load() + Observer(OnAdd) | Content Plugin → Infra |
| 场景入口参数 | `MapLoadParams` Resource 通道 | OnExit(Prev) → OnEnter(Curr) |
| MapAsset → GridMap | `load_map_system` 直接转换 | Infra → Tactical Domain |
| 地图加载完成 | `MapLoaded` Event (Observer) | Infra → 所有 Domain |
| 地图卸载通知 | `MapUnloaded` Event (Observer) | Infra → 所有 Domain |
| Object 实例化 | `MapObject` → ECS Entity | Infra → 对应 Domain |
| 移动范围查询 | `Res<GridMap>` 直接读取 | Tactical Domain → 已有 |
| Spawn 位置读取 | `Res<MapLoadParams>` + MapQuery | Combat Domain → Infra/facade |
| 渲染数据传递 | Component Write（Overlay 数据） | Domain → MapRenderer |

## 边界定义

### 允许
- MapPlugin 在 Infra 层注册 MapAsset 类型和加载系统
- MapRenderer 在 Infra 层处理瓦片渲染和高亮绘制
- Tactical/Terrain Domain 通过 `Res<GridMap>` 读取地图数据
- 场景系统通过 `MapLoadParams` 指定要加载的地图
- 任意 Domain 通过 `MapQuery` facade 查询地图对象和属性
- Importer 作为独立工具读取 TMX 并输出 RON

### 🟥 禁止
- MapAsset 中包含业务逻辑（寻路、战斗结算等）
- TileEntry 直接存储 Gameplay 数值（必须通过 TerrainDef 间接查询）
- 游戏二进制包含 TMX 解析代码（Importer 是独立工具）
- MapRenderer 包含领域业务逻辑（只渲染，不计算）
- 运行时修改 MapAsset 数据（Definition 不可变）
- Object 直接映射为 Entity（Object 是定义，实例化由 Domain 系统决定）
- MapLoader 处理单位生成/战斗初始化（这些是 Domain 的职责）
- Domains 间通过 MapAsset 直接传递数据（必须通过 Event/Query API）
- Region 系统在 v1 中实现运行时查询或触发（仅数据存储）

## Forbidden

| 禁止行为 | 理由 | 违反后果 |
|----------|------|----------|
| 🟥 Tile 存储 Gameplay 数值（move_cost/defense_bonus） | 违反 Def-Instance 分离、Rule-Content 分离 | 配置校验拒绝 |
| 🟥 运行时解析 TMX 文件 | 游戏不应耦合编辑器格式 | 架构评审不通过 |
| 🟥 Object 直接实例化为 ECS Entity | Object 是定义，实例化策略属 Domain 职责 | 运行期逻辑错误 |
| 🟥 MapRenderer 查询领域组件 | 表现层不应耦合领域逻辑 | 架构评审不通过 |
| 🟥 MapAsset 被运行时修改 | 违反 Definition 不可变原则 | 运行时断言失败 |
| 🟥 MapLoader 中执行业务逻辑（单位生成/战斗初始化） | Loader 职责是 ECS 状态初始化 | 架构评审不通过 |
| 🟥 Importer 依赖于游戏 Core 层 | Importer 是独立工具 | 编译期解耦 |
| 🟥 全局 AppError 用于地图管线错误 | 违反分领域错误枚举原则 | 代码审查不通过 |
| 🟥 硬编码地图尺寸/地形配置 | 违反数据驱动原则 | 代码审查不通过 |
| 🟥 使用 EventWriter/EventReader（Bevy 0.19 禁止） | 必须使用 trigger() + Observer | 编译期警告 |

### 需要同步到宪法的条款

以下条款需要追加到 `docs/00-governance/ai-constitution-complete.md`：

| 条款 | 内容 | 引用 |
|------|------|------|
| §X.1 | Tile 不承载 Gameplay 数值——Tile 只存 TerrainId，数值走 Config Registry | ADR-065 §4 |
| §X.2 | 地图内容管线三层分离：Tiled(TMX) → Importer → MapAsset(RON) | ADR-065 §1 |
| §X.3 | 对象层 Object 是定义而非实例——运行时由 Domain 系统决定实例化 | ADR-065 §5 |
| §X.4 | Importer 是核心资产——编辑器可换，MapAsset 不可变 | ADR-065 §2 |

## Definition / Instance Design

### Definition（不可变配置）

| 类型 | 位置 | 说明 |
|------|------|------|
| `MapAsset` | `assets/config/04_world/maps/*.ron` | Importer 输出的运行时地图数据（L4 World 层） |
| `TerrainDef` | `assets/config/terrains/*.ron` | 地形配置（已在 ADR-022 设计，需接入 Content Pipeline） |
| `TileEntry` | 内嵌于 MapAsset | 每个 Tile 的数据（terrain_id + height + flags） |

### Instance（运行时状态）

| 类型 | 位置 | 说明 |
|------|------|------|
| `GridMap` (Resource) | `src/core/domains/tactical/` | 从 MapAsset.TerrainGrid 构建（已有类型，不改动） |
| `TileMarker` (Component) | `src/core/domains/tactical/` | 交互式 Tile 实体化标记（已有类型，不改动） |
| `MapLoadParams` (Resource) | `src/infra/map/types.rs` | 场景入口参数（场景管道传递） |
| `MapLoaded` (Event) | `src/infra/map/events.rs` | 地图加载完成事件 |

## 本 ADR 与 ADR-022 的关系

本 ADR **补充而非取代** ADR-022：

| 维度 | ADR-022 负责 | ADR-065 补充 |
|------|-------------|-------------|
| GridMap | 内存格式、查询 API、寻路算法 | GridMap 的数据来源（MapAsset.TerrainGrid → GridMap） |
| TileData | packed 表示、TileFlags 定义 | TileData 的创建方式（Importer 从 TMX 填充） |
| TerrainDef | 地形定义结构、Tag 系统桥接 | TerrainDef RON 的 Content Pipeline 加载 |
| TileMarker | 条件性实体化策略 | Marker 的实例化来源（MapObject.class 映射） |
| 高度系统 | HeightSystem API 设计 | height 数据的来源（Importer 从 TMX 填充） |
| 网格系统 | GridPos 坐标、邻居计算 | 网格数据的文件格式（MapAsset.TerrainGrid） |

## 后果

### 正面
- 完整的内容管线：Tiled → Importer → MapAsset → GridMap
- Tile 与 Gameplay 数值解耦：修改地形平衡不改地图文件
- Object Layer 作为一等公民：触发、宝箱、门等交互对象通过 Tiled 编辑
- Importer 验证确保地图数据质量（GUID 唯一性、ID 有效性）
- 与已有架构无缝对接（不修改 Tactical/Terrain Domain 核心类型）
- 渲染自研，灵活控制，不依赖第三方地图库
- GUID 系统支持跨存档稳定的对象追溯

### 负面
- 新增约 15 个 Rust 源文件 + 1 个独立工具 crate
- 内容团队需要学习 Tiled 编辑器 + Importer 工作流
- v1 不包含 Region 运行时系统、Fog of War、动态地图，这些需要后续 ADR
- TerrainDef 需要从旧原型 RON 迁移到 Content Pipeline 格式

## 替代方案

| 方案 | 放弃理由 |
|------|----------|
| 运行时 TMX 解析（用 `tiled` crate） | 游戏二进制需要承载 TMX 解析代码，编辑器耦合运行时；TMX 格式变化影响游戏 |
| 使用 bevy_ecs_tilemap 渲染 | 渲染方式被第三方库绑定，灵活性受限；本项目已经有自研渲染管线 |
| Tile 存储完整 Gameplay 数值 | 违反 Def-Instance 分离；改平衡需改地图文件 |
| Object 直接实例化 Entity | 失去对实例化逻辑的控制；Class 映射不可扩展 |
| 自研地图编辑器替代 Tiled | 成本过高，Tiled 已成熟且功能完善 |
| 全量 Tile 实体化（每个 Tile 一个 Entity） | ADR-022 已论证 Entity 数量过大会 Archetype 爆炸 |
| 在 Config Registry 中嵌入地图数据 | 地图是 L4 World 层数据，不应与 L0-L3 Def 混同 |
| MapAsset 使用 JSON 而非 RON | 项目一致性优先，已有 RON 工具链（序列化/校验/版本控制） |

## 合规性自检

- [x] 符合 Feature First 原则：地图管线属于 Infra（技术实现），不创建设计全局模块
- [x] 符合三层运行时分离：MapAsset=Definition, GridMap=Instance, MapRenderer=Presentation
- [x] 符合 DDD三层+横切四层层间依赖方向：MapAsset 类型在 Infra（L2），不依赖 Core
- [x] Effect Pipeline 没有被绕过：Tile 不存储数值，不走 Effect 管线
- [x] Modifier Pipeline 没有被绕过：与地形数值无关
- [x] 定义了明确的 Forbidden 事项（12 条禁止）
- [x] 引用了上游领域规则：tactical_domain.md, terrain_domain.md
- [x] 引用了数据 Schema：tactical_schema.md, terrain_schema.md
- [x] 检查了内容架构：content-layering.md（L4 World 层），content-platform-manifesto.md
- [x] 检查了 UI 表现层架构：MapRenderer 在 Infra 层，不依赖 UI
- [x] Plugin 注册顺序符合层次要求：Phase 8 Infrastructure
- [x] 通信机制选择符合四级通信规范：Observer(MapLoaded) + Res<GridMap> 直接读取
- [x] 符合 Data Law（Def-Instance 分离、Rule-Content 分离、ID 引用）
- [x] 符合 Bevy 0.19 规范：使用 trigger() + Observer
- [x] 符合 Bevy 0.19 规范：使用 `ButtonInput<T>`（不涉及）
- [x] 符合 Bevy 0.19 规范：所有 Component/Event/Resource 包含 `Reflect`（MapAsset 需要）

## 后续调用建议

| 步骤 | 负责 Agent | 产出 | 依赖 |
|------|-----------|------|------|
| 1 | @content-architect | `docs/03-content/definitions/world/map-def.md` — MapDef L4 定义 | 本 ADR |
| 2 | @content-architect | `docs/03-content/definitions/vocabulary/terrain-def.md` — TerrainDef 内容注册方案 | 本 ADR |
| 3 | @data-architect | `docs/04-data/infrastructure/map-asset-schema.md` — MapAsset 数据 Schema（正式） | 本 ADR |
| 4 | @data-architect | `docs/04-data/infrastructure/map-importer-schema.md` — Importer TMX 映射设计 | 本 ADR |
| 5 | @presentation-architect | `docs/06-ui/04-data-flow/map-rendering.md` — MapRenderer 详细架构设计 | 本 ADR |
| 6 | @feature-developer | TMX Importer 工具实现（`tools/map_importer/`） | 步骤 1-5 |
| 7 | @feature-developer | MapAsset 类型 + MapPlugin + MapLoader 实现 | 步骤 1-5 |
| 8 | @feature-developer | MapRenderer 实现（替换 test_battle 占位彩块） | 步骤 5 |
| 9 | — | 更新宪法：追加 Tile/Config 分离、Importer/MapAsset 条款 | 步骤 6-8 |
| 10 | — | 更新 docs/01-architecture/README.md ADR 索引 | 本 ADR 接受后 |
