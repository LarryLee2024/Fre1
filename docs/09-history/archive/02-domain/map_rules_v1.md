---
id: history.archive.map_rules_v1
title: map_rules_v1
status: archived
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
superseded_by: ../../02-domain/map/map-terrain-rules.md
---

# Map 领域

Version: 1.1

Map 领域管理 SRPG 战场的地形数据、单位占位和寻路。地形数据与渲染分离，占用网格独立存在，寻路数据运行时生成。

核心原则：
- 🟩 地图优先看成 Grid 数据
- 🟥 地图数据与渲染分离（宪法 1.1.4）
- 🟥 OccupancyGrid 独立存在
- 🟩 寻路数据运行时生成
- 🟥 数据驱动（宪法 1.1.5）

---

# 术语定义

## TerrainGrid

地形网格，地形数据的唯一真相源。

不是 Tile Entity。TerrainGrid 是 Resource，不生成 Entity。

关键属性：
- width / height：地图尺寸
- cells：IVec2 → terrain_id 映射

---

## OccupancyGrid

占用网格，单位占位的唯一真相源。

不是 TerrainGrid。OccupancyGrid 记录"谁站在哪"，TerrainGrid 记录"地形是什么"。

关键属性：
- occupied：IVec2 → Entity 映射

---

## GameMap

地图元信息资源，提供坐标转换。

不是 TerrainGrid。GameMap 提供尺寸和坐标转换，TerrainGrid 提供地形数据。

关键属性：
- width / height / tile_size
- coord_to_world / world_to_coord

---

## TerrainDef

地形定义，描述地形属性。

不是 TerrainGrid。Def 是配置，Grid 是实例数据。

关键属性：
- id / name / move_cost / defense_bonus / passable / char_code

---

## TerrainCostCalculator

地形成本计算器，决定单位在不同地形上的移动消耗。

不是 TerrainDef。Calculator 是策略，Def 是数据。

关键属性：
- name()：计算器名称
- cost()：计算移动成本

---

# 领域边界

## 本领域负责

- TerrainGrid 和 OccupancyGrid 管理
- GameMap 坐标转换
- TerrainDef 定义和注册表（TerrainRegistry）
- LevelConfig 关卡配置和注册表（LevelRegistry）
- BFS 寻路（find_reachable_tiles / reconstruct_path）
- TerrainCostCalculator 注册表（TerrainCostRegistry）
- 地图渲染（spawn_map）

## 本领域不负责

- 单位移动动画（由 battle_rules 领域负责）
- 战斗逻辑（由 battle_rules 领域负责）
- 回合管理（由 turn_rules 领域负责）
- UI 交互（由 ui_rules 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 地形防御加成 | TerrainGrid.get() + TerrainRegistry | battle |
| 可达范围 | find_reachable_tiles 返回值 | battle |
| 占用信息 | OccupancyGrid 查询 | battle |
| 坐标转换 | GameMap 方法 | ui |

---

# 生命周期

## 地图生命周期

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Loaded | 关卡配置已加载 | Spawned |
| Spawned | 地形网格已生成，Sprite 已渲染 | Active |
| Active | 战斗进行中 | Cleared |
| Cleared | 战斗结束，地图清理 | — |

## 状态转换图

Loaded → Spawned → Active → Cleared

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Loaded | Spawned | spawn_map 系统执行 |
| Spawned | Active | 战斗开始 |
| Active | Cleared | 战斗结束 |

---

# 不变量

## 不变量1：TerrainGrid 是地形唯一真相源 🟥

宪法依据：2.1.5（禁止把 Resource 当全局变量仓库——但 TerrainGrid 是唯一真相源，不是仓库）

任意时刻：

所有地形查询必须通过 TerrainGrid，不存在其他地形数据副本。

违反表现：

Tile Entity 存储地形数据，与 TerrainGrid 不一致。

架构违规检测：

发现 Tile Entity 存储地形数据时，必须停止。必须输出：

```
ARCHITECTURE VIOLATION: Tile Entity 存储地形数据，违反 TerrainGrid 唯一真相源原则。
```

---

## 不变量2：OccupancyGrid 是占位唯一真相源 🟥

任意时刻：

所有占位查询必须通过 OccupancyGrid。

违反表现：

