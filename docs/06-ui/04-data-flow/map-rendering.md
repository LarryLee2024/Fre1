---
id: 06-ui.map-rendering
title: MapRenderer Architecture — 地图渲染器架构设计
status: draft
owner: presentation-architect
created: 2026-06-22
tags:
  - ui
  - map
  - renderer
  - infra
  - camera
  - overlay
  - batch-rendering
  - tilemap
---

# MapRenderer Architecture — 地图渲染器架构设计

> **职责**: @presentation-architect | **上游**: ADR-065 §7 (Map 渲染架构) | **实现位置**: `src/infra/map/renderer/`
> **层**: Infra (L2) — 基础设施渲染组件，非 UI 层
> **关键约束**: MapRenderer 是纯表现组件——不包含业务逻辑、不查 Domain 组件

> **SSPEC参考**: docs/06-ui/07-specs/ — AI-Consumable Screen Specification 标准。新增 Screen 必须先写 SSPEC，见 ADR-066。

---

## 1. 定位与边界

### 1.1 定义

MapRenderer 是 Fre 项目的 2D 瓦片地图渲染基础设施，负责：

- 将 GridMap 的地形数据渲染为 2D 瓦片地图
- 渲染半透明高亮覆盖层（移动范围、AOE 预览、悬停指示）
- 提供光标/选择指示器
- 集成 Camera 系统（坐标转换、可见性判断）
- 支持调试网格线

### 1.2 不属于 MapRenderer 的职责

| 职责 | 位置 | 理由 |
|------|------|------|
| 单位渲染 | 单位 Entity 自身持有 `Sprite` + `Transform` | 单位是独立 Gameplay Entity，非渲染层管理 |
| UI HUD | `src/ui/screens/battle/` | UI 层管理，使用 Screen/Widget/ViewModel 模式 |
| 地图数据管理 | `src/core/domains/tactical/` (GridMap Resource) | Domain 层管理，Def-Instance 分离 |
| 镜头控制 | `src/infra/camera/` | Camera 有独立状态机 (ADR-064) |
| 地图加载逻辑 | `src/infra/map/systems/map_loader_system.rs` | Loader 职责是 ECS 状态初始化，不含渲染 |
| Overlay 数据计算（移动范围/AOE） | Domain 层计算，通过 Resource 传递 | MapRenderer 只消费不计算 |

### 1.3 层定位

```
Domain Layer (L1 Core)
  └─ Tactical: GridMap, MovementRange
  └─ Combat:   AOESet
        │
        ▼  [App-level bridge: reads Domain output → writes Infra Resource]
        │
Infra Layer (L2)
  └─ camera/      — Camera 状态机、坐标转换 API (CameraQuery)
  └─ map/
       ├─ asset.rs         — MapAsset 类型定义
       ├─ loader.rs        — MapAsset → GridMap + ECS State
       └─ renderer/        — MapRenderer（本文档）
             └─ 读取 CameraQuery      ✅ 允许（Camera 是 Infra）
             └─ 读取 MapOverlayData   ✅ 允许（Overlay 是渲染数据）
             └─ 不包含业务逻辑          ✅ 强制

UI Layer (L3 Presentation)
  └─ screens/battle/  — BattleScreen（覆盖在地图之上的 HUD）
  └─ overlays/         — Tooltip/Notification（UI 空间，非世界空间）
```

