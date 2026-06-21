---
id: infrastructure.map-importer.schema.v1
title: Map Importer Schema — TMX 到 MapAsset 映射规范
status: draft
owner: data-architect
created: 2026-06-22
updated: 2026-06-22
layer: definition
replay-safe: true
supersedes: none
---

# Map Importer Schema — TMX 到 MapAsset 映射规范

> **领域归属**: Infrastructure — Map | **依赖 Schema**: MapAsset (TileFlags, TerrainGrid, ObjectLayer, etc.), TerrainDef, Tactical (GridPos, GridLayout) | **定义依据**: `docs/01-architecture/40-cross-cutting/ADR-065-map-content-pipeline.md`, `docs/03-content/definitions/world/map-def.md`, `docs/04-data/infrastructure/map-asset-schema.md`

---

## 1. 映射总览

```
Tiled TMX（编辑格式）                      MapAsset RON（运行时格式）
 │                                              │
 ├── <map>                                       ├── metadata
 │   ├── width/height/tilewidth/tileheight       │   ├── width, height
 │   ├── orientation                             │   ├── layout (GridLayout)
 │   ├── backgroundcolor                         │   └── pixel_width, pixel_height, tile_width, tile_height
 │                                               │
 ├── <tileset>                                   ├── terrain_grid
 │   └── <tile> (gid)                            │   └── tiles[].terrain_id ← Tileset配置映射
 │       └── <properties>                        │   └── tiles[].height ← Property "height"
 │           └── "terrain_id" → "ter:plain"      │   └── tiles[].flags ← TerrainDef.flags
 │                                               │   └── tiles[].gid ← TMX 原始 GID
 │                                               │
 ├── <layer> (tilelayer)                    ──→  │
 │   └── <data>                                  │
 │       └── <tile gid="1"/>                     │
 │                                               │
 ├── <objectgroup>                               ├── object_layers
 │   ├── <object>                                │   └── objects[]
 │   │   ├── id/name/class                       │       ├── guid ← 内容哈希生成
 │   │   ├── x, y                                │       ├── tiled_id ← TMX id
 │   │   ├── width, height                       │       ├── name, class
 │   │   ├── rotation                            │       ├── position ← pixel→grid
 │   │   ├── <point> / <ellipse> / <polygon>     │       ├── width, height (格数)
 │   │   ├── rotation                            │       ├── rotation
 │   │   └── <properties>                        │       ├── shape ← shape 类型
 │   │       └── ...                             │       └── properties ← property 转换
 │   │                                           │
 │   └── <object> class="SpawnPoint"        ──→  ├── spawn_points[]
 │       └── <properties>                        │       ├── guid ← 内容哈希
 │           └── "spawn_group_id"                │       ├── spawn_group_id ← Property
 │           └── "faction"                       │       ├── faction ← Property
 │                                                │       ├── position ← pixel→grid
 │                                                │       ├── facing ← Property
 │                                                │       └── properties ← 剩余属性
 │                                                │
 │                                               ├── regions
 │                                                  ├── id, name_key
 │                                                  ├── tiles ← (从 MapObject class="Region" 转换)
 │                                                  └── properties
 │
 ├── <tileset> ── terrain_id 映射表 ──→          ├── navigation_mask
 │   (不使用 TMX terrains 属性)                   │   ├── width, height ← 从 MapMetadata 复制
 │                                                │   └── data ← 从 TerrainDef.flags 预计算
 │
 └── <properties> (Map 级别 Property)
     └── "map_id" / "name_key"
```

### Importer 上下游结构

```
输入层                            Importer 处理                         输出层
┌─────────────┐           ┌─────────────────────────┐           ┌──────────────────┐
│ Tiled TMX   │  parse    │ tmx_parser.rs           │  convert  │ MapAsset (RON)   │
│ map.tmx     │ ────────→ │   ↓ TiledInternal        │ ────────→ │ map_dragon.ron   │
│             │           │ tiled_types.rs           │           │                  │
│ Tileset     │           │   ↓ converter.rs          │           │ ┌ metadata       │
│ tileset.tsx │  read     │   ↓ guid_gen.rs           │           │ ┌ terrain_grid   │
│             │ ────────→ │   ↓ nav_builder.rs         │           │ ┌ object_layers  │
│ TerrainDef  │           │   ↓ validator.rs           │           │ ┌ spawn_points   │
│ Registry    │  lookup   │   ↑ config.toml (Tileset   │           │ ┌ regions         │
│ (RON)       │ ────────→ │     → TerrainId 映射表)    │           │ ┌ navigation_mask│
└─────────────┘           └─────────────────────────┘           └──────────────────┘
```

