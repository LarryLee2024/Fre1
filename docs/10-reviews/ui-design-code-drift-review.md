---
title: UI 设计与代码偏移评审报告
reviewer: presentation-architect
date: 2026-06-22
status: active
---

# UI 设计与代码偏移评审报告

> 评审范围: `docs/06-ui/` 全部设计文档 与 `src/ui/` 全部 155 个 Rust 源文件
> 评审时间: 2026-06-22
> 评审人: @presentation-architect

---

## 1. 总体评估

**健康度: 开发中 (In Progress) — 架构骨架已建立，实现覆盖约 60%**

架构设计层面，代码与设计文档有良好的一致性: 三层分层 (Primitives/Widgets/Screens)、Projection/ViewModel、Dirty<T>、Focus 系统、Application 层、Navigation/Overlay 等核心基础设施均按设计实现。`plugin.rs` 的注册顺序与 `architecture.md` 第 8 节完全一致。

**主要问题:**

| 优先级 | 问题 | 影响 |
|--------|------|------|
| P0 | Screen 内存在直接 `spawn(Node{...})` + `BackgroundColor(Color::srgba(...))`，绕过 Primitives 层 | 违反架构铁律 5，长期导致 Factory 模式失效 |
| P0 | ViewModel 消费路径中断: Widgets 不消费 `Dirty<T>`，`UiBinding` 未实际用于绑定 | 数据流设计文档描述的 Dirty 消费链未落实 |
| P1 | 本地化渗透: 18 处 `Text::new()` 绕过了 LocalizationKey 体系 | 违反 Localization First 宪法原则 |
| P1 | SettingsScreen 实现极度简化: 2 个 Toggle vs Spec 的 4 个 Tab + 16 项设置 | Spec 与代码严重不一致 |
| P2 | 16 个设计文档定义的复合组件中仅 7 个实现 (44%) | 代码覆盖率不足，Screen 组合能力受限 |
| P2 | 50 处 `Color::srgb()` 调用散布在 theme/colors.rs 和部分 Screen 代码中 | 虽然 colors.rs 集中定义是合理模式，但 Screen 中仍存在硬编码 |

---

## 2. 架构原则合规度

### 2.1 工厂模式

**状态: ⚠️ 部分合规**

符合设计:
- MainMenuScreen: `spawn_main_menu` 使用 `spawn_panel`, `spawn_localized_button`, `spawn_text`, `spawn_list` 等工厂函数
- BattleScreen: `spawn_battle_screen` 使用 `spawn_panel`, `spawn_localized_button`, `spawn_character_card` 等
- InventoryScreen: `spawn_inventory_screen` 使用 `spawn_panel`, `spawn_inventory_grid`
- SettingsScreen: `spawn_settings_screen` 使用 `spawn_panel`, `spawn_toggle`, `spawn_localized_button`
- ShopScreen: `spawn_shop_screen` 使用 `spawn_panel`, `spawn_shop_panel`

违规:
- **`src/ui/screens/save_load/mod.rs:286-291`**: `commands.spawn((Node{...}, BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 1.0)), Name::new("AvatarPlaceholder")))` — 直接创建原始 Node + BackgroundColor 而不使用工厂
- **`src/ui/screens/save_load/mod.rs:307-315`**: 同样的模式创建 ScreenshotPlaceholder，使用 `Color::srgba(0.2, 0.2, 0.2, 1.0)`
- **`src/ui/overlay/layers.rs`**: 创建 5 个 UI Root 层使用了原始 `commands.spawn((Name::new("UiRoot::..."), ScreenLayer, Node{...}))` 模式 — 这是初始化阶段的合理例外 (Root 节点没有对应的 Primitives 工厂)，但未在架构文档中明确豁免
- **`src/ui/screens/main_menu/mod.rs:70-76`**: 覆盖 Title 文本时直接修改 `TextFont`，虽使用了 `asset_server.load(theme.typography.font_heading)` 读取 theme，但这段 override 逻辑应当在 Primitives 层处理

### 2.2 三层分层

**状态: ✅ 合规**

依赖方向检查:
- `primitives/` -> 依赖 `theme/`，无反向依赖 ✅
- `widgets/` -> 依赖 `primitives/` + `theme/`，无反向依赖 ✅
- `screens/` -> 依赖 `widgets/` + `primitives/` + `theme/` ✅
- `application/` -> 独立层，只依赖 `navigation/` ✅
- `projections/` -> 依赖 `view_models/` + `binding/` ✅

但是存在目录结构偏移:
- **设计文档要求** (`widget-composites.md` §7): Widget 应放在 `widgets/composites/molecules/` 和 `widgets/composites/organisms/` 下
- **代码实际**: 所有 Widget 平铺在 `widgets/` 下 (`widgets/action_menu/`, `widgets/skill_slot/`, `widgets/character_card/` 等)，没有 molecule/organism 子目录

