---
id: 07-specs.widget-id-map
title: Widget ID to UiBinding Mapping Table
status: active
owner: presentation-architect
created: 2026-06-22
tags:
  - ui
  - widget-id
  - ui-binding
  - reference
  - active
---

# Widget ID to UiBinding Mapping Table

> **职责**: @presentation-architect | **上游**: `07-specs/README.md` §2 (三位一体约束), `02-design-system/focus-binding.md` §4 (UiBinding)

> **widget_id 一旦分配，永久有效**: 重构时标记 deprecated，不重新分配同一 ID 给不同的 UI 元素。

---

## 1. 命名规范

命名格式: `{screen}_{region}_{element}_{variant}` (snake_case)

- `{screen}` — 所属 Screen 名，全小写 (main_menu, battle, inventory, shop, settings, save_load)
- `{region}` — 区域名 (title_area, button_list, top_bar, action_menu, grid...)
- `{element}` — 元素名 (title_text, new_game_btn, hp_bar, mp_bar...)
- `{variant}` — 可选变体后缀，用于同区域同类元素的多个实例 (如 buff_icons_0, buff_icons_1)

### 1.1 命名规则

| 规则 | 说明 |
|------|------|
| 全小写 snake_case | 禁止 camelCase 或 PascalCase |
| 以 screen 名开头 | 不同 Screen 的 widget_id 通过前缀区分 |
| 元素名使用通用缩写 | btn(button), bar(bar/indicator), txt(text), icon(icon), panel(panel), grid(grid), slot(slot) |
| 序号从 0 开始 | buff_icons_0, buff_icons_1, item_slot_0, item_slot_1... |
| 禁止数字 ID | 使用有意义的语义名称 |

---

## 2. 稳定性契约

```yaml
widget_id_stability:
  原则: "widget_id 一旦分配，永久有效"
  重构: "标记 deprecated，不重新分配"
  删除: "保留 ID 记录，标记为 deprecated"
  新增: "按命名规范申请新 ID"
  复用: "禁止将同一 ID 分配给不同的 UI 元素"
```

### 2.1 widget_id 生命周期

| 阶段 | 含义 | 操作 |
|------|------|------|
| `active` | 当前正在使用 | 正常查询和绑定 |
| `deprecated` | 不再使用，保留记录 | 代码中移除对应 UiBinding，ID 不再分配 |

---

## 3. UiBinding 变体参考

UiBinding 完整定义: `src/ui/binding/ui_binding.rs`

```rust
pub enum UiBinding {
    // ── Battle HUD ──
    Hp, MaxHp, Mp, MaxMp, Ap, MaxAp,
    Turn, Phase,
    BuffSlot(u8),

    // ── Character Panel ──
    Level, Exp, Name, CharacterLevel,

    // ── Skill Panel ──
    SkillSlot(u8), Cooldown,

    // ── Inventory ──
    ItemSlot(u8), Gold,

    // ── Quest ──
    QuestEntry(u16),

    // ── General ──
    Tooltip, Modal, Notification, Text, Icon,
}
```

### 3.1 UiBinding::None 标记规则

| 场景 | 使用 UiBinding::None | 说明 |
|------|---------------------|------|
| 纯容器 (Container/Panel) | 是 | 仅用于布局，无数据绑定 |
| 静态文本 (LocalizedKey) | 是 | 文本通过 LocalizationKey 设置，非 ViewModel 驱动 |
| 按钮 (Button) | 是 | 按钮交互通过 UiCommand 实现，不绑定 ViewModel |
| 装饰元素 | 是 | 纯视觉元素，无数据源 |
| 有数据绑定 (HP/MP/Turn) | 否 | 使用对应变体 (Hp/Mp/Turn/Phase 等) |
| 动态插槽 (buff/skill/item) | 否 | 使用参数化变体 (BuffSlot(n)/SkillSlot(n)/ItemSlot(n)) |
| 弹窗/通知/工具提示 | 否 | 使用 Tooltip/Modal/Notification |

---

## 4. 映射总表

> 以下按 Screen 分组。widget_id 前的 `*` 表示已分配但当前 Screen 尚未实现（预留 ID）。

### 4.1 MainMenuScreen

