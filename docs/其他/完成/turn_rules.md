# Turn 领域

Version: 1.0

Turn 领域管理 SRPG 的游戏流程状态机、行动队列和回合切换。敏捷驱动行动队列，所有单位按 Initiative 降序行动，队列耗尽时回合结束。

核心原则：
- 状态机负责流程，不负责业务细节
- OnEnter / OnExit 保持轻量
- Message 负责跨 Feature 广播
- 敏捷驱动行动队列

---

# 术语定义

## TurnPhase

回合阶段状态机，定义当前游戏处于哪个交互阶段。

不是 TurnOrder。Phase 是交互阶段，Order 是行动队列。

关键属性：
- SelectUnit / MoveUnit / ActionMenu / SelectTarget / ExecuteAction / WaitAction / TurnEnd

---

## TurnOrder

行动队列，按 Initiative 降序排列的单位列表。

不是 TurnState。Order 是队列，State 是当前阵营和回合号。

关键属性：
- queue：Entity 列表（按 Initiative 降序）
- current_index：当前行动索引
- turn_number：当前回合号

---

## TurnState

回合状态，记录当前阵营和回合号。

不是 TurnOrder。State 是全局状态，Order 是队列数据。

关键属性：
- current_faction：当前行动阵营
- turn_number：当前回合号

---

## Initiative

单位的敏捷值，决定行动顺序。

不是 Speed。Initiative 是排序依据，Speed 是属性名。

关键属性：
- 值越大越先行动
- 相同时保持原始顺序（稳定排序）

---

# 领域边界

## 本领域负责

- AppState 和 TurnPhase 状态机
- TurnOrder 行动队列构建和推进
- TurnState 回合状态管理
- 回合生命周期（开始/结束/重建）
- TurnStarted / TurnEnded / ForceEndTurn Message

## 本领域不负责

- 单位行动的具体执行（由 battle_rules 领域负责）
- Buff 回合结算（由 buff_rules 领域负责）
- AI 决策（由 ai_rules 领域负责）
- UI 交互（由 ui_rules 领域负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 回合开始 | TurnStarted Message | 全局 |
| 回合结束 | TurnEnded Message | 全局 |
| 强制结束 | ForceEndTurn Message | turn |
| 当前行动单位 | TurnOrder.current_unit() | battle / ai |

---

# 生命周期

## 回合生命周期

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| SelectUnit | 选择行动单位 | MoveUnit, TurnEnd |
| MoveUnit | 移动阶段 | ActionMenu |
| ActionMenu | 行动菜单 | SelectTarget, WaitAction |
| SelectTarget | 选择攻击目标 | ExecuteAction |
| ExecuteAction | 执行攻击 | SelectUnit |
| WaitAction | 待机 | SelectUnit |
| TurnEnd | 回合结束 | SelectUnit（新回合） |

## 状态转换图

