---
id: 07-specs.settings-screen
title: SettingsScreen Specification — AI-Consumable Layout & Interaction Spec
status: active
owner: presentation-architect
created: 2026-06-22
tags:
  - ui
  - screen-spec
  - settings
  - active
---

# SettingsScreen

> **职责**: @presentation-architect | **上游**: ADR-066 (Screen Spec), `07-specs/README.md` (总纲)
> **状态**: active

**P0 字段**: 1-14 (Screen Header / ASCII Wireframe / Widget Tree / Flexbox Layout / Responsive Rules / Region Responsibility / Widget Contract / State Mapping / Focus Nav / Interaction Zones / Overlay / Lifecycle / Data Ownership / Layout Intent)
**P1 字段**: 15-17 (Scroll & Overflow / Event Contract / Screen Metrics)

---

## 1. Screen Header

| 属性 | 值 |
|------|-----|
| Screen Name | `SettingsScreen` — 对应 `GameState::Settings` |
| Purpose | 游戏设置管理：提供 Gameplay/Graphics/Audio/Battle 四类设置的查看与修改，支持主题切换、音量调节、战斗速度等配置项的即时调整 |
| Navigation | MainMenu → Settings (UiCommand::OpenScreen(ScreenType::Settings))；点击 Close 或 Esc 返回前一 Screen（MainMenuScreen）；变更即时生效不阻塞导航 |
| GameState | `GameState::Settings` |
| ScreenLayer 层级 | 0（主界面层） |
| 加载模式 | Ephemeral（每次进入 Settings 重新 spawn） |
| 过渡动画 | 当前未实现，预留 Fade(0.3s) |
| 变体 | None |

---

## 2. ASCII Wireframe

> 纯文本线框图。所有区域必须命名（`widget_id`），禁止匿名面板。

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│  [settings_header]                                                               │
│  ┌──────────────────────────────────────────────────────────────────────────┐    │
│  │  [settings_header_title]                              [settings_header_close]│
│  │              Settings                                     [X]               │
│  └──────────────────────────────────────────────────────────────────────────┘    │
├──────────────────────────────────────────────────────────────────────────────────┤
│  [settings_tabpanel]                                                              │
│  ┌────────────────────┐  ┌────────────────────────────────────────────────────┐  │
│  │ [settings_tablist] │  │ [settings_gameplay — active tab]                   │  │
│  │                    │  │                                                    │  │
│  │  ▶ Gameplay       │  │  [settings_gameplay_damage]                         │  │
│  │    Graphics       │  │  ┌──────────────────────────────────────────────┐  │  │
│  │    Audio          │  │  │  ☑ Show Damage Numbers                      │  │  │
│  │    Battle         │  │  └──────────────────────────────────────────────┘  │  │
│  │                    │  │                                                    │  │
│  │                    │  │  [settings_gameplay_minimap]                       │  │
│  │                    │  │  ┌──────────────────────────────────────────────┐  │  │
│  │                    │  │  │  ☑ Show Minimap                             │  │  │
│  │                    │  │  └──────────────────────────────────────────────┘  │  │
│  │                    │  │                                                    │  │
│  │                    │  │  [settings_gameplay_grid]                          │  │
│  │                    │  │  ┌──────────────────────────────────────────────┐  │  │
│  │                    │  │  │  ☑ Show Grid                                │  │  │
│  │                    │  │  └──────────────────────────────────────────────┘  │  │
│  │                    │  │                                                    │  │
│  │                    │  │  [settings_gameplay_autobattle]                    │  │
│  │                    │  │  ┌──────────────────────────────────────────────┐  │  │
│  │                    │  │  │  ☐ Auto Battle                              │  │  │
│  │                    │  │  └──────────────────────────────────────────────┘  │  │
│  └────────────────────┘  └────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────────┤
│  [settings_footer]                                                               │
│                              ┌──────────────────────────────────────┐            │
│                              │  [settings_reset_btn]               │            │
│                              │     Reset to Default                 │            │
│                              └──────────────────────────────────────┘            │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Region 索引

| widget_id | 类型 | 用途 | 对应 Wireframe 位置 |
|-----------|------|------|-------------------|
| `settings_header` | Container | 包裹标题文本和关闭按钮，水平排列 | Wireframe 顶部第一行 |
| `settings_header_title` | HeadingText | 显示 "Settings" 标题文本 | 标题处（左侧） |
| `settings_header_close` | IconButton | 关闭按钮，点击触发返回上一级 Screen | 标题处（右侧） |
| `settings_tabpanel` | Container | 包裹 TabList 和 TabContent 两个子区域，水平分栏布局 | Wireframe 中部 |
| `settings_tablist` | Container | 垂直排列 4 个 Tab 按钮（Gameplay/Graphics/Audio/Battle） | 左侧栏（固定 200px） |
| `settings_tablist_gameplay` | TabButton | 选中 Gameplay Tab，显示游戏玩法相关设置（高亮表示当前激活） | 左侧第一个 Tab |
| `settings_tablist_graphics` | TabButton | 选中 Graphics Tab，显示画面/主题/语言设置 | 左侧第二个 Tab |
| `settings_tablist_audio` | TabButton | 选中 Audio Tab，显示音量相关设置 | 左侧第三个 Tab |
| `settings_tablist_battle` | TabButton | 选中 Battle Tab，显示战斗相关设置 | 左侧第四个 Tab |
| `settings_tabcontent` | Container | 包裹当前激活 Tab 的全部设置项控件 | 右侧内容区 |
| `settings_gameplay` | Container | Gameplay Tab 的内容容器，垂直排列 4 个 Toggle（仅在 Gameplay Tab 激活时可见） | 右侧内容区（Gameplay 激活） |
| `settings_gameplay_damage` | Toggle | 是否显示伤害数字 | Gameplay Tab 第一个 Toggle |
| `settings_gameplay_minimap` | Toggle | 是否显示小地图 | Gameplay Tab 第二个 Toggle |
| `settings_grid_toggle` | Toggle | 是否显示网格 | Gameplay Tab 第三个 Toggle |
| `settings_gameplay_autobattle` | Toggle | 是否启用自动战斗 | Gameplay Tab 第四个 Toggle |
| `settings_graphics` | Container | Graphics Tab 的内容容器，垂直排列 ThemeSelector + LanguageSelector | 右侧内容区（Graphics 激活） |
| `settings_graphics_theme` | SelectList | 主题选择下拉选项（Dark/Light/Pixel/HD2D） | Graphics Tab 第一个控件 |
| `settings_graphics_language` | SelectList | 语言选择下拉选项 | Graphics Tab 第二个控件 |
| `settings_audio` | Container | Audio Tab 的内容容器，垂直排列 3 个音量滑块 | 右侧内容区（Audio 激活） |
| `settings_audio_master` | ProgressBar | 主音量滑块 (0~100%) | Audio Tab 第一个滑块 |
| `settings_audio_bgm` | ProgressBar | BGM 音量滑块 (0~100%) | Audio Tab 第二个滑块 |
| `settings_audio_sfx` | ProgressBar | 音效音量滑块 (0~100%) | Audio Tab 第三个滑块 |
| `settings_battle` | Container | Battle Tab 的内容容器，垂直排列 2 个滑块 | 右侧内容区（Battle 激活） |
| `settings_battle_speed` | ProgressBar | 战斗速度倍率滑块 (0.5x~3.0x) | Battle Tab 第一个滑块 |
| `settings_battle_tooltip` | ProgressBar | 工具提示延迟滑块 (0ms~2000ms) | Battle Tab 第二个滑块 |
| `settings_footer` | Container | 底部操作栏，包裹 ResetButton | Wireframe 底部 |
| `settings_reset_btn` | Button, Danger | 重置所有设置为默认值，点击触发 ModalOverlay 确认 | 底部右侧 |

---

## 3. Widget Tree

> 标注 `[widget_id: WidgetType]` 的树结构。禁止隐藏节点，必须完整。

