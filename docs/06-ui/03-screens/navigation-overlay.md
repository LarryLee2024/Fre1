---
id: 06-ui.navigation-overlay
title: Navigation and Overlay Architecture — 导航与浮层架构
status: code-aligned
updated: 2026-06-21
owner: presentation-architect
created: 2026-06-20
tags:
  - ui
  - navigation
  - overlay
  - focus
  - screen-stack
---

# Navigation and Overlay Architecture — 导航与浮层架构

> **职责**: @presentation-architect | **上游**: ADR-055 §6 (UI Root 分层), §7 (状态分级) | domain rules §1 (术语), §2 (状态机), §5.4-§5.7 (流程定义), §INV-UI-006 | schema §7 (Navigation), §8 (Focus), §17 (Save)

---

> **实现状态**: ScreenStack, ScreenType, UiScreenState, ScreenLifecycle, 5 层 UI Root 和 Overlay 服务队列均已实现（code-aligned）。本节介绍的设计与代码一致。

## 1. 设计目的

SRPG 的导航体系比普通应用更复杂：多层状态（战斗/地图/菜单）叠加、Overlay（弹窗/通知/工具提示）独立于页面、手柄/键盘焦点导航。没有架构约束时，导航逻辑会散落在各个 Screen 中，导致：

- **P5**：导航状态无管理，Screen 堆栈混乱 → 返回逻辑不可预测（schema §2 P5）
- **P8**：焦点导航无数据结构支持 → 手柄/键盘导航无法实现（schema §2 P8）
- **Overlay 误销毁**：Screen 切换时 Notification/Tooltip 被误清理

本文档定义 ScreenStack 导航栈、Overlay 独立分层、Focus 焦点系统的完整架构。

---

## 2. UI Root 分层

### 2.1 三层独立的 UI Root

```
UiRoot
├── ScreenLayer     ← 当前 Screen（与 GameState 对应）
├── PopupLayer      ← Modal/Dialog（与 OverlayState 对应）
├── TooltipLayer    ← 独立 Tooltip
├── NotificationLayer ← Toast/Banner
└── DebugLayer      ← FPS/VM 状态（仅 dev feature）
```

**核心原则**：每层独立，Screen 销毁不影响 Tooltip/Notification/Debug。

### 2.2 各层生命周期

| 层 | 生命周期管理 | 销毁触发条件 | 不受影响的操作 |
|----|------------|-------------|--------------|
| ScreenLayer | OnEnter/OnExit GameState | GameState 切换 | Popup 打开、Tooltip 显示 |
| PopupLayer | PushOverlay/PopOverlay | Overlay 关闭 | Screen 切换、Notification 显示 |
| TooltipLayer | Focus/Hover 超时 | 焦点移出 | Screen 切换、Popup 显示 |
| NotificationLayer | 超时/手动关闭 | 超时/手动关闭 | Screen 切换 |
| DebugLayer | dev feature 开关 | Debug 开关 | 一切操作 |

（引用：ADR-055 §6 — UI Root 分层；domain rules §INV-UI-006 — Overlay 独立于 Screen）

### 2.3 INV-UI-006 例外：DamageTextOverlay

DamageTextOverlay（战斗伤害数字浮层）是 INV-UI-006 的显式例外。该 Overlay 挂在 BattleScreen 的 ScreenLayer 下，而非占用独立 Root 层。

**理由**：战斗伤害数字是 BattleScreen 场景特有的表现元素，生命周期天然与战斗绑定——每次战斗创建一个新的 DamageTextOverlay 实例并绑定到当前 BattleScreen，避免增加独立层带来的额外清理逻辑。

**约束条件**：详见 architecture.md §6.6。

---

## 3. ScreenStack 导航栈

### 3.1 设计目的

ScreenStack 管理所有 Screen 的推入/弹出，确保导航行为可预测、可持久化。

### 3.2 数据结构

```rust
#[derive(Resource, Debug, Clone, Reflect)]
#[reflect(Resource)]
pub struct ScreenStack {
    /// 屏幕类型栈，从底部（最早）到顶部（当前）。
    stack: Vec<ScreenType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum ScreenType {
    MainMenu,
    Battle,
    Inventory,
    Shop,
    Settings,
    SaveLoad,
}
```

### 3.3 操作接口

| 操作 | 行为 | 前置条件 | 后置条件 |
|------|------|---------|---------|
| push(screen) | 推入新 Screen 到栈顶。已在栈顶时不重复推入 | 无硬上限 | 当前 Screen → Background，新 Screen → Active |
| pop() | 弹出栈顶 Screen。保留根屏幕（len ≤ 1 时返回 None） | 栈深度 > 1 | 栈顶 Screen → Unloading → Destroyed，下层 Screen → Active |
| replace(screen) | 替换栈顶 Screen。栈为空时退化为 push | 无 | 旧栈顶（如有）→ Destroyed，新 Screen → Active |
| peek() | 查看栈顶 Screen | 栈非空 | 不变 |
| back() | 返回上一个 Screen | 栈深度 > 1 | 同 pop() |

