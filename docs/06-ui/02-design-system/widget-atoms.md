---
id: 06-ui.widget-atoms
title: Primitives — UI 原语层（Atoms 组件契约详细设计）
status: code-aligned
owner: presentation-architect
created: 2026-06-20
tags:
  - ui
  - widget
  - atom
  - contract
  - props
  - events
---

# Primitives — UI 原语层（组件契约详细设计）

> **职责**: @presentation-architect | **上游**: ADR-055 §5.4 (Contract 模式), §8 (Widget 分类), §12 (WidgetFactory) | domain rules §1 (统一术语), §8 (Contract 清单) | schema §4, §25 (WidgetFactory)

> **SSPEC参考**: docs/06-ui/07-specs/ — AI-Consumable Screen Specification 标准。新增 Screen 必须先写 SSPEC，见 ADR-066。

---

## 1. 设计目的

本文档定义 UI 原语层（Primitives）的组件契约。原语层是 UI 架构的最底层，包含 6 个原子控件系列（Button、ProgressBar、Panel、Text、List、Modal），是 `ui/` 中唯一允许直接操作 Bevy UI 底层实现（Node、Button、Interaction、BackgroundColor）的模块。

> **架构隔离规则**：Primitives 是 UI 层与底层 Bevy UI 实现的唯一桥梁。游戏业务控件（`widgets/`）和页面（`screens/`）不应绕过本层直接操作 Node/Button/Interaction。违反此规则会导致 UI 重构时波及范围失控。

每个 Primitives 控件（以下简称"Widget"，在本文档中仍保持与原来的 Widget Contract 模式一致）必须有明确的契约声明：
- **Props（输入）** — 所需的数据字段，Widget 通过 ViewModel 消费
- **Events（输出）** — 发射的交互事件，通过 UiAction 向上传递
- **Local State（本地状态）** — Widget 内部状态（如 hovered/selected），对 Screen 不可见
- **样式依赖** — 使用的 StyleToken（UiColors/UiSpacing/UiTypography）
- **变体** — 同一 Widget 的不同视觉/行为变体

Widget Contract 确保：
1. Widget 的输入输出边界清晰，Screen 组合时知道需要传递的数据
2. Widget 的本地状态不泄露到 Screen 层
3. Widget 的样式通过 StyleToken 引用而非硬编码，主题切换自动生效
4. Widget 变体可以共享同一 Contract 接口，Screen 在组合时选择变体

---

## 2. Button 系列

### 2.1 PrimaryButton

- **用途**: 主要操作按钮，如"确认"、"开始战斗"、"保存"
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | label_key | UiTextKey | 按钮文本本地化 Key |
  | enabled | bool | 是否可点击。当为 false 时禁用交互并应用 disabled 样式 |
  | width | Option\<Val\> | 固定宽度（None = 自适应内容） |
  | tooltip_key | Option\<UiTextKey\> | 长按时显示的 Tooltip Key |
- **Events**:
  | 事件 | 负载 | 触发条件 |
  |------|------|---------|
  | UiAction::Click | — | 鼠标左键点击 / 键盘 Enter / 手柄 A |
- **Local State**:
  | 状态 | 类型 | 初始值 | 说明 |
  |------|------|--------|------|
  | hovered | bool | false | 鼠标悬停时 true |
  | pressed | bool | false | 鼠标按下时 true（pointer_down 到 pointer_up 之间） |
- **样式依赖**:
  - UiColors::primary（背景色）/ UiColors::primary_hover（悬停）/ UiColors::text_primary（文本）
  - UiColors::text_disabled（禁用时文本色）
  - UiSpacing::button_padding / UiSpacing::border_radius
  - UiTypography::button_size
- **变体**:
  | 变体 | 视觉差异 | 适用场景 |
  |------|---------|---------|
  | PrimaryButton | 主色背景 + 白色文字 | 主要操作 |
  | SecondaryButton | 次要色边框 + 透明背景 | 次要操作 |
  | DangerButton | 危险色背景 + 白色文字 | 删除/解散 |

### 2.2 SecondaryButton

