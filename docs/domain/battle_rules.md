# 战斗系统领域

Version: 1.0
Status: Proposed

战斗系统领域管理战斗的整体生命周期、胜负条件判定、战斗终态管理和 Effect Pipeline 执行。

核心原则：
- 战斗状态机驱动全局流程，终态不可逆
- 胜负条件数据驱动、可组合，失败优先于胜利
- 所有战斗效果必须经过 Effect Pipeline，禁止绕过管线

---

# 术语定义

## 战斗（Battle）

从玩家进入关卡到胜利/失败判定的完整战斗过程。由 `AppState::InGame` 表示，包含回合循环直到终态。

不是单个回合。不是单个单位的行动。

关键属性：
- 一场战斗包含多个回合（Round），参见 `turn_rules.md#回合（Round）`
- 战斗由 `AppState::InGame` 驱动，终态为 `GameOver`
- 战斗进入时调用 `init_turn_order` 初始化行动队列

---

## 战斗终态（GameOverState）

战斗的最终结果状态，从 `Playing` 不可逆转换为 `Victory` 或 `Defeat`。

不是回合阶段。不是单位状态。

关键属性：
- `Playing` → `Victory` 或 `Playing` → `Defeat` 均为不可逆转换
- 终态后所有后续检查直接返回，不做任何修改
- 由 `check_victory_conditions` 系统在 TurnEnd 阶段写入
- 由 `check_all_dead_safety` 兜底系统每帧防御性保障

---

## 胜利条件（VictoryCondition）

从关卡配置（RON）数据驱动读取的可组合规则，决定玩家何时获胜。包含 `win_conditions` 列表，使用 OR 逻辑。

不是硬编码逻辑。不是单一固定条件。

关键属性：
- 由 `VictoryConditionDef` 定义，从 `LevelConfig.victory_condition` 读取
- 多个条件之间为 OR 关系：任一满足即胜利
- 支持的类型：KillAll（全灭敌方）、SurviveTurns（存活 N 回合）、DefeatBoss（击败 Boss）

---

## 失败条件（DefeatCondition）

从关卡配置数据驱动读取的可组合规则，决定玩家何时失败。包含 `lose_conditions` 列表，使用 OR 逻辑。

不是胜利条件的反面。不是可选配置。

关键属性：
- 由 `LoseConditionDef` 定义
- 多个条件之间为 OR 关系：任一满足即失败
- 支持的类型：AllDead（全灭玩家）、TurnLimitExceeded（超时）
- 默认全灭检查是绝对不变量，即使未配置也会生效

---

## 胜负判定管线（Victory Check Pipeline）

从终态检查到最终判定的五步严格管线，仅在 TurnEnd 阶段执行。

不是回合结算流程。不是伤害计算管线。

关键属性：
- 五步顺序：终态检查 → 失败条件 → 默认全灭 → 胜利条件 → 终态判定
- 失败条件先于胜利条件检查
- 默认全灭检查不可移除
- 终态不可逆，已为终态时直接返回

---

## CombatIntent（战斗意图）

传递玩家或 AI 的攻击意图到 Effect Pipeline 的 Resource。

不是攻击指令。不是技能效果。

关键属性：
- source_entity：发起攻击的单位
- target_coord：目标坐标
- skill_id：使用的技能 ID
- AI 和玩家共用同一 CombatIntent 资源
- 执行完毕后必须清除 CombatIntent

---

## LevelCompleted

战斗结束时发送的消息，通知 UI 展示结算界面。

不是状态。不是事件。

关键属性：
- 在终态（Victory/Defeat）达成后发送
- 携带战斗结果信息
- UI 层消费此消息展示结算界面

---

## 战斗阶段（BattlePhase）

战斗的宏观生命周期状态机，管理从战斗初始化到结束的 8 个阶段。映射为 Bevy State（SubState of AppState::InGame）。

不是回合阶段（TurnPhase）。不是 AppState。

