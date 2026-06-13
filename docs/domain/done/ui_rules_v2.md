# UI 领域

Version: 2.0

## Purpose

UI 领域负责所有面板、行动菜单、浮窗、视觉效果的表现层。遵循 Logic / Presentation 分离原则：UI 不操作 ECS，只发出意图；ViewModel 隔离游戏逻辑与 UI 渲染。

---

## Glossary

| 术语 | 定义 | 易混淆项 |
|------|------|----------|
| UiCommand | UI 命令事件，UI → Logic 的唯一交互通道 | ≠ ECS 操作：Command 是意图，不是直接状态修改 |
| ViewModel | 游戏 Logic → UI 的数据桥接层 | ≠ ECS Component：ViewModel 是只读视图，UI 不直接 Query 游戏组件 |
| UiTheme | 主题系统，统一样式配置 | ≠ 硬编码颜色：所有颜色/字号/间距从 UiTheme 读取 |
| UiFocusState | 焦点管理，控制模态面板是否阻止游戏输入 | ≠ TurnPhase：Focus 管理 UI 输入，Phase 管理游戏流程 |

---

## Responsibilities

### Owns

- UiCommand Message 定义和处理
- ViewModel 定义和更新
- UiTheme 主题系统
- UiFocusState 焦点管理
- 高亮与标记（Selected / MovableRange / AttackRange）
- 战斗日志表现层
- 战斗飘字表现层
- 面板和组件库

### Does Not Own

- 游戏逻辑执行 → battle_rules / turn_rules
- 属性计算 → stat_system
- 效果管线 → effect_pipeline
- 数据存储 → 各领域 Registry

---

## Invariants

### INV-UI-01：UI 不操作 ECS 🟥

宪法：1.1.4, 2.2.2

UI 只通过 UiCommand Message 发出意图，不直接修改 ECS 状态。

违反：UI 代码直接修改 HP、acted、TurnPhase 等游戏状态。

### INV-UI-02：ViewModel 隔离 🟥

宪法：1.1.4

UI 只读 ViewModel，不直接 Query 游戏组件。

违反：UI 代码直接读取 Attributes、ActiveBuffs 等组件。

### INV-UI-03：handle_ui_commands 仅玩家回合 🟥

UiCommand 处理时，TurnState.current_faction == Player。

违反：AI 回合时 UI 命令被执行。

### INV-UI-04：BlocksGameInput 阻止输入 🟥

模态面板打开时，游戏输入系统跳过处理。

违反：模态面板打开时，点击穿透到游戏层。

### INV-UI-05：UI 不保存业务真相 🟥

宪法：2.2.2

UI 不缓存或保存业务状态的副本，ViewModel 是只读投影。

违反：UI 组件缓存 HP 值并在本地计算伤害。

### INV-UI-06：主题统一样式 🟩

所有颜色/字号/间距从 UiTheme 读取，禁止硬编码。

违反：换皮肤需要修改所有 UI 代码。

---

## State Machine

### UI 交互状态

| 状态 | 含义 | 转换到 |
|------|------|--------|
| Idle | 无交互 | Interacting |
| Interacting | 用户正在操作 | Idle, Modal |
| Modal | 模态面板打开 | Idle |

```
Idle → Interacting → Idle
                    → Modal → Idle
```

| 从 | 到 | 条件 |
|----|-----|------|
| Idle | Interacting | 用户点击/悬停 |
| Interacting | Idle | 操作完成 |
| Interacting | Modal | 打开模态面板 |
| Modal | Idle | 关闭模态面板 |

---

## Business Rules

### BR-UI-01：UiCommand 是唯一交互通道

- 所有 UI→Logic 交互通过 UiCommand Message
- handle_ui_commands 转化为游戏状态变更

### BR-UI-02：ViewModel 刷新策略

- SelectedUnitView 仅在 HoveredEntity 变化时刷新
- TurnInfoView 在 TurnState/TurnOrder 变化时刷新
- CombatPreviewView 仅在 SelectTarget 阶段显示
- 禁止每帧重建 ViewModel

### BR-UI-03：Cancel 上下文推断