- **用途**: 次要操作按钮，如"取消"、"返回"
- **Props**: 同 PrimaryButton（label_key, enabled, width, tooltip_key）
- **Events**: 同 PrimaryButton（UiAction::Click）
- **Local State**: 同 PrimaryButton（hovered, pressed）
- **样式依赖**:
  - UiColors::secondary（边框色/文字色）/ UiColors::secondary_hover（悬停背景）
  - UiColors::text_disabled（禁用时文本色）
  - UiSpacing::button_padding / UiSpacing::border_radius
  - UiTypography::button_size
- **变体**: 无次级变体

### 2.3 DangerButton

- **用途**: 危险/删除操作，如"删除存档"、"解散队伍"
- **Props**: 同 PrimaryButton（label_key, enabled, width, tooltip_key）
- **Events**: 同 PrimaryButton（UiAction::Click）
- **Local State**: 同 PrimaryButton（hovered, pressed）
- **样式依赖**:
  - UiColors::danger（背景色）/ UiColors::danger_hover（悬停）
  - UiColors::text_primary（文本）
  - UiSpacing::button_padding / UiSpacing::border_radius
  - UiTypography::button_size
- **变体**: 无次级变体

### 2.4 IconButton

- **用途**: 仅图标的按钮，如关闭按钮、设置齿轮
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | icon_key | AssetKey | 图标资源 Key |
  | enabled | bool | 是否可点击 |
  | tooltip_key | Option\<UiTextKey\> | hover 时显示的提示 |
- **Events**: UiAction::Click
- **Local State**: hovered: bool, pressed: bool
- **样式依赖**:
  - UiColors::icon_primary（图标颜色）
  - UiSpacing::icon_size / UiSpacing::sm（内边距）
  - UiColors::icon_hover_bg（悬停背景）
- **变体**: CloseIcon, GearIcon, HelpIcon（仅 icon_key 不同，Contract 相同）

---

## 3. ProgressBar 系列

### 3.1 ProgressBar

- **用途**: 通用进度条，显示数值比例
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | current | f32 | 当前值 |
  | max | f32 | 最大值。约束：max > 0.0，current 在 0.0..=max 之间 |
  | show_label | bool | 是否显示"current/max"文本 |
  | height | Val | 进度条高度。默认 UiSpacing::md |
  | animate | bool | 是否播放过渡动画（true 时从旧值平滑过渡到新值） |
- **Events**: 无（纯展示组件）
- **Local State**: 无
- **样式依赖**:
  - UiColors::panel_bg（背景槽色）
  - UiColors::progress_fill（填充色——变体不同颜色不同）
  - UiSpacing::xs（进度条间距）/ UiSpacing::border_radius（圆角）
  - UiTypography::caption_size（标签文本字号）
- **变体**:
  | 变体 | 填充色 | 用途 |
  |------|--------|------|
  | HpBar | UiColors::hp_bar（红） | HP 值显示 |
  | MpBar | UiColors::mp_bar（蓝） | MP 值显示 |
  | ExpBar | UiColors::exp_bar（黄） | 经验值显示 |
  | CastBar | UiColors::cast_bar（紫） | 施法进度显示 |
- **Props 校验规则**:
  - 渲染比例 = clamp(current / max, 0.0, 1.0)
  - 当 max <= 0.0 时渲染为空条（比例 0.0）
  - 超出范围的值被 clamp 到 [0.0, max]

### 3.2 MultiSegmentBar

- **用途**: 多段进度条，显示分段比例（如 HP 中不同颜色段表示护盾/血量）
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | segments | Vec\<SegmentData\> | 分段数据 |
  | direction | SegmentDirection | 排列方向（Horizontal/Vertical） |
- **SegmentData**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | value | f32 | 段值 |
  | color | Color | 段颜色 |
  | label | Option\<UiTextKey\> | 段标签 |
- **Events**: 无
- **Local State**: 无
- **样式依赖**: 同 ProgressBar
- **变体**: 无（变体由调用方传入的 color 控制）

---

## 4. Panel 系列

### 4.1 Panel

- **用途**: 通用容器面板，带背景和边框
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | padding | Val | 内边距。默认 UiSpacing::panel_padding |
  | min_size | Option\<Vec2\> | 面板最小尺寸 |
  | children | Vec\<Widget\> | 子 Widget（通过 BSN Children 传递） |