### 3.4 导航流程

**Screen Push 流程**：
1. 校验 ScreenStack 深度（上限 10）
2. 当前 Active Screen → Background（暂停交互，保留 ViewModel）
3. 新 Screen：Defined → Loading（初始化 ViewModel，注册 Observer，加载资源）
4. 资源就绪 + ViewModel 初始化完成 → Active
5. 播放过渡动画

**Screen Pop 流程**：
1. 校验栈非空
2. 栈顶 Screen → Unloading（注销 Observer，清理定时器）
3. 清理完毕 → Destroyed（移除所有 Entity）
4. 下层 Screen Background → Active（恢复交互）
5. 刷新下层 Screen 的 ViewModel（可能 Dirty）

**Screen Replace 流程**：
1. 校验栈非空
2. 栈顶 Screen → Unloading → Destroyed（深度不变）
3. 新 Screen：Defined → Loading → Active
4. 播放过渡动画

(引用：domain rules §5.4 — Screen 导航流程；schema §7 — Navigation Schema)

### 3.5 过渡动画

| 过渡类型 | 参数 | 适用场景 |
|---------|------|---------|
| Instant | — | 战斗内快速切换 |
| Fade(duration) | 秒 | 菜单 → 战斗 |
| Slide(Direction) | 方向 | 菜单间切换（左/右滑动） |

不支持的过渡会回退到 Instant。

### 3.6 错误处理

| 错误场景 | 处理方式 |
|---------|---------|
| pop 保留根屏幕 | 栈深度 ≤ 1 时返回 None，不弹出 |
| replace 空栈 | 退化到 push 行为 |
| 重复 push 栈顶 | 静默忽略 |
| 过渡动画资源未就绪 | 回退到 Instant |

---

## 4. Overlay 独立层设计

### 4.1 Overlay 定义

Overlay 是**独立叠加层**，不挂在 Screen 下，生命周期独立于 Screen。

| 职责 | 不负责 |
|------|--------|
| Tooltip/Notification/Modal/Loading 等全局浮层 | Screen 内的布局 |
| 跨 Screen 共享的 UI 元素 | Screen 级业务逻辑 |
| 消费 Cue（伤害数字、状态提示） | Domain 业务计算 |

### 4.2 Overlay 层级顺序

从低到高（渲染层级递增）：

| 层级 | 层名称 | 内容 | 生命周期 |
|------|-------|------|---------|
| 0 | ScreenLayer | 当前 Screen | GameState 驱动 |
| 1 | PopupLayer | Modal/Dialog | OverlayState 驱动 |
| 2 | TooltipLayer | 工具提示 | Focus/Hover 驱动 |
| 3 | NotificationLayer | Toast/Banner 通知 | 超时/手动关闭 |
| 4 | DebugLayer | FPS/VM 调试 | dev feature 开关 |

(引用：ADR-055 §6 — UI Root 分层)

### 4.3 Overlay 详细定义

#### 4.3.1 TooltipOverlay

```
Input:  TooltipVm（内容、位置提示）
触发：   Focusable 元素获得焦点/hover 超过 0.3s
约束：   同一时间只显示一个 Tooltip（新 Tooltip 替换旧的）
销毁：   焦点移出时销毁
错误：   Tooltip 内容为空时不显示
```

(引用：domain rules §5.7 — Tooltip 显示流程)

#### 4.3.2 ModalOverlay

```
Input:  ModalVm（标题 Key、正文 Key、按钮列表）
触发：   ModalService 接收请求
约束：   Modal 栈深度上限 3
行为：   最底层 Modal 阻止 Screen 交互
输出：   UiAction::Confirm / UiAction::Cancel
错误：   栈深度超限时拒绝新 Modal
```

(引用：domain rules §5.6 — Modal 显示流程)

#### 4.3.3 NotificationOverlay

```
Input:  NotificationVm（消息 Key、优先级、持续时间、类型）
触发：   NotificationService 接收 NotificationVm
合并：   检查同类型 Notification 是否已存在
排序：   按优先级 Critical > Important > Normal
销毁：   超时/手动关闭
错误：   队列满时丢弃最低优先级 Notification
```

(引用：domain rules §5.5 — Notification 显示流程)

#### 4.3.4 DamageTextOverlay

```
Input:  CueTriggered（CueType::Popup）
触发：   Observer 监听 CueTriggered 事件
路由：   根据 CueType 分发到对应 Overlay
输出：   伤害数字/治疗数字/状态提示的浮动动画
销毁：   动画播完后延迟 despawn（Delayed Commands）
```