关键属性：
- 8 阶段枚举：PreBattle、RoundStart、PlayerPhase、EnemyPhase、TurnEnd、VictoryCheck、RoundEnd、PostBattle
- BattlePhase 是顶层 FSM，TurnPhase 是 PlayerPhase/EnemyPhase 内的 SubState
- 通过 NextState 驱动转换，禁止手动设置
- 每个阶段有 OnEnter/OnExit 钩子系统
- 来源：`docs/architecture/battle_fsm_design.md`

---

# 领域边界

## 本领域负责

- 战斗状态机（AppState：MainMenu → LevelSelect → InGame → GameOver）
- 战斗阶段状态机（BattlePhase：PreBattle → RoundStart → ... → PostBattle）
- 战斗终态管理（GameOverState：Playing → Victory/Defeat，不可逆）
- 胜负条件判定（数据驱动，OR 逻辑，失败优先）
- 战斗生命周期（BattleStart → RoundStart → TurnLoop → TurnEnd → VictoryCheck → RoundEnd → BattleEnd）
- Effect Pipeline 执行（Generate → Modify → Execute）
- CombatIntent 攻击意图管理
- LevelCompleted 消息广播

## 本领域不负责

- 回合阶段状态机（由 Turn 领域负责：TurnPhase SubState）
- 行动顺序编排（由 Turn 领域负责：TurnOrder 队列）
- 回合生命周期管理（由 Turn 领域负责：回合号递增、队列重建、acted 重置）
- 回合消息广播（由 Turn 领域负责：TurnStarted、TurnEnded）
- 属性值的计算与修饰（由 Core 属性系统负责）
- Buff/Debuff 持续效果的具体结算逻辑（由 Buff 领域负责）
- 单位的具体移动和寻路（由 Map 领域负责）
- 用户输入处理（由 Input 领域负责）
- UI 展示与交互（由 UI 领域负责）
- AI 策略选择与行为定义（由 AI 领域负责）
- 关卡地图数据加载（由 Map 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 战斗终态达成 | Message（LevelCompleted） | UI（结算界面）、Infrastructure（日志） |
| 攻击意图 | Resource（CombatIntent） | Battle（Effect Pipeline） |
| 回合开始/结束 | Observer（TurnStarted / TurnEnded） | Turn（记录回合号）、UI（回合指示器） |
| 伤害/治疗效果 | Message（DamageApplied / HealApplied） | UI（战斗日志、VFX） |
| 单位死亡 | Message（CharacterDied） | Turn（移除队列）、UI（日志） |

---

# 生命周期

## 状态列表

### AppState（游戏主状态）

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| MainMenu | 主菜单（初始状态） | LevelSelect |
| LevelSelect | 关卡选择 | InGame |
| InGame | 战斗进行中 | GameOver |
| GameOver | 战斗结束 | MainMenu |

### GameOverState（战斗终态 Resource）

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Playing | 战斗进行中（初始） | Victory, Defeat |
| Victory | 玩家胜利（终态） | 无（不可逆） |
| Defeat | 玩家失败（终态） | 无（不可逆） |

### BattlePhase（战斗阶段 FSM，SubState of AppState::InGame）

| 阶段 | 含义 | 可转换到 |
|------|------|----------|
| PreBattle | 战斗初始化：加载关卡、生成地图、部署单位 | RoundStart |
| RoundStart | 回合开始：重建行动队列、发送 TurnStarted | PlayerPhase |
| PlayerPhase | 玩家行动阶段（TurnPhase SubState 激活） | EnemyPhase |
| EnemyPhase | 敌方行动阶段（AI 执行） | TurnEnd |
| TurnEnd | 回合结算：胜负检查、回合号递增、acted 重置 | VictoryCheck |
| VictoryCheck | 胜负判定：检查所有胜利/失败条件 | RoundEnd, PostBattle |
| RoundEnd | 回合结束收尾：发送 TurnEnded、准备下一回合 | RoundStart |
| PostBattle | 战斗结束：发送 LevelCompleted、切换到 GameOver | 无（终态，切 AppState） |

