---
id: 02-domain.turn.turn-rules
title: Turn Rules
status: draft
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
tags:
  - domain
  - turn
---

# 回合系统领域

Version: 1.0
Status: Proposed

回合系统领域管理战斗中单位行动的时序调度、回合阶段流转、行动顺序编排和回合生命周期。

核心原则：
- 🟩 回合阶段状态机驱动单位行动循环，NextState 驱动转换（宪法 11.1.2）
- 🟩 行动队列按先攻值降序排列，回合开始时重建
- 🟩 每个单位每回合只能行动一次，acted 标志严格控制
- 🟩 所有操作入口为标准化命令（Command Layer），执行不区分来源（宪法 11.5.1-11.5.2）
- 🟩 读路径无副作用，写路径通过命令与执行系统统一处理（宪法 11.7 CQRS Lite）
- 🟥 禁止手动调用回合切换函数，必须通过状态机驱动（宪法 11.1.2）

---

# 宪法合规声明

本领域遵循以下宪法条款：

| 条款编号 | 条款名称 | 合规状态 | 说明 |
|----------|----------|----------|------|
| 11.1.1 | 阶段划分标准化 | 🟩 已合规 | 七个阶段覆盖完整回合生命周期 |
| 11.1.2 | 状态驱动回合流转 | 🟩 已合规 | 使用 SubState + NextState 驱动 |
| 11.5.1 | 所有操作入口为命令 | 🟩 已合规 | 操作通过 CombatIntent 传递 |
| 11.5.2 | 命令无差别执行 | 🟩 已合规 | AI 和玩家共用同一执行管线 |
| 11.7.1 | 读路径无副作用 | 🟩 已合规 | 阶段判定为纯读操作 |
| 11.7.2 | 写路径收口 | 🟩 已合规 | 状态修改通过 NextState 统一处理 |
| 2.2.4 | Message 跨域广播 | 🟩 已合规 | TurnStarted/TurnEnded 用于跨域通知 |
| 2.3.6 | States/SubStates 状态管理 | 🟩 已合规 | TurnPhase 为 AppState::InGame 的 SubState |
| 2.3.8 | Schedule 权责划分 | 🟩 已合规 | 回合逻辑在 Update 阶段执行 |

---

# 四级通信机制（宪法 2.2）

回合领域在四级通信机制中的定位：

| 通信层级 | 用途 | 回合领域应用 |
|----------|------|-------------|
| Hook（2.2.1） | 组件生命周期 | BattleEntity 标记组件的 OnEnter/OnExit |
| Trigger（2.2.2） | Feature 内事件链 | 回合内部阶段转换的连锁响应 |
| Observer（2.2.3） | 局部状态变化响应 | TurnPhase 变化触发的 UI 刷新 |
| Message（2.2.4） | 跨域广播 | TurnStarted/TurnEnded/ForceEndTurn 通知其他领域 |

禁止事项（宪法 2.2.5）：
- 🟥 禁止将回合内部的普通逻辑全部事件化
- 🟥 禁止滥用 Trigger 模拟函数调用
- 🟥 禁止为临时副作用随意新增领域事件（宪法 2.2.7）

---

# 术语定义

## 回合（Round）

所有存活单位各行动一次的完整循环。回合号由 `TurnState.turn_number` 和 `TurnOrder.turn_number` 跟踪。

不是单个单位的行动。不是整场战斗。

关键属性：
- 回合号在 `OnEnter(TurnPhase::TurnEnd)` 中递增
- 每个回合开始时重建行动队列（所有存活单位按 Initiative 降序）
- 每个回合结束时重置所有单位的 `acted` 标志

---

## 回合阶段（TurnPhase）

回合内的子状态，控制当前单位的行动流程。仅在 `AppState::InGame` 时激活。

不是行动动画。不是 Bevy AppState。

关键属性：
- 作为 `AppState::InGame` 的 SubState 存在
- 阶段转换必须通过 `NextState<TurnPhase>` 驱动
- 包含七个阶段：SelectUnit → MoveUnit → ActionMenu → SelectTarget → ExecuteAction → WaitAction → TurnEnd

---

## 行动顺序（TurnOrder）

所有存活单位按 Initiative 降序排列的行动队列。