(引用：ADR-055 §3 — 与 Cue 的对接)

#### 4.3.5 LoadingOverlay

```
触发：   Screen 加载中（Loading 状态）
行为：   显示加载进度指示器
销毁：   Screen 进入 Active 状态
```

#### 4.3.6 DebugOverlay

```
激活：   dev feature + Debug 开关
内容：   FPS、ViewModel 状态、Entity 计数
位置：   屏幕角落，不遮挡交互
```

(引用：domain rules §9 — Bevy 0.19 特性映射，DiagnosticsOverlay)

---

## 5. Focus 导航系统

### 5.1 设计目的

Focus 系统为键盘/手柄导航提供数据结构支持，确保 Widget 焦点移动的确定性。

### 5.2 数据结构

```rust
/// 可聚焦组件
#[derive(Component, Reflect)]
pub struct Focusable {
    pub focus_id: FocusId,
    pub group: FocusGroupId,
    pub priority: u32,
}

/// 焦点组
#[derive(Component, Reflect)]
pub struct FocusGroup {
    pub group_id: FocusGroupId,
    pub navigation: FocusNavigation,  // Grid / Linear / Custom
    pub wrap: bool,                    // 是否循环导航
}

#[derive(Clone, Reflect, Default, PartialEq)]
pub enum FocusNavigation {
    #[default]
    Grid { cols: u32 },  // 网格导航
    Linear,               // 线性导航（上/下）
    Custom,               // 自定义导航规则
}
```

(引用：schema §8 — Focus Schema)

### 5.3 焦点流转规则

| 操作 | Grid 模式 | Linear 模式 | Custom 模式 |
|------|----------|------------|------------|
| 上移 | 上一行同列 | 上一个元素 | 自定义映射 |
| 下移 | 下一行同列 | 下一个元素 | 自定义映射 |
| 左移 | 同行左一列 | — | 自定义映射 |
| 右移 | 同行右一列 | — | 自定义映射 |
| 循环 | wrap=true 时越界回绕 | wrap=true 时首尾相接 | — |

### 5.4 Focus 优先级

- 高优先级 Focusable 在 Screen 激活时自动获得焦点
- 同一 FocusGroup 内优先级数值越大越优先
- 焦点导航不跨组（除非显式的 FocusGroup 切换）

### 5.5 用户输入处理流程

```
原始输入（键盘/鼠标/手柄）
    │
    ▼
Input System 将原始输入转换为 UiIntent
    │
    ▼
UiIntent 路由到当前活跃的 FocusGroup
    │
    ▼
FocusGroup 将 UiIntent 映射为 Widget 的 UiAction
    │
    ▼
UiAction 转换为 UiCommand
    │
    ▼
UiCommand 发送到 Domain 层执行
```

**失败处理**：UiIntent 无法映射时静默忽略（如当前无 Focusable 元素）。

(引用：domain rules §5.3 — 用户输入处理流程)

---

## 6. 导航的 Save/Persistence 策略

### 6.1 进入 Save 的数据

| 数据 | 策略 | 说明 |
|------|------|------|
| ScreenStack | 只保存当前 Screen 类型（不保存完整栈） | 存档加载后从该 Screen 重建 |
| ThemeName | 随 UiSettings 保存 | 主题选择持久化 |

### 6.2 不进入 Save 的数据

| 数据 | 原因 |
|------|------|
| Screen 完整栈 | 存档加载后导航栈语义不同 |
| Notification 队列 | 瞬时信息，加载后清空 |
| Modal 栈 | 瞬时交互，加载后清空 |
| Focus 状态 | 加载后重新计算 |
| Tooltip 状态 | 瞬态，不持久化 |

### 6.3 加载后的重建策略

```
存档加载
    │
    ▼
ScreenStack 清空 → 从 Save 读取当前 ScreenType
    │
    ▼
ScreenType → push(ScreenType) → Defined → Loading → Active
    │
    ▼
所有 ViewModel 通过 Domain 数据重新投影
    │
    ▼
所有 Dirty 标记设为 true → Widget 首次全量刷新
```

(引用：schema §17 — Save Compatibility)

---

## 7. Overlay 不变量交叉引用

| 不变量 | 说明 | 关联文件 |
|--------|------|---------|
| INV-UI-006 | Overlay 不挂在 Screen 下，Screen 销毁不影响 Overlay | ADR-055 §6, domain rules §INV-UI-006 |
| INV-UI-003 | UI 动画不驱动业务逻辑（Overlay 动画同理） | domain rules §INV-UI-003 |

---

*本文档由 @presentation-architect 维护。导航架构变更需通过 Presentation Architect 审查。*
