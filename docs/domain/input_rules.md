# 输入处理领域

Version: 1.0
Status: Proposed

输入处理领域管理用户输入抽象、多平台输入映射、输入阻塞控制和输入路由分发。

核心原则：
- 输入系统只负责采集和路由，不执行游戏逻辑
- 所有用户操作通过 UiCommand Message 传递
- AI 回合期间玩家输入完全忽略

---

# 术语定义

## 输入抽象层（Input Abstraction）

将键盘/鼠标/手柄映射为统一游戏操作的中间层。

不是 Bevy Input。不是直接键位检测。不是硬件事件。

关键属性：
- 将 Pointer<Click> / ButtonInput<KeyCode> / ButtonInput<GamepadButton> 统一转换
- 输出与平台无关的 UiCommand Message
- 通过 Input Mapping 配置映射关系

---

## 输入映射（Input Mapping）

键位到游戏操作的映射表。

不是硬编码。不是平台特定。不是固定不变的。

关键属性：
- 鼠标左键点击 → SelectUnit / MoveUnit / SelectTarget（取决于 TurnPhase）
- 鼠标右键 → Cancel
- E 键 → EndTurn
- ESC 键 → Cancel（始终可用，不受 UiFocusState 阻止）
- 映射关系在 InputPlugin 中注册

---

## 输入阻塞（Input Block）

动画/模态面板期间阻止游戏操作输入的标记。

不是游戏暂停。不是 Input 消耗。不是关闭输入系统。

关键属性：
- 通过 UiFocusState.blocks_input 字段控制
- blocks_input = true 时，handle_click / handle_right_cancel / handle_end_turn 跳过
- ESC 键不受阻塞（始终可用）
- 由 UI 层的模态面板设置（打开时 blocks_input = true）

---

## UiFocusState（UI 焦点状态）

参见 `ui_architecture_rules.md#UI 焦点状态`。

模态面板阻止输入的判断依据。

不是游戏状态。不是 TurnPhase。不是 InputContext。

关键属性：
- blocks_input：bool — 是否有面板阻止输入
- 由 update_ui_focus_state 系统自动维护
- Input 系统读取此 Resource 决定是否跳过游戏操作

---

## 输入路由（Input Routing）

根据当前 TurnPhase 和游戏状态将原始输入分发到对应处理系统。

不是输入处理。不是事件广播。不是游戏逻辑。

关键属性：
- 根据 TurnPhase 决定点击行为（SelectUnit → 选中单位、MoveUnit → 移动、SelectTarget → 选择目标）
- 根据 Faction 决定是否接受输入（仅 Player 阵营回合）
- 根据 UiFocusState 决定是否阻塞输入

---

## 游戏操作（GameAction）

与平台无关的抽象操作枚举。

不是键盘按键。不是鼠标点击。不是手柄按钮。

关键属性：
- UiCommand 枚举承载游戏操作语义
- 变体：SelectUnit / MoveUnit / SelectTarget / Cancel / EndTurn / Attack / Skill / Wait
- 每个变体携带最小必要数据（Entity ID、坐标、技能 ID 等）

---

## 输入上下文（InputContext）

当前游戏状态决定的输入有效集合。

不是游戏状态。不是输入映射。不是 TurnPhase。

关键属性：
- MainMenu 上下文：菜单操作有效
- InGame 上下文：战斗操作有效
- GameOver 上下文：结算界面操作有效
- 由 AppState 决定当前上下文

---

# 领域边界

## 本领域负责

- 鼠标点击处理（单位点击、空格子点击）
- 键盘快捷键处理（E 键结束回合、ESC 键取消）
- 悬停实体追踪（HoveredEntity）
- 输入阻塞判断（UiFocusState.blocks_input）
- 输入路由分发（根据 TurnPhase 分发 UiCommand）
- 坐标转换（cursor_to_coord：屏幕坐标 → 格子坐标）

## 本领域不负责

- 游戏逻辑执行（由 command_handler 消费 UiCommand 后执行）
- UI 面板渲染（由 UI 架构领域负责）
- 回合阶段转换（由 Turn Battle 领域负责）
- 输入映射配置（由 InputPlugin 注册）
- 手柄输入支持（当前未实现，由未来扩展）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 用户操作意图 | UiCommand（Message） | Core（command_handler 消费） |
| 悬停实体更新 | Resource 变更（HoveredEntity） | UI（ViewModel 更新） |
| 输入阻塞状态 | Resource 读取（UiFocusState） | UI（模态面板设置） |
| 当前回合阶段 | Resource 读取（TurnPhase / TurnState） | Input（路由判断） |

