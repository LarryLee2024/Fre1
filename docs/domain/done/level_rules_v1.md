# 关卡配置 领域规则 v1.0

Version: 1.0
Status: Draft
Applies To: 关卡（Level）的配置定义、加载验证、战斗场景生成

---

## 1. 统一术语

| 术语 | 定义 | 职责边界 |
|------|------|----------|
| Level | 一个完整的战斗关卡，包含地图、单位部署、胜负条件、回合限制等全部配置 | 负责：关卡的完整数据描述；不负责：关卡运行时逻辑 |
| LevelConfig | Level 的运行时配置对象，从 RON 文件反序列化并转换后的不可变数据 | 负责：关卡数据的运行时表示；不负责：关卡数据的序列化存储 |
| LevelConfigDef | Level 的 RON 反序列化中间对象，使用字符串引用（如 template ID） | 负责：RON 文件解析；不负责：运行时使用 |
| LevelRegistry | 所有已加载关卡的注册表，全局唯一 | 负责：关卡的注册和查询；不负责：关卡的运行时状态 |
| Map | 战场的物理空间，由地形网格和单位占位组成 | 负责：地形数据和坐标系统；不负责：胜负判定或单位行为 |
| Battle | 一次完整的战斗过程，从关卡加载到胜负结算 | 负责：战斗流程和效果管线；不负责：关卡配置定义 |
| Stage | 关卡在战役流程中的位置标识（如"第一章第三关"） | 负责：关卡之间的串联关系；不负责：单个关卡内部逻辑 |
| UnitDeployDef | 单位部署条目，描述一个单位的模板 ID 和初始坐标 | 负责：单位出生点配置；不负责：单位属性或行为 |
| VictoryCondition | 关卡的胜利条件配置，描述玩家需要达成的目标 | 负责：胜利条件描述；不负责：条件检查逻辑（由 victory_condition_rules 领域负责） |
| TurnLimit | 关卡的回合上限，可选配置 | 负责：回合数限制描述；不负责：回合计数（由 turn 领域负责） |
| Reward | 关卡通关后的奖励配置，预留字段 | 负责：奖励描述；不负责：奖励发放逻辑 |

### 术语关系

```
Level
├── Map（地形数据：terrain_grid + width + height）
├── UnitDeployDef[]（玩家单位 + 敌方单位）
├── VictoryCondition（胜利条件）
├── TurnLimit?（可选回合限制）
├── Reward[]?（可选奖励配置）
└── Stage?（可选战役位置）

LevelRegistry
└── HashMap<id, LevelConfig>
```

### 与已有术语的关系

| 已有术语 | 所属领域 | 与 Level 的关系 |
|----------|----------|-----------------|
| TerrainGrid | map_rules | Level 提供 terrain_grid 数据，构建为 TerrainGrid |
| OccupancyGrid | map_rules | Level 提供 unit 部署坐标，初始化 OccupancyGrid |
| UnitTemplate | character_rules | Level 通过 template ID 引用 UnitTemplate |
| TurnPhase | turn_rules | Level 加载后进入 InGame 状态，激活 TurnPhase |
| GameOverState | turn（业务层 Resource） | Level 的 VictoryCondition 决定 GameOverState 的判定规则 |

---

## 2. 状态机

### 关卡生命周期

```
Loading
  │ [RON 文件读取成功]
  ▼
Validated
  │ [验证通过]
  ▼
Spawned
  │ [战斗开始，进入 InGame]
  ▼
Active
  │ [胜负条件达成]
  ▼
Completed

异常路径：
Loading ──[RON 解析失败]──► Failed
Validated ──[验证失败]──► Failed
```

状态列表：
- Loading：RON 文件正在读取和反序列化
- Validated：数据完整性验证通过
- Spawned：地形和单位已生成到战场
- Active：战斗进行中（TurnPhase 活跃）
- Completed：胜负条件已达成，战斗结束
- Failed：加载或验证失败

