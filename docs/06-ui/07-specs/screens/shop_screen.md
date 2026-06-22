---
id: 07-specs.shop-screen
title: ShopScreen Specification — AI-Consumable Layout & Interaction Spec
status: draft
owner: presentation-architect
created: 2026-06-22
tags:
  - ui
  - screen-spec
  - shop
  - draft
---

# ShopScreen

> **职责**: @presentation-architect | **上游**: ADR-066 (Screen Spec), `07-specs/README.md` (总纲), `06-ui/03-screens/screens.md` §5 (ShopScreen)
> **状态**: 初始 draft。P0 字段 1-14 待完成后改为 active。

**P0 字段**: 1-14 (Screen Header / ASCII Wireframe / Widget Tree / Flexbox Layout / Responsive Rules / Region Responsibility / Widget Contract / State Mapping / Focus Nav / Interaction Zones / Overlay / Lifecycle / Data Ownership / Layout Intent)
**P1 字段**: 15-17 (Scroll & Overflow / Event Contract / Screen Metrics)

---

## 1. Screen Header

| 属性 | 值 |
|------|-----|
| Screen Name | `ShopScreen` — 对应 `OverlayState::Shop` |
| Purpose | 商店界面：允许玩家浏览 NPC 商品列表、购买物品，以及出售背包中持有的物品 |
| Navigation | UiCommand::OpenOverlay(OverlayType::Shop, npc_entity) → OverlayStack.Push(ShopScreen)；点击 Close 或 Esc 返回下层 Screen（OverlayStack.Pop） |
| GameState | `GameState::Overlay`（父状态），`OverlayState::Shop`（子状态） |
| PopupLayer 层级 | 1（Popup 层浮窗，覆盖下层 Screen） |
| 加载模式 | Ephemeral（每次打开商店重新 spawn） |
| 过渡动画 | Slide(Direction::Up) |
| 变体 | None |

---

## 2. ASCII Wireframe

> 纯文本线框图。所有区域必须命名（`widget_id`），禁止匿名面板。
> ShopScreen 是 PopupLayer 浮窗，包含背景遮罩 + 居中弹窗。线框图展示弹窗内部。

```
┌───────────────────────────────────────────────────────────────────┐
│  ┌───────────────────────────────────────────────────────────┐    │
│  │  [shop_header]                                            │    │
│  │  ┌──────────────────────────────────────┐  ┌───────────┐ │    │
│  │  │  [shop_header_name]                  │  │ [shop_    │ │    │
│  │  │  Equipment Merchant                  │  │  header   │ │    │
│  │  │                                      │  │  close]   │ │    │
│  │  │                                      │  │    [X]    │ │    │
│  │  └──────────────────────────────────────┘  └───────────┘ │    │
│  │                                    ┌───────────────────┐  │    │
│  │                                    │ [shop_player_gold]│  │    │
│  │                                    │   Gold: 1500G     │  │    │
│  │                                    └───────────────────┘  │    │
│  ├───────────────────────────────────────────────────────────┤    │
│  │  [shop_tab_panel]                                         │    │
│  │  ┌──────────┐  ┌──────────┐                               │    │
│  │  │ [shop_   │  │ [shop_   │                               │    │
│  │  │  tab_buy]│  │  tab_sell]                               │    │
│  │  │   Buy    │  │   Sell   │                               │    │
│  │  └──────────┘  └──────────┘                               │    │
│  ├───────────────────────────────────────────────────────────┤    │
│  │  [shop_content]  (Buy Tab Active)                         │    │
│  │  ┌─────────────────────────────────────────────────────┐  │    │
│  │  │  [shop_item_card_1]                                  │  │    │
│  │  │  ┌─────┐ ┌─────────────────────────┐ ┌───────────┐  │  │    │
│  │  │  │icon │ │ Iron Sword              │ │ [shop_    │  │  │    │
│  │  │  │     │ │ ATK+5  Weight: 3.0      │ │  item_    │  │  │    │
│  │  │  │     │ │                         │ │  price]   │  │  │    │
│  │  │  │     │ │                         │ │  [Buy]    │  │  │    │
│  │  │  │     │ │                         │ │  300G     │  │  │    │
│  │  │  └─────┘ └─────────────────────────┘ └───────────┘  │  │    │
│  │  │  [shop_item_card_2]                                  │  │    │
│  │  │  ┌─────┐ ┌─────────────────────────┐ ┌───────────┐  │  │    │
│  │  │  │icon │ │ Health Potion            │ │ 50G       │  │  │    │
│  │  │  │     │ │ Restore 50 HP            │ │ [Buy][2]  │  │  │    │
│  │  │  └─────┘ └─────────────────────────┘ └───────────┘  │  │    │
│  │  │  [shop_item_card_3]                                  │  │    │
│  │  │  ┌─────┐ ┌─────────────────────────┐ ┌───────────┐  │  │    │
│  │  │  │icon │ │ Mana Potion              │ │ 50G       │  │  │    │
│  │  │  │     │ │ Restore 30 MP            │ │ [Buy][1]  │  │  │    │
│  │  │  └─────┘ └─────────────────────────┘ └───────────┘  │  │    │
│  │  └─────────────────────────────────────────────────────┘  │    │
│  │  (Sell Tab Active 时, shop_item_card 替换为 shop_item_row)   │    │
│  │  ┌─────────────────────────────────────────────────────┐  │    │
│  │  │  [shop_item_row_1]                                  │  │    │
│  │  │  ┌─────┐ ┌──────────────────┐ ┌────────┐ ┌───────┐ │  │    │
│  │  │  │icon │ │ Old Boots         │ │ x1     │ │ [Sell]│ │  │    │
│  │  │  │     │ │ Worn leather boot│ │        │ │ 10G   │ │  │    │
│  │  │  └─────┘ └──────────────────┘ └────────┘ └───────┘ │  │    │
│  │  │  [shop_item_row_2]                                  │  │    │
│  │  │  ┌─────┐ ┌──────────────────┐ ┌────────┐ ┌───────┐ │  │    │
│  │  │  │icon │ │ Wolf Pelt         │ │ x3     │ │ [Sell]│ │  │    │
│  │  │  │     │ │ From dire wolf   │ │        │ │ 25G   │ │  │    │
│  │  │  └─────┘ └──────────────────┘ └────────┘ └───────┘ │  │    │
│  │  └─────────────────────────────────────────────────────┘  │    │
│  ├───────────────────────────────────────────────────────────┤    │
│  │  [shop_cart_panel]                                        │    │
│  │  ┌─────────────────────────────────────────────────────┐  │    │
│  │  │  [shop_cart_summary]                                │  │    │
│  │  │  Selected: 2 items   Total: 350G                   │  │    │
│  │  │                    ┌──────────┐  ┌──────────────┐  │  │    │
│  │  │                    │ [shop_   │  │ [shop_       │  │  │    │
│  │  │                    │  buy_btn]│  │  cancel_btn] │  │  │    │
│  │  │                    │ Purchase │  │   Cancel     │  │  │    │
│  │  │                    └──────────┘  └──────────────┘  │  │    │
│  │  └─────────────────────────────────────────────────────┘  │    │
│  └───────────────────────────────────────────────────────────┘    │
└───────────────────────────────────────────────────────────────────┘
```

### 2.1 Region 索引

| widget_id | 类型 | 用途 | 对应 Wireframe 位置 |
|-----------|------|------|-------------------|
| `shop_screen_root` | Container | Popup 根容器，包含背景遮罩和居中弹窗 | 整个线框图外层 |
| `shop_backdrop` | Panel | 半透明遮罩背景，阻止下层交互 | 遮罩层 |
| `shop_popup` | Container | 商店弹窗主体内容面板 | 居中弹窗 |
| `shop_header` | Container | 顶部标题栏，水平排列商店名称和关闭按钮 | 弹窗顶部第一行 |
| `shop_header_name` | HeadingText | 商店名称文本（来自 ShopVm.shop_name） | Header 左侧 |
| `shop_header_close` | IconButton | 关闭按钮，点击触发 OverlayStack.Pop | Header 右上角 |
| `shop_player_gold` | StatText | 玩家当前金币数量显示 | 右上角，与 Header 平级 |
| `shop_tab_panel` | Container | 购买/出售标签页选择栏 | Header 下方 Tab 行 |
| `shop_tab_buy` | TabButton | 购买标签按钮 | TabPanel 左侧 |
| `shop_tab_sell` | TabButton | 出售标签按钮 | TabPanel 右侧 |
| `shop_content` | Container | 内容区域容器，包裹 BuyTab/SellTab 的内容区 | TabPanel 下方 |
| `shop_buy_list` | Container | 购买标签页的物品列表容器（仅 BuyTab 激活时可见） | Content 区域 BuyTab 激活时 |
| `shop_item_card` | ShopItemCard | 单个商品卡片：图标 + 名称 + 属性 + 价格 + 购买按钮 | BuyList 中的卡片条目 |
| `shop_sell_list` | Container | 出售标签页的背包物品列表容器（仅 SellTab 激活时可见） | Content 区域 SellTab 激活时 |
| `shop_item_row` | InventoryItemRow | 单个背包物品行：图标 + 名称 + 数量 + 出售按钮 + 价格 | SellList 中的行条目 |
| `shop_empty_buy` | EmptyWidget | 商品列表为空时的空状态提示 | BuyList 为空时 |
| `shop_empty_sell` | EmptyWidget | 无物品可出售时的空状态提示 | SellList 为空时 |
| `shop_loading` | Spinner | 数据加载中的加载指示器 | Content 区域 |
| `shop_error_state` | Container | 数据加载失败的错误提示容器 | Content 区域 |
| `shop_error_text` | BodyText | 错误信息文本 | 错误提示面板 |
| `shop_retry_btn` | Button, Secondary | 重试按钮 | 错误提示面板 |
| `shop_cart_panel` | Container | 底部购物车面板 | 弹窗最底部 |
| `shop_cart_summary` | CaptionText | 购物车摘要文本："已选 {N} 件，合计 {total}G" | CartPanel 左侧 |
| `shop_buy_btn` | Button, Primary | 购买/出售确认按钮（文本根据当前 Tab 动态："Purchase"/"Sell"） | CartPanel 右侧第一个 |
| `shop_cancel_btn` | Button, Secondary | 取消/关闭按钮 | CartPanel 右侧第二个 |

