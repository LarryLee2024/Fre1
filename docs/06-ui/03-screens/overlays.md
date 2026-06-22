---
id: 06-ui.overlays
title: Overlay Design — 浮层详细设计
status: code-aligned
updated: 2026-06-21
owner: presentation-architect
created: 2026-06-20
tags:
  - ui
  - overlay
  - tooltip
  - notification
  - modal
  - loading
  - debug
---

# Overlay Design — 浮层详细设计

> **职责**: @presentation-architect | **上游**: ADR-055 §6 (UI Root 分层) | domain rules §5.5-§5.7 (流程定义), §INV-UI-006 | schema §4.8-§4.9 (NotificationVm, ModalVm) | navigation-overlay.md §4 (Overlay 独立层设计), §2 (UI Root 分层)

> **SSPEC参考**: docs/06-ui/07-specs/ — AI-Consumable Screen Specification 标准。新增 Screen 必须先写 SSPEC，见 ADR-066。

---

> **实现状态**: NotificationOverlay, TooltipOverlay, DamageTextOverlay, LoadingOverlay, DebugOverlay 均已实现（code-aligned）。本文档的设计与代码一致。

## 1. 设计目的

Overlay 是独立于 Screen 的叠加层，生命周期不依赖当前 Screen。本文档定义每个 Overlay 的：
- 用途与触发条件
- 在 5 层 UI Root 中的层级
- 数据源（ViewModel 还是直接消费 Cue）
- 生命周期（自动消失的触发条件）
- 交互行为（可点击、可拖拽、可关闭）

Overlay 设计遵循以下原则：
- Overlay 不挂在 Screen 下，Screen 销毁不影响 Overlay（INV-UI-006）
- Overlay 有独立的 Root 节点，不与其他层级共享渲染上下文
- Overlay 的生命周期由专用的 Service 管理（ModalService, NotificationService, TooltipService）

---

## 2. TooltipOverlay

### 2.1 用途

在用户 hover 或聚焦 UI 元素时，显示该元素的上下文信息提示。适用于技能说明、物品信息、按钮功能解释等场景。

### 2.2 层级

| 属性 | 值 |
|------|------|
| 层级名称 | TooltipLayer |
| 层级序号 | 2（从低到高：ScreenLayer:0 → PopupLayer:1 → TooltipLayer:2 → NotificationLayer:3 → DebugLayer:4） |
| Z 顺序 | 高于 PopupLayer，低于 NotificationLayer |

### 2.3 数据源

| 数据源 | 说明 |
|--------|------|
| TooltipVm | 核心数据：content_key (UiTextKey), params (HashMap), position_hint (TooltipPosition), max_width (Val) |
| 来源 | TooltipService 构建 TooltipVm。Service 接收 Focusable 元素的 tooltip_key 或 Widget 显式请求 |

### 2.4 触发条件

| 触发条件 | 延迟 | 说明 |
|---------|------|------|
| Focusable 元素获得焦点/hover 超过 0.3s | 0.3s（可通过 UiSettings.tooltip_delay_secs 调整） | 键盘导航选中或鼠标悬停 |
| Widget 显式请求 | 无 | Widget 主动调用 TooltipService::show() |
| 禁用状态 | 0.3s | 即使 enabled=false，也可显示 Tooltip 说明禁用原因 |

### 2.5 生命周期

| 阶段 | 行为 |
|------|------|
| 显示 | Timer 触发后创建 TooltipOverlay Entity，设置 TooltipLayer 中 |
| 更新 | 焦点在同一元素上移动时更新 TooltipVm.position_hint |
| 隐藏 | 焦点移出触发元素时立即销毁 |
| 替换 | 新 Tooltip 触发时替换当前 Tooltip（同一时间只显示一个） |
| 错误 | TooltipVm.content_key 为空时不显示 |

### 2.6 交互行为