```
ScreenRoot                                                  [settings_root: Screen]
├── HeaderPanel                                             [settings_header: Container]
│   ├── ScreenTitle                                         [settings_header_title: HeadingText]
│   └── CloseButton                                         [settings_header_close: IconButton]
├── TabPanel                                                [settings_tabpanel: Container]
│   ├── TabList                                             [settings_tablist: Container]
│   │   ├── GameplayTabButton                               [settings_tablist_gameplay: TabButton]
│   │   ├── GraphicsTabButton                               [settings_tablist_graphics: TabButton]
│   │   ├── AudioTabButton                                  [settings_tablist_audio: TabButton]
│   │   └── BattleTabButton                                 [settings_tablist_battle: TabButton]
│   └── TabContent                                          [settings_tabcontent: Container]
│       ├── GameplayContent                                 [settings_gameplay: Container]
│       │   ├── ShowDamageToggle                            [settings_gameplay_damage: Toggle]
│       │   ├── ShowMinimapToggle                           [settings_gameplay_minimap: Toggle]
│       │   ├── ShowGridToggle                              [settings_grid_toggle: Toggle]
│       │   └── AutoBattleToggle                            [settings_gameplay_autobattle: Toggle]
│       ├── GraphicsContent                                 [settings_graphics: Container]
│       │   ├── ThemeSelector                               [settings_graphics_theme: SelectList]
│       │   └── LanguageSelector                            [settings_graphics_language: SelectList]
│       ├── AudioContent                                    [settings_audio: Container]
│       │   ├── MasterVolumeSlider                          [settings_audio_master: ProgressBar]
│       │   ├── BgmVolumeSlider                             [settings_audio_bgm: ProgressBar]
│       │   └── SfxVolumeSlider                             [settings_audio_sfx: ProgressBar]
│       └── BattleContent                                   [settings_battle: Container]
│           ├── BattleSpeedSlider                           [settings_battle_speed: ProgressBar]
│           └── TooltipDelaySlider                          [settings_battle_tooltip: ProgressBar]
└── FooterPanel                                             [settings_footer: Container]
    └── ResetButton                                         [settings_reset_btn: Button, Danger]
```

### 3.1 Widget Type 索引

| widget_id | WidgetType | 定义位置 | 复用于 |
|-----------|-----------|---------|--------|
| `settings_header_title` | `Atom: HeadingText` | `02-design-system/widget-atoms.md §HeadingText` | MainMenuScreen、各 Screen 标题 |
| `settings_header_close` | `Atom: IconButton` | `02-design-system/widget-atoms.md §IconButton` | InventoryScreen、SaveLoadScreen |
| `settings_tablist_gameplay` | `Atom: TabButton` | `02-design-system/widget-atoms.md §TabButton` | — |
| `settings_tablist_graphics` | `Atom: TabButton` | `02-design-system/widget-atoms.md §TabButton` | — |
| `settings_tablist_audio` | `Atom: TabButton` | `02-design-system/widget-atoms.md §TabButton` | — |
| `settings_tablist_battle` | `Atom: TabButton` | `02-design-system/widget-atoms.md §TabButton` | — |
| `settings_gameplay_damage` | `Atom: Toggle` | `02-design-system/widget-atoms.md §Toggle` | — |
| `settings_gameplay_minimap` | `Atom: Toggle` | `02-design-system/widget-atoms.md §Toggle` | — |
| `settings_grid_toggle` | `Atom: Toggle` | `02-design-system/widget-atoms.md §Toggle` | — |
| `settings_gameplay_autobattle` | `Atom: Toggle` | `02-design-system/widget-atoms.md §Toggle` | — |
| `settings_graphics_theme` | `Atom: SelectList` | `02-design-system/widget-atoms.md §SelectList` | — |
| `settings_graphics_language` | `Atom: SelectList` | `02-design-system/widget-atoms.md §SelectList` | — |
| `settings_audio_master` | `Atom: ProgressBar` | `02-design-system/widget-atoms.md §ProgressBar` | HP Bar、MP Bar |
| `settings_audio_bgm` | `Atom: ProgressBar` | `02-design-system/widget-atoms.md §ProgressBar` | — |
| `settings_audio_sfx` | `Atom: ProgressBar` | `02-design-system/widget-atoms.md §ProgressBar` | — |
| `settings_battle_speed` | `Atom: ProgressBar` | `02-design-system/widget-atoms.md §ProgressBar` | — |
| `settings_battle_tooltip` | `Atom: ProgressBar` | `02-design-system/widget-atoms.md §ProgressBar` | — |
| `settings_reset_btn` | `Atom: Button (Danger)` | `02-design-system/widget-atoms.md §Button` | — |

---

## 4. Flexbox Layout

> YAML 格式。每个 widget_id 必须有 direction / width / height / flex_grow / intent。

