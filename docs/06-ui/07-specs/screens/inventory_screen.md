---
id: 07-specs.inventory-screen
title: InventoryScreen Specification — AI-Consumable Layout & Interaction Spec
status: active
owner: presentation-architect
created: 2026-06-22
tags:
  - ui
  - screen-spec
  - inventory
  - active
---

# InventoryScreen

> **职责**: @presentation-architect | **上游**: ADR-066 (Screen Spec), `07-specs/README.md` (总纲)
> **状态**: active。P0 字段全部通过。

**P0 字段**: 1-14 (Screen Header / ASCII Wireframe / Widget Tree / Flexbox Layout / Responsive Rules / Region Responsibility / Widget Contract / State Mapping / Focus Nav / Interaction Zones / Overlay / Lifecycle / Data Ownership / Layout Intent)
**P1 字段**: 15-17 (Scroll & Overflow / Event Contract / Screen Metrics)

---

## 1. Screen Header

| 属性 | 值 |
|------|-----|
| Screen Name | `InventoryScreen` — 对应 `GameState::Inventory` |
| Purpose | 管理角色装备与道具：查看/使用/丢弃背包物品，切换角色查看各成员物品清单 |
| Navigation | MainMenu → Inventory (UiCommand::OpenScreen(ScreenType::Inventory))；点击 Close 或 Esc 返回前一 Screen（MainMenuScreen） |
| GameState | `GameState::Inventory` |
| ScreenLayer 层级 | 0（主界面层） |
| 加载模式 | Ephemeral（每次进入 Inventory 重新 spawn） |
| 过渡动画 | 当前未实现，预留 Fade(0.3s) |
| 变体 | None |

---

## 2. ASCII Wireframe

> 纯文本线框图。所有区域必须命名（`widget_id`），禁止匿名面板。

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│  [inventory_header]                                                                   │
│  ┌───────────────────────────────────────────────────────────────┐  ┌──────────────┐ │
│  │  [inventory_header_title]   Inventory                        │  │  [inventory_  │ │
│  │                                                              │  │   header_close]│ │
│  │                                                              │  │      [X]      │ │
│  └───────────────────────────────────────────────────────────────┘  └──────────────┘ │
├──────────────────────────────────────────────────────────────────────────────────────┤
│  [inventory_body]                                                                     │
│  ┌───────────────────────────┐  ┌──────────────────────────────────────────────────┐  │
│  │  [inventory_character_    │  │  [inventory_area]                                │  │
│  │   list]                   │  │  ┌─────────────────────────────────────────────┐ │  │
│  │                           │  │  │  [inventory_filter_bar]                     │ │  │
│  │  Characters               │  │  │  ┌──────┬──────┬──────┬──────┬──────────┐  │ │  │
│  │  ┌─────────────────────┐  │  │  │  │ All  │ Con- │ Equi-│ Key  │ Material │  │ │  │
│  │  │ ▶ Aria        Lv.5  │  │  │  │  │      │ sum- │ pment│ Item │          │  │ │  │
│  │  │   [avatar] [name]   │  │  │  │  │      │ able │      │      │          │  │ │  │
│  │  ├─────────────────────┤  │  │  │  └──────┴──────┴──────┴──────┴──────────┘  │ │  │
│  │  │   Cade        Lv.5  │  │  │  └─────────────────────────────────────────────┘ │  │
│  │  │   [avatar] [name]   │  │  │                                                    │  │
│  │  ├─────────────────────┤  │  │  ┌─────────────────────────────────────────────┐ │  │
│  │  │   Lian        Lv.4  │  │  │  │  [inventory_item_grid]                      │ │  │
│  │  │   [avatar] [name]   │  │  │  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐      │ │  │
│  │  ├─────────────────────┤  │  │  │  │Potion│ │ Mana │ │Anti- │ │Phoenix│      │ │  │
│  │  │   Finn        Lv.4  │  │  │  │  │  x5  │ │Potion│ │dote  │ │ Down │      │ │  │
│  │  │   [avatar] [name]   │  │  │  │  │      │ │  x3  │ │  x2  │ │  x1  │      │ │  │
│  │  └─────────────────────┘  │  │  │  └──────┘ └──────┘ └──────┘ └──────┘      │ │  │
│  │                           │  │  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐      │ │  │
│  │                           │  │  │  │Potion│ │ Mana │ │      │ │      │      │ │  │
│  │                           │  │  │  │  x5  │ │Potion│ │      │ │      │      │ │  │
│  │                           │  │  │  │      │ │  x3  │ │      │ │      │      │ │  │
│  │                           │  │  │  └──────┘ └──────┘ └──────┘ └──────┘      │ │  │
│  │                           │  │  └─────────────────────────────────────────────┘ │  │
│  └───────────────────────────┘  └──────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────────────┤
│  [inventory_description_panel]                                                         │
│  ┌──────────────────────────────────────────────────────────────────────────────────┐ │
│  │  [inventory_item_detail]                                                         │ │
│  │  ┌─────┐ ┌──────────────────────────────────────────────────────────────────┐   │ │
│  │  │icon │ │ [inventory_item_detail_name]  Health Potion                     │   │ │
│  │  │     │ │ [inventory_item_detail_desc]  恢复 50 HP                        │   │ │
│  │  └─────┘ │ [inventory_item_detail_stats] 类型: 消耗品  |  重量: 0.5         │   │ │
│  │          └──────────────────────────────────────────────────────────────────┘   │ │
│  └──────────────────────────────────────────────────────────────────────────────────┘ │
├──────────────────────────────────────────────────────────────────────────────────────┤
│  [inventory_action_bar]                                                               │
│                                      ┌──────────────────────┐  ┌──────────────────┐   │
│                                      │  [inventory_use_btn] │  │ [inventory_      │   │
│                                      │      Use Item        │  │  drop_btn]       │   │
│                                      └──────────────────────┘  │     Drop        │   │
│                                                                └──────────────────┘   │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Region 索引

| widget_id | 类型 | 用途 | 对应 Wireframe 位置 |
|-----------|------|------|-------------------|
| `inventory_header` | Container | 包裹标题文本和关闭按钮，水平排列 | Wireframe 顶部第一行 |
| `inventory_header_title` | HeadingText | 显示 "Inventory" 标题文本 | Header 左侧 |
| `inventory_header_close` | IconButton | 关闭按钮，点击触发返回上一级 Screen | Header 右上角 |
| `inventory_body` | Container | 水平分栏包裹 CharacterList 和 InventoryArea | Wireframe 中部主体 |
| `inventory_character_list` | Container | 角色选择侧栏，垂直排列角色头像+名称+等级（固定 320px） | 左侧栏 |
| `inventory_character_list_label` | CaptionText | 角色列表标题标签 "Characters" | 角色列表顶部 |
| `inventory_character_rows` | Container | 包裹 CharacterRow 列表的容器 | 角色列表中部 |
| `inventory_character_row` | CharacterRow | 可点击角色行：头像 + 名称 + 等级（选中态高亮标记） | 角色列表中的行条目 |
| `inventory_character_empty` | EmptyWidget | 无角色时的空状态提示 | 角色列表为空时显示 |
| `inventory_area` | Container | Inventory 主区域容器，flex_grow:1 填充剩余空间 | 右侧主区域 |
| `inventory_filter_bar` | Container | 分类筛选 Tab 栏容器 | InventoryArea 顶部 |
| `inventory_filter_all` | TabButton | 全部物品筛选 Tab | FilterBar 第一个 Tab |
| `inventory_filter_consumable` | TabButton | 消耗品筛选 Tab | FilterBar 第二个 Tab |
| `inventory_filter_equipment` | TabButton | 装备筛选 Tab | FilterBar 第三个 Tab |
| `inventory_filter_key_item` | TabButton | 关键物品筛选 Tab | FilterBar 第四个 Tab |
| `inventory_filter_material` | TabButton | 材料筛选 Tab | FilterBar 第五个 Tab |
| `inventory_item_grid` | Container | 物品网格容器，包裹 ItemCell（Grid 布局） | InventoryArea 中部 |
| `inventory_item_cell` | ItemCell | 物品格子：图标 + 数量堆叠（选中态高亮） | 物品网格中的格子 |
| `inventory_empty_state` | EmptyInventoryWidget | 空背包/空筛选结果时的空状态提示 Widget | 物品网格为空时显示 |
| `inventory_loading` | Spinner | 数据加载中的加载指示器 | 物品网格区域 |
| `inventory_error_state` | Container | 数据加载失败的错误提示容器 | 物品网格区域 |
| `inventory_error_text` | BodyText | 错误信息文本 | 错误提示面板 |
| `inventory_retry_btn` | Button, Secondary | 重试按钮 | 错误提示面板 |
| `inventory_description_panel` | Container | 底部物品详情面板（固定 180px 高度） | Wireframe 底部上方 |
| `inventory_item_detail` | Container | 选中物品的详细说明区域 | 描述面板内容区 |
| `inventory_item_detail_icon` | Image | 选中物品的大图标 | 详情区左侧 |
| `inventory_item_detail_name` | HeadingText | 选中物品的名称 | 详情区上方右侧 |
| `inventory_item_detail_desc` | BodyText | 选中物品的描述文本 | 详情区中部右侧 |
| `inventory_item_detail_stats` | CaptionText | 选中物品的属性数据 | 详情区底部右侧 |
| `inventory_action_bar` | Container | 底部操作按钮栏 | Wireframe 最底部 |
| `inventory_use_btn` | Button, Primary | 使用物品按钮 | 操作栏左侧 |
| `inventory_drop_btn` | Button, Danger | 丢弃物品按钮 | 操作栏右侧 |

---

## 3. Widget Tree

> 标注 `[widget_id: WidgetType]` 的树结构。禁止隐藏节点，必须完整。