| 行为 | 说明 |
|------|------|
| 鼠标交互 | Tooltip 本身不可点击（信息展示仅），鼠标移入 Tooltip 区域不销毁 |
| 位置计算 | 根据 position_hint + 触发元素位置计算实际渲染位置；超出屏幕边界时自动翻转 |
| 多元素 | 同一时间只显示一个 Tooltip。新的 Focusable hover 时立即替换旧的 |
| 延迟取消 | hover 移出触发元素后 0.1s 内回到触发元素不销毁（防止边缘闪烁） |

### 2.7 变体

| 变体 | Props 差异 | 用途 |
|------|-----------|------|
| SimpleTooltip | content_key 只含标题，无 params | 简短按钮说明 |
| RichTooltip | content_key + params + RichTooltipVm（标题 + 描述 + 图标） | 技能/物品详细说明 |

---

## 3. DamageTextOverlay

### 3.1 用途

战斗中显示伤害数字、治疗数字、状态提示的浮动动画。消费 Cue 系统的 CueType::Popup。

### 3.2 层级

> 🟥 **例外警告**：DamageTextOverlay 挂在 ScreenLayer 下，违反了 INV-UI-006（Overlay 必须有独立 Root 节点）。这是一项经架构评审批准的显式例外，详见 architecture.md §6.6。

| 属性 | 值 |
|------|------|
| 层级名称 | ScreenLayer（与 BattleScreen 同层，但 Z 排序更高） |
| 层级序号 | 0（附加到 BattleScreen 的专属浮层区，非独立层） |
| 说明 | DamageTextOverlay 直接挂在 BattleScreen 的根节点下，因为战斗数字是战斗场景特有的表现，不属于全局 Overlay |

### 3.3 数据源

| 数据源 | 说明 |
|--------|------|
| CueTriggered(CueType::Popup) | 核心数据。Observer 监听 Domain 发出的 CueTriggered 事件 |
| 事件字段 | damage_value: i32, target_pos: Vec2, cue_type: CueType::Popup, color_hint: Option\<Color\> |

### 3.4 触发条件

| 触发条件 | 说明 |
|---------|------|
| DamageApplied 触发 Cue | 伤害数字（红色，向下/上浮动） |
| HealingApplied 触发 Cue | 治疗数字（绿色，向上浮动） |
| StatusEffect 触发 Cue | 状态文字（紫色，"中毒！""沉默！"） |
| Miss/Dodge 触发 Cue | "未命中""闪避"文字（灰色） |

### 3.5 生命周期

| 阶段 | 行为 |
|------|------|
| 显示 | Observer 收到 CueTriggered → 创建 DamageText Entity（含 FloatingTextMotion 组件） |
| 动画 | 向上/下浮动 + 淡出（约 1.0s-1.5s） |
| 隐藏 | 动画播完后通过 Delayed Commands despawn |
| 批量 | 同一位置多个伤害数字合并显示（如 "15 + 8 + 3"） |
| 错误 | Cue 数据异常时不创建 Entity，记录日志 |

### 3.6 交互行为

| 行为 | 说明 |
|------|------|
| 纯展示 | 不可点击，不可交互 |
| 位置 | 浮动位置由 Cue 中的 target_pos 决定（一般为目标角色上方） |
| 合并 | 同一帧同一目标位置收到多个 Cue → 合并为一个浮动文字 |
| 速度 | 动画速度受 UiSettings.battle_speed 影响 |

### 3.7 样式依赖

- UiColors::damage_text（红色，伤害）
- UiColors::heal_text（绿色，治疗）
- UiColors::status_text（紫色，状态）
- UiColors::miss_text（灰色，未命中）
- UiTypography::heading_size（数字字号）
- FontSource::Family("mono")（数字等宽字体）

### 3.8 CalcBreakdown 扩展：结构化伤害明细

#### 3.8.1 设计目的

