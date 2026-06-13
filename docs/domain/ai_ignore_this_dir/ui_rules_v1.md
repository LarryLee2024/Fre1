# UI 领域

Version: 1.1

UI 领域负责所有面板、行动菜单、浮窗、视觉效果的表现层。遵循 Logic / Presentation 分离原则。

核心原则：
- 🟥 UI 不操作 ECS，只发出意图（宪法 1.1.4 逻辑与表现分离）
- 🟥 ViewModel 层隔离（宪法 1.1.4）
- 🟩 UI 监听状态变化刷新自己
- 🟩 主题系统统一样式
- 🟩 焦点管理

---

# 术语定义

## UiCommand

UI 命令事件，UI → Logic 的唯一交互通道。

不是 ECS 操作。UiCommand 是意图，不是直接状态修改。

关键属性：
- SelectUnit / MoveUnit / Attack / Skill / SelectTarget / Wait / Cancel / EndTurn

---

## ViewModel

游戏逻辑 → UI 的数据桥接层。

不是 ECS Component。ViewModel 是只读视图，UI 不直接 Query 游戏组件。

关键属性：
- SelectedUnitView / TurnInfoView / CombatPreviewView

---

## UiTheme

主题系统，统一样式配置。

不是硬编码颜色。所有颜色/字号/间距从 UiTheme 读取。

关键属性：
- 颜色常量 / 字号常量 / 阵营颜色映射

---

## UiFocusState

焦点管理，控制模态面板是否阻止游戏输入。

不是 TurnPhase。Focus 管理 UI 输入，Phase 管理游戏流程。

关键属性：
- blocks_input：是否阻止游戏输入

---

# 领域边界

## 本领域负责

- UiCommand Message 定义和处理
- ViewModel 定义和更新
- UiTheme 主题系统
- UiFocusState 焦点管理
- 高亮与标记（Selected / MovableRange / AttackRange）
- 战斗日志表现层
- 战斗飘字表现层
- 面板和组件库

## 本领域不负责

- 游戏逻辑执行（由 battle_rules / turn_rules 领域负责）
- 属性计算（由 stat_system 领域负责）
- 效果管线（由 effect_pipeline 领域负责）
- 数据存储（由各领域 Registry 负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| UI 意图 | UiCommand Message | battle / turn |
| 状态展示 | ViewModel 读取 | 各领域 |
| 战斗事件 | Message 监听 | combat_log / vfx |

---

# 生命周期

## UI 交互生命周期

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Idle | 无交互 | Interacting |
| Interacting | 用户正在操作 | Idle, Modal |
| Modal | 模态面板打开 | Idle |

## 状态转换图

Idle → Interacting → Idle
                    → Modal → Idle

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Idle | Interacting | 用户点击/悬停 |
| Interacting | Idle | 操作完成 |
| Interacting | Modal | 打开模态面板 |
| Modal | Idle | 关闭模态面板 |

---

# 不变量

## 不变量1：UI 不操作 ECS 🟥

宪法依据：1.1.4（逻辑与表现分离）、2.2.2（禁止业务逻辑直接操作 UI）

任意时刻：

UI 只通过 UiCommand Message 发出意图，不直接修改 ECS 状态。

违反表现：

UI 代码直接修改 HP、acted、TurnPhase 等游戏状态。

架构违规检测：

发现 UI 直接修改 ECS 状态时，必须停止。必须输出：

```
ARCHITECTURE VIOLATION: UI 直接修改 ECS 状态，违反"逻辑与表现分离"原则。
```

---

## 不变量2：ViewModel 隔离 🟥

宪法依据：1.1.4（逻辑与表现分离）、2.2.2（禁止 UI 保存业务真相）

UI 渲染时：

UI 只读 ViewModel，不直接 Query 游戏组件。

违反表现：

UI 代码直接读取 Attributes、ActiveBuffs 等组件。

架构违规检测：

发现 UI 直接 Query 游戏组件时，必须停止。必须输出：

```
ARCHITECTURE VIOLATION: UI 直接 Query 游戏组件，违反"ViewModel 隔离"原则。
```

---

## 不变量3：handle_ui_commands 仅玩家回合 🟥

UiCommand 处理时：

TurnState.current_faction == Player。

违反表现：

AI 回合时 UI 命令被执行。

---

## 不变量4：BlocksGameInput 阻止输入 🟥