```yaml
## Flexbox Layout — SettingsScreen
## width/height: px 值或 "auto" 或 "fill"
## flex_grow: 0=不增长, 1=等分剩余空间, 2=双倍增长
## shrink: none/low/high — 收缩优先级

settings_root:
  direction: column
  width: 100%
  height: 100%
  flex_grow: 0
  intent: "Screen 根容器，占满视口，垂直排列 Header / TabPanel / Footer"

settings_header:
  direction: row
  width: 100%
  height: 56
  flex_grow: 0
  shrink: none
  intent: "Header 栏，固定 56px 高度，水平排列标题（左）和关闭按钮（右）"

settings_header_title:
  direction: row
  width: auto
  height: auto
  flex_grow: 1
  shrink: none
  intent: "标题文本，auto 宽度适配文本长度，flex_grow:1 推动关闭按钮到右侧"

settings_header_close:
  direction: row
  width: 40
  height: 40
  flex_grow: 0
  shrink: none
  intent: "关闭按钮，固定 40x40 保证最小触摸目标"

settings_tabpanel:
  direction: row
  width: 100%
  height: fill
  flex_grow: 1
  intent: "TabPanel 主体区域，水平分栏，占据除 Header+Footer 外的所有剩余空间"

settings_tablist:
  direction: column
  width: 200
  height: 100%
  flex_grow: 0
  shrink: none
  intent: "Tab 列表侧栏，固定 200px 宽度，垂直排列 4 个 Tab 按钮"

settings_tablist_gameplay:
  direction: row
  width: 200
  height: 44
  flex_grow: 0
  shrink: none
  intent: "Gameplay Tab 按钮，固定 200x44，与 tablist 同宽"

settings_tablist_graphics:
  direction: row
  width: 200
  height: 44
  flex_grow: 0
  shrink: none
  intent: "Graphics Tab 按钮，固定 200x44"

settings_tablist_audio:
  direction: row
  width: 200
  height: 44
  flex_grow: 0
  shrink: none
  intent: "Audio Tab 按钮，固定 200x44"

settings_tablist_battle:
  direction: row
  width: 200
  height: 44
  flex_grow: 0
  shrink: none
  intent: "Battle Tab 按钮，固定 200x44"

settings_tabcontent:
  direction: column
  width: fill
  height: 100%
  flex_grow: 1
  intent: "Tab 内容区，弹性宽度填充剩余空间，垂直排列当前 Tab 的设置项"

settings_gameplay:
  direction: column
  width: 100%
  height: auto
  flex_grow: 0
  intent: "Gameplay Tab 内容容器，auto 高度包裹 4 个 Toggle"

settings_gameplay_damage:
  direction: row
  width: 100%
  height: 40
  flex_grow: 0
  shrink: none
  intent: "伤害数字 Toggle，固定 40px 高度，标签左 + Toggle 右"

settings_gameplay_minimap:
  direction: row
  width: 100%
  height: 40
  flex_grow: 0
  shrink: none
  intent: "小地图 Toggle，固定 40px 高度，与上一 Toggle 同尺寸"

settings_grid_toggle:
  direction: row
  width: 100%
  height: 40
  flex_grow: 0
  shrink: none
  intent: "网格显示 Toggle，固定 40px 高度，控制游戏视图网格可见性"

settings_gameplay_autobattle:
  direction: row
  width: 100%
  height: 40
  flex_grow: 0
  shrink: none
  intent: "自动战斗 Toggle，固定 40px 高度"

settings_graphics:
  direction: column
  width: 100%
  height: auto
  flex_grow: 0
  intent: "Graphics Tab 内容容器，auto 高度包裹 2 个 SelectList"

settings_graphics_theme:
  direction: column
  width: 100%
  height: auto
  flex_grow: 0
  intent: "主题选择 SelectList，auto 高度适应选项列表展开"

settings_graphics_language:
  direction: column
  width: 100%
  height: auto
  flex_grow: 0
  intent: "语言选择 SelectList，auto 高度适应选项列表展开"

settings_audio:
  direction: column
  width: 100%
  height: auto
  flex_grow: 0
  intent: "Audio Tab 内容容器，auto 高度包裹 3 个音量滑块"

settings_audio_master:
  direction: column
  width: 100%
  height: auto
  flex_grow: 0
  intent: "主音量滑块，标签+进度条垂直排列，auto 高度"

settings_audio_bgm:
  direction: column
  width: 100%
  height: auto
  flex_grow: 0
  intent: "BGM 音量滑块，与主音量同结构"

settings_audio_sfx:
  direction: column
  width: 100%
  height: auto
  flex_grow: 0
  intent: "SFX 音量滑块，与主音量同结构"

settings_battle:
  direction: column
  width: 100%
  height: auto
  flex_grow: 0
  intent: "Battle Tab 内容容器，auto 高度包裹 2 个滑块"

settings_battle_speed:
  direction: column
  width: 100%
  height: auto
  flex_grow: 0
  intent: "战斗速度滑块，标签+进度条+倍率数值，auto 高度"

settings_battle_tooltip:
  direction: column
  width: 100%
  height: auto
  flex_grow: 0
  intent: "工具提示延迟滑块，auto 高度"

settings_footer:
  direction: row
  width: 100%
  height: 56
  flex_grow: 0
  shrink: none
  intent: "底部操作栏，固定 56px 高度，水平右对齐排列 ResetButton"

settings_reset_btn:
  direction: row
  width: 180
  height: 40
  flex_grow: 0
  shrink: none
  intent: "重置按钮，固定 180x40，Danger 样式，右对齐置于 Footer"
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

### 6.1 settings_header

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示设置页面标题文本（LocalizationKey: `ui.settings.title`） | Display | 页面标题缺失 |
| R02 | 展示关闭按钮（IconButton），图标为 "X" 或返回箭头 | Display | 用户无法关闭设置 |
| R03 | 响应关闭按钮点击，触发 `UiCommand::CloseScreen` | Interaction | 关闭功能失效 |
| R04 | 标题文本左对齐，关闭按钮右对齐，水平两端布局 | Layout | 布局违反预期 |

**不负责**:
- 管理 Tab 切换（属于 settings_tablist 职责）
- 保存或应用设置变更

### 6.2 settings_tablist

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示 4 个 Tab 按钮（Gameplay / Graphics / Audio / Battle），垂直排列 | Display | Tab 缺失或顺序错误 |
| R02 | 标记当前激活的 Tab 按钮为高亮样式（▶ 前缀 + 强调色），其他 Tab 为默认样式 | Display | 当前 Tab 不可辨识 |
| R03 | 响应 Tab 按钮点击，切换激活的 Tab，更新 `settings_tabcontent` 显示对应内容 | Interaction | Tab 切换不生效 |
| R04 | 管理 Tab 按钮间的焦点位移（ArrowUp/Down 导航） | Focus | 键盘无法在 Tab 间移动 |
| R05 | TabList 固定 200px 宽度，不伸缩 | Layout | 布局宽度不一致 |

**不负责**:
- 显示设置项控件内容（属于 settings_tabcontent）
- 重置设置或关闭页面

### 6.3 settings_tabcontent

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 根据当前激活的 Tab 显示对应的内容容器（gameplay / graphics / audio / battle） | Display | 显示错误 Tab 内容 |
| R02 | 内容区弹性宽度填充剩余水平空间 | Layout | 内容区宽度异常 |
| R03 | 内容区支持垂直滚动（当设置项总高度超出可视区域时） | Scroll | 底部设置项不可见 |
| R04 | 管理内容区控件的焦点位移（ArrowUp/Down 导航） | Focus | 键盘无法操作设置项 |
| R05 | 显示网格可见性切换控件（ShowGridToggle），控制游戏视图网格的显示与隐藏 | Display | 用户无法控制网格显示 |

**不负责**:
- Tab 切换逻辑（由 settings_tablist 驱动）
- 设置的验证或持久化

### 6.4 settings_footer

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示 "Reset to Default" 重置按钮（Danger 样式） | Display | 重置入口缺失 |
| R02 | 响应重置按钮点击，弹出 ModalOverlay 确认弹窗 | Interaction | 直接重置无确认，导致误操作 |
| R03 | 按钮右对齐于底部操作栏 | Layout | 按钮位置异常 |

**不负责**:
- 直接执行重置逻辑（仅触发确认弹窗，确认后由 Modal 处理）
- Tab 切换或关闭页面

---

## 7. Widget Contract

> Inputs / Outputs / Selection Model。

### 7.1 settings_header_title

```yaml
widget_id: settings_header_title
widget_type: HeadingText
defined_in: "02-design-system/widget-atoms.md §HeadingText"

inputs:
  - name: text
    type: LocalizedText
    source: "ui.settings.title"
    default: "Settings"

outputs: []

selection_model:
  type: none
```

### 7.2 settings_header_close

```yaml
widget_id: settings_header_close
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

### 7.3 settings_tablist_gameplay

```yaml
widget_id: settings_tablist_gameplay
widget_type: TabButton
defined_in: "02-design-system/widget-atoms.md §TabButton"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.tab.gameplay"
    default: "Gameplay"
  - name: is_active
    type: bool
    source: "TabPanel.active_tab == TabId::Gameplay"
    default: true

outputs:
  - name: selected
    type: TabSelected(TabId::Gameplay)
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.4 settings_tablist_graphics

```yaml
widget_id: settings_tablist_graphics
widget_type: TabButton
defined_in: "02-design-system/widget-atoms.md §TabButton"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.tab.graphics"
    default: "Graphics"
  - name: is_active
    type: bool
    source: "TabPanel.active_tab == TabId::Graphics"
    default: false

outputs:
  - name: selected
    type: TabSelected(TabId::Graphics)
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.5 settings_tablist_audio

```yaml
widget_id: settings_tablist_audio
widget_type: TabButton
defined_in: "02-design-system/widget-atoms.md §TabButton"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.tab.audio"
    default: "Audio"
  - name: is_active
    type: bool
    source: "TabPanel.active_tab == TabId::Audio"
    default: false

outputs:
  - name: selected
    type: TabSelected(TabId::Audio)
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.6 settings_tablist_battle

```yaml
widget_id: settings_tablist_battle
widget_type: TabButton
defined_in: "02-design-system/widget-atoms.md §TabButton"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.tab.battle"
    default: "Battle"
  - name: is_active
    type: bool
    source: "TabPanel.active_tab == TabId::Battle"
    default: false

outputs:
  - name: selected
    type: TabSelected(TabId::Battle)
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.7 settings_gameplay_damage

```yaml
widget_id: settings_gameplay_damage
widget_type: Toggle
defined_in: "02-design-system/widget-atoms.md §Toggle"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.gameplay.show_damage"
    default: "Show Damage Numbers"
  - name: is_on
    type: bool
    source: "UiSettings.show_damage_numbers"
    default: true

outputs:
  - name: toggled
    type: UiCommand::ChangeSettings
    payload: "{ show_damage_numbers: new_value }"
    trigger: OnToggle

selection_model:
  type: none
```

### 7.8 settings_gameplay_minimap

```yaml
widget_id: settings_gameplay_minimap
widget_type: Toggle
defined_in: "02-design-system/widget-atoms.md §Toggle"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.gameplay.show_minimap"
    default: "Show Minimap"
  - name: is_on
    type: bool
    source: "UiSettings.show_minimap"
    default: true

outputs:
  - name: toggled
    type: UiCommand::ChangeSettings
    payload: "{ show_minimap: new_value }"
    trigger: OnToggle

selection_model:
  type: none
```

### 7.9 settings_gameplay_autobattle

```yaml
widget_id: settings_gameplay_autobattle
widget_type: Toggle
defined_in: "02-design-system/widget-atoms.md §Toggle"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.gameplay.auto_battle"
    default: "Auto Battle"
  - name: is_on
    type: bool
    source: "UiSettings.auto_battle"
    default: false

outputs:
  - name: toggled
    type: UiCommand::ChangeSettings
    payload: "{ auto_battle: new_value }"
    trigger: OnToggle

selection_model:
  type: none
```

### 7.10 settings_grid_toggle

