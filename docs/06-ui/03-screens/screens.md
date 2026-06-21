---
id: 06-ui.screens
title: Screen Design — 页面详细设计
status: partially-implemented
owner: presentation-architect
created: 2026-06-20
updated: 2026-06-21
tags:
  - ui
  - screen
  - battle
  - menu
  - inventory
  - shop
  - settings
  - save-load
---

# Screen Design — 页面详细设计

> **实现状态**: BattleScreen MVP ✅ / MainMenuScreen MVP ✅ / InventoryScreen MVP ✅ / ShopScreen ❌ / SettingsScreen ❌ / SaveLoadScreen ❌

> **职责**: @presentation-architect | **上游**: ADR-055 §5.5 (Screen 组合), §9 (GameState 映射) | domain rules §2.1 (Screen 状态机) | schema §7 (Navigation) | screen-lifecycle.md §2 (Screen 架构), §4 (映射表) | widget-composites.md §3 (Organism 定义)

---

## 1. 设计目的

Screen 是页面级 Widget 容器，与 GameState/OverlayState 一一对应。本文档定义每个 Screen 的：
- 对应的 GameState/OverlayState
- 消费的 ViewModel
- 组合的 Widget（引用 widget-atoms.md 中的原子组件契约 和 widget-composites.md 中的复合组件契约）
- 发射的 UiCommand
- 生命周期特殊行为
- Overlay 依赖

Screen 设计遵循以下原则：
- Screen 只做 Widget 组合，不直接拼 Node（INV-UI-005）
- Screen 不直接引用另一个 Screen，通信通过 ScreenStack（R-SCR-03）
- Screen 使用 WidgetFactory 组合 Widget（R-SCR-02）

---

## 2. BattleScreen

### 2.1 基本信息

| 属性 | 值 |
|------|------|
| 对应 GameState | GameState::Combat |
| ScreenLayer 层级 | 0（主界面层） |
| 加载模式 | Persistent（非 Ephemeral，战斗内 Widget 通过 Visibility 切换） |
| 过渡动画 | Fade(0.3s) |

### 2.2 消费的 ViewModel

| ViewModel | 用途 | 必选/可选 |
|-----------|------|-----------|
| BattleHudVm | HP/MP 条、回合数、行动点 | 必选（MVP 使用硬编码初始值） |
| SkillPanelVm | 技能面板、冷却、AP | 必选（MVP 未接入，SkillPanel 使用静态数据） |
| CharacterPanelVm | 角色信息面板 | 必选（MVP 使用 `spawn_character_card` 传入硬编码值） |

**MVP 过渡说明**：BattleScreen 当前通过工厂函数直接传入数值（如 `spawn_character_card(..., "Aria", 5, 80.0, 100.0, 40.0, 50.0)`），
尚未接入 UiStore 的 Dirty<T> 刷新机制。ViewModel 集成将在后续迭代完成。

### 2.3 组合的 Widget

BattleScreen 组合 5 个主要部分：

```
BattleScreen
├── TurnInfoBar [BodyText]                           — 回合信息文本（硬编码："Turn: 3    Phase: Player Turn"）
├── BattleArea [Panel — 占位区域]                     — 战斗场地占位（300px 高，后续接入 Tactical Domain）
├── CharacterCard [CharacterCard — Organism]         — 角色卡片（硬编码：Aria, Lv.5, 80/100 HP, 40/50 MP）
├── ActionMenu [ActionMenu — Organism]               — 行动菜单（Attack, Defend, Skill, Item, Wait）
└── EndTurnButton [Button, Danger variant]           — 结束回合按钮（携带 BattleAction::EndTurn 标记）
```

**当前实现限制**（MVP）：
- 回合信息文本为静态硬编码，未接入 BattleHudVm.turn_number / phase_key
- BattleArea 为空白占位区域
- CharacterCard 通过工厂参数传入硬编码角色数据
- ActionMenu 按钮点击处理待实现

### 2.4 发射的 UiCommand

| UiCommand | 触发 Widget | 条件 |
|-----------|------------|------|
| CastSkill(u32, u32) | ActionMenu → Skill/ActionMenu | 点击可用技能（待接线） |
| SelectTarget(u32) | BattleArea | 选择目标后（待接线） |
| EndTurn | EndTurnButton | 点击结束回合（携带 BattleAction::EndTurn 标记，待接线） |