### 2.3 主题系统

**状态: ⚠️ 部分合规**

积极点:
- `Theme` Resource 正确定义，包含 `colors`, `spacing`, `typography` 三个令牌子结构
- `UiColors` 提供 `dark()` 和 `light()` 两个调色板
- 按钮工厂 `button_background_color()` 从 `theme.colors` 读取颜色，正确使用令牌
- Theme 切换功能 (`switch_theme`) 实现，通过 `ThemeVariant::Dark` / `Light` 切换

违规点:
- **`src/ui/theme/colors.rs`**: 50 个 `Color::srgb()` 调用 -- 但这属于**可接受的集中定义**。colors.rs 就是主题定义所在，不应过度标记为违规。问题在于设计文档要求 4 个主题变体 (Dark/Light/Pixel/HD2D)，代码只实现了 2 个
- **`src/ui/screens/save_load/mod.rs:287,308`**: 硬编码 `Color::srgba(0.3, 0.3, 0.3, 1.0)` 和 `Color::srgba(0.2, 0.2, 0.2, 1.0)` 在 Screen 生产代码中，绕过 Theme Resource
- **设计文档要求** (`theme-localization.md` §3.3): Theme 配置文件应放在 `assets/config/ui/themes/` 下。当前代码中 Theme 通过 Rust 代码内建 (Theme::dark()/Theme::light())，没有独立的 RON 配置文件
- 所有 Theme 实例共享同一 `UiSpacing::default_scale()` 和 `UiTypography::default_values()` -- Light 主题理论上可能有不同的间距/字体设置，但当前完全复用

### 2.4 本地化优先

**状态: ⚠️ 部分合规**

积极点:
- `localization/text_keys.rs` 正确定义，从 `infra::localization::generated::loc` 重导出 UI 相关 key
- MainMenuScreen 使用 `spawn_localized_button(commands, &theme, loc::ui::NEW_GAME, "New Game", ...)` -- key 和 fallback 文本同时提供
- SettingsScreen 使用 `loc::ui::SETTINGS`, `loc::ui::CLOSE`, `loc::ui::SAVE`
- BattleScreen 使用 `loc::ui::BATTLE_END_TURN`
- SaveLoadScreen 使用 `loc::ui::SAVE_LOAD_TITLE_SAVE`, `loc::ui::SAVE_LOAD_EMPTY` 等

违规点:
- **`src/ui/primitives/button/factory.rs:115-125`**: `spawn_button` 使用 `Text::new(label_str)` 而非 `LocalizedText` -- 但这是原始按钮工厂，`spawn_localized_button` 则正确使用 `LocalizedText::static_text(key)`
- **`src/ui/primitives/text/factory.rs`**: `spawn_text` 使用 `Text::new(content_str.clone())` -- 作为 Primitives 层的纯文字工厂，这是可接受的 (它不绑定 LocalizationKey)，但需要调用方负责本地化
- **`src/ui/primitives/progress_bar/factory.rs`**: 使用 `Text::new(format!("{}", value))` -- 进度数值使用纯 `Text::new`，未考虑格式化
- **`src/ui/primitives/select_list/factory.rs`**: 使用 `Text::new(label_text)` 创建选项标签
- **`src/ui/primitives/modal/factory.rs`**: 使用 `Text::new(title_str)` 和 `Text::new(message_str)` -- 虽然工厂接收字符串，但调用方应传入本地化后的字符串
- **总计 18 处 `Text::new` 调用** -- 虽然多数在 Primitives 层是合理的 (Primitives 不感知本地化)，但违反设计文档中 "所有文本必须通过 LocalizedText" 的绝对断言

### 2.5 Spec 完整性

**状态: ⚠️ 部分合规**

- 6 个 Screen Spec 文件全部存在 ✅
- 4 个 Reference 文件全部存在 ✅
- 但 `widget-id-map.md` 中的 100+ widget_id 实际代码中**完全没有使用** (代码中 0 处引用 widget_id 命名规则)
- UiBinding 虽然正确定义了 20+ 变体，但实际代码中**没有任何 Widget 实体带有 UiBinding 组件**

---

## 3. Screen Spec 一致性

### 3.1 MainMenuScreen