```yaml
widget_id: settings_grid_toggle
widget_type: Toggle
defined_in: "02-design-system/widget-atoms.md §Toggle"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.gameplay.show_grid"
    default: "Show Grid"
  - name: is_on
    type: bool
    source: "UiSettings.show_grid"
    default: true

outputs:
  - name: toggled
    type: UiCommand::ChangeSettings
    payload: "{ show_grid: new_value }"
    trigger: OnToggle

selection_model:
  type: none
```

### 7.11 settings_graphics_theme

```yaml
widget_id: settings_graphics_theme
widget_type: SelectList
defined_in: "02-design-system/widget-atoms.md §SelectList"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.graphics.theme"
    default: "Theme"
  - name: options
    type: Vec<LocalizedOption>
    source: "content/settings/themes.ron"
    default:
      - { key: "ui.settings.graphics.theme_option_dark",  value: "Dark" }
      - { key: "ui.settings.graphics.theme_option_light", value: "Light" }
      - { key: "ui.settings.graphics.theme_option_pixel", value: "Pixel" }
      - { key: "ui.settings.graphics.theme_option_hd2d",  value: "HD2D" }
  - name: selected_index
    type: usize
    source: "UiSettings.theme_index"
    default: 0

outputs:
  - name: selected
    type: UiCommand::ChangeSettings
    payload: "{ theme_index: selected_index }"
    trigger: OnSelect

selection_model:
  type: single
  max_select: 1
  clear_on: none
```

### 7.12 settings_graphics_language

```yaml
widget_id: settings_graphics_language
widget_type: SelectList
defined_in: "02-design-system/widget-atoms.md §SelectList"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.graphics.language"
    default: "Language"
  - name: options
    type: Vec<LocalizedOption>
    source: "content/settings/languages.ron"
    default:
      - { key: "ui.settings.graphics.language_option_en", value: "English" }
      - { key: "ui.settings.graphics.language_option_zh", value: "中文" }
      - { key: "ui.settings.graphics.language_option_ja", value: "日本語" }
  - name: selected_index
    type: usize
    source: "UiSettings.language_index"
    default: 0

outputs:
  - name: selected
    type: UiCommand::ChangeSettings
    payload: "{ language_index: selected_index }"
    trigger: OnSelect

selection_model:
  type: single
  max_select: 1
  clear_on: none
```

### 7.13 settings_audio_master

```yaml
widget_id: settings_audio_master
widget_type: ProgressBar
defined_in: "02-design-system/widget-atoms.md §ProgressBar"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.audio.master_volume"
    default: "Master Volume"
  - name: value
    type: f32 (0.0 ~ 1.0)
    source: "UiSettings.master_volume"
    default: 0.8
  - name: display_value
    type: String
    source: "computed: format!('{}%', (value * 100.0) as u32)"
    default: "80%"

outputs:
  - name: changed
    type: UiCommand::ChangeSettings
    payload: "{ master_volume: new_value }"
    trigger: OnValueChange

selection_model:
  type: none
```

### 7.14 settings_audio_bgm

```yaml
widget_id: settings_audio_bgm
widget_type: ProgressBar
defined_in: "02-design-system/widget-atoms.md §ProgressBar"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.audio.bgm_volume"
    default: "BGM Volume"
  - name: value
    type: f32 (0.0 ~ 1.0)
    source: "UiSettings.bgm_volume"
    default: 0.7
  - name: display_value
    type: String
    source: "computed: format!('{}%', (value * 100.0) as u32)"
    default: "70%"

outputs:
  - name: changed
    type: UiCommand::ChangeSettings
    payload: "{ bgm_volume: new_value }"
    trigger: OnValueChange

selection_model:
  type: none
```

### 7.15 settings_audio_sfx

```yaml
widget_id: settings_audio_sfx
widget_type: ProgressBar
defined_in: "02-design-system/widget-atoms.md §ProgressBar"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.audio.sfx_volume"
    default: "SFX Volume"
  - name: value
    type: f32 (0.0 ~ 1.0)
    source: "UiSettings.sfx_volume"
    default: 0.7
  - name: display_value
    type: String
    source: "computed: format!('{}%', (value * 100.0) as u32)"
    default: "70%"

outputs:
  - name: changed
    type: UiCommand::ChangeSettings
    payload: "{ sfx_volume: new_value }"
    trigger: OnValueChange

selection_model:
  type: none
```

### 7.16 settings_battle_speed

```yaml
widget_id: settings_battle_speed
widget_type: ProgressBar
defined_in: "02-design-system/widget-atoms.md §ProgressBar"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.battle.speed"
    default: "Battle Speed"
  - name: value
    type: f32 (0.5 ~ 3.0)
    source: "UiSettings.battle_speed_multiplier"
    default: 1.0
  - name: display_value
    type: String
    source: "computed: format!('{:.1}x', value)"
    default: "1.0x"
  - name: step
    type: f32
    source: "hardcoded"
    default: 0.1

outputs:
  - name: changed
    type: UiCommand::ChangeSettings
    payload: "{ battle_speed_multiplier: new_value }"
    trigger: OnValueChange

selection_model:
  type: none
```

### 7.17 settings_battle_tooltip

```yaml
widget_id: settings_battle_tooltip
widget_type: ProgressBar
defined_in: "02-design-system/widget-atoms.md §ProgressBar"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.battle.tooltip_delay"
    default: "Tooltip Delay"
  - name: value
    type: f32 (0 ~ 2000)
    source: "UiSettings.tooltip_delay_ms"
    default: 500
  - name: display_value
    type: String
    source: "computed: format!('{}ms', value as u32)"
    default: "500ms"
  - name: step
    type: f32
    source: "hardcoded"
    default: 50

outputs:
  - name: changed
    type: UiCommand::ChangeSettings
    payload: "{ tooltip_delay_ms: new_value }"
    trigger: OnValueChange

selection_model:
  type: none
```

### 7.18 settings_reset_btn

```yaml
widget_id: settings_reset_btn
widget_type: Button, Danger
defined_in: "02-design-system/widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.settings.reset"
    default: "Reset to Default"

outputs:
  - name: clicked
    type: UiCommand::OpenOverlay(OverlayType::Modal)
    payload: "OverlayPayload::ResetConfirm"
    trigger: OnLeftClick

selection_model:
  type: none
```

---

## 8. State Mapping (Per-Region)

> 每个 region 独立的状态。SettingsScreen 无异步数据加载，仅 Normal 状态适用。
> Loading / Empty / Error 状态标记为 N/A（不适用）。

### 8.1 settings_header

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 标题和关闭按钮为静态 LocalizedKey + 主题图标，无异步加载 | — | — |
| **Empty** | N/A — 标题始终有值，关闭按钮始终存在 | — | — |
| **Normal** | 标题文本 HeadingText + 关闭图标 IconButton 正常显示 | OnEnter 完成 | Fade(0.3s) 入场动画 |
| **Error** | N/A — 静态元素无错误状态 | — | — |

### 8.2 settings_tablist

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — Tab 按钮标签为静态 LocalizedKey，无异步加载 | — | — |
| **Empty** | N/A — 4 个 Tab 按钮始终存在 | — | — |
| **Normal** | 4 个 Tab 按钮垂直排列，当前激活 Tab 高亮显示 | OnEnter 完成 | 无过渡动画 |
| **Error** | N/A — Tab 列表无错误状态 | — | — |

### 8.3 settings_tabcontent

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 设置项值从 UiSettings 即时读取，无异步加载 | — | — |
| **Empty** | N/A — 每个 Tab 至少 2 个设置项，无空状态 | — | — |
| **Normal** | 显示当前激活 Tab 对应的设置项控件，值已填充至 UiSettings 当前值 | Tab 切换后 / OnEnter 完成 | Tab 内容淡入切换 (Fade 0.15s) |
| **Error** | N/A — 设置项无错误状态 | — | — |

### 8.4 settings_footer

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 重置按钮标签为静态 LocalizedKey | — | — |
| **Empty** | N/A — 重置按钮始终存在 | — | — |
| **Normal** | 重置按钮正常显示，可点击 | OnEnter 完成 | Fade(0.3s) 入场动画 |
| **Error** | N/A — 重置按钮无错误状态 | — | — |

---

## 9. Focus Navigation

> Tab 导航路径。按 Tab 键的顺序就是导航路径的顺序。
> 注意：焦点路径跟随当前激活 Tab 变化。焦点进入 TabContent 区域后，仅遍历当前激活 Tab 的控件。