---

## 2. Tile Layer 映射

### 2.1 TMX Tile Layer → TerrainGrid

TMX 的 Tile Layer 是二维 GID 数组。Importer 将其转换为 `TerrainGrid.tiles: Vec<TileEntry>`。

```
TMX <layer>:
  <data>
    <tile gid="1"/> <tile gid="2"/> <tile gid="0"/>    ← 行优先顺序
    <tile gid="1"/> <tile gid="1"/> <tile gid="3"/>
  </data>
       │
       │  [GID → TerrainId 映射表查找]
       ▼
MapAsset TerrainGrid:
  tiles: [
    TileEntry { terrain_id: "ter:plain",  height: 0, flags: 0x03, rotation: 0, gid: Some(1) },
    TileEntry { terrain_id: "ter:forest", height: 0, flags: 0x03, rotation: 0, gid: Some(2) },
    ...
  ]
```

### 2.2 GID → TerrainId 映射表

GID（Global Tile ID）是 Tiled 中每个 Tile 的唯一整数标识。Importer 需要一个**配置映射表**来建立 GID 到 TerrainId 的关系。

**映射表来源**：Importer 从 `tools/map_importer/config/tileset_mappings.toml` 读取映射配置：

```toml
# tools/map_importer/config/tileset_mappings.toml
# Tileset 到 TerrainDef ID 的映射配置
#
# GID 范围由 tileset.firstgid 决定（TMX 中定义）。
# firstgid 在 TMX 中是动态的（取决于 tileset 加载顺序），
# 因此映射表使用 tileset name + tile index 而非绝对 GID。

[tilesets.basic_terrain]
# tileset 文件名（不含路径）
file = "basic_terrain.tsx"
# tileset 中每个 Tile 索引 → TerrainId
[tilesets.basic_terrain.mapping]
0 = "ter:void"         # 索引 0 = 空/空洞（通常不可通行）
1 = "ter:plain"        # 索引 1 = 平原
2 = "ter:forest"       # 索引 2 = 森林
3 = "ter:mountain"     # 索引 3 = 山地
4 = "ter:water"        # 索引 4 = 水域
5 = "ter:wall"         # 索引 5 = 墙壁
6 = "ter:road"         # 索引 6 = 道路
7 = "ter:sand"         # 索引 7 = 沙漠/沙地
```

**映射算法**：

```
对于每个 TMX Tile Layer 中的 <tile gid="N">:
  1. 找到包含 GID N 的 tileset（通过 firstgid ≤ N < firstgid + tilecount）
  2. tile_index = N - firstgid
  3. 从映射表中查询: tileset_mapping[tileset_name][tile_index]
  4. 得到的 TerrainId 写入 TileEntry.terrain_id
  5. 若 GID = 0（空 Tile），使用 "ter:void"（一个特殊 TerrainDef，passable=false）
```

**为什么用配置文件而不用 TMX 的 `<terrain>` 属性**：

| 方案 | 缺点 |
|------|------|
| TMX `<terrain>` 属性 | Tiled 的 Terrain 系统是"角/边"定义的刷子系统，不适合 SRPG 的 grid-level terrain mapping |
| TMX Tile `<properties>` | 每个 Tile 打 Property 在内容团队编辑时工作量大，容易遗漏 |
| 独立映射文件 | 集中管理，一次配置，团队共享，版本可控 |

### 2.3 高度填充

TileEntry.height 的填充优先级（从高到低）：

| 优先级 | 来源 | 说明 |
|--------|------|------|
| 1 | TMX Tile Custom Property `"height"` | 在 Tileset 中为特定 Tile 设置 `height` Property |
| 2 | TMX Tile Terrain Type 的 elevation 属性 | 如果使用 TMX <terrain> 定义了 elevation 属性 |
| 3 | 默认值 0 | 所有平坦地形使用 0 |

**高度值转换**：

```
TMX Property "height" (int)    → TileEntry.height (u8, clamped 0-255)
  有效范围: 0-255
  越界处理: <0 → 0, >255 → 255 + 警告
```

### 2.4 Rotation 填充

```
TMX <tile> 的 rotation 属性    → TileEntry.rotation (u8)
  rotation 是 0/90/180/270 → 转换到 0-3:
    0°   → 0
    90°  → 1
    180° → 2
    270° → 3
```

