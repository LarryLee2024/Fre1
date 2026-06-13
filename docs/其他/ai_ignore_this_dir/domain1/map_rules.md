# 地图领域规则 (Map Rules)

## 1. 领域概述

地图系统管理 SRPG 战场的地形数据、单位占位和寻路。采用 **地图优先看成 Grid 数据**原则，地形数据与渲染分离，占用网格独立存在，寻路数据运行时生成。

### 核心原则

- **地图优先看成 Grid 数据**：TerrainGrid 是地形唯一真相源
- **地图数据与渲染分离**：TerrainGrid 存数据，spawn_map 只画格子
- **OccupancyGrid 独立存在**：单位占位与地形数据分离
- **寻路数据运行时生成**：BFS 计算可达范围，不预存路径
- **数据驱动**：地形定义和关卡配置从 RON 文件加载

---

## 2. TerrainDef — 地形定义

```rust
pub struct TerrainDef {
    pub id: String,
    pub name: String,
    pub move_cost: Option<u32>,   // None=不可通行
    pub defense_bonus: i32,
    pub color: (f32, f32, f32),
    pub passable: bool,
    pub char_code: Option<char>,  // 关卡网格中的字符代码
}
```

### 2.1 内置默认地形

| ID | 名称 | move_cost | defense_bonus | passable | char_code |
|----|------|-----------|---------------|----------|-----------|
| `plain` | 草 | 1 | 0 | true | P |
| `forest` | 林 | 2 | 2 | true | F |
| `mountain` | 山 | None | 0 | false | M |
| `water` | 水 | None | 0 | false | W |

### 2.2 move_cost 规则

- `passable=true` 且 `move_cost > 0` → `Some(move_cost)`
- `passable=false` 或 `move_cost=0` → `None`（不可通行）

---

## 3. TerrainGrid — 地形网格

```rust
#[derive(Resource)]
pub struct TerrainGrid {
    pub width: u32,
    pub height: u32,
    cells: HashMap<IVec2, String>,  // (x,y) → terrain_id
}
```

| 方法 | 说明 |
|------|------|
| `from_terrain_map(w, h, map)` | 从 LevelConfig 的 terrain_map 构建 |
| `get(coord)` | 获取地形 ID |
| `set(coord, terrain_id)` | 设置地形 ID |
| `is_in_bounds(coord)` | 坐标是否在范围内 |
| `iter()` | 迭代所有格子 |
| `default_plain(w, h)` | 兜底全平地 |

**规则**：
- 地形数据的唯一真相源
- 寻路/UI/战斗都从这里读取
- 未配置的格子默认为 `"plain"`

---

## 4. OccupancyGrid — 占用网格

```rust
#[derive(Resource)]
pub struct OccupancyGrid {
    occupied: HashMap<IVec2, Entity>,  // (x,y) → Entity
}
```

| 方法 | 说明 |
|------|------|
| `set(coord, entity)` | 设置占用 |
| `remove(coord)` | 移除占用 |
| `is_occupied(coord)` | 是否被占用 |
| `get_entity(coord)` | 获取占用实体 |
| `is_occupied_except(coord, entity)` | 排除自身检查 |
| `rebuild(units)` | 从单位位置重建 |
| `occupied_coords()` | 获取占用坐标集合 |

**规则**：
- 单位占位的唯一真相源
- 每帧从单位位置 `rebuild`
- 寻路时排除自身位置

---

## 5. GameMap — 地图资源

```rust
#[derive(Resource)]
pub struct GameMap {
    pub width: u32,
    pub height: u32,
    pub tile_size: f32,
}
```

### 5.1 坐标转换

| 方法 | 说明 |
|------|------|
| `coord_to_world(coord)` | 网格坐标 → 世界坐标 |
| `world_to_coord(world)` | 世界坐标 → 网格坐标 |
| `is_in_bounds(coord)` | 坐标是否在范围内 |

### 5.2 坐标系

- 网格原点：左下角 (0, 0)
- 世界坐标：地图中心为原点
- 转换公式：`world = (coord - size/2 + 0.5) * tile_size`

---

## 6. LevelConfig — 关卡配置

```rust
pub struct LevelConfig {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub tile_size: f32,
    pub terrain_map: HashMap<(i32, i32), String>,
    pub player_units: Vec<UnitDeployDef>,
    pub enemy_units: Vec<UnitDeployDef>,
}
```

### 6.1 UnitDeployDef

```rust
pub struct UnitDeployDef {
    pub template: String,    // 单位模板 ID
    pub coord: (i32, i32),   // 部署坐标
}
```

### 6.2 地形网格解析

关卡 RON 中的 `terrain_grid` 是字符串数组，每个字符映射到地形 ID：

```
默认映射：P→plain, F→forest, M→mountain, W→water
自定义映射：LevelConfigDef.char_map 覆盖默认值
未识别字符：默认 "plain"
```

---

## 7. 寻路系统

### 7.1 TerrainCostCalculator — 地形成本计算

