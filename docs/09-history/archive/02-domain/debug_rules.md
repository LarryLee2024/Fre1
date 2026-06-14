---
id: history.archive.debug_rules
title: debug_rules
status: archived
owner: domain-designer
created: 2026-06-14
updated: 2026-06-14
superseded_by: ../../02-domain/input/input-rules.md
---

# Debug 领域

Version: 1.0

Debug 领域管理运行时可视化调试工具，为开发者提供系统状态的实时观测能力。

核心原则：
- 调试工具只观测，不修改业务状态
- 可视化优先于日志堆砌
- 所有调试功能仅在开发模式下启用

---

# 术语定义

## DebugPanelState

控制 egui 调试面板显隐的 Resource 状态。

不是 DebugOverlay。DebugPanelState 管理 egui 窗口面板，DebugOverlay 管理 Gizmos 可视化。

关键属性：
- show_battle_debugger：F1 面板显隐
- show_buff_viewer：F2 面板显隐
- show_damage_attribute：F4 面板显隐
- show_turn_queue：F5 面板显隐
- damage_attribute_tab：F4 面板内 Tab 切换

---

## DebugOverlay

控制 Gizmos 可视化开关的 Resource 状态。

不是 DebugPanelState。DebugOverlay 管理游戏内覆盖层绘制，DebugPanelState 管理 egui 窗口面板。

关键属性：
- show_pathfinding：寻路路径可视化
- show_ai_intent：AI 决策可视化
- show_occupancy：占用网格可视化
- show_range_outline：范围轮廓可视化

---

## Debug Stepping

系统级单步调试能力，允许逐个 System 执行进行调试。

不是断点调试。Debug Stepping 在 ECS 调度层面暂停/单步，不是代码行级别的断点。

关键属性：
- 基于 Bevy 的 Stepping Resource 实现
- 支持 Update、FixedUpdate、PostUpdate 三个 Schedule
- 可通过 F6/F7 快捷键控制

---

## Gizmos

Bevy 内置的游戏内覆盖层绘制系统，用于在游戏画面上叠加调试可视化。

不是 egui。Gizmos 绘制在游戏世界坐标系中，egui 绘制在屏幕空间中。

关键特性：
- 每帧自动清除，无需手动清理
- 支持线框、形状、文字等绘制
- 在 Last Schedule 中执行，确保在所有逻辑更新之后绘制

---

## egui

即时模式 GUI 库，用于创建调试面板窗口。

不是 Gizmos。egui 绘制在屏幕空间，用于创建可交互的调试面板。Gizmos 绘制在游戏世界中，用于可视化调试信息。

关键特性：
- 即时模式，无需状态管理
- 支持窗口、按钮、复选框、折叠面板等控件
- 通过 bevy_egui 集成到 Bevy

---

## World Inspector

bevy-inspector-egui 提供的全局 Entity/Resource 查看器。

不是业务调试面板。World Inspector 是通用的 ECS 世界检查工具，用于查看所有 Entity 和 Resource 的状态。

关键特性：
- F12 快捷键切换
- 可查看所有 Entity 的组件
- 可查看所有 Resource 的状态
- 可实时修改 Resource 值

---

## Battle Record

结构化记录所有战斗事件的数据，用于调试和回放。

不是 CombatLog。Battle Record 是类型化的结构数据，CombatLog 是文本日志。

关键特性：
- 记录 DamageApplied、HealApplied、CharacterDied 等事件
- 包含 DamageBreakdown 修饰详情
- 用于 Debug 面板展示和 Battle Replay

---

## DamageBreakdown

伤害计算全链路的结构化记录，包含每步修饰的详细信息。

不是最终伤害值。DamageBreakdown 记录从原始值到最终值的完整修饰过程。

关键属性：
- base_amount：原始效果值
- modifiers：修饰符列表（每步的 before/after/rule_name）
- modified_amount：修饰后伤害
- actual_damage：实际扣血值

---

# 领域边界

## 本领域负责

- 运行时调试面板（egui 窗口）
- Gizmos 可视化覆盖层
- 系统单步调试（Stepping）
- 战斗状态观测（Battle Record 展示）
- 属性修饰来源追踪（Modifier 展示）
- 地形/占用网格可视化
- 游戏设置运行时修改