### 2.5 GID 保留

```
TMX <tile gid="N"> 的 GID     → TileEntry.gid (Option<u32>)
  debug 构建: Some(N)
  release 构建: None（通过 #[serde(skip_serializing_if = "Option::is_none")] 省略）
```

---

## 3. Object Layer 映射

### 3.1 TMX Object Group → ObjectLayer

TMX 的每个 `<objectgroup>` 转换为一个 `ObjectLayer`。

```
TMX <objectgroup>:
  id="2"
  name="interactables"
  opacity="1.0"
  visible="true"
  offsetx="0", offsety="0"
       │
       │  [逐字段映射]
       ▼
MapAsset ObjectLayer:
  id: 2
  name: "interactables"
  opacity: 1.0
  visible: true
  offset_x: 0
  offset_y: 0
  objects: [ ... ]
```

**字段映射表**：

| TMX 字段 | MapAsset 字段 | 转换规则 |
|----------|--------------|----------|
| `objectgroup.id` | `id` | 直接复制 |
| `objectgroup.name` | `name` | 直接复制 |
| `objectgroup.opacity` | `opacity` | 直接复制 |
| `objectgroup.visible` | `visible` | 直接复制 |
| `objectgroup.offsetx` | `offset_x` | 直接复制 |
| `objectgroup.offsety` | `offset_y` | 直接复制 |
| `objectgroup.color` | — | 略（TMX 对象分组颜色，运行时不需要） |

### 3.2 TMX Object → MapObject

```
TMX <object>:
  id="101"
  name="chest_01"
  class="Chest"
  x="320", y="512"
  width="64", height="64"
  rotation="0"
  visible="true"
       │
       │  [手写映射 + GUID 生成]
       ▼
MapAsset MapObject:
  guid:        MapObjectGuid(hash("map:dragon_peak", "interactables", "Chest", 5, 8))  ← 内容哈希
  tiled_id:    101              ← TMX id（仅调试用）
  name:        "chest_01"       ← TMX name
  class:       "Chest"          ← TMX class（或 type 回退）
  position:    GridPos(5, 8, 0) ← 像素坐标 → GridPos 转换
  width:       1                ← 像素 → 格数转换
  height:      1                ← 像素 → 格数转换
  rotation:    0.0              ← TMX rotation
  properties:  { ... }          ← TMX properties 转换
  shape:       Rectangle(1, 1)  ← 推导自 width/height + 形状子元素
```

**字段映射表**：

| TMX 字段 | MapAsset 字段 | 转换规则 |
|----------|--------------|----------|
| `object.id` | `tiled_id` | 直接复制（仅用于调试追溯） |
| — | `guid` | **内容哈希生成**（见 §5） |
| `object.name` | `name` | 直接复制 |
| `object.class` | `class` | 直接复制。若 class 为空，尝试回退到 `object.type`（TMX 旧版 type 字段） |
| `object.x` / `object.y` → `position` | `position` | **像素→网格坐标转换**（见 §3.3） |
| `object.width` / `object.height` | `width`, `height` | **像素→格数转换**：`grid_units = ceil(pixels / tile_size)` |
| `object.rotation` | `rotation` | 直接复制（度） |
| `object.visible` | — | 略（Object 本身都是定义的实例化模板，visible 含义不同） |
| `<point>`, `<ellipse>`, `<polygon>`, `<polyline>` | `shape` | 形状转换（见 §3.4） |
| `<properties>` | `properties` | Property 转换（见 §4） |

### 3.3 像素坐标 → GridPos 转换