---

## 3. Widget Tree

> 标注 `[widget_id: WidgetType]` 的树结构。禁止隐藏节点，必须完整。

```
ShopScreenPopup                                         [shop_screen_root: Container]
├── Backdrop                                             [shop_backdrop: Panel]
└── ShopPopup                                            [shop_popup: Container]
    ├── Header                                           [shop_header: Container]
    │   ├── ShopName                                     [shop_header_name: HeadingText]
    │   └── CloseButton                                  [shop_header_close: IconButton]
    ├── PlayerGold                                       [shop_player_gold: StatText]
    ├── TabPanel                                         [shop_tab_panel: Container]
    │   ├── BuyTab                                       [shop_tab_buy: TabButton]
    │   └── SellTab                                      [shop_tab_sell: TabButton]
    ├── ContentArea                                      [shop_content: Container]
    │   ├── BuyList                                      [shop_buy_list: Container]          — 仅 BuyTab 激活时可见
    │   │   ├── ShopItemCard × N                         [shop_item_card: ShopItemCard]
    │   │   ├── EmptyState                               [shop_empty_buy: EmptyWidget]
    │   │   ├── LoadingIndicator                         [shop_loading: Spinner]
    │   │   └── ErrorPanel                               [shop_error_state: Container]
    │   │       ├── ErrorText                            [shop_error_text: BodyText]
    │   │       └── RetryButton                          [shop_retry_btn: Button, Secondary]
    │   └── SellList                                     [shop_sell_list: Container]          — 仅 SellTab 激活时可见
    │       ├── InventoryItemRow × N                     [shop_item_row: InventoryItemRow]
    │       ├── EmptyState                               [shop_empty_sell: EmptyWidget]
    │       ├── LoadingIndicator                         [shop_loading: Spinner]               — 与 BuyList 复用同一 Spinner 实体
    │       └── ErrorPanel                               [shop_error_state: Container]         — 与 BuyList 复用同一 ErrorPanel
    └── CartPanel                                        [shop_cart_panel: Container]
        ├── CartSummary                                  [shop_cart_summary: CaptionText]
        ├── BuyButton                                    [shop_buy_btn: Button, Primary]
        └── CancelButton                                 [shop_cancel_btn: Button, Secondary]
```

### 3.1 Widget Type 索引

| widget_id | WidgetType | 定义位置 | 复用于 |
|-----------|-----------|---------|--------|
| `shop_screen_root` | Container | — | 各 Screen/Overlay 根容器 |
| `shop_backdrop` | Panel | `02-design-system/widget-atoms.md §Panel` | ModalOverlay、各 PopupOverlay |
| `shop_popup` | Container | — | —（ShopScreen 专有） |
| `shop_header` | Container | — | MainMenuScreen、各 PopupOverlay Header |
| `shop_header_name` | Atom: HeadingText | `02-design-system/widget-atoms.md §HeadingText` | 各 Screen 标题 |
| `shop_header_close` | Atom: IconButton | `02-design-system/widget-atoms.md §IconButton` | InventoryScreen、SettingsScreen |
| `shop_player_gold` | Atom: StatText | `02-design-system/widget-atoms.md §StatText` | —（ShopScreen 专有） |
| `shop_tab_panel` | Container | — | SettingsScreen TabPanel（参照） |
| `shop_tab_buy` | Atom: TabButton | `02-design-system/widget-atoms.md §TabButton` | — |
| `shop_tab_sell` | Atom: TabButton | `02-design-system/widget-atoms.md §TabButton` | — |
| `shop_content` | Container | — | 各 Screen 内容区容器 |
| `shop_buy_list` | Container | — | — |
| `shop_item_card` | Molecule: ShopItemCard | `02-design-system/widget-composites.md §TBD — ShopItemCard Molecule` | —（ShopScreen 专有） |
| `shop_sell_list` | Container | — | — |
| `shop_item_row` | Molecule: InventoryItemRow | `02-design-system/widget-composites.md §TBD — InventoryItemRow Molecule` | InventoryScreen（参照） |
| `shop_empty_buy` | Atom: EmptyWidget | `02-design-system/widget-atoms.md §TBD — EmptyWidget Atom` | 各 Screen 空列表提示 |
| `shop_empty_sell` | Atom: EmptyWidget | `02-design-system/widget-atoms.md §TBD — EmptyWidget Atom` | 各 Screen 空列表提示 |
| `shop_loading` | Atom: Spinner | `02-design-system/widget-atoms.md §TBD — Spinner Atom` | InventoryScreen、各 Screen 加载 |
| `shop_error_state` | Container | — | InventoryScreen（参照） |
| `shop_error_text` | Atom: BodyText | `02-design-system/widget-atoms.md §BodyText` | InventoryScreen |
| `shop_retry_btn` | Atom: Button (Secondary) | `02-design-system/widget-atoms.md §Button` | InventoryScreen |
| `shop_cart_panel` | Container | — | — |
| `shop_cart_summary` | Atom: CaptionText | `02-design-system/widget-atoms.md §CaptionText` | —（ShopScreen 专有） |
| `shop_buy_btn` | Atom: Button (Primary) | `02-design-system/widget-atoms.md §Button` | — |
| `shop_cancel_btn` | Atom: Button (Secondary) | `02-design-system/widget-atoms.md §Button` | 各 Screen 取消按钮 |

---

## 4. Flexbox Layout

> YAML 格式。每个 widget_id 必须有 direction / width / height / flex_grow / intent。

```yaml
## Flexbox Layout — ShopScreen
## width/height: px 值或 "auto" 或 "fill"
## flex_grow: 0=不增长, 1=等分剩余空间, 2=双倍增长
## shrink: none/low/high — 收缩优先级

shop_screen_root:
  direction: column
  width: 100%
  height: 100%
  flex_grow: 0
  intent: "Popup 根容器，占满整个视口，垂直排列背景遮罩（全屏）和居中弹窗（position:absolute 居中）"

shop_backdrop:
  direction: column
  width: 100%
  height: 100%
  flex_grow: 0
  shrink: none
  intent: "半透明遮罩，占满全屏，z=300 阻止下层交互"

shop_popup:
  direction: column
  width: 600
  height: 520
  flex_grow: 0
  shrink: none
  intent: "商店弹窗主体，固定 600x520（自适应布局），居中于视口（position:center），垂直排列 Header + TabPanel + Content + CartPanel"

shop_header:
  direction: row
  width: 100%
  height: 48
  flex_grow: 0
  shrink: none
  intent: "Header 栏，固定 48px 高度，水平排列商店名称（左）和关闭按钮（右）"

shop_header_name:
  direction: row
  width: auto
  height: auto
  flex_grow: 1
  shrink: none
  intent: "商店名称文本，auto 宽度适配文本长度，flex_grow:1 推动关闭按钮到右侧"

shop_header_close:
  direction: row
  width: 40
  height: 40
  flex_grow: 0
  shrink: none
  intent: "关闭按钮，固定 40x40 保证最小触摸目标"

shop_player_gold:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  shrink: none
  intent: "玩家金币显示，absolute 定位或与 Header 同排右对齐，auto 尺寸适配文本'Gold: 1500G'"

shop_tab_panel:
  direction: row
  width: 100%
  height: 40
  flex_grow: 0
  shrink: none
  intent: "Tab 选择栏，固定 40px 高度，水平排列 Buy/Sell 两个 TabButton"

shop_tab_buy:
  direction: row
  width: auto
  height: 40
  flex_grow: 1
  shrink: low
  intent: "购买 Tab 按钮，flex_grow:1 与 SellTab 平分 TabPanel 宽度"

shop_tab_sell:
  direction: row
  width: auto
  height: 40
  flex_grow: 1
  shrink: low
  intent: "出售 Tab 按钮，flex_grow:1 与 BuyTab 平分 TabPanel 宽度"

shop_content:
  direction: column
  width: 100%
  height: fill
  flex_grow: 1
  intent: "内容区域，弹性填充 Header+TabPanel+CartPanel 之间的所有垂直空间"

shop_buy_list:
  direction: column
  width: 100%
  height: 100%
  flex_grow: 1
  intent: "购买标签页物品列表容器，垂直排列 ShopItemCard，可滚动"

shop_item_card:
  direction: row
  width: 100%
  height: 72
  flex_grow: 0
  shrink: none
  intent: "商品卡片，固定 72px 高度，水平排列图标(48x48) + 名称属性 + 价格购买按钮"

shop_sell_list:
  direction: column
  width: 100%
  height: 100%
  flex_grow: 1
  intent: "出售标签页物品列表容器，垂直排列 InventoryItemRow，可滚动"

shop_item_row:
  direction: row
  width: 100%
  height: 56
  flex_grow: 0
  shrink: none
  intent: "背包物品行，固定 56px 高度，水平排列图标 + 名称 + 数量 + 出售按钮"

shop_empty_buy:
  direction: column
  width: 100%
  height: fill
  flex_grow: 1
  intent: "购买列表空状态，居中显示空提示文本，占满剩余空间"

shop_empty_sell:
  direction: column
  width: 100%
  height: fill
  flex_grow: 1
  intent: "出售列表空状态，居中显示空提示文本，占满剩余空间"

shop_loading:
  direction: column
  width: 100%
  height: fill
  flex_grow: 1
  intent: "加载指示器，居中显示 Spinner，占满剩余空间"

shop_error_state:
  direction: column
  width: 100%
  height: fill
  flex_grow: 1
  intent: "错误提示容器，居中排列错误文本 + 重试按钮"

shop_error_text:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  intent: "错误信息文本，auto 尺寸适应错误内容"

shop_retry_btn:
  direction: row
  width: 120
  height: 40
  flex_grow: 0
  shrink: none
  intent: "重试按钮，固定 120x40 确保最小触摸目标"

shop_cart_panel:
  direction: row
  width: 100%
  height: 56
  flex_grow: 0
  shrink: none
  intent: "底部购物车面板，固定 56px 高度，水平排列摘要（左对齐）和操作按钮（右对齐）"

shop_cart_summary:
  direction: row
  width: auto
  height: auto
  flex_grow: 1
  shrink: none
  intent: "购物车摘要文本，auto 宽度自适应，flex_grow:1 推动按钮到右侧"

shop_buy_btn:
  direction: row
  width: 120
  height: 40
  flex_grow: 0
  shrink: none
  intent: "购买/出售确认按钮，固定 120x40，Primary 样式，大于最小触摸目标"

shop_cancel_btn:
  direction: row
  width: 100
  height: 40
  flex_grow: 0
  shrink: none
  intent: "取消按钮，固定 100x40，Secondary 样式，略小于 Primary 按钮"
```