## 本领域不负责

- 业务逻辑修改（由各业务模块负责）
- 游戏状态变更（由 Battle/Turn/Character 等模块负责）
- 动画播放（由 UI/特效模块负责）
- 持久化存储（由 Save 模块负责）
- 输入处理（由 Input 模块负责）

## 跨领域通信方式

| 通知内容 | 通信方式 | 目标领域 |
|----------|----------|----------|
| 读取 BattleRecord | Resource 访问 | battle |
| 读取 TurnOrder | Resource 访问 | turn |
| 读取 Attributes | Component Query | character |
| 读取 ActiveBuffs | Component Query | buff |
| 读取 EquipmentSlots | Component Query | equipment |
| 读取 GridPosition | Component Query | character |
| 读取 CombatIntent | Resource 访问 | battle |
| 读取 TerrainGrid | Resource 访问 | map |
| 读取 OccupancyGrid | Resource 访问 | map |
| 修改 GameSettings | Resource 访问 | ui/settings |

---

# 生命周期

## 状态列表

| 状态 | 含义 | 可转换到 |
|------|------|----------|
| Inactive | 调试面板隐藏 | Active |
| Active | 调试面板显示 | Inactive |

## 状态转换图

```
Inactive ↔ Active
```

## 转换条件

| 从 | 到 | 条件 |
|----|-----|------|
| Inactive | Active | 用户按下对应快捷键（F1-F5） |
| Active | Inactive | 用户再次按下对应快捷键（F1-F5） |
| Inactive | Active | DebugStepping 启用（F6） |
| Active | Inactive | DebugStepping 禁用（F6） |

---

# 不变量

## 不变量1：调试面板只读性

任意时刻：

Debug 面板必须只读取业务状态，禁止修改任何业务数据。

违反表现：

修改 Attributes、ActiveBuffs、EquipmentSlots 等组件会导致游戏状态不一致。

---

## 不变量2：Gizmos 无副作用

任意时刻：

Gizmos 可视化系统必须是纯读取+绘制，禁止触发任何业务逻辑或状态变更。

违反表现：

Gizmos 系统修改游戏状态会导致逻辑与渲染耦合，破坏 Logic/Presentation 分离。

---

## 不变量3：Stepping 全局一致性

Stepping 启用时：

所有 Schedule（Update、FixedUpdate、PostUpdate）必须同时处于 Stepping 模式。

违反表现：

部分 Schedule Stepping 会导致调试状态不一致，难以复现问题。

---

## 不变量4：快捷键唯一绑定

每个快捷键（F1-F7、F12）必须只绑定一个调试功能。

违反表现：

快捷键冲突会导致功能无法正常切换，或意外触发其他调试功能。

---

## 不变量5：面板位置稳定性

所有调试面板必须有固定默认位置，避免重叠遮挡游戏画面。

违反表现：

面板位置随机或重叠会导致开发体验下降，难以同时观察多个面板。

---

# 业务规则

## 规则1：调试面板显隐控制

允许：
- 通过快捷键（F1-F5）切换面板显隐
- 通过 egui 面板按钮切换显隐
- 面板内 Tab 切换子视图

禁止：
- 禁止调试面板修改业务状态
- 禁止调试面板触发业务逻辑
- 禁止调试面板绕过 ViewModel 直接 Query 游戏组件

必须：
- 面板只读取 Resource 和 Component 数据
- 面板显隐状态保存在 DebugPanelState 中

---

## 规则2：Gizmos 可视化控制

允许：
- 通过 DebugOverlay 开关控制各类可视化
- 通过 F3 快捷键批量切换
- Gizmos 绘制线框、形状等基础图元

禁止：
- 禁止 Gizmos 系统修改游戏状态
- 禁止 Gizmos 系统触发业务逻辑
- 禁止 Gizmos 在非 Last Schedule 中执行

必须：
- Gizmos 只读取 Resource 和 Component 数据
- Gizmos 在 Last Schedule 中执行

---

## 规则3：Stepping 调试控制

允许：
- 通过 F6 暂停/继续
- 通过 F7 单步执行
- 通过 egui 面板按钮控制

禁止：
- 禁止在生产环境启用 Stepping
- 禁止单步执行时修改调试状态