转换规则：
- Loading → Validated：RON 文件解析成功且数据完整性验证通过
- Validated → Spawned：地形网格构建完成，单位生成完成
- Spawned → Active：TurnPhase 进入 SelectUnit
- Active → Completed：VictoryCondition 判定为 Victory 或 Defeat
- Loading → Failed：RON 文件不存在、格式错误、或解析失败
- Validated → Failed：验证阶段发现不可修复的数据错误
- 禁止：Completed → Active（已结束的战斗不可重新激活）
- 禁止：Failed → Active（失败的关卡不可进入战斗）

---

## 3. 不变量（Invariants）

### 3.1 关卡 ID 全局唯一 🟥

- 条件：LevelRegistry 注册关卡时
- 不变量：每个关卡的 id 在 LevelRegistry 中唯一，不允许重复
- 违反后果：后注册的关卡覆盖先注册的关卡，查询结果不确定

### 3.2 关卡数据不可在运行时修改 🟥

- 条件：LevelConfig 加载到 LevelRegistry 后
- 不变量：LevelConfig 的所有字段在运行时不可修改（Definition 不可变原则）
- 违反后果：运行时修改导致关卡数据与 RON 配置不一致，破坏数据驱动原则
- 宪法依据：1.1.2（Definition / Instance 分离）

### 3.3 地形网格尺寸一致性 🟥

- 条件：LevelConfigDef 反序列化后
- 不变量：terrain_grid 的行数 = height，每行的字符数 = width
- 违反后果：地形数据与声明尺寸不匹配，地图渲染或寻路出错

### 3.4 单位部署坐标合法性 🟥

- 条件：LevelConfigDef 验证时
- 不变量：所有 UnitDeployDef 的 coord 必须在地图范围内（0 <= x < width, 0 <= y < height）
- 违反后果：单位生成在地图外，寻路和渲染异常

### 3.5 单位部署坐标不重叠 🟥

- 条件：LevelConfigDef 验证时
- 不变量：所有单位（玩家 + 敌方）的部署坐标互不重叠
- 违反后果：多个单位占据同一格子，OccupancyGrid 数据冲突

### 3.6 单位模板引用有效 🟥

- 条件：LevelConfigDef 验证时
- 不变量：所有 UnitDeployDef 的 template 必须在 UnitTemplateRegistry 中存在
- 违反后果：单位生成被跳过，关卡单位数量与预期不符

### 3.7 地形字符映射有效 🟩

- 条件：terrain_grid 解析时
- 不变量：terrain_grid 中的每个字符必须在 char_map（TerrainRegistry 或自定义）中有对应地形 ID
- 违反后果：未知字符回退为 "plain"（当前行为），但应记录警告

### 3.8 部署位置地形可通行 🟩

- 条件：UnitDeployDef 验证时
- 不变量：单位部署坐标对应的地形必须可通行（passable = true）
- 违反后果：单位生成在不可通行地形上，无法移动

### 3.9 关卡必须有胜利条件 🟥

- 条件：LevelConfigDef 验证时
- 不变量：victory_condition 字段必须存在且有效
- 违反后果：战斗无法判定胜负，游戏永远处于 Playing 状态
- 代码实现：`LevelConfigDef.victory_condition: Option<VictoryConditionDef>`，`None` 时使用默认 KillAll

### 3.10 LevelRegistry 空即为空 🟥

- 条件：关卡目录不存在或无 RON 文件时
- 不变量：LevelRegistry 为空，不创建假数据或兜底关卡
- 违反后果：关卡缺失时显示错误数据
- 宪法依据：1.1.5（数据驱动）

---

## 4. 禁止事项（Forbidden）