- **Events**: 无
- **Local State**: 无
- **样式依赖**:
  - UiColors::panel_bg（背景色）
  - UiColors::panel_border（边框色）
  - UiSpacing::border_radius（圆角）
- **变体**:
  | 变体 | 视觉差异 | 用途 |
  |------|---------|------|
  | CardPanel | 白色背景 + 阴影 | 信息卡片 |
  | DarkPanel | 深色半透明背景 | 浮层/覆盖面板 |
  | TransparentPanel | 透明背景 + 仅边框 | 轻量分组 |

### 4.2 ScrollPanel

- **用途**: 可滚动的容器面板，用于内容超出可见区域时
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | padding | Val | 内边距 |
  | max_height | Val | 最大高度（超出时显示滚动条） |
  | scroll_step | f32 | 每次滚动的像素数 |
  | children | Vec\<Widget\> | 子 Widget |
- **Events**: 无
- **Local State**:
  | 状态 | 类型 | 初始值 | 说明 |
  |------|------|--------|------|
  | scroll_offset | f32 | 0.0 | 当前滚动偏移 |
  | content_height | f32 | 0.0 | 内容总高度 |
  | visible_height | f32 | 0.0 | 可见区域高度 |
- **样式依赖**: 同 Panel + UiColors::scrollbar_fill / UiSpacing::xs

### 4.3 TabPanel

- **用途**: 带标签页切换的容器面板
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | tabs | Vec\<TabDef\> | 标签页定义（Key + 内容 Widget） |
  | default_index | usize | 默认选中的标签页索引 |
- **TabDef**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | label_key | UiTextKey | 标签文本 Key |
  | content | Vec\<Widget\> | 标签页内容 |
- **Events**:
  | 事件 | 负载 | 触发条件 |
  |------|------|---------|
  | UiAction::ChangeTab | tab_index: usize | 点击不同标签页 |
- **Local State**:
  | 状态 | 类型 | 初始值 | 说明 |
  |------|------|--------|------|
  | active_tab | usize | default_index | 当前激活的标签页 |
- **样式依赖**:
  - UiColors::tab_active（激活标签色）/ UiColors::tab_inactive（未激活标签色）
  - UiColors::panel_border（标签分隔线）
  - UiSpacing::sm / UiTypography::button_size
- **变体**: 无

---

## 5. List 系列

### 5.1 VirtualList

- **用途**: 虚拟列表，只渲染可见项，适用于大量数据
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | item_count | usize | 总条目数 |
  | item_height | Val | 每个条目的固定高度 |
  | visible_height | Val | 可见区域高度 |
  | render_item | Box\<dyn Fn(usize) -> Entity\> | 给定索引创建条目的闭包 |
- **Events**: 无（条目事件由子 Widget 自身处理）
- **Local State**:
  | 状态 | 类型 | 初始值 | 说明 |
  |------|------|--------|------|
  | scroll_offset | f32 | 0.0 | 滚动偏移 |
  | first_visible | usize | 0 | 第一个可见项索引 |
  | last_visible | usize | 0 | 最后一个可见项索引 |
- **样式依赖**:
  - UiColors::scrollbar_fill
  - UiSpacing::xs（列表项间距）
- **变体**: 无

### 5.2 SelectList

- **用途**: 可选择的条目列表，支持单选/多选
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | items | Vec\<SelectItemDef\> | 条目定义 |
  | selection_mode | SelectionMode | 单选/多选 |
  | default_selected | Vec\<usize\> | 默认选中项索引 |
- **SelectItemDef**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | label_key | UiTextKey | 条目文本 Key |
  | icon_key | Option\<AssetKey\> | 条目图标 |
  | enabled | bool | 是否可选中 |
- **Events**:
  | 事件 | 负载 | 触发条件 |
  |------|------|---------|
  | UiAction::SelectItem | item_index: usize | 单选时点击条目 |
  | UiAction::ToggleItem | item_index: usize | 多选时点击条目 |
  | UiAction::ConfirmSelection | selected: Vec\<usize\> | 确认多选 |
- **Local State**:
  | 状态 | 类型 | 初始值 | 说明 |
  |------|------|--------|------|
  | selected | Vec\<usize\> | default_selected | 当前选中项索引集 |
  | hovered | Option\<usize\> | None | 当前悬停项 |