### 2.5 生命周期特殊行为

| 事件 | 行为 |
|------|------|
| OnEnter | `spawn_battle_screen()` — 生成 UI 树全屏 Panel → TurnInfo / BattleArea / CharacterCard / ActionMenu / EndTurnButton |
| Active | MVP 无 ViewModel 刷新逻辑（数值硬编码在工厂参数中） |
| OnExit | `despawn_battle_screen()` — 通过 `With<BattleScreen>` 查询清理所有实体 |

### 2.6 Overlay 依赖

当前 MVP 无 Overlay 依赖。后续迭代补充 DamageTextOverlay、TooltipOverlay、NotificationOverlay、ModalOverlay。

### 2.7 变体

当前 MVP 只有一个标准布局。后续迭代补充 BossBattle、AutoBattle 等变体。

---

## 3. MainMenuScreen

### 3.1 基本信息

| 属性 | 值 |
|------|------|
| 对应 GameState | GameState::MainMenu |
| ScreenLayer 层级 | 0（主界面层） |
| 加载模式 | Ephemeral（每次进入 MainMenu 重新 spawn） |
| 过渡动画 | Fade(0.5s) |

### 3.2 消费的 ViewModel

| ViewModel | 用途 | 必选/可选 |
|-----------|------|-----------|
| 无 | MainMenu 不消费业务 ViewModel | — |

### 3.3 组合的 Widget

```
MainMenuScreen
├── Title [Text — "Fre", Title variant, 48px]
├── Subtitle [Text — "A Bevy SRPG", Caption variant]
├── List [List, Vertical, 200px]
│   ├── NewGameButton [PrimaryButton]    — MenuAction::NewGame
│   ├── LoadGameButton [SecondaryButton] — MenuAction::LoadGame
│   └── SettingsButton [SecondaryButton] — MenuAction::Settings
└── Version [Text — "v0.1.0", Caption variant]
```

### 3.4 发射的 UiCommand

| UiCommand | 触发 Widget | 说明 |
|-----------|------------|------|
| OpenScreen(ScreenType::Settings) | SettingsButton (MenuAction::Settings) | 打开设置界面（待接线） |
| NewGame | NewGameButton (MenuAction::NewGame) | 开始新游戏（待接线） |
| OpenScreen(ScreenType::SaveLoad) | LoadGameButton (MenuAction::LoadGame) | 加载游戏（待接线） |

### 3.5 生命周期特殊行为

| 事件 | 行为 |
|------|------|
| OnEnter | `spawn_main_menu()` — 生成 UI 树：Title / Subtitle / List(NewGame, LoadGame, Settings) / Version |
| OnExit | `despawn_main_menu()` — 通过 `With<MainMenuScreen>` 查询清理所有实体 |

### 3.6 Overlay 依赖

当前 MVP 无 Overlay 依赖。后续迭代补充 ModalOverlay（新游戏确认）、LoadingOverlay。

---

## 4. InventoryScreen

### 4.1 基本信息

| 属性 | 值 |
|------|------|
| 对应 GameState | GameState::Inventory |
| ScreenLayer 层级 | 0（主界面层） |
| 加载模式 | Ephemeral（spawn/despawn 生命周期） |
| 过渡动画 | 未实现 |

### 4.2 消费的 ViewModel

| ViewModel | 用途 | 必选/可选 |
|-----------|------|-----------|
| 无 | MVP 阶段使用静态样本数据，未接入 UiStore | — |

### 4.3 组合的 Widget

InventoryScreen 当前使用 InventoryGrid Organism + 直接组合：

```
InventoryScreen
└── InventoryGrid [InventoryGrid — Organism]
    ├── Panel Container (Basic, column, 400px)
    │   ├── Text ("Inventory", Heading, localized)       — 标题
    │   ├── Text ("Gold: 100", Caption, static)          — 金币显示（硬编码）
    │   ├── InventoryItemRow × 4                         — 示例物品行
    │   │   ├── "Health Potion" x5
    │   │   ├── "Mana Potion" x3
    │   │   ├── "Antidote" x2
    │   │   └── "Phoenix Down" x1
    │   └── Button ("Close", Secondary)                  — 关闭按钮
```