不是回合阶段。不是行动内的动作顺序。

关键属性：
- 存储在 `TurnOrder` 资源中（queue、current_index、turn_number）
- 使用稳定排序，相同 Initiative 保持原始顺序
- 队列耗尽时自动进入 TurnEnd 阶段
- AI 和玩家共用同一行动队列

---

## 先攻值（Initiative）

决定单位在行动队列中位置的派生属性值。由 `AttributeKind::Initiative` 获取，公式为 `Initiative = Agility * 2`。

不是敏捷属性本身。不是优先级标记。

关键属性：
- 值越高越先行动
- 降序排列生成行动队列
- 在回合重建队列时实时计算

---

## 行动（Action）

单个单位在自己的回合阶段内选择并执行的动作（移动、攻击、技能、道具、待机）。

不是回合阶段。不是一整个回合。

关键属性：
- 通过 `CombatIntent` 资源传递攻击意图
- 每个单位每回合只能行动一次（`unit.acted` 标志控制）
- AI 和玩家共用同一执行管线（Effect Pipeline）

---

## 回合结算（TurnEnd Phase）

回合结束时的处理阶段，在所有单位行动完毕后触发。

不是行动阶段。不是动画阶段。

关键属性：
- 胜负条件在此阶段检查（仅此阶段检查），参见 `battle_rules.md#胜负检查仅在 TurnEnd 阶段执行`
- 回合号在此阶段递增
- 所有单位 `acted` 标志在此阶段重置
- 行动队列在此阶段重建
- 持续效果结算标记 `NeedsResolve` 在此阶段设置
- 持续效果结算（参见 `docs/02-domain/duration_rules.md`）：所有Buff的duration在此阶段递减
- 触发点（参见 `docs/02-domain/trigger_rules.md`）：TurnStart和TurnEnd是Buff的触发时机点

---

## 回合消息（Turn Messages）

回合系统广播的事件消息，用于通知其他领域回合状态变化。

不是回合阶段。不是状态转换。

关键属性：
- TurnStarted — 回合开始时发送
- TurnEnded — 回合结束时发送
- ForceEndTurn — 强制结束当前回合，消费于 TurnEnd 阶段

---

# 领域边界

## 本领域负责

- 回合阶段状态机（TurnPhase：SelectUnit → MoveUnit → ActionMenu → SelectTarget → ExecuteAction → WaitAction → TurnEnd）
- 行动顺序编排（TurnOrder：Initiative 降序队列）
- 回合生命周期（回合号递增、队列重建、单位状态重置）
- 回合消息广播（TurnStarted、TurnEnded、ForceEndTurn）
- 先攻值计算（Initiative = Agility * 2）

## 本领域不负责

- 战斗状态机管理（由 Battle 领域负责：AppState 生命周期）
- 胜负条件判定（由 Battle 领域负责：Victory/Defeat Check Pipeline）
- 战斗终态管理（由 Battle 领域负责：GameOverState）
- 具体伤害计算（由 Attribute Modifier 领域负责：Effect Pipeline）
- Buff/Debuff 持续效果的具体结算逻辑（由 Buff 领域负责）
- 属性值的计算与修饰（由 Core 属性系统负责）
- 单位的具体移动和寻路（由 Map 领域负责）
- 用户输入处理（由 Input 领域负责）
- UI 展示与交互（由 UI 领域负责）
- AI 策略选择与行为定义（由 AI 领域负责）
- 关卡地图数据加载（由 Map 领域负责）
- Buff的持续策略管理（由 `docs/02-domain/duration_rules.md` 负责）
- Buff的触发时机判断（由 `docs/02-domain/trigger_rules.md` 负责）
- Buff的叠层合并逻辑（由 `docs/02-domain/stack_policy_rules.md` 负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 回合开始/结束 | Message（TurnStarted / TurnEnded） | Battle（record）、UI（回合指示器） |
| 强制结束回合 | Message（ForceEndTurn） | Turn（消费消息） |
| 攻击意图 | Resource（CombatIntent） | Battle（Effect Pipeline） |
| 单位死亡 | Message（CharacterDied） | Turn（移除队列）、UI（日志） |
| 胜负检查触发 | Message（TurnEnded） | Battle（Victory/Defeat Check Pipeline） |

---