基础 DamageTextOverlay（§3.1-§3.7）仅显示最终数值（如 "-85"）。当玩家需要理解伤害/治疗/效果是如何计算得出时，需要呈现完整的计算推导过程。本节定义如何通过 Cue 系统传递 `CalcBreakdown` 并渲染为结构化面板。

#### 3.8.2 数据源与传递路径

计算明细通过 Cue 系统传递，不直接处理 Domain Event：

```
Domain 计算（Explain trait 实现）
    │  impl Explain for DamageCalculation { fn explain(&self) -> CalcBreakdown }
    ▼
CalcBreakdown 作为 Cue 的负载字段
    │  CueTriggered { cue_type: CueType::Popup, breakdown: Option<CalcBreakdown>, ... }
    ▼
UI Observer（Overlay 层）→ DamageTextOverlay
    │  Cue 中携带 breakdown 时，Overlay 同时创建浮动数字 + 明细面板入口
    ▼
玩家点击/悬停明细入口 → 展开 BreakdownPanel
```

| 数据字段 | 类型 | 来源 |
|---------|------|------|
| `damage_value` | i32 | Cue 主值（同现有） |
| `breakdown` | `Option<CalcBreakdown>` | Domain 的 `Explain::explain()` 返回值 |
| `formula_expr` | `String` | 公式表达式，如 "Price = Base * Reputation * Supply" |
| `inputs` | `Vec<BreakdownInput>` | 输入参数列表，每项含 name + value 字符串 |
| `steps` | `Vec<BreakdownStep>` | 中间步骤列表，每项含 label + operation + output |
| `output` | f32 | 最终计算结果 |
| `source_doc` | `Option<String>` | 可选的文档引用 |

（引用：`src/shared/diagnostics/explain.rs` — `CalcBreakdown`, `BreakdownInput`, `BreakdownStep`, `Explain` trait；经济学域的 `Price::explain()` 为参考实现）

#### 3.8.3 触发条件

| 触发条件 | 说明 |
|---------|------|
| Cue 携带 `Some(CalcBreakdown)` | 自动创建 BreakdownDetailPanel 入口（不新增浮动文字） |
| Cue 不携带 `breakdown` | 降级为纯数值显示，无明细 |
| 玩家请求展开 | 通过交互（悬停/点击）展开明细面板，不影响浮动文字动画 |

#### 3.8.4 渲染设计

**分层展示**（多行文本，按步骤类型着色）：

```
┌─ Damage Breakdown ─────────────────┐
│ Formula: ATK * SkillMul - DEF       │  ← formula_expr（标题色）
│                                     │
│ Inputs:                             │  ← 输入参数区（信息色）
│   base_atk      = 120               │
│   skill_mult    = 1.5               │
│   target_def    = 95                │
│                                     │
│ Steps:                              │  ← 中间步骤区（步骤色）
│   after_skill   = 120 * 1.5 = 180   │
│   after_defense = 180 - 95   = 85   │
│                                     │
│ Result: 85                          │  ← 最终结果（结果色，同浮动数值色）
└─────────────────────────────────────┘
```

**颜色编码**：

| 段 | 语义色 | StyleToken | 说明 |
|----|--------|-----------|------|
| 公式标题 | 标题色 | `UiColors::panel_header` | formula_expr 行 |
| 输入参数 | 信息色 | `UiColors::info_text` | name + value 对 |
| 中间步骤 | 步骤色 | `UiColors::step_text` | label + operation + output |
| 最终结果 | 结果色 | `UiColors::result_text`（或根据伤害/治疗类型着色） | 最终数值，与浮动文字颜色一致 |
| 分隔线 | 装饰色 | `UiColors::divider` | 分段分隔 |

#### 3.8.5 生命周期

