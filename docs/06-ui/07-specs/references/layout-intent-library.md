---
id: 07-specs.layout-intent-library
title: Layout Intent Library — Cross-Screen Unified Reference
status: active
owner: presentation-architect
created: 2026-06-22
tags:
  - ui
  - layout-intent
  - reference
  - active
---

# Layout Intent Library

> **职责**: @presentation-architect | **上游**: 各 SSPEC §14 Layout Intent, `07-specs/README.md` §5 (Flexbox Layout 规则)

> 跨 Screen 共享的布局约束理由汇总。所有 SSPEC §14 中的通用约束提取至此，各 SSPEC 引用此库而非重复定义。

---

## 1. 固定高度 Header / Top Bar

| widget_id | screen | height | intent | shrink | reference |
|-----------|--------|--------|--------|--------|-----------|
| `inventory_header` | inventory_screen | 56px | "Header 栏高度 56px，略大于按钮高度 40px，提供充足上下内边距（8px），与 SettingsScreen Header 保持一致" | none | inventory_screen.md §14.1 |
| `settings_header` | settings_screen | 56px | "Header 栏高度 56px，略大于按钮高度 40px，提供充足上下内边距（8px），视觉舒适" | none | settings_screen.md §14.1 |
| `saveload_header` | save_load_screen | 56px | "Header 栏高度 56px，略大于按钮高度 40px，提供充足上下内边距（8px），视觉舒适" | none | save_load_screen.md §14.1 |
| `shop_header` | shop_screen | 48px | "Header 栏高度 48px，略大于按钮高度 40px，提供充足上下内边距(4px)，与 InventoryScreen Header 保持一致" | none | shop_screen.md §14.1 |
| `battle_top_bar` | battle_screen | 64px | "顶栏高度 64px 为回合信息和阶段标签提供充足显示空间，同时不会过度挤压战斗地图区域" | none | battle_screen.md §14.1 |

**共享模式**: Header 高度集中在 48-56px 范围，标准为 56px（比最小触摸目标 40px 多 16px 用于上下内边距）。BattleScreen 顶栏例外（64px）因需要承载更多信息（回合号 + 阶段标签 + 结束回合按钮）。

---

## 2. 固定高度 Action Panel / Footer

| widget_id | screen | height | intent | shrink | reference |
|-----------|--------|--------|--------|--------|-----------|
| `battle_action_menu` | battle_screen | 80px | "行动菜单 80px 高度 = 按钮 48px + 上下 padding 32px，为 6 个水平排列按钮提供充足空间，同时不会过度挤压战斗区域" | none | battle_screen.md §14.1 |
| `inventory_action_bar` | inventory_screen | 56px | "ActionBar 高度 56px，与 Header 等高，视觉对称" | none | inventory_screen.md §14.1 |
| `settings_footer` | settings_screen | 56px | "Footer 高度 56px，与 Header 等高，视觉对称，包裹按钮后上下留 8px 内边距" | none | settings_screen.md §14.1 |
| `saveload_action_panel` | save_load_screen | 56px | "ActionPanel 高度 56px，与 Header 等高，视觉对称，包裹按钮后上下留 8px 内边距" | none | save_load_screen.md §14.1 |
| `shop_cart_panel` | shop_screen | 56px | "CartPanel 高度 56px，容纳 40px 按钮 + 上下内边距(各 8px)，与 Header 等高视觉对称" | none | shop_screen.md §14.1 |

**共享模式**: Action Panel / Footer 高度统一为 56px（与 Header 等高视觉对称），BattleScreen ActionMenu 例外（80px）因容纳 6 个水平按钮需更高空间。

---

## 3. 固定宽度 40px 关闭按钮

| widget_id | screen | width | height | intent | shrink | reference |
|-----------|--------|-------|--------|--------|--------|-----------|
| `inventory_header_close` | inventory_screen | 40px | 40px | "关闭按钮最小触摸目标 40x40" | none | inventory_screen.md §14.1 |
| `settings_header_close` | settings_screen | 40px | 40px | "关闭按钮最小触摸目标 40x40，图标按钮无需更宽" | none | settings_screen.md §14.1 |
| `saveload_header_close` | save_load_screen | 40px | 40px | "关闭按钮最小触摸目标 40x40" | none | save_load_screen.md §14.1 |
| `shop_header_close` | shop_screen | 40px | 40px | "关闭按钮最小触摸目标 40x40" | none | shop_screen.md §14.1 |

**共享模式**: 所有 Header 关闭按钮统一 40x40，严格遵循全局最小触摸目标约束。

---

## 4. 固定宽度 120-140px 按钮