- 有 skill_id → SelectTarget 取消 → ActionMenu
- 有菜单实体 → ActionMenu 取消 → 回退位置 → SelectUnit
- 否则 → MoveUnit 取消 → SelectUnit

### BR-UI-04：主题统一样式

- 所有颜色/字号/间距从 UiTheme 读取
- 换皮肤只改 UiTheme

---

## Pipelines

### UI 命令处理管线

UiCommand → 条件检查 → 状态变更 → 范围标记

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 条件检查 | UiCommand + TurnPhase + TurnState | 是否执行 | 禁止 AI 回合执行 UI 命令（INV-UI-03） |
| 状态变更 | UiCommand 内容 | 游戏状态变化 | 禁止 UI 直接修改 ECS（INV-UI-01） |
| 范围标记 | 命令类型 + 可达范围 | 视觉标记 | 非关键路径 |

### ViewModel 更新管线

状态变化 → 检测变化 → 重建 ViewModel → UI 刷新

| 步骤 | 输入 | 输出 | 约束 |
|------|------|------|------|
| 检测变化 | HoveredEntity / TurnState / TurnOrder | 是否需要刷新 | 禁止每帧重建 |
| 重建 ViewModel | ECS 组件数据 | ViewModel 更新 | 禁止 UI 直接 Query（INV-UI-02） |

---

## Data Model

### UiCommand（Message）

UI → Logic 的唯一交互通道。

- SelectUnit / MoveUnit / Attack / Skill / SelectTarget / Wait / Cancel / EndTurn
- 每个命令有明确的阶段映射
- Cancel 支持上下文推断

### SelectedUnitView（Resource）

选中单位信息视图。

- name / race / class
- hp / mp / stamina
- core_attrs / combat_attrs / support_attrs
- skills / traits / buffs / equipment / inventory
- 仅在 HoveredEntity 变化时刷新

### UiTheme（Resource）

统一样式配置。

- 颜色常量（面板/按钮/文本/伤害/范围/高亮/进度条/Buff）
- 字号常量（large/medium/small/menu/log/damage/crit）
- 阵营颜色映射

### UiFocusState（Resource）

焦点管理。

- blocks_input：是否阻止游戏输入
- BlocksGameInput 组件标记模态面板

---

## Cross Domain Contracts

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| UI 意图 | UiCommand Message | battle / turn |
| 状态展示 | ViewModel 读取 | 各领域 |
| 战斗事件 | Message 监听 | combat_log / vfx |

---

## Change Rules

### 新增 UI 面板

- 允许：新增 ViewModel Resource + 新增面板组件
- 禁止：面板直接 Query 游戏组件、面板直接修改 ECS 状态
- 检查：ViewModel 是否正确隔离、UiCommand 是否覆盖新交互、UiFocusState 是否需要更新

### 新增 UiCommand

- 允许：新增 UiCommand 变体 + 在 handle_ui_commands 中添加处理
- 禁止：修改现有命令的处理逻辑、跳过阶段检查
- 检查：命令与阶段映射、是否需要新增 ViewModel、Cancel 上下文推断是否需要更新

### 修改主题

- 允许：修改 UiTheme 常量值
- 禁止：绕过 UiTheme 直接写样式
- 检查：所有 UI 组件是否从 UiTheme 读取、阵营颜色映射是否正确

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
| INV-UI-01 | UI 直接修改 ECS 状态 | 逻辑与表现分离 | 改为通过 UiCommand Message |
| INV-UI-02 | UI 直接 Query 游戏组件 | ViewModel 隔离 | 改为从 ViewModel 读取 |
| INV-UI-05 | UI 保存业务真相 | ViewModel 是只读投影 | 改为每次从 ViewModel 读取 |

---

## Test Requirements

宪法：13.0.1-13.0.3

- 单元测试：验证 UiCommand 分发和 ViewModel 隔离
- 集成测试：验证 UI 交互流程
- Bug 修复必须先编写重现测试

排查顺序：
1. UiCommand 是否正确发送
2. handle_ui_commands 是否在玩家回合执行
3. ViewModel 是否正确刷新
4. UiFocusState 是否阻止输入
5. 主题常量是否正确应用