```rust
/// 将 TMX 像素坐标转换为 GridPos。
///
/// TMX 坐标系统:
///   - X: 从左到右递增
///   - Y: 从上到下递增
///   - 原点: 地图左上角
///   - 像素坐标是 Tile 的左上角
///
/// GridPos 坐标系统:
///   - X: 从左到右递增
///   - Y: 从下到上递增（如果网格是 Hex, Y 翻转）
///   - 网格坐标是 Tile 的中心
///
/// 转换公式（方形网格）:
///   grid_x = floor(pixel_x / tile_width)
///   grid_y = floor(pixel_y / tile_height)
///   → GridPos { x: grid_x, y: grid_y, layer: 0 }
pub fn pixel_to_gridpos(
    pixel_x: f32,
    pixel_y: f32,
    tile_width: u32,
    tile_height: u32,
    layout: GridLayout,
) -> GridPos {
    let (gx, gy) = match layout {
        GridLayout::Square => {
            let gx = (pixel_x / tile_width as f32).floor() as i32;
            let gy = (pixel_y / tile_height as f32).floor() as i32;
            (gx, gy)
        }
        GridLayout::HexRowOdd | GridLayout::HexRowEven => {
            // Hex 坐标转换：使用 axial 坐标公式
            // 简化版：先做方形近似，再通过 hex math 精修
            let col = (pixel_x / (tile_width as f32 * 0.75)).floor() as i32;
            let row = (pixel_y / tile_height as f32).floor() as i32;
            // offset 矫正
            let is_row_offset = col % 2 != 0;
            let row_offset = if is_row_offset { -0.5 } else { 0.0 };
            let gy = ((pixel_y / tile_height as f32) + row_offset).round() as i32;
            (col, gy)
        }
        // ... 其他 Hex 变体类似
        _ => ((pixel_x / tile_width as f32).floor() as i32,
             (pixel_y / tile_height as f32).floor() as i32),
    };
    GridPos::new(gx, gy)
}
```

### 3.4 ObjectShape 转换

| TMX 子元素 | MapAsset::ObjectShape | 说明 |
|-----------|----------------------|------|
| (无子元素) | `Point` | 默认形状 |
| `<point/>` | `Point` | 显式点 |
| (width/height 都不为 0, 无子元素) | `Rectangle { width, height }` | 默认矩形 |
| `<ellipse/>` | `Ellipse { width, height }` | 椭圆，尺寸转换同矩形 |
| `<polygon points="..."/>` | `Polygon { points }` | 像素坐标顶点列表（相对于对象原点） |
| `<polyline points="..."/>` | `Polyline { points }` | 像素坐标顶点列表（相对于对象原点） |

**注意**：Rectangle 和 Ellipse 的 width/height 使用像素→网格转换（`ceil(pixel / tile_size)`），其他形状保持像素坐标。

### 3.5 SpawnPoint 的特殊处理

SpawnPoint 是一个特殊的 Object 实例，`class: "SpawnPoint"` 的 MapObject 会被转换为 `MapAsset.spawn_points` 列表中的 `SpawnPoint` 结构。

**SpawnPoint 转换规则**：

| 来源 | SpawnPoint 字段 | 规则 |
|------|----------------|------|
| 同 Object 的 GUID | `guid` | 内容哈希 |
| Property `"spawn_group_id"` | `spawn_group_id` | **必需** Property。若无，Importer 报错 |
| Object.position | `position` | 像素→网格坐标转换 |
| Property `"faction"` | `faction` | 可选。格式为 FactionId（如 `"faction:enemy"`）。解析失败则发出警告并设为 None |
| Property `"facing"` | `facing` | 可选。格式为 `"N"`/`"NE"`/`"S"` 等。未知方向值→默认 South。缺失→默认 South |
| Object.properties 中的剩余字段 | `properties` | 除 `spawn_group_id`/`faction`/`facing` 外的所有 Property 保留 |

**为什么不将 SpawnPoint 保留在 ObjectLayer 中**？
SpawnPoint 需要显式的类型化字段（spawn_group_id, faction, facing），放在 ObjectLayer 中作为泛型 MapObject 会丢失类型安全。将其作为顶级 `spawn_points: Vec<SpawnPoint>` 单独列出，使得运行时 EncounterSystem 可以直接消费，不需 PropertyMap 解析。

### 3.6 Region 的特殊处理

同 SpawnPoint，`class: "Region"` 的 MapObject 会被转换为 `MapAsset.regions` 列表中的 `MapRegion` 结构。

**Region 转换规则**：

| 来源 | MapRegion 字段 | 规则 |
|------|---------------|------|
| Object.name | `id` | 直接复制。须在同一地图内唯一 |
| Property `"name_key"` 或 Object.name | `name_key` | 优先使用 Property "name_key"。若不存在，使用 `"region.{map_id}.{object_name}.name"` 自动生成 |
| Object 覆盖的 Tile 范围 | `tiles` | 从 Object 的 shape + position + width/height 计算覆盖的所有 GridPos |
| Object.properties 中的剩余字段 | `properties` | 全部保留 |

**Tile 集计算**：如果 Object 是 Rectangle（width=3, height=2），覆盖的 Tile 位置从 (x, y) 到 (x+2, y+1) 的所有 GridPos。