| widget_id | UiBinding | 状态 | 首次引入 |
|-----------|-----------|------|---------|
| `main_menu_root` | `UiBinding::None` | active | main_menu_screen.md |
| `main_menu_title_area` | `UiBinding::None` | active | main_menu_screen.md |
| `main_menu_title_text` | `UiBinding::Text` | active | main_menu_screen.md |
| `main_menu_subtitle_text` | `UiBinding::None` | active | main_menu_screen.md |
| `main_menu_button_list` | `UiBinding::None` | active | main_menu_screen.md |
| `main_menu_new_game_btn` | `UiBinding::None` | active | main_menu_screen.md |
| `main_menu_load_game_btn` | `UiBinding::None` | active | main_menu_screen.md |
| `main_menu_settings_btn` | `UiBinding::None` | active | main_menu_screen.md |
| `main_menu_version_text` | `UiBinding::None` | active | main_menu_screen.md |

### 4.2 BattleScreen

| widget_id | UiBinding | 状态 | 首次引入 |
|-----------|-----------|------|---------|
| `battle_screen_root` | `UiBinding::None` | active | battle_screen.md |
| `battle_top_bar` | `UiBinding::None` | active | battle_screen.md |
| `battle_turn_indicator` | `UiBinding::Turn` | active | battle_screen.md |
| `battle_phase_label` | `UiBinding::Phase` | active | battle_screen.md |
| `battle_end_turn_btn` | `UiBinding::None` | active | battle_screen.md |
| `battle_battle_area` | `UiBinding::None` | active | battle_screen.md |
| `battle_minimap` | `UiBinding::None` | active | battle_screen.md |
| `battle_char_panel` | `UiBinding::None` | active | battle_screen.md |
| `battle_char_avatar` | `UiBinding::None` | active | battle_screen.md |
| `battle_char_name` | `UiBinding::Name` | active | battle_screen.md |
| `battle_char_level` | `UiBinding::CharacterLevel` | active | battle_screen.md |
| `battle_hp_bar` | `UiBinding::Hp` | active | battle_screen.md |
| `battle_hp_text` | `UiBinding::Hp` | active | battle_screen.md |
| `battle_mp_bar` | `UiBinding::Mp` | active | battle_screen.md |
| `battle_mp_text` | `UiBinding::Mp` | active | battle_screen.md |
| `battle_ap_bar` | `UiBinding::Ap` | active | battle_screen.md |
| `battle_buff_area` | `UiBinding::None` | active | battle_screen.md |
| `battle_buff_icons_0` | `UiBinding::BuffSlot(0)` | active | battle_screen.md |
| `battle_buff_icons_1` | `UiBinding::BuffSlot(1)` | active | battle_screen.md |
| `battle_buff_icons_2` | `UiBinding::BuffSlot(2)` | active | battle_screen.md |
| `battle_buff_icons_3` | `UiBinding::BuffSlot(3)` | active | battle_screen.md |
| `battle_action_menu` | `UiBinding::None` | active | battle_screen.md |
| `battle_attack_btn` | `UiBinding::None` | active | battle_screen.md |
| `battle_skill_btn` | `UiBinding::None` | active | battle_screen.md |
| `battle_defend_btn` | `UiBinding::None` | active | battle_screen.md |
| `battle_wait_btn` | `UiBinding::None` | active | battle_screen.md |

### 4.3 InventoryScreen

