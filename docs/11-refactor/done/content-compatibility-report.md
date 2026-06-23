---
id: 11-refactor.content-compatibility-report
title: Content Architecture Compatibility Report for Screen Spec
status: active
owner: content-architect
created: 2026-06-22
tags:
  - content
  - screen-spec
  - widget-id
  - localization-key
  - data-flow
  - registry
---

# Content 架构与 Screen Spec 兼容性报告

## 执行摘要

UI Screen Specification 重构（07-specs/）与现有的 Content Platform 架构**基本兼容**。在 P0 交付前需要关注三个维度：Widget-ID-to-Def 映射文档、BattleScreen 文本缺失的 LocalizationKeys，以及 Projection 模式的小备注。不需要对 03-content/ 进行架构变更。

---

## 1. Def Registry 映射审查

### 1.1 分类：三类 Widget ID

计划中的 Widget ID（`07-specs/references/widget-id-map.md`）相对于 Def Registry 分为三类：

**类别 A：显示 Def 派生数据的 Widget ID（通过 Projection 间接）**

这些 Widget ID 对应 ViewModel 中的字段（`UiStore` 字段），而这些字段由查询 `DefRegistry<T>` 的 Projection 填充。Widget 本身从不直接读取 Def Registry——只有 Projection 这样做。

| Widget ID | ViewModel 字段 | Def Registry | Projection |
|-----------|----------------|-------------|------------|
| `battle_hp_bar` | `BattleHudVm.hp / max_hp` | 不适用（运行时实例数据） | BattleProjection |
| `battle_mp_bar` | `BattleHudVm.mp / max_mp` | 不适用（运行时实例数据） | BattleProjection |
| `battle_buff_icons_0` | `BattleHudVm.buffs[0]` | `DefRegistry<BuffDef>`（通过 `BuffVm`） | BattleProjection |
| `battle_skill_slot_0` | `SkillPanelVm.skills[0]` | `DefRegistry<SpellDef>`（通过 `SkillSlotVm`） | BattleProjection |
| `battle_character_name` | `CharacterPanelVm.name_key` | `DefRegistry<CharacterDef>` | CharacterPanelProjection |
| `inventory_item_grid` | `InventoryVm.items` | `DefRegistry<ItemDef>` | InventoryProjection |
| `quest_entry_title` | `QuestLogVm.entries[0]` | `DefRegistry<QuestDef>` | QuestProjection |

**兼容性裁定**：完全兼容。现有管线（`projection-viewmodel.md` 8.1）已定义了 `DefRegistry → Projection → ViewModel` 作为标准流程。显示 Def 数据的 Widget ID 映射到 ViewModel 字段，而非直接映射到 Def ID。

**建议**：`widget-id-map.md` 应为类别 A 的 Widget ID 添加 `def_type` 列，记录它们最终从哪个 Def Registry 消费数据。这有助于影响分析（"我修改 SpellDef 时哪些 widget 会受影响？"）。

计划中 YAML 的示例补充：

```yaml
# Widget ID → Def Type reference (Category A only)
def_dependencies:
  battle_hp_bar:             ~                 # 运行时 HP，无 Def
  battle_buff_icons_0:       BuffDef           # 通过 BuffVm → Projection → DefRegistry<BuffDef>
  battle_skill_slot_0:       SpellDef          # 通过 SkillSlotVm → Projection → DefRegistry<SpellDef>
  battle_character_name:     CharacterDef      # 通过 CharacterPanelVm → Projection → DefRegistry<CharacterDef>
  inventory_item_grid:       ItemDef           # 通过 InventoryVm → Projection → DefRegistry<ItemDef>
```

---

**类别 B：发出 Def 引用命令的 Widget ID**

这些 Widget ID 发出携带类型 ID（`SkillId`、`ItemId` 等）的 `UiCommand` 事件。Widget 本身不解析 Def——它只触发命令。Def 解析在 Domain 下游进行。