| Spec 需求 | 代码状态 | 一致性 |
|-----------|---------|--------|
| Title "Fre", 48px, Title variant | `src/ui/screens/main_menu/mod.rs:64-79` ✅ | 一致 |
| Subtitle "A Bevy SRPG", Caption | `src/ui/screens/main_menu/mod.rs:82-89` ✅ | 一致 |
| List(Vertical) with 3 buttons | `src/ui/screens/main_menu/mod.rs:98-108` ✅ | 一致 |
| NewGameButton [PrimaryButton] | `src/ui/screens/main_menu/mod.rs:101` ✅ | 一致 |
| LoadGameButton [SecondaryButton] | `src/ui/screens/main_menu/mod.rs:106` ✅ | 一致 |
| SettingsButton [SecondaryButton] | `src/ui/screens/main_menu/mod.rs:111` ✅ | 一致 |
| Version text "v0.1.0", Caption | `src/ui/screens/main_menu/mod.rs:118` ✅ | 一致 |
| MenuAction::NewGame/LoadGame/Settings | `src/ui/screens/main_menu/mod.rs:114-116` ✅ | 一致 |
| OnExit(MainMenu) despawn | `screen_plugin.rs` `add_systems(OnExit(GameState::MainMenu), despawn_main_menu)` ✅ | 一致 |

**结论: ✅ 完全一致**

### 3.2 BattleScreen

| Spec 需求 | 代码状态 | 一致性 |
|-----------|---------|--------|
| 9-zone absolute positioning | `src/ui/screens/battle/layout.rs` ✅ | 一致 |
| BattleScreen marker component | `src/ui/screens/battle/mod.rs:24` ✅ | 一致 |
| TurnInfoBar (Z1) | `src/ui/screens/battle/mod.rs:57-63` ✅ | 一致 |
| PhaseText + TurnNumber (Z2) | `src/ui/screens/battle/mod.rs:71` `// TODO[P2][UI]` | ❌ 未实现 |
| UnitSummary (Z3) | `src/ui/screens/battle/mod.rs:76` `// TODO[P2][UI]` | ❌ 未实现 |
| CharacterCard (Z5) | `src/ui/screens/battle/mod.rs:82-92` ✅ | 一致 (硬编码数据) |
| ActionMenu (Z6) | `src/ui/screens/battle/mod.rs:95-98` ✅ | 一致 |
| SkillPanel (Z7) | `src/ui/screens/battle/mod.rs:103` `// TODO[P1][UI]` | ❌ 未实现 (P1) |
| TurnOrderBar (Z8) | `src/ui/screens/battle/mod.rs:119` `// TODO[P2][UI]` | ❌ 未实现 |
| EndTurnButton with BattleAction | `src/ui/screens/battle/mod.rs:106-115` ✅ | 一致 |
| Despawn on OnExit(Combat) | `src/ui/screens/battle/mod.rs:127-131` ✅ | 一致 |
| UiBinding::Hp/Mp/Turn/Phase on Widgets | `widget-id-map.md` 定义的绑定 | ❌ 代码中 0 处使用 UiBinding |
| ViewModel 驱动的 HpBar/MpBar | `screens.md §2.3` 描述 | ❌ CharacterCard 使用硬编码数值 |

**结论: ⚠️ 骨架一致，核心内容缺失 (Z2, Z3, Z7 SkillPanel, Z8 TurnOrderBar, 无 UiBinding, 硬编码角色数据)**

### 3.3 SettingsScreen

| Spec 需求 | 代码状态 | 一致性 |
|-----------|---------|--------|
| 4-tab TabPanel (Gameplay/Graphics/Audio/Battle) | `src/ui/screens/settings/mod.rs` | ❌ 无 TabPanel |
| GameplayTab: 4 Toggles | 仅 2 个 Toggle (ShowDamage, DarkTheme) | ❌ 缺失 Minimap/Grid/AutoBattle |
| GraphicsTab: ThemeSelector + LanguageSelector | 未实现 | ❌ 缺失 |
| AudioTab: MasterVolume + BgmVolume + SfxVolume | 未实现 | ❌ 缺失 |
| BattleTab: BattleSpeed + TooltipDelay | 未实现 | ❌ 缺失 |
| ResetButton (DangerButton) | 未实现 | ❌ 缺失 |
| TabPanel atom | 代码中无 TabPanel 使用 | ❌ 缺失 |
| SelectList atom | 代码中无 SelectList 使用 | ❌ 缺失 |
| ProgressBar (slider variant) | 代码中无 ProgressBar 在 Settings | ❌ 缺失 |
| Close Button | `settings/mod.rs:151-157` ✅ | 一致 |
| Save Button | `settings/mod.rs:160-171` ✅ | 一致 |
| 即时主题切换 | `src/ui/screens/settings/mod.rs:224-230` ✅ | 一致 |

**结论: ❌ 严重不一致。实现约 15% 的 Spec 需求，仅包含 2 个 Toggle + 2 个 Button，与 Spec 的 4 Tab + 16 项设置差距极大**

### 3.4 InventoryScreen

