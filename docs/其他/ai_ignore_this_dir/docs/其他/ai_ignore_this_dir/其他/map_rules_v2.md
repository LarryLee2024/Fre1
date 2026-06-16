# Map 领域

Version: 2.0

## Purpose

Map 领域管理 SRPG 战场的地形数据、单位占位和寻路。地形数据与渲染分离，占用网格独立存在，寻路数据运行时生成。

---

## Glossary

| 术语 | 定义 | 易混淆项 |
|------|------|----------|
| TerrainGrid | 地形网格，地形数据的唯一真相源 | ≠ Tile Entity：Grid 是 Resource，不生成 Entity |
| OccupancyGrid | 占用网格，单位占位的唯一真相源 | ≠ TerrainGrid：OccupancyGrid 记录"谁站在哪"，TerrainGrid 记录"地形是什么" |
| GameMap | 地图元信息资源，提供坐标转换 | ≠ TerrainGrid：GameMap 提供尺寸和转换，TerrainGrid 提供地形数据 |
| TerrainDef | 地形定义，描述地形属性 | ≠ TerrainGrid：Def 是配置，Grid 是实例数据 |
| TerrainCostCalculator | 地形成本计算器，决定单位在不同地形上的移动消耗 | ≠ TerrainDef：Calculator 是策略，Def 是数据 |

---

## Responsibilities

### Owns

- TerrainGrid 和 OccupancyGrid 管理
- GameMap 坐标转换
- TerrainDef 定义和注册表
- LevelConfig 关卡配置和注册表
- BFS 寻路
- TerrainCostCalculator 注册表
- 地图渲染（spawn_map）

### Does Not Own

- 单位移动动画 → battle_rules
- 战斗逻辑 → battle_rules
- 回合管理 → turn_rules
- UI 交互 → ui_rules

---

## Invariants

### INV-MAP-01：TerrainGrid 是地形唯一真相源 🟥

宪法：2.1.5

所有地形查询必须通过 TerrainGrid，不存在其他地形数据副本。禁止 Tile Entity 存储地形数据。

违反：Tile Entity 存储地形数据，与 TerrainGrid 不一致。

### INV-MAP-02：OccupancyGrid 是占位唯一真相源 🟥

所有占位查询必须通过 OccupancyGrid。

违反：临时 HashMap 存储占位信息，与 OccupancyGrid 不一致。

### INV-MAP-03：逻辑与表现分离 🟥

宪法：1.1.4

TerrainGrid 存数据，spawn_map 只渲染。渲染系统禁止修改 TerrainGrid。

违反：渲染副作用导致地形数据被意外修改。

### INV-MAP-04：占用排除自身 🟥

find_reachable_tiles 执行时，起始位置不被视为被占用（即使自身 Entity 在 OccupancyGrid 中）。

违反：单位无法从当前位置开始寻路。

### INV-MAP-05：Calculator 通过 Trait 分发 🟥

宪法：6.0.2

地形成本计算通过 TerrainCostCalculator trait 分发，禁止 match 分发单位类型。

违反：新增单位类型需修改寻路代码。

### INV-MAP-06：未配置格子默认 plain 🟩

未在 cells 中配置的坐标返回 "plain"。

违反：未配置格子返回空字符串或 panic。

### INV-MAP-07：成本计算器标签优先级 🟩

SWIMMING > FLYING > MOUNTED > 默认(ground)。

违反：飞行单位使用地面成本计算器。

---

## State Machine

### 地图生命周期

| 状态 | 含义 | 转换到 |
|------|------|--------|
| Loaded | 关卡配置已加载 | Spawned |
| Spawned | 地形网格已生成，Sprite 已渲染 | Active |
| Active | 战斗进行中 | Cleared |
| Cleared | 战斗结束，地图清理 | — |

```
Loaded → Spawned → Active → Cleared
```

---

## Business Rules

### BR-MAP-01：寻路

- BFS 四方向扩展
- 成本 > 剩余移动力 → 跳过
- 被占用格子（自身除外）→ 跳过
- 已访问且剩余移动力更多 → 更新
- 起始位置不包含在结果中
- 禁止预存路径

### BR-MAP-02：路径回溯

- start == target → [target]
- target 不在 reachable → [target]
- 从 target 向 start 回溯，选择剩余移动力最大的前驱