```yaml
focus_path:
  ## 阶段 1: Tab 按钮
  - settings_tablist_gameplay    # Tab 1 — 默认焦点
  - settings_tablist_graphics    # Tab 2
  - settings_tablist_audio       # Tab 3
  - settings_tablist_battle      # Tab 4

  ## 阶段 2: 当前激活 Tab 的内容控件（以 Gameplay Tab 为例）
  - settings_gameplay_damage     # Tab 5 — 仅 Gameplay Tab 激活时
  - settings_gameplay_minimap    # Tab 6
  - settings_grid_toggle         # Tab 7
  - settings_gameplay_autobattle # Tab 8

  ## Graphics Tab 激活时替代 Tab 5-8:
  # - settings_graphics_theme    # Tab 5
  # - settings_graphics_language # Tab 6

  ## Audio Tab 激活时替代 Tab 5-8:
  # - settings_audio_master      # Tab 5
  # - settings_audio_bgm         # Tab 6
  # - settings_audio_sfx         # Tab 7

  ## Battle Tab 激活时替代 Tab 5-8:
  # - settings_battle_speed      # Tab 5
  # - settings_battle_tooltip    # Tab 6

  ## 阶段 3: Footer
  - settings_reset_btn           # Tab 最后 — 所有 Tab 统一

special_keys:
  Escape: "触发 settings_header_close.clicked，返回前一 Screen"
  Enter: "激活当前焦点按钮 / 确认当前选择"
  ArrowUp: "焦点上移（列表/滑块组内逆序遍历）"
  ArrowDown: "焦点下移（列表/滑块组内顺序遍历）"
  ArrowLeft: "音量/速度滑块值减少（ProgressBar 类型控件减量）"
  ArrowRight: "音量/速度滑块值增加（ProgressBar 类型控件增量）"
  Tab: "按 focus_path 顺序前进"
  Shift+Tab: "按 focus_path 逆序后退"

focus_trap: true          # true = 焦点锁定在该 Screen 内，Tab 循环
```

### 9.1 默认焦点

进入 SettingsScreen 时，默认焦点落在 `settings_tablist_gameplay`（第一个 Tab 按钮）。
当从 Overlay（ModalOverlay）返回时，焦点回到触发该 Overlay 的按钮（`settings_reset_btn`）。

---

## 10. Interaction Zones

> 每个可交互区域的行为定义。

### 10.1 settings_header_close

```yaml
zone_id: settings_header_close
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::CloseScreen"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "按钮变为 Hover 样式（IconButton 高亮变体）"
    leave_effect: "恢复 IconButton 默认样式"
    delay: 0ms
```

### 10.2 settings_tablist_gameplay

```yaml
zone_id: settings_tablist_gameplay
interactions:
  - type: click
    button: Left
    effect: "激活 Gameplay Tab，隐藏其他 Tab 内容，显示 gameplay 设置项"
    cursor: Pointer
    conditions:
      - tab != already_active: "仅当 Gameplay Tab 未激活时切换"

  - type: hover
    enter_effect: "Tab 按钮变为 Hover 样式（TabButton 高亮变体）"
    leave_effect: "恢复 TabButton 默认样式（激活态除外）"
    delay: 0ms
```

### 10.3 settings_tablist_graphics

```yaml
zone_id: settings_tablist_graphics
interactions:
  - type: click
    button: Left
    effect: "激活 Graphics Tab，隐藏其他 Tab 内容，显示 graphics 设置项"
    cursor: Pointer
    conditions:
      - tab != already_active

  - type: hover
    enter_effect: "Tab 按钮变为 Hover 样式（TabButton 高亮变体）"
    leave_effect: "恢复 TabButton 默认样式（激活态除外）"
    delay: 0ms
```

### 10.4 settings_tablist_audio

```yaml
zone_id: settings_tablist_audio
interactions:
  - type: click
    button: Left
    effect: "激活 Audio Tab，隐藏其他 Tab 内容，显示 audio 设置项"
    cursor: Pointer
    conditions:
      - tab != already_active

  - type: hover
    enter_effect: "Tab 按钮变为 Hover 样式（TabButton 高亮变体）"
    leave_effect: "恢复 TabButton 默认样式（激活态除外）"
    delay: 0ms
```

### 10.5 settings_tablist_battle

```yaml
zone_id: settings_tablist_battle
interactions:
  - type: click
    button: Left
    effect: "激活 Battle Tab，隐藏其他 Tab 内容，显示 battle 设置项"
    cursor: Pointer
    conditions:
      - tab != already_active

  - type: hover
    enter_effect: "Tab 按钮变为 Hover 样式（TabButton 高亮变体）"
    leave_effect: "恢复 TabButton 默认样式（激活态除外）"
    delay: 0ms
```

### 10.6 settings_gameplay_damage

```yaml
zone_id: settings_gameplay_damage
interactions:
  - type: click
    button: Left
    effect: "切换 Toggle 状态（on→off / off→on），触发 UiCommand::ChangeSettings({ show_damage_numbers: new_value })"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "Toggle 变为 Hover 样式"
    leave_effect: "恢复 Toggle 默认样式"
    delay: 0ms
```

### 10.7 settings_gameplay_minimap

```yaml
zone_id: settings_gameplay_minimap
interactions:
  - type: click
    button: Left
    effect: "切换 Toggle 状态，触发 UiCommand::ChangeSettings({ show_minimap: new_value })"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "Toggle 变为 Hover 样式"
    leave_effect: "恢复 Toggle 默认样式"
    delay: 0ms
```

### 10.8 settings_gameplay_autobattle

```yaml
zone_id: settings_gameplay_autobattle
interactions:
  - type: click
    button: Left
    effect: "切换 Toggle 状态，触发 UiCommand::ChangeSettings({ auto_battle: new_value })"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "Toggle 变为 Hover 样式"
    leave_effect: "恢复 Toggle 默认样式"
    delay: 0ms
```

### 10.9 settings_grid_toggle

```yaml
zone_id: settings_grid_toggle
interactions:
  - type: click
    button: Left
    effect: "切换 Toggle 状态，触发 UiCommand::ChangeSettings({ show_grid: new_value })"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "Toggle 变为 Hover 样式"
    leave_effect: "恢复 Toggle 默认样式"
    delay: 0ms
```

### 10.10 settings_graphics_theme

```yaml
zone_id: settings_graphics_theme
interactions:
  - type: click
    button: Left
    effect: "展开/收起 SelectList 下拉选项列表"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "SelectList 变为 Hover 样式"
    leave_effect: "恢复 SelectList 默认样式"
    delay: 0ms
```

### 10.11 settings_graphics_language

```yaml
zone_id: settings_graphics_language
interactions:
  - type: click
    button: Left
    effect: "展开/收起 SelectList 下拉选项列表"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "SelectList 变为 Hover 样式"
    leave_effect: "恢复 SelectList 默认样式"
    delay: 0ms
```

### 10.12 settings_audio_master

```yaml
zone_id: settings_audio_master
interactions:
  - type: click + drag
    button: Left
    effect: "拖拽滑块手柄改变音量值 (0.0~1.0)，实时触发 UiCommand::ChangeSettings({ master_volume: new_value })"
    cursor: Pointer (水平拖拽)
    conditions: []

  - type: hover
    enter_effect: "滑块轨道高亮，手柄放大"
    leave_effect: "恢复滑块默认样式"
    delay: 0ms
```

### 10.13 settings_audio_bgm

```yaml
zone_id: settings_audio_bgm
interactions:
  - type: click + drag
    button: Left
    effect: "拖拽滑块手柄改变音量值 (0.0~1.0)，实时触发 UiCommand::ChangeSettings({ bgm_volume: new_value })"
    cursor: Pointer (水平拖拽)
    conditions: []

  - type: hover
    enter_effect: "滑块轨道高亮，手柄放大"
    leave_effect: "恢复滑块默认样式"
    delay: 0ms
```

### 10.14 settings_audio_sfx

```yaml
zone_id: settings_audio_sfx
interactions:
  - type: click + drag
    button: Left
    effect: "拖拽滑块手柄改变音量值 (0.0~1.0)，实时触发 UiCommand::ChangeSettings({ sfx_volume: new_value })"
    cursor: Pointer (水平拖拽)
    conditions: []

  - type: hover
    enter_effect: "滑块轨道高亮，手柄放大"
    leave_effect: "恢复滑块默认样式"
    delay: 0ms
```