---

## 4. Property 映射

### 4.1 Tiled Property → PropertyMap 类型转换

```
TMX Property                      MapAsset PropertyValue
────────────────────────────────  ────────────────────────
<string name="event_id"           PropertyValue::String("evt:chest_open")
  value="evt:chest_open"/>

<int name="difficulty"            PropertyValue::Int(3)
  value="3"/>

<float name="chance"              PropertyValue::Float(0.75)
  value="0.75"/>

<bool name="locked"               PropertyValue::Bool(true)
  value="true"/>

<color name="tint"                PropertyValue::Color([1.0, 0.5, 0.5, 1.0])
  value="#ff8080ff"/>

<file name="script"               PropertyValue::File("scripts/trigger.lua")
  value="scripts/trigger.lua"/>
```

**TMX Property 类型到 MapAsset 类型的映射**：

| TMX Property Type | MapAsset::PropertyValue | 说明 |
|-------------------|------------------------|------|
| `string` | `String(String)` | 默认类型。无 explicit type 时也按 string 处理 |
| `int` | `Int(i32)` | 整数值，TMX 存为 int。注意 Tiled 中 color 也是 int 格式 |
| `float` | `Float(f32)` | 浮点值 |
| `bool` | `Bool(bool)` | 布尔值 |
| `color` | `Color([f32; 4])` | TMX 颜色格式 `#AARRGGBB`，转换为 RGBA 各分量 0.0-1.0 |
| `file` | `File(String)` | 文件路径，相对路径保持相对，绝对路径发出警告 |

**颜色格式转换算法**：

```rust
/// TMX 颜色格式 `#AARRGGBB` → `[f32; 4]` (RGBA, 0.0-1.0)
/// TMX 颜色格式 `#RRGGBB` → `[f32; 4]` (RGBA, A=1.0)
pub fn tmx_color_to_rgba(hex: &str) -> [f32; 4] {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        8 => { // AARRGGBB
            let a = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255) as f32 / 255.0;
            let r = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[6..8], 16).unwrap_or(0) as f32 / 255.0;
            [r, g, b, a]
        }
        6 => { // RRGGBB
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
            [r, g, b, 1.0]
        }
        _ => [1.0, 1.0, 1.0, 1.0], // 默认白色
    }
}
```

### 4.2 Property 命名约定

内容团队在 Tiled 中设置 Property 时应遵循以下命名约定，确保 PropertyMap 对 Domain System 可用：

| Property 名 | 类型 | 消费方 | 说明 |
|-------------|------|--------|------|
| `spawn_group_id` | string | EncounterSystem | SpawnPoint 必需。引用 SpawnGroupDef |
| `faction` | string | EncounterSystem | SpawnPoint 可选。FactionId |
| `facing` | string | EncounterSystem | SpawnPoint 可选。方向字面量 |
| `event_id` | string | InteractionSystem | Chest/Trigger 必需。引用 EventDef |
| `locked` | bool | InteractionSystem | Chest/Door 可选。默认 false |
| `key_id` | string | InteractionSystem | Chest/Door 可选。所需钥匙 ItemId |
| `hazard_id` | string | TerrainDomain | Hazard 必需。引用 HazardZoneDef |
| `trigger_condition` | string | TerrainDomain | Hazard 可选。触发条件 |
| `encounter_id` | string | CombatDomain | Region 可选。引用 EncounterDef |
| `no_combat` | bool | CombatDomain | Region 可选。安全区域标记 |
| `dialogue_id` | string | NarrativeSystem | Trigger 可选。引用 DialogueDef |
| `one_shot` | bool | NarrativeSystem | Trigger 可选。仅触发一次 |

**未知 Property**：Importer 不丢弃未知 Property——所有未识别的 Property 都被保留到 PropertyMap 中。Domain System 读取时决定的字段才决定其含义。

### 4.3 Map 级别 Property

TMX 的 Map 级别 Property（`<map><properties>...</properties></map>`）被映射到 MapMetadata：

| TMX Map Property | MapMetadata 字段 | 规则 |
|------------------|-----------------|------|
| `"map_id"` | `id` | **必需**。格式: `map:dragon_peak`。缺失→使用 TMX filename 自动生成 |
| `"name_key"` | `name_key` | **必需**。LocalizationKey。缺失→使用 `"map.{file_name}.name"` 自动生成 |

Map 级别的其他 Property 被丢弃（不属于 MapAsset Schema 定义的字段）。

---

## 5. GUID 生成

### 5.1 算法规范

MapObject 和 SpawnPoint 的 GUID 使用**确定性内容哈希**生成，不依赖 Tiled 内部 ID。

```
GUID = SipHash-2-4(
    map_id: &str,
    layer_name: &str,
    object_class: &str,
    position_x: i32,
    position_y: i32,
)
```

**实现参考**：

```rust
use std::hash::{Hash, Hasher};
use siphasher::sip::SipHasher;

