---
id: 01-architecture.ADR-022
title: ADR-022 — Grid / Terrain / Faction System Design
status: approved
owner: architect
created: 2026-06-16
updated: 2026-06-16
supersedes: none
---

# ADR-022: 网格 / 地形 / 阵营系统设计

## 状态

**Approved** — 依赖 ADR-000（Feature Module Map），本架构决策正式生效。

## 背景

网格地图、地形和阵营是 SRPG 的物理基础。网格决定"单位在哪"，地形决定"这个格子有什么效果"，阵营决定"谁和谁战斗"。这三个 Feature 属于 Layer 1（Tactical Foundation），不依赖任何游戏玩法逻辑。

## 引用的领域规则与数据架构

- `docs/02-domain/domains/tactical_domain.md` — Tactical 领域规则
- `docs/02-domain/domains/terrain_domain.md` — Terrain 领域规则
- `docs/02-domain/domains/faction_domain.md` — Faction 领域规则
- `docs/04-data/domains/tactical_schema.md` — Tactical Schema
- `docs/04-data/domains/terrain_schema.md` — Terrain Schema
- `docs/04-data/domains/faction_schema.md` — Faction Schema
- `.trae/rules/SRPG专项规则.md` §三 — 地图系统规范

## 决策

### 1. Grid Map 架构

#### 1.1 网格数据模型

```rust
/// GridMap — 全局网格数据
/// 存储为 Resource，非 Entity 集合（网格数量大，不宜每个 Tile 一个 Entity）
#[derive(Resource)]
pub struct GridMap {
    /// 网格尺寸
    pub width: u32,
    pub height: u32,
    /// 平铺 Tile 数据（每 Tile 4 字节）
    pub tiles: Vec<TileData>,
    /// 坐标系统
    pub layout: GridLayout,
}

/// 每 Tile 数据 — 紧凑存储
#[derive(Clone, Copy)]
pub struct TileData {
    pub terrain_def_id: TerrainDefId,   // 地形 ID
    pub height: u8,                      // 高度（0-255）
    pub flags: TileFlags,                // 位标记：可通过/可飞行/可建造
}

/// 坐标系统
pub enum GridLayout {
    Square,       // 四向网格（简单）
    HexRowOdd,    // 六边形，奇数列偏移
    HexRowEven,   // 六边形，偶数列偏移
    HexColOdd,    // 六边形，奇数行偏移
}
```

#### 1.2 网格查询 API

```rust
impl GridMap {
    /// 获取 Tile
    pub fn get_tile(&self, pos: GridPos) -> Option<&TileData>;
    pub fn get_tile_mut(&mut self, pos: GridPos) -> Option<&mut TileData>;

    /// 邻接 Tile
    pub fn neighbors(&self, pos: GridPos) -> Vec<GridPos>;

    /// 范围内的 Tile
    pub fn tiles_in_range(&self, center: GridPos, range: u32) -> Vec<GridPos>;

    /// 路径查找（A*）
    pub fn find_path(
        &self,
        from: GridPos,
        to: GridPos,
        unit: Entity,
        unit_data: &UnitMovementData,
    ) -> Option<Vec<GridPos>>;

    /// 坐标转换
    pub fn world_to_grid(&self, world_pos: Vec2) -> Option<GridPos>;
    pub fn grid_to_world(&self, grid_pos: GridPos) -> Vec2;
}
```

#### 1.3 Tile 实体化决策

> 🟩 Tile **默认不实例化为 Entity**，存储在 `GridMap` Resource 的连续数组中。

**实体化时机**（满足任一条件）：
- Tile 上存在单位
- Tile 被标记为交互点（宝箱、传送门、对话触发）
- Tile 的状态在运行时频繁变化（如"燃烧的地板"）

```rust
/// Tile 实体化标记 — 仅当 Tile 需要实体化时添加
#[derive(Component)]
pub struct TileMarker {
    pub grid_pos: GridPos,
}
```

#### 1.4 GridPos 与坐标系统

```rust
/// 网格坐标 — Layer 1 基础类型
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    /// 六边形距离
    pub fn hex_distance(self, other: GridPos) -> u32;
    /// 方形网格距离（曼哈顿）
    pub fn manhattan_distance(self, other: GridPos) -> u32;
    /// 方形网格距离（切比雪夫）
    pub fn chebyshev_distance(self, other: GridPos) -> u32;
}
```