**当前实现限制**（MVP）：
- 物品数据为静态硬编码样本，不从 UiStore 读取
- 无筛选器（FilterBar）、搜索框（SearchBox）、排序（SortDropdown）
- 无物品详情面板（ItemDetailPanel）
- 无 ViewModel 绑定，无 Dirty<T> 消费
- 关闭按钮携带 `InventoryGridAction::Close` 标记，按钮点击处理待实现

### 4.4 发射的 UiCommand

| UiCommand | 触发 Widget | 条件 |
|-----------|------------|------|
| UiCommand::CloseScreen | CloseButton / Escape | MVP 中关闭功能待接线 |

### 4.5 生命周期特殊行为

| 事件 | 行为 |
|------|------|
| OnEnter | `spawn_inventory_screen()` — 生成 UI 树（全屏 Panel → InventoryGrid） |
| Active | 无（MVP 无 ViewModel 刷新逻辑） |
| OnExit | `despawn_inventory_screen()` — 通过 `With<InventoryScreen>` 查询清理所有实体 |

### 4.6 Overlay 依赖

当前 MVP 无 Overlay 依赖。后续迭代补充 TooltipOverlay、ModalOverlay、NotificationOverlay。

---

## 5. ShopScreen

### 5.1 基本信息

| 属性 | 值 |
|------|------|
| 对应 OverlayState | OverlayState::Shop |
| PopupLayer 层级 | 1（浮层，覆盖下层 Screen） |
| 加载模式 | Ephemeral |
| 过渡动画 | Slide(Direction::Up) |

(暂未实现。将在后续迭代中补充 ShopPanel Organism 和 EconomyProjection 支持。)

### 5.2 消费的 ViewModel

| ViewModel | 用途 | 必选/可选 |
|-----------|------|-----------|
| ShopVm | 商品列表、玩家金币、阵营 ID | 必选 |

### 5.3 组合的 Widget

ShopScreen 主要组合 ShopPanel Organism：

```
ShopScreen
├── ShopPanel [ShopPanel — Organism]               — 商店面板（widget-composites.md §3.8）
│   ├── Header（Panel）
│   │   ├── ShopName [HeadingText]                  — 商店名称
│   │   ├── PlayerGold [StatText]                   — "玩家金币: {ShopVm.player_gold}"
│   │   └── CloseButton [IconButton]                — 关闭
│   ├── TabPanel                                    — 购买/出售标签页
│   │   ├── BuyTab
│   │   │   └── ShopItemCard × N [Molecule]         — 商品卡片网格（widget-composites.md §2.6）
│   │   └── SellTab
│   │       └── InventoryItemRow × N [Molecule]     — 背包物品行（widget-composites.md §2.3）
│   └── ConfirmPanel（Panel — 底部）
│       ├── CartSummary [CaptionText]               — "已选 {count} 件，合计 {total}"
│       ├── BuyButton [PrimaryButton]               — "购买"
│       └── CancelButton [SecondaryButton]          — "取消"
```

### 5.4 发射的 UiCommand

| UiCommand | 触发 Widget | 条件 |
|-----------|------------|------|
| UiCommand::BuyItem(ItemId, u32) | BuyButton | 选中商品后点击购买（数量在弹窗中选择） |
| UiCommand::SellItem(ItemId, u32) | SellButton | 选中背包物品后出售 |
| UiCommand::CloseScreen | CloseButton / Escape | 关闭商店 |

### 5.5 生命周期特殊行为

| 事件 | 行为 |
|------|------|
| OnEnter | 注册 EconomyProjection Observer |
| OnEnter | 初始化 ShopPanelVm（从当前交互的 NPC 阵营获取商品列表） |
| Active | 每帧检测 ShopPanelVm.dirty → 刷新 ShopPanel（商品列表和金币显示） |
| OnExit | 注销 EconomyProjection Observer |
| OnExit | 清理 ShopPanelVm 为 Default |

### 5.6 Overlay 依赖

| Overlay | 用途 | 触发条件 |
|---------|------|---------|
| ModalOverlay | 购买确认/购买数量选择 | 点击 BuyButton |
| NotificationOverlay | 购买/出售结果反馈 | 交易完成后 |

---

## 6. SettingsScreen

### 6.1 基本信息

