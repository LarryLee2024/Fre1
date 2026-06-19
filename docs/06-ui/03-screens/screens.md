---
id: 06-ui.screens
title: Screen Design — 页面详细设计
status: draft
owner: presentation-architect
created: 2026-06-20
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
| BattleHudVm | HP/MP 条、回合数、行动点 | 必选 |
| SkillPanelVm | 技能面板、冷却、AP | 必选 |
| CharacterPanelVm | 角色信息面板 | 必选（从 UiStore.character_panel 读取） |

### 2.3 组合的 Widget

BattleScreen 组合 3 个 Organism + 3 个直接 Atom/组合：

```
BattleScreen
├── TurnOrderBar [TurnOrderBar — Organism]         — 回合顺序条（widget-composites.md §3.4）
│   └── TurnIndicator × N [Molecule]               — 每个角色一个指示器
├── BattleHud [BattleHud — Organism]               — 战斗 HUD 主体（widget-composites.md §3.3）
│   ├── CharacterStatusPanel × N [Organism]        — 队伍角色状态面板（widget-composites.md §3.2）
│   │   ├── CharacterPortrait [Molecule]            — 角色头像（widget-composites.md §2.2）
│   │   ├── MpBar [MpBar]                          — MP 条
│   │   ├── ApBar [ProgressBar]                    — AP 条
│   │   ├── BuffIcon × N [Molecule]                — Buff/Debuff 图标（widget-composites.md §2.7）
│   │   └── StatusText [BodyText]                  — 状态文本
│   ├── SkillPanel [Organism]                      — 技能面板（widget-composites.md §3.1）
│   │   └── SkillSlot × N [Molecule]               — 技能槽位（widget-composites.md §2.1）
│   └── TopBar（Panel + LocalizedText 组合）
│       ├── TurnCounter [LocalizedText]             — "第 3 回合"
│       ├── PhaseIndicator [LocalizedText]          — "玩家回合"/"敌方回合"
│       └── EndTurnButton [SecondaryButton]        — 结束回合按钮
├── EnemyStatusBar（Panel — 精简敌方状态，仅显示可见敌方）
│   └── EnemyPortrait × N [CharacterPortrait]       — 精简版头像（widget-composites.md §2.2）
├── BattleFieldArea（Panel）
│   └── GridOverlay                                 — 网格地图覆盖（由 Tactical Domain 提供）
└── ActionMenu（Panel — 根据上下文动态显示）
    ├── AttackButton [PrimaryButton]
    ├── SkillButton [SecondaryButton]
    ├── WaitButton [SecondaryButton]
    └── CancelButton [DangerButton]
```

### 2.4 发射的 UiCommand

| UiCommand | 触发 Widget | 条件 |
|-----------|------------|------|
| UiCommand::CastSkill(SkillId) | BattleHud → SkillPanel → SkillSlot | 点击可用技能 |
| UiCommand::SelectTarget(CharacterId) | BattleHud → CharacterStatusPanel / BattleFieldArea | 选择目标后 |
| UiCommand::SelectCharacter(CharacterId) | BattleHud → CharacterStatusPanel → CharacterPortrait | 点击角色头像 |
| UiCommand::EndTurn | BattleHud → TopBar → EndTurnButton | 点击结束回合 |
| UiCommand::MoveToPosition(GridPos) | BattleFieldArea | 点击网格位置 |
| UiCommand::TogglePause | — | 按 Escape |

### 2.5 生命周期特殊行为

| 事件 | 行为 |
|------|------|
| OnEnter | 初始化 BattleHudVm（从 Domain Event 投影）、注册 Combat Domain Event Observer |
| OnEnter | 激活 BattleScreen 的 FocusGroup，默认焦点在 EndTurnButton |
| Active | 每帧检测 BattleHudVm.dirty → 刷新 BattleHud（级联刷新子 Organism：CharacterStatusPanel、SkillPanel、TurnOrderBar） |
| Active | 每帧检测 SkillPanelVm.dirty → 刷新 SkillPanel 内技能冷却和可用状态 |
| OnExit(Combat) | 注销所有 Combat Domain Event Observer |
| OnExit(Combat) | 清理 BattleHudVm、SkillPanelVm 为 Default |
| OnEnter(BattleScreen) after background | 重新投影 ViewModel（从最新的 Domain Event） |
| Pause | 显示 PauseMenu（ModalOverlay） |

### 2.6 Overlay 依赖

| Overlay | 用途 | 触发条件 |
|---------|------|---------|
| DamageTextOverlay | 伤害数字/治疗数字浮动 | Observer 监听 CueTriggered(CueType::Popup) |
| TooltipOverlay | 技能说明 / Buff 说明 | 技能图标 hover 超过 0.3s |
| NotificationOverlay | 战斗消息（升级、获得物品） | NotificationService 接收 NotificationVm |
| ModalOverlay | 暂停菜单 / 确认退出 | 按 Escape / 触发需要确认的 UiCommand |