| 阶段 | 行为 |
|------|------|
| 显示 | Cue 中携带 `breakdown: Some(...)` → DamageTextOverlay 创建浮动数字 + 在浮动数字旁创建 `BreakdownHint` 点击入口 |
| 展开 | 点击 `BreakdownHint` → 创建 BreakdownDetailPanel（Panel 原语 + 多行 LocalizedText），位于浮动数字附近 |
| 更新 | 不更新（CalcBreakdown 是一次性快照） |
| 关闭 | 与 DamageText 动画同步淡出；或玩家点击关闭按钮提前关闭 |
| 批量合并 | 同一目标多个 Cue 各自独立的 breakdown 不合并——合并仅在纯数值显示时生效 |

#### 3.8.6 交互行为

| 行为 | 说明 |
|------|------|
| BreakdownHint | 小型 "i" 图标按钮，悬停/点击展开明细 |
| BreakdownDetailPanel | 可点击关闭，不阻挡战斗操作 |
| 动画同步 | 明细面板跟随 DamageText 的淡出动画 |
| 无明细降级 | Cue 中没有 breakdown 字段时不显示 Hint 图标，降级为标准 §3.1-§3.7 的纯数值显示 |

#### 3.8.7 实现约束

- `CalcBreakdown` 结构定义在 `src/shared/diagnostics/explain.rs`，UI 模块通过 `use crate::shared::diagnostics::CalcBreakdown` 引用
- 禁止 UI 模块自行构建或修改 `CalcBreakdown`——它只能由 Domain 层的 `Explain::explain()` 生成并通过 Cue 传递
- 逐行渲染使用 `LocalizedText` 组件配合动态 params 实现，每行是一个独立 Text 节点
- 步骤数量无硬上限（但建议 UI 渲染限制最多 20 步，超出截断并显示 "..."）

---

## 4. NotificationOverlay

### 4.1 用途

非模态通知，自动消失，不影响用户当前操作。适用于获得物品、任务更新、等级提升等系统消息。

### 4.2 层级

| 属性 | 值 |
|------|------|
| 层级名称 | NotificationLayer |
| 层级序号 | 3（从低到高：ScreenLayer:0 → PopupLayer:1 → TooltipLayer:2 → NotificationLayer:3 → DebugLayer:4） |
| Z 顺序 | 高于 TooltipLayer，低于 DebugLayer |

### 4.3 数据源

| 数据源 | 说明 |
|--------|------|
| NotificationVm | 核心数据：message_key (UiTextKey), params (HashMap), priority (NotificationPriorityVm), notification_type (NotificationTypeVm), duration_secs (f32) |
| 来源 | NotificationService::show(vm: NotificationVm)。Service 接收来自 Projection 或直接来自 Observer 的通知请求 |
| 队列 | UiStore.notification_queue: Vec\<NotificationVm\> |

### 4.4 触发条件

| 触发条件 | 说明 |
|---------|------|
| ItemAcquired | 获得物品通知 |
| LevelUp | 升级通知（Critical 优先级） |
| QuestUpdated | 任务进度更新 |
| QuestCompleted | 任务完成通知（Important 优先级） |
| BuffApplied/Expired | 状态效果通知 |
| GoldChanged | 金币变更通知 |

### 4.5 生命周期

| 阶段 | 行为 |
|------|------|
| 入列 | NotificationService::show(vm) → 按优先级插入 UiStore.notification_queue |
| 合并 | 同类型、同 message_key 的 Notification 合并（刷新 duration_secs，不重复显示） |
| 显示 | 从队列头部取出，创建 NotificationOverlay Entity |
| 自动关闭 | duration_secs 计时结束 → 淡出动画 → despawn |
| 手动关闭 | 用户点击关闭按钮 → UiAction::Dismiss → 立即 despawn |
| 队列满 | 队列长度超过上限（默认 10）时丢弃最低优先级 Notification |
| 错误 | duration_secs <= 0.0 时使用默认值 3.0s |

### 4.6 交互行为