必须：
- Stepping 启用时覆盖所有主要 Schedule
- Stepping 状态显示在调试面板中

---

## 规则4：数据源只读访问

允许：
- 读取 BattleRecord 展示战斗记录
- 读取 Attributes 展示属性值和修饰来源
- 读取 ActiveBuffs 展示 Buff 状态
- 读取 EquipmentSlots 展示装备状态
- 读取 GridPosition 展示单位位置

禁止：
- 禁止直接修改上述 Resource 或 Component
- 禁止通过 Debug 模块触发业务事件
- 禁止缓存调试数据的引用

必须：
- 每次渲染时重新查询最新数据
- 数据源变更时面板自动更新

---

## 规则5：快捷键绑定规范

允许：
- F1-F7 用于调试功能切换
- F12 用于 World Inspector
- 快捷键在 PreUpdate 或 PostUpdate 中处理

禁止：
- 禁止在业务系统中处理调试快捷键
- 禁止快捷键与游戏输入冲突
- 禁止同一快捷键绑定多个功能

必须：
- 快捷键处理系统只更新 DebugPanelState 或 DebugOverlay
- 快捷键处理使用 just_pressed 避免重复触发

---

# 流程管线

## 调试面板渲染管线

```
PreUpdate（快捷键处理）
↓
PostUpdate（egui 面板渲染）
↓
Last（Gizmos 绘制）
```

### Step1：快捷键处理

输入：ButtonInput<KeyCode>
处理：检测快捷键按下，更新 DebugPanelState 或 DebugOverlay
输出：状态变更
禁止：禁止在快捷键处理中执行业务逻辑

### Step2：egui 面板渲染

输入：DebugPanelState、业务 Resource/Component
处理：根据面板显隐状态渲染对应 egui 窗口
输出：屏幕上的调试面板
禁止：禁止在渲染中修改业务状态

### Step3：Gizmos 绘制

输入：DebugOverlay、业务 Resource/Component
处理：根据可视化开关绘制对应 Gizmos
输出：游戏画面中的覆盖层
禁止：禁止在绘制中修改业务状态

---

## Debug Stepping 执行管线

```
Main（begin_frame）
↓
Stepping 检查
↓
Update / FixedUpdate / PostUpdate
↓
单步或继续执行
```

### Step1：begin_frame

输入：Stepping Resource
处理：重置每帧执行计数
输出：Stepping 状态更新
禁止：禁止修改业务状态

### Step2：Stepping 检查

输入：Stepping 状态
处理：检查是否启用、是否需要暂停
输出：执行或暂停决策
禁止：禁止在检查中修改 Stepping 以外的状态

### Step3：System 执行

输入：目标 System
处理：执行单个 System
输出：System 执行结果
禁止：禁止在单步执行中触发额外的 Schedule 切换

---

# 数据结构

## DebugPanelState

职责：控制 egui 调试面板的显隐状态

结构：
- show_battle_debugger：bool — F1 面板显隐
- show_buff_viewer：bool — F2 面板显隐
- show_damage_attribute：bool — F4 面板显隐
- show_turn_queue：bool — F5 面板显隐
- damage_attribute_tab：u32 — F4 面板内 Tab 切换（0=Damage, 1=Attribute）

要求：
- 作为 Resource 存储
- 实现 Default trait
- 实现 Reflect trait 用于 Inspector

---

## DebugOverlay

职责：控制 Gizmos 可视化开关

结构：
- show_pathfinding：bool — 寻路路径可视化
- show_ai_intent：bool — AI 决策可视化
- show_occupancy：bool — 占用网格可视化
- show_range_outline：bool — 范围轮廓可视化

要求：
- 作为 Resource 存储
- 实现 Default trait
- 实现 Reflect trait 用于 Inspector

---

## GridViewerState

职责：控制 Grid Viewer 的视口滚动状态

结构：
- scroll_row：i32 — 视口起始行（包含）
- page_rows：i32 — 每次滚动的行数

要求：
- 作为 Resource 存储
- 实现 Default trait
- 避免每帧全量扫描，使用预计算优化

---

## DamageBreakdown（只读展示）

职责：展示伤害计算全链路的修饰详情

结构：
- base_amount：f32 — 原始效果值
- modifiers：Vec<ModifierEntry> — 修饰符列表
- modified_amount：f32 — 修饰后伤害
- actual_damage：f32 — 实际扣血值