```
ScreenRoot                                                  [inventory_screen_root: Screen]
├── HeaderPanel                                             [inventory_header: Container]
│   ├── ScreenTitle                                         [inventory_header_title: HeadingText]
│   └── CloseButton                                         [inventory_header_close: IconButton]
├── BodyPanel                                               [inventory_body: Container]
│   ├── CharacterList                                       [inventory_character_list: Container]
│   │   ├── CharacterListLabel                              [inventory_character_list_label: CaptionText]
│   │   ├── CharacterRows                                   [inventory_character_rows: Container]
│   │   │   └── CharacterRow × N                            [inventory_character_row: CharacterRow]
│   │   └── EmptyState                                      [inventory_character_empty: EmptyWidget]
│   └── InventoryArea                                       [inventory_area: Container]
│       ├── FilterBar                                       [inventory_filter_bar: Container]
│       │   ├── AllTab                                      [inventory_filter_all: TabButton]
│       │   ├── ConsumableTab                               [inventory_filter_consumable: TabButton]
│       │   ├── EquipmentTab                                [inventory_filter_equipment: TabButton]
│       │   ├── KeyItemTab                                  [inventory_filter_key_item: TabButton]
│       │   └── MaterialTab                                 [inventory_filter_material: TabButton]
│       ├── ItemGrid                                        [inventory_item_grid: Container]
│       │   └── ItemCell × N                                [inventory_item_cell: ItemCell]
│       ├── EmptyState                                      [inventory_empty_state: EmptyInventoryWidget]
│       ├── LoadingIndicator                                [inventory_loading: Spinner]
│       └── ErrorPanel                                      [inventory_error_state: Container]
│           ├── ErrorText                                   [inventory_error_text: BodyText]
│           └── RetryButton                                 [inventory_retry_btn: Button, Secondary]
├── DescriptionPanel                                        [inventory_description_panel: Container]
│   └── ItemDetail                                          [inventory_item_detail: Container]
│       ├── ItemIcon                                        [inventory_item_detail_icon: Image]
│       ├── ItemName                                        [inventory_item_detail_name: HeadingText]
│       ├── ItemDescription                                 [inventory_item_detail_desc: BodyText]
│       └── ItemStats                                       [inventory_item_detail_stats: CaptionText]
└── ActionBar                                               [inventory_action_bar: Container]
    ├── UseButton                                           [inventory_use_btn: Button, Primary]
    └── DropButton                                          [inventory_drop_btn: Button, Danger]
```

### 3.1 Widget Type 索引

| widget_id | WidgetType | 定义位置 | 复用于 |
|-----------|-----------|---------|--------|
| `inventory_header_title` | `Atom: HeadingText` | `02-design-system/widget-atoms.md §HeadingText` | MainMenuScreen、各 Screen 标题 |
| `inventory_header_close` | `Atom: IconButton` | `02-design-system/widget-atoms.md §IconButton` | SettingsScreen、SaveLoadScreen |
| `inventory_character_list_label` | `Atom: CaptionText` | `02-design-system/widget-atoms.md §CaptionText` | 各 Screen 列表标签 |
| `inventory_character_row` | `Molecule: CharacterRow` | `02-design-system/widget-composites.md §TBD` | BattleScreen char_panel（参照） |
| `inventory_character_empty` | `Atom: EmptyWidget` | `02-design-system/widget-atoms.md §TBD` | 各 Screen 的空列表提示 |
| `inventory_filter_bar` | `Molecule: FilterBar` | `02-design-system/widget-composites.md §TBD` | ShopScreen |
| `inventory_filter_all` | `Atom: TabButton` | `02-design-system/widget-atoms.md §TabButton` | SettingsScreen |
| `inventory_filter_consumable` | `Atom: TabButton` | `02-design-system/widget-atoms.md §TabButton` | — |
| `inventory_filter_equipment` | `Atom: TabButton` | `02-design-system/widget-atoms.md §TabButton` | — |
| `inventory_filter_key_item` | `Atom: TabButton` | `02-design-system/widget-atoms.md §TabButton` | — |
| `inventory_filter_material` | `Atom: TabButton` | `02-design-system/widget-atoms.md §TabButton` | — |
| `inventory_item_cell` | `Molecule: ItemCell` | `02-design-system/widget-composites.md §TBD` | ShopScreen ShopItemCard |
| `inventory_empty_state` | `Molecule: EmptyInventoryWidget` | `02-design-system/widget-composites.md §TBD` | — |
| `inventory_loading` | `Atom: Spinner` | `02-design-system/widget-atoms.md §TBD` | 各 Screen 加载状态 |
| `inventory_error_text` | `Atom: BodyText` | `02-design-system/widget-atoms.md §BodyText` | 各 Screen 错误提示 |
| `inventory_retry_btn` | `Atom: Button (Secondary)` | `02-design-system/widget-atoms.md §Button` | 各 Screen 重试场景 |
| `inventory_item_detail_icon` | `Atom: Image` | `02-design-system/widget-atoms.md §Image` | 各 Screen 图标显示 |
| `inventory_item_detail_name` | `Atom: HeadingText` | `02-design-system/widget-atoms.md §HeadingText` | 各 Screen 标题 |
| `inventory_item_detail_desc` | `Atom: BodyText` | `02-design-system/widget-atoms.md §BodyText` | 各 Screen 正文 |
| `inventory_item_detail_stats` | `Atom: CaptionText` | `02-design-system/widget-atoms.md §CaptionText` | 各 Screen 辅助文本 |
| `inventory_use_btn` | `Atom: Button (Primary)` | `02-design-system/widget-atoms.md §Button` | — |
| `inventory_drop_btn` | `Atom: Button (Danger)` | `02-design-system/widget-atoms.md §Button` | — |

---

## 4. Flexbox Layout

> YAML 格式。每个 widget_id 必须有 direction / width / height / flex_grow / intent。

```yaml
## Flexbox Layout — InventoryScreen
## width/height: px 值或 "auto" 或 "fill"
## flex_grow: 0=不增长, 1=等分剩余空间, 2=双倍增长
## shrink: none/low/high — 收缩优先级

inventory_screen_root:
  direction: column
  width: 100%
  height: 100%
  flex_grow: 0
  intent: "Screen 根容器，占满视口，垂直排列 Header / Body / DescriptionPanel / ActionBar"

inventory_header:
  direction: row
  width: 100%
  height: 56
  flex_grow: 0
  shrink: none
  intent: "Header 栏，固定 56px 高度，水平排列标题（左）和关闭按钮（右）"

inventory_header_title:
  direction: row
  width: auto
  height: auto
  flex_grow: 1
  shrink: none
  intent: "标题文本，auto 宽度适配文本长度，flex_grow:1 推动关闭按钮到右侧"

inventory_header_close:
  direction: row
  width: 40
  height: 40
  flex_grow: 0
  shrink: none
  intent: "关闭按钮，固定 40x40 保证最小触摸目标"

inventory_body:
  direction: row
  width: 100%
  height: fill
  flex_grow: 1
  intent: "Body 主体区域，水平分栏（CharacterList + InventoryArea），占据除 Header/DescriptionPanel/ActionBar 外的所有剩余空间"

inventory_character_list:
  direction: column
  width: 320
  height: 100%
  flex_grow: 0
  shrink: none
  intent: "角色选择侧栏，固定 320px 宽度（与 BattleScreen char_panel 一致），垂直排列角色行"

inventory_character_list_label:
  direction: row
  width: 100%
  height: auto
  flex_grow: 0
  shrink: none
  intent: "角色列表标签文本，auto 高度，固定宽度与 character_list 同宽"

inventory_character_rows:
  direction: column
  width: 100%
  height: fill
  flex_grow: 1
  shrink: low
  intent: "角色行列表容器，垂直排列，可压缩但不可完全隐藏"

inventory_character_row:
  direction: row
  width: 320
  height: 56
  flex_grow: 0
  shrink: none
  intent: "单行角色条目，固定 56px 高度，水平排列头像(40x40) + 名称 + 等级"

inventory_character_empty:
  direction: column
  width: 100%
  height: auto
  flex_grow: 0
  intent: "角色列表空状态提示，auto 尺寸包裹文本内容"

inventory_area:
  direction: column
  width: fill
  height: 100%
  flex_grow: 1
  intent: "Inventory 主区域，弹性填充 CharacterList 右侧剩余水平空间，垂直排列 FilterBar + 内容区"

inventory_filter_bar:
  direction: row
  width: 100%
  height: 44
  flex_grow: 0
  shrink: none
  intent: "分类筛选 Tab 栏，固定 44px 高度，水平排列 5 个 TabButton"

inventory_filter_all:
  direction: row
  width: auto
  height: 44
  flex_grow: 0
  shrink: low
  intent: "全部物品 Tab 按钮，auto 宽度适配标签文本，可低优先级压缩"

inventory_filter_consumable:
  direction: row
  width: auto
  height: 44
  flex_grow: 0
  shrink: low
  intent: "消耗品 Tab 按钮，auto 宽度适配标签文本"

inventory_filter_equipment:
  direction: row
  width: auto
  height: 44
  flex_grow: 0
  shrink: low
  intent: "装备 Tab 按钮，auto 宽度适配标签文本"

inventory_filter_key_item:
  direction: row
  width: auto
  height: 44
  flex_grow: 0
  shrink: low
  intent: "关键物品 Tab 按钮，auto 宽度适配标签文本"

inventory_filter_material:
  direction: row
  width: auto
  height: 44
  flex_grow: 0
  shrink: low
  intent: "材料 Tab 按钮，auto 宽度适配标签文本"

inventory_item_grid:
  direction: row
  width: 100%
  height: fill
  flex_grow: 1
  intent: "物品网格容器，弹性填充 FilterBar 下方剩余空间，Grid wrap 布局排列 ItemCell"

inventory_item_cell:
  direction: column
  width: 72
  height: 80
  flex_grow: 0
  shrink: none
  intent: "单个物品格子，固定 72x80（图标 48x48 + 数量文本 12px），Grid 布局固定间距排列"

inventory_empty_state:
  direction: column
  width: 100%
  height: fill
  flex_grow: 1
  intent: "空背包/空筛选结果提示 Widget，居中显示图标 + 提示文本，占满剩余空间"

inventory_loading:
  direction: column
  width: 100%
  height: fill
  flex_grow: 1
  intent: "加载指示器，居中显示 Spinner，占满剩余空间"

inventory_error_state:
  direction: column
  width: 100%
  height: fill
  flex_grow: 1
  intent: "错误提示容器，居中排列错误文本 + 重试按钮，占满剩余空间"

inventory_error_text:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  intent: "错误信息文本，auto 尺寸适应错误内容"

inventory_retry_btn:
  direction: row
  width: 120
  height: 40
  flex_grow: 0
  shrink: none
  intent: "重试按钮，固定 120x40 确保最小触摸目标"

inventory_description_panel:
  direction: row
  width: 100%
  height: 180
  flex_grow: 0
  shrink: none
  intent: "底部物品详情面板，固定 180px 高度，容纳物品图标 + 名称 + 描述 + 属性"

inventory_item_detail:
  direction: row
  width: 100%
  height: 100%
  flex_grow: 1
  intent: "物品详情内容区，水平排列图标（左）和文本信息（右），填充整个面板"

inventory_item_detail_icon:
  direction: column
  width: 72
  height: 72
  flex_grow: 0
  shrink: none
  intent: "物品大图标，固定 72x72，左对齐显示"

inventory_item_detail_name:
  direction: row
  width: fill
  height: auto
  flex_grow: 0
  intent: "物品名称 HeadingText，auto 高度，位于详情右上方"

inventory_item_detail_desc:
  direction: row
  width: fill
  height: auto
  flex_grow: 0
  intent: "物品描述 BodyText，auto 高度，位于详情右中部"

inventory_item_detail_stats:
  direction: row
  width: fill
  height: auto
  flex_grow: 0
  intent: "物品属性 CaptionText，auto 高度，位于详情右下方"

inventory_action_bar:
  direction: row
  width: 100%
  height: 56
  flex_grow: 0
  shrink: none
  intent: "底部操作栏，固定 56px 高度，水平右对齐排列 Use + Drop 按钮"

inventory_use_btn:
  direction: row
  width: 140
  height: 40
  flex_grow: 0
  shrink: none
  intent: "使用物品按钮，固定 140x40，Primary 样式，大于最小触摸目标"

inventory_drop_btn:
  direction: row
  width: 140
  height: 40
  flex_grow: 0
  shrink: none
  intent: "丢弃物品按钮，固定 140x40，Danger 样式，与使用按钮同宽对齐"
```