#### 合法转换表

| 从 | 到 | 条件 |
|----|-----|------|
| PreBattle | RoundStart | 关卡初始化完成 |
| RoundStart | PlayerPhase | 回合开始完成 |
| PlayerPhase | EnemyPhase | 玩家方所有单位行动完毕或 ForceEndTurn |
| EnemyPhase | TurnEnd | 敌方所有单位行动完毕 |
| TurnEnd | VictoryCheck | 回合结算完成 |
| VictoryCheck | RoundEnd | 未达到终态 |
| VictoryCheck | PostBattle | 终态达成（Victory/Defeat） |
| RoundEnd | RoundStart | 新回合开始 |

#### 非法转换（禁止）

| 从 | 到 | 禁止原因 |
|----|-----|---------|
| PreBattle | PlayerPhase | 跳过 RoundStart，行动队列未初始化 |
| PlayerPhase | TurnEnd | 跳过 EnemyPhase，敌方未行动 |
| PlayerPhase | VictoryCheck | 跳过回合结算 |
| PlayerPhase | PostBattle | 跳过所有后续阶段 |
| RoundEnd | PlayerPhase | 跳过 RoundStart，队列未重建 |
| TurnEnd | RoundStart | 跳过 VictoryCheck，胜负未判定 |
| PostBattle | 任意 | 终态不可逆 |

## 状态转换图

```
AppState:
MainMenu → LevelSelect → InGame → GameOver
                                ↓
                           OnExit → cleanup → MainMenu

GameOverState:
Playing → Victory（不可逆）
Playing → Defeat（不可逆）

BattlePhase:
PreBattle → RoundStart → PlayerPhase → EnemyPhase → TurnEnd → VictoryCheck → RoundEnd
                ↑                                                              │
                └──────────────────────────────────────────────────────────────┘
TurnEnd → VictoryCheck → PostBattle（终态达成时）
VictoryCheck → RoundEnd（未终态时）
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| AppState::MainMenu | LevelSelect | 玩家选择开始游戏 |
| LevelSelect | InGame | 选择关卡，加载关卡数据，生成地图和单位 |
| InGame | GameOver | GameOverState 变为 Victory 或 Defeat |
| GameOver | MainMenu | 玩家选择返回主菜单 |
| GameOverState::Playing | Victory | 胜利条件满足且未先触发失败 |
| GameOverState::Playing | Defeat | 失败条件满足或全灭玩家 |

---

# 不变量

## 不变量1：全灭玩家即失败

任意时刻：

只要没有任何 `Faction::Player` 的存活单位（排除 `Dead` 组件），`GameOverState` 必须为 `Defeat`。

违反表现：

所有玩家单位死亡后，`GameOverState` 仍为 `Playing`，游戏继续进行。

---

## 不变量2：失败优先于胜利

回合结算阶段：

当失败条件和胜利条件同时满足时，`GameOverState` 必须为 `Defeat`，不是 `Victory`。

违反表现：

全灭双方单位时，`GameOverState` 被设为 `Victory` 而非 `Defeat`。

---

## 不变量3：终态不可逆

回合结算阶段及每帧检查：

`GameOverState` 为 `Victory` 或 `Defeat` 后，后续所有检查必须直接返回，不做任何修改。

违反表现：

已判定 `Victory` 后，下一帧被覆盖为 `Defeat`；或已判定 `Defeat` 后被恢复为 `Playing`。

---

## 不变量4：默认全灭检查不可移除

任意时刻：

即使关卡配置的 `lose_conditions` 中没有显式配置 `AllDead` 类型，"全灭玩家即失败"检查仍然必须生效。

违反表现：

关卡未配置 `AllDead` 条件时，全灭玩家不触发失败。

---

## 不变量5：胜负检查仅在 TurnEnd 阶段执行

回合生命周期：

响应式胜负检查（`check_victory_conditions`）仅在 `OnEnter(TurnPhase::TurnEnd)` 时执行，不在其他阶段执行。参见 `turn_rules.md#回合结算（TurnEnd Phase）`。

