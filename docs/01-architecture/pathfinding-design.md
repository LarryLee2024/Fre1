---
id: 01-architecture.pathfinding-design
title: Pathfinding Design
status: draft
owner: architect
created: 2026-06-14
updated: 2026-06-14
tags:
  - architecture
  - design
---

# 寻路与范围计算架构

> Version: 1.0
> Status: Proposed
> 来源：`docs/其他/31遗漏.md` Section 二（第241-249行）

---

## 1. 概述

寻路和范围计算是 SRPG 的**核心性能瓶颈**和**最易重复造轮子**的模块。本设计定义：

- 寻路算法的抽象层（支持算法替换）
- 移动范围、攻击范围、技能范围的统一计算接口
- 地形消耗、单位阻挡的统一处理
- 缓存策略与失效规则
- 性能预算

与 `map_terrain_rules.md` 的关系：**本文档是寻路基础设施的分层设计，`map_terrain_rules.md` 是地形领域的业务规则**。本文档定义"怎么算"，`map_terrain_rules.md` 定义"算什么"。

---

## 2. 设计

### 2.1 算法抽象层

#### PathFinder Trait

```rust
/// 寻路算法抽象 trait
/// 支持不同算法实现（BFS、A*、Dijkstra 等）
pub trait PathFinder: Send + Sync {
    /// 计算从起点出发的可达范围
    /// 
    /// # 参数
    /// - start: 起点坐标
    /// - move_points: 剩余移动力
    /// - terrain: 地形网格
    /// - occupancy: 占用网格
    /// - cost_calculator: 地形成本计算器
    /// - blocker: 阻挡规则
    ///
    /// # 返回
    /// HashMap<IVec2, u32> — 可达格子 → 到达后剩余移动力
    fn find_reachable_tiles(
        &self,
        start: IVec2,
        move_points: u32,
        terrain: &TerrainGrid,
        occupancy: &OccupancyGrid,
        cost_calculator: &dyn TerrainCostCalculator,
        blocker: &dyn UnitBlocker,
    ) -> HashMap<IVec2, u32>;

    /// 从可达范围结果中回溯路径
    fn reconstruct_path(
        &self,
        target: IVec2,
        reachable: &HashMap<IVec2, u32>,
        start: IVec2,
    ) -> Option<Vec<IVec2>>;

    /// 算法名称（用于调试和日志）
    fn name(&self) -> &str;
}
```

#### PathFindingContext 封装

当参数列表过长时，封装为上下文结构体提升可读性和可维护性：

```rust
/// 寻路上下文 — 封装所有寻路计算所需的输入参数
pub struct PathFindingContext<'a> {
    pub start: IVec2,
    pub move_points: u32,
    pub terrain: &'a TerrainGrid,
    pub occupancy: &'a OccupancyGrid,
    pub cost_calculator: &'a dyn TerrainCostCalculator,
    pub blocker: &'a dyn UnitBlocker,
}

// 简化后的接口
fn find_reachable_tiles(&self, ctx: &PathFindingContext) -> HashMap<IVec2, u32>;
```

> **优化来源**: `docs/其他/59.md`（PathFindingContext 封装建议）

#### 默认实现：BFS PathFinder

```rust
/// BFS 寻路实现（当前默认）
/// 适用于移动范围计算，保证最短路径
pub struct BfsPathFinder;

impl PathFinder for BfsPathFinder {
    fn find_reachable_tiles(&self, ...) -> HashMap<IVec2, u32> {
        // BFS 四方向扩展
        // 每步消耗 terrain_cost
        // 剩余移动力 > 0 时继续扩展
        // 跳过不可通行格子和被占据格子（除自身外）
    }

    fn reconstruct_path(&self, ...) -> Option<Vec<IVec2>> {
        // 从目标回溯到起点
        // 返回路径（不含起点）
    }
}
```

#### 未来扩展：A* PathFinder

```rust
/// A* 寻路实现（未来扩展）
/// 适用于从起点到特定终点的最短路径
pub struct AStarPathFinder;

impl PathFinder for AStarPathFinder {
    fn find_reachable_tiles(&self, ...) -> HashMap<IVec2, u32> {
        // A* 算法，启发式函数为曼哈顿距离
    }

    fn reconstruct_path(&self, ...) -> Option<Vec<IVec2>> {
        // 从目标回溯到起点
    }
}
```

#### 算法选择策略