| Widget ID | 发出的命令 | ID 类型 | Def Registry |
|-----------|-----------|---------|-------------|
| `battle_attack_btn` | `UiCommand::CastSkill(skill_id, target_id)` | `SkillId` | `DefRegistry<SpellDef>`（由 Domain 解析） |
| `battle_skill_btn` | `UiCommand::CastSkill(skill_id, target_id)` | `SkillId` | `DefRegistry<SpellDef>` |
| `battle_item_btn` | `UiCommand::UseItem(item_id, target_id)` | `ItemId` | `DefRegistry<ItemDef>` |
| `inventory_item_slot_0` | `UiCommand::UseItem(item_id)` | `ItemId` | `DefRegistry<ItemDef>` |
| `shop_buy_btn` | `UiCommand::BuyItem(item_id, quantity)` | `ItemId` | `DefRegistry<ItemDef>` |

**兼容性裁定**：完全兼容。命令发出模式已存在于 `screens.md` 2.4 和 4.4 中。不需要修改 Content Platform。

---

**类别 C：无 Def 映射的纯 UI 容器**

这些 Widget ID 对任何 Def 类型都零依赖。它们是结构性容器或状态指示器。

| Widget ID | UiBinding | Def 依赖 |
|-----------|-----------|---------|
| `battle_root` | `UiBinding::None` | 无 |
| `battle_top_bar` | `UiBinding::None` | 无 |
| `battle_turn_indicator` | `UiBinding::Turn` | 无（读取 BattleHudVm.turn_number） |
| `battle_phase_label` | `UiBinding::Phase` | 无（读取 BattleHudVm.phase_key） |
| `battle_end_turn_btn` | `UiBinding::None` | 无 |
| `battle_area` | `UiBinding::None` | 无 |
| `battle_action_menu` | `UiBinding::None` | 无 |
| `battle_defend_btn` | `UiBinding::None` | 无 |
| `battle_wait_btn` | `UiBinding::None` | 无 |
| `main_menu_root` | `UiBinding::None` | 无 |
| `main_menu_title_text` | `UiBinding::Text` | 无 |
| `main_menu_version_text` | `UiBinding::None` | 无 |

**兼容性裁定**：完全兼容。不涉及 Content Platform。

### 1.2 现有 UiBinding 对新 Widget ID 的覆盖

现有的 `UiBinding` 枚举（`focus-binding.md` 4.2）定义了 17 个变体。与计划的 Widget ID 映射：

| UiBinding 变体 | 映射的 Widget ID | 状态 |
|----------------|-----------------|------|
| `Hp` | `battle_hp_bar`、`battle_hp_text` | 已存在 |
| `Mp` | `battle_mp_bar`、`battle_mp_text` | 已存在 |
| `Ap` | `battle_ap_bar` | 已存在 |
| `Turn` | `battle_turn_indicator` | 已存在 |
| `Phase` | `battle_phase_label` | 已存在 |
| `Level` | `battle_character_level` | 已存在 |
| `SkillSlot(u8)` | `battle_skill_slot_0`、`battle_skill_slot_1` 等 | 已存在 |
| `ItemSlot(u8)` | `inventory_item_slot_0` 等 | 已存在 |
| `Gold` | `shop_gold_display`、`inventory_gold` | 已存在 |
| `QuestEntry(u16)` | `quest_entry_title` | 已存在 |
| `Text` | `main_menu_title_text`、通用文本 widget | 已存在 |

**发现缺口**：像 `end_turn_btn`、`battle_attack_btn`、`battle_defend_btn`、`battle_wait_btn` 这样的 Widget 在计划示例中使用了 `UiBinding::None`。这是正确的，因为它们是不显示绑定数据的交互式按钮。但是，如果它们需要通过查询来定位（例如启用/禁用所有操作按钮），添加专用的 UiBinding 变体（`ActionAttack`、`ActionDefend`、`ActionWait`）将比扫描 `UiBinding::None` + 辅助组件标记更高效。这是 @presentation-architect 的决定，不是 content 架构关注的事项。

---

## 2. LocalizationKey 覆盖审查

### 2.1 现有 LocalizationKey 覆盖

现有 schema（`localization_schema.md` 3.1 和 `theme-localization.md` 4.2）定义的格式为：

```
ui.<scope>.<id>.<suffix>
```

现有文档中提到的示例 key：