### BR-MAP-03：成本计算器扩展

- 通过 TerrainCostCalculator trait 扩展
- 新增单位类型只需实现 Calculator 并注册
- 标签解析优先级：SWIMMING > FLYING > MOUNTED > ground

### BR-MAP-04：关卡配置

- LevelRegistry 空即为空，不创建假数据
- 地形配置从 RON 加载
- 禁止硬编码关卡数据

---

## Pipelines

### 地图生成管线

LevelConfig → TerrainGrid → Sprite 渲染

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 构建 TerrainGrid | LevelConfig.terrain_map | TerrainGrid | 禁止生成 Tile Entity（INV-MAP-01） |
| 渲染 Sprite | TerrainGrid + TerrainRegistry | 渲染的地图 | 禁止修改 TerrainGrid（INV-MAP-03） |

### 寻路管线

起始位置 → BFS 扩展 → 可达范围 → 路径回溯

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| BFS 扩展 | start + move_points + TerrainGrid + OccupancyGrid + Calculator | HashMap<IVec2, u32> | 禁止跳过占用检查、禁止跳过成本计算 |
| 路径回溯 | start + target + reachable | Vec<IVec2> | target 不在 reachable 时返回 [target] |

---

## Data Model

### TerrainGrid（Resource）

地形数据的唯一真相源。

- width / height：地图尺寸
- cells：HashMap<IVec2, String>

### OccupancyGrid（Resource）

单位占位的唯一真相源。

- occupied：HashMap<IVec2, Entity>
- 每帧从单位位置 rebuild

### GameMap（Resource）

地图元信息和坐标转换。

- width / height / tile_size
- coord_to_world / world_to_coord

### TerrainDef（Definition）

地形属性定义，不可变。

- id / name / move_cost / defense_bonus / passable / char_code / color
- move_cost = None 表示不可通行
- 配置来源：RON（assets/terrains/）

### TerrainCostCalculator（Trait）

地形成本计算策略。

- 内置四种计算器
- 标签解析优先级

---

## Cross Domain Contracts

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 地形防御加成 | TerrainGrid.get() + TerrainRegistry | battle |
| 可达范围 | find_reachable_tiles 返回值 | battle |
| 占用信息 | OccupancyGrid 查询 | battle |
| 坐标转换 | GameMap 方法 | ui |

---

## Change Rules

### 新增地形类型

- 允许：新增 TerrainDef RON 配置 + 新增 char_code 映射
- 禁止：修改 TerrainGrid 结构、修改 OccupancyGrid 结构、修改 BFS 算法
- 检查：TerrainRegistry 注册、char_code 是否冲突、move_cost 和 defense_balance

### 新增移动类型

- 允许：新增 TerrainCostCalculator 实现 + 注册到 TerrainCostRegistry
- 禁止：修改 BFS 算法
- 检查：TerrainCostRegistry 注册、标签解析优先级、各地形成本值

### 新增关卡

- 允许：新增 LevelConfig RON 配置
- 禁止：硬编码关卡数据
- 检查：terrain_grid 字符与 TerrainDef.char_code 对应、部署坐标在地图范围内、单位模板 ID 存在

---

## Architecture Violations

发现架构违规时统一输出：

```
ARCHITECTURE VIOLATION:
Rule: <RuleID>
Reason: <Why>
Fix: <How>
```

| RuleID | 违规行为 | Reason | Fix |
|--------|----------|--------|-----|
| INV-MAP-01 | Tile Entity 存储地形数据 | TerrainGrid 是唯一真相源 | 改为从 TerrainGrid 查询 |
| INV-MAP-03 | 渲染系统修改 TerrainGrid | 逻辑与表现分离 | 渲染只读，不修改 |
| INV-MAP-05 | match 分发单位类型 | 应通过 Trait 扩展 | 改为 TerrainCostCalculator trait |

---

## Test Requirements

宪法：13.0.1-13.0.3

- 单元测试：验证寻路算法正确性
- 集成测试：验证地图生成和渲染
- Bug 修复必须先编写重现测试

排查顺序：
1. TerrainGrid 是否正确构建
2. OccupancyGrid 是否正确 rebuild
3. 成本计算器是否正确选择
4. BFS 是否跳过占用格子
5. 路径回溯是否正确