| widget_id | screen | width | height | intent | shrink | reference |
|-----------|--------|-------|--------|--------|--------|-----------|
| `battle_end_turn_btn` | battle_screen | 120px | 40px | "按钮宽度 120px 容纳 'End Turn' / '结束回合' 等本地化文本（中英文均在 12 字符以内），单行显示不折行" | none | battle_screen.md §14.1 |
| `battle_attack_btn` | battle_screen | 120px | 48px | "按钮 120px 宽度确保 'Attack' / '普通攻击' 等本地化文本单行显示，与 EndTurn 按钮同宽保证视觉节奏感" | none | battle_screen.md §14.1 |
| `inventory_use_btn` | inventory_screen | 140px | 40px | "使用按钮 140px 确保标签 'Use Item'/'使用物品' 中英文单行显示" | none | inventory_screen.md §14.1 |
| `inventory_drop_btn` | inventory_screen | 140px | 40px | "与 UseButton 同宽，保持视觉对称" | none | inventory_screen.md §14.1 |
| `inventory_retry_btn` | inventory_screen | 120px | 40px | "重试按钮 120px 足够容纳 'Retry'/'重试'" | none | inventory_screen.md §14.1 |
| `shop_buy_btn` | shop_screen | 120px | 40px | "购买按钮 120px 容纳 'Purchase'/'Confirm Sell' 英文文本，最小触摸目标 40px" | none | shop_screen.md §14.1 |
| `shop_cancel_btn` | shop_screen | 100px | 40px | "取消按钮 100px 容纳 'Cancel'/'取消'，略小于 Primary 按钮传达次级优先级" | none | shop_screen.md §14.1 |
| `shop_retry_btn` | shop_screen | 120px | 40px | "重试按钮 120px 足够容纳 'Retry'/'重试'" | none | shop_screen.md §14.1 |
| `saveload_confirm_btn` | save_load_screen | 160px | 40px | "确认按钮宽度 160px，确保 'Save' / 'Load' / 中文 '保存' / '加载' 均单行显示" | none | save_load_screen.md §14.1 |
| `saveload_delete_btn` | save_load_screen | 120px | 40px | "删除按钮宽度 120px，标签 'Delete' / '删除' 单行显示即可" | none | save_load_screen.md §14.1 |

**共享模式**: 主要操作按钮 120-140px 宽度，次级操作按钮 100-120px，高度统一为 40px（最小触摸目标）。Battle Attack 按钮 48px 突出主行动按钮地位。

---

## 5. 固定宽度 320px 区域

| widget_id | screen | width | intent | shrink | reference |
|-----------|--------|-------|--------|--------|-----------|
| `inventory_character_list` | inventory_screen | 320px | "角色列表固定 320px，与 BattleScreen char_panel 宽度一致，容纳头像(40x40) + 名称(最长 10 字) + 等级(4 字符) 在一行内" | none | inventory_screen.md §14.1 |

---

## 6. 固定宽度特殊区域

| widget_id | screen | width | height | intent | shrink | reference |
|-----------|--------|-------|--------|--------|--------|-----------|
| `battle_minimap` | battle_screen | 160px | 120px | "小地图宽度 160px 在 1080p 分辨率下占战斗区域约 12%，足够显示战场概览，不会过度遮挡地图" | none | battle_screen.md §14.1 |
| `battle_char_avatar` | battle_screen | 100px | 100px | "头像 100x100 在 1080p 下约为视口高度的 9%，大小足以展示角色特征，不会过度占据 CharPanel" | none | battle_screen.md §14.1 |
| `battle_hp_bar` | battle_screen | 200px | 16px | "HP 条 200px 宽度确保在 1080p 下精细显示血量变化（每像素约 0.5%），同时保持与面板宽度的比例协调" | low | battle_screen.md §14.1 |
| `settings_tablist` | settings_screen | 200px | — | "Tab 列表宽度 200px，Tab 标签 'Gameplay/Graphics/Audio/Battle' 中英文均能在单行内显示，无需折行" | none | settings_screen.md §14.1 |
| `saveload_slot_list` | save_load_screen | 400px | — | "槽位列表 400px 宽度，足以显示 'Aria - Lv.5' / 'River Crossing' / '12h 34m' 等中文摘要信息单行显示，无需折行" | none | save_load_screen.md §14.1 |
| `shop_popup` | shop_screen | 600px | 520px | "商店弹窗宽度 600px，比标准 Modal(480px) 更宽以容纳商品卡片水平布局，比全屏(1280px+) 更紧凑。足够展示商品名称+属性+价格按钮在一行" | none | shop_screen.md §14.1 |

---

## 7. FlexGrow 弹性区域