---

## 5. Responsive Rules

| 条件 | 行为 | 影响区域 |
|------|------|---------|
| width < 1280px | strategy: "none" — 当前不实现响应式。ShopScreen 作为固定尺寸 Popup 弹窗（600x520），不受视口缩放影响 | 全部 |

**最小支持分辨率**: 1280 x 720 (16:9)
**设计分辨率**: 1920 x 1080 (16:9)

---

## 6. Region Responsibility

> 每个 region 3-8 条职责。明确该区域"展示什么"和"不做什么"。

### 6.1 shop_header

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示商店名称文本（LocalizationKey 或 ShopVm.shop_name） | Display | 商店标题缺失 |
| R02 | 展示关闭按钮（IconButton），图标为 "X" | Display | 用户无法关闭商店 |
| R03 | 响应关闭按钮点击，触发 `OverlayStack.Pop(ShopScreen)` | Interaction | 关闭功能失效 |
| R04 | 标题文本左对齐，关闭按钮右对齐，水平两端布局 | Layout | 布局违反预期 |

**不负责**:
- 金币显示（属于 shop_player_gold 职责）
- 购买/出售逻辑

### 6.2 shop_player_gold

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示玩家当前金币数量（来自 ShopVm.player_gold），格式："Gold: {n}" | Display | 玩家无法看到自己的金币 |
| R02 | 购买/出售完成后自动刷新显示更新后的金币数量 | Display | 交易后金币未更新误导玩家 |

**不负责**:
- 商店名称显示（属于 shop_header）
- 物品价格计算

### 6.3 shop_tab_panel

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示两个 Tab 按钮：Buy（购买）和 Sell（出售） | Display | 用户无法切换模式 |
| R02 | 标记当前激活的 Tab 为高亮/下划线样式（is_active） | Display | 当前模式不可辨识 |
| R03 | 响应 Tab 按钮点击，切换激活 Tab，切换显示 BuyList/SellList | Interaction | 无法切换购买/出售视图 |
| R04 | 管理 Tab 按钮间焦点位移（ArrowLeft/Right 导航） | Focus | 键盘无法在 Tab 间切换 |

**不负责**:
- 具体商品列表渲染（属于 shop_buy_list/shop_sell_list）

### 6.4 shop_content

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 根据当前激活 Tab 显示对应的列表（BuyTab → shop_buy_list, SellTab → shop_sell_list） | Display | 内容与 Tab 不匹配 |
| R02 | 数据加载中显示加载指示器（shop_loading: Spinner） | Display | 加载无反馈 |
| R03 | 数据加载失败时显示错误面板（shop_error_state） | Display | 错误无反馈/无法重试 |
| R04 | 管理列表项（shop_item_card/shop_item_row）间焦点位移（ArrowUp/Down 导航） | Focus | 键盘无法在物品间移动 |

**不负责**:
- Tab 切换逻辑（属于 shop_tab_panel）
- 购物车汇总（属于 shop_cart_panel）

### 6.5 shop_item_card（BuyTab）

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示商品图标（IconType, 48x48） | Display | 商品无图标 |
| R02 | 展示商品名称（LocalizedText, from ShopVm.items[n].name） | Display | 商品名称不可见 |
| R03 | 展示商品属性简述（类型、效果等，from ShopVm.items[n].description） | Display | 商品属性不可见 |
| R04 | 展示商品价格（数字 + "G"，from ShopVm.items[n].price） | Display | 价格不可见 |
| R05 | 点击购买按钮触发购买流程（弹出 ModalOverlay 选择数量） | Interaction | 购买功能失效 |
| R06 | 如玩家金币不足以购买该商品，价格显示为红色警示色 | Display | 玩家可能误买 |
| R07 | 如商品已售罄或不可购买（库存为 0），显示"已售罄"标记，购买按钮 disabled | Display | 玩家可对售罄商品操作 |

### 6.6 shop_item_row（SellTab）

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示物品图标（IconType, 40x40） | Display | 物品无图标 |
| R02 | 展示物品名称（LocalizedText, from InventoryVm.items[n].name） | Display | 物品名称不可见 |
| R03 | 展示物品数量（x1, x3 等） | Display | 数量不可见 |
| R04 | 展示出售价格（from ShopVm.sell_prices[n]） | Display | 出售价格不可见 |
| R05 | 点击出售按钮触发出售流程（弹出 ModalOverlay 选择数量） | Interaction | 出售功能失效 |

### 6.7 shop_cart_panel

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示购物车摘要：已选物品数量 + 总价（"已选 {N} 件，合计 {total}G"） | Display | 玩家看不到购物车状态 |
| R02 | 购物车为空时显示"未选择任何物品"占位文本 | Display | 空购物车无提示 |
| R03 | 展示购买/出售按钮（Primary 样式，文本根据 Tab 动态变化） | Display | 交易入口缺失 |
| R04 | 展示取消按钮（Secondary 样式） | Display | 取消入口缺失 |
| R05 | 购物车为空时 BuyButton 为 disabled 状态 | Interaction | 空购物车可误操作 |
| R06 | 点击 BuyButton 触发最后确认并执行交易（ModalOverlay 确认 → UiCommand::BuyItem/SellItem） | Interaction | 购买/出售功能失效 |
| R07 | 点击 CancelButton 或 Esc 触发 OverlayStack.Pop | Interaction | 无法关闭商店 |
| R08 | 交易完成后显示 NotificationOverlay 反馈结果，更新 CartPanel 为初始空状态 | Interaction | 交易后无反馈 |

**不负责**:
- 商品列表渲染（属于 shop_content）
- 金币显示（属于 shop_player_gold）

---

## 7. Widget Contract

> Inputs / Outputs / Selection Model。

### 7.1 shop_header_name

```yaml
widget_id: shop_header_name
widget_type: HeadingText
defined_in: "02-design-system/widget-atoms.md §HeadingText"

inputs:
  - name: text
    type: LocalizedText
    source: "ShopVm.shop_name"
    default: "Shop"

outputs: []

selection_model:
  type: none
```

### 7.2 shop_header_close

```yaml
widget_id: shop_header_close
widget_type: IconButton
defined_in: "02-design-system/widget-atoms.md §IconButton"

inputs:
  - name: icon
    type: IconType::Close
    source: "Theme Icon set"
    default: "close_x"

outputs:
  - name: clicked
    type: UiCommand::CloseOverlay
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.3 shop_player_gold

```yaml
widget_id: shop_player_gold
widget_type: StatText
defined_in: "02-design-system/widget-atoms.md §StatText"

inputs:
  - name: text
    type: FormattedText
    source: "format!('Gold: {}', ShopVm.player_gold)"
    default: "Gold: 0"

outputs: []

selection_model:
  type: none