| Key | 文件 | 状态 |
|-----|------|------|
| `ui.battle.end_turn` | theme-localization.md 4.2 | 已定义 |
| `ui.battle.victory` | theme-localization.md 4.2 | 已定义 |
| `ui.battle.phase.player` | projection-viewmodel.md 7 | 已定义 |
| `ui.battle.attack` | localization_schema.md 3.6（生成的 key 示例） | 已定义 |
| `ui.battle.defend` | localization_schema.md 3.6（生成的 key 示例） | 已定义 |
| `ui.battle.damage_dealt.text` | localization_schema.md 3.6（生成的 key 示例） | 已定义 |
| `ui.battle.heal_received.text` | localization_schema.md 3.6（生成的 key 示例） | 已定义 |
| `ui.battle.unit_died.text` | localization_schema.md 3.6（生成的 key 示例） | 已定义 |
| `ui.menu.settings` | localization_schema.md 3.6 | 已定义 |
| `ui.menu.quit` | localization_schema.md 3.6 | 已定义 |
| `ui.inventory.empty_slot` | theme-localization.md 4.2 | 已定义 |
| `ui.shop.buy_confirm` | theme-localization.md 4.2 | 已定义 |
| `ui.quest.abandon_confirm` | theme-localization.md 4.2 | 已定义 |
| `ui.settings.show_grid` | theme-localization.md 4.2 | 已定义 |
| `ui.notification.item_acquired` | theme-localization.md 4.2 | 已定义 |

### 2.2 缺口：BattleScreen 文本（P0 关键）

`screens.md` 2.3 记录了 BattleScreen 中的硬编码文本字符串。以下文本缺少 LocalizationKey，必须在 Screen Spec 定稿前定义：

| 当前硬编码文本 | Screen 区域 | 建议的 LocalizationKey | 优先级 |
|----------------|-------------|------------------------|--------|
| "Turn: {n}" | TurnInfoBar | `ui.battle.turn_indicator.text` | P0 |
| "Phase: Player Turn" | TurnInfoBar | `ui.battle.phase.player`（已存在） | P0 |
| "Phase: Enemy Turn" | TurnInfoBar | `ui.battle.phase.enemy` | P0 |
| "Phase: Victory" | TurnInfoBar | `ui.battle.phase.victory` | P0 |
| "Phase: Defeat" | TurnInfoBar | `ui.battle.phase.defeat` | P0 |
| "Attack" | ActionMenu | `ui.battle.action.attack` | P0 |
| "Defend" | ActionMenu | `ui.battle.action.defend` | P0 |
| "Skill" | ActionMenu | `ui.battle.action.skill` | P0 |
| "Item" | ActionMenu | `ui.battle.action.item` | P0 |
| "Wait" | ActionMenu | `ui.battle.action.wait` | P0 |
| "HP" 标签 | CharacterCard | `ui.battle.hp_label` | P0 |
| "MP" 标签 | CharacterCard | `ui.battle.mp_label` | P0 |
| "AP" 标签 | CharacterCard | `ui.battle.ap_label` | P0 |
| "Lv." 前缀 | CharacterCard | `ui.battle.level_prefix` | P0 |

注意：`phase_key` 在 `projection-viewmodel.md` 7 中已被记录为 `"ui.battle.phase.player"`，但现有的 BattleHudVm 直接使用 `&'static str` 类型写为 `"ui.battle.phase.player"`，而不是使用 `generated/keys.rs` 中的命名常量。这在 key 生成系统实现时需要进行对齐。

### 2.3 缺口：MainMenuScreen 文本（P0）

`screens.md` 3.3 列出了硬编码文本：

| 当前文本 | 建议的 LocalizationKey | 优先级 |
|---------|------------------------|--------|
| "Fre"（标题） | `ui.main_menu.title` | P0 |
| "A Bevy SRPG"（副标题） | `ui.main_menu.subtitle` | P0 |
| "New Game" | `ui.main_menu.new_game` | P0 |
| "Load Game" | `ui.main_menu.load_game` | P0 |
| "Settings" | `ui.main_menu.settings` | P0 |
| "v0.1.0"（版本） | `ui.main_menu.version` | P0 |

注意：`ui.menu.settings` 和 `ui.menu.quit` 已经存在于 `localization_schema.md` 3.6 中。建议的 `ui.main_menu.*` 命名空间与现有约定一致。

### 2.4 缺口：InventoryScreen 文本（P1）

| 当前文本 | 建议的 LocalizationKey | 优先级 |
|---------|------------------------|--------|
| "Inventory" | `ui.inventory.title` | P1 |
| "Gold: {n}" | `ui.inventory.gold_display` | P1 |
| "Close" | `ui.inventory.close` | P1 |

### 2.5 缺口：SettingsScreen 文本（P1）