- **样式依赖**:
  - UiColors::list_selected_bg（选中背景）/ UiColors::list_hover_bg（悬停背景）
  - UiColors::text_primary / UiColors::text_secondary
  - UiSpacing::sm（条目间距）
- **变体**: 无

### 5.3 ItemGrid（InventoryGrid）

- **用途**: 网格布局的物品列表，用于背包/商店等场景
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | slots | Vec\<InventorySlotVm\> | 物品槽位数据 |
  | cols | u32 | 网格列数 |
  | selected | Option\<ItemId\> | 当前选中物品（来自 ViewModel） |
- **Events**:
  | 事件 | 负载 | 触发条件 |
  |------|------|---------|
  | UiAction::SelectItem | ItemId | 左键点击物品图标 |
  | UiAction::UseItem | ItemId | 双击物品 / 右键上下文菜单选择使用 |
  | UiAction::ShowContextMenu | (ItemId, Vec2) | 右键点击（显示上下文菜单的位置） |
- **Local State**:
  | 状态 | 类型 | 初始值 | 说明 |
  |------|------|--------|------|
  | hovered_slot | Option\<usize\> | None | 当前悬停槽位索引 |
  | drag_source | Option\<usize\> | None | 拖拽源槽位索引（拖拽开始后设置） |
- **样式依赖**:
  - UiColors::slot_bg（槽位背景）/ UiColors::slot_hover_bg（悬停背景）
  - UiColors::slot_selected_border（选中边框）
  - UiColors::rarity_common / rarity_uncommon / rarity_rare / rarity_epic / rarity_legendary（稀有度边框色）
  - UiSpacing::icon_size / UiSpacing::xs（网格间距）
- **变体**: 无

---

## 6. Text 系列

### 6.1 LocalizedText

- **用途**: 统一的本地化文本组件，对接 ADR-053
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | key | UiTextKey | 文本本地化 Key |
  | params | HashMap\<String, String\> | 参数插值变量 |
  | style | TextStyleDef | 文本样式（颜色、字体、字号、对齐） |
- **TextStyleDef**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | color | Color | 文本颜色。默认为 UiColors::text_primary |
  | font_size | FontSize | 字号。默认为 UiTypography::body_size |
  | font_source | FontSource | 字体。默认为 UiTypography::body_font |
  | alignment | TextAlignment | 对齐方式。默认 Left |
  | line_height | f32 | 行高倍率。默认 1.2 |
- **Events**: 无
- **Local State**: 无
- **样式依赖**:
  - 通过 TextStyleDef 间接引用 UiColors/UiSpacing/UiTypography
- **变体**:
  | 变体 | 默认 style 差异 | 用途 |
  |------|----------------|------|
  | HeadingText | heading_size + heading_font | 标题 |
  | BodyText | body_size + body_font | 正文 |
  | CaptionText | caption_size + body_font | 注释/小字 |
  | StatText | body_size + mono_font | 数值/统计文本 |
  | ButtonText | button_size + body_font | 按钮文本（Button Widget 内部使用） |
- **数据流**: 语言切换时自动刷新，Widget 代码不需要手动调用翻译函数

### 6.2 FormattedText

- **用途**: 支持富文本格式（颜色、图标嵌入、换行）的文本组件
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | segments | Vec\<TextSegment\> | 文本段落列表 |
- **TextSegment**:
  | 枚举变体 | 负载 | 说明 |
  |---------|------|------|
  | Text | (UiTextKey, Option\<TextStyleDef\>) | 普通文本段落 |
  | Icon | AssetKey | 嵌入式图标 |
  | LineBreak | — | 换行 |
- **Events**: 无
- **Local State**: 无
- **样式依赖**: 同 LocalizedText
- **变体**: 无

---

## 7. Tooltip

### 7.1 Tooltip

- **用途**: 工具提示，在元素 hover/focus 时显示上下文信息
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | content_key | UiTextKey | 提示内容 Key |
  | params | HashMap\<String, String\> | 内容参数插值 |
  | position_hint | TooltipPosition | 提示框相对触发元素的位置提示 |
  | max_width | Val | 提示框最大宽度 |
- **TooltipPosition**:
  | 变体 | 说明 |
  |------|------|
  | FollowCursor(Vec2) | 跟随鼠标偏移 |
  | AnchorTop | 在触发元素上方 |
  | AnchorBottom | 在触发元素下方 |