| Spec 需求 | 代码状态 | 一致性 |
|-----------|---------|--------|
| Full-screen Panel | `src/ui/screens/inventory/mod.rs:36-48` ✅ | 一致 |
| InventoryGrid Organism | `src/ui/screens/inventory/mod.rs:51` ✅ | 一致 |
| Title "Inventory" (Heading, localized) | `widgets/inventory_grid/factory.rs` | ⚠️ 硬编码 "Gold: 100" |
| Gold display | 同上 | ❌ 硬编码值 |
| 4 InventoryItemRow examples | `widgets/inventory_grid/factory.rs` ✅ | 一致 (静态样本) |
| Close button | ✅ | 一致 |
| FilterBar, SearchBox, SortDropdown | 未实现 | ❌ 缺失 |
| ItemDetailPanel | 未实现 | ❌ 缺失 |
| ViewModel binding | 代码中无 UiStore 关联 | ❌ 缺失 |
| Dirty<T> consumption | 代码中无 Dirty 检测 | ❌ 缺失 |

**结论: ⚠️ 骨架完成，功能严重欠奉。仅有静态样本数据，无 ViewModel/Projection/Dirty 集成**

### 3.5 ShopScreen

| Spec 需求 | 代码状态 | 一致性 |
|-----------|---------|--------|
| ShopScreen marker | `src/ui/screens/shop/mod.rs:40` ✅ | 一致 |
| ShopPanel Organism | `src/ui/screens/shop/mod.rs:63` ✅ | 一致 |
| ShopItemCard molecules | `widgets/shop_item_card/` ✅ | 一致 |
| InventoryItemRow for sell tab | `widgets/shop_item_card/` (shop_panel 内部) | ⚠️ 存在但无数据 |
| TabPanel (Buy/Sell) | `spawn_shop_panel` 使用 TabPanel | ⚠️ 硬编码样本数据 |
| ShopPlugin with Observer | `src/ui/screens/shop/mod.rs:82-97` ✅ | 一致 |
| EconomyProjection | `src/ui/projections/economy.rs` | ❌ 纯 skeleton，不可用 |
| BuyItem/SellItem command mapping | `src/ui/screens/shop/mod.rs:109-127` ✅ | 一致 (但缺少真实 ViewModel) |
| ViewModel/PlayerGold | 代码中无 UiStore 字段 | ❌ 缺失 |

**结论: ⚠️ UI 骨架完成，数据后勤缺失。ShopItemCard 和 ShopPanel Widget 已实现，但 EconomyProjection 和 ViewModel 是空壳**

### 3.6 SaveLoadScreen

| Spec 需求 | 代码状态 | 一致性 |
|-----------|---------|--------|
| 10 save slots | `src/ui/screens/save_load/mod.rs:169-212` ✅ | 一致 |
| Slot selection (SelectedSlot Resource) | `components.rs:50-55` ✅ | 一致 |
| Save/Load mode toggle | `components.rs:28-36` ✅ | 一致 |
| Preview panel with placeholders | `mod.rs:217-283` ✅ | 一致 (全部占位) |
| Confirm, Delete buttons | `mod.rs:287-329` ✅ | 一致 |
| Close button | `mod.rs:151-160` ✅ | 一致 |
| Command mapping (SaveGame/LoadGame) | `systems.rs:82-89` ✅ | 一致 |
| SaveSlotVm with real metadata | `components.rs:59-68` `// TODO[P2]` | ❌ 全部 "Empty" |
| ModalOverlay for confirm | 未实现 | ❌ 缺失 |
| LoadingOverlay during save/load | 未实现 | ❌ 缺失 |
| UiCommand::DeleteSlot variant | `systems.rs:98` `// TODO[P2]` | ❌ 缺失 |

**结论: ⚠️ UI 骨架完成度最高 (所有按钮和布局元素实现)，但数据为空。Avatar/Screenshot placeholder 使用了原始 `Color::srgba()` 硬编码 (P0 违规)**

---

## 4. 代码健康度问题

### 4.1 空目录 / 未实现 Widget

设计文档 `widget-composites.md` 定义了 16 个复合组件 (8 Molecule + 8 Organism)，当前代码实现 7 个:

| 复合组件 | 类型 | 目录 | 状态 |
|---------|------|------|------|
| SkillSlot | Molecule | `widgets/skill_slot/` | ✅ 已实现 (4 files) |
| ActionMenu | Molecule | `widgets/action_menu/` | ✅ 已实现 (4 files) |
| CharacterCard | Molecule | `widgets/character_card/` | ✅ 已实现 (4 files) |
| InventoryItemRow | Molecule | `widgets/inventory_item_row/` | ✅ 已实现 (4 files) |
| ShopItemCard | Molecule | `widgets/shop_item_card/` | ✅ 已实现 (3 files) |
| InventoryGrid | Organism | `widgets/inventory_grid/` | ✅ 已实现 (3 files) |
| ShopPanel | Organism | `widgets/shop_panel/` | ✅ 已实现 (3 files) |
| BuffIcon | Molecule | `widgets/buff_icon/` | ⚠️ 有目录但内部仅 3 个文件 (components/factory/mod)，功能有限 |
| CharacterPortrait | Molecule | `widgets/` 下无对应目录 | ❌ 未创建 |
| TurnIndicator | Molecule | `widgets/` 下无对应目录 | ❌ 未创建 |
| QuestEntry | Molecule | `widgets/` 下无对应目录 | ❌ 未创建 |
| DialogueChoice | Molecule | `widgets/` 下无对应目录 | ❌ 未创建 |
| CharacterStatusPanel | Organism | `widgets/` 下无对应目录 | ❌ 未创建 |
| BattleHud | Organism | `widgets/` 下无对应目录 | ❌ 未创建 |
| TurnOrderBar | Organism | `widgets/` 下无对应目录 | ❌ 未创建 |
| QuestLog | Organism | `widgets/` 下无对应目录 | ❌ 未创建 |
| DialoguePanel | Organism | `widgets/` 下无对应目录 | ❌ 未创建 |