### 2. Terrain 架构

#### 2.1 地形定义

```rust
/// TerrainDef — 配置文件加载
#[derive(Asset, TypePath)]
pub struct TerrainDef {
    pub id: TerrainDefId,
    pub name: LocalizationKey,
    pub movement_cost: MovementCost,     // 移动消耗倍率
    pub defense_bonus: f32,              // 防御加成
    pub avoid_bonus: f32,                // 闪避加成
    pub flags: TerrainFlags,             // BlockSight, Impassable, Flyable 等
    pub visual: TerrainVisual,           // 渲染信息
    pub tags: Vec<TagId>,                // 可被 Tag 系统引用
}
```

#### 2.2 地形效果系统

地形对 gameplay 的影响通过 **Tag 机制**桥接到 Layer 2 的能力系统：

```
TerrainDef.tags: [Tag("rough_terrain"), Tag("low_elevation")]
       │
       ▼
Layer 2 (Capability System)
  Tag("rough_terrain")
    ├── Modifier: MovementCost × 2.0
    └── Modifier: Avoid -10
  Tag("low_elevation")
    └── Condition: CanTrigger("ambush") = false
```

> 🟩 地形本身不包含业务逻辑（Data Law 002），它只提供 Tag。Tag 到数值/行为的映射由 Layer 2 的 Modifier/Condition 处理。

#### 2.3 地形高度

```rust
/// 高度系统 — 影响视距和攻击范围
pub struct HeightSystem;

impl HeightSystem {
    /// 两个 GridPos 之间的高度差
    pub fn height_difference(grid: &GridMap, a: GridPos, b: GridPos) -> i8;

    /// 是否有视距（Line of Sight）
    pub fn has_line_of_sight(grid: &GridMap, from: GridPos, to: GridPos) -> bool;

    /// 攻击是否受高度影响
    pub fn height_attack_bonus(grid: &GridMap, attacker: GridPos, defender: GridPos) -> f32;
}
```

### 3. Faction 架构

#### 3.1 阵营定义

```rust
/// FactionDef — 配置文件加载
#[derive(Asset, TypePath)]
pub struct FactionDef {
    pub id: FactionDefId,
    pub name: LocalizationKey,
    pub default_relation: RelationLevel,  // 默认对其他阵营的态度
}

/// 阵营关系
#[derive(Resource)]
pub struct FactionRelations {
    relations: HashMap<(FactionDefId, FactionDefId), RelationLevel>,
}

/// 关系等级
pub enum RelationLevel {
    Ally,       // 盟友
    Neutral,    // 中立
    Hostile,    // 敌对
}

/// Faction Component — 挂在实体上
#[derive(Component)]
pub struct Faction {
    pub faction_id: FactionDefId,
}
```

#### 3.2 阵营判定 API

```rust
impl FactionRelations {
    /// 两个 Entity 之间的关系
    pub fn relation_between(
        &self,
        faction_a: FactionDefId,
        faction_b: FactionDefId,
    ) -> RelationLevel;

    /// Entity A 可以对 Entity B 造成伤害？
    pub fn can_deal_damage(&self, a: FactionDefId, b: FactionDefId) -> bool {
        self.relation_between(a, b) == RelationLevel::Hostile
    }

    /// Entity A 可以治疗 Entity B？
    pub fn can_heal(&self, a: FactionDefId, b: FactionDefId) -> bool {
        self.relation_between(a, b) != RelationLevel::Hostile
    }
}
```

### 4. 三 Feature 的协作

```
grid_map::GridMap (Resource)
    │
    ├── tile.terrain_def_id → terrain::TerrainDef (Asset)
    │       │
    │       └── terrain.def.tags → tag::TagSystem (Layer 2)
    │
    └── entity → faction::Faction (Component)
            │
            └── faction_relations (Resource) → 判定敌友
```

**寻路示例** — 结合网格 + 地形 + 阵营：

```rust
fn movement_pathfinding(
    grid: Res<GridMap>,
    terrain_assets: Res<Assets<TerrainDef>>,
    terrain_defs: Res<DefinitionRegistry>,
    unit_query: Query<(&GridPos, &Faction, &MovementRange)>,
    enemy_query: Query<(&GridPos, &Faction)>,
) -> Vec<GridPos> {
    // 1. 获取单位的网格位置和阵营
    // 2. 获取 TerrainDef 计算移动消耗
    // 3. 排除敌对阵营占据的格子
    // 4. A* 寻路
}
```