| 场景 | 推荐算法 | 原因 |
|------|---------|------|
| 移动范围计算 | BFS | 需要所有可达格子 |
| 从 A 到 B 最短路径 | A* | 只需单目标路径 |
| 大地图范围查询 | Dijkstra | 有权重的可达范围 |

---

### 2.2 统一范围计算接口

所有范围计算遵循**相同的接口模式**。

#### RangeCalculator Trait

```rust
/// 范围计算统一接口
pub trait RangeCalculator: Send + Sync {
    /// 计算范围
    /// 
    /// # 参数
    /// - source: 计算源点（单位位置）
    /// - params: 范围计算参数
    /// - context: 计算上下文
    ///
    /// # 返回
    /// RangeResult — 范围内的合法目标集合
    fn calculate(
        &self,
        source: IVec2,
        params: &RangeParams,
        context: &RangeContext,
    ) -> RangeResult;
}
```

#### 三种范围计算器

```rust
/// 移动范围计算器
pub struct MoveRangeCalculator {
    path_finder: Box<dyn PathFinder>,
}

/// 攻击范围计算器
pub struct AttackRangeCalculator;

/// 技能范围计算器
pub struct SkillRangeCalculator;
```

#### 统一参数结构

```rust
/// 范围计算参数
pub struct RangeParams {
    /// 范围类型
    pub range_type: RangeType,
    /// 基础范围值（格子数）
    pub base_range: u32,
    /// 额外范围修正（装备/Buff 加成）
    pub bonus_range: i32,
    /// 是否包含自身位置
    pub include_self: bool,
}

/// 范围类型
pub enum RangeType {
    /// 十字形（上下左右）
    Cross,
    /// 菱形（曼哈顿距离）
    Diamond,
    /// 方形（切比雪夫距离）
    Square,
    /// 直线（视线检测）
    Line,
    /// 全地图
    Global,
}

/// 范围计算上下文
pub struct RangeContext {
    pub terrain: TerrainGrid,
    pub occupancy: OccupancyGrid,
    pub cost_calculator: Box<dyn TerrainCostCalculator>,
    pub blocker: Box<dyn UnitBlocker>,
}

/// 范围计算结果
pub struct RangeResult {
    /// 范围内的合法坐标集合
    pub positions: HashSet<IVec2>,
    /// 每个坐标的路径（可选，用于移动动画）
    pub paths: HashMap<IVec2, Vec<IVec2>>,
}
```

#### 使用示例

```rust
// 移动范围
let move_range = MoveRangeCalculator::new(BfsPathFinder)
    .calculate(unit_pos, &RangeParams::movement(5), &context);

// 攻击范围（十字形，范围 2）
let attack_range = AttackRangeCalculator
    .calculate(unit_pos, &RangeParams::cross(2), &context);

// 技能范围（方形，范围 3）
let skill_range = SkillRangeCalculator
    .calculate(unit_pos, &RangeParams::square(3), &context);
```

---

### 2.3 阻挡规则

#### UnitBlocker Trait

```rust
/// 单位阻挡规则 trait
pub trait UnitBlocker: Send + Sync {
    /// 检查指定格子是否阻挡移动
    fn is_blocked(
        &self,
        coord: IVec2,
        moving_entity: Entity,
        occupancy: &OccupancyGrid,
    ) -> bool;
}
```

#### 内置阻挡规则

| 规则 | 说明 | 阻挡条件 |
|------|------|---------|
| FriendlyBlocker | 友方单位阻挡 | Occupancy 中有友方单位（排除自身） |
| EnemyBlocker | 敌方单位阻挡 | Occupancy 中有敌方单位 |
| AllBlocker | 所有单位阻挡 | Occupancy 中有任何单位（排除自身） |
| NoBlocker | 无阻挡 | 不阻挡任何单位 |

#### 地形阻挡

地形阻挡由 `TerrainCostCalculator` 处理：

```rust
// TerrainCostCalculator.cost() 返回 None 时 = 不可通行
// 这等同于地形阻挡
fn cost(&self, terrain_id: &str, base_cost: Option<u32>) -> Option<u32> {
    match terrain_id {
        "water" => None,  // 水域对地面单位不可通行
        "mountain" => None,  // 山地对地面单位不可通行
        _ => base_cost,
    }
}
```

#### 视线检测抽象

技能范围的 Line 类型需要明确的视线检测规则：