临时 HashMap 存储占位信息，与 OccupancyGrid 不一致。

---

## 不变量3：未配置格子默认 plain 🟩

TerrainGrid 构建完成后：

未在 cells 中配置的坐标返回 "plain"。

违反表现：

未配置格子返回空字符串或 panic。

---

## 不变量4：占用排除自身 🟥

find_reachable_tiles 执行时：

起始位置不被视为被占用（即使自身 Entity 在 OccupancyGrid 中）。

违反表现：

单位无法从当前位置开始寻路。

---

## 不变量5：成本计算器标签优先级 🟩

TerrainCostRegistry 解析时：

SWIMMING > FLYING > MOUNTED > 默认(ground)。

违反表现：

飞行单位使用地面成本计算器。

---

# 业务规则

## 规则1：地形数据与渲染分离 🟥

宪法依据：1.1.4（逻辑与表现强制分离）

禁止：
- 🟥 Tile Entity 存储地形数据
- 🟥 渲染系统修改 TerrainGrid

必须：
- TerrainGrid 存数据
- spawn_map 只画格子
- 地形属性从 TerrainRegistry 读取

---

## 规则2：寻路 🟥

禁止：
- 🟥 预存路径
- 🟥 跳过占用检查
- 🟥 跳过地形成本计算

必须：
- BFS 四方向扩展
- 成本 > 剩余移动力 → 跳过
- 被占用格子（自身除外）→ 跳过
- 已访问且剩余移动力更多 → 更新

允许：
- 🟩 起始位置不包含在结果中

---

## 规则3：路径回溯 🟩

禁止：
- target 不在 reachable 时返回空

必须：
- start == target → [target]
- target 不在 reachable → [target]
- 从 target 向 start 回溯，选择剩余移动力最大的前驱

---

## 规则4：成本计算器扩展 🟩

宪法依据：6.0.2（Trait 用于扩展点）

禁止：
- 🟥 match 分发单位类型

必须：
- 通过 TerrainCostCalculator trait 扩展
- 新增单位类型只需实现 Calculator 并注册
- 标签解析优先级：SWIMMING > FLYING > MOUNTED > ground

---

# 流程管线

## 地图生成管线

LevelConfig → TerrainGrid → Sprite 渲染

### Step1：构建 TerrainGrid

输入：LevelConfig.terrain_map / terrain_grid
处理：字符映射 → terrain_id，填入 cells
输出：TerrainGrid
🟥 禁止：生成 Tile Entity

### Step2：渲染 Sprite

输入：TerrainGrid + TerrainRegistry
处理：遍历 cells，生成 TileSprite
输出：渲染的地图
🟥 禁止：修改 TerrainGrid

---

## 寻路管线

起始位置 → BFS 扩展 → 可达范围 → 路径回溯

### Step1：BFS 扩展

输入：start + move_points + TerrainGrid + OccupancyGrid + Calculator
处理：四方向扩展，计算成本，检查占用
输出：HashMap<IVec2, u32>（坐标 → 剩余移动力）
🟥 禁止：跳过占用检查、跳过成本计算

### Step2：路径回溯

输入：start + target + reachable
处理：从 target 回溯到 start
输出：Vec<IVec2>（路径坐标序列）
🟩 禁止：target 不在 reachable 时返回空

---

# 数据结构

## TerrainGrid（Resource）

职责：地形数据的唯一真相源

结构：
- width / height：地图尺寸
- cells：HashMap<IVec2, String>

要求：
- 🟩 未配置格子返回 "plain"
- 🟥 不生成 Tile Entity

---

## OccupancyGrid（Resource）

职责：单位占位的唯一真相源

结构：
- occupied：HashMap<IVec2, Entity>

要求：
- 🟩 每帧从单位位置 rebuild
- 🟥 寻路时排除自身

---

## GameMap（Resource）

职责：地图元信息和坐标转换

结构：
- width / height / tile_size

要求：
- 🟩 coord_to_world / world_to_coord 转换
- 🟩 网格原点左下角

---

## TerrainDef（Definition）

职责：地形属性定义

结构：
- id / name / move_cost / defense_bonus / passable / char_code / color

要求：
- 🟩 move_cost = None 表示不可通行
- 🟥 RON 配置路径：assets/terrains/（宪法 1.1.5）

---