| 行为 | 说明 |
|------|------|
| 点击 | 部分 Notification 支持点击交互（如任务完成点击查看详情） |
| 关闭 | 显示关闭按钮（IconButton），点击后 UiAction::Dismiss |
| 堆叠 | 最多同时显示 3 条 Notification（Banner:1 + Toast:2），超出排队 |
| 位置 | ToastNotification 在右下角堆叠，BannerNotification 在顶部居中 |

### 4.7 变体

| 变体 | NotificationTypeVm | 位置 | 样式 |
|------|-------------------|------|------|
| ToastNotification | Toast | 右下角滑入 | 小卡片，auto-close |
| BannerNotification | Banner | 顶部横幅 | 全宽，auto-close 或手动 |
| PopupNotification | Popup | 屏幕中央 | 大卡片，需要手动关闭 |

---

## 5. LoadingOverlay

### 5.1 用途

Screen 加载中或存档加载/保存时，显示加载进度指示器，提示用户等待。

### 5.2 层级

| 属性 | 值 |
|------|------|
| 层级名称 | PopupLayer |
| 层级序号 | 1（从低到高：ScreenLayer:0 → PopupLayer:1，覆盖全部下层内容） |
| Z 顺序 | 高于全部 Screen 内容 |

### 5.3 数据源

| 数据源 | 说明 |
|--------|------|
| 无 ViewModel | LoadingOverlay 不消费业务 ViewModel |
| LoadingState Resource | 加载进度信息（current_step: u32, total_steps: u32, step_label: UiTextKey） |

### 5.4 触发条件

| 触发条件 | 说明 |
|---------|------|
| Screen Loading 状态 | ScreenStack::push 后，Screen 处于 Loading 状态时自动显示 |
| SaveGame / LoadGame | 确认存档操作后显示直到完成 |
| GameState 切换 | 资源密集型 GameState 切换时显示 |

### 5.5 生命周期

| 阶段 | 行为 |
|------|------|
| 显示 | LoadingState resource 创建（或 dirty=true）→ 创建 LoadingOverlay Entity |
| 更新 | LoadingState 进度更新 → ProgressBar 刷新 |
| 隐藏 | 加载完成 → LoadingState 移除 → 淡出动画 → despawn |
| 错误 | 加载超时（>30s）→ 显示"加载超时"提示 + 重试按钮 |

### 5.6 交互行为

| 行为 | 说明 |
|------|------|
| 不可交互 | LoadingOverlay 显示时阻止下层所有 Screen 和 Overlay 交互 |
| 进度 | 通过 LoadingProgressBar [ProgressBar] 显示加载进度 |
| 文字 | 通过 LoadingLabel [CaptionText] 显示当前加载步骤描述 |

### 5.7 样式依赖

- UiColors::modal_overlay_bg（遮罩背景）
- UiColors::panel_bg（加载面板）
- UiTypography::body_size / caption_size

---

## 6. DebugOverlay

### 6.1 用途

开发调试信息显示：FPS、ViewModel 状态、Entity 计数、性能数据。仅在 dev feature 下启用。

### 6.2 层级

| 属性 | 值 |
|------|------|
| 层级名称 | DebugLayer |
| 层级序号 | 4（最顶层，从低到高：ScreenLayer:0 → PopupLayer:1 → TooltipLayer:2 → NotificationLayer:3 → DebugLayer:4） |
| Z 顺序 | 覆盖全部下层 UI 元素 |

### 6.3 数据源

| 数据源 | 说明 |
|--------|------|
| Bevy Diagnostics | FPS, frame_time, entity_count, system_step |
| UiStore | 各 ViewModel 当前值的实时显示 |
| FocusState | 当前焦点元素信息 |

### 6.4 触发条件

| 触发条件 | 说明 |
|---------|------|
| dev feature + Debug 快捷键 | 默认 F12 切换 DebugOverlay 显示 |

### 6.5 生命周期