---

# 生命周期

## 状态列表

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Idle（默认） | 等待用户输入 | Processing |
| Processing | 正在处理输入事件 | Idle |

## 状态转换图

```
Idle → Processing → Idle
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Idle | Processing | 检测到用户输入事件 |
| Processing | Idle | 输入事件处理完成（发送 UiCommand） |

---

# 不变量

## 不变量1：AI 回合期间玩家输入完全忽略

回合阶段流转：

turn_state.current_faction != Faction::Player 时，所有输入处理系统必须 early return，不发送任何 UiCommand。

违反表现：

AI 回合中玩家点击单位导致状态不一致、AI 回合中按 E 键结束回合。

---

## 不变量2：所有用户操作通过 UiCommand 传递

任意时刻：

用户的所有游戏操作必须通过发送 UiCommand Message 实现。禁止在 Input 系统中直接修改 ECS 组件。

违反表现：

Input 系统中出现 `commands.entity(e).insert(Selected)` 而不通过 UiCommand。

---

## 不变量3：输入阻塞由 UiFocusState 控制

任意时刻：

当 UiFocusState.blocks_input = true 时，handle_click / handle_right_cancel / handle_end_turn 必须跳过。ESC 键不受此限制。

违反表现：

模态面板打开时仍能执行游戏操作、ESC 键被错误阻塞。

---

## 不变量4：输入系统不执行游戏逻辑

任意时刻：

Input 系统只负责采集原始输入、路由分发、发送 UiCommand。游戏逻辑由 command_handler 消费 UiCommand 后执行。

违反表现：

Input 系统中直接修改 TurnPhase 或设置 CombatIntent。

---

## 不变量5：右键在 MoveUnit 阶段发送 Cancel

回合阶段流转：

在 TurnPhase::MoveUnit 阶段，鼠标右键必须发送 UiCommand::Cancel。在 ActionMenu / SelectTarget 阶段，右键也发送 Cancel。

违反表现：

MoveUnit 阶段右键无反应、取消操作不生效。

---

## 不变量6：坐标转换必须验证边界

任意时刻：

cursor_to_coord 返回的坐标必须通过 map.is_in_bounds() 验证。超出边界的坐标不发送 UiCommand。

违反表现：

点击地图边界外的区域导致越界坐标发送给 MoveUnit / SelectTarget。

---

# 业务规则

## 规则1：单位点击处理

禁止：
- 非玩家阵营回合处理单位点击
- 选择不在行动队列中的单位（SelectUnit 阶段）
- 点击已死亡的单位

必须：
- 仅处理 PointerButton::Primary（左键）
- TurnPhase::SelectUnit 时只选择当前行动单位（turn_order.current_unit()）
- TurnPhase::MoveUnit 时发送 MoveUnit 命令
- TurnPhase::SelectTarget 时发送 SelectTarget 命令

允许：
- 悬停在非 Unit 实体上时更新 HoveredEntity

---

## 规则2：空格子点击处理

禁止：
- 鼠标悬停在单位上时处理空格子点击（由 Pointer<Click> Observer 处理）
- 发送越界坐标

必须：
- 使用 cursor_to_coord 计算格子坐标
- 验证坐标边界（is_in_bounds）
- 根据 TurnPhase 发送对应的 UiCommand

允许：
- 仅在 MoveUnit 和 SelectTarget 阶段响应空格子点击

---

## 规则3：键盘快捷键

禁止：
- AI 回合响应键盘快捷键
- 模态面板打开时响应非 ESC 快捷键

必须：
- E 键仅在 TurnPhase::SelectUnit 时发送 EndTurn
- ESC 键始终可用（不受 UiFocusState 阻塞）
- ESC 在 SelectUnit 阶段无操作可取消时跳过

允许：
- ESC 在 ActionMenu / SelectTarget / MoveUnit 阶段发送 Cancel

---

## 规则4：悬停追踪

禁止：
- 非 Unit 实体触发悬停更新
- 重复更新相同的 HoveredEntity

必须：
- Pointer<Over> 时检查实体是否为 Unit
- Pointer<Out> 时清除 HoveredEntity
- HoveredEntity 变化时触发 ViewModel 更新

允许：
- HoveredEntity 为 None 时表示无悬停

---

## 规则5：输入路由

禁止：
- 跳过 TurnPhase 判断直接发送 UiCommand
- 在不匹配的阶段发送不匹配的命令

必须：
- 根据 TurnPhase 决定 UiCommand 类型
- SelectUnit 阶段只发送 SelectUnit
- MoveUnit 阶段发送 MoveUnit 或 Cancel
- SelectTarget 阶段发送 SelectTarget 或 Cancel

允许：
- ActionMenu 阶段由 UI 按钮处理，Input 系统不直接处理

---

# 流程管线

## 输入采集管线

```
原始输入事件 → 当前 InputContext 过滤 → Input Mapping 转换 → GameAction
```

### Step1：原始输入事件

输入：Pointer<Click> / ButtonInput<KeyCode> / ButtonInput<MouseButton>
处理：接收 Bevy 原始输入事件
输出：原始输入数据
禁止：在此步骤中发送 UiCommand

### Step2：当前 InputContext 过滤

输入：TurnState + TurnPhase + UiFocusState
处理：检查是否在玩家回合、是否被阻塞、当前阶段是否允许输入
输出：是否继续处理
禁止：跳过过滤直接处理

### Step3：Input Mapping 转换

输入：原始输入 + TurnPhase
处理：根据映射关系确定操作类型（SelectUnit / MoveUnit / SelectTarget / Cancel / EndTurn）
输出：UiCommand 类型
禁止：硬编码映射关系（应通过注册机制）

### Step4：发送 GameAction

输入：UiCommand 类型 + 必要数据
处理：通过 MessageWriter 发送 UiCommand Message
输出：UiCommand Message
禁止：构造不合法的 UiCommand

---

## 输入路由管线

```
GameAction → TurnPhase/State 路由 → UiCommand 发送 or 直接 System 调用
```

### Step1：GameAction 接收

输入：UiCommand Message
处理：读取消息内容
输出：操作类型和参数
禁止：在路由前执行游戏逻辑

### Step2：TurnPhase/State 路由

输入：UiCommand + TurnPhase + TurnState
处理：根据当前阶段决定处理方式
输出：路由结果
禁止：在非玩家回合路由

### Step3：UiCommand 发送

输入：路由后的 UiCommand
处理：通过 MessageWriter 发送最终的 UiCommand
输出：UiCommand Message 传递到 command_handler
禁止：在 Input 系统中执行 UiCommand 的处理逻辑

---

# 数据结构

## HoveredEntity（悬停实体 Resource）

职责：记录当前鼠标悬停的单位实体，驱动 ViewModel 更新

结构：
- entity：Option<Entity> — 悬停的实体，None 表示无悬停

要求：
- 是 Bevy Resource
- 只被 Input 系统的 Pointer<Over> / Pointer<Out> 写入
- 变化时触发 update_selected_unit_view 更新
- Core 层不读取此 Resource

---

## UiFocusState（UI 焦点状态 Resource）

参见 `ui_architecture_rules.md#UiFocusState`。