| 当前文本 | 建议的 LocalizationKey | 优先级 |
|---------|------------------------|--------|
| "Show Damage Numbers" | `ui.settings.show_damage_numbers` | P1 |
| "Show Minimap" | `ui.settings.show_minimap` | P1 |
| "Show Grid" | `ui.settings.show_grid`（已存在） | P1 |
| "Auto Battle" | `ui.settings.auto_battle` | P1 |
| "Theme" | `ui.settings.theme_label` | P1 |
| "Language" | `ui.settings.language_label` | P1 |
| "Master Volume" | `ui.settings.master_volume` | P1 |
| "BGM Volume" | `ui.settings.bgm_volume` | P1 |
| "SFX Volume" | `ui.settings.sfx_volume` | P1 |
| "Battle Speed" | `ui.settings.battle_speed` | P1 |
| "Tooltip Delay" | `ui.settings.tooltip_delay` | P1 |
| "Reset to Defaults" | `ui.settings.reset_defaults` | P1 |

### 2.6 缺口：ShopScreen 文本（P1）

| 当前文本 | 建议的 LocalizationKey | 优先级 |
|---------|------------------------|--------|
| 商店名称 | `ui.shop.greeting`（更广范围） | P1 |
| "Buy" 选项卡 | `ui.shop.tab_buy` | P1 |
| "Sell" 选项卡 | `ui.shop.tab_sell` | P1 |
| "Cart: {count} items, {total}" | `ui.shop.cart_summary` | P1 |
| "Buy" 按钮 | `ui.shop.buy` | P1 |
| "Cancel" | `ui.shop.cancel` | P1 |
| "Confirm" | `ui.shop.confirm` | P1 |

### 2.7 命名约定一致性

现有约定（`theme-localization.md` 4.2）使用 `ui.<scope>.<id>`，其中 scope 是功能域名（battle、inventory、shop 等）。计划提出更细粒度的 `ui.battle.{screen}.{widget}.{field}` 模式。

**冲突**：计划使用 `battle_phase_label` 作为 widget_id，它显示从 `"ui.battle.phase.player"` 解析的值。Screen 级别命名（`battle_<element>`）用于 Widget ID，而 LocalizationKey 遵循现有的 `ui.<scope>.<id>` 模式。

**建议**：对 LocalizationKey 保持现有的 `ui.<scope>.<id>` 模式（不变）。Widget ID 使用 `{screen}_{region}_{element}` snake_case。这是两个独立的命名系统：
- LocalizationKey（`ui.battle.end_turn`）= .ftl 文件中的文本查找 key
- widget_id（`battle_end_turn_btn`）= 永久的 widget 实例标识符

不需要修改现有的 LocalizationKey 约定。

---

## 3. 数据流路径审查

### 3.1 现有路径

`projection-viewmodel.md` 8.1 中定义的路径为：

```
Content（assets/config/*.ron）
    ↓ AssetServer 加载
DefRegistry（Resource）
    ↓ Projection 查询
ViewModel（UiStore）
    ↓ Dirty<T> 标记
Widget
```

这是正确且当前的架构。无需修改即可适应 Screen Specs。

### 3.2 widget_id 如何融入现有流程

```
07-specs/references/widget-id-map.md（仅文档）
    │  映射：widget_id → UiBinding 变体
    │        widget_id → DefRegistry 类型（类别 A）
    │        widget_id → ViewModel 字段
    ▼
Widget 生成代码（src/ui/screens/*.rs）
    │  使用 widget_id 作为 UiBinding + 实体命名约定
    ▼
Dirty<T> + UiBinding → 运行时 Widget 刷新
```

widget_id 是一个文档层概念。它**不**引入新的运行时组件、新系统或新数据结构。它映射到：
- 一个 `UiBinding` 变体（已存在，在 `focus-binding.md` 4.2 中定义）
- 一个 ViewModel 字段（已存在，在 `projection-viewmodel.md` 3.4 中定义）
- 可选地，一个 `DefRegistry<T>` 类型（已存在，在 `03-content/README.md` 5 中定义）

### 3.3 Screen Spec 的 Projection 调整

计划的事件契约部分（3.8）在 Screen Spec 中引入了结构化的 `Projection` 注解。这是文档，不是运行时变更：