/// 生成稳定 GUID——相同输入永远产生相同输出。
///
/// 输入参数：
///   map_id: MapAsset 的 metadata.id（如 "map:dragon_peak"）
///   layer_name: 对象所在的层名称
///   object_class: 对象的 class 字段
///   pos: 对象的网格位置
///
/// 输出：u64 GUID
pub fn generate_guid(
    map_id: &str,
    layer_name: &str,
    object_class: &str,
    pos: GridPos,
) -> MapObjectGuid {
    let mut hasher = SipHasher::new_with_keys(0, 0); // 固定 key 确保确定性
    map_id.hash(&mut hasher);
    layer_name.hash(&mut hasher);
    object_class.hash(&mut hasher);
    pos.x.hash(&mut hasher);
    pos.y.hash(&mut hasher);
    MapObjectGuid(hasher.finish())
}
```

### 5.2 设计要点

| 属性 | 说明 |
|------|------|
| **稳定性** | 同一 TMX 文件 → 同一 MapAsset → 同一 GUID。不依赖 Tiled ID（Tiled ID 在重新编辑时会变） |
| **确定性** | 使用固定 key 的 SipHash-2-4，非 HashMap 默认的随机 key |
| **跨存档** | GUID 在存档不保存 MapAsset 的前提下，通过 map_id + layer + class + pos 可重新生成 |
| **唯一性保证** | 同一地图内唯一（map_id + layer + class + pos 的组合在不同对象上不会重复） |
| **不可逆** | u64 哈希不可反向推导，不适合安全性场景（不需要，仅用于标识） |

### 5.3 为什么不使用 UUID？

| 方案 | 问题 |
|------|------|
| UUID v4（随机） | 非确定性，每次 Importer 运行生成不同 ID → 无法跨版本追溯 |
| TMX Object ID | Tiled 重新编辑后可能变化 → 不稳定 |
| 自增 ID | 依赖 Importer 运行顺序 → 稳定性差 |

内容哈希是唯一满足所有需求的方案。

---

## 6. NavigationMask 生成

### 6.1 算法

```
对于 terrain_grid.tiles 中的每个 TileEntry:
  1. 通过 terrain_id 查询对应的 TerrainDef
  2. 根据 TerrainDef.flags 设置 NavigationMask byte:
     - 位 0 (WALK) = TerrainDef.flags.passable
     - 位 1 (FLY)  = TerrainDef.flags.flyable
     - 位 2 (SWIM) = terrain_id == "ter:water" 或自定义规则
     - 位 3-7: 保留位 (0)
  3. 写入 navigation_mask.data[index]