职责：追踪是否有模态面板正在阻止游戏输入

结构：
- blocks_input：bool — 是否有面板阻止输入

要求：
- 是 Bevy Resource
- 默认值 blocks_input = false
- Input 系统读取此 Resource 决定是否跳过游戏操作
- ESC 键不受 blocks_input 限制

---

## UiCommand（UI 命令枚举）

参见 `ui_architecture_rules.md#UiCommand`。

职责：封装用户的所有操作意图，作为 Bevy Message 传递

结构：
- SelectUnit：携带 Entity — 选中玩家单位
- MoveUnit：携带 IVec2 坐标 — 移动到目标格子
- SelectTarget：携带 IVec2 坐标 — 选择攻击目标
- Cancel：无参数 — 取消当前操作
- EndTurn：无参数 — 结束回合
- Attack：无参数 — 选择基础攻击
- Skill：携带 skill_id String — 选择技能
- Wait：无参数 — 待机

要求：
- 派生 Message trait
- 变体携带最小必要数据
- 禁止携带游戏状态数据
- 所有变体在 ui/events.rs 中定义

---

## cursor_to_coord（坐标转换函数）

职责：将屏幕光标位置转换为格子坐标

结构：
- 输入：windows、camera、cam_transform、map
- 输出：Option<IVec2>（超出边界返回 None）