```yaml
TurnStarted:
  source: Domain Event (Combat Domain)
  projection: BattleProjection.project_turn()
  vm_update: BattleHudVm.turn_number += 1
  vm_update: BattleHudVm.phase_key = "ui.battle.phase.player"
  side_effect: mark_dirty::<BattleHudVm>()
```

**兼容性裁定**：完全兼容。现有的 `projection-viewmodel.md` 7 已经以表格形式记录了 Projection 映射。Screen Spec 的 YAML 格式是同一信息的不同表示形式。

### 3.4 区域状态映射影响

计划引入了按区域的状态映射（Loading/Empty/Error 状态，见计划 1.1 表）。这完全是 @presentation-architect 关注的事项——Content Platform 不涉及，因为：

- Loading 状态由数据可用性决定（不是 Def 加载）
- Empty 状态取决于过滤后的 ViewModel 内容（不是 Def 存在与否）
- Error 状态用于网络/验证错误（不是 schema 错误）

Content Platform 已经在验证管道中有自己的错误处理（8 条验证规则，`content-platform-manifesto.md`）。Screen 级别的错误状态是正交的。

### 3.5 Def ↔ UiBinding ↔ ViewModel 字段映射表

为完善 Screen Spec，应在 `widget-id-map.md` 中包含以下交叉引用：

```yaml
# Widget ID → UiBinding → ViewModel Field → Def Registry
# 此表连接 widget 文档与 Content Platform

BattleScreen:
  battle_hp_bar:
    uibinding: UiBinding::Hp
    vm_field: BattleHudVm.hp / max_hp
    def_registry: ~（运行时实例数据）
  battle_mp_bar:
    uibinding: UiBinding::Mp
    vm_field: BattleHudVm.mp / max_mp
    def_registry: ~
  battle_buff_icons_0:
    uibinding: UiBinding::BuffSlot(0)
    vm_field: BattleHudVm.buffs[0]
    def_registry: DefRegistry<BuffDef>
  battle_skill_slot_0:
    uibinding: UiBinding::SkillSlot(0)
    vm_field: SkillPanelVm.skills[0]
    def_registry: DefRegistry<SpellDef>
  battle_character_name:
    uibinding: UiBinding::Name
    vm_field: CharacterPanelVm.name_key
    def_registry: DefRegistry<CharacterDef>
```

此表作为 Content Platform（Def 类型）、UI Binding 系统（UiBinding 枚举）和 Screen Spec（widget ID）之间的正式映射。

---

## 4. Widget ID 命名约定建议

### 4.1 格式

计划已提出 `snake_case`，这与 Rust 变量命名一致。这被确认为正确选择。

**建议**：采用 `{screen}_{region}_{element}` 作为基本格式，并进行以下细化：

```
格式：{screen}_{region}_{element}_{variant}

screen    = 功能 Screen 名称（battle、main_menu、inventory、shop、settings、save_load）
region    = 水平/垂直区域或功能部分（top_bar、action_menu、char_panel）
element   = 特定 widget 功能（hp_bar、end_turn_btn、title_text、item_grid）
variant   = 编号实例的可选后缀（_0、_1、_2）或子变体（_icon、_label）

示例：
battle_top_bar_turn_indicator    # 顶栏中的回合数显示
battle_action_menu_skill_btn_0   # 操作菜单中的第一个技能按钮
inventory_main_grid_item_slot_3  # 物品栏网格中的第四个物品位
settings_graphics_theme_selector # 图形选项卡设置中的主题下拉框
save_load_list_slot_0            # 存档/读档列表中的第一个存档位
```

### 4.2 最大长度指南

考虑到 5 年以上和 10,000 以上的资源量，widget ID 不应超过 60 个字符。这确保它们适合典型的工具显示和调试覆盖层。

避免过度深层嵌套：`battle_screen_char_panel_buff_section_buff_icon_0`（56 字符，临界值）应缩短为 `battle_buff_icon_0`（19 字符）。

### 4.3 稳定性契约

计划的宪法修正案（Widget ID 稳定，P0）正确地指出：widget_id 是永久的。一旦分配，就不能重命名，只能弃用。

**Content 架构影响**：如果 widget_id 映射到 Def Registry 类型（类别 A），并且该 Def 类型被重命名或弃用，则 `widget-id-map.md` 必须记录弃用链：