---

## 5. Responsive Rules

| 条件 | 行为 | 影响区域 |
|------|------|---------|
| width < 1280px | strategy: "none" — 当前不实现响应式 | 全部 |
| height < 720px | strategy: "none" — 当前不实现响应式 | 全部 |

**最小支持分辨率**: 1280 x 720 (16:9)
**设计分辨率**: 1920 x 1080 (16:9)

---

## 6. Region Responsibility

> 每个 region 3-8 条职责。明确该区域"展示什么"和"不做什么"。

### 6.1 inventory_header

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示背包页面标题文本（LocalizationKey: `ui.inventory.title`） | Display | 页面标题缺失 |
| R02 | 展示关闭按钮（IconButton），图标为 "X" 或返回箭头 | Display | 用户无法关闭背包 |
| R03 | 响应关闭按钮点击，触发 `UiCommand::CloseScreen` | Interaction | 关闭功能失效 |
| R04 | 标题文本左对齐，关闭按钮右对齐，水平两端布局 | Layout | 布局违反预期 |

**不负责**:
- 角色选择逻辑（属于 inventory_character_list 职责）
- 物品使用/丢弃操作

### 6.2 inventory_character_list

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示角色列表标签（LocalizationKey: `ui.inventory.character_list`） | Display | 角色列表标签缺失 |
| R02 | 展示所有可操作角色的头像 + 名称 + 等级（CharacterRow），垂直排列 | Display | 角色信息不可见 |
| R03 | 标记当前选中角色行为高亮样式（▶ 前缀 + 强调色背景） | Display | 当前选中角色不可辨识 |
| R04 | 响应角色行点击，更新选中角色（更新 `selected_character`），触发 ItemGrid 刷新 | Interaction | 无法切换角色背包 |
| R05 | 管理角色行间焦点位移（ArrowUp/Down 导航） | Focus | 键盘无法在角色间移动 |
| R06 | 角色列表为空时显示空状态提示（inventory_character_empty） | Display | 空列表无提示 |

**不负责**:
- 物品网格/筛选逻辑（属于 inventory_area）
- 物品详情显示（属于 inventory_description_panel）

### 6.3 inventory_area

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示分类筛选 Tab 栏，可见 5 个筛选 Tab | Display | 用户无法筛选物品 |
| R02 | 响应 Tab 按钮点击，更新当前筛选条件（更新 `filter`），刷新 ItemGrid | Interaction | 筛选功能不生效 |
| R03 | 数据加载中显示加载指示器（inventory_loading） | Display | 加载无反馈 |
| R04 | 数据加载完成且有物品时显示 ItemGrid（inventory_item_grid + inventory_item_cell × N） | Display | 物品不可见 |
| R05 | 数据加载完成但无物品时显示空状态（inventory_empty_state: EmptyInventoryWidget） | Display | 空背包无提示 |
| R06 | 数据加载失败时显示错误面板（inventory_error_state: error_text + retry_btn） | Display | 错误无反馈/无法重试 |
| R07 | 管理 ItemCell 间焦点位移（Arrow keys 网格导航） | Focus | 键盘无法在物品间移动 |
| R08 | 管理 FilterBar Tab 按钮间焦点位移（ArrowLeft/Right 导航） | Focus | 键盘无法切换筛选 |

**不负责**:
- 角色选择（属于 inventory_character_list）
- 物品详情显示（属于 inventory_description_panel）

### 6.4 inventory_description_panel

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示选中物品的大图标（inventory_item_detail_icon） | Display | 物品图标缺失 |
| R02 | 展示选中物品的名称（inventory_item_detail_name） | Display | 物品名称缺失 |
| R03 | 展示选中物品的描述文本（inventory_item_detail_desc） | Display | 物品说明不可见 |
| R04 | 展示选中物品的属性数据：类型、重量、效果值等（inventory_item_detail_stats） | Display | 物品属性不可见 |
| R05 | 未选中物品时显示空状态提示文本（"Select an item to view details"） | Display | 空面板无引导提示 |

**不负责**:
- 物品使用/丢弃操作（属于 inventory_action_bar）
- 物品筛选（属于 inventory_area）

### 6.5 inventory_action_bar

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示 Use Item 按钮（Primary 样式） | Display | 使用物品入口缺失 |
| R02 | 展示 Drop 按钮（Danger 样式） | Display | 丢弃物品入口缺失 |
| R03 | 未选中物品时两个按钮均为 disabled 状态 | Interaction | 空选状态可误操作 |
| R04 | 选中物品后 UseButton 启用，点击触发 `UiCommand::UseItem` | Interaction | 使用功能失效 |
| R05 | 选中物品后 DropButton 启用，点击弹出 ModalOverlay（丢弃确认），确认后触发 `UiCommand::DropItem` | Interaction | 丢弃无二次确认导致误操作 |

**不负责**:
- 物品详情展示（属于 inventory_description_panel）
- 物品筛选（属于 inventory_area）

---

## 7. Widget Contract

> Inputs / Outputs / Selection Model。

### 7.1 inventory_header_title

```yaml
widget_id: inventory_header_title
widget_type: HeadingText
defined_in: "02-design-system/widget-atoms.md §HeadingText"

inputs:
  - name: text
    type: LocalizedText
    source: "ui.inventory.title"
    default: "Inventory"

outputs: []

selection_model:
  type: none
```

### 7.2 inventory_header_close

```yaml
widget_id: inventory_header_close
widget_type: IconButton
defined_in: "02-design-system/widget-atoms.md §IconButton"

inputs:
  - name: icon
    type: IconType::Close
    source: "Theme Icon set"
    default: "close_x"

outputs:
  - name: clicked
    type: UiCommand::CloseScreen
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.3 inventory_character_list_label

```yaml
widget_id: inventory_character_list_label
widget_type: CaptionText
defined_in: "02-design-system/widget-atoms.md §CaptionText"

inputs:
  - name: text
    type: LocalizedText
    source: "ui.inventory.character_list"
    default: "Characters"

outputs: []

selection_model:
  type: none
```

### 7.4 inventory_character_row

```yaml
widget_id: inventory_character_row
widget_type: CharacterRow
defined_in: "02-design-system/widget-composites.md §TBD — CharacterRow Molecule"

inputs:
  - name: character_id
    type: EntityId
    source: "PartyVm.character_ids[n]"
    default: None
  - name: name
    type: LocalizedText
    source: "PartyVm.characters[n].name"
    default: ""
  - name: level
    type: u32
    source: "PartyVm.characters[n].level"
    default: 1
  - name: avatar_icon
    type: IconType
    source: "PartyVm.characters[n].portrait"
    default: "default_avatar"
  - name: is_selected
    type: bool
    source: "InventoryScreenState.selected_character == character_id"
    default: false

outputs:
  - name: selected
    type: UiCommand::SelectCharacter
    payload: EntityId
    trigger: OnLeftClick

  - name: hovered
    type: HoveredCharacter
    payload: EntityId
    trigger: OnHover

selection_model:
  type: none
```

### 7.5 inventory_character_empty

```yaml
widget_id: inventory_character_empty
widget_type: EmptyWidget
defined_in: "02-design-system/widget-atoms.md §TBD — EmptyWidget Atom"

inputs:
  - name: text
    type: LocalizedText
    source: "ui.inventory.character_list.empty"
    default: "No characters available"

outputs: []

selection_model:
  type: none
```

### 7.6 inventory_filter_all

```yaml
widget_id: inventory_filter_all
widget_type: TabButton
defined_in: "02-design-system/widget-atoms.md §TabButton"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.inventory.filter.all"
    default: "All"
  - name: is_active
    type: bool
    source: "InventoryScreenState.filter == FilterType::All"
    default: true

outputs:
  - name: selected
    type: UiCommand::FilterChanged
    payload: FilterType::All
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.7 inventory_filter_consumable