## TerrainCostCalculator（Trait）

职责：地形成本计算策略

结构：
- name()：计算器名称
- cost()：计算移动成本

要求：
- 🟩 内置四种计算器
- 🟩 标签解析优先级

---

# 禁止事项

🟥 禁止：Tile Entity 存储地形数据

原因：TerrainGrid 是唯一真相源

违反后果：地形数据不一致，渲染与逻辑脱节

架构违规检测：

```
ARCHITECTURE VIOLATION: Tile Entity 存储地形数据，违反 TerrainGrid 唯一真相源原则。
```

---

🟥 禁止：预存寻路路径

原因：路径依赖实时占用状态，必须运行时计算

违反后果：路径与实际占用状态不一致

---

🟥 禁止：跳过占用检查

原因：单位不能穿过其他单位

违反后果：单位重叠在同一格子

---

🟥 禁止：渲染系统修改 TerrainGrid

原因：渲染只读数据，不修改数据（宪法 1.1.4）

违反后果：渲染副作用导致地形数据被意外修改

---

🟥 禁止：关卡硬编码兜底

原因：LevelRegistry 空即为空，不创建假数据（宪法 1.1.5 数据驱动）

违反后果：关卡缺失时显示错误数据

---

🟥 禁止：match 分发单位类型计算成本

原因：宪法 6.0.2 要求通过 Trait 扩展，禁止 match 分发

违反后果：新增单位类型需修改寻路代码

---

# AI 修改规则

## 如果新增地形类型

允许：
- 新增 TerrainDef RON 配置
- 新增 char_code 映射

禁止：
- 🟥 修改 TerrainGrid 结构
- 🟥 修改 OccupancyGrid 结构
- 🟥 修改 BFS 算法

优先检查：
- TerrainRegistry 注册
- char_code 是否冲突
- move_cost 和 defense_bonus 数值平衡

---

## 如果新增移动类型

允许：
- 新增 TerrainCostCalculator 实现
- 注册到 TerrainCostRegistry

禁止：
- 🟥 修改 BFS 算法
- 🟩 修改标签解析优先级（除非必要）

优先检查：
- TerrainCostRegistry 注册
- 标签解析优先级
- 各地形成本值

---

## 如果新增关卡

允许：
- 新增 LevelConfig RON 配置

禁止：
- 🟥 硬编码关卡数据

优先检查：
- terrain_grid 字符与 TerrainDef.char_code 对应
- 部署坐标在地图范围内
- 单位模板 ID 存在

---

## 如果测试失败

排查顺序：
1. 检查 TerrainGrid 是否正确构建
2. 检查 OccupancyGrid 是否正确 rebuild
3. 检查成本计算器是否正确选择
4. 检查 BFS 是否跳过占用格子
5. 检查路径回溯是否正确

测试要求（宪法 13.0.1-13.0.3）：
- 🟩 单元测试：验证寻路算法正确性
- 🟩 集成测试：验证地图生成和渲染
- 🟩 Bug 修复必须先编写重现测试（宪法 13.0.2）

---

# 宪法条款映射

| 宪法条款 | 本领域对应 |
|----------|-----------|
| 1.1.4 逻辑与表现分离 | TerrainGrid 存数据，spawn_map 只渲染 |
| 1.1.5 数据驱动 | TerrainDef / LevelConfig 从 RON 加载 |
| 2.1.5 Resource 不是全局仓库 | TerrainGrid/OccupancyGrid 是唯一真相源 |
| 6.0.2 Trait 用于扩展点 | TerrainCostCalculator trait |
| 1.1.3 Rule/Content 分离 | BFS 是规则，地形配置是内容 |

---

# 架构违规检测

| 违规行为 | 检测方式 | 输出 |
|----------|----------|------|
| Tile Entity 存储地形数据 | 代码审查 | ARCHITECTURE VIOLATION: Tile Entity 存储地形数据，违反 TerrainGrid 唯一真相源原则。 |
| 渲染系统修改 TerrainGrid | 代码审查 | ARCHITECTURE VIOLATION: 渲染系统修改 TerrainGrid，违反逻辑与表现分离原则。 |
| match 分发单位类型 | 代码审查 | ARCHITECTURE VIOLATION: match 分发单位类型计算成本，违反 Trait 扩展原则。 |