```yaml
# 已弃用的 widget ID（永不重新分配）
battle_old_spell_icon:
  status: deprecated
  replaced_by: battle_skill_slot_0
  deprecation_reason: "Spell → Skill 术语对齐，ADR-0XX"

battle_old_mp_bar:
  status: deprecated
  replaced_by: battle_mp_bar
  deprecation_reason: "为与 hp_bar 一致而重命名"
```

这种弃用跟踪对于 mod 兼容性至关重要：引用 `battle_old_spell_icon` 的 mod 应仍能编译（带有弃用警告），而不是静默失败。

### 4.4 现有 UiBinding 命名对齐

现有的 `UiBinding` 枚举使用 PascalCase（`Hp`、`Mp`、`SkillSlot(u8)`）。Widget ID 使用 snake_case（`battle_hp_bar`、`battle_skill_slot_0`）。这是正确的——它们用于不同目的：

- `UiBinding::Hp` = ECS 组件标识符，按 Rust 约定使用 PascalCase
- `battle_hp_bar` = 设计文档标识符，按 UI 规范约定使用 snake_case
- 映射：`battle_hp_bar` ↔ `UiBinding::Hp` 在 `widget-id-map.md` 中

不存在命名冲突。

---

## 5. 兼容性裁定

### 5.1 完全兼容（无需修改）

| 维度 | 原因 |
|------|------|
| Def → Projection → ViewModel 数据流 | 现有流程（`projection-viewmodel.md` 8.1）直接支持所有 Screen Spec 需求 |
| Widget ID ↔ UiBinding 映射 | UiBinding 枚举已覆盖所有 BattleScreen 和 MainMenuScreen widget 类型 |
| 类别 B 和 C 的 Widget ID | 纯 UI 容器和发出命令的 ID 没有 Def 依赖 |
| 按区域状态映射 | 与 Content Platform 正交（纯 ViewModel 关注事项） |
| 事件契约文档 | Screen Spec 中的 YAML 格式仅为文档，与现有 Projection 映射一致 |
| LocalizationKey 格式 | 保持现有的 `ui.<scope>.<id>` 模式；widget ID 是独立的命名空间 |

### 5.2 需要补充（仅文档）

| 项目 | 要添加的内容 | 位置 | 优先级 |
|------|------------|------|--------|
| widget-id-map 中的 Def Registry 列 | 对类别 A 的 Widget ID，添加 `def_registry` 字段 | `07-specs/references/widget-id-map.md` | P0 |
| UiBinding ↔ ViewModel 字段映射 | 显示管道路径的交叉引用表 | `07-specs/references/widget-id-map.md` | P1 |
| Screen Spec 的 Def 依赖表 | 当 Screen Spec 提到显示 Def 数据的 widget 时，标注 Def 类型 | 每个 Screen Spec 的事件契约部分 | P1 |
| Widget ID 的弃用跟踪 | 记录已弃用/替换的 widget ID 的格式 | `07-specs/references/widget-id-map.md` | P2 |

### 5.3 需要创建（Content Platform 工作）

| 项目 | 内容 | 位置 | 优先级 |
|------|------|------|--------|
| 缺失的 LocalizationKeys | BattleScreen 文本 15+ 个 key，MainMenuScreen 5+ 个 key | `assets/localization/en-US/ui.ftl`（或等效文件） | **P0** |
| UiTextKey 枚举对齐 | 确保生成的 keys 模块（`generated/keys.rs`）覆盖所有 UI 文本 key | `src/infra/localization/generated/keys.rs` | P0（与 localization 实现同步） |

### 5.4 总结

```
Screen Spec 的 Content Platform 架构：████████████████████████ 95% 兼容
                                                  ████                   5% 需要文档补充
                                                  ▏                      1% 需要新的 LocalizationKeys
```

Content Platform 支持 Screen Spec 重构**不需要架构变更、Schema 变更、Registry 变更或管道变更**。上述识别的工作项是：
- 向 `widget-id-map.md` 添加文档（将类别 A 的 Widget ID 映射到 Def 类型）
- 为当前在 Screen 实现中硬编码的文本定义 LocalizationKey

这些都是前瞻性的补充，用于使现有文档与新的 Screen Spec 格式对齐，而非对现有架构缺陷的修复。

---

*报告由 @content-architect 编写。为 UI Screen Specification 重构第一阶段生成（参见 `docs/11-refactor/ui-screen-spec-execution-plan.md`）。*