## Module Design

```
src/core/domains/tactical/
  ├── plugin.rs              — GridMapPlugin
  ├── components.rs          — TileMarker (条件性实体化)
  ├── resources.rs           — GridMap
  ├── systems.rs             — 网格初始化、坐标转换
  └── api.rs                 — GridPos, 网格查询函数

src/core/domains/terrain/
  ├── plugin.rs              — TerrainPlugin
  ├── resources.rs           — TerrainRegistry (Asset 加载后)
  ├── systems.rs             — 地形效果应用
  └── api.rs                 — TerrainDef, MovementCost

src/core/domains/faction/
  ├── plugin.rs              — FactionPlugin
  ├── components.rs          — Faction (Component, Tag)
  ├── resources.rs           — FactionRelations
  └── api.rs                 — FactionDef, RelationLevel
```

## Communication Design

| 通信 | 机制 | 方向 |
|------|------|------|
| 地形 Tag → Capability System | Definition 引用 | 编译时绑定 |
| 阵营判定 | `FactionRelation` Resource 查询 | 任意 Feature → faction |
| 网格位置更新 | `GridPos` Component 变化 | movement → grid_map |
| 地形变化 | 直接修改 `GridMap.tiles` | terrain 内部 |
| 寻路请求 | `PathRequest` Event | 外部 → grid_map |

## 边界定义

### 允许
- 任意 Feature 通过 `Res<GridMap>` 读取网格数据
- 任意 Feature 通过 `FactionRelations` 判定阵营关系
- Terrain 通过 Tag 系统影响游戏玩法

### 🟥 禁止
- GridMap 中存储业务逻辑（每个 Tile 上的单位列表、战斗状态等）
- Terrain 直接修改角色属性（必须通过 Tag → Modifier 链路）
- Faction 决定 AI 行为（AI 是独立 Feature）
- 网格坐标与渲染坐标混为一谈（GridPos ↔ Vec2 转换必须通过 GridMap）

## Forbidden

| 禁止行为 | 理由 |
|---------|------|
| 每个 Tile 一个 Entity | Entity 数量过大，Archetype 爆炸 |
| Terrain.Def 中包含业务逻辑 | 违反 Rule/Content 分离 |
| 阵营动态关系硬编码 | 应该配置化 |
| 网格坐标与像素坐标混淆 | 必须通过 GridMap 转换 |
| 在 GridMap.Resource 中存储可变 World 引用 | Resource 不应持有 World 引用 |

## Definition / Instance Design

- **Definition**: `TerrainDef` (Asset), `FactionDef` (Asset), `GridLayout` (config)
- **Instance**: `GridMap` (Resource), `Faction` (Component), `GridPos` (Component)
- **Persistence**: `GridMap.tiles`（地形变化）、`FactionRelations`、单位的 `GridPos`

## 后果

### 正面
- 网格用 Resource 连续数组存储，性能好
- Tile 实体化按需触发，不浪费
- 地形效果通过 Tag 桥接到能力系统，不污染 Layer 1
- 阵营关系配置化，扩展新阵营不改代码

### 负面
- 连续的 `GridMap::tiles: Vec<TileData>` 不支持稀疏网格（大地图浪费内存，需要 Chunk 机制优化）
- 实体化 Tile 的选择逻辑需要精心设计（避免频繁实体化/反实体化）

## 替代方案

| 方案 | 放弃理由 |
|------|---------|
| 全 Tile Entity | Entity 数量可能上万，Query 性能差 |
| Terrain 直接包含数值效果 | 违反 Rule/Content 分离 |
| Faction 用 u8 枚举硬编码 | 扩展新阵营需要改代码 |
| 网格寻路走 ECS System | 寻路是纯算法，资源+函数调用更合适 |

## 评审要点

- [ ] 是否支持大地图分块（Chunk）？当前 GridMap 是全量加载
- [ ] Hex 和 Square 网格的寻路算法是否统一？
- [ ] 阵营关系是否支持运行时临时变更（如"说服敌人加入"）？
- [ ] 地形高度对视距的影响是否需要区块预计算？