违反表现：

在 `SelectUnit` 或 `ExecuteAction` 阶段提前检查胜负条件导致过早结束战斗。

---

## 不变量6：PreBattle 必须在所有战斗逻辑之前

任意时刻：

任何战斗逻辑执行前，BattlePhase 必须已经过 PreBattle 阶段。禁止直接从 MainMenu 跳到 PlayerPhase。

违反表现：

关卡初始化未完成就开始玩家行动，行动队列未创建，单位未部署。

---

## 不变量7：PlayerPhase 和 EnemyPhase 互斥

任意时刻：

同一时刻只能处于 PlayerPhase 或 EnemyPhase 之一，不可同时激活。

违反表现：

玩家和敌方同时执行行动逻辑，回合顺序混乱，单位状态竞争。

---

## 不变量8：每个回合必须完成完整循环

任意时刻：

每个回合必须完成 RoundStart → PlayerPhase → EnemyPhase → TurnEnd → VictoryCheck → RoundEnd 完整循环。除终态时 VictoryCheck → PostBattle 外，禁止跳过任何阶段。

违反表现：

跳过 VictoryCheck 导致胜负未判定；跳过 TurnEnd 导致 acted 未重置，单位无法行动；跳过 RoundStart 导致队列未重建。

---

# 业务规则

## 规则1：胜负条件组合

禁止：
- 硬编码胜负条件逻辑
- 修改数据驱动的 ConditionTypeDef 枚举
- 在 TurnEnd 以外的阶段检查胜负条件

必须：
- 多个胜利条件之间使用 OR 逻辑
- 多个失败条件之间使用 OR 逻辑
- 失败条件先于胜利条件检查
- 终态（Victory/Defeat）达成后发送 LevelCompleted 消息

允许：
- 关卡配置中不设置 `victory_condition`（回退到默认全灭检查）
- 关卡配置中设置多个 win_conditions 和 lose_conditions

---

## 规则2：Effect Pipeline 执行

禁止：
- 跳过 Generate → Modify → Execute 三步管线
- 在 Generate 阶段直接扣血
- AI 绕过 Effect Pipeline 独立计算伤害

必须：
- 玩家和 AI 共用同一 Effect Pipeline
- CombatIntent 是唯一攻击意图通道
- 伤害下限 ≥ 1
- 治疗下限 ≥ 0
- 所有修饰记录写入 BattleRecord

允许：
- 通过 EffectHandler trait 扩展新的效果类型
- 通过 TraitTrigger 枚举扩展新的触发时机

---

## 规则3：BattlePhase OnEnter/OnExit 钩子

禁止：
- 在 PreBattle、RoundStart、TurnEnd 的 OnEnter 中修改单位属性
- 在 OnEnter 中执行跨阶段跳转（破坏状态机确定性）
- 在 OnExit 中修改业务逻辑状态

必须：
- OnEnter 职责：初始化系统、发送消息、触发特质效果
- OnExit 职责：清理高亮、清理 AI 状态、清理战斗资源
- 钩子通过 Plugin 在扩展点注入，禁止在 FSM 内部硬编码

允许：
- OnEnter(RoundStart)：发送消息、记录日志、触发 Trait 效果
- OnExit(TurnEnd)：Buff 过期检查、持续效果结算
- OnEnter(VictoryCheck)：准备判定数据
- OnEnter(PostBattle)：播放结算动画、保存结果

扩展点位置：

| 扩展点 | 位置 | 允许行为 |
|--------|------|---------|
| OnEnter(RoundStart) | 回合开始 | 发送消息、记录日志、触发 Trait 效果 |
| OnExit(TurnEnd) | 回合结束 | Buff 过期检查、持续效果结算 |
| OnEnter(VictoryCheck) | 胜负判定前 | 准备判定数据 |
| OnEnter(PostBattle) | 战斗结束 | 播放结算动画、保存结果 |