```

```rust
/// 从 TerrainGrid 预计算 NavigationMask。
///
/// 参数:
///   grid: TerrainGrid（MapAsset 中的地形网格）
///   terrain_registry: TerrainDef Registry（查询通行性）
///
/// 返回: NavigationMask
pub fn build_navigation_mask(
    grid: &TerrainGrid,
    terrain_registry: &HashMap<TerrainId, TerrainDef>,
) -> NavigationMask {
    let size = (grid.width * grid.height) as usize;
    let mut data = vec![0u8; size];

    for (i, tile) in grid.tiles.iter().enumerate() {
        if let Some(def) = terrain_registry.get(&tile.terrain_id) {
            let mut byte = 0u8;
            if def.flags.passable { byte |= 0x01; }
            if def.flags.flyable  { byte |= 0x02; }
            // SWIM 位：从 terrain_id 判断或通过额外规则
            data[i] = byte;
        }
    }

    NavigationMask {
        width: grid.width,
        height: grid.height,
        data,
    }
}
```

### 6.2 一致性验证

NavigationMask 生成后，Importer 执行一致性检查（V17）：

```rust
/// 验证 NavigationMask 与 TileEntry.flags 是否一致。
///
/// WALK 位应与 TileFlags.PASSABLE 一致。
/// FLY 位应与 TileFlags.FLYABLE 一致。
pub fn validate_navigation_consistency(
    grid: &TerrainGrid,
    nav: &NavigationMask,
) -> Vec<String> {
    let mut warnings = Vec::new();
    for (i, tile) in grid.tiles.iter().enumerate() {
        let nav_walk = (nav.data[i] & 0x01) != 0;
        let tile_walk = tile.flags.contains(TileFlags::PASSABLE);
        if nav_walk != tile_walk {
            let x = i as u32 % grid.width;
            let y = i as u32 / grid.width;
            warnings.push(format!(
                "POS({},{}) Walk: nav={} tile={}",
                x, y, nav_walk, tile_walk
            ));
        }
    }
    warnings
}
```

---

## 7. 验证规则

### 7.1 Importer 构建时验证（8 项）

Importer 在生成 MapAsset RON 之前执行以下验证，验证失败时拒绝输出。

| # | 验证项 | 检查内容 | 失败处理 |
|---|--------|----------|----------|
| V1 | GUID 唯一性 | 所有 ObjectLayer + SpawnPoints 中无重复 GUID | 报错 + 停止 |
| V2 | TerrainId 有效性 | 每个 `TileEntry.terrain_id` 在 TerrainDef Registry 中存在 | 报错 + 停止 |
| V3 | SpawnGroupId 引用完整性 | 每个 `SpawnPoint.spawn_group_id` 在 DefRegistry<SpawnGroupDef> 中存在 | 警告 + 继续 |
| V4 | 高度连续性 | 相邻 Tile 高度差 ≤ 3（最大坡度） | 警告 + 继续 |
| V5 | 网格完整性 | 所有 grid 位置有有效 TileEntry（GID=0 → "ter:void" 填充，不出现空洞） | 报错 + 停止 |
| V6 | 尺寸一致性 | width/height 与 tiles.len() 一致 | 报错 + 停止 |
| V7 | 必要属性 | Object 的必需 Property 存在（如 Trigger 需要 event_id） | 警告 + 继续 |
| V8 | 导航一致性 | navigation_mask 与 terrain_id 的通行性一致 | 警告 + 继续 |

### 7.2 验证失败处理策略

| 级别 | 行为 | 示例 |
|------|------|------|
| **Error** | 中断导入，不生成 MapAsset。输出错误信息到 stderr | GUID 冲突、TerrainId 不存在、尺寸不一致 |
| **Warning** | 继续导入，输出警告到 stdout | SpawnGroupId 未注册、高度连续性问题 |

### 7.3 运行时验证

MapLoader（debug 模式）在加载 MapAsset 后进行快速完整性检查：

| # | 验证项 | 说明 |
|---|--------|------|
| R1 | width * height == tiles.len() | 网格完整性 |
| R2 | SpawnGroupId 二次校验 | 所有 SpawnPoint 的 spawn_group_id 在 L3 已注册 |
| R3 | GUID 无冲突 | 实例化时检测 GUID 冲突 |
| R4 | TerrainId 快速检查 | tiles[0..5] 的 terrain_id 可解析（抽样） |

运行时验证仅在 debug 模式下执行，release 模式跳过——这些检查已在 Importer 阶段完成。

---

## 8. 转换为 GridMap 的算法

MapLoader 将 MapAsset.TerrainGrid 转换为 Tactical Domain 的 GridMap（packed u32）：

```rust
/// 将 MapAsset.TerrainGrid 转换为 GridMap Resource。
///
/// 关键转换：
///   TileEntry.terrain_id (String) → TileData.terrain_def_id (u16)
///   TileEntry.height (u8)         → TileData.height (u8)
///   TileEntry.flags (TileFlags)   → TileData.flags (TileFlags)
///
/// 需要 terrain_id → u16 索引映射表（在 TerrainDef Registry 加载后由
/// Content Plugin 构建）。若映射表中不存在某 terrain_id，报错。
pub fn terrain_grid_to_gridmap(
    terrain_grid: &TerrainGrid,
    terrain_index: &HashMap<TerrainId, u16>,
    layout: GridLayout,
) -> GridMap {
    let tiles: Vec<TileData> = terrain_grid.tiles
        .iter()
        .map(|entry| {
            let def_id = terrain_index
                .get(&entry.terrain_id)
                .expect(&format!(
                    "TerrainId '{}' not found in registry",
                    entry.terrain_id
                ));
            TileData::new(*def_id, entry.height, entry.flags)
        })
        .collect();

    GridMap::from_tiles(terrain_grid.width, terrain_grid.height, tiles, layout)
}
```

**terrain_index 构建**（在 Content Plugin 加载 TerrainDef 后执行）：

```rust
/// 构建 terrain_id → u16 索引映射表。
///
/// TerrainDef Registry 按注册顺序分配 u16 索引。
/// 索引在应用运行期间稳定（DefRegistry 冻结后不变）。
pub fn build_terrain_index(registry: &DefRegistry<TerrainDef>) -> HashMap<TerrainId, u16> {
    registry.iter()
        .enumerate()
        .map(|(i, def)| (def.id.clone(), i as u16))
        .collect()
}
```

---

## 9. 依赖分析（Importer 工具视角）

| 依赖 | 说明 |
|------|------|
| `quick-xml` 或 `tiled` crate | TMX XML 解析 |
| `siphasher` | 内容哈希（稳定性优先于加密） |
| MapAsset 类型定义 | `TerrainGrid`, `TileEntry`, `MapObject`, `PropertyMap` 等 |
| TerrainDef 格式 | 读取 TerrainDef RON 获取通行性标志（TerrainDef.flags） |
| Tileset 映射配置 | `tools/map_importer/config/tileset_mappings.toml` |
| `serde` + `ron` | MapAsset RON 序列化输出 |

**反向依赖**：Importer 是独立工具，不依赖游戏 Core 层或 ECS。仅依赖 MapAsset 类型定义（作为共享 crate 或通过模块引用）。

---

## 10. 配置示例

### 10.1 tileset_mappings.toml

```toml
# tools/map_importer/config/tileset_mappings.toml
# Tileset → TerrainId 映射配置