要求：
- 只读展示，禁止修改
- 从 BattleRecord 中提取

---

# 禁止事项

禁止：调试面板修改任何业务状态

原因：违反 Logic/Presentation 分离原则，调试工具只观测不修改。

违反后果：游戏状态与调试视图不一致，导致逻辑错误难以追踪。

---

禁止：Gizmos 系统触发业务逻辑

原因：违反单向数据流原则，Gizmos 只负责可视化渲染。

违反后果：渲染系统与业务系统耦合，破坏架构分层。

---

禁止：在生产环境启用 Debug 面板

原因：调试工具影响性能，且可能暴露内部状态。

违反后果：发布版本包含调试代码，影响性能和安全性。

---

禁止：快捷键与游戏输入冲突

原因：调试快捷键不应影响正常游戏操作。

违反后果：玩家误触快捷键导致游戏状态异常。

---

禁止：调试面板缓存业务数据引用

原因：业务数据可能在每帧更新，缓存引用会导致显示过期数据。

违反后果：调试面板显示错误信息，误导开发者判断。

---

禁止：Stepping 调试中修改业务状态

原因：Stepping 用于观察状态流转，修改状态会破坏调试目的。

违反后果：无法准确复现和定位问题。

---

禁止：调试模块依赖业务模块的具体实现

原因：违反模块边界原则，调试模块应通过公共接口访问数据。

违反后果：业务模块重构会破坏调试模块，增加维护成本。

---

禁止：通过堆砌日志进行调试

原因：日志是临时调试手段，调试面板是永久解决方案。

违反后果：日志过多影响性能，且难以定位问题。

---

# AI 修改规则

## 如果新增调试面板

允许：
- 在 DebugPanelState 中添加新的 bool 字段
- 在 mod.rs 中添加条件渲染系统
- 在 viewers/ 下创建新的 viewer 模块
- 绑定新的快捷键（F 键）

禁止：
- 禁止在 viewer 中修改业务状态
- 禁止在 viewer 中触发业务逻辑
- 禁止创建与现有面板功能重叠的面板

优先检查：
- 面板是否只读取数据
- 面板是否在 PostUpdate 中执行
- 面板位置是否与现有面板冲突

---

## 如果新增 Gizmos 可视化

允许：
- 在 DebugOverlay 中添加新的 bool 字段
- 在 gizmos_viz.rs 中添加新的可视化系统
- 在 Last Schedule 中执行

禁止：
- 禁止在 Gizmos 系统中修改业务状态
- 禁止在 Gizmos 系统中触发业务逻辑
- 禁止在非 Last Schedule 中执行 Gizmos 绘制

优先检查：
- Gizmos 系统是否只读取数据
- Gizmos 系统是否在 Last Schedule 中执行
- 可视化颜色是否与现有可视化区分

---

## 如果修改快捷键绑定

允许：
- 修改 debug_hotkey_system 中的快捷键检测
- 修改 overlay.rs 中的 F3 快捷键处理
- 修改 stepping_control.rs 中的 F6/F7 快捷键处理

禁止：
- 禁止使用业务系统处理调试快捷键
- 禁止快捷键与游戏输入冲突
- 禁止同一快捷键绑定多个功能

优先检查：
- 快捷键是否在 PreUpdate 或 PostUpdate 中处理
- 快捷键是否使用 just_pressed 避免重复触发
- 快捷键是否与现有绑定冲突

---

## 如果修改调试面板位置

允许：
- 修改 egui::Window 的 default_pos 参数
- 调整面板默认大小

禁止：
- 禁止将面板放在游戏画面中心
- 禁止面板完全重叠遮挡游戏画面

优先检查：
- 面板位置是否与现有面板冲突
- 面板大小是否适合常见分辨率
- 面板是否会被游戏 UI 遮挡

---

## 如果测试失败

排查顺序：
1. 检查 DebugPanelState 或 DebugOverlay 字段是否正确初始化
2. 检查快捷键绑定是否正确（是否有冲突）
3. 检查 egui 面板是否在 PostUpdate 中执行
4. 检查 Gizmos 系统是否在 Last Schedule 中执行
5. 检查数据源（Resource/Component）是否存在且可访问