```

### 7.4 shop_tab_buy

```yaml
widget_id: shop_tab_buy
widget_type: TabButton
defined_in: "02-design-system/widget-atoms.md §TabButton"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.shop.tab.buy"
    default: "Buy"
  - name: is_active
    type: bool
    source: "ShopScreenState.selected_tab == TabType::Buy"
    default: true

outputs:
  - name: selected
    type: UiCommand::SwitchShopTab
    payload: TabType::Buy
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.5 shop_tab_sell

```yaml
widget_id: shop_tab_sell
widget_type: TabButton
defined_in: "02-design-system/widget-atoms.md §TabButton"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.shop.tab.sell"
    default: "Sell"
  - name: is_active
    type: bool
    source: "ShopScreenState.selected_tab == TabType::Sell"
    default: false

outputs:
  - name: selected
    type: UiCommand::SwitchShopTab
    payload: TabType::Sell
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.6 shop_item_card

```yaml
widget_id: shop_item_card
widget_type: ShopItemCard
defined_in: "02-design-system/widget-composites.md §TBD — ShopItemCard Molecule"

inputs:
  - name: item_id
    type: ItemId
    source: "ShopVm.items[n].id"
    default: None
  - name: icon
    type: IconType
    source: "ShopVm.items[n].icon"
    default: "default_item"
  - name: name
    type: LocalizedText
    source: "ShopVm.items[n].name"
    default: ""
  - name: description
    type: LocalizedText
    source: "ShopVm.items[n].short_desc"
    default: ""
  - name: price
    type: u32
    source: "ShopVm.items[n].price"
    default: 0
  - name: stock
    type: u32
    source: "ShopVm.items[n].stock"
    default: 0
  - name: is_affordable
    type: bool
    source: "ShopVm.player_gold >= ShopVm.items[n].price"
    default: true
  - name: is_in_stock
    type: bool
    source: "ShopVm.items[n].stock > 0"
    default: true
  - name: quantity_in_cart
    type: u32
    source: "ShopScreenState.cart_items[item_id]"
    default: 0

outputs:
  - name: add_to_cart
    type: UiCommand::AddToCart
    payload: "{ item_id: ItemId, quantity: u32 }"
    trigger: OnLeftClick
    conditions:
      - is_in_stock == true
      - is_affordable == true

  - name: remove_from_cart
    type: UiCommand::RemoveFromCart
    payload: "{ item_id: ItemId }"
    trigger: OnLeftClick
    conditions:
      - quantity_in_cart > 0

  - name: hovered
    type: UiCommand::ShowTooltip
    payload: "{ item_id: ItemId }"
    trigger: OnHover(500ms)

selection_model:
  type: none
```

### 7.7 shop_item_row

```yaml
widget_id: shop_item_row
widget_type: InventoryItemRow
defined_in: "02-design-system/widget-composites.md §TBD — InventoryItemRow Molecule"

inputs:
  - name: item_id
    type: ItemId
    source: "InventoryVm.items[n].id"
    default: None
  - name: icon
    type: IconType
    source: "InventoryVm.items[n].icon"
    default: "default_item"
  - name: name
    type: LocalizedText
    source: "InventoryVm.items[n].name"
    default: ""
  - name: description
    type: LocalizedText
    source: "InventoryVm.items[n].short_desc"
    default: ""
  - name: quantity
    type: u32
    source: "InventoryVm.items[n].quantity"
    default: 1
  - name: sell_price
    type: u32
    source: "ShopVm.sell_prices[n]"
    default: 0

outputs:
  - name: add_to_cart
    type: UiCommand::AddToCart
    payload: "{ item_id: ItemId, quantity: u32 }"
    trigger: OnLeftClick

  - name: hovered
    type: UiCommand::ShowTooltip
    payload: "{ item_id: ItemId }"
    trigger: OnHover(500ms)

selection_model:
  type: none
```

### 7.8 shop_empty_buy / shop_empty_sell

```yaml
widget_id: shop_empty_buy
widget_type: EmptyWidget
defined_in: "02-design-system/widget-atoms.md §TBD — EmptyWidget Atom"

inputs:
  - name: icon
    type: IconType
    source: "hardcoded"
    default: "empty_box"
  - name: title
    type: LocalizedText
    source: "ui.shop.empty.buy.title"
    default: "No Items Available"
  - name: description
    type: LocalizedText
    source: "ui.shop.empty.buy.description"
    default: "This merchant has nothing to sell at the moment."

outputs: []

selection_model:
  type: none
```

```yaml
widget_id: shop_empty_sell
widget_type: EmptyWidget
defined_in: "02-design-system/widget-atoms.md §TBD — EmptyWidget Atom"

inputs:
  - name: icon
    type: IconType
    source: "hardcoded"
    default: "empty_bag"
  - name: title
    type: LocalizedText
    source: "ui.shop.empty.sell.title"
    default: "Nothing to Sell"
  - name: description
    type: LocalizedText
    source: "ui.shop.empty.sell.description"
    default: "You have no items that can be sold."

outputs: []

selection_model:
  type: none
```

### 7.9 shop_loading

```yaml
widget_id: shop_loading
widget_type: Spinner
defined_in: "02-design-system/widget-atoms.md §TBD — Spinner Atom"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.shop.loading"
    default: "Loading..."

outputs: []

selection_model:
  type: none
```

### 7.10 shop_error_text

```yaml
widget_id: shop_error_text
widget_type: BodyText
defined_in: "02-design-system/widget-atoms.md §BodyText"

inputs:
  - name: text
    type: LocalizedText
    source: "ShopVm.error_message"
    default: "Failed to load shop data."

outputs: []

selection_model:
  type: none
```

### 7.11 shop_retry_btn

```yaml
widget_id: shop_retry_btn
widget_type: Button, Secondary
defined_in: "02-design-system/widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.shop.retry"
    default: "Retry"

outputs:
  - name: clicked
    type: UiCommand::ReloadShopData
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.12 shop_cart_summary

```yaml
widget_id: shop_cart_summary
widget_type: CaptionText
defined_in: "02-design-system/widget-atoms.md §CaptionText"

inputs:
  - name: text
    type: FormattedText
    source: "format!('Selected: {} items  Total: {}G', cart_count, cart_total)"
    default: "No items selected"
  - name: visible
    type: bool
    source: "ShopScreenState.cart_items.len() > 0"
    default: false

outputs: []

selection_model:
  type: none
```

### 7.13 shop_buy_btn

```yaml
widget_id: shop_buy_btn
widget_type: Button, Primary
defined_in: "02-design-system/widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.shop.purchase"  # BuyTab: "Purchase", SellTab: "Confirm Sell"
    default: "Purchase"
  - name: enabled
    type: bool
    source: "ShopScreenState.cart_items.len() > 0"
    default: false

outputs:
  - name: clicked
    type: UiCommand::ConfirmPurchase
    payload: "{ tab: TabType, items: Vec<(ItemId, u32)> }"
    trigger: OnLeftClick
    conditions:
      - buy_button.enabled == true

selection_model:
  type: none
```

### 7.14 shop_cancel_btn