```rust
/// 视线检测 trait
/// 用于技能范围计算中的 Line 类型判定
pub trait LineOfSightChecker: Send + Sync {
    /// 检查从 source 到 target 的直线上是否存在遮挡
    fn has_line_of_sight(&self, source: IVec2, target: IVec2, terrain: &TerrainGrid) -> bool;
}

/// 默认实现：Bresenham 直线扫描
pub struct DefaultLineOfSightChecker;

impl LineOfSightChecker for DefaultLineOfSightChecker {
    fn has_line_of_sight(&self, source: IVec2, target: IVec2, terrain: &TerrainGrid) -> bool {
        // Bresenham 算法遍历 source→target 直线上的每个格子
        // 若任一格子为不可通行地形 → 返回 false（被遮挡）
        // 飞行单位的视线不被地面单位阻挡
    }
}
```

**飞行单位阻挡规则补充**：
- 飞行单位的视线不被地面单位阻挡（`FlyingUnitBlocker` 实现）
- 飞行单位可穿过敌方单位（`FlyThroughEnemyBlocker` 实现）
- 但飞行单位仍被不可通行地形阻挡（如墙壁、悬崖）

> **优化来源**: `docs/其他/59.md`（视线检测抽象与飞行单位阻挡规则建议）

---

### 2.4 缓存策略

#### 可缓存的范围

| 范围类型 | 可缓存 | 缓存键 | 失效条件 |
|---------|--------|--------|---------|
| 移动范围 | 🟥 | (Entity, GridPosition, MovePoints) | 单位移动、地形变化、移动力变化 |
| 攻击范围 | 🟩 | (Entity, AttackRange) | 单位移动、攻击范围变化 |
| 技能范围 | 🟩 | (SkillId, SourcePosition) | 单位移动、技能范围变化 |

#### 缓存结构

```rust
/// 范围缓存 Resource
#[derive(Resource)]
pub struct RangeCache {
    /// 移动范围缓存
    move_ranges: HashMap<Entity, CachedRange>,
    /// 攻击范围缓存
    attack_ranges: HashMap<Entity, CachedRange>,
    /// 技能范围缓存
    skill_ranges: HashMap<(Entity, SkillId), CachedRange>,
}

/// 缓存条目
struct CachedRange {
    result: RangeResult,
    frame_created: u32,  // 创建时的帧号
    valid: bool,         // 是否有效
}
```

#### 缓存失效规则

```
缓存失效触发条件：
1. 单位移动 → 清除该单位的 move_range 缓存
2. 地形变化 → 清除所有 range 缓存
3. 单位死亡 → 清除该单位的所有缓存
4. Buff 应用/移除（影响移动力）→ 清除该单位的 move_range 缓存
5. 装备穿脱（影响攻击范围）→ 清除该单位的 attack_range 缓存
6. 回合开始 → 清除所有缓存（acted 重置影响可达范围）
7. **软失效** → N 帧未使用的缓存条目自动标记为无效，下次访问时重新计算

#### 缓存软失效策略

冷门缓存长期占用内存的问题，通过"软失效"机制解决：

```rust
/// 缓存条目（增加 last_accessed 字段）
struct CachedRange {
    result: RangeResult,
    frame_created: u32,
    last_accessed: u32,   // 最后一次被访问的帧号
    valid: bool,
}

/// 软失效常量
const CACHE_SOFT_EXPIRY_FRAMES: u32 = 120;  // 120 帧（约 2 秒）未使用则失效

/// 缓存清理系统（每帧执行）
fn soft_expire_cache(mut cache: ResMut<RangeCache>, current_frame: Res<FrameCounter>) {
    for entry in cache.all_entries_mut() {
        if entry.valid && (current_frame.0 - entry.last_accessed > CACHE_SOFT_EXPIRY_FRAMES) {
            entry.valid = false;  // 标记为失效，下次访问时重新计算
        }
    }
}
```

**优势**：
- 频繁访问的缓存保持有效（每次访问更新 `last_accessed`）
- 冷门缓存自动释放，避免内存泄漏
- 比立即清空更温和，不会导致突发的计算压力

> **优化来源**: `docs/其他/59.md`（缓存"软失效"策略建议）
```

#### 缓存内存预算

```
每个缓存条目约 200-500 bytes（取决于地图大小）
标准 20×20 地图，10 个单位：
  - 移动范围：10 × 500 = 5KB
  - 攻击范围：10 × 300 = 3KB
  - 技能范围：10 × 3 × 300 = 9KB
  - 总计：约 17KB（可接受）

禁止：
  🟥 无上限缓存（内存泄漏）
  🟥 缓存无失效条件（数据过期）
  🟥 缓存不命中时重新计算所有范围
```