---

# 流程管线

## Battle Lifecycle Pipeline：战斗生命周期管线

```
BattleStart → RoundStart → TurnLoop → TurnEnd → VictoryCheck → RoundEnd → BattleEnd
```

### BattleStart：战斗初始化

输入：关卡配置（LevelConfig）、地形数据
处理：加载关卡、生成地图、部署单位、初始化行动队列
输出：GameState::InGame、TurnOrder 队列
禁止：在初始化时检查胜负条件

### RoundStart：回合开始

输入：上一回合的 TurnOrder（或初始队列）
处理：发送 TurnStarted 消息，记录回合号
输出：回合日志
禁止：修改单位状态

### TurnLoop：单位行动循环

输入：TurnOrder 队列、当前单位
处理：按队列顺序执行 SelectUnit → MoveUnit → ActionMenu → SelectTarget → ExecuteAction（参见 `turn_rules.md#Turn Execution Pipeline`）
输出：单位状态变化、战斗日志
禁止：在循环中修改行动队列顺序

### TurnEnd：回合结算

输入：当前回合状态
处理：检查胜负条件、递增回合号、重置 acted、重建队列、设置 NeedsResolve
输出：新的 TurnOrder、可能的终态
禁止：执行攻击逻辑、跳过任何结算步骤

### VictoryCheck：胜负条件检查

输入：存活单位列表、关卡配置的胜负条件
处理：先检查 lose_conditions（OR）、再检查 win_conditions（OR）、默认全灭检查
输出：GameOverState 变化、LevelCompleted 消息
禁止：在非 TurnEnd 阶段执行

### RoundEnd：回合结束收尾

输入：更新后的 TurnOrder
处理：发送 TurnEnded 消息，切换到 TurnPhase::SelectUnit
输出：新回合开始
禁止：在 RoundEnd 后跳过 RoundStart

### BattleEnd：战斗结束

输入：GameOverState 终态
处理：发送 LevelCompleted 消息，切换到 AppState::GameOver
输出：战斗结算界面
禁止：在终态后继续执行战斗逻辑

---

## Victory/Defeat Check Pipeline：胜负检查管线

```
终态检查 → 失败条件检查 → 默认全灭检查 → 胜利条件检查 → 终态判定
```

### 终态检查

输入：GameOverState 当前状态
处理：如果不是 Playing，直接返回
输出：不修改任何状态
禁止：在终态后继续检查

### 失败条件检查

输入：lose_conditions 列表、存活玩家单位、当前回合号
处理：逐条检查，任一满足则标记失败
输出：is_defeat 标志
禁止：跳过此步骤

### 默认全灭检查

输入：存活玩家单位列表
处理：检查是否有 Faction::Player 存活单位
输出：all_dead 标志
禁止：即使有显式 AllDead 配置也必须执行

### 胜利条件检查

输入：win_conditions 列表、存活敌方单位、所有存活单位、当前回合号
处理：逐条检查，任一满足则标记胜利
输出：is_victory 标志
禁止：在 is_defeat 为 true 时设置 Victory

### 终态判定

输入：is_defeat、is_victory 标志
处理：is_defeat → Defeat；!is_defeat && is_victory → Victory；否则 Playing
输出：GameOverState 变化、LevelCompleted 消息
禁止：在已为终态时修改 GameOverState

---

# 数据结构

## GameOverState（战斗终态）

职责：标识战斗的最终结果

结构：
- Playing — 战斗进行中（默认）
- Victory — 玩家胜利（终态）
- Defeat — 玩家失败（终态）

要求：
- Playing → Victory 或 Playing → Defeat 为不可逆转换
- 终态后所有检查必须 early return
- 由 check_victory_conditions 写入
- UI 层只读不写

---

## VictoryConditionDef（胜负条件配置）

职责：定义关卡的完整胜负条件（Definition，不可变）