- **Events**: 无（Tooltip 不发射交互事件，仅展示信息）
- **Local State**: 无（由 TooltipService 管理生命周期）
- **样式依赖**:
  - UiColors::tooltip_bg（提示框背景）
  - UiColors::tooltip_border（提示框边框）
  - UiSpacing::sm（内边距）/ UiSpacing::border_radius（圆角）
  - UiTypography::caption_size
- **变体**:
  | 变体 | 差异 | 用途 |
  |------|------|------|
  | SimpleTooltip | 纯文本 | 简短提示 |
  | RichTooltip | 标题 + 描述 + 图标 | 详细说明（技能/物品信息） |

---

## 8. Modal

### 8.1 Modal 对话框

- **用途**: 模态弹窗，阻止用户与下方内容交互直到弹窗被处理
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | title_key | UiTextKey | 标题 Key |
  | body_key | UiTextKey | 正文 Key |
  | body_params | HashMap\<String, String\> | 正文参数插值 |
  | buttons | Vec\<ModalButtonConfig\> | 按钮配置列表。约束: 1 <= len <= 3 |
  | modal_type | ModalTypeVm | 弹窗类型（Confirm/Alert/Custom） |
- **ModalButtonConfig**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | label_key | UiTextKey | 按钮文本 Key |
  | action | UiAction | 点击后发送的动作 |
  | style | ButtonStyleVm | 按钮样式（Primary/Secondary/Danger） |
- **Events**:
  | 事件 | 负载 | 触发条件 |
  |------|------|---------|
  | UiAction::Confirm | — | 点击确认/主按钮 |
  | UiAction::Cancel | — | 点击取消/关闭按钮 |
- **Local State**: 无（Modal 是 Ephemeral 模式，由 ModalService 管理栈）
- **样式依赖**:
  - UiColors::modal_overlay_bg（遮罩背景半透明色）
  - UiColors::panel_bg（弹窗背景）/ UiColors::panel_border（弹窗边框）
  - UiSpacing::xl（弹窗外边距）/ UiSpacing::lg（内部间距）
  - UiTypography::heading_size（标题）/ UiTypography::body_size（正文）
- **变体**:
  | 变体 | 按钮配置 | 用途 |
  |------|---------|------|
  | ConfirmModal | [取消(Danger), 确认(Primary)] | 确认操作 |
  | AlertModal | [确定(Primary)] | 信息提示 |
  | CustomModal | 自定义按钮列表 | 灵活场景 |

---

## 9. Notification

### 9.1 Notification Widget

- **用途**: 非模态通知，自动消失，不影响用户当前操作
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | message_key | UiTextKey | 通知消息 Key |
  | params | HashMap\<String, String\> | 消息参数插值 |
  | priority | NotificationPriorityVm | 优先级（Normal/Important/Critical） |
  | notification_type | NotificationTypeVm | 类型（Toast/Banner/Popup） |
  | duration_secs | f32 | 显示时长。约束: > 0.0 |
- **Events**:
  | 事件 | 负载 | 触发条件 |
  |------|------|---------|
  | UiAction::Dismiss | — | 点击关闭按钮 / 手动关闭 |
- **Local State**:
  | 状态 | 类型 | 初始值 | 说明 |
  |------|------|--------|------|
  | elapsed | f32 | 0.0 | 已显示时间（秒），达到 duration_secs 时自动关闭 |
  | dismissed | bool | false | 是否已被用户关闭 |
- **样式依赖**:
  - UiColors::notification_bg / UiColors::notification_border
  - UiColors::notification_critical_bg / notification_important_bg / notification_normal_bg（优先级背景色）
  - UiSpacing::md（内边距）/ UiSpacing::border_radius
  - UiTypography::body_size / UiTypography::caption_size
- **变体**:
  | 变体 | 视觉差异 | 用途 |
  |------|---------|------|
  | ToastNotification | 右下角滑入 | 简短提示 |
  | BannerNotification | 顶部横幅 | 重要公告 |
  | PopupNotification | 屏幕中央 | 关键提示 |

---

## 10. Input 系列

### 10.1 TextInput