- 🟥 禁止：在运行时修改 LevelConfig 的任何字段 — 理由：LevelConfig 是 Definition，运行时不可变（宪法 1.1.2）
- 🟥 禁止：硬编码关卡数据 — 理由：所有关卡数据必须从 RON 文件加载（宪法 1.1.5）
- 🟥 禁止：绕过 LevelRegistry 直接创建 LevelConfig — 理由：所有关卡必须通过注册表统一管理
- 🟥 禁止：为不存在的关卡生成兜底数据 — 理由：LevelRegistry 空即为空（宪法 1.1.5）
- 🟥 禁止：关卡配置中引用不存在的模板 ID 时静默忽略 — 理由：引用校验失败必须记录错误日志
- 🟥 禁止：在关卡配置中存储运行时状态（如当前回合数、单位 HP） — 理由：Definition / Instance 分离
- 🟥 禁止：跳过数据验证直接进入战斗 — 理由：无效数据导致运行时崩溃
- 🟥 禁止：胜负条件硬编码在代码中 — 理由：胜负条件属于关卡配置内容，必须数据驱动（宪法 1.1.3）

---

## 5. 流程定义

### 5.1 关卡加载

- 输入：RON 文件路径（assets/maps/*.ron）+ TerrainRegistry
- 处理：
  1. 读取 RON 文件，反序列化为 LevelConfigDef
  2. 验证数据完整性（尺寸一致性、坐标合法性、模板引用有效性）
  3. 调用 LevelConfig::from_def()，将 terrain_grid 解析为 terrain_map
  4. 注册到 LevelRegistry
- 输出：LevelRegistry 中新增一条 LevelConfig 记录
- 失败处理：
  - RON 解析失败：记录错误日志，跳过该文件，继续加载其他关卡
  - 验证失败：记录错误日志，该关卡不注册到 LevelRegistry
  - 目录不存在：记录警告日志，返回空 LevelRegistry

### 5.2 关卡验证

- 输入：LevelConfigDef + TerrainRegistry + UnitTemplateRegistry
- 处理：
  1. 检查 terrain_grid 行数 = height
  2. 检查每行字符数 = width
  3. 检查所有 UnitDeployDef 坐标在地图范围内
  4. 检查所有单位部署坐标互不重叠
  5. 检查所有 template ID 在 UnitTemplateRegistry 中存在
  6. 检查地形字符在 char_map 中有对应地形 ID（无对应时记录警告，回退 plain）
  7. 检查部署位置地形可通行
  8. 检查 victory_condition 存在且有效
  9. 检查 turn_limit 合理性（如配置，必须 > 0）
- 输出：验证通过返回 Validated 状态；验证失败返回 Failed + 错误列表
- 失败处理：记录错误日志，关卡不注册

### 5.3 战斗场景生成

- 输入：LevelConfig（从 LevelRegistry 查询）
- 处理：
  1. 从 LevelConfig 构建 TerrainGrid（terrain_map → cells）
  2. 构建 GameMap（width、height、tile_size）
  3. 按 player_units 列表生成玩家单位（从 UnitTemplate 创建 Entity）
  4. 按 enemy_units 列表生成敌方单位
  5. 构建 OccupancyGrid（从所有单位的 GridPosition）
  6. 初始化 TurnOrder（按 Initiative 排序）
  7. 初始化 GameOverState 为 Playing
  8. 设置 AppState 为 InGame
- 输出：完整的战场初始状态（地图、单位、回合系统就绪）
- 失败处理：
  - 模板不存在：记录错误日志，跳过该单位
  - LevelRegistry 为空：不生成战场，保持 MainMenu 状态

### 5.4 关卡查询

- 输入：关卡 ID
- 处理：从 LevelRegistry 查询 LevelConfig
- 输出：LevelConfig 引用（只读）或 None
- 失败处理：返回 None，由调用方决定处理方式

---

## 6. 关卡配置字段定义

### 6.1 必要字段

| 字段 | 类型 | 说明 |
|------|------|------|
| id | String | 关卡唯一标识，全局唯一 |
| name | String | 关卡显示名称 |
| width | u32 | 地图宽度（格子数） |
| height | u32 | 地图高度（格子数） |
| terrain_grid | Vec&lt;String&gt; | 地形网格，每行一个字符串 |
| player_units | Vec&lt;UnitDeployDef&gt; | 玩家单位部署列表 |
| enemy_units | Vec&lt;UnitDeployDef&gt; | 敌方单位部署列表 |
| victory_condition | VictoryConditionDef | 胜利条件配置（引用 victory_condition_rules） |

### 6.2 可选字段

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| version | u32 | 0 | 配置版本号 |
| tile_size | f32 | 64.0 | 格子渲染尺寸 |
| char_map | HashMap&lt;char, String&gt; | 空 | 自定义字符→地形 ID 映射（覆盖 TerrainRegistry 默认值） |
| turn_limit | Option&lt;u32&gt; | None | 回合上限（None = 无限制） |
| rewards | Vec&lt;RewardDef&gt; | 空 | 通关奖励列表（预留） |
| terrain_bonuses | Vec&lt;TerrainBonusDef&gt; | 空 | 关卡特殊地形效果（预留） |

### 6.3 UnitDeployDef 字段

| 字段 | 类型 | 说明 |
|------|------|------|
| template | String | UnitTemplate ID 引用 |
| coord | (i32, i32) | 部署坐标 (x, y) |

---

## 7. 领域事件

| 事件名 | 触发时机 | 携带数据 | 订阅者 |
|--------|----------|----------|--------|
| LevelLoaded | 关卡 RON 文件解析成功并注册到 LevelRegistry | level_id, level_name | 调试面板 |
| LevelValidationFailed | 关卡数据验证失败 | level_id, 错误列表 | 调试面板 |
| BattleSceneReady | 战斗场景生成完毕，所有单位就绪 | level_id, 单位数量 | turn（初始化 TurnOrder）、ui（显示战斗界面） |
| LevelCompleted | 关卡胜负条件达成 | level_id, 结果（Victory/Defeat） | ui（显示结算界面）、battle_record（保存记录） |

---

## 8. 与已有领域规则的一致性检查

### 与 map_rules_v1 的关系

- LevelConfig 属于 map 领域的数据层（map_rules_v1 已定义 LevelConfig 和 LevelRegistry）
- 本文档扩展 LevelConfig 的字段定义（victory_condition、turn_limit、rewards），不修改已有字段
- TerrainGrid、OccupancyGrid、GameMap 的定义和规则不变

### 与 battle_rules_v1 的关系

- Level 提供战斗的初始配置（谁在哪、用什么），Battle 管理战斗过程
- Level 不负责 Effect Pipeline、CombatIntent 等战斗逻辑

### 与 turn_rules 的关系

- Level 的 turn_limit 引用 TurnState.turn_number 进行判定
- Level 加载后进入 InGame → TurnPhase 状态机激活

### DOMAIN CONFLICT 检查

- **无冲突**：本文档是对 map_rules_v1 中 LevelConfig 的扩展，不修改已有字段和规则

---

## 9. 宪法条款映射

| 宪法条款 | 本领域对应 |
|----------|-----------|
| 1.1.2 Definition/Instance 分离 | LevelConfig 是 Definition，运行时不可修改 |
| 1.1.3 Rule/Content 分离 | 胜负检查是规则，关卡配置是内容 |
| 1.1.5 数据驱动 | 所有关卡数据从 RON 加载，禁止硬编码 |
| 2.1.5 Resource 不是全局仓库 | LevelRegistry 有明确的注册和查询职责 |

---

## 10. 架构违规检测

| 违规行为 | 检测方式 | 输出 |
|----------|----------|------|
| 运行时修改 LevelConfig | 代码审查 | ARCHITECTURE VIOLATION: 运行时修改 LevelConfig，违反 Definition/Instance 分离原则。 |
| 硬编码关卡数据 | 代码审查 | ARCHITECTURE VIOLATION: 硬编码关卡数据，违反数据驱动原则。 |
| 胜负条件硬编码在代码中 | 代码审查 | ARCHITECTURE VIOLATION: 胜负条件硬编码在 [文件] 中，违反 Rule/Content 分离原则。应通过关卡配置的 victory_condition 字段实现。 |
| 跳过关卡验证直接生成战场 | 代码审查 | ARCHITECTURE VIOLATION: 跳过关卡验证直接生成战场，无效数据可能导致运行时崩溃。 |