结构：
- win_conditions：List — 胜利条件列表（OR 逻辑）
- lose_conditions：List — 失败条件列表（OR 逻辑）

要求：
- 从 RON 配置文件反序列化
- 未配置时回退到默认全灭检查
- 每个条件包含 condition_type 和可选 params

---

## ConditionTypeDef（条件类型枚举）

职责：定义所有支持的胜负条件类型

结构：
- KillAll — 消灭所有敌方单位
- SurviveTurns — 存活 N 回合
- DefeatBoss — 击败指定 Boss
- AllDead — 全灭（默认失败条件）
- TurnLimitExceeded — 超时失败

要求：
- 每个类型有明确的语义
- 缺失参数时条件永不触发（安全默认值）
- 新增类型需要修改此枚举和检查函数

---

## CombatIntent（战斗意图）

职责：传递玩家或 AI 的攻击意图到 Effect Pipeline

结构：
- source_entity：Entity — 发起攻击的单位
- target_coord：Coord — 目标坐标
- skill_id：SkillId — 使用的技能 ID

要求：
- AI 和玩家共用同一 CombatIntent 资源
- 执行完毕后必须清除 CombatIntent
- 通过 Effect Pipeline 执行，禁止直接扣血

---

# 禁止事项

禁止：绕过 Effect Pipeline 直接扣血

原因：Effect Pipeline 是战斗效果执行的唯一通道。绕过管线会跳过 Modifier 修饰、Trait 触发、BattleRecord 记录等关键步骤。

违反后果：伤害计算不一致、修饰规则失效、战斗日志缺失、AI 与玩家伤害计算不同。

---

禁止：在非 TurnEnd 阶段检查胜负条件

原因：胜负检查依赖完整的回合状态（所有单位已行动）。在其他阶段检查会导致不完整的判定。

违反后果：单位尚未行动就被判定胜负、战斗提前结束或延迟结束。

---

禁止：终态后修改 GameOverState

原因：终态不可逆是战斗结果一致性的基础。终态后修改会导致 UI 展示混乱和存档数据不一致。

违反后果：Victory 被覆盖为 Defeat、Defeat 被恢复为 Playing、UI 展示闪烁。

---

禁止：AI 绕过 CombatIntent 独立执行攻击

原因：CombatIntent 是攻击意图的唯一通道。AI 独立执行会导致 AI 和玩家使用不同的伤害计算路径。

违反后果：AI 伤害计算不走 Modifier 管线、AI 行为与玩家行为不一致、测试无法覆盖 AI 伤害。

---

禁止：跳过默认全灭检查

原因："全灭玩家即失败"是绝对不变量，即使关卡配置了其他失败条件或未配置任何失败条件，此检查必须生效。

违反后果：全灭玩家后游戏不结束、战斗无限循环。

---

禁止：在战斗中修改 Definition 配置数据

原因：Definition 是不可变配置。战斗中的数值修改应通过 Modifier 管线作用于 Instance。

违反后果：全局配置被污染、多场战斗数据不一致、热重载失效。

---

禁止：在 BattleStart 阶段检查胜负条件

原因：BattleStart 是战斗初始化阶段，所有单位尚未行动，检查胜负条件无意义。

违反后果：战斗刚开始就被判定胜负、游戏无法正常进行。

---

禁止：BattleEnd 后继续执行战斗逻辑

原因：终态达成后战斗已结束，继续执行会导致状态不一致。

违反后果：终态后单位继续行动、伤害继续计算、UI 展示混乱。

---

禁止：手动设置 BattlePhase 而不经过 NextState

原因：直接设置 BattlePhase 值会跳过 OnEnter/OnExit 生命周期钩子，破坏状态机的确定性。

违反后果：钩子未触发、状态不一致、初始化或清理逻辑缺失。

---

禁止：跳过 TurnEnd 直接开始新回合

原因：TurnEnd 负责 acted 重置、队列重建、回合号递增。跳过 TurnEnd 会导致单位无法行动。