| widget_id | UiBinding | 状态 | 首次引入 |
|-----------|-----------|------|---------|
| `inventory_screen_root` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_header` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_header_title` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_header_close` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_body` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_character_list` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_character_list_label` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_character_rows` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_character_row` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_character_empty` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_area` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_filter_bar` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_filter_all` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_filter_consumable` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_filter_equipment` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_filter_key_item` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_filter_material` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_item_grid` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_item_cell` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_empty_state` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_loading` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_error_state` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_error_text` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_retry_btn` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_description_panel` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_item_detail` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_item_detail_icon` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_item_detail_name` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_item_detail_desc` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_item_detail_stats` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_action_bar` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_use_btn` | `UiBinding::None` | active | inventory_screen.md |
| `inventory_drop_btn` | `UiBinding::None` | active | inventory_screen.md |

> **Deprecated IDs**: 以下旧版 InventoryScreen widget_id 已废弃（保留记录，不重新分配）：
> — 第一轮废弃（替换前 SSPEC 的旧 18 个 ID）：
> `inventory_top_bar`, `inventory_gold_display`, `inventory_grid`,
> `inventory_item_slot_0`, `inventory_item_slot_1`, `inventory_item_slot_2`,
> `inventory_item_slot_3`, `inventory_item_slot_4`, `inventory_item_slot_5`,
> `inventory_char_panel`, `inventory_char_name`, `inventory_char_level`,
> `inventory_equipment_area`, `inventory_weapon_slot`, `inventory_armor_slot`,
> `inventory_accessory_slot`
> — 第二轮废弃（上一轮映射中命名与 SSPEC 不一致的 ID）：
> `inventory_title_text`, `inventory_close_btn`,
> `inventory_character_portrait`, `inventory_character_name`,
> `inventory_filter_weapon`, `inventory_filter_armor`, `inventory_filter_key`,
> `inventory_item_cell_0`, `inventory_item_cell_1`, `inventory_item_cell_2`,
> `inventory_item_cell_3`, `inventory_item_cell_4`, `inventory_item_cell_5`,
> `inventory_loading_spinner`, `inventory_error_panel`,
> `inventory_item_name`, `inventory_item_desc`

### 4.4 ShopScreen

| widget_id | UiBinding | 状态 | 首次引入 |
|-----------|-----------|------|---------|
| `shop_screen_root` | `UiBinding::None` | active | shop_screen.md |
| `shop_header` | `UiBinding::None` | active | shop_screen.md |
| `shop_tab_bar` | `UiBinding::None` | active | shop_screen.md |
| `shop_buy_tab_btn` | `UiBinding::None` | active | shop_screen.md |
| `shop_sell_tab_btn` | `UiBinding::None` | active | shop_screen.md |
| `shop_item_list` | `UiBinding::None` | active | shop_screen.md |
| `shop_item_card` | `UiBinding::None` | active | shop_screen.md |
| `shop_item_name` | `UiBinding::Text` | active | shop_screen.md |
| `shop_item_price` | `UiBinding::Text` | active | shop_screen.md |
| `shop_buy_btn` | `UiBinding::None` | active | shop_screen.md |
| `shop_sell_btn` | `UiBinding::None` | active | shop_screen.md |
| `shop_player_gold` | `UiBinding::Gold` | active | shop_screen.md |
| `shop_close_btn` | `UiBinding::None` | active | shop_screen.md |

### 4.5 SettingsScreen

| widget_id | UiBinding | 状态 | 首次引入 |
|-----------|-----------|------|---------|
| `settings_screen_root` | `UiBinding::None` | active | settings_screen.md |
| `settings_header` | `UiBinding::None` | active | settings_screen.md |
| `settings_title_text` | `UiBinding::Text` | active | settings_screen.md |
| `settings_close_btn` | `UiBinding::None` | active | settings_screen.md |
| `settings_tab_panel` | `UiBinding::None` | active | settings_screen.md |
| `settings_tab_list` | `UiBinding::None` | active | settings_screen.md |
| `settings_gameplay_tab_btn` | `UiBinding::None` | active | settings_screen.md |
| `settings_graphics_tab_btn` | `UiBinding::None` | active | settings_screen.md |
| `settings_audio_tab_btn` | `UiBinding::None` | active | settings_screen.md |
| `settings_battle_tab_btn` | `UiBinding::None` | active | settings_screen.md |
| `settings_tab_content` | `UiBinding::None` | active | settings_screen.md |
| `settings_damage_toggle` | `UiBinding::None` | active | settings_screen.md |
| `settings_minimap_toggle` | `UiBinding::None` | active | settings_screen.md |
| `settings_grid_toggle` | `UiBinding::None` | active | settings_screen.md |
| `settings_autobattle_toggle` | `UiBinding::None` | active | settings_screen.md |
| `settings_theme_selector` | `UiBinding::None` | active | settings_screen.md |
| `settings_language_selector` | `UiBinding::None` | active | settings_screen.md |
| `settings_master_volume` | `UiBinding::None` | active | settings_screen.md |
| `settings_bgm_volume` | `UiBinding::None` | active | settings_screen.md |
| `settings_sfx_volume` | `UiBinding::None` | active | settings_screen.md |
| `settings_battle_speed` | `UiBinding::None` | active | settings_screen.md |
| `settings_tooltip_delay` | `UiBinding::None` | active | settings_screen.md |
| `settings_reset_btn` | `UiBinding::None` | active | settings_screen.md |

### 4.6 SaveLoadScreen

| widget_id | UiBinding | 状态 | 首次引入 |
|-----------|-----------|------|---------|
| `save_load_screen_root` | `UiBinding::None` | active | save_load_screen.md |
| `save_load_header` | `UiBinding::None` | active | save_load_screen.md |
| `save_load_mode_toggle` | `UiBinding::None` | active | save_load_screen.md |
| `save_load_save_tab_btn` | `UiBinding::None` | active | save_load_screen.md |
| `save_load_load_tab_btn` | `UiBinding::None` | active | save_load_screen.md |
| `save_load_slot_list` | `UiBinding::None` | active | save_load_screen.md |
| `save_load_slot_0` | `UiBinding::None` | active | save_load_screen.md |
| `save_load_slot_1` | `UiBinding::None` | active | save_load_screen.md |
| `save_load_slot_2` | `UiBinding::None` | active | save_load_screen.md |
| `save_load_slot_3` | `UiBinding::None` | active | save_load_screen.md |
| `save_load_slot_name` | `UiBinding::Text` | active | save_load_screen.md |
| `save_load_slot_timestamp` | `UiBinding::Text` | active | save_load_screen.md |
| `save_load_slot_preview` | `UiBinding::None` | active | save_load_screen.md |
| `save_load_confirm_btn` | `UiBinding::None` | active | save_load_screen.md |
| `save_load_back_btn` | `UiBinding::None` | active | save_load_screen.md |

---

## 5. 按 UiBinding 变体索引

> 反向索引：给定 UiBinding 变体，查找所有关联的 widget_id。

| UiBinding | 关联 widget_id |
|-----------|---------------|
| `UiBinding::None` | 所有容器和按钮（见各 Screen 列表） |
| `UiBinding::Hp` | `battle_hp_bar`, `battle_hp_text` |
| `UiBinding::MaxHp` | *预留* |
| `UiBinding::Mp` | `battle_mp_bar`, `battle_mp_text` |
| `UiBinding::MaxMp` | *预留* |
| `UiBinding::Ap` | `battle_ap_bar` |
| `UiBinding::MaxAp` | *预留* |
| `UiBinding::Turn` | `battle_turn_indicator` |
| `UiBinding::Phase` | `battle_phase_label` |
| `UiBinding::BuffSlot(n)` | `battle_buff_icons_0`, `battle_buff_icons_1`, `battle_buff_icons_2`, `battle_buff_icons_3` |
| `UiBinding::Level` | *预留* |
| `UiBinding::Exp` | *预留* |
| `UiBinding::Name` | `battle_char_name` |
| `UiBinding::CharacterLevel` | `battle_char_level` |
| `UiBinding::SkillSlot(n)` | *预留* |
| `UiBinding::Cooldown` | *预留* |
| `UiBinding::ItemSlot(n)` | `inventory_item_cell_0..5` |
| `UiBinding::Gold` | `shop_player_gold` |
| `UiBinding::QuestEntry(n)` | *预留* |
| `UiBinding::Tooltip` | *预留* |
| `UiBinding::Modal` | *预留* |
| `UiBinding::Notification` | *预留* |
| `UiBinding::Text` | `inventory_title_text`, `main_menu_title_text`, `shop_item_name`, `shop_item_price`, `save_load_slot_name`, `save_load_slot_timestamp`, `settings_title_text` |
| `UiBinding::Icon` | *预留* |

> `*预留*` 状态: UiBinding 变体已定义，但尚无 widget_id 关联。预留表示该变体预期被未来 Screen 使用，当前不产生编译错误。
>
> **Deprecated InventoryScreen entries in Section 5**: 以下旧版 InventoryScreen widget_id 已从关联映射中移除（完整废弃列表见 §4.3 Deprecated IDs）：
> - `UiBinding::Level` — 移除 `inventory_char_level`
> - `UiBinding::Name` — 移除 `inventory_char_name`
> - `UiBinding::ItemSlot(n)` — 移除 `inventory_item_slot_0..5`, `inventory_weapon_slot`, `inventory_armor_slot`, `inventory_accessory_slot`
> - `UiBinding::Gold` — 移除 `inventory_gold_display`

---

## 6. 维护规则

| 规则 | 说明 |
|------|------|
| 新增 Screen 时 | 必须在 widget-id-map.md 注册所有 widget_id 后，再创建 SSPEC |
| 新增 widget_id 时 | 按命名规范生成，添加到对应 Screen 分组末尾 |
| 废弃 widget_id 时 | 标记为 `deprecated`，不移除记录 |
| 修改 UiBinding 映射时 | 同步更新本表 + 对应 Screen 的 SSPEC |
| UiBinding 新增变体时 | 先在 `ui_binding.rs` 定义，再在本表注册关联的 widget_id |

---

## 附录 A: 变更记录

| 日期 | 变更 | 原因 |
|------|------|------|
| 2026-06-22 | 初始创建 | 首次 Screen Spec 体系建立 |
| 2026-06-22 | 更新 InventoryScreen §4.3 + §5 | 18 个旧 ID 替换为 33 个新 ID（与 inventory_screen.md Spec 对齐） |

---

*本文档由 @presentation-architect 维护。widget_id 变更必须通过 Presentation Architect 审查。*
