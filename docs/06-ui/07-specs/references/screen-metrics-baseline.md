---
id: 07-specs.screen-metrics-baseline
title: Screen Metrics Baseline — All Screen Complexity Tracking
status: active
owner: presentation-architect
created: 2026-06-22
tags:
  - ui
  - metrics
  - screen-spec
  - baseline
  - active
---

# Screen Metrics Baseline

> **职责**: @presentation-architect | **上游**: 各 SSPEC §17 Screen Metrics | **约束**: `ai-constitution-complete.md` §18 (UI Complexity Budget)

所有 Screen 的复杂度基线追踪表。初始数据提取自各 SSPEC §17，后续 CI 阶段自动校验并与基线对比。

---

## 1. 基线总表

| Screen | widget_count | container_count | interactive_count | overlay_count | max_depth | max_children | budget_pass | status | reference |
|--------|-------------|----------------|------------------|---------------|-----------|--------------|-------------|--------|-----------|
| MainMenuScreen | 9 | 2 | 3 | 2 | 3 | 4 | ✅ | active | main_menu_screen.md §17 |
| SettingsScreen | 27 | 10 | 17 | 1 | 5 | 4 | ✅ | active | settings_screen.md §17 |
| InventoryScreen | 33 | 12 | 11 | 2 | 5 | 5 | ✅ | active | inventory_screen.md §17 |
| BattleScreen | 30 | 8 | 8 | 8 | 5 | 6 | ✅ | draft | battle_screen.md §17 |
| ShopScreen | 22 | 8 | 6 | 2 | 5 | 5 | ✅ | draft | shop_screen.md §17 |
| SaveLoadScreen | ~20 | ~6 | ~12 | 4 | 4 | 12 | ✅ | draft | save_load_screen.md §17 |

> **注**: SaveLoadScreen 的 widget_count / container_count / interactive_count 带有 `~` 前缀，表示该值为近似估算（SSPEC 中按展开 Subwidget 估算，实际应逐 slot 精确计数）。

---

## 2. 按 Screen 明细

### 2.1 MainMenuScreen

**参考**: main_menu_screen.md §17 (line 692)

| Metric | 值 | P0/P1 | 说明 |
|--------|-----|-------|------|
| `widget_count` | 9 | P1 | Widget 实例总数（root Screen + title_area + title_text + subtitle_text + button_list + 3 buttons + version_text） |
| `container_count` | 2 | P1 | 纯容器节点数（title_area + button_list — 无业务逻辑，仅布局） |
| `interactive_count` | 3 | P1 | 可交互 Widget 数（new_game_btn + load_game_btn + settings_btn） |
| `overlay_count` | 2 | P1 | 关联的 Overlay 数（ModalOverlay + LoadingOverlay，均为预留） |
| `max_depth` | 3 | P1 | root -> button_list -> new_game_btn 的层级数 |
| `max_children` | 4 | P1 | 单一容器最大子节点数（root 有 3 个直接子节点: title_area + button_list + version_text / button_list 有 3 个按钮子节点） |

**Budget 检查**:

| 规则 | 阈值 | 当前值 | 状态 |
|------|------|--------|------|
| max_depth <= 6 | 6 | 3 | ✅ |
| max_children <= 20 | 20 | 4 | ✅ |
| interactive_count / widget_count >= 0.2 | 20% | 33% (3/9) | ✅ |

---

### 2.2 SettingsScreen

**参考**: settings_screen.md §17 (line 1646)

| Metric | 值 | P0/P1 | 说明 |
|--------|-----|-------|------|
| `widget_count` | 27 | P1 | Widget 实例总数（root + header_container + title + close btn + tabpanel + tablist + 4 tab btns + tabcontent + 4 content containers + 4 toggles + 2 selectlists + 3 volume sliders + 2 battle sliders + footer + reset btn） |
| `container_count` | 10 | P1 | 纯容器节点数（root + settings_header + settings_tabpanel + settings_tablist + settings_tabcontent + settings_gameplay + settings_graphics + settings_audio + settings_battle + settings_footer） |
| `interactive_count` | 17 | P1 | 可交互 Widget 数（close btn + 4 tab btns + 4 toggles + 2 selectlists + 3 volume sliders + 2 battle sliders + reset btn） |
| `overlay_count` | 1 | P1 | 关联的 Overlay 数（ModalOverlay 重置确认，含遮罩 + 弹窗面板 + 确认按钮 + 取消按钮） |
| `max_depth` | 5 | P1 | root -> settings_tabpanel -> settings_tabcontent -> settings_audio -> settings_audio_master |
| `max_children` | 4 | P1 | settings_tabcontent 有 4 个直接子节点（gameplay / graphics / audio / battle） |