# 生命周期

## 状态列表

### TurnPhase（回合阶段 SubState）

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| SelectUnit | 选择单位（默认） | MoveUnit, TurnEnd |
| MoveUnit | 移动阶段 | ActionMenu, SelectUnit |
| ActionMenu | 行动菜单 | SelectTarget, WaitAction, SelectUnit |
| SelectTarget | 选择目标 | ExecuteAction, ActionMenu |
| ExecuteAction | 执行攻击 | TurnEnd, SelectUnit |
| WaitAction | 待机 | TurnEnd |
| TurnEnd | 回合结算 | SelectUnit |

## 状态转换图

```
TurnPhase（仅 InGame 时激活）:
SelectUnit → MoveUnit → ActionMenu → SelectTarget → ExecuteAction → TurnEnd → SelectUnit
    ↑                                ↑              ↑                ↑
    └────────────────────────────────┘              │                │
    └─────────────── WaitAction ───────────────────┘                │
                                                                    │
    TurnEnd 总是回到 SelectUnit（回合循环）──────────────────────────┘
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| TurnPhase::SelectUnit | MoveUnit | 玩家点击已方单位并选择移动 |
| TurnPhase::MoveUnit | ActionMenu | 移动完成，弹出行动菜单 |
| TurnPhase::MoveUnit | SelectUnit | 取消移动 |
| TurnPhase::ActionMenu | SelectTarget | 选择攻击/技能行动 |
| TurnPhase::ActionMenu | WaitAction | 选择待机 |
| TurnPhase::ActionMenu | SelectUnit | 取消行动 |
| TurnPhase::SelectTarget | ExecuteAction | 选择目标并确认 |
| TurnPhase::SelectTarget | ActionMenu | 取消目标选择 |
| TurnPhase::ExecuteAction | TurnEnd | 队列耗尽 |
| TurnPhase::ExecuteAction | SelectUnit | 队列未耗尽，前进到下一单位 |
| TurnPhase::WaitAction | TurnEnd | 队列耗尽 |
| TurnPhase::WaitAction | SelectUnit | 队列未耗尽，前进到下一单位 |
| TurnPhase::TurnEnd | SelectUnit | 回合结算完成，总是回到 SelectUnit |
| TurnPhase::SelectUnit | TurnEnd | 队列耗尽时直接进入 TurnEnd |

---

# 不变量

## 不变量1：回合结束必须重置所有单位行动状态

回合生命周期：

`OnEnter(TurnPhase::TurnEnd)` 执行时，所有存活单位的 `acted` 标志必须重置为 `false`。

违反表现：

部分单位在新回合开始后仍为 `acted = true`，无法行动。

---

## 不变量2：行动队列在回合开始时必须重建

回合生命周期：

每个新回合开始时，行动队列必须基于所有存活单位的 Initiative 降序重建。不保留上一回合的队列顺序。

违反表现：

死亡单位仍在行动队列中；或新加入的单位未被排入队列。

---

## 不变量3：每个单位每回合只能行动一次

回合阶段流转：

每个单位在一个回合内，`unit.acted` 标志从 `false` 变为 `true` 后，不能再次被选中行动。

违反表现：

同一单位在同一回合内行动两次。

---

## 不变量4：TurnEnd 总是回到 SelectUnit

回合阶段流转：

无论何种路径进入 TurnEnd，结算完成后必须切换回 `TurnPhase::SelectUnit`（回合循环继续）或在终态时保持不变。

违反表现：

TurnEnd 后停留在 TurnEnd 阶段，回合循环中断。

---

# 业务规则

## 规则1：行动顺序编排

禁止：
- 在回合中途修改行动队列顺序
- 跳过队列中的存活单位
- 让同一单位在同一回合内行动两次

必须：
- 行动队列在回合开始时按 Initiative 降序构建
- 相同 Initiative 保持稳定排序（先入队先行动）
- 队列耗尽时自动进入 TurnEnd 阶段

允许：
- 通过 ForceEndTurn 消息提前结束当前阵营回合
- AI 通过 AiTimer 延迟执行（0.4 秒）

---

## 规则2：回合结算流程

禁止：
- 在 TurnEnd 阶段执行攻击逻辑
- 跳过回合号递增
- 跳过单位 acted 重置
- 跳过行动队列重建

必须：
- TurnEnd 阶段先检查胜负条件，再执行回合结算
- 回合号递增 1
- 所有存活单位 `acted` 重置为 `false`
- 行动队列基于存活单位 Initiative 降序重建
- `NeedsResolve` 标记设为 `true`
- 总是切换回 `TurnPhase::SelectUnit`
- TurnEnd阶段触发所有duration的递减（委托给duration系统）
- TurnEnd阶段触发TurnEnd时机的Buff效果（委托给trigger系统）

允许：
- 在 TurnEnd 中消费 ForceEndTurn 消息

---

## 规则3：阶段转换机制

禁止：
- 手动设置 TurnPhase 而不经过 NextState
- 在 OnEnter 中执行跨阶段跳转
- 在 OnEnter(TurnEnd) 中执行重逻辑（应在 Update 中处理）

必须：
- 所有 TurnPhase 转换通过 `NextState<TurnPhase>` 驱动
- TurnEnd 阶段总是回到 SelectUnit
- 胜负检查系统在 turn_end_on_enter 之前运行

允许：
- 在 Update 系统中设置 NextState
- 在 OnEnter 系统中发送 Message

---

## 规则4：独立 SubState 状态

禁止：
- 玩家操作和 AI 思考使用同一状态（时间尺度不同，无法独立做超时/加速）

必须：
- WaitingForPlayerInput 作为独立 SubState，等待玩家输入
- WaitingForAiDecision 作为独立 SubState，等待 AI 决策完成
- 两者可独立做超时/加速逻辑

允许：
- AI 决策完成后自动切换到下一阶段
- 玩家输入超时后强制跳过

---

## 规则5：ActionTimeline / Command Queue 模式

禁止：
- 操作执行和动画播放同步阻塞 FSM 推进

必须：
- 操作排队（Command Queue）：玩家/AI 的操作先入队，按序执行
- 动画解耦：Command 执行后，动画入队 QueuedEffect，FSM 不等待动画完成
- ImmediateEffect（伤害结算）立即生效，QueuedEffect（动画/音效）延迟执行

允许：
- Command Queue 支持优先级排序（紧急操作插队）
- 动画播放完成通知回 FSM（通过 Event）

---

## 规则6：BattleEntity marker 组件

禁止：
- OnExit(InGame) 时遗留战斗实体

必须：
- 所有战斗中生成的实体附加 `BattleEntity` marker 组件
- `OnExit(InGame)` 时 despawn 所有 `BattleEntity` 实体
- BattleEntity 包含：单位实体、UI 实体、VFX 实体等所有战斗运行时实体

允许：
- 非战斗实体（如持久化 UI）不标记 BattleEntity

---

## 规则7：禁止 Timer 替代 TurnPhase 驱动

禁止：
- 使用 Timer 控制状态持续时间（如"等待 2 秒后自动切换"）
- Timer 驱动 FSM 状态转换

必须：
- TurnPhase 必须由 Command / Event 驱动转换
- 使用 Tick 计数（回合号、帧计数）替代 Timer
- Timer 仅用于表现层（动画持续时间、UI 延迟显示）

允许：
- 表现层使用 Timer 控制动画/音效时长
- Timer 不影响 FSM 状态转换时机

---

# 流程管线

## Turn Execution Pipeline：单单位行动管线

```
SelectPhase → ActionPhase → ExecutePhase → TurnEndPhase
```

### SelectPhase：选择单位

输入：TurnOrder 当前单位
处理：高亮可选单位，等待玩家选择或 AI 自动选择
输出：选中的单位实体
禁止：选择已行动过的单位、选择死亡单位

### ActionPhase：选择行动

输入：选中的单位
处理：展示移动范围 → 移动 → 展示行动菜单 → 选择行动类型
输出：CombatIntent（攻击意图）或 WaitAction（待机）
禁止：执行攻击逻辑

### ExecutePhase：执行行动

输入：CombatIntent
处理：Effect Pipeline（Generate → Modify → Execute）、Trait 触发
输出：伤害/治疗效果、可能的单位死亡
禁止：跳过 Effect Pipeline、直接修改 HP

### TurnEndPhase：转向下一单位

输入：执行完成的行动
处理：路由到下一存活单位或进入 TurnEnd
输出：TurnPhase 状态切换
禁止：跳过死亡单位检测

---

## 行动队列管理管线

```
回合开始 → 初始化队列 → 推进队列 → 回合结束
```

### 回合开始

输入：上一回合的 TurnOrder（或初始队列）
处理：扫描所有存活单位，按 Initiative 降序排列
输出：新的行动队列
禁止：保留上一回合的队列顺序

### 初始化队列

输入：存活单位列表
处理：计算每个单位的 Initiative（Agility * 2），稳定排序，构建 queue
输出：TurnOrder.queue（降序）
禁止：将死亡单位加入队列

### 推进队列

输入：TurnOrder 当前状态
处理：advance() 返回下一个单位，current_index 加 1
输出：下一个单位实体（或 None 表示队列耗尽）
禁止：跳过队列中的单位

### 回合结束

输入：队列耗尽信号
处理：进入 TurnEnd 阶段，执行结算流程
输出：回合号递增、acted 重置、队列重建
禁止：在结算前执行攻击逻辑

---

# 数据结构

## TurnOrder（行动队列）

职责：管理当前回合的行动顺序和进度

结构：
- queue：Vec — 按 Initiative 降序排列的单位实体列表
- current_index：usize — 当前应行动的单位索引
- turn_number：u32 — 当前回合号

要求：
- queue 必须在每个新回合开始时重建
- current_index 指向当前应行动的单位
- advance() 返回 None 时表示队列耗尽，必须进入 TurnEnd
- 稳定排序保证相同 Initiative 时保持原始顺序

---

## TurnState（回合状态）

职责：保存当前回合的基本状态信息，供 UI 和日志使用

结构：
- current_faction：Faction — 当前行动单位的阵营
- turn_number：u32 — 当前回合号（从 1 开始）

要求：
- turn_number 在每个回合结束时递增
- current_faction 随队首单位变化而更新

---

## TurnPhase（回合阶段枚举）

职责：控制回合内单位行动的子状态机

结构：
- SelectUnit — 选择单位
- MoveUnit — 移动阶段
- ActionMenu — 行动菜单
- SelectTarget — 选择目标
- ExecuteAction — 执行攻击
- WaitAction — 待机
- TurnEnd — 回合结算

要求：
- 作为 AppState::InGame 的 SubState
- 阶段转换通过 NextState 驱动
- TurnEnd 总是回到 SelectUnit
- 默认初始阶段为 SelectUnit

---

# 禁止事项

禁止：手动设置 TurnPhase 而不经过 NextState

原因：Bevy 状态机通过 NextState 驱动转换才能正确触发 OnEnter/OnExit 生命周期回调。手动设置跳过回调。

违反后果：OnEnter(TurnEnd) 未触发导致回合结算不完整、单位 acted 未重置。

---

禁止：在 TurnEnd 阶段执行攻击逻辑

原因：TurnEnd 是结算阶段，不是行动阶段。攻击逻辑应在 ExecuteAction 或 WaitAction 阶段执行。

违反后果：回合结算和战斗逻辑耦合、回合号递增时序混乱。

---

禁止：回合结束时不重建行动队列

原因：单位可能在回合中死亡，必须排除死亡单位。Initiative 值可能被 Buff 修改，必须重新排序。

违反后果：死亡单位仍在行动队列中、行动顺序不正确。

---

禁止：在回合中途修改行动队列顺序

原因：回合中途修改会导致已行动单位被跳过或重复行动，破坏回合公平性。

违反后果：部分单位无法行动、部分单位行动两次、回合节奏混乱。

---

禁止：跳过回合号递增

原因：回合号是游戏时间的唯一计数器，跳过会导致依赖回合号的条件（如 SurviveTurns、TurnLimitExceeded）判定错误。

违反后果：存活回合条件差一、超时条件差一。

---

禁止：TurnEnd 阶段中胜负检查在回合号递增之后执行

原因：胜负检查依赖当前回合号进行判定（如 SurviveTurns、TurnLimitExceeded）。先递增会导致判定使用错误的回合号。

违反后果：存活回合条件差一、超时条件差一。

---

# AI 修改规则

## 如果新增回合阶段

允许：
- 在 TurnPhase 枚举中新增阶段变体
- 为新阶段添加 OnEnter/OnExit 系统
- 通过 NextState 驱动进入新阶段

禁止：
- 新阶段跳过 TurnEnd 回到 SelectUnit 的循环
- 手动设置 TurnPhase 而不通过 NextState

优先检查：
- 新阶段是否有明确的进入条件和退出条件
- 新阶段是否遵循 SelectUnit → ... → TurnEnd 的循环结构
- 新阶段是否与现有阶段冲突

---

## 如果修改行动顺序逻辑

允许：
- 修改 Initiative 计算公式（需同步修改属性系统）
- 修改 TurnOrder::build 的排序逻辑

禁止：
- 在回合中途修改行动队列
- 让同一单位在同一回合内行动两次
- 改变 TurnOrder::advance 的返回语义

优先检查：
- 排序是否仍为降序（高 Initiative 先行动）
- 稳定排序是否保持（相同 Initiative 保持原始顺序）
- 新增的排序逻辑是否影响已有测试

---

## 如果修改回合结算流程

允许：
- 在 TurnEnd 中新增结算步骤
- 调整结算步骤的执行顺序

禁止：
- 移除 acted 重置步骤
- 移除行动队列重建步骤
- 跳过回合号递增

优先检查：
- TurnEnd 后是否回到 SelectUnit
- acted 重置是否覆盖所有存活单位
- 行动队列是否排除死亡单位
- 回合号递增在胜负检查之后执行

---

## 如果修改阶段转换逻辑

允许：
- 在 Update 系统中设置 NextState
- 在 OnEnter 系统中发送 Message
- 新增允许的转换路径

禁止：
- 在 OnEnter 中执行跨阶段跳转
- 在 OnEnter(TurnEnd) 中执行重逻辑
- 手动设置 TurnPhase

优先检查：
- 新增的转换路径是否在生命周期表中登记
- NextState 驱动是否正确触发 OnEnter/OnExit
- 新路径是否破坏 SelectUnit → ... → TurnEnd 的循环结构

---

## 如果测试失败

排查顺序：
1. 检查 TurnPhase 转换是否通过 NextState 驱动
2. 检查回合结束时 acted 是否重置、队列是否重建
3. 检查队列是否排除死亡单位
4. 检查 Initiative 排序是否为降序
5. 检查 TurnEnd 后是否回到 SelectUnit
6. 检查回合号递增时序是否正确

---

# 宪法禁止事项

以下禁止事项源自 AI 开发宪法，回合领域必须严格遵守：

## 禁止：手动调用回合切换函数（宪法 11.1.2）

原因：回合切换必须通过状态机驱动，手动调用会绕过 OnEnter/OnExit 生命周期回调。

违反后果：回合结算不完整、状态不一致、无法回放。

---

## 禁止：回合内部普通逻辑全部事件化（宪法 2.2.5）

原因：事件增加调试复杂度，仅在需要解耦的场景使用。

违反后果：调试困难、性能下降、逻辑碎片化。

---

## 禁止：读路径产生副作用（宪法 11.7.1）

原因：伤害预览、范围查询等读操作必须是纯函数，修改状态会导致仿真/AI决策产生意外副作用。

违反后果：预览改变真实游戏状态、AI 决策污染战场数据。

---

## 禁止：零散分布的状态写入（宪法 11.7.2）

原因：所有状态修改必须通过命令与执行系统统一处理。

违反后果：状态变更不可追踪、不可回滚、无法支持回放。

---

## 禁止：为临时副作用随意新增领域事件（宪法 2.2.7）

原因：领域事件必须纳入白名单管理，控制事件数量爆炸。

违反后果：事件泛滥、领域事件权威性下降、维护成本上升。

---

## 禁止：命令来源差异化执行（宪法 11.5.2）

原因：真人操作、AI 操作、回放操作必须使用同一套执行链。

违反后果：回放结果与实际不一致、测试无法覆盖所有场景。

---

## 禁止：核心领域逻辑直接依赖 Bevy ECS 类型（宪法 1.4.1）

原因：回合核心规则应实现为纯函数，参数为数据结构体而非 ECS Query/Entity。

违反后果：无法离线仿真、无法独立测试、绑定引擎生命周期。