| 属性 | 值 |
|------|------|
| 对应 GameState | GameState::Settings（或在任何状态下通过 UiCommand::OpenScreen 打开） |
| ScreenLayer 层级 | 0（全屏设置时）或 1（浮层设置时） |
| 加载模式 | Ephemeral |
| 过渡动画 | Fade(0.3s) |

(暂未实现。将在后续迭代中补充 TabPanel + Toggle + ProgressBar 等设置项 Widget。)

### 6.2 消费的 ViewModel

| ViewModel | 用途 | 必选/可选 |
|-----------|------|-----------|
| 无 | SettingsScreen 直接读写 UiSettings（不经过 ViewModel） | — |

### 6.3 组合的 Widget

```
SettingsScreen
├── Header（Panel）
│   ├── ScreenTitle [HeadingText] — "设置"
│   └── CloseButton [IconButton] — 关闭
├── TabPanel [TabPanel]
│   ├── GameplayTab
│   │   ├── ShowDamageToggle [Toggle] — "显示伤害数字"
│   │   ├── ShowMinimapToggle [Toggle] — "显示小地图"
│   │   ├── ShowGridToggle [Toggle] — "显示网格"
│   │   └── AutoBattleToggle [Toggle] — "自动战斗"
│   ├── GraphicsTab
│   │   ├── ThemeSelector [SelectList] — 主题选择（Dark/Light/Pixel/HD2D）
│   │   └── LanguageSelector [SelectList] — 语言选择
│   ├── AudioTab
│   │   ├── MasterVolume [ProgressBar] — 主音量滑块
│   │   ├── BgmVolume [ProgressBar] — BGM 音量
│   │   └── SfxVolume [ProgressBar] — 音效音量
│   └── BattleTab
│       ├── BattleSpeedSlider [ProgressBar] — 战斗速度倍率 (0.5~3.0)
│       └── TooltipDelaySlider [ProgressBar] — 工具提示延迟
└── ResetButton [DangerButton] — "重置为默认值"
```

### 6.4 发射的 UiCommand

| UiCommand | 触发 Widget | 条件 |
|-----------|------------|------|
| UiCommand::ChangeSettings(UiSettings) | 任何设置变更 | 设置值变更时立即发送（或"应用"按钮点击时） |
| UiCommand::CloseScreen | CloseButton / Escape | 关闭设置 |

### 6.5 生命周期特殊行为

| 事件 | 行为 |
|------|------|
| OnEnter | 从 UiSettings 读取当前配置，初始化各 Widget 状态 |
| Active | 设置项变更时通过 UiCommand::ChangeSettings 即时持久化 |
| Active | 主题切换即时生效：Theme Resource 替换 → 全局 Dirty 标记 → 全 Widget 刷新 |
| OnExit | 无特殊清理 |

### 6.6 Overlay 依赖

| Overlay | 用途 | 触发条件 |
|---------|------|---------|
| ModalOverlay | 重置设置为默认值的确认 | 点击 ResetButton |

---

## 7. SaveLoadScreen

### 7.1 基本信息

| 属性 | 值 |
|------|------|
| 对应 GameState | GameState::SaveLoad（或在菜单中通过 UiCommand::OpenScreen 打开） |
| ScreenLayer 层级 | 0（主界面层） |
| 加载模式 | Ephemeral |
| 过渡动画 | Fade(0.3s) |
| 子模式 | SaveMode / LoadMode |

(暂未实现。将在后续迭代中补充 SaveSlot 列表和存档/读档 Widget。)

### 7.2 消费的 ViewModel

| ViewModel | 用途 | 必选/可选 |
|-----------|------|-----------|
| 无 | SaveLoadScreen 通过 Save Domain 接口查询存档信息 | — |

### 7.3 组合的 Widget

```
SaveLoadScreen
├── Header（Panel）
│   ├── ScreenTitle [HeadingText] — "保存游戏" / "加载游戏"
│   ├── ModeToggle [SecondaryButton] — 切换保存/加载模式
│   └── CloseButton [IconButton] — 关闭
├── SaveSlotList [SelectList] — 存档槽位列表
│   └── SaveSlot × N（最多 10 个槽位）
│       ├── SlotNumber [BodyText] — "存档位 1"
│       ├── SlotInfo [CaptionText] — 存档信息（时间、地点、等级、章节）
│       ├── PlayTime [CaptionText] — 游戏时间
│       └── EmptyLabel [CaptionText] — "空"（无存档的槽位显示）
└── ActionPanel（Panel — 底部）
    ├── ConfirmButton [PrimaryButton] — "保存" / "加载"
    └── DeleteButton [DangerButton] — "删除存档"
```