```yaml
widget_id: shop_cancel_btn
widget_type: Button, Secondary
defined_in: "02-design-system/widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.shop.cancel"
    default: "Cancel"

outputs:
  - name: clicked
    type: UiCommand::CloseOverlay
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

---

## 8. State Mapping (Per-Region)

> 每个 region 独立的状态。必须定义 Loading / Empty / Normal / Error 四种状态。
> AI 实现时必须为每种状态提供对应的 UI 展示。

### 8.1 shop_header

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 标题和关闭按钮为静态 LocalizedKey + 主题图标，无异步加载 | — | — |
| **Empty** | N/A — 标题始终有值，关闭按钮始终存在 | — | — |
| **Normal** | 商店名称 HeadingText + 关闭图标 IconButton 正常显示 | OnEnter 完成 | SlideUp 入场动画 |
| **Error** | N/A — 静态元素无错误状态 | — | — |

### 8.2 shop_player_gold

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 金币数值随 ShopVm 一起加载 | — | — |
| **Empty** | N/A — 金币始终有值（最低为 0） | — | — |
| **Normal** | "Gold: {n}G" — 正常显示；若金币不足以购买当前选中商品，颜色变红 | ShopVm.player_gold 就绪 | 数值变化时有数字滚动动画 (0.15s) |
| **Error** | N/A — 金币数据不独立加载 | — | — |

### 8.3 shop_tab_panel

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — Tab 按钮为静态 LocalizedKey | — | — |
| **Empty** | N/A — Tab 按钮始终存在 | — | — |
| **Normal** | BuyTab + SellTab 两个 TabButton，当前激活的 Tab 高亮样式 | OnEnter 完成，默认选中 BuyTab | Tab 切换时滑动条动画 0.15s |
| **Error** | N/A — Tab 按钮无错误状态 | — | — |

### 8.4 shop_content (BuyTab)

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | `shop_loading`（Spinner 居中 + "Loading..." CaptionText） | ShopVm 商品数据未就绪（首次加载或重试中） | Spinner 旋转动画 |
| **Empty** | `shop_empty_buy`（EmptyWidget: 图标 + "No Items Available" + 描述文本） | ShopVm.items.len() == 0 | 淡入 0.2s |
| **Normal** | `shop_buy_list` 中排列 ShopItemCard × N，显示商品名称/属性/价格/购买按钮 | ShopVm.items.len() > 0 | 卡片逐个淡入 (stagger 30ms) |
| **Error** | `shop_error_state`（`shop_error_text` + `shop_retry_btn`） | ShopVm 数据加载失败 | 错误提示淡入 0.2s |

### 8.5 shop_content (SellTab)

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | `shop_loading`（Spinner，复用 BuyTab 的 Spinner 实体） | InventoryVm 数据未就绪 | Spinner 旋转动画 |
| **Empty** | `shop_empty_sell`（EmptyWidget: 图标 + "Nothing to Sell" + 描述文本） | 无可出售的物品 | 淡入 0.2s |
| **Normal** | `shop_sell_list` 中排列 InventoryItemRow × N，物品名称/数量/出售按钮 | 有可出售的物品 | 行逐个淡入 (stagger 30ms) |
| **Error** | `shop_error_state`（`shop_error_text` + `shop_retry_btn`，与 BuyTab 复用） | InventoryVm 数据加载失败 | 错误提示淡入 0.2s |

**注意**:
- BuyList 和 SellList 共用同一个 Content 区域，同一时间仅一个 Tab 的列表可见
- 切换 Tab 时，旧列表立即隐藏，新列表根据数据状态显示 Loading/Empty/Normal/Error
- Loading 和 Error 状态在两个 Tab 间复用，Empty 状态各有独立 EmptyWidget

### 8.6 shop_cart_panel

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 购物车汇总为本地状态，无异步加载 | — | — |
| **Empty** | `shop_cart_summary` 显示 "No items selected", BuyButton disabled | ShopScreenState.cart_items.len() == 0 | 无过渡 |
| **Normal** | `shop_cart_summary` 显示已选数量和总价, BuyButton enabled | ShopScreenState.cart_items.len() > 0 | 摘要数字变更动画 0.1s |
| **PostPurchase** | `shop_cart_summary` 显示 "Transaction complete!", BuyButton 短暂 disabled 后重置 | 交易成功返回 | 提示文本淡入 0.2s → 2s 后重置为空状态 |
| **Error** | `shop_cart_summary` 显示错误提示和总价（不影响购买按钮状态） | 交易失败返回错误信息 | 错误文本淡入 0.2s |

---

## 9. Focus Navigation

> Tab 导航路径。按 Tab 键的顺序就是导航路径的顺序。
> ShopScreen 作为 PopupOverlay，焦点被锁定在弹窗内（focus_trap: true）。

```yaml
focus_path:
  ## 阶段 1: Tab 选择
  - shop_tab_buy               # Tab 1 — 默认激活
  - shop_tab_sell              # Tab 2

  ## 阶段 2: 物品列表
  - shop_buy_list              # Tab 组 3 — ShopItemCard 集合 (BuyTab 激活时)
    ## 子组内: ArrowUp/ArrowDown 在 ShopItemCard 间移动
    # - shop_item_card_0       # 默认第一行
    # - shop_item_card_1
    # - shop_item_card_N
  - shop_sell_list             # Tab 组 4 — InventoryItemRow 集合 (SellTab 激活时)
    ## 子组内: ArrowUp/ArrowDown 在 InventoryItemRow 间移动
    # - shop_item_row_0
    # - shop_item_row_1
    # - shop_item_row_N

  ## 阶段 3: CartPanel 操作按钮
  - shop_buy_btn               # Tab 5 — 主要操作按钮
  - shop_cancel_btn            # Tab 6

  ## 阶段 4: 关闭按钮
  - shop_header_close          # Tab 7

special_keys:
  Escape: "触发 shop_header_close.clicked, OverlayStack.Pop"
  Enter: "激活当前焦点按钮 / 确认当前选中"
  ArrowUp: "物品列表内上移一行"
  ArrowDown: "物品列表内下移一行"
  ArrowLeft: "Tab 按钮间切换"                  # 仅在 shop_tab_buy/shop_tab_sell 组内有效
  ArrowRight: "Tab 按钮间切换"                 # 仅在 shop_tab_buy/shop_tab_sell 组内有效
  Tab: "按 focus_path 顺序前进"
  Shift+Tab: "按 focus_path 逆序后退"

focus_trap: true          # true = 焦点锁定在 ShopScreen 弹窗内，Tab 循环
```

### 9.1 默认焦点

进入 ShopScreen 时，默认焦点落在 `shop_tab_buy`（购买 Tab 按钮）。
当从 ModalOverlay（购买数量确认）返回时，焦点回到 `shop_buy_btn`。

---

## 10. Interaction Zones

> 每个可交互区域的行为定义。

### 10.1 shop_header_close

```yaml
zone_id: shop_header_close
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::CloseOverlay, OverlayStack.Pop, 关闭商店弹窗"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "IconButton 变为 Hover 样式（高亮变体）"
    leave_effect: "恢复 IconButton 默认样式"
    delay: 0ms
```

### 10.2 shop_tab_buy

```yaml
zone_id: shop_tab_buy
interactions:
  - type: click
    button: Left
    effect: "激活 Buy Tab, 设置 selected_tab=Buy, 隐藏 SellList 显示 BuyList"
    cursor: Pointer
    conditions:
      - tab != already_active: "仅当 BuyTab 未激活时切换"

  - type: hover
    enter_effect: "TabButton 变为 Hover 样式"
    leave_effect: "恢复 TabButton 默认样式（激活态除外）"
    delay: 0ms
```

### 10.3 shop_tab_sell

```yaml
zone_id: shop_tab_sell
interactions:
  - type: click
    button: Left
    effect: "激活 Sell Tab, 设置 selected_tab=Sell, 隐藏 BuyList 显示 SellList"
    cursor: Pointer
    conditions:
      - tab != already_active

  - type: hover
    enter_effect: "TabButton 变为 Hover 样式"
    leave_effect: "恢复 TabButton 默认样式"
    delay: 0ms
```

### 10.4 shop_item_card

```yaml
zone_id: shop_item_card
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::AddToCart { item_id, quantity: 1 }，将商品添加到购物车，更新 CartPanel 摘要。如商品已在购物车中，点击改为从购物车移除（toggle 行为）或弹出 ModalOverlay 选择数量。"
    cursor: Pointer
    conditions:
      - card.is_in_stock == true

  - type: hover
    enter_effect: "ShopItemCard 边框高亮或背景轻微变化，延迟 500ms 后弹出 TooltipOverlay（商品完整说明 + 属性详情）"
    leave_effect: "取消 TooltipOverlay，恢复卡片默认样式"
    delay: 500ms
```

### 10.5 shop_item_row

```yaml
zone_id: shop_item_row
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::AddToCart { item_id, quantity: 1 }，将物品添加到出售购物车"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "InventoryItemRow 背景轻微高亮，延迟 500ms 后弹出 TooltipOverlay（物品完整说明）"
    leave_effect: "取消 TooltipOverlay，恢复行默认样式"
    delay: 500ms
```

### 10.6 shop_buy_btn

```yaml
zone_id: shop_buy_btn
interactions:
  - type: click
    button: Left
    effect: |
      BuyTab 激活时: 弹出 ModalOverlay（购买确认：商品列表 + 总价 + 确认/取消按钮）
      确认后触发 UiCommand::ConfirmPurchase { tab: Buy, items: Vec<(ItemId, u32)> }
      SellTab 激活时: 弹出 ModalOverlay（出售确认：物品列表 + 总收入 + 确认/取消按钮）
      确认后触发 UiCommand::ConfirmPurchase { tab: Sell, items: Vec<(ItemId, u32)> }
    cursor: Pointer
    conditions:
      - cart_items.len() > 0

  - type: hover
    enter_effect: "Primary Button 高亮变体"
    leave_effect: "恢复 Primary 默认样式"
    delay: 0ms
```

### 10.7 shop_cancel_btn

```yaml
zone_id: shop_cancel_btn
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::CloseOverlay, 关闭商店弹窗，不清空购物车（购物车状态随 ShopScreen despawn 自然清理）"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "Secondary Button 高亮变体"
    leave_effect: "恢复 Secondary 默认样式"
    delay: 0ms
```

### 10.8 shop_retry_btn

```yaml
zone_id: shop_retry_btn
interactions:
  - type: click
    button: Left
    effect: "重新触发 ShopVm/InventoryVm 数据加载，Content 区域切换到 Loading 状态"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "Secondary Button 高亮变体"
    leave_effect: "恢复 Secondary 默认样式"
    delay: 0ms