**依赖验证**：
- MapRenderer **依赖** infra/camera（使用 CameraQuery）— 允许，同层
- MapRenderer **不依赖** core/domains/*（不查询领域组件）— 强制
- MapRenderer **不依赖** ui/*（对 UI 层一无所知）— 强制
- Camera **不依赖** MapRenderer — 强制（Camera 是通用基础设施）

### 1.4 与 UI 层的关系

MapRenderer 和 UI 层是两个独立的 Presentaiton 组件，使用不同的渲染空间：

| 维度 | MapRenderer | UI (BattleScreen) |
|------|------------|-------------------|
| 渲染空间 | 世界空间 (2D Sprite) | 屏幕空间 (UI Node) |
| 渲染机制 | `Sprite` / `Mesh2d` / `Material2d` | `Node` / `Button` / `Text` |
| 负责内容 | 地形、高亮、光标 | HUD、面板、按钮、文本 |
| z-order 关系 | 底层（世界空间） | 上层（UI Pass 覆盖 Camera 输出） |
| 坐标系统 | GridPos → World Pos | Screen Pos |

**交互方式**：

```
用户点击地图上的 Tile
  → CameraQuery::screen_to_world(screen_pos, ...) → world_pos
  → GridMap.world_to_grid(world_x, world_y) → GridPos
  → UiAction::TileClicked(grid_pos)
  → BattleScreen 处理 → Domain Command
```

---

## 2. 渲染层结构

### 2.1 六层渲染体系

```rust
// 每层的 z 值常量（统一管理，避免硬编码）
pub mod map_z {
    pub const TERRAIN:     f32 = 0.0;   // 地面层
    pub const DECORATION:  f32 = 0.1;   // 装饰层
    pub const GRID_LINES:  f32 = 0.5;   // 网格线
    pub const OVERLAY:     f32 = 0.8;   // 高亮覆盖层
    pub const CURSOR:      f32 = 1.0;   // 光标层
    pub const UNIT_SHADOW: f32 = 2.0;   // 单位阴影（在 Unit Entity 上）
    pub const UNIT:        f32 = 3.0;   // 单位层（在 Unit Entity 上）
}
```

| 层 | z-value | 内容 | 渲染方式 | 数据来源 | 帧更新? |
|----|---------|------|---------|---------|---------|
| **地面层** (Terrain) | 0.0 | 地形瓦片 Sprite | `Material2d` 批处理 | MapAsset → TerrainTextureMap | 否（地图加载时一次） |
| **装饰层** (Decoration) | 0.1 | 静态装饰（树、岩石） | 独立 `Sprite` Entity | MapAsset.object_layers (class:Decor) | 否（实例化时一次） |
| **网格线层** (Grid Overlay) | 0.5 | 调试网格线 | `Mesh2d` 线框 | GridMap.width/height | 否（调试开关） |
| **高亮层** (Overlay) | 0.8 | 移动范围/AOE/悬停 | `Material2d` 批处理 | `MapOverlayData` Resource | 是（高亮数据变化时） |
| **光标层** (Cursor) | 1.0 | 鼠标悬停/选择指示 | 独立 `Sprite` Entity | 悬停 GridPos + 样式 | 是（每帧） |
| **单位层** (Unit) | 2.0~4.0 | 单位 Sprite + 阴影 | 独立 `Sprite` Entity | 单位 Entity 自身 | 是（状态变化时） |

### 2.2 z-order 设计依据

1. **地面层在最底层 (0.0)** — 所有内容的基础
2. **装饰层略高 (0.1)** — 贴在地面上，可能有立体感
3. **网格线在装饰之上 (0.5)** — 确保网格线不被遮挡
4. **高亮层在网格之上 (0.8)** — 覆盖层必须可见
5. **光标在最高覆盖层 (1.0)** — 始终看到鼠标位置
6. **单位阴影/单位 (2.0~4.0)** — 高于所有地图层，确保单位始终可见
7. **单位之间有子排序** — 同一 z 值的单位通过 y 坐标排序（类似 2D 游戏的自上而下排序）

单位层属于 Unit Entity 而非 MapRenderer，这里列出是为了展示完整的 z-order 体系。

### 2.3 渲染管线结构

```
[Per-frame update]
      │
      ├── Overlay 数据更新（Domain → MapOverlayData）
      │     └── MovementRange, AOESet, HoverTile, SelectionTile
      │
      ├── MapRenderer Overlay 批处理重构建
      │     └── 读取 MapOverlayData → 更新 Overlay Mesh
      │
      ├── Cursor 位置更新（Input → hover GridPos → 更新 Cursor Transform）
      │
      └── (Unit 独立更新 Transform，不属于 MapRenderer 管线)
```

---

## 3. Tile Sprite 管理

### 3.1 架构决策: Material2d 批处理

**决策**: 地面层使用 `Material2d` + 纹理图集批处理渲染，不用 Entity-per-Tile。

**对比**: 两种方案在 30x30 网格下均可行，但选择批处理以保证架构扩展性。

| 维度 | Entity-per-Tile | Material2d 批处理 |
|------|----------------|-------------------|
| 900 Tile 的 Entity 数 | 900 Sprite + 900 Transform | 1 Mesh2d + 1 Material |
| Draw calls | ~900 | ~1 |
| CPU 负载 | ~900 Transform sync | ~1 sync + 1 mesh rebuild |
| 单 Tile 交互 | 容易被 Widget/Tooltip 引用 | 需要 screen_to_grid 转换 |
| 热更新单 Tile | 直接改 Sprite | 需要局部网格更新 |
| 大网格扩展性 (200x200) | ~40000 Entity，架构脆弱 | Mesh 更新，架构不变 |

**推荐路径**: V1 实现使用批处理（Material2d），Mesh 在 MapAsset 加载时构建一次。如果未来需要单 Tile 高亮/动画效果（如选中的 Tile 闪烁），可以在材质中加 Shader 参数或使用 Instance 数据。

### 3.2 批处理实现方案

```rust
/// 地形瓦片批处理网格
///
/// 一个 Mesh2d Entity，包含地图上所有 Tile 的四边形。
/// 在 `OnEnter(Combat)` / 地图加载时构建一次，后续不变。
///
/// 构建流程:
/// 1. 遍历 GridMap 所有 Tile
/// 2. 对每个 Tile:
///    a. grid_to_world(pos) → 世界坐标
///    b. terrain_id → TerrainTextureMap → 图集 UV 坐标
///    c. 生成一个四边形（4 顶点 + 6 索引）
/// 3. 合并所有四边形到一个 Mesh
pub fn build_terrain_batch(
    grid_map: Res<GridMap>,
    texture_map: Res<TerrainTextureMap>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    map_renderer: ResMut<MapRendererState>,
) {
    // 生成合并网格
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for y in 0..grid_map.height {
        for x in 0..grid_map.width {
            let pos = GridPos::new(x as i32, y as i32);
            let tile = grid_map.get_tile(pos).unwrap();

            let (wx, wy) = grid_map.grid_to_world(pos);  // ← 与 Domain 共享的坐标转换
            let terrain_id = tile.terrain_def_id();
            let atlas_idx = texture_map.terrain_to_atlas.get(&terrain_id);

            // 生成四边形顶点...
            // 位置: (wx, wy) + Tile 尺寸偏移
            // UV: 根据 atlas_idx 计算
            // 顶点颜色: 1.0 (白)
        }
    }

    // 创建 Mesh
    let mesh = Mesh::new(/* ... */);
    let mesh_handle = meshes.add(mesh);

    // 生成 Mesh2d Entity
    commands.spawn((
        Mesh2d(mesh_handle),
        MeshMaterial2d(texture_map.material_handle.clone()),
        Transform::from_xyz(0.0, 0.0, map_z::TERRAIN),
        Visibility::default(),
        Name::new("TerrainBatch"),
    ));
}
```

### 3.3 纹理映射系统

```rust
/// 地形 ID → 纹理图集索引映射
///
/// 在 MapAsset 加载阶段构建，是 TileEntry.terrain_id
/// 到渲染所需纹理信息的桥梁。
#[derive(Resource)]
pub struct TerrainTextureMap {
    /// terrain_def_id (u16) → 图集 tile 索引
    pub terrain_to_atlas: HashMap<u16, usize>,
    /// 纹理图集布局（描述每个 tile 的 UV 矩形）
    pub atlas_layout: Handle<TextureAtlasLayout>,
    /// 原始纹理 image
    pub texture: Handle<Image>,
    /// 材质 Handle（共享材质，被 TerrainBatch Entity 使用）
    pub material_handle: Handle<TileTerrainMaterial>,
    /// 每 Tile 像素尺寸（用于 UV 计算和顶点位移）
    pub tile_pixel_size: (u32, u32),
}
```

**数据来源**: terrain_id → atlas index 映射由 MapAsset 加载时确定。映射数据来自：
- Importer 输出的 `MapAsset` 内嵌的 tileset 映射信息（Tiled 原始映射）
- 或由 Content 层的 `TerrainVisualDef` 定义（v2 可选方案）

```rust
// MapAsset 中的图集映射信息（Importer 生成）
// 仅在加载 MapAsset 时使用，运行时不保留
pub struct ImporterAtlasInfo {
    /// 每 tile 像素尺寸
    pub tile_width: u32,
    pub tile_height: u32,
    /// terrain_id → 图集坐标
    pub mappings: Vec<(u16, AtlasCoordinate)>,
}
pub struct AtlasCoordinate {
    pub x: u32,
    pub y: u32,
}
```

### 3.4 材质设计

```rust
/// 地形瓦片材质 — 接收纹理图集 + 顶点 UV
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TileTerrainMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub tileset_texture: Handle<Image>,
}
```

**扩展路径**: 未来可以为材质添加 Shader 参数实现：
- 旗帜动画（波动的草地）
- 季节变化（顶部 tint 颜色）
- 高度阴影（根据 TileData.height 调整亮度）
- 战争迷雾（额外 overlay 纹理）

**设计原则**: 材质是表现层，不感知业务语义。Season/Tint 等效果应该传递颜色/纹理参数而非领域标签。

### 3.5 Tile 批处理重建时机

| 场景 | 处理方式 | 频率 |
|------|---------|------|
| 地图加载 | 构建完整批处理 Mesh | 一次 |
| 地形变化（如破坏地形） | 局部更新 Mesh（受影响的 Tile 区域） | 极低 |
| 视觉调试 | 仅在 dev 模式支持重建 | 手动触发 |
| 场景切换 | 销毁 MapRenderer，OnEnter 时重建 | 一次 |

**注意**: V1 中不支持运行时地形变化。地形变化由未来动态地图特性覆盖（见 ADR-065 §13 "当前不做范围"）。

---

## 4. 高亮层设计

### 4.1 高亮类型

| 高亮类型 | 颜色 | 数据源 | 更新时机 |
|---------|------|--------|---------|
| **移动范围** (Movement) | 蓝色半透明 `(0.2, 0.4, 0.9, 0.4)` | `MapOverlayData.movement_tiles` | 选中不同单位时 |
| **AOE 预览** (AOE) | 红色半透明 `(0.9, 0.2, 0.2, 0.4)` 或 绿色 `(0.2, 0.8, 0.2, 0.4)` | `MapOverlayData.aoe_tiles` | 技能悬停/施法目标选择时 |
| **威胁范围** (Threat) | 橙红 `(0.8, 0.3, 0.1, 0.3)` | `MapOverlayData.enemy_threat_tiles` | 选中单位时（显示敌方威胁范围） |
| **路径预览** (Path) | 带方向箭头的蓝色序列 | `MapOverlayData.path_tiles` (有序) | 移动目标悬停时 |
| **悬停** (Hover) | 亮黄边框 `(1.0, 1.0, 0.5, 0.6)` | `MapOverlayData.hover_tile` | 每帧鼠标移动 |
| **选中** (Selection) | 绿白闪烁边框 | `MapOverlayData.selected_tile` | 选中单位/格子时 |

**注意**: 悬停和选中作为**光标层**设计为独立 Sprite Entity（见 §6），而非合并到高亮批处理中。这是因为光标是单个 Tile，不需要批处理。

### 4.2 高亮批处理

高亮层使用与地面层相同的 Material2d 批处理方案，但材质不同（纯色 + 透明度）：

```rust
/// 高亮覆盖层材质 — 单色半透明渲染
/// 所有高亮 Tile 使用同一个 Mesh，每帧根据 MapOverlayData 重建
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TileOverlayMaterial {
    /// 默认高亮颜色（可通过 vertex color 覆盖）
    #[uniform(0)]
    pub color: LinearRgba,
}