### 4.2 TODO 分布

共 20 个 TODO，分布于以下文件:

| 文件 | 数量 | 类型 |
|------|------|------|
| `projections/battle.rs` | 6 | P2 Projection: CharacterPanelVm 填充, DamageDealt 匹配, UnitDied 处理 |
| `projections/economy.rs` | 4 | P3 Economy: 全部为骨架 (EconomyVm, 领域事件等待) |
| `screens/battle/mod.rs` | 4 | P1-P2 BattleScreen: SkillPanel, TurnOrderBar, PhaseText, UnitSummary |
| `screens/battle/systems.rs` | 1 | P1 unit selection state |
| `screens/battle/layout.rs` | 1 | P2 Z2TopCenter horizontal centering |
| `screens/save_load/systems.rs` | 1 | P2 DeleteSlot UiCommand 变体 |
| `screens/save_load/components.rs` | 1 | P2 SaveSlotVm 真实数据 |
| `screens/inventory/mod.rs` | 1 | P3 "Gold: 100" ViewModel 绑定 |
| `primitives/button/tests/unit/button_test.rs` | 1 | 测试文件 |

按严重程度: P1: 2 个, P2: 11 个, P3: 4 个, 未标记: 3 个

### 4.3 Dead Code

- **UiBinding 枚举** (`binding/ui_binding.rs`): 20+ 变体完全定义，但**没有任何 Widget 实体携带 `UiBinding` 组件**。这是半个未连接的设计模式 -- 定义已存在但消费端缺失
- **`screens/settings/mod.rs` 的 `settings_toggle_system`**: 使用 `Changed<ToggleState>` 检测变更 -- 这个 Update 系统运行但实际 Toggle 状态变更通过 ButtonClicked Observer 传播，两者之间存在冗余
- **`navigation/screen_stack.rs`**: ScreenStack 完整实现 (push/pop/replace/peek)，但实际 Screen 导航通过 `OnEnter/OnExit` + Observer 直接管理，ScreenStack 未被用作导航决策的中央源
- **`navigation/screen_state.rs`**: `UiScreenState` 和 `ScreenLifecycle` 定义了完整的状态机，但**代码中没有任何地方设置或读取 `ScreenLifecycle` 状态** -- 状态机定义存在但无人驱动

### 4.4 Bevy API 兼容性

代码正确使用了 Bevy 0.19 API:

- **Observer**: `add_observer()`, `On<Event>`, `trigger()` 在 `screen_plugin.rs`, `settings/mod.rs`, `shop/mod.rs` 等广泛使用 ✅
- **OnEnter/OnExit**: ScreenPlugin 正确使用 `OnEnter(GameState::Combat)` 和 `OnExit(GameState::Combat)` ✅
- **`Commands::trigger`**: 在 `shop/mod.rs:130` 等用于触发 UiCommand ✅
- **`TextFont` + `FontSize`**: 使用 Bevy 0.19 新 API (如 `main_menu/mod.rs:74`) ✅
- **BSN 禁止**: `src/ui/` 下无 `bsn!{}` 宏使用 ✅

---

## 5. 具体问题清单

### P0 严重违规

| # | 文件 | 行号 | 问题描述 | 修复建议 |
|---|------|------|---------|---------|
| P0-1 | `src/ui/screens/save_load/mod.rs` | 286-291 | 直接 `commands.spawn(Node{...}, BackgroundColor(Color::srgba(...)))` 创建 avatar placeholder，绕过 Primitives 工厂。违反架构铁律 5 | 创建 Panel 工厂变体支持占位元素，或使用 `spawn_panel` + 自定义样式 |
| P0-2 | `src/ui/screens/save_load/mod.rs` | 307-315 | 同样模式创建 screenshot placeholder，硬编码 `Color::srgba(0.2, 0.2, 0.2, 1.0)` | 同上。在 Primitives 层添加 PanelVariant::Placeholder 变体 |
| P0-3 | 全部 Screens | -- | 没有任何 Widget 使用 `UiBinding` 或消费 `Dirty<T>`。ViewModel 数据流断链 | 在 Screen spawn 函数中为数据驱动的 Widget 添加 `UiBinding::Hp` 等组件，创建 `on_dirty` System 消费 `Dirty<BattleHudVm>` |
| P0-4 | `src/ui/screens/battle/mod.rs` | 82-92 | `spawn_character_card` 传入硬编码数值 ("Aria", 5, 80.0, 100.0...)，不读取 UiStore | 从 `UiStore.battle_hud` 读取 HP/MP/AP 值，移除 hardcoded 工厂参数 |