```

---

## 11. Overlay Definition

> Overlay 列表 + Z-Layer。ShopScreen 本身已是 PopupOverlay，在其之上需要附加的 Overlay 定义如下。

| Overlay | 用途 | Z-Layer | 类型 | 触发条件 |
|---------|------|---------|------|---------|
| ModalOverlay | 购买确认 + 数量选择（显示商品清单、总价、确认/取消） | popup_layer (303) | Modal | 点击 `shop_buy_btn`（购物车非空时） |
| NotificationOverlay | 交易结果反馈（购买成功/失败/金币不足等） | notification_layer (400) | Notification | 交易处理完成后（UiCommand::ConfirmPurchase 的结果） |

### 11.1 Z-Layer 分配

| Z-Layer | 用途 | 包含 |
|---------|------|------|
| 300 | ShopScreen 自身背景遮罩 | `shop_backdrop` |
| 302 | ShopScreen 弹窗主体 | `shop_popup` 及所有子 region（分配值遵循 z-layer-spec.md §2.3.4） |
| 303 | ModalOverlay (购买确认) | ModalOverlay 遮罩 (z=303) + ModalOverlay 弹窗 (z=304) |
| 400 | 交易结果通知 | NotificationOverlay (交易结果 Toast) |

### 11.2 ModalOverlay 内部结构（购买确认）

```
ModalOverlay (PurchaseConfirm)
├── ModalBackdrop    [modal_backdrop: Panel — 半透明遮罩, z=303]
└── ModalDialog      [shop_modal_dialog: Container — 居中弹窗, z=304]
    ├── ModalTitle   [shop_modal_title: HeadingText]              — "ui.shop.confirm.title"
    ├── ModalBody    [shop_modal_body: Container]
    │   ├── ItemList [shop_modal_item_list: Container]            — 购物车物品清单
    │   │   └── ItemRow × N                                      — 物品名 + 数量 + 小计
    │   └── TotalRow [shop_modal_total: CaptionText]              — "Total: 350G"
    ├── ConfirmBtn   [shop_modal_confirm_btn: Button, Primary]    — "ui.shop.confirm.yes"
    └── CancelBtn    [shop_modal_cancel_btn: Button, Secondary]   — "ui.shop.confirm.no"
```

### 11.3 NotificationOverlay 内部结构（交易结果）

```
NotificationOverlay (TransactionResult)
└── ToastPanel      [shop_toast: Container — 浮动 Toast, z=400]
    ├── ToastIcon   [shop_toast_icon: Image]                     — 成功/失败图标
    ├── ToastTitle  [shop_toast_title: HeadingText]              — "Purchase Successful" / "Transaction Failed"
    └── ToastDesc   [shop_toast_desc: CaptionText]               — "You acquired 2 items for 350G."
```

### 11.4 Overlay 生命周期

| Overlay | OnOpen | OnClose | 依赖 |
|---------|--------|---------|------|
| ModalOverlay (PurchaseConfirm) | 创建 Modal 实体：半透明遮罩 (z=303) + 确认弹窗 (z=304) + 购物车物品清单 + 总价 + 确认/取消按钮，焦点锁定在弹窗内 | 确认: 触发 `UiCommand::ConfirmPurchase`（携带 cart_items 数据），销毁 Modal 实体；取消: 销毁 Modal 实体，焦点回到 `shop_buy_btn` | 无 |
| NotificationOverlay (TransactionResult) | 创建 Toast 实体：浮动面板 (z=400) + 图标 + 标题 + 描述文本，3s 后自动关闭 | 3s 自动关闭，或点击 Toast 手动关闭 | 无 |

---

## 12. Lifecycle

> Screen 的完整生命周期行为。遵守 `screen-lifecycle.md` 定义的状态机。

| 阶段 | 行为 | 触发条件 | 清理 |
|------|------|---------|------|
| **OnEnter** | `spawn_shop_screen()` — 生成 Popup UI 树（shop_screen_root → backdrop + popup → Header + PlayerGold + TabPanel + Content + CartPanel），从 ShopVm/InventoryVm 读取数据初始化各 Widget 状态 | `OverlayState::Shop` 遮罩弹出 | — |
| **OnReady** | 注册 EconomyProjection Observer（监听金币变更、交易结果事件）；初始化 ShopVm 商品数据加载 | OnEnter 完成，UI 树就绪 | — |
| **Active** | 等待 Tab 切换、添加/移除购物车、确认购买/出售等交互事件，通过 UiCommand 下发至 Economy Domain | OnReady 完成 | — |
| **OnExit** | `despawn_shop_screen()` — 清理所有标记了 `With<ShopScreen>` 的实体，注销 EconomyProjection Observer | `OverlayState::Shop` 状态退出 | 清理标记: `With<ShopScreen>` |

### 12.1 生命周期事件处理

```yaml
on_enter:
  - action: "spawn_ui_tree"
    spawner: "spawn_shop_screen()"
    description: "生成 ShopScreen 完整 Popup UI 树：Backdrop + Popup(Header + PlayerGold + TabPanel + Content(BuyList/SellList) + CartPanel)"
  - action: "read_shop_data"
    source: "ShopVm"
    description: "从 ShopVm 读取商店名称、商品列表、玩家金币、出售价格映射"
  - action: "read_inventory_data"
    source: "InventoryVm"
    description: "从 InventoryVm 读取玩家背包物品列表，用于 SellTab 展示"
  - action: "set_default_tab"
    target: "shop_tab_buy"
    description: "默认选中 BuyTab，BuyList 可见，SellList 隐藏"

on_ready:
  - action: "register_observer"
    target: "EconomyProjection::GoldChanged"
    handler: "当玩家金币发生变更时，刷新 shop_player_gold 显示"
  - action: "register_observer"
    target: "EconomyProjection::TransactionCompleted"
    handler: "当交易完成时，显示 NotificationOverlay（交易结果），刷新 ShopVm（更新库存）和 InventoryVm（更新背包）"
  - action: "register_observer"
    target: "EconomyProjection::TransactionFailed"
    handler: "当交易失败时，显示 NotificationOverlay（错误信息），恢复 CartPanel 状态"

active:
  - trigger: "shop_tab_buy.selected"
    action: "设置 selected_tab = Buy, 隐藏 SellList 显示 BuyList"
    scope: "shop_content"
  - trigger: "shop_tab_sell.selected"
    action: "设置 selected_tab = Sell, 隐藏 BuyList 显示 SellList"
    scope: "shop_content"
  - trigger: "shop_item_card.add_to_cart"
    action: "将商品添加到 ShopScreenState.cart_items（更新数量和总价），刷新 CartPanel"
    scope: "shop_cart_panel"
  - trigger: "shop_item_row.add_to_cart"
    action: "将物品添加到 ShopScreenState.cart_items（Sell 模式），刷新 CartPanel"
    scope: "shop_cart_panel"
  - trigger: "shop_item_card.remove_from_cart"
    action: "从 ShopScreenState.cart_items 中移除商品，刷新 CartPanel"
    scope: "shop_cart_panel"
  - trigger: "shop_buy_btn.clicked"
    action: "弹出 ModalOverlay（购买确认弹窗），展示购物车物品清单和总价，阻止下层交互"
    scope: "shop_cart_panel → ModalOverlay"
  - trigger: "ModalOverlay.confirm (PurchaseConfirm)"
    action: "构建 UiCommand::ConfirmPurchase { tab: Buy/Sell, items } 下发至 Economy Domain。Buy: 扣除金币 + 获得物品；Sell: 获得金币 + 移除物品。销毁 Modal"
    scope: "ModalOverlay → Domain"
  - trigger: "ModalOverlay.cancel (PurchaseConfirm)"
    action: "销毁 ModalOverlay，焦点回到 shop_buy_btn"
    scope: "ModalOverlay"
  - trigger: "TransactionCompleted"
    action: "通过 Observer 刷新 ShopVm（更新商品库存）和 InventoryVm（更新背包物品）；显示 NotificationOverlay（交易成功 Toast）；清空购物车，重置 CartPanel 为空状态；刷新 shop_player_gold"
    scope: "全局"
  - trigger: "TransactionFailed"
    action: "显示 NotificationOverlay（交易失败 Toast + 原因）；保留购物车状态供用户重试"
    scope: "全局"
  - trigger: "shop_cancel_btn.clicked"
    action: "触发 UiCommand::CloseOverlay, OverlayStack.Pop"
    scope: "shop_cart_panel"
  - trigger: "shop_header_close.clicked"
    action: "触发 UiCommand::CloseOverlay, OverlayStack.Pop"
    scope: "shop_header"
  - trigger: "shop_retry_btn.clicked"
    action: "重新加载 ShopVm/InventoryVm 数据，Content 区域进入 Loading 状态"
    scope: "shop_content"
  - trigger: "EconomyProjection::GoldChanged"
    action: "刷新 shop_player_gold 显示数值，同时更新所有 shop_item_card 的 is_affordable 状态（金币不足的价格变红）"
    scope: "shop_player_gold, shop_content"

on_exit:
  - action: "unregister_observer"
    target: "EconomyProjection::GoldChanged"
  - action: "unregister_observer"
    target: "EconomyProjection::TransactionCompleted"
  - action: "unregister_observer"
    target: "EconomyProjection::TransactionFailed"
  - action: "despawn_ui_tree"
    query: "With<ShopScreen>"
    description: "清理所有标记了 ShopScreen 组件的实体（包括 Backdrop、Popup 及其所有子 region）"