模态面板打开时：

游戏输入系统跳过处理。

违反表现：

模态面板打开时，点击穿透到游戏层。

---

## 不变量5：UI 不保存业务真相 🟥

宪法依据：2.2.2（禁止 UI 保存业务真相或修改业务状态）

任意时刻：

UI 不缓存或保存业务状态的副本，ViewModel 是只读投影。

违反表现：

UI 组件缓存 HP 值并在本地计算伤害，而非从 ViewModel 读取。

架构违规检测：

发现 UI 保存业务状态副本时，必须停止。必须输出：

```
ARCHITECTURE VIOLATION: UI 保存业务真相，违反"UI 不保存业务真相"原则。
```

---

# 业务规则

## 规则1：UiCommand 是唯一交互通道 🟥

宪法依据：1.1.4（逻辑与表现分离）、5.0（通信三原则）

禁止：
- 🟥 UI 直接修改 ECS 状态
- 🟥 UI 直接调用游戏逻辑函数

必须：
- 所有 UI→Logic 交互通过 UiCommand Message
- handle_ui_commands 转化为游戏状态变更

---

## 规则2：ViewModel 刷新策略 🟩

禁止：
- 🟥 每帧重建 ViewModel

必须：
- SelectedUnitView 仅在 HoveredEntity 变化时刷新
- TurnInfoView 在 TurnState/TurnOrder 变化时刷新
- CombatPreviewView 仅在 SelectTarget 阶段显示

---

## 规则3：Cancel 上下文推断 🟩

禁止：
- Cancel 总是回到同一阶段

必须：
- 有 skill_id → SelectTarget 取消 → ActionMenu
- 有菜单实体 → ActionMenu 取消 → 回退位置 → SelectUnit
- 否则 → MoveUnit 取消 → SelectUnit

---

## 规则4：主题统一样式 🟩

禁止：
- 🟥 硬编码颜色/字号
- 🟥 绕过 UiTheme 直接写样式

必须：
- 所有颜色/字号/间距从 UiTheme 读取
- 换皮肤只改 UiTheme

---

# 流程管线

## UI 命令处理管线

UiCommand → 条件检查 → 状态变更 → 范围标记

### Step1：条件检查

输入：UiCommand + TurnPhase + TurnState
处理：检查是否玩家回合 + 阶段是否匹配
输出：是否执行
🟥 禁止：AI 回合执行 UI 命令

### Step2：状态变更

输入：UiCommand 内容
处理：修改 TurnPhase / 设置 CombatIntent / 生成 MovingUnit
输出：游戏状态变化
🟥 禁止：UI 直接修改 ECS

### Step3：范围标记

输入：命令类型 + 可达范围
处理：显示/清除移动范围/攻击范围
输出：视觉标记
🟩 允许：跳过范围标记（非关键路径）

---

## ViewModel 更新管线

状态变化 → 检测变化 → 重建 ViewModel → UI 刷新

### Step1：检测变化

输入：HoveredEntity / TurnState / TurnOrder
处理：比较新旧值
输出：是否需要刷新
🟥 禁止：每帧重建

### Step2：重建 ViewModel

输入：ECS 组件数据
处理：构建 ViewModel
输出：ViewModel 更新
🟥 禁止：UI 直接 Query

---

# 数据结构

## UiCommand（Message）

职责：UI → Logic 的唯一交互通道

结构：
- SelectUnit / MoveUnit / Attack / Skill / SelectTarget / Wait / Cancel / EndTurn

要求：
- 🟥 每个命令有明确的阶段映射
- 🟩 Cancel 支持上下文推断

---

## SelectedUnitView（Resource）

职责：选中单位信息视图

结构：
- name / race / class
- hp / mp / stamina
- core_attrs / combat_attrs / support_attrs
- skills / traits / buffs / equipment / inventory

要求：
- 🟩 仅在 HoveredEntity 变化时刷新

---

## UiTheme（Resource）

职责：统一样式配置

结构：
- 颜色常量（面板/按钮/文本/伤害/范围/高亮/进度条/Buff）
- 字号常量（large/medium/small/menu/log/damage/crit）
- 阵营颜色映射

要求：
- 🟥 所有 UI 组件从 UiTheme 读取样式

---

## UiFocusState（Resource）

职责：焦点管理

结构：
- blocks_input：是否阻止游戏输入