```yaml
widget_id: inventory_filter_consumable
widget_type: TabButton
defined_in: "02-design-system/widget-atoms.md §TabButton"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.inventory.filter.consumable"
    default: "Consumable"
  - name: is_active
    type: bool
    source: "InventoryScreenState.filter == FilterType::Consumable"
    default: false

outputs:
  - name: selected
    type: UiCommand::FilterChanged
    payload: FilterType::Consumable
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.8 inventory_filter_equipment

```yaml
widget_id: inventory_filter_equipment
widget_type: TabButton
defined_in: "02-design-system/widget-atoms.md §TabButton"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.inventory.filter.equipment"
    default: "Equipment"
  - name: is_active
    type: bool
    source: "InventoryScreenState.filter == FilterType::Equipment"
    default: false

outputs:
  - name: selected
    type: UiCommand::FilterChanged
    payload: FilterType::Equipment
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.9 inventory_filter_key_item

```yaml
widget_id: inventory_filter_key_item
widget_type: TabButton
defined_in: "02-design-system/widget-atoms.md §TabButton"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.inventory.filter.key_item"
    default: "Key Item"
  - name: is_active
    type: bool
    source: "InventoryScreenState.filter == FilterType::KeyItem"
    default: false

outputs:
  - name: selected
    type: UiCommand::FilterChanged
    payload: FilterType::KeyItem
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.10 inventory_filter_material

```yaml
widget_id: inventory_filter_material
widget_type: TabButton
defined_in: "02-design-system/widget-atoms.md §TabButton"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.inventory.filter.material"
    default: "Material"
  - name: is_active
    type: bool
    source: "InventoryScreenState.filter == FilterType::Material"
    default: false

outputs:
  - name: selected
    type: UiCommand::FilterChanged
    payload: FilterType::Material
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.11 inventory_item_cell

```yaml
widget_id: inventory_item_cell
widget_type: ItemCell
defined_in: "02-design-system/widget-composites.md §TBD — ItemCell Molecule"

inputs:
  - name: item_id
    type: ItemId
    source: "InventoryVm.items[n].id"
    default: None
  - name: icon
    type: IconType
    source: "InventoryVm.items[n].icon"
    default: "default_item"
  - name: quantity
    type: u32
    source: "InventoryVm.items[n].quantity"
    default: 1
  - name: is_selected
    type: bool
    source: "InventoryScreenState.selected_item == item_id"
    default: false

outputs:
  - name: selected
    type: UiCommand::SelectItem
    payload: ItemId
    trigger: OnLeftClick

  - name: hovered
    type: UiCommand::ShowTooltip
    payload: "{ item_id: ItemId }"
    trigger: OnHover(500ms)

selection_model:
  type: none
```

### 7.12 inventory_empty_state

```yaml
widget_id: inventory_empty_state
widget_type: EmptyInventoryWidget
defined_in: "02-design-system/widget-composites.md §TBD — EmptyInventoryWidget Molecule"

inputs:
  - name: icon
    type: IconType
    source: "hardcoded"
    default: "empty_box"
  - name: title
    type: LocalizedText
    source: "ui.inventory.empty.title"
    default: "No Items"
  - name: description
    type: LocalizedText
    source: "ui.inventory.empty.description"
    default: "This character has no items in their inventory."

outputs: []

selection_model:
  type: none
```

### 7.13 inventory_item_detail_name

```yaml
widget_id: inventory_item_detail_name
widget_type: HeadingText
defined_in: "02-design-system/widget-atoms.md §HeadingText"

inputs:
  - name: text
    type: LocalizedText
    source: "InventoryVm.selected_item.name"
    default: ""

outputs: []

selection_model:
  type: none
```

### 7.14 inventory_item_detail_desc

```yaml
widget_id: inventory_item_detail_desc
widget_type: BodyText
defined_in: "02-design-system/widget-atoms.md §BodyText"

inputs:
  - name: text
    type: LocalizedText
    source: "InventoryVm.selected_item.description"
    default: ""

outputs: []

selection_model:
  type: none
```

### 7.15 inventory_item_detail_stats

```yaml
widget_id: inventory_item_detail_stats
widget_type: CaptionText
defined_in: "02-design-system/widget-atoms.md §CaptionText"

inputs:
  - name: text
    type: LocalizedText
    source: "InventoryVm.selected_item.stats"
    default: ""

outputs: []

selection_model:
  type: none
```

### 7.16 inventory_use_btn

```yaml
widget_id: inventory_use_btn
widget_type: Button, Primary
defined_in: "02-design-system/widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.inventory.use"
    default: "Use Item"
  - name: enabled
    type: bool
    source: "InventoryScreenState.selected_item != None"
    default: false

outputs:
  - name: clicked
    type: UiCommand::UseItem
    payload: "{ item_id: ItemId, character_id: EntityId }"
    trigger: OnLeftClick
    conditions:
      - use_button.enabled == true

selection_model:
  type: none
```

### 7.17 inventory_drop_btn

```yaml
widget_id: inventory_drop_btn
widget_type: Button, Danger
defined_in: "02-design-system/widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.inventory.drop"
    default: "Drop"
  - name: enabled
    type: bool
    source: "InventoryScreenState.selected_item != None"
    default: false

outputs:
  - name: clicked
    type: UiCommand::OpenOverlay(OverlayType::Modal)
    payload: "OverlayPayload::DropConfirm { item_id: ItemId }"
    trigger: OnLeftClick
    conditions:
      - drop_button.enabled == true

selection_model:
  type: none
```

---

## 8. State Mapping (Per-Region)

> 每个 region 独立的状态。必须定义 Loading / Empty / Normal / Error 四种状态。
> AI 实现时必须为每种状态提供对应的 UI 展示。

### 8.1 inventory_header

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 标题和关闭按钮为静态 LocalizedKey + 主题图标，无异步加载 | — | — |
| **Empty** | N/A — 标题始终有值，关闭按钮始终存在 | — | — |
| **Normal** | 标题文本 HeadingText + 关闭图标 IconButton 正常显示 | OnEnter 完成 | Fade(0.3s) 入场动画 |
| **Error** | N/A — 静态元素无错误状态 | — | — |

### 8.2 inventory_character_list

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | 3 个骨架屏行（SkeletonRow — 灰色矩形容器，无文本） | `PartyVm` 数据未就绪 | Spinner 旋转动画 |
| **Empty** | `inventory_character_empty`（EmptyWidget: "No characters available"）+ 空列表图标 | `PartyVm.characters.len() == 0` | 淡入 0.2s |
| **Normal** | `inventory_character_list_label` + `inventory_character_rows`（CharacterRow × N），其中一个高亮为选中态 | `PartyVm.characters.len() > 0` | 行逐个淡入 (stagger 50ms) |
| **Error** | N/A — 角色列表数据来自 Party Domain，始终有有效值 | — | — |

### 8.3 inventory_area (ItemGrid + FilterBar)

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | `inventory_loading`（Spinner 居中 + "Loading..." CaptionText） | `InventoryVm` 数据未就绪（首次加载或角色切换后数据加载中） | Spinner 旋转动画 |
| **Empty** | `inventory_empty_state`（EmptyInventoryWidget: 图标 + 标题 "No Items" + 描述文本） | `InventoryVm.items.len() == 0` 或筛选后无匹配物品 | 淡入 0.2s |
| **Normal** | FilterBar（5 个 TabButton）+ ItemGrid（ItemCell × N），选中 Tab 高亮 | `InventoryVm.items.len() > 0` | ItemCell 逐个淡入 (stagger 30ms) |
| **Error** | `inventory_error_state`（`inventory_error_text` + `inventory_retry_btn`） | `InventoryVm` 数据加载失败 | 错误提示淡入 0.2s |

**注意**:
- FilterBar 在所有状态下均**不可见**（除 Normal 外过滤无意义），在 Empty 状态下 FilterBar 隐藏
- FilterBar 在 Loading/Empty/Error 状态下不渲染
- Empty 状态包含两种子场景：角色无任何物品 / 所选筛选条件下无匹配物品

### 8.4 inventory_description_panel

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 物品详情数据随 ItemGrid 数据一起加载，无需独立 Loading | — | — |
| **Empty** | 占位文本 "Select an item to view details"（CaptionText），图标占位灰显 | `InventoryScreenState.selected_item == None` | 淡入 0.2s |
| **Normal** | `inventory_item_detail`: icon + name + desc + stats 完整显示 | `InventoryScreenState.selected_item != None` | 详情滑入 0.15s（SlideUp） |
| **Error** | N/A — 物品详情不独立加载，错误状态由 inventory_area 处理 | — | — |

### 8.5 inventory_action_bar

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 按钮文本为静态 LocalizedKey | — | — |
| **Empty** | UseButton 和 DropButton 均 disabled（灰色不可点击） | `InventoryScreenState.selected_item == None` | 无过渡 |
| **Normal** | UseButton 和 DropButton 均为 enabled 状态，可点击 | `InventoryScreenState.selected_item != None` | 按钮启用过渡 0.1s |
| **Error** | N/A — 按钮无错误状态 | — | — |

---

## 9. Focus Navigation

> Tab 导航路径。按 Tab 键的顺序就是导航路径的顺序。
> 注意：焦点路径中 CharacterRow 和 ItemCell 是动态集合，焦点在集合中按 Arrow 键移动。