```

---

## 13. Data Ownership

> Owns / Uses 分离。ShopScreen 拥有本地 UI 状态（选中的 Tab、购物车物品），消费 ShopVm 和 InventoryVm（来自 Domain Projection）。

### 13.1 ViewModel 映射

| ViewModel | 字段 | 归属 (Owns/Uses) | 更新频率 | Projection 源 |
|-----------|------|-----------------|---------|--------------|
| `ShopScreenState::selected_tab` | `TabType` | Owns — Screen 独享的本地 UI 状态 | Tab 点击事件 | 无（仅 UI 内部使用） |
| `ShopScreenState::cart_items` | `HashMap<ItemId, u32>` | Owns — Screen 独享的本地 UI 状态 | 物品添加/移除事件 | 无（仅 UI 内部使用） |
| `ShopScreenState::cart_total` | `u32` | Owns — 派生自 cart_items 和单价 | 物品添加/移除事件 | 无（计算属性） |
| `ShopVm::shop_name` | `LocalizedText` | Uses — ShopScreen 独享消费 | OnEnter 加载 | `EconomyProjection::on_shop_loaded` |
| `ShopVm::items` | `Vec<ShopItemInfo>` | Uses — ShopScreen 独享消费 | OnEnter 加载 / 交易完成后刷新 | `EconomyProjection::on_shop_loaded` / `on_transaction_completed` |
| `ShopVm::sell_prices` | `HashMap<ItemId, u32>` | Uses — ShopScreen 独享消费 | OnEnter 加载 | `EconomyProjection::on_shop_loaded` |
| `ShopVm::player_gold` | `u32` | Uses — 多个 Screen 共享 | 交易完成后 / 金币变更事件 | `EconomyProjection::on_gold_changed` |
| `InventoryVm::items` | `Vec<ItemInfo>` | Uses — 多个 Screen 共享 | 交易完成后刷新 | `InventoryProjection::on_inventory_changed` |

### 13.2 数据流

```
Domain Event (TradeRequested/GoldChanged/TransactionCompleted)
    ↓
Projection (EconomyProjection / InventoryProjection)
    ↓
ViewModel (ShopVm / InventoryVm) 更新 → mark_dirty()
    ↓
ShopScreen 消费 ViewModel → Widget 数据绑定刷新
    ↓
用户交互 → Widget 输出事件 → UiCommand
    ↓
UiLayer 处理 → Domain (Economy Domain: BuyItem/SellItem)
    ↓
Domain Event → 回到顶部 (循环)
```

### 13.3 本地 UI 状态

| 字段 | 类型 | 初始值 | 修改时机 | 影响区域 |
|------|------|--------|---------|---------|
| `selected_tab` | `TabType` | `TabType::Buy` | 点击 shop_tab_buy/shop_tab_sell | shop_tab_panel (高亮), shop_content (显示哪个列表) |
| `cart_items` | `HashMap<ItemId, u32>` | 空 | 点击 shop_item_card.add_to_cart / shop_item_card.remove_from_cart | shop_cart_summary, shop_buy_btn (enabled) |

---

## 14. Layout Intent

> 每个关键尺寸的**理由说明**。为什么选这个尺寸而不是别的？

### 14.1 固定尺寸意图

| widget_id | 属性 | 值 | 意图 | shrink |
|-----------|------|----|------|--------|
| `shop_popup` | width | 600px | "商店弹窗宽度 600px，比标准 Modal(480px) 更宽以容纳商品卡片水平布局，比全屏(1280px+) 更紧凑。足够展示商品名称+属性+价格按钮在一行" | none |
| `shop_popup` | height | 520px | "弹窗高度 520px，容纳 Header(48) + TabPanel(40) + 物品列表(~340) + CartPanel(56) + 内边距，典型场景显示 4-6 个商品" | none |
| `shop_header` | height | 48px | "Header 栏高度 48px，略大于按钮高度 40px，提供充足上下内边距(4px)，与 InventoryScreen Header 保持一致" | none |
| `shop_header_close` | width | 40px | "关闭按钮最小触摸目标 40x40" | none |
| `shop_header_close` | height | 40px | "最小触摸目标 40px" | none |
| `shop_tab_panel` | height | 40px | "Tab 栏高度 40px，刚好容纳 TabButton（最小触摸目标高度）" | none |
| `shop_item_card` | height | 72px | "商品卡片 72px 高度，容纳 48x48 图标 + 名称(1行) + 属性(1行) + 上下内边距(各 4px)，紧凑展示商品信息" | none |
| `shop_item_row` | height | 56px | "背包物品行 56px，容纳 40x40 图标 + 名称 + 数量 + 出售按钮(40px)在一行" | none |
| `shop_cart_panel` | height | 56px | "CartPanel 高度 56px，容纳 40px 按钮 + 上下内边距(各 8px)，与 Header 等高视觉对称" | none |
| `shop_buy_btn` | width | 120px | "购买按钮 120px 容纳 'Purchase'/'Confirm Sell' 英文文本，最小触摸目标 40px" | none |
| `shop_buy_btn` | height | 40px | "最小触摸目标 40px" | none |
| `shop_cancel_btn` | width | 100px | "取消按钮 100px 容纳 'Cancel'/'取消'，略小于 Primary 按钮传达次级优先级" | none |
| `shop_cancel_btn` | height | 40px | "最小触摸目标 40px" | none |
| `shop_retry_btn` | width | 120px | "重试按钮 120px 足够容纳 'Retry'/'重试'" | none |
| `shop_retry_btn` | height | 40px | "最小触摸目标 40px" | none |

### 14.2 弹性尺寸意图

| widget_id | flex_grow | 理由 |
|-----------|-----------|------|
| `shop_header_name` | 1 | "标题弹性增长，将关闭按钮推至右侧（Space-between 等效行为）" |
| `shop_content` | 1 | "内容区域弹性填充 Header+TabPanel+CartPanel 之外的所有垂直空间，商品列表越大越好" |
| `shop_tab_buy` | 1 | "BuyTab 借助 flex_grow 与 SellTab 平分 TabPanel 宽度" |
| `shop_tab_sell` | 1 | "与 BuyTab 对称平分布局" |
| `shop_buy_list` | 1 | "BuyList 弹性填充 Content 区域的垂直空间" |
| `shop_sell_list` | 1 | "SellList 弹性填充 Content 区域的垂直空间" |
| `shop_cart_summary` | 1 | "摘要文本弹性增长，将按钮组推至右侧" |

### 14.3 通用约束

```yaml
global:
  min_interactive_height: 40px   # 可交互元素最小高度 (触摸友好)
  min_interactive_width: 40px    # 可交互元素最小宽度 (触摸友好)
  standard_padding: 8px          # 标准内边距
  standard_gap: 8px              # 标准间距 (Flexbox gap)
  popup_border_radius: 8px       # 弹窗圆角
```

---

## 15. Scroll & Overflow Policy

> ShopScreen 的 Content 区域物品列表可能超出可见高度需要滚动。

### 15.1 滚动区域

| widget_id | 方向 | Scroll Policy | Overflow Policy | 理由 |
|-----------|------|--------------|---------------|------|
| `shop_buy_list` | vertical | auto | clip | "商品列表可能超过 10 个，弹窗固定高度时需垂直滚动查看" |
| `shop_sell_list` | vertical | auto | clip | "背包物品列表可能较多，SellTab 也需垂直滚动" |
| `shop_header` | horizontal | none | clip | "Header 内容固定，不会溢出" |
| `shop_tab_panel` | horizontal | none | clip | "2 个 TabButton 在 600px 宽度下足够，不会溢出" |
| `shop_cart_panel` | horizontal | none | clip | "摘要文本 + 2 个按钮在 600px 宽度下足够，不会溢出" |

### 15.2 文本溢出

| widget_id | max_lines | overflow | 多语言风险 |
|-----------|-----------|----------|-----------|
| `shop_header_name` | 1 | ellipsis | "商店名称通常不超过 20 字符，风险低" |
| `shop_player_gold` | 1 | ellipsis | "'Gold: 1500G' 格式固定，风险低" |
| `shop_tab_buy` | 1 | ellipsis | "'Buy'/'购买' 极短，风险低" |
| `shop_tab_sell` | 1 | ellipsis | "'Sell'/'出售' 极短，风险低" |
| `shop_item_card` | 1 (name) | ellipsis | "物品名称在卡片内显示 1 行，英文名称可能过长，使用 ellipsis" |
| `shop_item_card` | 1 (description) | ellipsis | "物品属性简述在卡片内显示 1 行，使用 ellipsis" |
| `shop_item_row` | 1 (name) | ellipsis | "背包物品行名称限 1 行，使用 ellipsis" |
| `shop_cart_summary` | 1 | ellipsis | "'Selected: 2 items Total: 350G' 格式固定，风险低" |
| `shop_buy_btn` | 1 | ellipsis | "'Purchase'/'Confirm Sell' 英文较长但 120px+40px 足够，风险低" |
| `shop_error_text` | 3 | ellipsis | "错误消息中英文长度不定，3 行截断处理" |

---

## 16. Event Contract

> UI -> Domain 事件 + Domain -> UI 事件的完整契约。

### 16.1 UI -> Domain（通过 UiCommand 传递）

```yaml
SwitchShopTab:
  trigger_widget: "shop_tab_panel → shop_tab_buy/shop_tab_sell → click"
  data:
    tab: TabType  # Buy | Sell
  conditions:
    - tab != selected_tab  # 仅当切换 Tab 时触发
  emits: UiCommand::SwitchShopTab(TabType)
  domain_event: "None — 纯 UI 本地状态变更"

AddToCart:
  trigger_widget: "shop_content → shop_buy_list → shop_item_card → click"
                  "shop_content → shop_sell_list → shop_item_row → click"
  data:
    item_id: ItemId
    quantity: u32  # 默认为 1
  conditions:
    - (Buy mode) item.stock > 0
    - (Buy mode) item.price * quantity <= player_gold
  emits: UiCommand::AddToCart(ItemId, u32)
  domain_event: "None — 纯 UI 本地状态变更"