```rust
pub trait TerrainCostCalculator: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn cost(&self, terrain_id: &str, base_cost: Option<u32>) -> Option<u32>;
}
```

### 7.2 内置计算器

| 计算器 | name | plain | forest | mountain | water |
|--------|------|-------|--------|----------|-------|
| `GroundCostCalculator` | "ground" | 1 | 2 | × | × |
| `FlyingCostCalculator` | "flying" | 1 | 1 | 1 | 1 |
| `MountedCostCalculator` | "mounted" | 1 | 3 | × | × |
| `SwimmingCostCalculator` | "swimming" | 2 | 3 | × | 1 |

**规则**：
- Ground：使用基础成本（从 TerrainRegistry 读取）
- Flying：所有地形成本为 1
- Mounted：平原 1，森林 3，山地/水域不可通行
- Swimming：水域 1，平原 2，森林 3，山地不可通行

### 7.3 TerrainCostRegistry

```rust
#[derive(Resource)]
pub struct TerrainCostRegistry {
    calculators: HashMap<String, Box<dyn TerrainCostCalculator>>,
}
```

**标签解析优先级**：SWIMMING > FLYING > MOUNTED > 默认(ground)

### 7.4 find_reachable_tiles — BFS 可达范围

```rust
pub fn find_reachable_tiles(
    start: IVec2,
    move_points: u32,
    map: &GameMap,
    terrain_grid: &TerrainGrid,
    terrain_registry: &TerrainRegistry,
    occupancy: &OccupancyGrid,
    moving_entity: Option<Entity>,
    calculator: &dyn TerrainCostCalculator,
) -> HashMap<IVec2, u32>  // 坐标 → 剩余移动力
```

**算法**：BFS，四方向扩展

**规则**：
- 起始位置不包含在结果中
- 成本 > 剩余移动力 → 跳过
- 被占用格子（自身除外）→ 跳过
- 已访问且剩余移动力更多 → 更新

### 7.5 reconstruct_path — 路径回溯

```rust
pub fn reconstruct_path(
    start, target, reachable, move_points,
    map, terrain_grid, terrain_registry, calculator,
) -> Vec<IVec2>
```

**规则**：
- start == target → `[target]`
- target 不在 reachable 中 → `[target]`
- 从 target 向 start 回溯，选择剩余移动力最大的前驱
- 返回从 start（不含）到 target（含）的坐标序列

---

## 8. 注册表

### 8.1 TerrainRegistry

```rust
#[derive(Resource)]
pub struct TerrainRegistry {
    pub terrains: HashMap<String, TerrainDef>,
}
```

- 加载目录：`assets/terrains/`
- `char_map()`：从已注册地形构建字符→ID映射

### 8.2 LevelRegistry

```rust
#[derive(Resource)]
pub struct LevelRegistry {
    pub levels: HashMap<String, LevelConfig>,
}
```

- 加载目录：`assets/maps/`
- `first()`：获取第一个关卡（默认关卡）
- 无硬编码兜底

---

## 9. 地图渲染

`spawn_map` 系统：
1. 从 LevelRegistry 获取关卡配置
2. 构建 TerrainGrid
3. 遍历 TerrainGrid 生成 Sprite（TileSprite 组件）
4. 每个格子显示坐标、地形名、移动成本

**规则**：
- 渲染层与数据层分离
- 不生成 Tile Entity，只生成纯渲染 Sprite
- 地形属性从 TerrainRegistry 读取

---

## 10. RON 配置格式

### 10.1 地形定义

```ron
(
    id: "forest",
    name: "林",
    move_cost: 2,
    defense_bonus: 2,
    color: (0.20, 0.50, 0.18),
    passable: true,
    char_code: Some('F'),
)
```

### 10.2 关卡配置

```ron
(
    id: "tutorial",
    name: "教学关",
    width: 5,
    height: 4,
    terrain_grid: [
        "MMMMM",
        "MPPPM",
        "MPFPM",
        "MMMMM",
    ],
    player_units: [
        (template: "player_warrior", coord: (2, 2)),
    ],
    enemy_units: [
        (template: "enemy_goblin", coord: (3, 2)),
    ],
)
```

---

## 11. 关键约束

1. **TerrainGrid 是地形唯一真相源**：不使用 Tile Entity
2. **OccupancyGrid 是占位唯一真相源**：替代临时 HashMap
3. **地形数据与渲染分离**：spawn_map 只画格子，不存数据
4. **寻路数据运行时生成**：BFS 计算可达范围，不预存路径
5. **成本计算器可扩展**：新增单位类型只需实现 TerrainCostCalculator
6. **标签解析优先级**：SWIMMING > FLYING > MOUNTED > ground
7. **占用排除自身**：寻路时自身位置不算被占用
8. **关卡无硬编码兜底**：LevelRegistry 空即为空
9. **地形有硬编码兜底**：TerrainRegistry 默认注册四种地形
10. **未配置格子默认 plain**：TerrainGrid 和 LevelConfig 的兜底