```
SelectUnit → MoveUnit → ActionMenu → SelectTarget → ExecuteAction → SelectUnit
                     ↘ WaitAction → SelectUnit
                     ↘ ActionMenu → WaitAction
SelectUnit → TurnEnd → SelectUnit（新回合）
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| SelectUnit | MoveUnit | 选择了一个可行动单位 |
| MoveUnit | ActionMenu | 单位移动完成或跳过 |
| ActionMenu | SelectTarget | 选择攻击 |
| ActionMenu | WaitAction | 选择待机 |
| SelectTarget | ExecuteAction | 选择目标 |
| ExecuteAction | SelectUnit | 攻击执行完成 |
| WaitAction | SelectUnit | 待机完成 |
| SelectUnit | TurnEnd | 队列耗尽 |
| TurnEnd | SelectUnit | 回合重建完成 |

---

# 不变量

## 不变量1：行动顺序由 Initiative 决定

TurnOrder 构建完成后：

queue 按 Initiative 降序排列，不区分阵营。

违反表现：

低 Initiative 单位先行动，或同阵营连续行动。

---

## 不变量2：队列耗尽自动进入 TurnEnd

advance() 返回 None 时：

必须切换到 TurnPhase::TurnEnd。

违反表现：

队列耗尽后停留在 SelectUnit，无法继续游戏。

---

## 不变量3：回合结束重置所有单位 acted

TurnEnd 处理完成后：

所有存活单位的 acted = false。

违反表现：

新回合单位无法行动。

---

## 不变量4：回合结束总是回到 SelectUnit

TurnEnd 处理完成后：

必须切换到 TurnPhase::SelectUnit。

违反表现：

跳过 SelectUnit 直接进入其他阶段。

---

# 业务规则

## 规则1：行动队列构建

禁止：
- 区分阵营排序
- 修改 Initiative 值

必须：
- 按 Initiative 降序排列
- 相同 Initiative 保持原始顺序（稳定排序）
- 空队列返回空 Vec

---

## 规则2：回合结束流程

禁止：
- 跳过 Buff 结算
- 跳过 acted 重置
- 跳过队列重建

必须：
- 发送 TurnEnded Message
- turn_number += 1
- 重置所有单位 acted = false
- 设置 needs_resolve = true
- 重建行动队列
- 更新 current_faction
- 重置 AI 计时器
- 发送 TurnStarted Message
- 切换到 SelectUnit

---

## 规则3：强制结束回合

禁止：
- 直接跳到新回合

必须：
- 发送 ForceEndTurn Message
- 队列自然耗尽后进入 TurnEnd

允许：
- 未行动单位跳过

---

## 规则4：NeedsResolve 防重复

禁止：
- 每次进入 SelectUnit 都结算 Buff

必须：
- 只在 TurnEnd 设置 needs_resolve = true
- 结算后设置 needs_resolve = false

---

# 流程管线

## 游戏开始管线

OnEnter(InGame) → Camera → Map → Unit → init_turn_order → TurnStarted

### Step1：初始化

输入：GameSet 链式执行
处理：Camera → Map → Unit → Ui
输出：地图和单位已生成
禁止：跳过任何 GameSet

### Step2：构建队列

输入：所有存活单位 + Initiative
处理：按 Initiative 降序排列
输出：TurnOrder
禁止：单位未生成时构建队列

### Step3：发送消息

输入：turn_number = 1
处理：发送 TurnStarted { turn: 1 }
输出：全局通知
禁止：跳过消息发送

---

## 回合结束管线

TurnEnded → 消费 ForceEndTurn → acted 重置 → needs_resolve → 队列重建 → TurnStarted → SelectUnit

### Step1：发送 TurnEnded

输入：当前 turn_number
处理：发送 TurnEnded Message
输出：全局通知
禁止：跳过消息发送

### Step2：重置和结算

输入：所有单位 + NeedsResolve
处理：acted = false，needs_resolve = true
输出：重置后的状态
禁止：跳过 acted 重置

### Step3：队列重建

输入：所有存活单位 + Initiative
处理：按 Initiative 降序排列
输出：新 TurnOrder
禁止：使用旧队列

### Step4：新回合开始

输入：新 turn_number
处理：发送 TurnStarted Message，切换到 SelectUnit
输出：新回合状态
禁止：跳过 SelectUnit

---

# 数据结构

## TurnOrder（Resource）

职责：行动队列

结构：
- queue：Vec<Entity>（按 Initiative 降序）
- current_index：usize
- turn_number：u32

要求：
- build() 按 Initiative 降序稳定排序
- advance() 返回 None 表示队列耗尽
- current_unit() 返回当前行动单位

---

## TurnState（Resource）

职责：回合全局状态

结构：
- current_faction：Faction
- turn_number：u32

要求：
- 默认 Faction::Player，turn_number = 1

---

## TurnPhase（SubState）

职责：回合阶段状态机

结构：
- SelectUnit / MoveUnit / ActionMenu / SelectTarget / ExecuteAction / WaitAction / TurnEnd

要求：
- 仅在 AppState::InGame 时激活
- OnEnter / OnExit 保持轻量

---

## AiTimer（Resource）

职责：AI 行动延迟

结构：
- timer：Timer（0.8 秒，Once 模式）

要求：
- 每回合重置

---

# 禁止事项

禁止：状态机处理业务细节

原因：状态机只负责流程转换

违反后果：OnEnter/OnExit 膨胀，状态机难以维护

---

禁止：跳过队列耗尽检查

原因：队列耗尽是回合结束的唯一触发条件

违反后果：回合无法正常结束

---

禁止：区分阵营排序

原因：Initiative 是唯一排序依据，不区分阵营

违反后果：同阵营连续行动，破坏游戏平衡

---

禁止：直接跳到新回合

原因：ForceEndTurn 必须让队列自然耗尽

违反后果：未行动单位的 Buff/效果未正确处理

---

禁止：每次 SelectUnit 都结算 Buff

原因：NeedsResolve 控制结算时机

违反后果：Buff 效果被重复应用

---

# AI 修改规则

## 如果新增回合阶段

允许：
- 新增 TurnPhase 变体
- 新增阶段转换逻辑

禁止：
- 修改现有阶段的转换条件
- 在 OnEnter/OnExit 中处理业务细节

优先检查：
- 阶段转换是否完整
- 是否影响队列推进
- UI 是否需要适配

---

## 如果修改行动顺序

允许：
- 修改 Initiative 计算方式

禁止：
- 修改 TurnOrder.build() 的排序逻辑
- 区分阵营排序

优先检查：
- 稳定排序是否保持
- 队列耗尽是否正确触发
- AI 行动是否受影响

---

## 如果测试失败

排查顺序：
1. 检查 TurnOrder 是否正确构建
2. 检查 advance() 是否正确推进
3. 检查队列耗尽是否触发 TurnEnd
4. 检查 acted 是否在回合结束时重置
5. 检查 NeedsResolve 是否防止重复结算