### 10.15 settings_battle_speed

```yaml
zone_id: settings_battle_speed
interactions:
  - type: click + drag
    button: Left
    effect: "拖拽滑块手柄改变战斗速度倍率 (0.5~3.0, step 0.1)，实时触发 UiCommand::ChangeSettings({ battle_speed_multiplier: new_value })"
    cursor: Pointer (水平拖拽)
    conditions: []

  - type: hover
    enter_effect: "滑块轨道高亮，手柄放大，显示实时倍率值"
    leave_effect: "恢复滑块默认样式"
    delay: 0ms
```

### 10.16 settings_battle_tooltip

```yaml
zone_id: settings_battle_tooltip
interactions:
  - type: click + drag
    button: Left
    effect: "拖拽滑块手柄改变延迟值 (0~2000ms, step 50)，实时触发 UiCommand::ChangeSettings({ tooltip_delay_ms: new_value })"
    cursor: Pointer (水平拖拽)
    conditions: []

  - type: hover
    enter_effect: "滑块轨道高亮，手柄放大，显示实时延迟值"
    leave_effect: "恢复滑块默认样式"
    delay: 0ms
```

### 10.17 settings_reset_btn

```yaml
zone_id: settings_reset_btn
interactions:
  - type: click
    button: Left
    effect: "弹出 ModalOverlay（重置确认弹窗：'确定要重置所有设置为默认值？' + 确认/取消按钮）"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "按钮变为 Danger Hover 样式（背景变亮/边框高亮）"
    leave_effect: "恢复 Danger 默认样式"
    delay: 0ms
```

---

## 11. Overlay Definition

> Overlay 列表 + Z-Layer。Z-Layer 分配遵循 `07-specs/references/z-layer-spec.md`。

| Overlay | 用途 | Z-Layer | 类型 | 触发条件 |
|---------|------|---------|------|---------|
| ModalOverlay | 重置设置为默认值的二次确认（"确定重置？" + 确认/取消按钮） | popup_layer (300) | Modal | 点击 `settings_reset_btn` |

### 11.1 Z-Layer 分配

| Z-Layer | 用途 | 包含 |
|---------|------|------|
| 0 | Screen 主界面层 | `settings_root` 及所有子 region |
| 1 | Tooltip 层 | 预留（设置项说明 Tooltip） |
| 2 | Notification 层 | 预留 |
| 3 | Modal 层 | ModalOverlay（重置确认） |
| 4 | Popup 层 | 预留（SelectList 下拉展开可申请此层子层） |
| 9 | Debug 层 | DebugOverlay (FPS/日志) |

### 11.2 Overlay 生命周期

| Overlay | OnOpen | OnClose | 依赖 |
|---------|--------|---------|------|
| ModalOverlay (ResetConfirm) | 创建 Modal 实体：半透明遮罩 (z=299) + 弹窗面板 (z=300) + "确认重置"按钮 (Danger) + "取消"按钮 (Secondary)，焦点锁定到弹窗内 | 确认: 执行重置（UiCommand::ChangeSettings 发送默认值），销毁 Modal 实体，焦点回到 settings_reset_btn；取消: 销毁 Modal 实体，焦点回到 settings_reset_btn | `UiSettings` 默认值常量 |

### 11.3 ModalOverlay 内部结构

```
ModalOverlay (ResetConfirm)
├── ModalBackdrop    [modal_backdrop: Panel — 半透明遮罩, z=299]
└── ModalDialog      [modal_dialog: Container — 居中弹窗, z=300]
    ├── ModalTitle   [modal_title: HeadingText]     — "ui.settings.modal.reset_confirm_title"
    ├── ModalBody    [modal_body: BodyText]          — "ui.settings.modal.reset_confirm_body"
    ├── ConfirmBtn   [modal_confirm_btn: Button, Danger]     — "ui.settings.modal.reset_confirm_yes"
    └── CancelBtn    [modal_cancel_btn: Button, Secondary]   — "ui.settings.modal.reset_confirm_no"
```

---

## 12. Lifecycle

> Screen 的完整生命周期行为。遵守 `screen-lifecycle.md` 定义的状态机。

| 阶段 | 行为 | 触发条件 | 清理 |
|------|------|---------|------|
| **OnEnter** | `spawn_settings_screen()` — 生成完整 UI 树（settings_root -> Header + TabPanel + Footer），从 UiSettings 资源读取当前所有配置项值，初始化各 Widget 的状态 | `GameState::Settings` 状态进入 | — |
| **OnReady** | 注册 Theme Resource Observer（用于主题切换即时生效的全局刷新标记） | OnEnter 完成，UI 树就绪 | — |
| **Active** | 等待设置项变更事件（Toggle / SelectList / ProgressBar 交互），通过 UiCommand::ChangeSettings 即时下发更新 | OnReady 完成 | — |
| **OnExit** | `despawn_settings_screen()` — 清理所有标记了 `With<SettingsScreen>` 的实体 | `GameState::Settings` 状态退出 | 清理标记: `With<SettingsScreen>` |

### 12.1 生命周期事件处理

```yaml
on_enter:
  - action: "spawn_ui_tree"
    spawner: "spawn_settings_screen()"
    description: "生成 SettingsScreen 完整 UI 树：HeaderPanel(标题+关闭按钮) + TabPanel(TabList+TabContent) + Footer(重置按钮)"
  - action: "read_settings"
    source: "UiSettings resource"
    description: "从 UiSettings 资源读取所有设置项的当前值，初始化各 Widget 的初始状态"

on_ready:
  - action: "register_observer"
    target: "Theme Resource changed"
    handler: "当主题 Resource 被替换时，标记全局 Widget 刷新（重绘颜色/字体 Token）"

active:
  - trigger: "settings_tablist_{id}.selected"
    action: "切换 TabContent 显示对应的 settings_{tab} 容器，隐藏其他 Tab 内容"
    scope: "settings_tabcontent"
  - trigger: "{widget}.toggle / {widget}.changed / {widget}.selected"
    action: "构建 UiCommand::ChangeSettings(UiSettings) 并下发至 Domain（UiLayer），变更即时生效"
    scope: "所有设置项 Widget"
  - trigger: "settings_reset_btn.clicked"
    action: "弹出 ModalOverlay（重置确认），阻止下层交互"
    scope: "settings_footer → ModalOverlay"
  - trigger: "ModalOverlay.confirm"
    action: "将 UiSettings 所有字段重置为默认值，下发 UiCommand::ChangeSettings(default_UiSettings) 并销毁 Modal"
    scope: "ModalOverlay"
  - trigger: "ModalOverlay.cancel"
    action: "销毁 ModalOverlay，焦点回到 settings_reset_btn"
    scope: "ModalOverlay"

on_exit:
  - action: "unregister_observer"
    target: "Theme Resource observer"
  - action: "despawn_ui_tree"
    query: "With<SettingsScreen>"
    description: "清理所有标记了 SettingsScreen 组件的实体（不保存 UiSettings——变更已即时持久化）"
```

---

## 13. Data Ownership

> Owns / Uses 分离。SettingsScreen 通过 UiSettings resource 直接读写，不经过 ViewModel。

### 13.1 ViewModel 映射

| ViewModel | 字段 | 归属 (Owns/Uses) | 更新频率 | Projection 源 |
|-----------|------|-----------------|---------|--------------|
| 无 | UiSettings (直接读 Resource) | Uses — 全局唯一的配置资源 | 每帧 / 用户交互触发 | 无（直接操作 Resource） |

### 13.2 数据流

```
应用启动 → UiSettings (Resource) 从配置文件加载
                               ↓
                SettingsScreen OnEnter 读取当前值初始化 Widget
                               ↓
        用户交互 → Widget 输出事件 → UiCommand::ChangeSettings(partial)
                               ↓
                UiLayer 处理 → 更新 UiSettings Resource
                               ↓
                （可选）Domain 监听 UiSettings 变更 → 响应式调整（如 BGM 音量实时变化）
                               ↓
        重置操作 → ModalOverlay 确认 → UiCommand::ChangeSettings(defaults)
```

### 13.3 设置项字段映射

