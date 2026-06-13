# 地图地形领域

Version: 1.0
Status: Proposed

地图地形领域管理战术网格地图的数据存储、地形效果计算与寻路。

核心原则：
- 地图数据（TerrainGrid）是纯数据 Resource，不使用 ECS Entity
- 地形效果通过 Effect Pipeline 的 GenerateContext 传入，不直接修改属性
- 寻路算法基于 TerrainCostCalculator trait 数据驱动扩展

---

# 术语定义

## 地图（Map）

由 Tile 组成的二维网格战场，定义关卡的空间布局。

不是关卡配置。不是地形效果。

关键属性：
- 地图尺寸由 GameMap 资源存储（width、height、tile_size）
- 坐标系原点在左下角，coord_to_world / world_to_coord 互转
- is_in_bounds() 判断坐标合法性（0 ≤ x < width, 0 ≤ y < height）

---

## Tile（格子）

地图的最小单元，携带 terrain_id 和坐标。

不是 Entity。不是地形类型。

关键属性：
- Tile 是 TerrainGrid 中的 HashMap 键值对（IVec2 → terrain_id: String）
- Tile 不是 ECS Entity，不挂载 Component
- Tile 的地形属性通过 TerrainRegistry 查询获得
- 渲染层生成纯 Sprite（TileSprite），不生成携带地形数据的 Entity

---

## 地形类型（Terrain Type）

格子的地形分类，由地形 ID 字符串标识（如 "plain"、"forest"、"mountain"、"water"）。

不是 Tile。不是地形效果。