要求：
- 🟥 BlocksGameInput 组件标记模态面板
- 🟩 update_ui_focus_state 自动检测

---

# 禁止事项

🟥 禁止：UI 直接修改 ECS 状态

原因：UiCommand 是唯一交互通道（宪法 1.1.4）

违反后果：UI 绕过游戏逻辑直接修改状态，逻辑不一致

架构违规检测：

```
ARCHITECTURE VIOLATION: UI 直接修改 ECS 状态，违反"逻辑与表现分离"原则。
```

---

🟥 禁止：UI 直接 Query 游戏组件

原因：ViewModel 层隔离（宪法 1.1.4）

违反后果：UI 与游戏逻辑耦合，难以维护

架构违规检测：

```
ARCHITECTURE VIOLATION: UI 直接 Query 游戏组件，违反"ViewModel 隔离"原则。
```

---

🟥 禁止：硬编码颜色/字号

原因：UiTheme 统一样式

违反后果：换皮肤需要修改所有 UI 代码

---

🟥 禁止：AI 回合执行 UI 命令

原因：UI 命令仅限玩家回合

违反后果：AI 回合被 UI 干扰

---

🟥 禁止：每帧重建 ViewModel

原因：性能优化

违反后果：UI 每帧重建，性能下降

---

🟥 禁止：UI 保存业务真相

原因：宪法 2.2.2 禁止 UI 保存业务真相或修改业务状态

违反后果：UI 缓存与实际状态不一致

架构违规检测：

```
ARCHITECTURE VIOLATION: UI 保存业务真相，违反"UI 不保存业务真相"原则。
```

---

# AI 修改规则

## 如果新增 UI 面板

允许：
- 新增 ViewModel Resource
- 新增面板组件

禁止：
- 🟥 面板直接 Query 游戏组件
- 🟥 面板直接修改 ECS 状态

优先检查：
- ViewModel 是否正确隔离
- UiCommand 是否覆盖新交互
- UiFocusState 是否需要更新

---

## 如果新增 UiCommand

允许：
- 新增 UiCommand 变体
- 在 handle_ui_commands 中添加处理

禁止：
- 🟥 修改现有命令的处理逻辑
- 🟥 跳过阶段检查

优先检查：
- 命令与阶段映射
- 是否需要新增 ViewModel
- Cancel 上下文推断是否需要更新

---

## 如果修改主题

允许：
- 修改 UiTheme 常量值

禁止：
- 🟥 绕过 UiTheme 直接写样式

优先检查：
- 所有 UI 组件是否从 UiTheme 读取
- 阵营颜色映射是否正确
- 已行动单位变灰效果

---

## 如果测试失败

排查顺序：
1. 检查 UiCommand 是否正确发送
2. 检查 handle_ui_commands 是否在玩家回合执行
3. 检查 ViewModel 是否正确刷新
4. 检查 UiFocusState 是否阻止输入
5. 检查主题常量是否正确应用

测试要求（宪法 13.0.1-13.0.3）：
- 🟩 单元测试：验证 UiCommand 分发和 ViewModel 隔离
- 🟩 集成测试：验证 UI 交互流程
- 🟩 Bug 修复必须先编写重现测试（宪法 13.0.2）

---

# 宪法条款映射

| 宪法条款 | 本领域对应 |
|----------|-----------|
| 1.1.4 逻辑与表现分离 | UI 不操作 ECS，ViewModel 隔离 |
| 2.2.2 禁止 UI 保存业务真相 | ViewModel 是只读投影 |
| 5.0 通信三原则 | UiCommand Message |
| 2.1.5 Resource 不是全局仓库 | ViewModel/UiTheme 是只读投影 |

---

# 架构违规检测

| 违规行为 | 检测方式 | 输出 |
|----------|----------|------|
| UI 直接修改 ECS 状态 | 代码审查 | ARCHITECTURE VIOLATION: UI 直接修改 ECS 状态，违反"逻辑与表现分离"原则。 |
| UI 直接 Query 游戏组件 | 代码审查 | ARCHITECTURE VIOLATION: UI 直接 Query 游戏组件，违反"ViewModel 隔离"原则。 |
| UI 保存业务真相 | 代码审查 | ARCHITECTURE VIOLATION: UI 保存业务真相，违反"UI 不保存业务真相"原则。 |