```yaml
focus_path:
  ## 阶段 1: 角色列表
  - inventory_character_rows      # Tab 组 1 — 角色行集合
    ## 子组内: ArrowUp/ArrowDown 在 CharacterRow 间移动
    # - inventory_character_row_0  # 默认第一行
    # - inventory_character_row_1
    # - inventory_character_row_N

  ## 阶段 2: 筛选 Tab
  - inventory_filter_all          # Tab 2 — 默认选中
  - inventory_filter_consumable   # Tab 3
  - inventory_filter_equipment    # Tab 4
  - inventory_filter_key_item     # Tab 5
  - inventory_filter_material     # Tab 6

  ## 阶段 3: 物品网格
  - inventory_item_grid           # Tab 组 3 — ItemCell 集合
    ## 子组内: Arrow keys 网格导航 (左/右/上/下)
    # - inventory_item_cell_00    # 第一行第一列
    # - inventory_item_cell_01    # 第一行第二列
    # - inventory_item_cell_10    # 第二行第一列
    # ...

  ## 阶段 4: 关闭按钮
  - inventory_header_close        # Tab 7

  ## 阶段 5: 操作按钮
  - inventory_use_btn             # Tab 8
  - inventory_drop_btn            # Tab 9

special_keys:
  Escape: "触发 inventory_header_close.clicked，返回前一 Screen"
  Enter: "激活当前焦点按钮 / 确认当前选中"
  ArrowUp: "CharacterRow 集合内上移 / ItemGrid 集合内上移一行"
  ArrowDown: "CharacterRow 集合内下移 / ItemGrid 集合内下移一行"
  ArrowLeft: "ItemGrid 集合内左移一列 / Tab 按钮间左移"
  ArrowRight: "ItemGrid 集合内右移一列 / Tab 按钮间右移"
  Tab: "按 focus_path 顺序前进"
  Shift+Tab: "按 focus_path 逆序后退"

focus_trap: true          # true = 焦点锁定在该 Screen 内，Tab 循环
```

### 9.1 默认焦点

进入 InventoryScreen 时，默认焦点落在 `inventory_character_row_0`（角色列表第一行）。
当从 Overlay（ModalOverlay）返回时，焦点回到 `inventory_drop_btn`。

---

## 10. Interaction Zones

> 每个可交互区域的行为定义。

### 10.1 inventory_header_close

```yaml
zone_id: inventory_header_close
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::CloseScreen"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "IconButton 变为 Hover 样式（高亮变体）"
    leave_effect: "恢复 IconButton 默认样式"
    delay: 0ms
```

### 10.2 inventory_character_row

```yaml
zone_id: inventory_character_row
interactions:
  - type: click
    button: Left
    effect: "选中该角色行，更新 InventoryScreenState.selected_character，触发 InventoryVm 刷新为该角色的物品列表，选中行高亮"
    cursor: Pointer
    conditions:
      - row != already_selected: "仅当该角色未选中时触发切换"

  - type: hover
    enter_effect: "CharacterRow 变为 Hover 样式（背景色轻微高亮）"
    leave_effect: "恢复 CharacterRow 默认样式（选中态除外）"
    delay: 0ms
```

### 10.3 inventory_filter_all

```yaml
zone_id: inventory_filter_all
interactions:
  - type: click
    button: Left
    effect: "激活 All Tab，设置 filter=All，筛选后的物品重新排列在 ItemGrid 中"
    cursor: Pointer
    conditions:
      - tab != already_active: "仅当该 Tab 未激活时切换"

  - type: hover
    enter_effect: "TabButton 变为 Hover 样式"
    leave_effect: "恢复 TabButton 默认样式（激活态除外）"
    delay: 0ms
```

### 10.4 inventory_filter_consumable

```yaml
zone_id: inventory_filter_consumable
interactions:
  - type: click
    button: Left
    effect: "激活 Consumable Tab，设置 filter=Consumable，仅显示消耗品"
    cursor: Pointer
    conditions:
      - tab != already_active

  - type: hover
    enter_effect: "TabButton 变为 Hover 样式"
    leave_effect: "恢复 TabButton 默认样式"
    delay: 0ms
```

### 10.5 inventory_filter_equipment

```yaml
zone_id: inventory_filter_equipment
interactions:
  - type: click
    button: Left
    effect: "激活 Equipment Tab，设置 filter=Equipment，仅显示装备"
    cursor: Pointer
    conditions:
      - tab != already_active

  - type: hover
    enter_effect: "TabButton 变为 Hover 样式"
    leave_effect: "恢复 TabButton 默认样式"
    delay: 0ms
```

### 10.6 inventory_filter_key_item

```yaml
zone_id: inventory_filter_key_item
interactions:
  - type: click
    button: Left
    effect: "激活 Key Item Tab，设置 filter=KeyItem，仅显示关键物品"
    cursor: Pointer
    conditions:
      - tab != already_active

  - type: hover
    enter_effect: "TabButton 变为 Hover 样式"
    leave_effect: "恢复 TabButton 默认样式"
    delay: 0ms
```

### 10.7 inventory_filter_material

```yaml
zone_id: inventory_filter_material
interactions:
  - type: click
    button: Left
    effect: "激活 Material Tab，设置 filter=Material，仅显示材料"
    cursor: Pointer
    conditions:
      - tab != already_active

  - type: hover
    enter_effect: "TabButton 变为 Hover 样式"
    leave_effect: "恢复 TabButton 默认样式"
    delay: 0ms
```

### 10.8 inventory_item_cell

```yaml
zone_id: inventory_item_cell
interactions:
  - type: click
    button: Left
    effect: "选中该物品，更新 InventoryScreenState.selected_item，刷新 DescriptionPanel + ActionBar 状态"
    cursor: Pointer
    conditions:
      - cell != already_selected: "仅当该物品未选中时触发"

  - type: hover
    enter_effect: "ItemCell 边框高亮，延迟 500ms 后弹出 TooltipOverlay（物品名称 + 基础说明）"
    leave_effect: "取消 TooltipOverlay，恢复边框样式"
    delay: 500ms
```

### 10.9 inventory_use_btn

```yaml
zone_id: inventory_use_btn
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::UseItem { item_id, character_id }，使用后刷新 InventoryVm"
    cursor: Pointer
    conditions:
      - selected_item != None

  - type: hover
    enter_effect: "Primary Button 高亮变体"
    leave_effect: "恢复 Primary 默认样式"
    delay: 0ms
```

### 10.10 inventory_drop_btn

```yaml
zone_id: inventory_drop_btn
interactions:
  - type: click
    button: Left
    effect: "弹出 ModalOverlay（丢弃确认弹窗：'确定丢弃 {item_name}？' + 确认/取消按钮）"
    cursor: Pointer
    conditions:
      - selected_item != None

  - type: hover
    enter_effect: "Danger Button 高亮变体（背景变亮/边框高亮）"
    leave_effect: "恢复 Danger 默认样式"
    delay: 0ms
```

### 10.11 inventory_retry_btn

```yaml
zone_id: inventory_retry_btn
interactions:
  - type: click
    button: Left
    effect: "重新触发 InventoryVm 数据加载，切换到 Loading 状态"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "Secondary Button 高亮变体"
    leave_effect: "恢复 Secondary 默认样式"
    delay: 0ms
```

---

## 11. Overlay Definition

> Overlay 列表 + Z-Layer。Z-Layer 分配遵循 `07-specs/references/z-layer-spec.md`。

| Overlay | 用途 | Z-Layer | 类型 | 触发条件 |
|---------|------|---------|------|---------|
| TooltipOverlay | 物品悬浮提示：显示物品名称 + 基础说明 | tooltip_layer (200) | Tooltip | Hover `inventory_item_cell` 延迟 500ms |
| ModalOverlay | 丢弃物品二次确认："确定丢弃 {item_name}？" + 确认/取消按钮 | popup_layer (300) | Modal | 点击 `inventory_drop_btn`（有选中物品时） |

### 11.1 Z-Layer 分配

| Z-Layer | 用途 | 包含 |
|---------|------|------|
| 0 | Screen 主界面层 | `inventory_screen_root` 及所有子 region |
| 1 | Tooltip 层 | TooltipOverlay（物品悬浮提示） |
| 2 | Notification 层 | 预留 |
| 3 | Modal 层 | ModalOverlay（丢弃确认） |
| 4 | Popup 层 | 预留 |
| 9 | Debug 层 | DebugOverlay (FPS/日志) |

### 11.2 Overlay 生命周期

| Overlay | OnOpen | OnClose | 依赖 |
|---------|--------|---------|------|
| TooltipOverlay | 创建 Tooltip 实体（屏幕坐标跟随鼠标位置），内容为物品名称(HeadingText) + 物品基础说明(CaptionText) | 销毁 Tooltip 实体 | 无 |
| ModalOverlay (DropConfirm) | 创建 Modal 实体：半透明遮罩 (z=299) + 弹窗面板 (z=300) + "确认丢弃"按钮 (Danger) + "取消"按钮 (Secondary)，焦点锁定到弹窗内 | 确认：触发 `UiCommand::DropItem { item_id }`，销毁 Modal 实体，焦点回到 `inventory_drop_btn`；取消：销毁 Modal 实体，焦点回到 `inventory_drop_btn` | 无 |

### 11.3 ModalOverlay 内部结构

```
ModalOverlay (DropConfirm)
├── ModalBackdrop    [modal_backdrop: Panel — 半透明遮罩, z=299]
└── ModalDialog      [modal_dialog: Container — 居中弹窗, z=300]
    ├── ModalTitle   [modal_title: HeadingText]              — "ui.inventory.drop_confirm.title"
    ├── ModalBody    [modal_body: BodyText]                  — "ui.inventory.drop_confirm.body" (含 item_name 参数)
    ├── ConfirmBtn   [modal_confirm_btn: Button, Danger]     — "ui.inventory.drop_confirm.yes"
    └── CancelBtn    [modal_cancel_btn: Button, Secondary]   — "ui.inventory.drop_confirm.no"
```

### 11.4 TooltipOverlay 内部结构

```
TooltipOverlay
└── TooltipPanel    [tooltip_panel: Container — 浮动面板, z=200]
    ├── TooltipName [tooltip_name: HeadingText]              — "InventoryVm.selected_item.name"
    └── TooltipDesc [tooltip_desc: CaptionText]              — "InventoryVm.selected_item.short_desc"
```

---

## 12. Lifecycle

> Screen 的完整生命周期行为。遵守 `screen-lifecycle.md` 定义的状态机。