[tilesets.basic_terrain]
file = "basic_terrain.tsx"
[tilesets.basic_terrain.mapping]
0 = "ter:void"
1 = "ter:plain"
2 = "ter:forest"
3 = "ter:mountain"
4 = "ter:water"
5 = "ter:wall"
6 = "ter:road"

[tilesets.dungeon_tileset]
file = "dungeon_tileset.tsx"
[tilesets.dungeon_tileset.mapping]
0 = "ter:void"
1 = "ter:stone_floor"
2 = "ter:stone_wall"
3 = "ter:trap_floor"
4 = "ter:lava"
5 = "ter:door_closed"
```

### 10.2 Importer CLI 使用

```bash
# 基本用法
cargo run --bin importer -- -i maps/dragon_peak.tmx -o assets/config/04_world/maps/map_dragon_peak.ron

# 带 TerrainDef Registry 路径（用于验证 TerrainId 存在性）
cargo run --bin importer -- \
  -i maps/dragon_peak.tmx \
  -o assets/config/04_world/maps/map_dragon_peak.ron \
  --terrain-registry assets/config/00_vocabulary/terrains.ron \
  --tileset-config tools/map_importer/config/tileset_mappings.toml

# 批处理模式（处理目录下所有 TMX）
cargo run --bin importer -- --batch maps/ -o assets/config/04_world/maps/
```

---

## 11. Risks

| 风险 | 级别 | 说明 | 缓解措施 |
|------|------|------|----------|
| Tiled 版本升级导致 TMX 格式变化 | 低 | Tiled XML 格式相对稳定，但可能会有扩展 | Importer 使用成熟的 TMX 解析库（`tiled` crate），依赖库维护 |
| Tileset 映射配置与实际 TMX 不匹配 | 中 | Tileset firstgid 变化（增加 tileset 时）导致映射错误 | Importer 读取 TMX 中的 firstgid 动态计算绝对 GID，不与配置文件中的硬编码 GID 比较 |
| TerrainDef Registry 更新 | 低 | TerrainDef 通行性变化后 NavigationMask 过时 | 工作流要求：修改 TerrainDef 后重新导入所有变更 TMX |
| 大文件性能 | 低 | 100x100 地图 1 万个 Tile，导入时间可忽略 | 如果未来地图超过 256x256，考虑导入进度条 |
| TMX 中 GID 0 处理 | 低 | GID=0 表示空 Tile | 映射 "ter:void"（passable=false 的特殊 terrain） |
| Object 数量过多 | 低 | 单个地图数千个 Object 时 GUID 生成性能 | 内容哈希无瓶颈 |

---

*本文档由 @data-architect 维护。*