| UiSettings 字段 | Widget | 类型 | 默认值 |
|----------------|--------|------|--------|
| `show_damage_numbers` | `settings_gameplay_damage` | bool | true |
| `show_minimap` | `settings_gameplay_minimap` | bool | true |
| `show_grid` | `settings_grid_toggle` | bool | true |
| `auto_battle` | `settings_gameplay_autobattle` | bool | false |
| `theme_index` | `settings_graphics_theme` | usize | 0 (Dark) |
| `language_index` | `settings_graphics_language` | usize | 0 (English) |
| `master_volume` | `settings_audio_master` | f32 (0.0~1.0) | 0.8 |
| `bgm_volume` | `settings_audio_bgm` | f32 (0.0~1.0) | 0.7 |
| `sfx_volume` | `settings_audio_sfx` | f32 (0.0~1.0) | 0.7 |
| `battle_speed_multiplier` | `settings_battle_speed` | f32 (0.5~3.0) | 1.0 |
| `tooltip_delay_ms` | `settings_battle_tooltip` | f32 (0~2000) | 500 |

---

## 14. Layout Intent

> 每个关键尺寸的**理由说明**。为什么选这个尺寸而不是别的？

### 14.1 固定尺寸意图

| widget_id | 属性 | 值 | 意图 | shrink |
|-----------|------|----|------|--------|
| `settings_header` | height | 56px | "Header 栏高度 56px，略大于按钮高度 40px，提供充足上下内边距（8px），视觉舒适" | none |
| `settings_header_close` | width | 40px | "关闭按钮最小触摸目标 40x40，图标按钮无需更宽" | none |
| `settings_header_close` | height | 40px | "最小触摸目标 40px" | none |
| `settings_tablist` | width | 200px | "Tab 列表宽度 200px，Tab 标签 'Gameplay/Graphics/Audio/Battle' 中英文均能在单行内显示，无需折行" | none |
| `settings_tablist_*` | height | 44px | "Tab 按钮高度 44px，大于最小触摸目标 40px，4 个 Tab 总计 176px 远低于 720px 视口高度" | none |
| `settings_reset_btn` | width | 180px | "重置按钮宽度 180px，确保标签 'Reset to Default' 中英文单行显示（英文约 16 字符，中文 6 字）" | none |
| `settings_reset_btn` | height | 40px | "最小触摸目标 40px" | none |
| `settings_footer` | height | 56px | "Footer 高度 56px，与 Header 等高，视觉对称，包裹按钮后上下留 8px 内边距" | none |
| `settings_gameplay_damage` | height | 40px | "Toggle 行高 40px，最小触摸目标，每行含标签+Toggle 开关" | none |
| `settings_gameplay_minimap` | height | 40px | "与上同，保持列表项视觉对齐" | none |
| `settings_grid_toggle` | height | 40px | "与上同" | none |
| `settings_gameplay_autobattle` | height | 40px | "与上同" | none |

### 14.2 弹性尺寸意图

| widget_id | flex_grow | 理由 |
|-----------|-----------|------|
| `settings_header_title` | 1 | "标题弹性增长，将关闭按钮推至右侧（Space-between 等效行为）" |
| `settings_tabpanel` | 1 | "TabPanel 占据 Header 和 Footer 之外的所有垂直空间，内容区越大越好" |
| `settings_tabcontent` | 1 | "内容区弹性填充 TabList 右侧的剩余水平空间，越宽越便于显示长文本设置项" |

### 14.3 通用约束

```yaml
global:
  min_interactive_height: 40px   # 可交互元素最小高度 (触摸友好)
  min_interactive_width: 40px    # 可交互元素最小宽度 (触摸友好)
  standard_padding: 8px          # 标准内边距
  standard_gap: 8px              # 标准间距 (Flexbox gap)
  tablist_content_gap: 0px       # TabList 与 TabContent 之间无间距（相邻视觉）
```

---

## 15. Scroll & Overflow Policy

> SettingsScreen 的 TabContent 区域可能因设置项过多而需要垂直滚动。

### 15.1 滚动区域

| widget_id | 方向 | Scroll Policy | Overflow Policy | 理由 |
|-----------|------|--------------|---------------|------|
| `settings_tabcontent` | vertical | auto | clip | "Audio Tab 有 3 个滑块 + 标签说明，总高度可能超出可视区域，需垂直滚动；仅在内容溢出时显示滚动条" |
| `settings_gameplay` | vertical | none | clip | "Gameplay Tab 仅 4 个 Toggle (4x40=160px)，远低于视口高度，无需滚动" |
| `settings_graphics` | vertical | none | clip | "Graphics Tab 仅 2 个 SelectList，auto 高度，无需滚动" |
| `settings_audio` | vertical | auto | clip | "Audio Tab 3 个滑块 + 标签，可能溢出，支持自动滚动" |
| `settings_battle` | vertical | none | clip | "Battle Tab 仅 2 个滑块，auto 高度，无需滚动" |
| `settings_header` | horizontal | none | clip | "Header 内容固定，不会溢出" |
| `settings_tablist` | vertical | none | clip | "4 个 Tab 按钮 (4x44=176px) 远低于视口高度，无需滚动" |
| `settings_footer` | horizontal | none | clip | "Footer 仅 1 个按钮，不会溢出" |

### 15.2 文本溢出

| widget_id | max_lines | overflow | 多语言风险 |
|-----------|-----------|----------|-----------|
| `settings_header_title` | 1 | clip | "标题 'Settings' / '设置' 极短，无多语言溢出风险" |
| `settings_tablist_*` | 1 | ellipsis | "Tab 标签 'Gameplay' / 'Graphics' / 'Audio' / 'Battle' 中英文均短（最长 'Graphics' 8 字符），风险低" |
| `settings_gameplay_damage` | 1 | ellipsis | "标签 'Show Damage Numbers' 英文约 20 字符，中文约 6 字，200px 宽度足够，风险低" |
| `settings_gameplay_minimap` | 1 | ellipsis | "'Show Minimap' 短，风险低" |
| `settings_grid_toggle` | 1 | ellipsis | "'Show Grid' / '显示网格' 短，风险低" |
| `settings_gameplay_autobattle` | 1 | ellipsis | "'Auto Battle' / '自动战斗' 极短，风险低" |
| `settings_reset_btn` | 1 | ellipsis | "'Reset to Default' 英文约 16 字符，180px 宽度绰绰有余，风险低" |

---

## 16. Event Contract

> UI -> Domain 事件 + Domain -> UI 事件的完整契约。

### 16.1 UI -> Domain（通过 UiCommand 传递）

```yaml
CloseSettings:
  trigger_widget: "settings_header → settings_header_close → click"
  data: {}
  conditions: []
  emits: UiCommand::CloseScreen
  domain_event: "None — 纯 UI 导航，ScreenStack 直接处理"

ChangeSetting:
  trigger_widget: "settings_tabcontent → settings_* → toggle/change/select/drag"
  data:
    field: String          # 被修改的字段名 (如 "master_volume", "show_grid")
    new_value: Dynamic     # 新值 (bool / f32 / usize)
  conditions:
    - new_value != current_value  # 仅值变化时下发
  emits: UiCommand::ChangeSettings(UiSettings)
  domain_event: "UiSettingsChanged { field, old_value, new_value }"

ResetSettings:
  trigger_widget: "settings_footer → settings_reset_btn → click → ModalOverlay.confirm"
  data: {}
  conditions:
    - modal_confirm == true  # 仅确认后执行
  emits: UiCommand::ChangeSettings(UiSettings::default())
  domain_event: "UiSettingsReset"
```

### 16.2 Domain -> UI（通过 Projection 消费）

```yaml
# SettingsScreen 不消费 Domain Event。
# SettingsScreen 从 UiSettings Resource 直接读取配置，不经过 Projection/ViewModel。
# 所有交互均为 UI 自包含操作：读取 → 修改 → 写回 UiSettings Resource。
# Domain 监听 UiSettings 变更事件（UiSettingsChanged）做响应式调整（如音量变更→音频引擎调节）。
```

---

## 17. Screen Metrics

> 复杂度基线。所有数值初始创建时手动填写，后续 CI 阶段自动校验。