### 7.4 发射的 UiCommand

| UiCommand | 触发 Widget | 条件 |
|-----------|------------|------|
| UiCommand::SaveGame(SaveSlot) | ConfirmButton（SaveMode） | 选中空槽位或确认覆盖 |
| UiCommand::LoadGame(SaveSlot) | ConfirmButton（LoadMode） | 选中非空槽位 |
| UiCommand::CloseScreen | CloseButton / Escape | 关闭 |

### 7.5 生命周期特殊行为

| 事件 | 行为 |
|------|------|
| OnEnter | 从 Save Domain 查询所有槽位元数据（时间、等级、地点） |
| OnEnter | 根据打开模式设置子模式（SaveMode/LoadMode） |
| Active | SaveMode 下选中非空槽位时弹出覆盖确认 |
| Active | LoadMode 下非空槽位的 ConfirmButton 才 enabled |
| OnExit | 无特殊清理 |

### 7.6 Overlay 依赖

| Overlay | 用途 | 触发条件 |
|---------|------|---------|
| ModalOverlay | 覆盖存档确认 / 加载存档确认 / 删除存档确认 | 点击保存/加载/删除按钮 |
| LoadingOverlay | 加载存档时的资源加载 | 确认 LoadGame 后 |

---

## 8. 全部 Screen 一览

| Screen | GameState | ViewModel | 主要复合组件 | 实现状态 |
|--------|-----------|-----------|------------|---------|
| BattleScreen | Combat | BattleHudVm, SkillPanelVm, CharacterPanelVm（均硬编码） | CharacterCard, ActionMenu | MVP ✅ |
| MainMenuScreen | MainMenu | 无 | 无（仅 Text + List + Button 组合） | MVP ✅ |
| InventoryScreen | Inventory | 无（静态样本数据） | InventoryGrid | MVP ✅ |
| ShopScreen | — | — | — | ❌ |
| SettingsScreen | — | — | — | ❌ |
| SaveLoadScreen | — | — | — | ❌ |

---

## 9. Screen ViewModel 映射表

| Screen | 消费的 UiStore 字段 | 写入 Projection | 备注 |
|--------|-------------------|----------------|------|
| BattleScreen | battle_hud（MVP 硬编码） | BattleProjection（仅 on_turn_started 实现） | ViewModel 集成待完成 |
| InventoryScreen | 无（MVP 静态样本数据） | 无 | — |
| MainMenuScreen | 无 | — | — |
| SettingsScreen | 无 | — | ❌ 未实现 |
| SaveLoadScreen | 无 | — | ❌ 未实现 |
| ShopScreen | 无 | — | ❌ 未实现 |

---

## 10. 验证规则

| # | 规则 | 校验逻辑 |
|---|------|----------|
| S-VAL-01 | Screen 不直接引用另一个 Screen | Screen 代码中禁止 import 其他 Screen 的 mod |
| S-VAL-02 | Screen 不直接拼 Node | Screen 代码中无 Node/BackgroundColor/Interaction 等原语 |
| S-VAL-03 | Screen Widget 组合使用工厂函数 | Screen 通过 `spawn_*` 工厂函数组合 Widget，不直接操作 Node/Interaction |
| S-VAL-04 | Screen 优先组合 Organism，不直接组合大量 Atom | Screen 首先引用已定义的 Organism，仅当无对应 Organism 时才直接组合 Atom |
| S-VAL-05 | Screen 对应 GameState/OverlayState | 每个 Screen 有明确的状态对应关系 |
| S-VAL-06 | Screen OnExit 注销 Observer | Screen OnExit 时清理注册的 Observer |

---

*本文档由 @presentation-architect 维护。新增 Screen 必须先在本文档定义，再在 screen-lifecycle.md §4 注册映射关系。Screen 使用的复合组件必须先在 widget-composites.md 中定义。*

*最后更新: 2026-06-21 — 与实际代码实现对齐 (commit 903d039)*