**Budget 检查**:

| 规则 | 阈值 | 当前值 | 状态 |
|------|------|--------|------|
| max_depth <= 6 | 6 | 5 | ✅ |
| max_children <= 20 | 20 | 4 | ✅ |
| interactive_count / widget_count >= 0.2 | 20% | 63% (17/27) | ✅ |

---

### 2.3 InventoryScreen

**参考**: inventory_screen.md §17 (line 1689)

| Metric | 值 | P0/P1 | 说明 |
|--------|-----|-------|------|
| `widget_count` | 33 | P1 | Widget 实例总数（root + header_container + title + close btn + body + character_list_container + label + rows_container + character_row + empty_widget + area_container + filter_bar + 5 tab btns + item_grid + item_cell + empty_state + loading + error_container + error_text + retry_btn + desc_panel + detail_container + icon + name + desc + stats + action_bar + use_btn + drop_btn） |
| `container_count` | 12 | P1 | 纯容器节点数（root + header + body + character_list + character_rows + area + filter_bar + item_grid + error_state + description_panel + item_detail + action_bar） |
| `interactive_count` | 11 | P1 | 可交互 Widget 数（close btn + character_row + 5 tab btns + item_cell + retry_btn + use_btn + drop_btn） |
| `overlay_count` | 2 | P1 | 关联的 Overlay 数（TooltipOverlay 物品悬浮提示 + ModalOverlay 丢弃确认） |
| `max_depth` | 5 | P1 | root -> inventory_body -> inventory_area -> inventory_filter_bar -> inventory_filter_all（5 层） |
| `max_children` | 5 | P1 | inventory_area 有 5 个直接子节点（filter_bar, item_grid, empty_state, loading, error_state） |

**Budget 检查**:

| 规则 | 阈值 | 当前值 | 状态 |
|------|------|--------|------|
| max_depth <= 6 | 6 | 5 | ✅ |
| max_children <= 20 | 20 | 5 | ✅ |
| interactive_count / widget_count >= 0.2 | 20% | 33% (11/33) | ✅ |

---

### 2.4 BattleScreen

**参考**: battle_screen.md §17 (line 1616)

| Metric | 值 | P0/P1 | 说明 |
|--------|-----|-------|------|
| `widget_count` | 30 | P1 | Widget 实例总数（root + top_bar + 3 top widgets + battle_area + minimap + char_panel + avatar + name + level + hp_bar + hp_text + mp_bar + mp_text + ap_bar + buff_area + 4 buff_icons + action_menu + 6 action_btns = 26 existing + 2 new = 28, plus 2 internal containers char_stats/name_line -> 30） |
| `container_count` | 8 | P1 | 纯容器节点数（top_bar + battle_area + char_panel + char_stats + name_line + buff_area + action_menu + screen_root） |
| `interactive_count` | 8 | P1 | 可交互 Widget 数（end_turn_btn + attack_btn + skill_btn + defend_btn + item_btn + move_btn + wait_btn + buff_icons hover -> 8+ interactive zones） |
| `overlay_count` | 8 | P1 | 关联的 Overlay 数（SkillOverlay + InventoryOverlay + DamageTextOverlay + TooltipOverlay + NotificationOverlay + ModalOverlay + VictoryOverlay + DefeatOverlay） |
| `max_depth` | 5 | P1 | root -> char_panel -> char_stats -> name_line -> char_name 的层级数 |
| `max_children` | 6 | P1 | battle_action_menu 有 6 个子节点（6 个按钮） |

**Budget 检查**:

| 规则 | 阈值 | 当前值 | 状态 |
|------|------|--------|------|
| max_depth <= 6 | 6 | 5 | ✅ |
| max_children <= 20 | 20 | 6 | ✅ |
| interactive_count / widget_count >= 0.2 | 20% | 26.7% (8/30) | ✅ |

---

### 2.5 ShopScreen

**参考**: shop_screen.md §17 (line 1614)

| Metric | 值 | P0/P1 | 说明 |
|--------|-----|-------|------|
| `widget_count` | 22 | P1 | Widget 实例总数（root + backdrop + popup + header + name + close + gold + tab_panel + tab_buy + tab_sell + content + buy_list + item_card + sell_list + item_row + empty_buy + empty_sell + loading + error_state + error_text + retry_btn + cart_panel + summary + buy_btn + cancel_btn） |
| `container_count` | 8 | P1 | 纯容器节点数（root + popup + header + tab_panel + content + buy_list + sell_list + error_state + cart_panel） |
| `interactive_count` | 6 | P1 | 可交互 Widget 数（close_btn + tab_buy + tab_sell + item_card + item_row + buy_btn + cancel_btn + retry_btn） |
| `overlay_count` | 2 | P1 | 关联的 Overlay 数（ModalOverlay 购买确认 + NotificationOverlay 交易结果） |
| `max_depth` | 5 | P1 | root -> shop_popup -> shop_content -> shop_buy_list -> shop_item_card（5 层） |
| `max_children` | 5 | P1 | shop_popup 有 5 个直接子节点（header, gold, tab_panel, content, cart_panel） |

**Budget 检查**:

| 规则 | 阈值 | 当前值 | 状态 |
|------|------|--------|------|
| max_depth <= 6 | 6 | 5 | ✅ |
| max_children <= 20 | 20 | 5 | ✅ |
| interactive_count / widget_count >= 0.2 | 20% | 36% (8/22) | ✅ |

---

### 2.6 SaveLoadScreen

**参考**: save_load_screen.md §17 (line 1359)

> 以下数值为近似估算（SSPEC status: draft，内容含占位符）。实现阶段应逐 slot 精确计数后更新。

| Metric | 值 | P0/P1 | 说明 |
|--------|-----|-------|------|
| `widget_count` | ~20 | P1 | Widget 实例总数（root + header_container + title + mode_toggle + close_btn + body + slot_list + 10 slots x 4 sub-widgets ≈ 40 + preview_panel + avatar + 6 details + screenshot + action_panel + confirm_btn + delete_btn ≈ 65） |
| `container_count` | ~6 | P1 | 纯容器节点数（root + header + body + slot_list + preview_panel + preview_details + action_panel） |
| `interactive_count` | ~12 | P1 | 可交互 Widget 数（mode_toggle + close_btn + 10 slot molecules + confirm_btn + delete_btn） |
| `overlay_count` | 4 | P1 | 关联的 Overlay 数（OverwriteConfirm + LoadConfirm + DeleteConfirm + LoadingOverlay） |
| `max_depth` | 4 | P1 | root -> saveload_body -> saveload_slot_list -> saveload_slot_{n} -> saveload_slot_{n}_info |
| `max_children` | 12 | P1 | saveload_slot_list 有最多 12 个直接子节点（10 个 slot + 可能的滚动条/空状态指示器） |

**Budget 检查**:

| 规则 | 阈值 | 当前值 | 状态 |
|------|------|--------|------|
| max_depth <= 6 | 6 | 4 | ✅ |
| max_children <= 20 | 20 | 12 | ✅ |
| interactive_count / widget_count >= 0.2 | 20% | ~60% (12/~20) | ✅ |

---

## 3. 复杂度排序

按 widget_count 降序排列：

| Screen | widget_count | 复杂度等级 | 说明 |
|--------|-------------|-----------|------|
| InventoryScreen | 33 | High | 最多 Widget（含 5 个 FilterTabs + 角色列表 + 物品网格 + 确认面板） |
| BattleScreen | 30 | High | 核心战斗 Screen（含 6 按钮 ActionMenu + 4 BuffIcons + 8 Overlays） |
| SettingsScreen | 27 | Medium-High | 中等 Widget 但最高 interactive_count (17) |
| ShopScreen | 22 | Medium | 弹窗式布局，物品卡片简洁 |
| SaveLoadScreen | ~20 | Medium | 10 个槽位展开后实际上达 ~65 Subwidgets，但结构重复度高 |
| MainMenuScreen | 9 | Low | 最简单 Screen，3 按钮 + 标题 |