要求：
- 通过 camera.viewport_to_world_2d 转换为世界坐标
- 通过 map.world_to_coord 转换为格子坐标
- 通过 map.is_in_bounds 验证边界

---

# 禁止事项

禁止：AI 回合中处理玩家输入

原因：AI 和玩家共用同一行动队列，AI 回合中处理玩家输入会导致状态不一致

违反后果：AI 行动被玩家输入干扰、回合阶段混乱、战斗状态不一致

---

禁止：在 Input 系统中直接修改 ECS 组件

原因：用户操作必须通过 UiCommand Message 传递给 command_handler，由 Core 层决定如何修改状态

违反后果：游戏状态被 UI 绕过逻辑层修改，状态变更不可追踪、不可回滚

---

禁止：模态面板打开时执行游戏操作

原因：模态面板（如单位信息面板）打开时，用户意图是查看信息而非操作游戏

违反后果：点击操作穿透到游戏逻辑、状态混乱

---

禁止：跳过 UiFocusState 检查直接处理输入

原因：UiFocusState 是输入阻塞的唯一判断依据，跳过检查会绕过 UI 焦点管理

违反后果：模态面板打开时仍能执行游戏操作

---

禁止：Input 系统中执行游戏逻辑

原因：Input 系统只负责采集和路由，游戏逻辑由 command_handler 消费 UiCommand 后执行

违反后果：Input 系统与游戏逻辑耦合、无法独立测试

---

禁止：坐标转换不验证边界

原因：超出地图边界的坐标会导致寻路失败、单位移动到非法位置

违反后果：运行时 panic、单位移动到地图外

---

禁止：ESC 键被 UiFocusState 阻塞

原因：ESC 是用户的"安全出口"，始终可用以关闭面板或取消操作

违反后果：模态面板打开时无法关闭、用户被困在面板中

---

禁止：重复更新 HoveredEntity

原因：重复更新会触发不必要的 ViewModel 重绘，影响性能

违反后果：UI 面板每帧重绘、性能下降

---

# AI 修改规则

## 如果新增输入设备支持（如手柄）

允许：
- 在 InputPlugin 中注册新的输入处理系统
- 新系统遵循相同的路由规则（TurnPhase / Faction / UiFocusState 检查）

禁止：
- 为每种输入设备写独立的游戏逻辑
- 手柄输入绕过 UiCommand 直接修改游戏状态

优先检查：
- 新输入设备是否遵循相同的 Input Mapping 规则
- 新输入设备是否正确检查 UiFocusState
- 新输入设备是否在 AI 回合中被忽略

---

## 如果新增 UiCommand 变体

允许：
- 在 ui/events.rs 的 UiCommand 枚举中追加新变体
- 在 command_handler.rs 的 handle_ui_commands 中添加处理分支
- 新变体携带最小必要数据

禁止：
- 在 Input 系统中直接处理新命令（必须通过 command_handler）
- 新变体携带游戏状态数据

优先检查：
- 新变体是否在 events.rs 中定义
- command_handler 中是否正确处理
- 是否在 player_turn 条件下运行

---

## 如果修改输入路由逻辑

允许：
- 调整 TurnPhase 到 UiCommand 的映射关系
- 添加新的阶段特定输入行为

禁止：
- 移除 AI 回合的输入忽略检查
- 移除 UiFocusState 检查
- 在 Input 系统中执行游戏逻辑

优先检查：
- 所有输入处理系统是否正确检查 Faction::Player
- 所有输入处理系统是否正确检查 UiFocusState.blocks_input
- 新的路由规则是否与现有 TurnPhase 状态机兼容

---

## 如果修改悬停追踪

允许：
- 调整 HoveredEntity 的更新逻辑
- 添加新的悬停视觉反馈

禁止：
- 非 Unit 实体触发悬停更新
- 重复更新相同的 HoveredEntity

优先检查：
- HoveredEntity 变化时是否触发 update_selected_unit_view
- Pointer<Out> 时是否正确清除
- Core 层是否不读取 HoveredEntity

---

## 如果测试失败

排查顺序：
1. 检查 turn_state.current_faction 是否正确判断（AI 回合是否被忽略）
2. 检查 UiFocusState.blocks_input 是否正确阻塞（模态面板是否生效）
3. 检查 TurnPhase 路由是否正确（不同阶段是否发送正确的 UiCommand）
4. 检查 cursor_to_coord 边界验证是否正确
5. 检查 UiCommand 是否正确发送到 command_handler