---

### 2.5 性能预算

#### 目标指标

| 指标 | 预算 | 说明 |
|------|------|------|
| 单次移动范围计算 | ≤ 2ms | 标准 20×20 地图，10 个单位 |
| 单次攻击范围计算 | ≤ 0.5ms | 方形/十字形范围 |
| 单次技能范围计算 | ≤ 1ms | 含视线检测 |
| 缓存命中率 | ≥ 80% | 正常战斗流程 |
| 总寻路开销/帧 | ≤ 5ms | 所有范围计算总和 |

#### 性能优化策略

| 策略 | 说明 | 适用场景 |
|------|------|---------|
| 缓存优先 | 先查缓存，命中则直接返回 | 所有范围计算 |
| 增量更新 | 只重算受影响的单位范围 | 单位移动后 |
| 提前终止 | BFS 扩展到移动力耗尽时停止 | 移动范围计算 |
| 视线剪枝 | A* 中提前剪枝不可见目标 | 技能范围计算 |

#### 性能监控

```rust
/// 寻路性能统计 Resource
#[derive(Resource)]
pub struct PathfindingStats {
    pub total_calculations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_calculation_time_us: f64,  // 微秒
    pub max_calculation_time_us: f64,
}
```

### Bevy Schedule 集成

寻路系统在 Bevy Schedule 中的位置应明确划分：

> **优化来源**: `docs/其他/59.md`（Bevy Schedule 集成建议）

```rust
/// 寻路系统注册到 Bevy Schedule
impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, clear_range_cache)       // 清除过期缓存
           .add_systems(Update, calculate_ranges)            // 范围计算
           .add_systems(PostUpdate, apply_range_results)     // 结果应用
           .insert_resource(PathfindingStats::default())
           .insert_resource(RangeCache::default());
    }
}
```

**阶段职责**：
- **PreUpdate**：清除过期缓存（事件驱动 + 软失效），释放内存
- **Update**：执行范围计算（BFS/A*），更新缓存
- **PostUpdate**：将计算结果应用到游戏状态（触发 UI 更新等）

### 并行计算支持

多单位同时计算范围时，可利用 rayon 并行加速：

> **优化来源**: `docs/其他/59.md`（并行计算支持建议）

```rust
use rayon::prelude::*;

/// 并行计算多个单位的移动范围
pub fn calculate_ranges_parallel(
    units: &[(Entity, IVec2, u32)],  // (Entity, 位置, 移动力)
    ctx: &PathFindingContext,
) -> HashMap<Entity, HashMap<IVec2, u32>> {
    units.par_iter()
        .map(|&(entity, start, move_points)| {
            // 注意：ChaCha8Rng 需 clone 给每个线程
            let thread_ctx = PathFindingContext {
                start,
                move_points,
                terrain: ctx.terrain,
                occupancy: ctx.occupancy,
                cost_calculator: ctx.cost_calculator,
                blocker: ctx.blocker,
            };
            let result = BfsPathFinder.find_reachable_tiles(&thread_ctx);
            (entity, result)
        })
        .collect()
}
```

**注意事项**：
- `PathFindingContext` 中的所有引用必须是 `Sync` 的（已满足，因为底层数据是不可变的）
- 若使用随机数生成器（如 ChaCha8Rng），需为每个线程 clone 独立实例
- 并行计算仅适用于独立的单位范围计算，有依赖关系的计算（如链式触发）需串行

### 大地图兜底策略

对于 40×40 及以上的大型地图，单帧计算可能超时，需要兜底策略：

> **优化来源**: `docs/其他/59.md`（大地图兜底策略建议）

| 地图规模 | 策略 | 说明 |
|---------|------|------|
| ≤ 30×30 | 正常计算 | 单帧完成所有范围计算 |
| 31-50 | 分帧计算 | 每帧最多 8 个路径请求，超限排队 |
| > 50 | 分帧 + 降精度 | 每帧 4 个请求，BFS 限制最大搜索步数 |