| 阶段 | 行为 | 触发条件 | 清理 |
|------|------|---------|------|
| **OnEnter** | `spawn_inventory_screen()` — 生成完整 UI 树（inventory_screen_root → Header + Body + DescriptionPanel + ActionBar），从 PartyVm/InventoryVm 读取数据初始化各 Widget 状态 | `GameState::Inventory` 状态进入 | — |
| **OnReady** | 注册 InventoryProjection Observer（监听物品变更事件） | OnEnter 完成，UI 树就绪 | — |
| **Active** | 等待角色选择、物品筛选、物品选中、使用/丢弃等交互事件，通过 UiCommand 下发至 Domain | OnReady 完成 | — |
| **OnExit** | `despawn_inventory_screen()` — 清理所有标记了 `With<InventoryScreen>` 的实体；注销 InventoryProjection Observer | `GameState::Inventory` 状态退出 | 清理标记: `With<InventoryScreen>` |

### 12.1 生命周期事件处理

```yaml
on_enter:
  - action: "spawn_ui_tree"
    spawner: "spawn_inventory_screen()"
    description: "生成 InventoryScreen 完整 UI 树：Header(标题+关闭按钮) + Body(CharacterList+InventoryArea) + DescriptionPanel + ActionBar"
  - action: "read_party_data"
    source: "PartyVm"
    description: "从 PartyVm 读取角色列表，填充 CharacterRow 列表"
  - action: "read_inventory_data"
    source: "InventoryVm"
    description: "从 InventoryVm 读取当前选中角色的物品列表，填充 ItemGrid"
  - action: "set_default_selection"
    target: "inventory_character_row_0"
    description: "默认选中角色列表第一行（如有角色），触发其物品列表加载"

on_ready:
  - action: "register_observer"
    target: "InventoryProjection::InventoryChanged"
    handler: "当物品发生变更（使用/丢弃/获得）时，刷新 InventoryVm 并标记 dirty，触发 ItemGrid 重渲染"
  - action: "register_observer"
    target: "PartyProjection::PartyChanged"
    handler: "当队伍成员变更（加入/离队）时，刷新 CharacterRow 列表"

active:
  - trigger: "inventory_character_row.selected"
    action: "更新 InventoryScreenState.selected_character，加载该角色的物品数据，ItemGrid 进入 Loading → Normal 状态转换"
    scope: "inventory_character_list"
  - trigger: "inventory_filter_*.selected"
    action: "更新 InventoryScreenState.filter，按新筛选条件重新过滤物品列表，ItemGrid 重新排列"
    scope: "inventory_area"
  - trigger: "inventory_item_cell.selected"
    action: "更新 InventoryScreenState.selected_item，刷新 DescriptionPanel 和 ActionBar 状态"
    scope: "inventory_description_panel, inventory_action_bar"
  - trigger: "inventory_item_cell.hovered"
    action: "弹出 TooltipOverlay（延迟 500ms），显示物品名称 + 简短说明"
    scope: "tooltip_layer"
  - trigger: "inventory_use_btn.clicked"
    action: "构建 UiCommand::UseItem { item_id, character_id } 下发至 Domain，物品使用后通过 Observer 刷新 InventoryVm"
    scope: "inventory_action_bar → Domain"
  - trigger: "inventory_drop_btn.clicked"
    action: "弹出 ModalOverlay（丢弃确认），阻止下层交互"
    scope: "inventory_action_bar → ModalOverlay"
  - trigger: "ModalOverlay.confirm (DropConfirm)"
    action: "构建 UiCommand::DropItem { item_id } 下发至 Domain，销毁 Modal，焦点回到 inventory_drop_btn，物品删除后通过 Observer 刷新"
    scope: "ModalOverlay → Domain"
  - trigger: "ModalOverlay.cancel (DropConfirm)"
    action: "销毁 ModalOverlay，焦点回到 inventory_drop_btn"
    scope: "ModalOverlay"
  - trigger: "inventory_retry_btn.clicked"
    action: "重新加载 InventoryVm 数据，ItemGrid 进入 Loading 状态"
    scope: "inventory_area"

on_exit:
  - action: "unregister_observer"
    target: "InventoryProjection::InventoryChanged"
  - action: "unregister_observer"
    target: "PartyProjection::PartyChanged"
  - action: "despawn_ui_tree"
    query: "With<InventoryScreen>"
    description: "清理所有标记了 InventoryScreen 组件的实体"
```

---

## 13. Data Ownership

> Owns / Uses 分离。InventoryScreen 拥有选中状态和筛选状态（本地 UI 状态），消费 InventoryVm 和 PartyVm（来自 Domain Projection）。

### 13.1 ViewModel 映射

| ViewModel | 字段 | 归属 (Owns/Uses) | 更新频率 | Projection 源 |
|-----------|------|-----------------|---------|--------------|
| `InventoryScreenState::selected_character` | `EntityId` | Owns — Screen 独享的本地 UI 状态 | 角色切换事件 | 无（仅 UI 内部使用） |
| `InventoryScreenState::selected_item` | `Option<ItemId>` | Owns — Screen 独享的本地 UI 状态 | 物品点击事件 | 无（仅 UI 内部使用） |
| `InventoryScreenState::filter` | `FilterType` | Owns — Screen 独享的本地 UI 状态 | Tab 点击事件 | 无（仅 UI 内部使用） |
| `PartyVm::characters` | `Vec<CharacterInfo>` | Uses — 多个 Screen 共享 | 队伍变更事件 | `PartyProjection::on_party_changed` |
| `InventoryVm::items` | `Vec<ItemInfo>` | Uses — InventoryScreen 独享消费 | 物品变更事件 | `InventoryProjection::on_inventory_changed` |
| `InventoryVm::selected_item` | `Option<ItemDetail>` | Uses — InventoryScreen 独享消费 | 物品选中事件 | 派生自 `InventoryVm.items` |

### 13.2 数据流

```
Domain Event (物品变更/队伍变更)
    ↓
Projection (InventoryProjection / PartyProjection)
    ↓
ViewModel (InventoryVm / PartyVm) 更新 → mark_dirty()
    ↓
InventoryScreen 消费 ViewModel → Widget 数据绑定刷新
    ↓
用户交互 → Widget 输出事件 → UiCommand
    ↓
UiLayer 处理 → Domain (UseItem / DropItem / SelectCharacter)
    ↓
Domain Event → 回到顶部 (循环)
```

### 13.3 本地 UI 状态

| 字段 | 类型 | 初始值 | 修改时机 | 影响区域 |
|------|------|--------|---------|---------|
| `selected_character` | `Option<EntityId>` | `PartyVm.characters[0]` (如有) | 点击 CharacterRow | inventory_character_rows, inventory_area, inventory_action_bar |
| `selected_item` | `Option<ItemId>` | `None` | 点击 ItemCell | inventory_item_cell (选中态), inventory_description_panel, inventory_action_bar |
| `filter` | `FilterType` | `FilterType::All` | 点击 FilterBar TabButton | inventory_filter_bar (激活态), inventory_item_grid (过滤后) |

---

## 14. Layout Intent

> 每个关键尺寸的**理由说明**。为什么选这个尺寸而不是别的？

### 14.1 固定尺寸意图

| widget_id | 属性 | 值 | 意图 | shrink |
|-----------|------|----|------|--------|
| `inventory_header` | height | 56px | "Header 栏高度 56px，略大于按钮高度 40px，提供充足上下内边距（8px），与 SettingsScreen Header 保持一致" | none |
| `inventory_header_close` | width | 40px | "关闭按钮最小触摸目标 40x40" | none |
| `inventory_header_close` | height | 40px | "最小触摸目标 40px" | none |
| `inventory_character_list` | width | 320px | "角色列表固定 320px，与 BattleScreen char_panel 宽度一致，容纳头像(40x40) + 名称(最长 10 字) + 等级(4 字符) 在一行内" | none |
| `inventory_character_row` | height | 56px | "单行高度 56px，大于 40px 最小触摸目标，容纳头像(40px) + 上下内边距(8px)" | none |
| `inventory_description_panel` | height | 180px | "底部详情面板固定 180px，容纳图标(72x72) + 名称 + 描述(最多 3 行) + 属性(1 行)，与底部 ActionBar 区分" | none |
| `inventory_item_detail_icon` | width | 72px | "物品图标 72x72，大于 ItemCell 中的图标(48px)，细节更清晰" | none |
| `inventory_item_detail_icon` | height | 72px | "与宽度一致保持正方形" | none |
| `inventory_item_cell` | width | 72px | "物品格子 72x80，图标 48x48 + 数量文本 12px 一行，Grid 布局紧凑排列" | none |
| `inventory_item_cell` | height | 80px | "80px 容纳图标 + 文本 + 内边距" | none |
| `inventory_use_btn` | width | 140px | "使用按钮 140px 确保标签 'Use Item'/'使用物品' 中英文单行显示" | none |
| `inventory_use_btn` | height | 40px | "最小触摸目标 40px" | none |
| `inventory_drop_btn` | width | 140px | "与 UseButton 同宽，保持视觉对称" | none |
| `inventory_drop_btn` | height | 40px | "最小触摸目标 40px" | none |
| `inventory_retry_btn` | width | 120px | "重试按钮 120px 足够容纳 'Retry'/'重试'" | none |
| `inventory_retry_btn` | height | 40px | "最小触摸目标 40px" | none |
| `inventory_action_bar` | height | 56px | "ActionBar 高度 56px，与 Header 等高，视觉对称" | none |
| `inventory_filter_bar` | height | 44px | "FilterBar 高度 44px，TabButton 高度 > 最小触摸目标 40px" | none |

### 14.2 弹性尺寸意图

| widget_id | flex_grow | 理由 |
|-----------|-----------|------|
| `inventory_header_title` | 1 | "标题弹性增长，将关闭按钮推至右侧（Space-between 等效行为）" |
| `inventory_body` | 1 | "Body 占据 Header/DescPanel/ActionBar 之外的所有垂直空间，内容区越大越好" |
| `inventory_character_rows` | 1 | "角色行列表弹性填充 CharacterList 中标签下方的空间，容纳尽可能多的角色" |
| `inventory_area` | 1 | "Inventory 主区域弹性填充 CharacterList 右侧的剩余空间" |
| `inventory_item_grid` | 1 | "物品网格弹性填充 FilterBar 下方的剩余空间，显示尽可能多的物品" |