- **用途**: 文本输入框，用于角色命名、搜索、聊天
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | placeholder_key | UiTextKey | 占位文本 Key |
  | max_length | usize | 最大字符数 |
  | initial_value | String | 初始文本值 |
  | input_type | InputType | 输入类型（Text/Number/Password） |
- **Events**:
  | 事件 | 负载 | 触发条件 |
  |------|------|---------|
  | UiAction::TextChanged | String | 输入文本变更（每字符） |
  | UiAction::TextConfirmed | String | Enter 确认输入 |
- **Local State**:
  | 状态 | 类型 | 初始值 | 说明 |
  |------|------|--------|------|
  | value | String | initial_value | 当前输入文本 |
  | cursor_position | usize | 0 | 光标位置 |
  | is_focused | bool | false | 是否获得输入焦点 |
- **样式依赖**:
  - UiColors::input_bg（输入框背景）/ UiColors::input_border（输入框边框）
  - UiColors::input_focus_border（聚焦时边框色）
  - UiSpacing::sm（内边距）/ UiSpacing::border_radius
  - UiTypography::body_size
- **变体**: 无

### 10.2 Toggle / Checkbox

- **用途**: 开关/复选框，用于设置界面
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | label_key | UiTextKey | 标签文本 Key |
  | checked | bool | 当前选中状态 |
  | enabled | bool | 是否可交互 |
- **Events**:
  | 事件 | 负载 | 触发条件 |
  |------|------|---------|
  | UiAction::Toggle | checked: bool | 点击切换 |
- **Local State**:
  | 状态 | 类型 | 初始值 | 说明 |
  |------|------|--------|------|
  | hovered | bool | false | 悬停状态 |
- **样式依赖**:
  - UiColors::toggle_active（开启色）/ UiColors::toggle_inactive（关闭色）
  - UiColors::text_primary（标签文字）
  - UiSpacing::sm
- **变体**: Toggle（滑块样式）, Checkbox（勾选框样式）

---

## 11. Progress / Status 系列

### 11.1 TurnBar

- **用途**: 回合顺序条，显示角色行动顺序
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | turn_order | Vec\<TurnEntryVm\> | 回合顺序列表 |
  | current_turn | u32 | 当前回合数 |
  | active_character | Option\<CharacterId\> | 当前行动角色 |
- **TurnEntryVm**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | character_id | CharacterId | 角色 ID |
  | name_key | UiTextKey | 角色名 Key |
  | icon_key | AssetKey | 头像图标 |
  | is_active | bool | 是否当前回合 |
- **Events**: 无
- **Local State**: 无
- **样式依赖**:
  - UiColors::panel_bg（背景）/ UiColors::panel_border（边框）
  - UiColors::turn_active_border（当前角色高亮边框）
  - UiSpacing::sm（头像间距）
- **变体**: 无

### 11.2 StatusIcon

- **用途**: 状态图标（Buff/Debuff），显示在角色头像旁
- **Props**:
  | 字段 | 类型 | 说明 |
  |------|------|------|
  | buff_data | BuffVm | Buff 数据（icon_key, stacks, remaining_turns, is_debuff） |
- **Events**: 无
- **Local State**:
  | 状态 | 类型 | 初始值 | 说明 |
  |------|------|--------|------|
  | hovered | bool | false | 悬停状态（显示 Tooltip） |
- **样式依赖**:
  - UiColors::buff_icon / UiColors::debuff_icon
  - UiSpacing::icon_size（图标尺寸）
- **变体**: BuffIcon, DebuffIcon（由 is_debuff 决定颜色）

---

## 12. Widget Contract 验证规则

| # | 规则 | 触发时机 | 校验逻辑 |
|---|------|----------|----------|
| W-CON-01 | Props 字段不可为 Domain 类型 | 代码审查 | Props 字段类型不引用 Domain Component |
| W-CON-02 | Events 使用 UiAction 枚举 | 编译期 | Widget 输出的交互事件必须为 UiAction 变体 |
| W-CON-03 | Local State 不持久化 | 代码审查 | 本地状态不进入 Save/Replay |
| W-CON-04 | 样式引用 StyleToken | 代码审查 | Widget 样式字段引用自 UiColors/UiSpacing/UiTypography |
| W-CON-05 | 用户可见文本使用 UiTextKey | 代码审查 | label_key / message_key 等字段类型为 UiTextKey，非 String |
| W-CON-06 | Widget 禁止持有 Entity | 代码审查 | Widget 组件中无 Entity 字段 |
| W-CON-07 | Widget Props 有明确 Default | 测试 | Props 的 Default 实现不会导致渲染异常 |
| W-CON-08 | 变体共享同一 Contract 接口 | 代码审查 | 同一 Widget 的不同变体输入输出签名一致 |