### P1 重要违规

| # | 文件 | 行号 | 问题描述 | 修复建议 |
|---|------|------|---------|---------|
| P1-1 | `src/ui/primitives/button/factory.rs` | 115 | `spawn_button` 使用 `Text::new(label_str)` -- 非本地化 | 接受 local key 参数，添加 fallback label |
| P1-2 | `src/ui/primitives/modal/factory.rs` | 4460, 4471 | `Text::new(title_str)` / `Text::new(message_str)` 非本地化 | 接受 UiTextKey 参数 |
| P1-3 | `src/ui/primitives/progress_bar/factory.rs` | 2914 | `Text::new(format!("{}", value))` | 使用 LocalizedText 或格式化的文本组件 |
| P1-4 | `src/ui/primitives/select_list/factory.rs` | 5136 | `Text::new(label_text)` | 使用 `spawn_localized_text` |
| P1-5 | `src/ui/primitives/text/factory.rs` | 5827, 5882 | `spawn_text` 使用 `Text::new` -- 这是纯文本工厂，但文档称"所有文本必须通过 LocalizedText" | 文档过于绝对。建议更新架构文档明确 Primitives 层可以操作 Text::new，只要 Widget/Screen 层使用走 LocalizationKey |
| P1-6 | `src/ui/screens/settings/mod.rs` | -- | Spec 要求 4 个 Tab + 16 项设置，实现仅 2 个 Toggle + 2 个 Button | 按优先级逐步实现 Gameplay 和 Graphics Tab 的设置项 |
| P1-7 | `src/ui/screens/battle/mod.rs` | 103 | `// TODO[P1]` SkillPanel 缺失 -- 这是核心战斗交互 | 集成 `widgets/skill_slot` 到 BattleScreen Z7 区域，创建 SkillPanel Organism |
| P1-8 | `src/ui/screens/battle/mod.rs` | 65-67 | 回合信息文本使用硬编码 fallback "-- / ----"，未从 ViewModel 读取 | 使用 `battle_turn_indicator` 和 `battle_phase_label` widget_id + UiBinding |

### P2 中等违规

| # | 文件 | 行号 | 问题描述 | 修复建议 |
|---|------|------|---------|---------|
| P2-1 | `src/ui/navigation/screen_stack.rs` | 全部 | ScreenStack 实现了但未用于实际导航 | 迁移 Screen 生命周期管理到 ScreenStack: `push()` 驱动 spawn + OnEnter，`pop()` 驱动 despawn + OnExit |
| P2-2 | `src/ui/navigation/screen_state.rs` | 全部 | `UiScreenState` 的 `ScreenLifecycle` 状态机定义了但无人驱动 | 在 ScreenStack 操作中更新 `UiScreenState.lifecycle` |
| P2-3 | `src/ui/binding/ui_binding.rs` | 全部 | UiBinding 完全定义但未在代码中引用 | 为每个数据驱动的 Widget 实体添加 `UiBinding::Hp` / `UiBinding::Turn` 等组件 |
| P2-4 | `src/ui/widgets/` | -- | 缺少 9 个复合组件 (CharacterPortrait, TurnIndicator, CharacterStatusPanel, BattleHud, TurnOrderBar 等) | 按优先级实现 BattleHud 和 CharacterStatusPanel (与 BattleScreen Z5 角色面板集成) |
| P2-5 | `src/ui/widgets/` 目录结构 | -- | 设计文档要求 `widgets/composites/molecules/` 子目录，代码为扁平结构 | 对现有 Widget 迁移到 molecule/organism 子目录 |
| P2-6 | `src/ui/screens/battle/mod.rs` | 119 | TurnOrderBar 缺失 | 实现 TurnOrderBar 复合组件 |
| P2-7 | `src/ui/screens/battle/layout.rs` | 36 | Z2TopCenter 的 `left: Val::Auto` -- TODO 标记未水平居中 | 通过父 Flexbox 或 `transform` 实现水平居中 |
| P2-8 | `src/ui/view_models/mod.rs` | -- | UiStore 只包含 3 个字段 (battle_hud, character_panel, skill_panel)，缺少 inventory, shop, quest 等 | 添加 InventoryVm, ShopPanelVm 等字段 |
| P2-9 | `src/ui/projections/economy.rs` | 全部 | EconomyProjection 是纯骨架 (4 个 TODO)，无 Observer 注册 | 等待 EconomyVm + 领域事件定义完成后实现 |