RemoveFromCart:
  trigger_widget: "shop_content → shop_buy_list → shop_item_card → click (已加入购物车的商品再点移除)"
  data:
    item_id: ItemId
  conditions:
    - cart_items.contains(item_id)
  emits: UiCommand::RemoveFromCart(ItemId)
  domain_event: "None — 纯 UI 本地状态变更"

ConfirmPurchase:
  trigger_widget: "shop_cart_panel → shop_buy_btn → click → ModalOverlay.confirm"
  data:
    tab: TabType
    items: Vec<(ItemId, u32)>  # (item_id, quantity) pairs
  conditions:
    - cart_items.len() > 0
    - (Buy mode) total_price <= player_gold  # 最终检查
    - (Buy mode) all items have sufficient stock
  emits: UiCommand::ConfirmPurchase(TabType, Vec<(ItemId, u32)>)
  domain_event: "TransactionRequested { type: Buy | Sell, items }"

CloseShop:
  trigger_widget: "shop_header → shop_header_close → click"
                  "shop_cart_panel → shop_cancel_btn → click"
  data: {}
  conditions: []
  emits: UiCommand::CloseOverlay
  domain_event: "None — 纯 UI 导航，OverlayStack 直接处理"

ReloadShopData:
  trigger_widget: "shop_content → shop_error_state → shop_retry_btn → click"
  data: {}
  conditions: []
  emits: UiCommand::ReloadShopData
  domain_event: "None — 触发 ShopVm/InventoryVm 重新加载"
```

### 16.2 Domain -> UI（通过 Projection 消费）

```yaml
GoldChanged:
  source: "Domain Event (Economy Domain)"
  projection: "EconomyProjection.on_gold_changed()"
  vm_update:
    - "ShopVm.player_gold ← new_gold"
  side_effect:
    - "mark_dirty::<ShopVm>()"
    - "更新 shop_player_gold 显示"
    - "重新计算所有 shop_item_card 的 is_affordable 状态（价格变红/恢复）"

TransactionCompleted:
  source: "Domain Event (Economy Domain)"
  projection: "EconomyProjection.on_transaction_completed()"
  vm_update:
    - "ShopVm.player_gold ← updated_gold"
    - "ShopVm.items ← updated (减少对应商品库存)"
    - "InventoryVm.items ← updated (购买: 新增物品 / 出售: 移除物品)"
    - "ShopScreenState.cart_items ← empty"
  side_effect:
    - "mark_dirty::<ShopVm>()"
    - "mark_dirty::<InventoryVm>()"
    - "显示 NotificationOverlay（交易成功 Toast: 获得/出售物品列表 + 金币变动）"
    - "清空购物车，CartPanel 重置为空状态"

TransactionFailed:
  source: "Domain Event (Economy Domain)"
  projection: "EconomyProjection.on_transaction_failed()"
  vm_update:
    - "ShopScreenState.cart_items ← unchanged (保留购物车状态)"
  side_effect:
    - "显示 NotificationOverlay（交易失败 Toast: 失败原因，如'金币不足'/'库存不足'）"
    - "CartPanel 显示错误提示"

ShopDataLoaded:
  source: "Domain Event (Economy Domain)"
  projection: "EconomyProjection.on_shop_loaded()"
  vm_update:
    - "ShopVm.shop_name ← loaded_name"
    - "ShopVm.items ← loaded_items"
    - "ShopVm.sell_prices ← loaded_sell_prices"
    - "ShopVm.player_gold ← current_gold"
  side_effect:
    - "mark_dirty::<ShopVm>()"
    - "shop_content 从 Loading → Normal 状态转换"

ShopDataLoadFailed:
  source: "Domain Event (Economy Domain)"
  projection: "EconomyProjection.on_shop_load_failed()"
  vm_update:
    - "ShopVm.error_message ← error message"
  side_effect:
    - "mark_dirty::<ShopVm>()"
    - "shop_content 从 Loading → Error 状态转换"
```

---

## 17. Screen Metrics

> 复杂度基线。所有数值初始创建时手动填写，后续 CI 阶段自动校验。

| Metric | 值 | P0/P1 | 说明 |
|--------|-----|-------|------|
| `widget_count` | 22 | P1 | Widget 实例总数（root + backdrop + popup + header + name + close + gold + tab_panel + tab_buy + tab_sell + content + buy_list + item_card + sell_list + item_row + empty_buy + empty_sell + loading + error_state + error_text + retry_btn + cart_panel + summary + buy_btn + cancel_btn） |
| `container_count` | 8 | P1 | 纯容器节点数（root + popup + header + tab_panel + content + buy_list + sell_list + error_state + cart_panel） |
| `interactive_count` | 6 | P1 | 可交互 Widget 数（close_btn + tab_buy + tab_sell + item_card + item_row + buy_btn + cancel_btn + retry_btn） |
| `overlay_count` | 2 | P1 | 关联的 Overlay 数（ModalOverlay 购买确认 + NotificationOverlay 交易结果） |
| `max_depth` | 5 | P1 | root → shop_popup → shop_content → shop_buy_list → shop_item_card（5 层） |
| `max_children` | 5 | P1 | shop_popup 有 5 个直接子节点（header, gold, tab_panel, content, cart_panel） |

### 17.1 Budget 检查

| 规则 | 阈值 | 当前值 | 状态 |
|------|------|--------|------|
| max_depth ≤ 6 | 6 | 5 | ✅ |
| max_children ≤ 20 | 20 | 5 | ✅ |
| interactive_count / widget_count ≥ 0.2 | 20% | 36% (8/22) | ✅ |

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
| D09 | State Mapping 完整 (每个 region 的 Loading/Empty/Normal/Error) | [x] | CartPanel 有额外 PostPurchase 状态 |
| D10 | Focus Navigation 已定义 (Tab 路径完整) | [x] | P1 |
| D11 | Interaction Zones 已定义 (Click/Hover) | [x] | |
| D12 | Overlay Definition 已定义 (Overlay 列表 + Z-Layer) | [x] | ModalOverlay + NotificationOverlay |
| D13 | Lifecycle 已定义 (OnEnter/OnReady/Active/OnExit) | [x] | |
| D14 | Data Ownership 已定义 (Owns/Uses) | [x] | |
| D15 | Layout Intent 已定义 (关键尺寸理由) | [x] | P1 |
| D16 | Scroll & Overflow Policy 已定义 | [x] | P1 |
| D17 | Event Contract 已定义 (UI->Domain + Domain->UI) | [x] | P1 |
| D18 | Screen Metrics 已定义 | [x] | P1 |

**P0 字段全部通过日期**: {YYYY-MM-DD}
**status 改为 active 日期**: {YYYY-MM-DD}

---

## 附录 B: 引用文档

| 文档 | 用途 |
|------|------|
| `07-specs/README.md` | SSPEC 总纲、AI 14 条规则、DoD 18 项清单 |
| `07-specs/references/widget-id-map.md` | Widget ID -> UiBinding 映射总表 |
| `07-specs/references/z-layer-spec.md` | Z-Layer 统一规范 |
| `07-specs/references/layout-intent-library.md` | 跨 Screen 共享的 Layout Intent |
| `06-ui/02-design-system/widget-atoms.md` | 原子组件 Contract（HeadingText, IconButton, TabButton, Button, BodyText, CaptionText, StatText, Spinner, EmptyWidget, Image, Panel） |
| `06-ui/02-design-system/widget-composites.md` | 复合组件 Contract（ShopItemCard, InventoryItemRow） |
| `06-ui/02-design-system/theme-localization.md` | StyleToken / Theme / UiTextKey |
| `06-ui/02-design-system/focus-binding.md` | Focusable / FocusGroup / Dirty<T> / UiBinding |
| `06-ui/03-screens/screens.md` | ShopScreen 设计规格（§5） |
| `06-ui/03-screens/screen-lifecycle.md` | Screen 生命周期状态机 |
| `06-ui/04-data-flow/projection-viewmodel.md` | Projection / ViewModel 映射 |
| `03-content/localization/ui-screen-keys.md` | 商店界面 LocalizationKeys |

---

## 附录 C: Z-Layer 对齐说明

ShopScreen 自身是 PopupOverlay，位于 `popup_layer` (z=300)。其 Z-Layer 分配如下：

| 实体 | Z 值 | 层 |
|------|------|----|
| shop_backdrop (遮罩) | 300 | popup_layer 底层 |
| shop_popup (弹窗主体) | 302 | popup_layer (遵循 z-layer-spec.md §2.3.4 ShopScreen Popup) |
| ModalOverlay (购买确认) | 303-304 | popup_layer 子层 (嵌套弹窗) |
| NotificationOverlay (交易结果 Toast) | 400 | notification_layer |

此分配与 `07-specs/references/z-layer-spec.md` 保持一致：ShopScreen Popup 位于 z=302（如 §2.3.4 定义），嵌套的 ModalOverlay 购买确认使用 z=303（遮罩）+ z=304（弹窗），位于 popup_layer 子层预留范围（z=303-399）。NotificationOverlay 使用独立的 notification_layer（z=400）。

---

*本文档是 ShopScreen SSPEC，由 @feature-developer 根据 `07-specs/screen-spec-template.md` 模板创建。所有 17 个字段已填充。当前 status: draft。*