关键属性：
- 地形 ID 由 TerrainDef 定义，存储在 TerrainRegistry 注册表中
- 每种地形类型有唯一 ID（String）
- 内置 4 种默认地形：plain、forest、mountain、water
- 地形类型从 RON 文件加载（assets/terrains/*.ron），Rule/Content 分离

---

## 地形效果（Terrain Effect）

地形对单位的战斗加成，包括防御加成、回避率、移动消耗等。

不是 Modifier 本身（但防御加成会传入 EffectContext 影响伤害计算）。不是 Buff。

关键属性：
- defense_bonus：地形防御值，由 GenerateContext 传入 Effect Pipeline
- move_cost：移动消耗，由 TerrainCostCalculator 计算实际通行成本
- 地形效果数据存储在 TerrainDef（Definition），不可在战斗中修改

---

## 地形防御加成（Terrain Defense Bonus）

terrain_id 对应的防御值，影响伤害计算公式中的减伤。

不是装备防御。不是 Buff 防御。

关键属性：
- 存储在 TerrainDef.defense_bonus（i32）
- 在 Generate 阶段从 TerrainRegistry 查询后传入 GenerateContext
- 伤害公式：final_damage = (effective_atk - effective_def - terrain_defense_bonus) × multiplier
- forest 地形默认 defense_bonus = 2，其他地形默认 = 0

---

## 移动消耗（Move Cost）

经过一个格子消耗的移动力。

不是地形标签。不是单位属性。

关键属性：
- 基础值存储在 TerrainDef.move_cost（Option<u32>）
- None 表示基础不可通行（如 mountain、water）
- Some(n) 表示消耗 n 点移动力（如 plain=1、forest=2）
- 实际消耗由 TerrainCostCalculator 根据单位类型重算（如飞行单位所有地形成本=1）

---

## 寻路（Pathfinding）

从起点到终点的最优移动路径计算。

不是直线距离。不是传送。

关键属性：
- 使用 BFS 算法计算可达范围（find_reachable_tiles）
- 使用 reconstruct_path 从可达结果回溯最短路径
- 寻路输入：起点、剩余移动力、地图、地形网格、地形注册表、占用网格、单位实体、成本计算器
- 寻路直接从 TerrainGrid + OccupancyGrid 读取数据，不依赖 Tile Entity

---

## 移动范围（Moveable Range）

单位在当前移动力内可达的格子集合。

不是移动路径。不是单位实体。

关键属性：
- 由 find_reachable_tiles 返回 HashMap<IVec2, u32>（坐标 → 剩余移动力）
- 起点不在返回集合中（起点视为已被占据）
- 每个格子的值为到达后剩余的移动力
- 占据格子（除自身外）不可达

---

## 战争迷雾（Fog of War）

限制玩家视野的机制。

不是 Tile 属性。不是关卡配置。

关键属性：
- 当前版本未实现
- 实现时战争迷雾数据必须由 Core 层管理
- UI 层只读取可见性状态，不修改迷雾数据
- 战争迷雾状态必须在每回合结束时重新计算

---

## 关卡配置（Level Config）

RON 文件中定义的完整关卡数据，包含地图尺寸、地形网格、单位部署位置、胜负条件。

不是地图本身。不是单位配置。

关键属性：
- 定义态为 LevelConfigDef（RON 反序列化用），运行态为 LevelConfig
- 地形网格 terrain_grid 为 Vec<String>，每行一个字符串，每个字符映射到地形 ID
- 自定义 char_map 可覆盖 TerrainRegistry 的默认字符映射
- 包含 player_units 和 enemy_units 部署列表
- 包含可选的 victory_condition 和 turn_limit

---

## 寻路器（PathFinder）

可替换的寻路算法抽象，通过 trait 定义统一接口。

不是 BFS。不是固定算法。

关键属性：
- 默认实现为 BFS PathFinder（适用于移动范围计算）
- 未来扩展：A* PathFinder（适用于 A-B 最短路径）、Dijkstra（适用于大地图）
- 通过 trait 抽象，支持运行时替换算法
- 来源：`docs/architecture/pathfinding_design.md`

---

## 范围计算器（RangeCalculator）

统一的移动范围、攻击范围、技能范围计算接口。

不是单独函数。不是硬编码范围。

关键属性：
- 移动范围（MoveRangeCalculator）：基于 PathFinder 计算可达格子
- 攻击范围（AttackRangeCalculator）：十字形/菱形/方形范围
- 技能范围（SkillRangeCalculator）：含视线检测的范围计算
- 所有范围计算通过统一接口，禁止在业务代码中手写 BFS/A* 逻辑

---

## 单位阻挡器（UnitBlocker）

阻挡规则抽象，决定哪些单位阻挡移动。

不是碰撞检测。不是地形消耗。

关键属性：
- FriendlyBlocker：友方单位阻挡（排除自身）
- EnemyBlocker：敌方单位阻挡
- AllBlocker：所有单位阻挡（排除自身）
- NoBlocker：无阻挡
- 地形阻挡由 TerrainCostCalculator.cost() 返回 None 处理

---

## 范围缓存（RangeCache）

寻路结果缓存及失效策略，避免重复计算。

不是永久缓存。不是无界存储。

关键属性：
- 缓存键：(Entity, GridPosition, MovePoints) 用于移动范围
- 有明确的失效触发条件（单位移动、地形变化、回合开始等）
- 每个条目有 valid 标记和 frame_created 帧号
- 内存有预算限制（标准 20×20 地图约 17KB）

---

# 领域边界

## 本领域负责

- 地图数据存储（TerrainGrid：纯数据 Resource）
- 占用数据存储（OccupancyGrid：记录单位占位）
- 地形定义注册（TerrainRegistry：从 RON 加载地形类型）
- 地图渲染（spawn_map：生成 TileSprite，不生成 Tile Entity）
- 坐标转换（GameMap：coord_to_world / world_to_coord）
- 寻路算法（BFS 可达范围 + 路径回溯）
- 地形成本计算（TerrainCostCalculator trait + 4 种内置实现）
- 地形防御加成查询（defense_bonus 从 TerrainRegistry 读取）
- 关卡配置加载（LevelConfigDef → LevelConfig 转换）

## 本领域不负责

- 属性修饰与伤害计算（由 Core 属性修饰管线负责）
- 战斗效果管线执行（由 Battle Effect Pipeline 负责）
- 回合阶段与行动顺序（由 Turn 领域负责）
- Buff/Debuff 的生命周期（由 Buff 领域负责）
- 单位组件与模板加载（由 Character 领域负责）
- 用户输入处理（由 Input 领域负责）
- UI 面板与交互（由 UI 领域负责）
- AI 策略选择（由 AI 领域负责）
- 战争迷雾数据管理（由 Core 层负责，未来实现）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| defense_bonus 查询 | 函数调用 | Battle（Generate 阶段读取 TerrainRegistry） |
| 移动范围计算 | 函数调用 | Input（玩家点击时查询）/ AI（决策时查询） |
| 单位位置变化 | 函数调用 | OccupancyGrid.rebuild()（每帧从单位位置更新） |
| 地形网格查询 | 函数调用 | 所有领域（TerrainGrid.get()） |
| 关卡数据加载 | Resource | LevelRegistry、TerrainRegistry |

---

# 生命周期

本领域无状态机，为纯数据存储和函数式计算。

TerrainGrid 和 OccupancyGrid 是 Resource，生命周期为：
- 系统启动时初始化（从 LevelConfig 构建）
- 战斗中持续更新（OccupancyGrid 每帧从单位位置重建）
- 关卡切换时重新构建

唯一有状态的是 OccupancyGrid（Resource），其更新时机为：
- 每帧 Update：update_occupancy_grid 从所有单位位置重建
- 单位移动后：手动调用 set/remove 更新
- 关卡加载时：rebuild() 全量重建

---

# 不变量

## 不变量1：TerrainGrid 是地形数据的唯一真相源

任意时刻：

寻路、UI 渲染、战斗 Generate 阶段查询地形数据，必须从 TerrainGrid + TerrainRegistry 获取，禁止从其他来源读取。

违反表现：

寻路直接从 LevelConfig.terrain_map 读取而非 TerrainGrid；或 UI 使用硬编码的地形数据。

---

## 不变量2：OccupancyGrid 是单位占位的唯一真相源

任意时刻：

判断格子是否被占据，必须从 OccupancyGrid 读取，禁止维护独立的占位 HashMap。

违反表现：

寻路使用临时 HashMap 判断占用，而 OccupancyGrid 已有最新数据；或两个来源数据不一致。

---

## 不变量3：地形防御加成必须通过 EffectContext 传入

任意时刻：

terrain_defense_bonus 必须在 Generate 阶段从 TerrainRegistry 查询后，通过 GenerateContext 传入 Effect Pipeline。禁止在 Execute 阶段直接读取 TerrainRegistry 修改伤害。

违反表现：

Execute 阶段直接调用 TerrainRegistry 获取 defense_bonus 并修改 HP；Generate 阶段未传入 defense_bonus。

---

## 不变量4：飞行单位忽略地形移动消耗但不忽略地形防御加成

任意时刻：

FlyingCostCalculator 对所有地形返回 move_cost = Some(1)，但地形的 defense_bonus 仍按 TerrainDef.defense_bonus 值正常参与伤害计算。

违反表现：

飞行单位忽略 forest 地形的 defense_bonus = 2；或飞行单位被 mountain/water 的 move_cost=Some(1) 以外的值阻挡。

---

## 不变量5：关卡配置数据不可在战斗中修改

任意时刻：

LevelConfig 和 TerrainDef 是 Definition（不可变），战斗中的数值修改应通过 Modifier 管线作用于 Instance，不修改 Definition 数据。

违反表现：

战斗中调用 terrain_registry.get("forest").defense_bonus = 99 修改地形防御值；或修改 LevelConfig.terrain_map。

---

## 不变量6：移动范围基于当前剩余移动力计算

任意时刻：

find_reachable_tiles 的 move_points 参数必须是单位当前剩余移动力（受已移动距离影响），不是最大移动力。

违反表现：

单位已移动 2 格后仍按最大移动力计算可达范围；或 move_points 为 0 时仍返回非空可达集合。

---

# 业务规则

## 规则1：地形成本计算数据驱动

允许：
- 通过 TerrainCostCalculator trait 实现新的地形成本计算器
- 通过 TerrainCostRegistry.register() 注册自定义计算器
- 使用 GameplayTag 解析单位类型对应的计算器

禁止：
- 硬编码单位类型的地形通行逻辑（如 if unit_type == "flying"）
- 跳过 TerrainCostRegistry 直接调用内置计算器

必须：
- TerrainCostCalculator.cost() 返回 Some(n) 表示可通行（消耗 n 移动力），None 表示不可通行
- 优先级：SWIMMING > FLYING > MOUNTED > 默认（ground）

---

## 规则2：地形数据与 ECS Entity 分离

允许：
- TerrainGrid 作为 Resource 存储地形数据
- TileSprite Entity 仅用于渲染，不携带地形数据
- OccupancyGrid 作为 Resource 存储单位占位

禁止：
- 为每个 Tile 生成携带 terrain_id Component 的 ECS Entity
- 通过 ECS 查询获取地形数据（必须通过 TerrainGrid Resource）
- 将地形数据存储在 Unit Entity 的 Component 中

必须：
- 寻路从 TerrainGrid + OccupancyGrid 读取数据
- UI 渲染从 TerrainGrid 读取地形 ID 查询 TerrainRegistry 获取颜色/属性

---

## 规则3：关卡加载管线

允许：
- 从 assets/maps/*.ron 加载 LevelConfigDef
- LevelConfigDef 使用 char_map 自定义字符到地形 ID 映射
- 未映射的字符回退到 "plain"

禁止：
- 在关卡加载后修改 TerrainDef 的 move_cost 或 defense_bonus
- 跳过 TerrainRegistry 直接解析地形字符
- 硬编码地形字符映射（必须从 TerrainRegistry.char_map() 获取）

必须：
- 关卡加载顺序：加载 TerrainRegistry → 加载 LevelRegistry → spawn_map 构建 TerrainGrid
- LevelConfig::from_def() 使用 TerrainRegistry 的 char_map 作为默认映射

---

## 规则4：寻路算法规则

允许：
- 使用 BFS 算法计算可达范围（默认）
- 使用 reconstruct_path 回溯最短路径
- 满足条件时重算寻路（单位位置变化、地形变化、移动力变化）

禁止：
- 忽略 OccupancyGrid 的占用信息
- 忽略 TerrainCostCalculator 的通行判断
- 在可移动范围计算中使用最大移动力而非当前移动力

必须：
- find_reachable_tiles 排除起点（起点不在可达集合中）
- 起点自身位置不算被占用（传入 moving_entity 参数）
- 被占据的格子（除自身外）不可达

---

## 规则6：寻路算法选择

允许：
- 移动范围计算使用 BFS（默认，需所有可达格子）
- A-B 最短路径使用 A*（单目标路径）
- 大地图范围查询使用 Dijkstra（有权重可达范围）

禁止：
- 算法硬编码不抽象（必须通过 PathFinder trait）
- 在业务代码中直接实现 BFS/A* 逻辑（必须使用 RangeCalculator）
- 绕过 RangeCalculator 直接操作 TerrainGrid

必须：
- 新算法实现 PathFinder trait 并注册到 PathFinderRegistry
- 算法选择通过 RangeCalculator 统一调度

---

## 规则7：范围缓存策略

允许：
- 移动范围使用缓存（键：Entity, GridPosition, MovePoints）
- 攻击范围使用缓存（键：Entity, AttackRange）
- 技能范围使用缓存（键：SkillId, SourcePosition）

禁止：
- 无失效条件的缓存（所有缓存必须有明确的失效触发）
- 无界缓存（必须有内存预算限制）
- 缓存不命中时重新计算所有范围（只重算失效条目）

必须：
- 缓存失效条件：单位移动、地形变化、单位死亡、Buff 变化（影响移动力）、装备变化（影响攻击范围）、回合开始
- 缓存条目有 valid 标记和 frame_created 帧号
- 标准 20×20 地图缓存总内存 ≤ 17KB

---

## 规则8：寻路性能预算

允许：
- 单次移动范围计算 ≤ 2ms（标准 20×20 地图，10 个单位）
- 单次攻击范围计算 ≤ 0.5ms（方形/十字形范围）
- 单次技能范围计算 ≤ 1ms（含视线检测）
- 缓存命中率 ≥ 80%（正常战斗流程）

禁止：
- 总寻路开销/帧 > 5ms（所有范围计算总和）
- 缓存命中率 < 80%（需优化失效策略）

必须：
- 缓存优先：先查缓存，命中则直接返回
- 增量更新：只重算受影响的单位范围
- 提前终止：BFS 扩展到移动力耗尽时停止

---

## 规则5：伤害计算中的地形防御加成

允许：
- terrain_defense_bonus 在 Generate 阶段从 TerrainRegistry 查询
- 通过 GenerateContext 传入 Effect Pipeline
- 伤害公式中减去 terrain_defense_bonus

禁止：
- 在 Execute 阶段直接读取 TerrainRegistry 修改伤害
- 硬编码地形防御值（必须从 TerrainDef.defense_bonus 读取）
- 飞行单位忽略地形防御加成

必须：
- Generate 阶段：defense_bonus = terrain_registry.get(terrain_id).defense_bonus
- 伤害公式：final_damage = ((effective_atk - effective_def - terrain_defense_bonus) × multiplier).max(1)
- terrain_defense_bonus 为 0 时不影响计算

---

# 流程管线

## 关卡加载管线

```
RON 反序列化 → LevelConfigDef → LevelConfig 构建 → TerrainGrid 生成 → OccupancyGrid 初始化 → 初始视野计算
```

### Step1：RON 反序列化

输入：assets/maps/*.ron 文件
处理：ron::de::from_bytes 反序列化为 LevelConfigDef
输出：LevelConfigDef 实例
禁止：修改 RON 原始数据

### Step2：LevelConfig 构建

输入：LevelConfigDef + TerrainRegistry
处理：LevelConfig::from_def() 解析 terrain_grid，使用 char_map 映射字符到地形 ID
输出：LevelConfig（terrain_map: HashMap<(i32,i32), String>）
禁止：跳过 TerrainRegistry 的 char_map 映射

### Step3：TerrainGrid 生成

输入：LevelConfig 的 terrain_map
处理：TerrainGrid::from_terrain_map() 构建地形网格 Resource
输出：TerrainGrid Resource 插入 ECS World
禁止：为每个格子生成 ECS Entity

### Step4：OccupancyGrid 初始化

输入：所有单位的 Entity 和 GridPosition
处理：OccupancyGrid::rebuild() 从单位位置重建占用表
输出：OccupancyGrid Resource 初始状态
禁止：跳过占用表初始化

### Step5：初始视野计算

输入：TerrainGrid、单位位置、视野范围
处理：计算初始视野（战争迷雾未实现，当前版本跳过）
输出：视野数据（未来实现）
禁止：直接修改 TerrainGrid 的地形数据

---

## 寻路管线

```
起点坐标 → 剩余移动力 → 可达范围计算（BFS） → 最优路径回溯
```

### Step1：起点校验

输入：起点坐标、地图尺寸
处理：is_in_bounds() 检查起点合法性
输出：起点合法性
禁止：起点超出地图范围时继续寻路

### Step2：成本计算器选择

输入：单位 GameplayTags
处理：TerrainCostRegistry.resolve_from_tags() 根据标签选择计算器
输出：TerrainCostCalculator 实例
禁止：跳过标签解析直接使用 ground 计算器

### Step3：BFS 可达范围计算

输入：起点、剩余移动力、TerrainGrid、OccupancyGrid、TerrainCostCalculator
处理：BFS 四方向扩展，计算每个格子的剩余移动力
输出：HashMap<IVec2, u32>（可达格子 → 剩余移动力）
禁止：忽略 OccupancyGrid 占用信息；忽略 TerrainCostCalculator 的 None 返回

### Step4：路径回溯

输入：目标坐标、可达范围 HashMap
处理：reconstruct_path 从目标回溯到起点
输出：Vec<IVec2> 路径序列（不含起点）
禁止：在不可达格子上回溯路径

---

## 移动执行管线

```
路径输入 → 逐步移动 → 消耗移动力 → OccupancyGrid 更新 → 到达终点
```

### Step1：路径验证

输入：寻路返回的路径、当前 OccupancyGrid
处理：验证路径上每个格子仍然可达（未被其他单位占据）
输出：有效路径或失效标记
禁止：跳过路径验证直接移动

### Step2：逐步移动

输入：有效路径
处理：沿路径逐步移动单位，每步更新 GridPosition
输出：单位位置变化
禁止：跳过中间格子直接传送

### Step3：移动力消耗

输入：路径长度和地形消耗
处理：根据 TerrainCostCalculator 逐步扣减移动力
输出：剩余移动力
禁止：不消耗移动力或消耗错误的值

### Step4：OccupancyGrid 更新

输入：单位旧位置和新位置
处理：OccupancyGrid.remove(旧位置) + OccupancyGrid.set(新位置, entity)
输出：占用表更新
禁止：不更新占用表

---

# 数据结构

## TerrainGrid（地形网格 Resource）

职责：存储每个坐标的地形 ID，是地形数据的唯一真相源

结构：
- width：地图宽度（格子数）
- height：地图高度（格子数）
- cells：HashMap — 坐标（IVec2）到地形 ID（String）的映射

要求：
- 从 LevelConfig 的 terrain_map 构建（from_terrain_map）
- get(coord) 返回地形 ID，未配置的格子默认 "plain"
- set(coord, terrain_id) 可修改地形（如技能改变地形）
- is_in_bounds(coord) 检查坐标合法性
- iter() 迭代所有格子供渲染使用

---

## OccupancyGrid（占用网格 Resource）

职责：记录每个坐标被哪个 Entity 占据，是单位占位的唯一真相源

结构：
- occupied：HashMap — 坐标（IVec2）到 Entity 的映射

要求：
- set(coord, entity) 设置占用
- remove(coord) 移除占用
- is_occupied(coord) 检查是否被占用
- is_occupied_except(coord, except) 排除自身检查占用（寻路用）
- rebuild(iter) 从所有单位位置重建占用表
- 每帧由 update_occupancy_grid 系统从 GridPosition 组件重建

---

## TerrainDef（地形定义）

职责：定义单种地形类型的属性（Definition，不可变）

结构：
- id：字符串 — 地形唯一标识（如 "plain"、"forest"）
- name：字符串 — 地形显示名称
- move_cost：可选值 — 基础移动消耗（None 表示不可通行）
- defense_bonus：整数 — 地形防御加成值
- color：元组 — 渲染颜色（RGB 0.0-1.0）
- passable：布尔 — 是否可通行
- char_code：可选字符 — 关卡网格中的字符代码

要求：
- 从 assets/terrains/*.ron 加载（TerrainDefRon → TerrainDef 转换）
- move_cost = 0 时转为 None（不可通行）
- 不可在战斗中修改

---

## TerrainCostCalculator（地形成本计算器 trait）

职责：描述不同单位类型的地形通行能力

结构：
- name() → 计算器名称（如 "ground"、"flying"）
- cost(terrain_id, base_cost) → Option<u32>（Some=可通行消耗 n，None=不可通行）

要求：
- 每种单位类型实现一个计算器
- 内置 4 种：GroundCostCalculator、FlyingCostCalculator、MountedCostCalculator、SwimmingCostCalculator
- 通过 TerrainCostRegistry 注册和查找
- resolve_from_tags() 根据 GameplayTag 选择计算器

---

## TerrainCostRegistry（地形成本注册表 Resource）

职责：管理所有地形成本计算器的注册和查找

结构：
- calculators：HashMap — 计算器名称到实例的映射

要求：
- 默认注册 4 种内置计算器
- register() 注册自定义计算器
- get(name) 按名称查找
- resolve_from_tags(tags) 按标签解析（优先级：SWIMMING > FLYING > MOUNTED > ground）
- ground() 获取默认计算器（带 fallback）

---

## GameMap（地图资源）

职责：存储地图尺寸和坐标转换方法

结构：
- width：地图宽度（格子数）
- height：地图高度（格子数）
- tile_size：格子尺寸（像素）

要求：
- from_level(level) 从 LevelConfig 创建
- coord_to_world(coord) 网格坐标转世界坐标
- world_to_coord(world) 世界坐标转网格坐标
- is_in_bounds(coord) 检查坐标合法性

---

## LevelConfigDef（关卡配置 Definition，RON 反序列化用）

职责：RON 文件中定义的完整关卡数据

结构：
- version：版本号
- id：关卡唯一标识
- name：关卡显示名称
- width / height：地图尺寸
- tile_size：格子尺寸
- terrain_grid：地形网格字符串列表
- char_map：自定义字符到地形 ID 映射
- player_units / enemy_units：单位部署列表
- victory_condition：可选胜负条件配置
- turn_limit：可选回合上限

要求：
- 从 assets/maps/*.ron 反序列化
- terrain_grid 每行一个字符串，字符长度必须等于 width
- 未配置 char_map 时使用 TerrainRegistry 的默认映射

---

## LevelConfig（关卡配置，运行时）

职责：从 LevelConfigDef 转换后的运行时关卡数据

结构：
- id：关卡唯一标识
- name：关卡显示名称
- width / height：地图尺寸
- tile_size：格子尺寸
- terrain_map：HashMap — 坐标（(i32,i32)）到地形 ID 的映射
- player_units / enemy_units：单位部署列表
- victory_condition：胜负条件配置
- turn_limit：回合上限

要求：
- 从 LevelConfigDef::from_def() 转换
- terrain_map 在战斗中不可修改（Definition）
- LevelRegistry 存储所有加载的 LevelConfig

---

# 禁止事项

禁止：为每个 Tile 生成携带地形数据的 ECS Entity

原因：Tile 是纯数据，不是游戏实体。生成 Entity 会导致寻路、渲染、战斗都依赖 ECS 查询，破坏数据与表现分离。

违反后果：寻路每帧遍历数百个 Tile Entity 查询地形数据，性能下降；修改地形需要同步 Entity 和数据两个来源，一致性难以保证。

---

禁止：在战斗中修改 TerrainDef 或 LevelConfig 的数据

原因：TerrainDef 和 LevelConfig 是 Definition（不可变配置），战斗中的数值修改应通过 Modifier 管线作用于 Instance。

违反后果：全局配置被污染、多场战斗数据不一致、热重载失效。

---

禁止：在 Execute 阶段直接读取 TerrainRegistry 修改伤害

原因：terrain_defense_bonus 必须在 Generate 阶段传入 EffectContext，Execute 阶段只执行不计算。

违反后果：伤害计算在管线中间被意外修改，Modify 阶段的修饰记录不包含地形影响，BattleRecord 数据不完整。

---

禁止：飞行单位忽略地形防御加成

原因：FlyingCostCalculator 只重写 move_cost（移动消耗），defense_bonus 是独立属性，不影响通行判断。

违反后果：飞行单位在 forest 地形上受到与平原相同的伤害，破坏战斗平衡。

---

禁止：跳过 OccupancyGrid 占用信息进行寻路

原因：OccupancyGrid 是单位占位的唯一真相源，跳过占用信息会导致寻路结果允许单位移动到已被占据的格子。

违反后果：两个单位移动到同一格子、寻路结果显示的可达格子实际已被占据。

---

禁止：寻路使用最大移动力而非当前剩余移动力

原因：移动范围必须反映单位当前状态（已移动的距离影响剩余移动力），使用最大移动力会扩展实际可达范围。

违反后果：单位在已经移动过的情况下仍能移动更远，破坏游戏平衡。

---

禁止：硬编码单位类型的地形通行逻辑

原因：地形通行能力必须通过 TerrainCostCalculator trait 数据驱动扩展，硬编码违反开闭原则。

违反后果：每次新增单位类型都需要修改寻路核心代码。

---

禁止：在关卡加载后修改 TerrainDef 的 move_cost 或 defense_bonus

原因：TerrainDef 是 Definition，战斗中的数值变化应通过 Modifier 管线作用于 Instance。直接修改 Definition 会污染全局数据。

违反后果：修改 forest.defense_bonus 后所有后续关卡的 forest 都使用新值，多场战斗数据不一致。

---

禁止：算法硬编码不抽象

原因：寻路算法必须通过 PathFinder trait 抽象，硬编码无法替换算法，违反开闭原则。

违反后果：新增算法（如 A*、Dijkstra）需要修改核心寻路代码，无法按场景选择最优算法。

---

禁止：无失效条件的缓存

原因：缓存数据会因单位移动、地形变化等原因过期，无失效条件会导致寻路结果错误。

违反后果：单位移动后缓存未失效，可达范围计算结果不准确，游戏逻辑错误。

---

禁止：无界缓存

原因：缓存无内存上限会导致内存泄漏，标准地图约 17KB 可接受，但无界增长会 OOM。

违反后果：长时间战斗后内存持续增长，最终导致内存溢出崩溃。

---

# AI 修改规则

## 如果新增地形类型

允许：
- 在 assets/terrains/*.ron 中添加新 RON 配置文件
- 在 LevelConfigDef 的 char_map 中添加新字符映射
- 在 TerrainCostCalculator 各实现中添加对新地形的处理

禁止：
- 修改 TerrainDef 的结构（新增字段需要 ADR）
- 在 TerrainRegistry::register_defaults() 中硬编码新地形（应从 RON 加载）
- 不更新 TerrainCostCalculator 就使用新地形

优先检查：
- TerrainDefRon 的 move_cost 和 defense_bonus 值是否合理
- 所有 TerrainCostCalculator 实现是否处理了新地形 ID
- char_map 映射是否正确
- 测试覆盖新地形的寻路行为

---

## 如果新增移动类型（如飞行、骑乘）

允许：
- 实现 TerrainCostCalculator trait 创建新计算器
- 在 TerrainCostRegistry 中注册新计算器
- 添加对应的 GameplayTag 用于标签解析

禁止：
- 修改现有 TerrainCostCalculator 的 cost() 语义
- 跳过 TerrainCostRegistry 直接调用新计算器
- 硬编码标签解析优先级

优先检查：
- 新计算器是否正确返回 None（不可通行）和 Some(n)（可通行消耗）
- GameplayTag 优先级是否正确（SWIMMING > FLYING > MOUNTED > 新类型 > ground）
- find_reachable_tiles 测试是否覆盖新计算器

---

## 如果修改伤害计算中的地形防御加成

允许：
- 调整 GenerateContext 中 terrain_defense_bonus 的查询逻辑
- 修改 calculate_damage_from_effect 中 terrain_defense_bonus 的使用方式

禁止：
- 在 Execute 阶段直接读取 TerrainRegistry
- 移除 terrain_defense_bonus 参与伤害计算
- 飞行单位跳过 terrain_defense_bonus

优先检查：
- Generate 阶段是否正确传入 defense_bonus
- 伤害公式中 terrain_defense_bonus 的减法顺序
- 飞行单位的 defense_bonus 是否按 TerrainDef 值正常参与
- 测试覆盖不同地形的伤害计算

---

## 如果修改寻路算法

允许：
- 修改 BFS 的扩展方向（如支持对角线移动）
- 调整 reconstruct_path 的回溯逻辑
- 优化可达范围计算性能

禁止：
- 忽略 OccupancyGrid 占用信息
- 忽略 TerrainCostCalculator 的通行判断
- 修改 find_reachable_tiles 的返回语义（HashMap<IVec2, u32>）

优先检查：
- 起点不在可达集合中（find_reachable_tiles 返回时 remove(&start)）
- 自身位置不算被占用（is_occupied_except）
- 被占据格子不可达
- 所有现有测试通过

---

## 如果新增寻路算法

允许：
- 实现 PathFinder trait 创建新算法（如 A*、Dijkstra）
- 在 PathFinderRegistry 中注册新算法
- 通过 RangeCalculator 选择算法

禁止：
- 算法硬编码不抽象（必须通过 trait）
- 在业务代码中直接实现 BFS/A* 逻辑
- 绕过 RangeCalculator 直接操作 TerrainGrid

优先检查：
- 新算法是否实现 PathFinder trait 的所有方法
- 新算法的 find_reachable_tiles 返回值语义是否一致
- 缓存键是否适配新算法
- 性能预算是否满足（移动范围 ≤ 2ms）
- 现有测试是否通过

---

## 如果修改缓存策略

允许：
- 调整缓存失效条件
- 优化缓存内存使用
- 修改缓存键结构

禁止：
- 移除缓存失效条件
- 无界缓存增长
- 缓存不命中时重新计算所有范围

优先检查：
- 所有失效条件是否覆盖（单位移动、地形变化、单位死亡、Buff 变化、装备变化、回合开始）
- 缓存内存是否在预算内（标准地图 ≤ 17KB）
- 缓存命中率是否 ≥ 80%

---

## 如果测试失败

排查顺序：
1. 检查 TerrainGrid 是否正确构建（from_terrain_map 是否解析了所有格子）
2. 检查 TerrainCostCalculator 是否正确返回 cost（None=不可通行，Some=可通行）
3. 检查 OccupancyGrid 是否正确更新（rebuild 是否从单位位置重建）
4. 检查 GameplayTag 解析优先级（SWIMMING > FLYING > MOUNTED > ground）
5. 检查 defense_bonus 是否正确传入 GenerateContext
6. 检查 find_reachable_tiles 是否排除了起点