### P3 轻微

| # | 文件 | 行号 | 问题描述 |
|---|------|------|---------|
| P3-1 | `src/ui/theme/colors.rs` | 全部 | Theme 只有 Dark/Light 两个变体，设计文档要求 Dark/Light/Pixel/HD2D |
| P3-2 | `src/ui/theme/resource.rs` | 全部 | Theme 配置通过 Rust 代码内建，设计文档要求独立的 RON 配置文件 (`assets/config/ui/themes/*.ron`) |
| P3-3 | `src/ui/settings.rs` | 全部 | `save_settings` 和 `load_settings` 使用 `std::fs::read_to_string("ui_settings.ron")` 直接文件 I/O，设计文档要求 Bevy 0.19 `SettingsGroup` |
| P3-4 | `src/ui/primitives/button/factory.rs` | 全部 | `button_background_color` 和 `button_text_color` 对 ButtonVariant 匹配使用了部分重复逻辑 -- 如果新增变体需要更新多处 |
| P3-5 | `src/ui/screens/save_load/components.rs` | 59-68 | `SaveSlotVm` 结构体标记了 TODO 但从未实际使用 -- 数据流完全未连接 |

---

## 6. 改进建议

### 6.1 立即修复 (P0)

1. **SaveLoadScreen 硬编码绕过**: 为 panel 工厂添加 `PanelVariant::Placeholder` 支持颜色/尺寸参数，或创建专用工厂 `spawn_placeholder`。移除所有直接 `commands.spawn(Node{...}, BackgroundColor(...))` 模式。

2. **激活 UiBinding + Dirty<T> 数据链**:
   - 在 BattleScreen spawn 时，为 HP/MP/AP 条添加 `UiBinding::Hp` / `UiBinding::Mp` / `UiBinding::Ap` 组件
   - 创建 `on_dirty_battle_hud` System 消费 `Dirty<BattleHudVm>.consume()`
   - 将 CharacterCard 从硬编码参数迁移到读取 `Res<UiStore>.battle_hud`

3. **BattleScreen 硬编码数据**:
   - 从 `UiStore.battle_hud` 读取 HP/MP/AP 显示值
   - 创建 BattleHud Organism 作为 CharacterCard 的父级，管理角色状态显示

### 6.2 短期内修复 (P1)

4. **SettingsScreen 扩展至 TabPanel**:
   - 实现 TabPanel Primitives 连接 SettingsScreen 的 Spec
   - 至少完成 GameplayTab (4 Toggles) 和 GraphicsTab (Theme/Language selector)

5. **BattleScreen SkillPanel 集成**:
   - 将 `widgets/skill_slot` 集成到 BattleScreen Z7 区域
   - 创建 SkillPanel Organism (TabPanel + ScrollPanel + SkillSlot 列表)
   - 从 `UiStore.skill_panel` 提供技能数据

6. **本地化全面审计**:
   - 扫描 18 处 `Text::new`，评估哪些可以迁移到 `spawn_localized_text`
   - Primitives 层的 Text::new 应保留 (文档需更新明确此豁免)，但 Screen 层不应有硬编码文本

7. **SaveLoadScreen 数据集成**:
   - 将 `SaveSlotVm` 数据挂载到真实存档元数据
   - 实现 `UiCommand::DeleteSlot` 变体

### 6.3 中期修复 (P2)

8. **激活 ScreenStack 导航**:
   - 将 Screen 生命周期管理从直连 OnEnter/OnExit 迁移到 ScreenStack::push/pop
   - ScreenStack::push 触发 OnEnter，pop 触发 OnExit
   - 更新 UiScreenState.lifecycle 跟踪状态机转换

9. **实现缺失的复合组件**:
   - 优先级: BattleHud > CharacterStatusPanel > TurnOrderBar > CharacterPortrait > BuffIcon
   - 低优先级: QuestLog > QuestEntry > DialoguePanel > DialogueChoice

10. **UiStore 扩展**:
    - 添加 InventoryVm, ShopPanelVm, EconomyVm 到 UiStore
    - 创建对应的 Projection 骨架 (inventory.rs, shop.rs)

11. **UiBinding 实际绑定**:
    - 在所有数据驱动的 Screen Widget 上添加 UiBinding 组件
    - 创建消费 Dirty<T> 的 Update systems

### 6.4 长期规划 (P3)

12. **Theme RON 配置文件**: 将 Theme 定义从 Rust 代码迁移到 `assets/config/ui/themes/` RON 文件，支持热重载