// 高亮层渲染 Entity
commands.spawn((
    Mesh2d(overlay_mesh_handle),
    MeshMaterial2d(overlay_material_handle),
    Transform::from_xyz(0.0, 0.0, map_z::OVERLAY),
    Visibility::default(),
    Name::new("OverlayBatch"),
));
```

**高亮 Tile 的顶点颜色编码**：

```
顶点颜色.xyz → 高亮类型编码（用于着色器区分不同类型）
顶点颜色.w   → 透明度（不同类型可能有不同透明度）

类型编码:
  (0.0, 0.0, 1.0) → MOVEMENT  → 材质颜色 = 蓝色
  (1.0, 0.0, 0.0) → AOE_HOSTILE  → 材质颜色 = 红色
  (0.0, 1.0, 0.0) → AOE_FRIENDLY → 材质颜色 = 绿色
  (1.0, 0.5, 0.0) → THREAT     → 材质颜色 = 橙色
  (0.0, 0.0, 1.0) → PATH       → 材质颜色 = 蓝色（路径+方向箭头）
```

或者更简单的方案：每个高亮类型使用独立的 Mesh + Material（功能独立，易于修改）。

**推荐**: 使用**独立 Mesh 和 Material**（每个高亮类型一个 Entity），而非顶点编码。理由：
- 高亮类型很少（5 种），独立 Entity 开销可忽略
- Shader 简单，不需要类型编码/解码
- 可以独立控制每种高亮的动画（例如 AOE 脉动、移动范围静态）
- 容易 Debug（一个 Entity 一种高亮）

```rust
/// 高亮层状态 — 所有高亮数据聚合
///
/// 被 MapRenderer 的 overlay 更新系统消费。
/// 数据由 Domain 层 → App 集成桥 → 写入此 Resource。
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct MapOverlayData {
    /// 移动范围 Tile 列表
    pub movement_tiles: Vec<GridPos>,
    /// AOE 覆盖 Tile 列表
    pub aoe_tiles: Vec<GridPos>,
    /// AOE 类型（hostile/friendly/heal）
    pub aoe_type: AoeOverlayType,
    /// 路径预览 Tile 序列（有序）
    pub path_tiles: Vec<GridPos>,
    /// 悬停位置
    pub hover_tile: Option<GridPos>,
    /// 选中位置
    pub selected_tile: Option<GridPos>,
    /// 标记：高亮数据已更新，需要重建 Mesh（Dirty 标志）
    pub dirty: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AoeOverlayType {
    Hostile,
    Friendly,
    Heal,
    Custom(Color),
}
```

### 4.3 高亮更新流程

```
每帧:
  1. Domain/Input 系统写入 MapOverlayData（设置 dirty = true）
  2. MapRenderer 的 Overlay 更新系统运行:
     a. 检查 dirty 标志
     b. dirty==true → 重建所有高亮 Mesh
     c. dirty==false → 跳过（保留上一帧的 Mesh）
  3. 渲染: Bevy 渲染管线每帧正常渲染 Mesh

高亮重建 API:

fn rebuild_overlay_meshes(
    overlay_data: Res<MapOverlayData>,
    map_renderer: Res<MapRendererState>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if !overlay_data.dirty {
        return;
    }

    // 1. 重建移动范围 Mesh
    update_movement_mesh(&overlay_data, &map_renderer, &mut meshes);
    // 2. 重建 AOE Mesh
    update_aoe_mesh(&overlay_data, &map_renderer, &mut meshes);
    // 3. 重建路径 Mesh
    update_path_mesh(&overlay_data, &map_renderer, &mut meshes);
}
```

**性能边界**: 30x30 网格下，重建所有高亮 Mesh 耗时 < 0.1ms（纯 CPU 工作，无 GPU 操作）。即使每帧重建也完全可行。

### 4.4 高亮脉冲动画

高亮区域支持简单的时间脉冲动画（闪烁/呼吸效果）。

```rust
/// 高亮动画参数（嵌入 Overlay Material）
#[uniform(0)]
pub struct OverlayAnimationParams {
    /// 当前时间（秒，每帧由 System 更新）
    pub time: f32,
    /// 脉冲幅度 (0.0 = 无脉冲, 0.2 = 20% 亮度波动)
    pub pulse_amplitude: f32,
    /// 脉冲频率 (Hz)
    pub pulse_frequency: f32,
}
```

**实现方式**: Overlay Material 的 Fragment Shader 中叠加 `abs(sin(time * frequency)) * amplitude` 到 alpha 通道。

**默认参数**:
- Move overlay: 无脉冲 (static)
- AOE overlay: 脉冲 0.3 amplitude, 2.0 Hz (准备降临的危险感)
- Hover overlay: 脉冲 0.15 amplitude, 1.5 Hz (柔和呼吸)
- Selection overlay: 脉冲 0.2 amplitude, 1.0 Hz (稳态脉冲)

### 4.5 高亮纹理选择

V1 中使用纯色半透明四边形实现高亮。V2 可以引入纹理化高亮：
- 移动范围：点状图案（dots pattern）
- AOE 范围：斜线/网格图案
- 路径预览：箭头图案沿路径方向

纹理化高亮需要额外的 Pattern 纹理 + Shader 支持。

---

## 5. 坐标系统

### 5.1 GridPos → World Pos

MapRenderer 与 Tactical Domain 共享同一坐标转换函数 `GridMap::grid_to_world()`。

```rust
// MapRenderer 使用 Domain 的坐标转换（通过 GridMap Resource）
// 确保 Tile 渲染位置 = Domain 逻辑位置的一致性
fn tile_sprite_position(grid_map: &GridMap, pos: GridPos) -> (f32, f32) {
    grid_map.grid_to_world(pos)
}

// 测试战斗当前使用的手工计算方式（将被 GridMap.grid_to_world() 替代）:
// let x = pos.x as f32 * CELL_SIZE + CELL_SIZE / 2.0;
// let y = pos.y as f32 * CELL_SIZE + CELL_SIZE / 2.0;
```

**关键保证**: MapRenderer 和 Domain 系统使用**相同的 GridMap 实例**读取 `grid_to_world()`。坐标一致性由共享 GridMap Resource 保证。

### 5.2 World Pos → Screen Pos

由 CameraQuery 提供（参见 `docs/06-ui/04-data-flow/camera-ui-interaction.md` §2.1）：

```rust
fn screen_to_world_grid(
    screen_pos: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    window: &Window,
    grid_map: &GridMap,
) -> Option<GridPos> {
    // 1. Screen → World (CameraQuery)
    let world_pos = CameraQuery::screen_to_world(screen_pos, camera, camera_transform, window)?;
    // 2. World → Grid (GridMap)
    grid_map.world_to_grid(world_pos.x, world_pos.y)
}
```

### 5.3 Tile 对齐偏移

当使用 `grid_to_world()` 返回值时，Tile Sprite 位置是**格子的中心坐标**。在渲染时需要根据 Tile 尺寸计算顶点偏移：

```rust
// 假设 tile_size = 80.0 (像素)
// grid_to_world 返回 (pos.x * 80.0, pos.y * 80.0) 即格子的左下角中心
//
// 四边形顶点偏移:
// 左下: (cx - 40.0, cy - 40.0)   ← cx, cy = grid_to_world 返回的中心坐标
// 右下: (cx + 40.0, cy - 40.0)
// 右上: (cx + 40.0, cy + 40.0)
// 左上: (cx - 40.0, cy + 40.0)
//
// 注意: grid_to_world 返回的中心坐标的计算方式由 GridLayout 决定。
//       Square 格: (x * tile_size + tile_size/2, y * tile_size + tile_size/2)
//       Hex 格: 有奇偶列偏移
```

**Tile 尺寸配置**: Tile 的世界空间尺寸由 `GridMap` 的 `grid_to_world()` 函数间接定义。如果使用 1.0 单位的 grid_to_world（当前实现），Tile 的世界尺寸为 `(1.0, 1.0)`，像素尺寸由 Camera 的缩放决定。

**推荐**: 保留 `grid_to_world()` 的返回值作为 Tile 中心，Tile 渲染的偏移量由 MapRenderer 根据 `TILE_WORLD_SIZE` 常量计算：

```rust
/// Tile 的世界空间尺寸（与 grid_to_world 的间距系数一致）
/// Square 格: grid_to_world 步进 = TILE_WORLD_SIZE = 1.0
pub const TILE_WORLD_SIZE: f32 = 1.0;
```

这样 MapRenderer 不依赖硬编码像素尺寸，与 Camera 缩放解耦。

---

## 6. 光标与悬停

### 6.1 光标层独立设计

光标（悬停高亮、选中高亮）作为独立 Sprite Entity 实现，而非合入高亮批处理：

```rust
/// 光标 Sprite — 每帧根据鼠标位置更新
pub fn update_cursor(
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window>,
    grid_map: Res<GridMap>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    overlay_data: Res<MapOverlayData>,
    mut cursor_query: Query<(&mut Transform, &mut Visibility), With<MapCursor>>,
) {
    // 获取鼠标屏幕位置 → 计算 GridPos
    // 更新 Cursor Transform
}
```

**光标 Entity**:
```rust
commands.spawn((
    Sprite {
        image: cursor_texture.clone(),
        color: Color::srgba(1.0, 1.0, 0.5, 0.6),
        custom_size: Some(Vec2::splat(TILE_WORLD_SIZE)),
        ..default()
    },
    Transform::from_xyz(0.0, 0.0, map_z::CURSOR),
    Visibility::Hidden,  // 初始隐藏，鼠标进入地图区域后显示
    MapCursor,
    Name::new("MapCursor"),
));
```

**为什么光标独立于高亮批处理**：
- 光标频繁更新（每帧），批处理 Mesh 重建开销虽小但仍非必要
- 光标需添加额外视觉效果（边框、脉冲动画），独立 Entity 更灵活
- 光标只需要一个 Tile，不值得走批处理

### 6.2 选中指示器

选中指示器与悬停光标类似，但持续显示在被选中的 Tile/单位位置：

```rust
/// 选中指示器 Entity — 在单位选中/格子选中时可见
pub struct SelectionIndicator;

commands.spawn((
    Sprite {
        image: selection_ring_texture.clone(),  // 环形边框纹理
        color: Color::srgba(0.2, 0.9, 0.2, 0.8),
        custom_size: Some(Vec2::splat(TILE_WORLD_SIZE * 1.2)),
        ..default()
    },
    Transform::from_xyz(0.0, 0.0, map_z::CURSOR + 0.1),
    Visibility::Hidden,
    SelectionIndicator,
    Name::new("SelectionIndicator"),
));
```

---

## 7. 单位渲染模式

### 7.1 架构原则

**单位不在 MapRenderer 内部渲染**。每个单位是独立的 ECS Entity，持有自己的 `Sprite` 和 `Transform` 组件。

```rust
// 单位 Entity 在 SpawnUnit 时创建:
commands.spawn((
    // Domain 组件
    UnitIdComponent { id: unit_id },
    GridPos { x: spawn_pos.x, y: spawn_pos.y },
    CombatParticipant { team_id: ... },
    HitPoints { current: max_hp, max: max_hp },
    // ... 其他 Domain 组件

    // 渲染组件（由 UnitVisualSystem 添加，或直接在 SpawnUnit 中插入）
    Sprite {
        image: unit_texture_handle,
        color: team_color,
        custom_size: Some(Vec2::splat(0.8)),     // Tile 尺寸的 80%
        ..default()
    },
    Transform::from_xyz(world_x, world_y, map_z::UNIT),
    Visibility::default(),
));
```

### 7.2 单位位置同步

单位的世界坐标由以下系统维护：

```rust
/// 同步: GridPos → Transform（每帧运行）
/// 确保单位 Sprite 位置始终与 Domain 逻辑位置一致
pub fn sync_unit_position(
    grid_map: Res<GridMap>,
    mut unit_query: Query<(&GridPos, &mut Transform), (With<UnitMarker>, Without<MapCursor>)>,
) {
    for (grid_pos, mut transform) in &mut unit_query {
        let (wx, wy) = grid_map.grid_to_world(*grid_pos);
        transform.translation.x = wx;
        transform.translation.y = wy;
    }
}
```

**注意**: `sync_unit_position` 系统属于 `src/infra/map/systems/`（或 app 层的编排系统），不属于 MapRenderer，也不属于 Domain。它是一个**跨层集成系统**——读取 Domain 的 GridPos，写入 Infra 的 Transform。

### 7.3 单位 z-ordering

对于 2D 俯视角游戏，单位的 z-order 遵循**从下到上 = 从后到前**的原则：

```rust
/// 单位排序 — 根据 y 坐标决定渲染顺序（越靠下越靠前）
/// 例如: y 大的单位渲染在 y 小的单位之上
pub fn sort_unit_z_order(
    mut unit_query: Query<&mut Transform, With<UnitMarker>>,
) {
    // 对同一 z 层的单位，根据 y 坐标微调 z
    for mut transform in &mut unit_query {
        // base_z = map_z::UNIT (3.0)
        // 微调量 = y / MAX_GRID_SIZE * 0.1（保证在 UNIT z 层内排序）
        transform.translation.z = map_z::UNIT
            + transform.translation.y / 1000.0 * 0.1;
    }
}
```

这个排序在单位移动时每帧运行。1000 个单位以内的排序耗时 < 0.01ms。

---

## 8. 与 Camera 的关系

### 8.1 依赖方向

```
MapRenderer → CameraQuery (读取坐标转换)
Camera      → 不知晓 MapRenderer 的存在
```

MapRenderer 使用 CameraQuery 读取 Camera 状态，用于：
- **可见性判断**: 仅渲染 Camera 可视区域内的 Tile（可选优化，V1 不做）
- **光标定位**: 无 Camera 则无法将屏幕鼠标位置映射为 GridPos
- **高亮脉冲对齐**: 脉冲动画可在世界空间或屏幕空间进行

### 8.2 可见性裁剪（V2 优化）

对于大网格（> 50x50），MapRenderer 可以裁剪 Camera 可视区域外的 Tile：

```rust
/// V2 优化: 仅构建可视 Tile 的批处理 Mesh
///
/// 1. CameraQuery::visible_rect() → visible_world_rect (Rect)
/// 2. 计算 visible_world_rect 覆盖的网格范围
/// 3. 只生成该范围内的 Tile Mesh，范围外的 Tile 不占顶点
///
/// 注意: 只在 Camera 移动结束（非插帧状态）时重建 Mesh
///       移动中保留上一帧的 Mesh（视觉可接受）
```

**V1 不实现可见性裁剪**。30x30 网格（900 Tile）全部渲染，顶点数约 3600（900*4），GPU 负载极低。Camera 移出地图边界时，Bevy 的 Frustum Culling 自动裁切。

### 8.3 Camera 移动时的行为

Camera 移动不影响 MapRenderer 的工作方式：

- **Terrain Batch**: 不移动，Camera 变换自动产生滚动效果（世界空间渲染）
- **Overlay Batch**: 不移动，自动跟随 Camera
- **Cursor**: 屏幕位置转换成世界位置再转换成网格位置，Cursor Entity 放置在世界坐标
- **Unit**: 不移动，Camera 变换自动产生单位"跟随"效果

**关键**: MapRenderer 中的所有 Entity 都是**世界空间**的。Camera 的移动和缩放通过 Transform/Projection 矩阵影响最终屏幕输出。

---

## 9. 与不渲染成 Sprite 的 Tile 的交互

### 9.1 Tile 点击检测

用户点击地图 Tile 的流程：

```
1. 用户点击屏幕位置 (screen_x, screen_y)
2. CameraQuery::screen_to_world(screen_x, screen_y) → world_pos
3. GridMap::world_to_grid(world_pos.x, world_pos.y) → Option<GridPos>
4. 如果 Some(grid_pos) → 发射 UiAction::TileClicked(grid_pos)
5. BattleScreen 接收 → 决定是移动单位/选中单位/施放技能
```

**不依赖碰撞检测**: 不使用 RayCast / AABB 碰撞进行 Tile 点击检测。因为：
- 性能更好（纯数学计算，无场景遍历）
- 精确（不受 Sprite 尺寸/旋转影响）
- 与 Domain 坐标一致（不因渲染偏移导致点击偏差）

### 9.2 悬停检测

与 Tile 点击相同流程，但不发射 UiAction，而是更新 `MapOverlayData.hover_tile`：

```rust
fn update_hover_tile(
    camera: Query<(&Camera, &GlobalTransform)>,
    window: Query<&Window>,
    grid_map: Res<GridMap>,
    cursor_position: Res<CursorPosition>,  // UI 层提供的当前鼠标屏幕位置
    mut overlay_data: ResMut<MapOverlayData>,
) {
    let Ok((cam, cam_transform)) = camera.single() else { return };
    let Ok(window) = window.single() else { return };

    let world_pos = CameraQuery::screen_to_world(
        cursor_position.0, cam, cam_transform, &window
    );

    let hover_pos = grid_map.world_to_grid(world_pos.x, world_pos.y);

    if overlay_data.hover_tile != hover_pos {
        overlay_data.hover_tile = hover_pos;
        overlay_data.dirty = true;
    }
}
```

---

## 10. 性能分析与边界

### 10.1 30x30 网格 (900 Tile) 性能估算

| 指标 | 估算值 | 说明 |
|------|--------|------|
| 地面层顶点数 | 3600 (900*4) | 四边形顶点，共享索引约 5400 |
| 地面层 Draw calls | 1 | 批处理 Material2d |
| 高亮层顶点数 | ~2000 (500 个高亮 Tile * 4) | 非全图高亮 |
| 高亮层 Draw calls | 5 (每种高亮类型 1 个) | 独立 Material/Mesh |
| 单位 Entity 数 | ~20-50 | SRPG 典型单位数量 |
| Cursor Entity 数 | 2 (hover + selection) | 可忽略 |
| 总 Entity 数 | ~30-60 (不含批处理内部) | 不包括 MapRenderer 管理的批处理 Mesh Entity |
| 总 Draw calls | ~15 | 地面 + 高亮 *5 + 单位 + 光标 + UI |
| 每帧 CPU 时间 (渲染) | < 0.5ms | 纯 CPU 侧 |
| GPU 负载 | < 1% | 现代 GPU 无压力 |

### 10.2 100x100 网格 (10000 Tile)

| 指标 | 估算值 | 说明 |
|------|--------|------|
| 地面层顶点数 | 40000 | 仍远低于 GPU 上限 |
| 总 Draw calls | ~15 | 不变（批处理保持） |
| 每帧 CPU 时间 | ~1ms | Mesh 更新略有增加 |
| 是否需要可见性裁剪 | 可选 | 不裁剪也可运行 |

**结论**: 30x30 网格下无任何性能瓶颈。100x100 下仍可流畅运行。批处理架构保证网格尺寸增大时 Draw calls 不增加。

### 10.3 性能调试工具

```rust
// dev 模式下注册的性能计数器
#[derive(Resource, Default, Reflect)]
pub struct MapRendererMetrics {
    pub terrain_tile_count: u32,
    pub overlay_tile_count: u32,
    pub terrain_mesh_rebuilds: u64,
    pub overlay_mesh_rebuilds: u64,
    pub unit_count: u32,
    pub cursor_position: Option<GridPos>,
}
```

仅在 `--features dev` 模式下收集，release 中零开销。

---

## 11. 场景生命周期

### 11.1 MapRenderer 生命周期绑定

MapRenderer 绑定到 `GameState::Combat` 的生命周期：

```
OnEnter(GameState::Combat):
  1. MapLoader: MapAsset → GridMap
  2. ObjectInstantiator: MapObject → ECS Entity
  3. build_terrain_batch: GridMap → Terrain Mesh (TerrainBatch Entity)
  4. build_overlay_entities: → 5x Overlay Entity (初始为空，dirty=true)
  5. spawn_cursor_entity: → Cursor Sprite (hidden)
  6. spawn_selection_indicator: → Selection Sprite (hidden)
  7. init TerrainTextureMap, MapRendererState, MapOverlayData

Update (每帧):
  1. sync_unit_position: GridPos → Transform
  2. update_hover_tile: Cursor → MapOverlayData
  3. rebuild_overlay_meshes: MapOverlayData → 重建高亮 Mesh
  4. update_cursor: MapOverlayData.hover → Cursor Transform

OnExit(GameState::Combat):
  cleanup_map_system:
  1. Despawn TerrainBatch Entity
  2. Despawn x5 Overlay Entity
  3. Despawn Cursor Entity
  4. Despawn Selection Entity
  5. 移除 TerrainTextureMap, MapRendererState, MapOverlayData Resource
```

### 11.2 生命周期代码

```rust
// MapRendererPlugin — 注册生命周期系统和 Resource
impl Plugin for MapRendererPlugin {
    fn build(&self, app: &mut App) {
        app
            // 注册 Resource
            .init_resource::<MapOverlayData>()
            .init_resource::<MapRendererMetrics>()

            // 生命周期系统
            .add_systems(OnEnter(GameState::Combat), (
                build_terrain_batch,
                build_overlay_entities,
                spawn_cursor_entity,
                init_renderer_state,
            ).chain())

            // 每帧更新系统
            .add_systems(Update, (
                update_hover_tile,
                rebuild_overlay_meshes,
                update_cursor,
                update_selection_indicator,
            ).run_if(in_state(GameState::Combat)))

            // 清理
            .add_systems(OnExit(GameState::Combat), cleanup_map_renderer);
    }
}
```

**重要**: `cleanup_map_renderer` 不负责清理 GridMap Resource 或 Domain 数据。GridMap 的清理由 MapLoader 的对应清理系统处理。MapRenderer 只清理自己创建的 Entity 和 Resource。

### 11.3 与 MapLoader 的协作

```
OnEnter(GameState::Combat):
  Phase 1: MapLoader (MapPlugin)
    1. Load MapAsset → GridMap Resource
    2. Instance objects (class→Entity)
    3. Set up NavigationMask

  Phase 2: MapRenderer (MapRendererPlugin)
    1. Read GridMap → build Terrain Batch
    2. Init overlay entities
    3. Init cursor
```

**Phase 顺序**通过 SystemSet 或显式 `.after()` 保证：

```rust
.add_systems(OnEnter(GameState::Combat), build_terrain_batch
    .after(map_loader_system)  // MapLoader 先产生 GridMap
)
```

---

## 12. 数据流全景

```
Domain Layer (L1 Core)
─────────────────────
  Tactical Domain
    GridMap (Resource)
      ├── grid_to_world()     → 被 MapRenderer 读取
      ├── tiles_in_range()    → 被 MovementSystem 使用
      │
    MovementRange (Resource/Component)
      └── Vec<GridPos>        → 移动范围计算结果

  Combat Domain
    AOESet (Resource/Component)
      └── Vec<GridPos> + Type → AOE 范围计算结果


App / Integration Layer
─────────────────────
  Bridge System (app-level or infra/map/systems)
    └── 读取 MovementRange + AOESet
    └── 写入 MapOverlayData (infra/map/renderer Resource)


Infra Layer (L2)
─────────────────────
  infra/camera
    CameraQuery             → 被 MapRenderer 读取

  infra/map/renderer
    MapRendererState (Resource)         → 管理渲染 Entity 引用
    MapOverlayData (Resource)           → 高亮数据（每帧更新）
    TerrainTextureMap (Resource)        → terrain_id → 纹理映射
    TerrainBatch (Entity)               → 地面层 Mesh2d
    OverlayBatch x5 (Entity)            → 高亮层 Mesh2d × 5
    MapCursor (Component)               → 光标 Sprite
    SelectionIndicator (Component)      → 选中 Sprite


UI Layer (L3 Presentation)
─────────────────────
  BattleScreen
    UiAction::TileClicked(GridPos)      → Tile 点击 → 单位移动/选中
    UiAction::UnitHovered(CharacterId)  → 单位悬停 → 面板/提示显示
```

---

## 13. 文件结构

```
src/infra/map/
├── mod.rs
├── plugin.rs              ← MapPlugin（注册 Asset 类型 + MapLoader）
├── asset.rs               ← MapAsset 类型定义
├── types.rs               ← MapObjectGuid, PropertyMap 等辅助类型
├── loader.rs              ← MapAsset → GridMap + ECS State
├── events.rs              ← MapLoaded, MapUnloaded 事件
├── systems/
│   ├── mod.rs
│   ├── map_loader_system.rs     ← OnEnter: 加载地图
│   ├── map_cleanup_system.rs    ← OnExit: 清除地图
│   └── object_instantiator.rs   ← MapObject → ECS Entity
├── renderer/                    ← ★ MapRenderer (本文档)
│   ├── mod.rs
│   ├── plugin.rs                ← MapRendererPlugin
│   ├── state.rs                 ← MapRendererState Resource
│   ├── terrain_batch.rs         ← 地面 Tile 批处理构建
│   ├── overlay.rs               ← 高亮层（OverlayData + 批处理建立）
│   ├── overlay_data.rs          ← MapOverlayData Resource 定义
│   ├── cursor.rs                ← 光标 + 选中指示器
│   ├── texture_map.rs           ← TerrainTextureMap Resource 构建
│   ├── materials/
│   │   ├── mod.rs
│   │   ├── terrain_material.rs  ← TileTerrainMaterial
│   │   └── overlay_material.rs  ← TileOverlayMaterial + Animation
│   ├── systems/
│   │   ├── mod.rs
│   │   ├── build_terrain.rs     ← OnEnter: 地形批处理构建
│   │   ├── build_overlay.rs     ← OnEnter: 高亮层初始化
│   │   ├── rebuild_overlay.rs   ← Update: 高亮 Mesh 更新
│   │   ├── update_cursor.rs     ← Update: 光标位置
│   │   ├── update_hover.rs      ← Update: 悬停检测
│   │   └── cleanup.rs           ← OnExit: 清理
│   └── metrics.rs               ← 调试性能计数器（dev only）
└── tests/
    ├── mod.rs
    ├── unit/
    └── integration/
```

---

## 14. 与 bevy_ecs_tilemap 的对比

| 维度 | bevy_ecs_tilemap | 自研 MapRenderer |
|------|-----------------|------------------|
| 实体数量 | 每个 Tile 一个 Entity (~900) | 每个 Layer 一个 Mesh (~7) |
| 批处理 | 自动/按 Chunk 合批 | 显式 Material2d 批处理 |
| 自定义 Shader | 需要了解 bevy_ecs_tilemap 的渲染管线 | 直接 Material2d API |
| 高亮覆盖层 | 需要额外 Tile 层或覆盖层 | 原生支持 |
| 坐标系统 | 独立 tile ID → world 映射 | 与 GridMap.grid_to_world() 一致 |
| 升级风险 | 版本 API 不稳定 | 完全可控 |
| 编译时间 | +~10s (tiled crate) | 0 额外（使用 Bevy 内置） |
| 复杂度 | 学习第三方 API | 学习 Bevy Material2d（官方） |
| 编辑器集成 | 有 Tiled 解析 | Importer 独立工具 |

**自研方案的真正优势**不是性能（bevy_ecs_tilemap 的批处理也很高效），而是：

1. **坐标系统一**: 直接使用 Domain 的 GridMap.grid_to_world()，不存在两套坐标
2. **高亮层原生**: 不需要 hack bevy_ecs_tilemap 的层系统来实现半透明覆盖
3. **不依赖第三方**: Importer 处理编辑器格式，运行时只消费 MapAsset，编译器解耦
4. **维护可控**: 渲染行为完全由项目控制，不会被第三方库的 breaking change 影响
5. **与 Camera-UI 一致**: CameraQuery API 已在项目中定义，MapRenderer 是自然扩展

---

## 15. 架构规则

### 15.1 强制规则

| # | 规则 | 说明 |
|---|------|------|
| MAP-REND-01 | MapRenderer 不查询 Domain 组件 | 只读 `MapOverlayData`、`GridMap`（Resource）、`TerrainTextureMap` |
| MAP-REND-02 | MapRenderer 不包含业务逻辑 | 不计算移动范围、不决定高亮内容 |
| MAP-REND-03 | Camera 不依赖 MapRenderer | Camera 是通用基础设施，不知晓地图渲染 |
| MAP-REND-04 | 单位不在 MapRenderer 内部渲染 | 单位是独立 Entity，使用自己的 Sprite/Transform |
| MAP-REND-05 | Overlay 数据由 Domain 计算，MapRenderer 只消费 | App 桥接系统负责 Domain → Infra 的数据转换 |
| MAP-REND-06 | z-order 值统一管理 | 使用 `map_z` 模块常量，禁止硬编码 |
| MAP-REND-07 | Tile 坐标引用 GridMap API | 禁止手算坐标偏移，必须通过 `grid_to_world()` |
| MAP-REND-08 | 高亮 Mesh 使用 dirty flag 更新 | 非每帧重建，仅在 `dirty==true` 时重建 |
| MAP-REND-09 | MapRenderer Plugin 注册在 `OnEnter`/`OnExit` | 绑定场景生命周期，不持久保留 |
| MAP-REND-10 | Terrain Batch 构建必须在 MapLoader 之后 | 通过 `.after(map_loader_system)` 显式保证 |

### 15.2 禁止事项

| # | 禁止 | 理由 | 后果 |
|---|------|------|------|
| 🟥 | MapRenderer 中直接 `Query<&GridPos, With<CombatParticipant>>` | 表现层耦合领域组件 | CI 审查不通过 |
| 🟥 | Unit Entity 由 MapRenderer 管理 | 单位有独立生命周期 | 架构评审不通过 |
| 🟥 | Overlay 数据含业务计算 | 表现层不应计算领域数据 | 架构评审不通过 |
| 🟥 | 硬编码 Tile 渲染尺寸 | 必须与 GridMap.grid_to_world() 一致 | 代码审查不通过 |
| 🟥 | 运行时动态创建/销毁 TerrainBatch（除 OnExit） | 滥用 Entity 操作 | 代码审查不通过 |
| 🟥 | 使用 `bevy_ecs_tilemap` 依赖 | ADR-065 自研决策 | 架构评审不通过 |
| 🟥 | 在 MapRenderer 中使用 EventWriter/EventReader | 必须使用 trigger() + Observer | 编译期警告 (Bevy 0.19) |

---

## 16. 实现路径

| 阶段 | 功能 | 依赖 |
|------|------|------|
| **V1 (当前 Sprint)** | Entity-per-Tile 替代方案：在 MapAsset 加载时，为每个 Tile 生成一个 Sprite Entity。共享纹理，通过 Sprite.color 区分地形类型。与 test_battle 现有模式兼容。 | MapAsset 类型定义 + MapLoader |
| **V2 (后续 Sprint)** | Material2d 批处理：实现 TerrainBatch、TerrainTextureMap、TileTerrainMaterial。用批处理替代 Entity-per-Tile。 | V1 的地面层渲染 |
| **V3** | 高亮层：MapOverlayData + 5 种高亮类型的 Mesh 批处理。悬停/选择光标独立 Entity。 | V2 的批处理框架 |
| **V4** | 脉冲动画 + Path overlay 箭头 + 调试工具 | V3 的高亮层框架 |

**V1 快速方案（Entity-per-Tile）**:

```rust
// 简单的 Entity-per-Tile 实现
// 每个 Tile 一个 Sprite Entity，共享 TextureAtlas
fn spawn_tile_entities(
    grid_map: Res<GridMap>,
    texture_map: Res<TerrainTextureMap>,
    mut commands: Commands,
) {
    for y in 0..grid_map.height {
        for x in 0..grid_map.width {
            let pos = GridPos::new(x as i32, y as i32);
            let tile = grid_map.get_tile(pos).unwrap();
            let (wx, wy) = grid_map.grid_to_world(pos);
            let atlas_idx = texture_map.terrain_to_atlas.get(&tile.terrain_def_id());

            commands.spawn((
                Sprite::from_atlas_image(
                    texture_map.texture.clone(),
                    TextureAtlas {
                        layout: texture_map.atlas_layout.clone(),
                        index: *atlas_idx.unwrap_or(&0),
                    },
                ),
                Transform::from_xyz(wx, wy, map_z::TERRAIN),
                Visibility::default(),
                TileEntity,  // Marker 组件，OnExit 时统一清除
                Name::new(format!("Tile_{}_{}", x, y)),
            ));
        }
    }
}
```

**注意**: V1 实体模式下，OnExit 清理使用 `commands.entity(query).despawn()` 配合 `TileEntity` Marker。V2 批处理模式下，只需要 despawn 一个 TerrainBatch Entity。

---

## 17. 测试策略

### 17.1 单元测试（Unit）

| 测试 | 内容 | 位置 |
|------|------|------|
| `terrain_texture_map_build` | 验证 TerrinTextureMap 从 ImporterAtlasInfo 正确构建 | `renderer/tests/unit/` |
| `grid_to_world_consistency` | 验证 MapRenderer 使用的 grid_to_world 与 Domain 一致 | `renderer/tests/unit/` |
| `overlay_data_dirty_flag` | 验证 dirty flag 的 set/clear 行为 | `renderer/tests/unit/` |
| `cursor_position_math` | 验证 world_pos → GridPos 转换的正确性 | `renderer/tests/unit/` |

### 17.2 集成测试（Integration）

| 测试 | 内容 |
|------|------|
| `map_renderer_lifecycle` | OnEnter → verify TerrainBatch exists → OnExit → verify cleanup |
| `overlay_update_flow` | write MapOverlayData → rebuild → verify Mesh updated |

### 17.3 快照测试（Snapshot）

- TerrainBatch 的初始状态（Tile 数量、位置一致性）
- 各高亮类型的颜色和位置

### 17.4 不测试（当前范围外）

- 具体材质渲染结果（像素比对，需要 screenshot 测试框架）
- 性能基准测试（独立 bench suite）

---

## 18. 决策记录

| 决策 | 选择 | 替代方案 | 理由 |
|------|------|---------|------|
| 渲染层位置 | `src/infra/map/renderer/` | `src/ui/map/` | MapRenderer 是基础设施渲染，非 UI 层 |
| Tile 渲染方式 | Material2d 批处理 | Entity-per-Tile | 扩展性、Draw call 数量 |
| 高亮类型实现 | 独立 Mesh/Material × 5 | 顶点颜色编码 | Shader 简单，易 Debug，可独立动画 |
| 光标层 | 独立 Sprite Entity | 合入高亮批处理 | 每帧更新，不值得批处理重建 |
| 坐标系统 | GridMap.grid_to_world() | 独立计算 | 与 Domain 保持一致 |
| Overlay 数据流 | Domain → App Bridge → MapOverlayData | Domain → MapRenderer 直写 | 保持依赖方向：Domain 不依赖 Infra |
| 可见性裁剪 | V1 不做，V2 可选优化 | V1 就做 | 30x30 不需要 |
| 单位渲染 | 单位 Entity 自持 Sprite | MapRenderer 统一渲染 | 单位生命周期属于 Domain |
| z-order 管理 | `map_z` 模块常量 | 运行时排序 | 编译期确定，零运行时开销 |
| V1 快速实现 | Entity-per-Tile + TextureAtlas | 直接上 Material2d | 降低初始工作量，快速替换 test_battle |

---

*本文档由 @presentation-architect 维护。MapRenderer 架构的变更需要 Presentation Architect 审查。*