---

## 13. Widget Contract 索引

| Widget | 文件名（目标） | 主要 Props | 输出 Events |
|--------|-------------|-----------|------------|
| PrimaryButton | `widgets/button/primary.rs` | label_key: UiTextKey, enabled: bool | UiAction::Click |
| SecondaryButton | `widgets/button/secondary.rs` | 同上 | UiAction::Click |
| DangerButton | `widgets/button/danger.rs` | 同上 | UiAction::Click |
| IconButton | `widgets/button/icon.rs` | icon_key: AssetKey, enabled: bool | UiAction::Click |
| ProgressBar | `widgets/progress_bar/mod.rs` | current: f32, max: f32, show_label: bool | 无 |
| MultiSegmentBar | `widgets/progress_bar/segment.rs` | segments: Vec\<SegmentData\> | 无 |
| Panel | `widgets/panel/mod.rs` | padding: Val, children: Vec\<Widget\> | 无 |
| ScrollPanel | `widgets/panel/scroll.rs` | max_height: Val, children: Vec\<Widget\> | 无 |
| TabPanel | `widgets/panel/tab.rs` | tabs: Vec\<TabDef\>, default_index: usize | UiAction::ChangeTab |
| VirtualList | `widgets/list/virtual_list.rs` | item_count: usize, item_height: Val | 无 |
| SelectList | `widgets/list/select.rs` | items: Vec\<SelectItemDef\>, selection_mode | UiAction::SelectItem |
| ItemGrid | `widgets/list/item_grid.rs` | slots: Vec\<InventorySlotVm\>, cols: u32 | UiAction::SelectItem, UiAction::UseItem |
| LocalizedText | `widgets/text/localized_text.rs` | key: UiTextKey, params: HashMap | 无 |
| FormattedText | `widgets/text/formatted.rs` | segments: Vec\<TextSegment\> | 无 |
| Tooltip | `widgets/tooltip/mod.rs` | content_key: UiTextKey, position_hint | 无 |
| Modal | `widgets/modal/mod.rs` | title_key: UiTextKey, buttons: Vec\<ModalButtonConfig\> | UiAction::Confirm, UiAction::Cancel |
| Notification | `widgets/notification/mod.rs` | message_key: UiTextKey, priority, duration_secs | UiAction::Dismiss |
| TextInput | `widgets/input/text_input.rs` | placeholder_key: UiTextKey, max_length | UiAction::TextChanged, UiAction::TextConfirmed |
| Toggle | `widgets/input/toggle.rs` | label_key: UiTextKey, checked: bool | UiAction::Toggle |
| TurnBar | `widgets/progress_bar/turn_bar.rs` | turn_order: Vec\<TurnEntryVm\> | 无 |
| StatusIcon | `widgets/progress_bar/status_icon.rs` | buff_data: BuffVm | 无 |

---

## 14. 与上游文档的交叉引用

| 概念 | 本文 § | 上游来源 |
|------|--------|---------|
| Widget Contract 模式 | §1, §12 | ADR-055 §5.4, domain rules §8 |
| PrimaryButton | §2.1 | domain rules §8 Contract 清单 |
| ProgressBar 变体 | §3.1 | ADR-055 §8 ProgressBar Widget |
| Modal 变体 | §8.1 | schema §4.9 ModalVm, schema §25 WidgetFactory |
| Notification 类型 | §9.1 | schema §4.8 NotificationTypeVm |
| LocalizedText | §6.1 | ADR-055 §4, domain rules §INV-UI-007, ADR-053 |
| UiBinding | — | projection-viewmodel.md §8, schema §23 |
| WidgetFactory | Architecture §9 | ADR-055 §12, schema §25 |

---

*本文档由 @presentation-architect 维护。新增 Widget 必须先在本文档定义 Contract，再写代码。*