| widget_id | screen | flex_grow | intent | reference |
|-----------|--------|-----------|--------|-----------|
| `battle_battle_area` | battle_screen | 1 | "战斗地图区域是 BattleScreen 的核心，弹性填充所有剩余空间——地图越大，玩家对战场的感知越清晰" | battle_screen.md §14.2 |
| `inventory_body` | inventory_screen | 1 | "Body 占据 Header/DescPanel/ActionBar 之外的所有垂直空间，内容区越大越好" | inventory_screen.md §14.2 |
| `inventory_item_grid` | inventory_screen | 1 | "物品网格弹性填充 FilterBar 下方的剩余空间，显示尽可能多的物品" | inventory_screen.md §14.2 |
| `inventory_area` | inventory_screen | 1 | "Inventory 主区域弹性填充 CharacterList 右侧的剩余空间" | inventory_screen.md §14.2 |
| `inventory_character_rows` | inventory_screen | 1 | "角色行列表弹性填充 CharacterList 中标签下方的空间，容纳尽可能多的角色" | inventory_screen.md §14.2 |
| `settings_tabpanel` | settings_screen | 1 | "TabPanel 占据 Header 和 Footer 之外的所有垂直空间，内容区越大越好" | settings_screen.md §14.2 |
| `settings_tabcontent` | settings_screen | 1 | "内容区弹性填充 TabList 右侧的剩余水平空间，越宽越便于显示长文本设置项" | settings_screen.md §14.2 |
| `saveload_body` | save_load_screen | 1 | "主体区域占据 Header 和 ActionPanel 之外的所有垂直空间，留给内容区越多越好" | save_load_screen.md §14.2 |
| `saveload_preview_panel` | save_load_screen | 1 | "预览面板弹性填充 SlotList (400px) 右侧的剩余水平空间，越宽越便于显示详情文本和截图" | save_load_screen.md §14.2 |
| `shop_content` | shop_screen | 1 | "内容区域弹性填充 Header+TabPanel+CartPanel 之外的所有垂直空间，商品列表越大越好" | shop_screen.md §14.2 |
| `shop_header_name` | shop_screen | 1 | "标题弹性增长，将关闭按钮推至右侧（Space-between 等效行为）" | shop_screen.md §14.2 |

**共享模式**: 核心内容区域（BattleArea / Body / TabPanel / Content）均使用 `flex_grow: 1` 优先填充剩余空间。标题文本通过 `flex_grow: 1` 将关闭/操作按钮推至右侧（Space-between 等效）。

---

## 8. 全局约束

所有 Screen 共享以下全局约束（定义于各 SSPEC §14.3）：

```yaml
global:
  min_interactive_height: 40px   # 可交互元素最小高度 (触摸友好)
  min_interactive_width: 40px    # 可交互元素最小宽度 (触摸友好)
  standard_padding: 8px          # 标准内边距
  standard_gap:
    default: 8px                 # InventoryScreen, SettingsScreen, ShopScreen, SaveLoadScreen
    compact: 4px                 # BattleScreen, MainMenuScreen
  popup_border_radius: 8px       # ShopScreen 弹窗圆角
  item_grid_cell_gap: 8px        # InventoryScreen 物品网格格间距
  character_list_gap: 4px        # InventoryScreen 角色行间距
  slot_list_gap: 4px             # SaveLoadScreen 槽位列表项间距
  tablist_content_gap: 0px       # SettingsScreen TabList 与 TabContent 之间无间距
```

### 8.1 gap 模式对比

| 模式 | gap 值 | 使用 Screen | 理由 |
|------|--------|-------------|------|
| compact | 4px | BattleScreen, MainMenuScreen | 有限空间内最大化内容面积 |
| default | 8px | InventoryScreen, SettingsScreen, ShopScreen, SaveLoadScreen | 操作型界面需要更清晰的视觉分组 |

### 8.2 约束理由

| 约束 | 值 | 理由 |
|------|-----|------|
| min_interactive_height | 40px | 最小触摸友好高度，适配手指点击（参考 iOS HIG / Material Design 最小触摸目标 44px，取整 40px 作为最小值） |
| min_interactive_width | 40px | 与高度同宽，保持正方形最小触摸区域 |
| standard_padding | 8px | 标准内边距，在紧凑与透气之间取得平衡 |

---

## 9. 跨 Screen 尺寸对称性

| 对称组 | 相关 Screen | 尺寸 | 理由 | 引用 |
|--------|-------------|------|------|------|
| Header 高度 | InventoryScreen / SettingsScreen / SaveLoadScreen | 56px | 三屏 Header 高度一致，保持视觉统一 | 各 SSPEC §14.1 |
| ActionBar 高度 | InventoryScreen / SettingsScreen / SaveLoadScreen / ShopScreen | 56px | 四屏 ActionPanel/Footer 高度一致，与 Header 等高 | 各 SSPEC §14.1 |
| 关闭按钮 40x40 | InventoryScreen / SettingsScreen / ShopScreen / SaveLoadScreen | 40x40 | 四屏关闭按钮统一最小触摸目标 | 各 SSPEC §14.1 |
| 角色列表/面板 320px | InventoryScreen / BattleScreen | ~320px | InventoryScreen character_list 宽度引用 BattleScreen char_panel 宽度 | inventory_screen.md §14.1 |

---

## 附录 A: 变更记录

| 日期 | 变更 | 原因 |
|------|------|------|
| 2026-06-22 | 初始创建 | 从 6 个 SSPEC §14 提取共享布局约束，建立统一参考库 |

---

*本文档由 @presentation-architect 维护。新增跨 Screen 布局约束时同步更新本库。*