13. **Bevy SettingsGroup 集成**: 迁移 `UiSettings` 持久化从 `std::fs` 到 Bevy 0.19 的 `SettingsGroup` trait

14. **新增 Theme 变体**: 实现 Pixel 和 HD2D 主题变体 (需要独特的美术资源)

15. **Widget ID 真实使用**: 在代码中集成 `widget-id-map.md` 的命名规则，用于调试和自动化测试

### 6.5 文档修正

| 文件 | 问题 | 修复建议 |
|------|------|---------|
| `architecture.md` §6.5 铁律 5 | "Widgets 和 Screens 禁止直接 import Bevy UI 原语" -- SaveLoadScreen 已违反 | 添加豁免条款: UI Root 层创建允许直接 spawn |
| `theme-localization.md` §4.4 | "Widget 代码不需要手动调用翻译函数" -- Primitives 层使用 Text::new 与此矛盾 | 明确 Primitives 层允许 Text::new，Widget/Screen 层必须用 LocalizedText |
| `widget-composites.md` §7 | 目录结构要求 `widgets/composites/molecules/` -- 代码未遵循 | 对齐到实际扁平目录，或在架构上强制要求分离 |

---

## 7. 修复状态跟踪

> 根据 2026-06-22 评审结果安排 3 轮 8 个 agent 修复。更新人: @Claude Code

### 已修复 (8/15 项)

| # | 问题 | 文件 | 修复内容 | 状态 |
|---|------|------|---------|------|
| P0-1 | SaveLoadScreen 直接 commands.spawn | `primitives/panel/`, `screens/save_load/` | 新增 `PanelVariant::Placeholder` 变体，两处占位改用工厂 | ✅ 已修复 |
| P0-2 | SaveLoadScreen 硬编码 Color::srgba | `screens/save_load/mod.rs` | 同上，颜色改由 Placeholder 变体携带 | ✅ 已修复 |
| P0-3 | UiBinding 未使用 | `screens/battle/mod.rs` | CharacterCard 实体携带 UiBinding::Hp/Mp | ✅ 部分修复 |
| P0-4 | BattleScreen 硬编码角色数据 | `screens/battle/`, `view_models/` | 创建 BattleHudData Resource 替代硬编码 | ✅ 已修复 |
| P1-5 | SkillPanel 缺失 (Z7) | `widgets/skill_panel/`, `screens/battle/` | 新建 skill_panel widget，集成 skill_slot 到 Z7 | ✅ 已修复 |
| P1-6 | SettingsScreen 过于简化 | `screens/settings/`, `ui/settings.rs` | 从 2 Toggle 扩展到 5 Toggle，分 Gameplay/Display 组 | ✅ 部分修复 |
| P1-7 | Z2 PhaseText + TurnNumber 缺失 | `screens/battle/`, `localization/`, FTL 文件 | 添加阶段文本和回合数，水平居中，三语言 FTL | ✅ 已修复 |
| P2-7 | Z2TopCenter 未水平居中 | `screens/battle/layout.rs` | 添加 justify_content/align_items 居中 | ✅ 已修复 |
| P2-8 | UiStore 字段不足 | `view_models/`, `plugin.rs` | 新增 InventoryVm/ShopPanelVm/EconomyVm | ✅ 已修复 |
| P2-9 | EconomyProjection 骨架 | `projections/mod.rs` | 添加 inline skeleton module | ✅ 部分修复 |

### 待修复 (7 项)

| # | 问题 | 计划安排 | 优先级 |
|---|------|---------|--------|
| P0-3 | Dirty<T> 消费链完整接入 | 需 Projection 系统就绪后激活 on_dirty_battle_hud | P0 |
| P1-1~4 | 本地化 Text::new 审计 | Primitives 层可接受，需更新架构文档明确豁免 | P1 |
| P2-1/2 | ScreenStack/ScreenLifecycle 未激活 | 架构决策：是否迁移到 ScreenStack 管理 | P2 |
| P2-4 | 9 个缺失复合组件 | 按优先级实现 BattleHud/TurnOrderBar/CharacterStatusPanel | P2 |
| P2-6 | TurnOrderBar (Z8) | 已安排 agent 实现 | P2 |
| P2-9 | EconomyProjection 领域事件 | 等待 Economy 领域事件定义 | P2 |
| P3-1~3 | Theme RON/变体/SettingsGroup | 长期规划 | P3 |

### 文档修正状态

| 文件 | 问题 | 状态 |
|------|------|------|
| `architecture.md` §6.5 铁律 5 | 需添加豁免条款 | ⏳ 待处理 |
| `theme-localization.md` §4.4 | 明确 Primitives Text::new 豁免 | ⏳ 待处理 |
| `widget-composites.md` §7 | 目录结构对齐 | ⏳ 待处理 |

---

*本报告由 @presentation-architect 生成。修复状态由 @Claude Code 跟踪更新。*