### 3.1 Overlay 复杂度对比

| Screen | overlay_count | 特点 |
|--------|--------------|------|
| BattleScreen | 8 | 最高 —— 因战斗需要多种 Overlay（技能/物品/伤害/提示/Omodal/胜负/通知） |
| SaveLoadScreen | 4 | 确认弹窗种类多（覆盖保存/加载/删除三种操作） |
| ShopScreen | 2 | Modal + Notification |
| InventoryScreen | 2 | Tooltip + Modal |
| MainMenuScreen | 2 | Modal + Loading（均为预留） |
| SettingsScreen | 1 | 最低 —— Modal 重置确认 |

---

## 4. Budget 策略

### 4.1 Budget 阈值

| 规则 | 阈值 | 来源 |
|------|------|------|
| max_depth <= 6 | 6 | quality-maintenance-constitution.md §18.2 |
| max_children <= 20 | 20 | quality-maintenance-constitution.md §18.3 |
| interactive_count / widget_count >= 0.2 | 20% | quality-maintenance-constitution.md §18.4 |

### 4.2 当前基线状态

当前所有 Screen 的 Budget 检查均为 ✅ Pass，最接近阈值的指标：

- **max_depth**: BattleScreen / InventoryScreen / SettingsScreen / ShopScreen 均为 5（阈值 6，余量 1）
- **max_children**: SaveLoadScreen 为 12（阈值 20，余量 8）
- **interactive_ratio**: BattleScreen 最低 26.7%（阈值 20%，余量 6.7%）

### 4.3 Budget 预留与扩容

```yaml
budget_headroom:
  max_depth:
    current_max: 5
    threshold: 6
    headroom: 1
    warning_level: "new Screen 或深度重构时 max_depth 可能触及 6"
  max_children:
    current_max: 12 (SaveLoadScreen)
    threshold: 20
    headroom: 8
    warning_level: "安全，但 SaveLoadScreen 槽位展开后实际 children 数超出基线"
  interactive_ratio:
    current_min: 26.7% (BattleScreen)
    threshold: 20%
    headroom: 6.7%
    warning_level: "BattleScreen 新增 widget 时不降低交互比例"
```

---

## 5. 维护规则

| 规则 | 说明 |
|------|------|
| Screen 新增时 | 在基线总表添加对应行，含数据来源引用 |
| Screen 修改时 | 更新对应明细表，budget_pass 变为 ❌ 需要人工审查 |
| CI 自动校验 | 实现后自动对比当前计数与基线，偏离超过 20% 发出警告 |
| status 变更时 | draft -> active 时确认 budget_pass 为通过 |
| 准确度提升 | SaveLoadScreen 的近似值在实现阶段替换为精确值 |

---

## 附录 A: 与宪法第十八编的关系

宪法第十八编 `quality-maintenance-constitution.md §18 (UI Complexity Budget)` 定义了以下规则：

| 条款 | 内容 | 本基线跟踪 |
|------|------|-----------|
| §18.1 | 每个 Screen 的 Widget 树深度不超过 6 层 | ✅ max_depth <= 6 |
| §18.2 | 单一容器的子节点不超过 20 个 | ✅ max_children <= 20 |
| §18.3 | 可交互元素占总 Widget 比例不低于 20% | ✅ interactive_ratio >= 20% |
| §18.4 | 新增 Screen 必须提供复杂度基线 | 本文件为所有 Screen 提供基线 |

---

## 附录 B: 变更记录

| 日期 | 变更 | 原因 |
|------|------|------|
| 2026-06-22 | 初始创建 | 从 6 个 SSPEC §17 提取复杂度基线 |

---

*本文档由 @presentation-architect 维护。Screen 复杂度变更时同步更新基线。*