### 14.3 通用约束

```yaml
global:
  min_interactive_height: 40px   # 可交互元素最小高度 (触摸友好)
  min_interactive_width: 40px    # 可交互元素最小宽度 (触摸友好)
  standard_padding: 8px          # 标准内边距
  standard_gap: 8px              # 标准间距 (Flexbox gap)
  character_list_gap: 4px        # 角色行间距略小
  item_grid_cell_gap: 8px        # 物品网格格间距
```

---

## 15. Scroll & Overflow Policy

> InventoryScreen 的 ItemGrid 区域可能因物品过多需要垂直滚动；CharacterList 也可能因角色过多需要滚动。

### 15.1 滚动区域

| widget_id | 方向 | Scroll Policy | Overflow Policy | 理由 |
|-----------|------|--------------|---------------|------|
| `inventory_character_rows` | vertical | auto | clip | "角色行列表可能超过可视高度（>5 行），当内容溢出时显示滚动条" |
| `inventory_item_grid` | vertical | auto | clip | "物品网格可能因物品数量过多（>20 个）超出可视高度，每页显示 24 个（4 列 x 6 行），超出时垂直分页滚动" |
| `inventory_header` | horizontal | none | clip | "Header 内容固定，不会溢出" |
| `inventory_filter_bar` | horizontal | none | clip | "5 个 TabButton auto 宽度总和在 1280px 宽度下足够，不会溢出" |
| `inventory_description_panel` | vertical | none | clip | "详情内容固定 180px 高度，文本使用 ellipsis 处理溢出" |
| `inventory_action_bar` | horizontal | none | clip | "2 个按钮（140x2=280px）远低于 1280px 宽度，不会溢出" |

### 15.2 文本溢出

| widget_id | max_lines | overflow | 多语言风险 |
|-----------|-----------|----------|-----------|
| `inventory_header_title` | 1 | clip | "标题 'Inventory' / '背包' 极短，无多语言溢出风险" |
| `inventory_character_list_label` | 1 | ellipsis | "'Characters' / '角色' 极短，风险低" |
| `inventory_character_row` | 1 | ellipsis | "名称 + 等级格式：'Aria Lv.5' / '阿瑞雅 Lv.5'，320px 宽度足够，风险低" |
| `inventory_filter_all` | 1 | ellipsis | "'All' / '全部' 极短，风险低" |
| `inventory_filter_consumable` | 1 | ellipsis | "'Consumable' 英文 10 字符，auto 宽度，风险低" |
| `inventory_filter_equipment` | 1 | ellipsis | "'Equipment' 英文 9 字符，auto 宽度，风险低" |
| `inventory_filter_key_item` | 1 | ellipsis | "'Key Item' 英文 8 字符，auto 宽度，风险低" |
| `inventory_filter_material` | 1 | ellipsis | "'Material' 英文 8 字符，auto 宽度，风险低" |
| `inventory_item_detail_name` | 2 | ellipsis | "物品名称通常不超过 20 字符，2 行足够，风险低" |
| `inventory_item_detail_desc` | 3 | ellipsis | "物品描述中文约 30 字 / 英文约 80 字符，3 行可能溢出，使用 ellipsis 截断" |
| `inventory_item_detail_stats` | 1 | ellipsis | "属性文本通常有固定格式（'Type: Consumable | Weight: 0.5'），风险中，使用 ellipsis" |
| `inventory_use_btn` | 1 | ellipsis | "'Use Item' / '使用物品' 极短，风险低" |
| `inventory_drop_btn` | 1 | ellipsis | "'Drop' / '丢弃' 极短，风险低" |
| `inventory_error_text` | 3 | ellipsis | "错误消息中英文长度不定，3 行截断处理" |

---

## 16. Event Contract

> UI -> Domain 事件 + Domain -> UI 事件的完整契约。

### 16.1 UI -> Domain（通过 UiCommand 传递）

```yaml
SelectCharacter:
  trigger_widget: "inventory_character_list → inventory_character_rows → inventory_character_row → click"
  data:
    character_id: EntityId
  conditions:
    - character_id != selected_character  # 仅当切换角色时触发
  emits: UiCommand::SelectCharacter(EntityId)
  domain_event: "InventoryCharacterSelected { character_id }"

SelectItem:
  trigger_widget: "inventory_area → inventory_item_grid → inventory_item_cell → click"
  data:
    item_id: ItemId
  conditions:
    - item_id != selected_item  # 仅当切换物品时触发
  emits: UiCommand::SelectItem(ItemId)
  domain_event: "InventoryItemSelected { item_id }"

FilterChanged:
  trigger_widget: "inventory_area → inventory_filter_bar → inventory_filter_* → click"
  data:
    filter_type: FilterType
  conditions:
    - filter_type != current_filter
  emits: UiCommand::FilterChanged(FilterType)
  domain_event: "None — 纯 UI 本地状态变更，不触发 Domain Event"

UseItem:
  trigger_widget: "inventory_action_bar → inventory_use_btn → click"
  data:
    item_id: ItemId
    character_id: EntityId
  conditions:
    - selected_item != None
    - use_btn.enabled == true
  emits: UiCommand::UseItem(ItemId, EntityId)
  domain_event: "InventoryItemUsed { item_id, character_id }"

DropItem:
  trigger_widget: "inventory_action_bar → inventory_drop_btn → click → ModalOverlay.confirm"
  data:
    item_id: ItemId
  conditions:
    - selected_item != None
    - modal_confirm == true  # 丢弃二次确认通过
  emits: UiCommand::DropItem(ItemId)
  domain_event: "InventoryItemDropped { item_id }"

CloseInventory:
  trigger_widget: "inventory_header → inventory_header_close → click"
  data: {}
  conditions: []
  emits: UiCommand::CloseScreen
  domain_event: "None — 纯 UI 导航，ScreenStack 直接处理"
```

### 16.2 Domain -> UI（通过 Projection 消费）

```yaml
InventoryCharacterSelected:
  source: "Domain Event (Inventory Domain)"
  projection: "InventoryProjection.on_character_selected()"
  vm_update:
    - "InventoryVm.items ← new character's items"
    - "InventoryScreenState.selected_character ← character_id"
    - "InventoryScreenState.selected_item ← None"
  side_effect:
    - "mark_dirty::<InventoryVm>()"
    - "ItemGrid 从 Empty/Normal → Loading → Normal 状态转换"

InventoryItemUsed:
  source: "Domain Event (Inventory Domain)"
  projection: "InventoryProjection.on_item_used()"
  vm_update:
    - "InventoryVm.items ← updated (减少数量/移除物品)"
  side_effect:
    - "mark_dirty::<InventoryVm>()"
    - "如果数量降为 0: selected_item → None, DescriptionPanel → Empty 状态"

InventoryItemDropped:
  source: "Domain Event (Inventory Domain)"
  projection: "InventoryProjection.on_item_dropped()"
  vm_update:
    - "InventoryVm.items ← updated (减少数量/移除物品)"
    - "InventoryVm.selected_item ← None"
  side_effect:
    - "mark_dirty::<InventoryVm>()"
    - "selected_item → None, DescriptionPanel → Empty 状态"

PartyChanged:
  source: "Domain Event (Party Domain)"
  projection: "PartyProjection.on_party_changed()"
  vm_update:
    - "PartyVm.characters ← updated list"
    - "如果当前 selected_character 不在新列表中: selected_character ← None"
  side_effect:
    - "mark_dirty::<PartyVm>()"
    - "CharacterList 重新渲染"
```

---

## 17. Screen Metrics

> 复杂度基线。所有数值初始创建时手动填写，后续 CI 阶段自动校验。

| Metric | 值 | P0/P1 | 说明 |
|--------|-----|-------|------|
| `widget_count` | 33 | P1 | Widget 实例总数（root + header_container + title + close btn + body + character_list_container + label + rows_container + character_row + empty_widget + area_container + filter_bar + 5 tab btns + item_grid + item_cell + empty_state + loading + error_container + error_text + retry_btn + desc_panel + detail_container + icon + name + desc + stats + action_bar + use_btn + drop_btn） |
| `container_count` | 12 | P1 | 纯容器节点数（root + header + body + character_list + character_rows + area + filter_bar + item_grid + error_state + description_panel + item_detail + action_bar） |
| `interactive_count` | 11 | P1 | 可交互 Widget 数（close btn + character_row + 5 tab btns + item_cell + retry_btn + use_btn + drop_btn） |
| `overlay_count` | 2 | P1 | 关联的 Overlay 数（TooltipOverlay 物品悬浮提示 + ModalOverlay 丢弃确认） |
| `max_depth` | 5 | P1 | root → inventory_body → inventory_area → inventory_filter_bar → inventory_filter_all（5 层） |
| `max_children` | 5 | P1 | inventory_area 有 5 个直接子节点（filter_bar, item_grid, empty_state, loading, error_state） |

### 17.1 Budget 检查

| 规则 | 阈值 | 当前值 | 状态 |
|------|------|--------|------|
| max_depth ≤ 6 | 6 | 5 | ✅ |
| max_children ≤ 20 | 20 | 5 | ✅ |
| interactive_count / widget_count ≥ 0.2 | 20% | 33% (11/33) | ✅ |

---

## 附录 A: DoD Checklist

> 以下 18 项全部通过后，本文件 status 改为 `active`。