| 阶段 | 行为 |
|------|------|
| 显示 | Debug 快捷键按下 → 创建 DebugOverlay Entity（DebugLayer） |
| 更新 | 每帧刷新诊断数据 |
| 隐藏 | Debug 快捷键再次按下 → despawn |

### 6.6 交互行为

| 行为 | 说明 |
|------|------|
| 不可点击 | 仅展示信息，不拦截下层 UI 交互 |
| 位置 | 屏幕右下角，半透明背景 |
| 内容 | FPS 显示、ViewModel 快照、当前 Screen/Overlay 状态 |

### 6.7 样式依赖

- UiColors::debug_bg（半透明深色背景）
- FontSource::Family("mono")（等宽字体）
- UiTypography::caption_size

---

## 7. Overlay 一览表

| Overlay | 层级 | 数据源 | 生命周期管理 | 可点击 | 自动消失 | 最大并发数 |
|---------|------|--------|------------|--------|---------|-----------|
| TooltipOverlay | TooltipLayer (2) | TooltipVm | Focus/Hover 超时 | 否 | 是（焦点移出） | 1 |
| DamageTextOverlay | ScreenLayer (0) | CueTriggered | 动画播完 | 否 | 是（动画结束） | 无限制 |
| NotificationOverlay | NotificationLayer (3) | NotificationVm | 超时/手动关闭 | 部分可点击 | 是（duration） | 3 |
| LoadingOverlay | PopupLayer (1) | LoadingState | 加载完毕 | 否 | 是（加载完成） | 1 |
| DebugOverlay | DebugLayer (4) | Diagnostics | 快捷键切换 | 否 | 否（手动关闭） | 1 |

---

## 8. Overlay Service 结构

每个需要管理生命周期的 Overlay 对应一个 Service：

| Service | 管理对象 | 关键方法 |
|---------|---------|---------|
| TooltipService | TooltipOverlay | show(vm: TooltipVm), hide(), update_position(pos: Vec2) |
| NotificationService | NotificationOverlay | show(vm: NotificationVm), dismiss(id: NotificationId), clear_all() |
| ModalService | ModalOverlay | push(vm: ModalVm), pop(), peek() → Option\<ModalVm\> |
| LoadingService | LoadingOverlay | start(total_steps), update(progress), finish() |

### 8.1 Service 设计约束

- Service 是全局单例（Resource）
- Service 不直接管理 Entity，通过 Commands 和 Event 间接控制
- Service 方法的调用者可以是任何 Projection 或 Screen（但 Widget 不直接调用 Service）
- Service 内部保持 Overlay 状态的一致性和边界校验

---

## 9. Overlay 错误处理

| 错误场景 | 处理方式 |
|---------|---------|
| Tooltip 内容为空 | 不显示 Tooltip，静默忽略 |
| Notification 队列满（>10） | 丢弃最低优先级 Notification 并记录日志 |
| Modal 栈深度超限（>3） | 拒绝新 Modal，记录警告 |
| DamageText 动画组件缺失 | 跳过该 Entity，标记错误 |
| Loading 超时（>30s） | 显示"加载超时"提示 + 重试按钮 |
| DebugOverlay 诊断数据不可用 | 显示"N/A"替代数值，不崩溃 |

---

## 10. 验证规则

| # | 规则 | 校验逻辑 |
|---|------|----------|
| OVL-VAL-01 | Overlay 不挂在 Screen 下 | Overlay Entity 父节点是独立的 Root 层，非 Screen 节点 |
| OVL-VAL-02 | Overlay 不持久化 | Overlay 状态不进入 Save/Replay |
| OVL-VAL-03 | Overlay 不阻塞 Domain 逻辑 | Overlay 动画不影响 Domain 执行 |
| OVL-VAL-04 | Overlay 超出限制不崩溃 | 超限时优雅降级（丢弃/排队/记录日志） |

---

*本文档由 @presentation-architect 维护。新增 Overlay 必须先在本文档定义，再在 navigation-overlay.md §4 注册层级。*