| Metric | 值 | P0/P1 | 说明 |
|--------|-----|-------|------|
| `widget_count` | 27 | P1 | Widget 实例总数（root + header_container + title + close btn + tabpanel + tablist + 4 tab btns + tabcontent + 4 content containers + 4 toggles + 2 selectlists + 3 volume sliders + 2 battle sliders + footer + reset btn） |
| `container_count` | 10 | P1 | 纯容器节点数（root + settings_header + settings_tabpanel + settings_tablist + settings_tabcontent + settings_gameplay + settings_graphics + settings_audio + settings_battle + settings_footer） |
| `interactive_count` | 17 | P1 | 可交互 Widget 数（close btn + 4 tab btns + 4 toggles + 2 selectlists + 3 volume sliders + 2 battle sliders + reset btn） |
| `overlay_count` | 1 | P1 | 关联的 Overlay 数（ModalOverlay 重置确认，含遮罩 + 弹窗面板 + 确认按钮 + 取消按钮） |
| `max_depth` | 5 | P1 | root → settings_tabpanel → settings_tabcontent → settings_audio → settings_audio_master |
| `max_children` | 4 | P1 | settings_tabcontent 有 4 个直接子节点（gameplay / graphics / audio / battle） |

### 17.1 Budget 检查

| 规则 | 阈值 | 当前值 | 状态 |
|------|------|--------|------|
| max_depth <= 6 | 6 | 5 | ✅ |
| max_children <= 20 | 20 | 4 | ✅ |
| interactive_count / widget_count >= 0.2 | 20% | 63% (17/27) | ✅ |

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
| D09 | State Mapping 完整 (每个 region 的 Loading/Empty/Normal/Error) | [x] | 全部标记为 N/A（Settings 无异步数据） |
| D10 | Focus Navigation 已定义 (Tab 路径完整) | [x] | P1，含 Tab 切换后的动态路径 |
| D11 | Interaction Zones 已定义 (Click/Hover/Drag) | [x] | |
| D12 | Overlay Definition 已定义 (Overlay 列表 + Z-Layer) | [x] | |
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
| `06-ui/02-design-system/widget-atoms.md` | 原子组件 Contract（HeadingText, IconButton, Toggle, ProgressBar, SelectList, Button） |
| `06-ui/02-design-system/widget-composites.md` | 复合组件 Contract（TabPanel, TabButton） |
| `06-ui/02-design-system/theme-localization.md` | StyleToken / Theme / UiTextKey |
| `06-ui/02-design-system/focus-binding.md` | Focusable / FocusGroup / Dirty<T> / UiBinding |
| `06-ui/03-screens/screens.md` | 设置 Screen 定义（§6） |
| `06-ui/03-screens/screen-lifecycle.md` | Screen 生命周期状态机 |
| `06-ui/04-data-flow/projection-viewmodel.md` | Projection / ViewModel 映射 |
| `03-content/localization/ui-screen-keys.md` | 设置菜单 LocalizationKeys |

---

## 附录 C: 架构审查记录

> **审查人**: @presentation-architect | **日期**: 2026-06-22 | **结论**: PASS — 全部阻塞项已解决，status 改为 active

### 通过确认

| 维度 | 结果 | 备注 |
|------|------|------|
| 模板合规 (17字段) | [x] | 全部 17 字段已填充 |
| 架构一致性 (vs screens.md §6) | [x] | ShowGridToggle 已补充; widget-id-map 已同步更新 |
| ASCII Wireframe (无匿名面板) | [x] | 所有区域已命名 |
| Widget ID 命名规范 (`settings_{region}_{element}`) | [x] | SSPEC 内一致; widget-id-map 遵循自有命名规范同步更新 |
| Flexbox Layout 完整性 | [x] | 5 必需字段齐全; shrink 字段不完整 (minor) |
| LocalizationKey 一致性 | [x] | 所有文本使用 LocalizedKey 模式 |
| Widget Contract 一致性 | [x] | step source 不精确 (minor) |
| 过约束检查 | [x] | 未指定视觉样式 |
| 单向数据流 | [x] | 无违反; UiSettings 是 UI 配置资源非 Domain |

### 未通过原因

| # | 优先级 | 问题 | 影响 |
|---|--------|------|------|
| 1 | FIXED | widget-id-map.md §4.5 已由 @content-architect 更新为 23 个新 ID，与 SSPEC 意图一致 | 2026-06-22 修复 |
| 2 | FIXED | ShowGridToggle 已补充 | 2026-06-22 @feature-developer 已修复: 在 Wireframe/WidgetTree/Flexbox/WidgetContract/Focus/Event 等 9 章节中新增 |
| 3 | MINOR | `settings_root` vs `settings_screen_root` 命名不一致 | 在 #1 更新时自然解决 |
| 4 | MINOR | Flexbox shrink 字段不完整 | 建议补全 |
| 5 | MINOR | `settings_battle_speed.step` source 为 "hardcoded" | 应引用配置常量 |

### 详细审查记录

#### 1. CRITICAL → FIXED — widget-id-map.md §4.5 已更新

原 SSPEC 定义了约 27 个 widget_id，而 `widget-id-map.md §4.5 (SettingsScreen)` 仅列着 13 个旧 ID。@content-architect 已于 2026-06-22 替换为完整的 23 个新 ID，与 SSPEC 意图一致。

旧表对照（保留用于追溯）：

| widget-id-map.md（旧，待更新） | SSPEC（新，权威） |
|---|---|
| `settings_screen_root` | `settings_root` |
| `settings_tab_bar` | `settings_tabpanel` + `settings_tablist` + `settings_tabcontent` |
| `settings_gameplay_tab_btn` | `settings_tablist_gameplay` |
| `settings_video_tab_btn` | `settings_tablist_graphics` |
| `settings_audio_tab_btn` | `settings_tablist_audio` |
| `settings_volume_master/bgm/sfx` | `settings_audio_master/bgm/sfx` |
| `settings_resolution_dropdown` | 不存在（从 SSPEC 移除） |
| `settings_fullscreen_toggle` | 不存在（从 SSPEC 移除） |
| `settings_language_dropdown` | `settings_graphics_language` |
| `settings_back_btn` | `settings_reset_btn` |

**已修复 (2026-06-22)**: `widget-id-map.md §4.5` 已替换为 23 个新 ID，与 SSPEC 对齐。

#### 2. CRITICAL → ✅ FIXED — ShowGridToggle 已补充

`screens.md §6.3 (SettingsScreen -> GameplayTab)` 列出 4 个 Toggle：
- ShowDamageToggle ✅ SSPEC 中存在
- ShowMinimapToggle ✅ SSPEC 中存在
- ShowGridToggle ✅ **SSPEC 中已补充** (`widget_id: settings_grid_toggle`)
- AutoBattleToggle ✅ SSPEC 中存在

`@feature-developer` 已按 screens.md 要求将 ShowGridToggle 加入 SSPEC 的以下 9 章节：
Wireframe (§2)、Widget Tree (§3)、Flexbox (§4)、Region Responsibility (§6)、Widget Contract (§7)、Focus Navigation (§9)、Interaction Zones (§10)、Data Ownership (§13)、Layout Intent (§14)。GameplayTab 现为 4 个 Toggle。

#### 3. MINOR — `settings_root` 命名不一致

SSPEC 使用 `settings_root`，但 widget-id-map.md 中类似页面级根节点的模式是 `battle_screen_root`、`inventory_screen_root`。改为 `settings_screen_root` 以保持一致，或确认有意简化。此问题在 widget-id-map 更新（见 #1）时会自然解决。

#### 4. MINOR — Flexbox shrink 字段不完整

部分 Flexbox YAML 条目包含 `shrink: none`（如 header、tab_btn、toggle），但其他条目缺失该字段（如 `settings_root`、`settings_tabpanel`、`settings_tabcontent`、`settings_gameplay` 等）。模板将 shrink 列为文档性注释，但应保持一致以免产生疑惑。建议所有条目都添加。

#### 5. MINOR — `step` source 为 "hardcoded" 不精确

`settings_battle_speed` Widget Contract（§7.15）：

```yaml
  - name: step
    type: f32
    source: "hardcoded"      # ← 不精确，应引用配置常量
    default: 0.1
```

`source: "hardcoded"` 没有说明该值在代码库中的定义位置。应引用具体的配置常量或定义文件。

### 通过条件（已满足）

以上 #1 已于 2026-06-22 由 @content-architect 修复。DoD 全部 18 项已核对通过。status 已改为 `active`。

---

*本文档是 SettingsScreen SSPEC，由 @feature-developer 根据 `07-specs/screen-spec-template.md` 模板创建。所有 17 个字段已填充。当前 status: active（2026-06-22 审查通过）。*