### 2.7 变体

| 变体 | 差异 | 用途 |
|------|------|------|
| NormalBattle | 标准战斗 HUD | 常规战斗场景 |
| BossBattle | 额外 Boss HP 条 + 阶段提示 | Boss 战 |
| AutoBattle | 隐藏技能面板，显示自动战斗状态 | 自动战斗模式 |

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
├── BackgroundImage [Panel（全屏无边框）]
├── TitlePanel（Panel — 屏幕中央）
│   └── GameTitle [HeadingText] — 游戏标题
├── ButtonPanel（Panel — 屏募中央偏下）
│   ├── NewGameButton [PrimaryButton] — 新游戏 (label_key: "ui.menu.new_game")
│   ├── ContinueButton [SecondaryButton] — 继续 (label_key: "ui.menu.continue")
│   │   （enabled: 存在存档时 = true）
│   ├── LoadGameButton [SecondaryButton] — 加载游戏 (label_key: "ui.menu.load_game")
│   └── SettingsButton [SecondaryButton] — 设置 (label_key: "ui.menu.settings")
└── VersionText [CaptionText] — 版本号（屏幕角落）
```

### 3.4 发射的 UiCommand

| UiCommand | 触发 Widget | 说明 |
|-----------|------------|------|
| UiCommand::OpenScreen(ScreenType::SaveLoad) | ContinueButton | 继续游戏（直接加载最新存档） |
| UiCommand::OpenScreen(ScreenType::SaveLoad) | LoadGameButton | 打开读取存档界面 |
| UiCommand::OpenScreen(ScreenType::Settings) | SettingsButton | 打开设置界面 |
| UiCommand::NewGame | NewGameButton | 开始新游戏 |

### 3.5 生命周期特殊行为

| 事件 | 行为 |
|------|------|
| OnEnter | 检查是否存在存档 → 设置 ContinueButton.enabled |
| OnEnter | 播放标题动画（Logo 淡入、背景滚动） |
| OnExit | 无特殊清理（Ephemeral 模式，Screen despawn 时自动销毁全部 Widget） |

### 3.6 Overlay 依赖

| Overlay | 用途 | 触发条件 |
|---------|------|---------|
| ModalOverlay | 新游戏确认（是否覆盖现有存档） | 点击 NewGameButton 且存在存档 |
| LoadingOverlay | 新游戏/加载游戏时的资源加载 | 切换到其他 GameState 时 |

---

## 4. InventoryScreen

### 4.1 基本信息

| 属性 | 值 |
|------|------|
| 对应 GameState | GameState::Inventory（或在 OverlayState::Inventory 下打开） |
| ScreenLayer 层级 | 0（主界面层，全屏覆盖底部 Screen） |
| 加载模式 | Ephemeral |
| 过渡动画 | Slide(Direction::Left) |

### 4.2 消费的 ViewModel

| ViewModel | 用途 | 必选/可选 |
|-----------|------|-----------|
| InventoryVm | 物品列表、金币、筛选、选中 | 必选 |
| CharacterPanelVm | 角色装备信息 | 可选（选中 Equipment 类型物品时显示） |

### 4.3 组合的 Widget

InventoryScreen 组合 1 个 Organism + 直接 Atom：

```
InventoryScreen
├── InventoryGrid [InventoryGrid — Organism]       — 背包网格（widget-composites.md §3.5）
│   ├── Header（Panel）
│   │   ├── ScreenTitle [HeadingText]               — "背包"
│   │   └── GoldDisplay [StatText]                  — "金币: {InventoryVm.gold}"
│   ├── FilterBar（内置 TabPanel）                   — 分类过滤标签：全部/装备/消耗品/材料/关键物品
│   ├── SearchBox [TextInput]                       — 搜索输入框
│   ├── SortDropdown [SelectList]                   — 排序选项
│   ├── InventoryItemRow × N [Molecule]             — 物品行（widget-composites.md §2.3）
│   └── ScrollPanel                                 — 列表滚动容器
├── ItemDetailPanel（Panel — 右侧或底部，选中物品时显示）
│   ├── ItemName [HeadingText]
│   ├── ItemDescription [BodyText]
│   ├── ItemStats [StatText]
│   └── ActionButtons
│       ├── UseButton [PrimaryButton]               — "使用"
│       ├── EquipButton [SecondaryButton]           — "装备"
│       └── DropButton [DangerButton]               — "丢弃"
└── CloseButton [IconButton]                       — 关闭按钮
```

### 4.4 发射的 UiCommand

| UiCommand | 触发 Widget | 条件 |
|-----------|------------|------|
| UiCommand::UseItem(ItemId) | UseButton | 选中可使用的物品 |
| UiCommand::EquipItem(ItemId, EquipmentSlot) | EquipButton | 选中的是可装备物品 |
| UiCommand::DropItem(ItemId) | DropButton | 点击丢弃并确认 |
| UiCommand::CloseScreen | CloseButton / Escape | 关闭背包 |

### 4.5 生命周期特殊行为

| 事件 | 行为 |
|------|------|
| OnEnter | 注册 InventoryProjection Observer（监听 ItemAcquired/ItemUsed） |
| OnEnter | InventoryGrid 初始筛选为 All，搜索框为空 |
| Active | 每帧检测 InventoryGridVm.dirty → 刷新 InventoryGrid（级联刷新子 InventoryItemRow 列表） |
| Active | 筛选/搜索/排序变更 → 更新 InventoryGridVm.filter/search/sort + mark_dirty |
| OnExit | 注销 InventoryProjection Observer |
| OnExit | InventoryGridVm 不清理（下次进入重新投影） |

### 4.6 Overlay 依赖

| Overlay | 用途 | 触发条件 |
|---------|------|---------|
| TooltipOverlay | 物品详细说明 | 物品图标 hover 超过 0.3s |
| ModalOverlay | 确认丢弃 | 点击 DropButton 时弹出确认 |
| NotificationOverlay | 使用物品结果反馈 | 使用物品后 |

---

## 5. ShopScreen

### 5.1 基本信息

| 属性 | 值 |
|------|------|
| 对应 OverlayState | OverlayState::Shop |
| PopupLayer 层级 | 1（浮层，覆盖下层 Screen） |
| 加载模式 | Ephemeral |
| 过渡动画 | Slide(Direction::Up) |

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

| Screen | GameState | ViewModel | 主要复合组件 | 关键 UiCommand |
|--------|-----------|-----------|------------|---------------|
| BattleScreen | Combat | BattleHudVm, SkillPanelVm, CharacterPanelVm | BattleHud, SkillPanel, CharacterStatusPanel, TurnOrderBar | CastSkill, SelectTarget, EndTurn |
| MainMenuScreen | MainMenu | 无 | 无（仅 Panel + Atom 按钮组合） | OpenScreen, NewGame |
| InventoryScreen | Inventory | InventoryGridVm, CharacterPanelVm | InventoryGrid | UseItem, EquipItem, DropItem |
| ShopScreen | OverlayState::Shop | ShopPanelVm | ShopPanel, InventoryGrid（SellTab 内） | BuyItem, SellItem |
| SettingsScreen | Settings | 无（直接读 UiSettings） | 无（仅 TabPanel + Toggle + ProgressBar 组合） | ChangeSettings |
| SaveLoadScreen | SaveLoad | 无（通过 Save Domain 接口） | 无（仅 SelectList + Panel + Button 组合） | SaveGame, LoadGame |

---

## 9. Screen ViewModel 映射表

| Screen | 消费的 UiStore 字段 | 写入 Projection | 只读（不写入） |
|--------|-------------------|----------------|---------------|
| BattleScreen | battle_hud, skill_panel, character_panel | BattleProjection | — |
| InventoryScreen | inventory_grid, character_panel | InventoryProjection | — |
| ShopScreen | shop_panel | EconomyProjection | — |
| MainMenuScreen | 无 | — | — |
| SettingsScreen | 无（直接 UiSettings） | — | UiSettings |
| SaveLoadScreen | 无（通过 Save Domain） | — | SaveSlot 元数据 |

---

## 10. 验证规则

| # | 规则 | 校验逻辑 |
|---|------|----------|
| S-VAL-01 | Screen 不直接引用另一个 Screen | Screen 代码中禁止 import 其他 Screen 的 mod |
| S-VAL-02 | Screen 不直接拼 Node | Screen 代码中无 Node/BackgroundColor/Interaction 等原语 |
| S-VAL-03 | Screen Widget 组合使用 WidgetFactory | Screen 通过 WidgetFactory::create 组合 Widget |
| S-VAL-04 | Screen 优先组合 Organism，不直接组合大量 Atom | Screen 首先引用 widget-composites.md 中定义的 Organism，仅当该 Screen 无对应 Organism 时才直接组合 Atom |
| S-VAL-05 | Screen 对应 GameState/OverlayState | 每个 Screen 有明确的状态对应关系 |
| S-VAL-06 | Screen OnExit 注销 Observer | Screen OnExit 时清理注册的 Observer |

---

*本文档由 @presentation-architect 维护。新增 Screen 必须先在本文档定义，再在 screen-lifecycle.md §4 注册映射关系。Screen 使用的复合组件必须先在 widget-composites.md 中定义。*