```rust
/// 大地图分帧计算 Resource
#[derive(Resource)]
pub struct LargeMapScheduler {
    /// 排队中的路径请求
    pending_requests: VecDeque<PathRequest>,
    /// 每帧最大处理数
    max_per_frame: usize,
    /// 当前帧已处理数
    processed_this_frame: usize,
}

impl LargeMapScheduler {
    /// 根据地图大小初始化
    pub fn for_map_size(width: u32, height: u32) -> Self {
        let max_per_frame = if width * height > 2500 {
            4   // 超大地图
        } else if width * height > 900 {
            8   // 大地图
        } else {
            usize::MAX  // 正常地图，不限制
        };
        Self {
            pending_requests: VecDeque::new(),
            max_per_frame,
            processed_this_frame: 0,
        }
    }
}
```

---

## 3. 不变量

### 不变量1：所有范围计算必须通过统一接口

```
移动范围、攻击范围、技能范围必须通过对应的 RangeCalculator 计算。
禁止：在业务代码中手写 BFS/A* 逻辑
禁止：绕过 RangeCalculator 直接操作 TerrainGrid
```

### 不变量2：缓存必须有失效条件

```
每个缓存条目必须有明确的失效触发条件。
禁止：无限期缓存
禁止：缓存无 valid 标记
```

### 不变量3：寻路必须考虑单位阻挡

```
find_reachable_tiles 必须通过 UnitBlocker 检查占用格子。
禁止：忽略 OccupancyGrid
禁止：硬编码阻挡逻辑
```

### 不变量4：飞行单位只忽略移动消耗

```
FlyingCostCalculator 对所有地形返回 move_cost = Some(1)，
但地形的 defense_bonus 仍正常参与伤害计算。
禁止：飞行单位跳过地形防御加成
```

### 不变量5：移动范围基于当前剩余移动力

```
find_reachable_tiles 的 move_points 参数必须是当前剩余移动力。
禁止：使用最大移动力计算
```

---

## 4. 禁止事项

| 禁止项 | 理由 | 违反后果 |
|--------|------|---------|
| 🟥 硬编码 A* 实现无抽象层 | 无法替换算法 | 重构成本高 |
| 🟥 每种技能/目标类型单独实现范围计算 | 重复代码 | 维护困难 |
| 🟥 缓存无失效条件 | 数据过期 | 寻路结果错误 |
| 🟥 无限制缓存 | 内存泄漏 | OOM |
| 🟥 寻路直接查询 Entity | 破坏数据与表现分离 | 性能下降 |
| 🟥 硬编码移动成本 | 违反数据驱动 | 新增地形需改代码 |
| 🟥 忽略 OccupancyGrid 占用信息 | 单位重叠 | 游戏逻辑错误 |
| 🟥 使用最大移动力而非当前移动力 | 范围膨胀 | 游戏平衡破坏 |

---

## 5. 交叉引用

| 文档 | 关系 |
|------|------|
| `docs/02-domain/map_terrain_rules.md` | 地形业务规则（TerrainGrid、TerrainCostCalculator） |
| `docs/01-architecture/determinism_rules.md` | 寻路结果必须确定性 |
| `docs/02-domain/turn_rules.md` | 移动范围影响回合行动选择 |
| `docs/02-domain/battle_rules.md` | 攻击范围影响 Effect Pipeline 目标选择 |
| `docs/01-architecture/README.md` | Pathfinding 模块边界定义 |
| `docs/AI开发宪法完整版.md` | §9.0.1-9.0.7 地图系统、§1.4.1 领域纯度、§11.7 读写分离 |

---

## 宪法合规说明

| 条款 | 合规状态 | 说明 |
|------|---------|------|
| 🟩 §9.0.6 寻路数据动态生成 | ✅ 合规 | PathFinder trait 在运行时计算 |
| 🟩 §9.0.4 OccupancyGrid 独立 | ✅ 合规 | OccupancyGrid 作为独立数据结构 |
| 🟩 §9.0.5 数据与渲染分离 | ✅ 合规 | 纯数据计算，与渲染层完全分离 |
| 🟩 §1.4.1 领域纯度 | ✅ 合规 | PathFinder trait 和 PathFindingContext 为纯数据结构，不绑定 Bevy ECS |
| 🟩 §1.4.2 领域无副作用 | ✅ 合规 | 寻路函数为纯计算，不修改外部状态 |
| 🟩 §11.7 读写分离 | ✅ 合规 | 范围计算为只读操作，结果通过独立步骤应用 |
| 🟩 §2.3.4 Resource 规范 | ✅ 合规 | RangeCache 为全局唯一缓存状态 |