| # | 检查项 | 状态 | 备注 |
|---|--------|------|------|
| D01 | ASCII Wireframe 存在, 所有区域已命名 (region_id) | [x] | |
| D02 | 无匿名面板 — 每个区域都有 widget_id 标注 | [x] | |
| D03 | Widget Tree 完整 — 从 root 到叶子，无隐藏节点 | [x] | |
| D04 | 所有引用的 widget_id 在 Widget Tree 中存在 | [x] | |
| D05 | Flexbox Layout 完整 — 每个 widget_id 有 direction/width/height/flex_grow/intent | [x] | |
| D06 | Responsive Rules 已定义 (至少 strategy: none) | [x] | |
| D07 | Region Responsibility 已定义 (每 region 3-8 条) | [x] | |
| D08 | Widget Contract 已定义 (Inputs/Outputs/Selection Model) | [x] | |
| D09 | State Mapping 完整 (每个 region 的 Loading/Empty/Normal/Error) | [x] | ItemGrid 定义完整 4 状态；Header/DescPanel/ActionBar 部分 N/A |
| D10 | Focus Navigation 已定义 (Tab 路径完整) | [x] | P1，含 CharacterRow 和 ItemGrid 集合导航 |
| D11 | Interaction Zones 已定义 (Click/Hover) | [x] | |
| D12 | Overlay Definition 已定义 (Overlay 列表 + Z-Layer) | [x] | TooltipOverlay + ModalOverlay |
| D13 | Lifecycle 已定义 (OnEnter/OnReady/Active/OnExit) | [x] | |
| D14 | Data Ownership 已定义 (Owns/Uses) | [x] | |
| D15 | Layout Intent 已定义 (关键尺寸理由) | [x] | P1 |
| D16 | Scroll & Overflow Policy 已定义 | [x] | P1 |
| D17 | Event Contract 已定义 (UI->Domain + Domain->UI) | [x] | P1 |
| D18 | Screen Metrics 已定义 | [x] | P1 |

**P0 字段全部通过日期**: 2026-06-22
**status 改为 active 日期**: 2026-06-22

---

## 附录 B: 引用文档

| 文档 | 用途 |
|------|------|
| `07-specs/README.md` | SSPEC 总纲、AI 14 条规则、DoD 18 项清单 |
| `07-specs/references/widget-id-map.md` | Widget ID -> UiBinding 映射总表 |
| `07-specs/references/z-layer-spec.md` | Z-Layer 统一规范 |
| `07-specs/references/layout-intent-library.md` | 跨 Screen 共享的 Layout Intent |
| `06-ui/02-design-system/widget-atoms.md` | 原子组件 Contract（HeadingText, IconButton, CaptionText, TabButton, Button, BodyText, Image, Spinner, EmptyWidget） |
| `06-ui/02-design-system/widget-composites.md` | 复合组件 Contract（CharacterRow, ItemCell, EmptyInventoryWidget, FilterBar） |
| `06-ui/02-design-system/theme-localization.md` | StyleToken / Theme / UiTextKey |
| `06-ui/02-design-system/focus-binding.md` | Focusable / FocusGroup / Dirty<T> / UiBinding |
| `06-ui/03-screens/screens.md` | InventoryScreen 定义（§4） |
| `06-ui/03-screens/screen-lifecycle.md` | Screen 生命周期状态机 |
| `06-ui/04-data-flow/projection-viewmodel.md` | Projection / ViewModel 映射 |
| `03-content/localization/ui-screen-keys.md` | 背包界面 LocalizationKeys |

---

## 附录 C: 架构审查记录

> **审查人**: @presentation-architect | **日期**: 2026-06-22 | **结论**: 不通过 — 需要修复 (见下文)

### 审查结果

| 维度 | 结果 |
|------|------|
| 模板合规 (17字段) | 通过 |
| 架构一致性 (vs screens.md §4) | 通过 — SSPEC 是目标设计，screens.md §4 描述当前 MVP；二者不同但无矛盾 |
| ASCII Wireframe (无匿名面板) | 通过 |
| Widget ID 命名规范 (`inventory_{region}_{element}`) | **发现问题** — `inventory_screen_root` 应为 `inventory_screen_root` 以匹配 `{screen}_screen_root` 规范 |
| Flexbox Layout 完整性 | 通过 |
| LocalizationKey 一致性 | 通过 — 所有 Key 使用 `ui.inventory.*` 命名空间 |
| Widget Contract 完整性 | 通过 — §7 所有 Inputs/Outputs/Selection Models 均已定义 |
| Per-Region State Mapping | 通过 — 首个实现该模式的 SSPEC，正确 |
| 单向数据流 | 通过 — Domain → Projection → ViewModel → UI → UiCommand → Domain 完整 |
| widget-id-map.md 引用 | **不通过** — 见下方问题 #1 |

### C.1 必须修复才允许激活

| # | 问题 | 严重级别 | 修复要求 |
|---|------|---------|---------|
| 1 | **widget-id-map.md §4.3 严重过时** | **阻塞** | 新增 SSPEC 的 33+ widget ID 到 widget-id-map.md，旧 18 个 ID 标记为 `deprecated` |
| 2 | **`inventory_screen_root` 命名不规范** | 高 | 改为 `inventory_screen_root` 以符合 `{screen}_screen_root` 规范，并补加到 §2.1 区域索引和 §3.1 Widget Type 索引 |
| 3 | **§3.1 Widget Type 索引缺少 `inventory_screen_root`** | 中 | Widget Tree 和 Flexbox Layout 中有 `inventory_screen_root`，但 §3.1 索引表无对应条目 |
| 4 | **TBD 引用过多** | 中 | `EmptyInventoryWidget`、`CharacterRow`、`FilterBar`、`ItemCell`、`EmptyWidget`、`Spinner` 均引用 `§TBD`。不阻塞 SSPEC status，但实现前必须在 widget-composites.md / widget-atoms.md 中定义合同 |

### C.2 二次审查（2026-06-22）

> **审查人**: @presentation-architect | **日期**: 2026-06-22 | **审查内容**: 校验 C.1 中问题 #1 (widget-id-map) 和 #2 (inventory_root 命名) 的修复

#### 修复验证结果

| 原问题 | 修复状态 | 说明 |
|--------|---------|------|
| #1: widget-id-map.md §4.3 过时 | **部分通过 — 遗留新问题** | 数量从 18 增至 33，但 ID 命名与 SSPEC §2.1 区域索引存在约 50% 不匹配（见下文） |
| #2: `inventory_root` → `inventory_screen_root` | **通过** | 全部 3 个文件（SSPEC、widget-id-map、settings_screen 引用）已正确使用 `inventory_screen_root` |
| #3: §3.1 缺少 `inventory_screen_root` | **原问题已随命名修复自然解决** | |
| SS13: Data Ownership 使用旧 widget_id | **通过** | SS13 全部使用 `InventoryScreenState.{field}` + `EntityId`/`ItemId`/`FilterType`，无 widget_id 引用 |

#### 新发现问题（修复引入）

widget-id-map.md §4.3 被替换为 33 个新 ID，但这些 ID **与 SSPEC §2.1 区域索引不一致**：

| 问题类别 | 数量 | 示例 |
|---------|------|------|
| 映射中命名与 SSPEC 不同的 ID | 11 | `inventory_title_text` (SSPEC: `inventory_header_title`), `inventory_close_btn` (SSPEC: `inventory_header_close`), `inventory_filter_weapon` (SSPEC: `inventory_filter_equipment`), `inventory_filter_armor` (SSPEC 无), `inventory_filter_key` (SSPEC: `inventory_filter_key_item`), `inventory_item_cell_0..5` (SSPEC: `inventory_item_cell`), `inventory_loading_spinner` (SSPEC: `inventory_loading`), `inventory_error_panel` (SSPEC: `inventory_error_state`), `inventory_item_name` (SSPEC: `inventory_item_detail_name`), `inventory_item_desc` (SSPEC: `inventory_item_detail_desc`) |
| 映射中存在但 SSPEC 无此 ID | 2 | `inventory_character_portrait`, `inventory_character_name` |
| SSPEC 中存在但映射遗漏 | 17 | `inventory_header_title`, `inventory_header_close`, `inventory_character_list_label`, `inventory_character_row`, `inventory_filter_equipment`, `inventory_filter_key_item`, `inventory_filter_material`, `inventory_item_cell`, `inventory_loading`, `inventory_error_state`, `inventory_error_text`, `inventory_retry_btn`, `inventory_item_detail`, `inventory_item_detail_icon`, `inventory_item_detail_name`, `inventory_item_detail_desc`, `inventory_item_detail_stats` |

**后果**: 实现者将面临 ID 冲突——SSPEC 定义了一套 ID，widget-id-map 定义了另一套。无法确定哪个是权威。

#### 当前状态

保留 `status: draft`。需要修复 widget-id-map.md §4.3 的 ID 命名以精确对齐 SSPEC §2.1。修复后在下次审查中改为 `active`。

---

### C.3 三次审查（2026-06-22）— 激活

> **审查人**: @presentation-architect | **日期**: 2026-06-22 | **审查内容**: 校验 Fix 1 (widget-id-map §4.3 对齐) 和 Fix 2 (inventory_root → inventory_screen_root) 的修复完成度

#### 修复验证结果

| 原问题 | 修复状态 | 说明 |
|--------|---------|------|
| #1: widget-id-map.md §4.3 过时 | **通过** | @feature-developer 从 SSPEC §2.1 提取了 33 个真实 widget_id，替换了 §4.3。经逐行比对，33/33 与 SSPEC 100% 一对一匹配。旧 ID 均标记为 deprecated |
| #2: `inventory_root` → `inventory_screen_root` | **通过** | SSPEC 和 widget-id-map §4.3 均正确使用 `inventory_screen_root` |
| #3: §3.1 缺少 `inventory_screen_root` | **通过** | 随命名修复自然解决。`inventory_screen_root` 在 Widget Tree (§3) 和 Flexbox Layout (§4) 中均已存在 |
| #4: TBD 引用过多 | **不阻塞** | `EmptyInventoryWidget`、`CharacterRow`、`FilterBar`、`ItemCell`、`EmptyWidget`、`Spinner` 仍引用 `§TBD`。确认不阻塞 status activation，但实现前须在 widget-composites.md / widget-atoms.md 中定义合同 |

#### 激活结论

所有 P0 字段完整。DoD 18 项全部通过。本 SSPEC 自即日起从 `draft` 转为 `active`。

---

*本文档是 InventoryScreen SSPEC，由 @feature-developer 根据 `07-specs/screen-spec-template.md` 模板创建。所有 17 个字段已填充。当前 status: active（三次审查通过 — 见 §C.3）。*