违反后果：单位 acted 标记未重置、行动队列未重建、回合号未递增。

---

禁止：在 PlayerPhase 内执行敌方逻辑

原因：PlayerPhase 和 EnemyPhase 职责混淆，AI 和玩家逻辑耦合。

违反后果：AI 决策与玩家操作使用同一阶段，测试无法分离，行为不一致。

---

# AI 修改规则

## 如果新增胜负条件类型

允许：
- 在 ConditionTypeDef 枚举中新增变体
- 在 check_single_win_condition / check_single_lose_condition 中添加匹配分支
- 在 ConditionParamsDef 中添加新参数

禁止：
- 硬编码新的胜负条件逻辑（必须数据驱动）
- 修改现有条件类型的行为
- 在 TurnEnd 以外的阶段检查新条件

优先检查：
- 新条件类型的语义是否明确
- 新条件的参数是否从 ConditionParamsDef 读取
- 缺失参数时是否有安全默认值（条件永不触发）
- 是否需要修改 LevelConfigDef 的 RON Schema

---

## 如果修改胜负检查逻辑

允许：
- 调整条件检查顺序（但失败必须先于胜利）
- 新增兜底检查系统

禁止：
- 移除默认全灭检查
- 让胜利优先于失败
- 修改终态不可逆语义
- 在非 TurnEnd 阶段触发胜负检查

优先检查：
- 全灭玩家时 GameOverState 是否为 Defeat
- 胜负同时满足时是否判定为 Defeat
- 终态后检查是否 early return
- 新增条件的缺失参数是否使用安全默认值

---

## 如果新增 Effect Pipeline 效果类型

允许：
- 实现 EffectHandler trait（type_name / generate / preview / execute）
- 注册到 EffectHandlerRegistry
- 添加对应的 EffectDef 变体

禁止：
- 修改管线调度代码（generate.rs / modify.rs / execute.rs 的调度逻辑）
- 在 execute_effects 中添加 match 分支

优先检查：
- EffectDef::type_name 与 EffectHandler::type_name 是否一致
- generate 返回 None 是否正确处理（类型不匹配）
- execute 返回 None 是否正确处理（类型不匹配）

---

## 如果修改战斗生命周期

允许：
- 在 Battle Lifecycle Pipeline 中新增阶段
- 调整阶段执行顺序

禁止：
- 移除 VictoryCheck 阶段
- 在 BattleStart 检查胜负条件
- 在 BattleEnd 后继续执行战斗逻辑

优先检查：
- 新阶段是否有明确的输入和输出
- 新阶段是否遵循 BattleStart → ... → BattleEnd 的流程结构
- 新阶段是否与 Turn 领域的回合结算冲突

---

## 如果修改 BattlePhase 状态机

允许：
- 在 BattlePhase 枚举中新增阶段（需 ADR）
- 通过 Plugin 在 OnEnter/OnExit 扩展点注入系统
- 调整合法转换路径（需更新转换表）

禁止：
- 手动设置 BattlePhase 值（必须通过 NextState）
- 跳过 TurnEnd 直接开始新回合
- 在 PlayerPhase 执行敌方逻辑
- 在 PreBattle/RoundStart/TurnEnd 的 OnEnter 中修改单位属性

优先检查：
- 新阶段是否有对应的 OnEnter/OnExit 钩子
- 合法转换表是否更新
- 非法转换表是否更新
- 新阶段是否与 TurnPhase SubState 冲突
- 扩展点注册是否通过 Plugin 而非硬编码

---

## 如果测试失败

排查顺序：
1. 检查 GameOverState 终态是否不可逆（终态后是否有 early return）
2. 检查胜负检查顺序是否正确（失败先于胜利）
3. 检查默认全灭检查是否生效（即使未配置 AllDead）
4. 检查胜负检查是否仅在 TurnEnd 阶段执行
5. 检查 Effect Pipeline 三步是否完整执行（Generate → Modify → Execute）
6. 检查 CombatIntent 执行后是否清除
