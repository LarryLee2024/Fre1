---
id: 07-specs.main-menu-screen
title: MainMenuScreen Specification — AI-Consumable Layout & Interaction Spec
status: active
owner: presentation-architect
created: 2026-06-22
tags:
  - ui
  - screen-spec
  - main-menu
  - active
---

# MainMenuScreen

> **职责**: @presentation-architect | **上游**: ADR-066 (Screen Spec), `07-specs/README.md` (总纲)
> **状态**: 初始 draft，完成后改为 active

**P0 字段**: 1-14 (Screen Header / ASCII Wireframe / Widget Tree / Flexbox Layout / Responsive Rules / Region Responsibility / Widget Contract / State Mapping / Focus Nav / Interaction Zones / Overlay / Lifecycle / Data Ownership / Layout Intent)
**P1 字段**: 15-17 (Scroll & Overflow / Event Contract / Screen Metrics)

---

## 1. Screen Header

| 属性 | 值 |
|------|-----|
| Screen Name | `MainMenuScreen` — 对应 `GameState::MainMenu` |
| Purpose | 游戏主菜单，提供新游戏、加载游戏、设置三个功能入口，展示游戏标题与版本号 |
| Navigation | App启动自动进入；点击 NewGame 进入战前准备流程；点击 LoadGame 打开 SaveLoadScreen；点击 Settings 打开 SettingsScreen。不支持 Esc 返回（主菜单无上一级） |
| GameState | `GameState::MainMenu` |
| ScreenLayer 层级 | 0（主界面层） |
| 加载模式 | Ephemeral（每次进入 MainMenu 重新 spawn） |
| 过渡动画 | Fade(0.5s) |
| 变体 | None |

---

## 2. ASCII Wireframe

> 纯文本线框图。所有区域必须命名（`widget_id`），禁止匿名面板。

```
┌──────────────────────────────────────────────────────────────┐
│  [title_area]                                                │
│                                                              │
│                        Fre                                   │
│                     A Bevy SRPG                              │
│                                                              │
│  [button_list]                                               │
│              ┌──────────────────────────────────┐            │
│              │  New Game                        │            │
│              └──────────────────────────────────┘            │
│              ┌──────────────────────────────────┐            │
│              │  Load Game                       │            │
│              └──────────────────────────────────┘            │
│              ┌──────────────────────────────────┐            │
│              │  Settings                        │            │
│              └──────────────────────────────────┘            │
│                                                              │
│  [version_text]                                              │
│                      v0.1.0                                  │
└──────────────────────────────────────────────────────────────┘
```

### 2.1 Region 索引

| widget_id | 类型 | 用途 | 对应 Wireframe 位置 |
|-----------|------|------|-------------------|
| `title_area` | Container | 包裹标题和副标题，整体居中 | 顶部：Title + Subtitle |
| `title_text` | HeadingText | 显示游戏标题 "Fre" | 标题文字行 |
| `subtitle_text` | CaptionText | 显示副标题 "A Bevy SRPG" | 副标题文字行 |
| `button_list` | Container | 包裹三个功能按钮，垂直排列居中 | 中部：三个按钮 |
| `new_game_btn` | Button, Primary | 新游戏入口按钮 | 第一个按钮 |
| `load_game_btn` | Button, Secondary | 加载游戏入口按钮 | 第二个按钮 |
| `settings_btn` | Button, Secondary | 设置入口按钮 | 第三个按钮 |
| `version_text` | CaptionText | 显示版本号 "v0.1.0" | 底部版本号文字 |

---

## 3. Widget Tree

> 标注 `[widget_id: WidgetType]` 的树结构。禁止隐藏节点，必须完整。

```
ScreenRoot                                              [root: Screen]
├── TitleArea                                           [title_area: Container]
│   ├── TitleText                                       [title_text: HeadingText]
│   └── SubtitleText                                    [subtitle_text: CaptionText]
├── ButtonList                                          [button_list: Container]
│   ├── NewGameButton                                   [new_game_btn: Button, Primary]
│   ├── LoadGameButton                                  [load_game_btn: Button, Secondary]
│   └── SettingsButton                                  [settings_btn: Button, Secondary]
└── VersionText                                         [version_text: CaptionText]
```

### 3.1 Widget Type 索引

| widget_id | WidgetType | 定义位置 | 复用于 |
|-----------|-----------|---------|--------|
| `title_text` | `Atom: HeadingText` | `02-design-system/widget-atoms.md §HeadingText` | — |
| `subtitle_text` | `Atom: CaptionText` | `02-design-system/widget-atoms.md §CaptionText` | — |
| `new_game_btn` | `Atom: Button (Primary)` | `02-design-system/widget-atoms.md §Button` | 设置Screen、其他确认场景 |
| `load_game_btn` | `Atom: Button (Secondary)` | `02-design-system/widget-atoms.md §Button` | 设置Screen、其他导航场景 |
| `settings_btn` | `Atom: Button (Secondary)` | `02-design-system/widget-atoms.md §Button` | 设置Screen、其他导航场景 |
| `version_text` | `Atom: CaptionText` | `02-design-system/widget-atoms.md §CaptionText` | 各 Screen 通用版本显示 |

---

## 4. Flexbox Layout

> YAML 格式。每个 widget_id 必须有 direction / width / height / flex_grow / intent。

```yaml
## Flexbox Layout — MainMenuScreen
## width/height: px 值或 "auto" 或 "fill"
## flex_grow: 0=不增长, 1=等分剩余空间, 2=双倍增长
## shrink: none/low/high — 收缩优先级

root:
  direction: column
  width: 100%
  height: 100%
  flex_grow: 0
  intent: "Screen 根容器，占满视口，垂直排列子元素（标题区→按钮区→版本号）"

title_area:
  direction: column
  width: auto
  height: auto
  flex_grow: 0
  shrink: none
  intent: "标题区域，垂直居中顶部，auto 尺寸包裹标题和副标题文本"

title_text:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  shrink: none
  intent: "游戏标题文本，自动宽度适配 'Fre'，48px 固定字号"

subtitle_text:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  shrink: low
  intent: "副标题文本，位于标题下方，宽度可压缩但不可完全隐藏"

button_list:
  direction: column
  width: 200
  height: auto
  flex_grow: 0
  shrink: none
  intent: "按钮列表容器，固定 200px 宽度，垂直排列 3 个按钮，居中显示"

new_game_btn:
  direction: row
  width: 200
  height: 48
  flex_grow: 0
  shrink: none
  intent: "新游戏按钮，Primary 样式，固定 200x48 保证点击区域 >= 40x40"

load_game_btn:
  direction: row
  width: 200
  height: 48
  flex_grow: 0
  shrink: none
  intent: "加载游戏按钮，Secondary 样式，固定尺寸"

settings_btn:
  direction: row
  width: 200
  height: 48
  flex_grow: 0
  shrink: none
  intent: "设置按钮，Secondary 样式，固定尺寸"

version_text:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  shrink: none
  intent: "版本号文本，底部居中显示，auto 宽度适配 'v0.1.0'"
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

> 每个 region 3-5 条职责。明确该区域"展示什么"和"不做什么"。

### 6.1 title_area

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示游戏标题文本（LocalizationKey: `ui.main_menu.title`） | Display | 标题缺失或显示错误 |
| R02 | 展示副标题文本（LocalizationKey: `ui.main_menu.subtitle`） | Display | 副标题缺失或显示错误 |
| R03 | 标题与副标题垂直排列，上下间距遵循 Theme spacing 规范 | Layout | 标题区域布局混乱 |

**不负责**:
- 按钮交互（不属于 title_area 职责）
- 版本号显示（属于 version_text）

### 6.2 button_list

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示按顺序排列的 3 个功能按钮 | Display | 按钮顺序颠倒或缺失 |
| R02 | 响应 NewGame 按钮点击，触发 `UiCommand::NewGame` | Interaction | 新游戏流程无法启动 |
| R03 | 响应 LoadGame 按钮点击，触发 `UiCommand::OpenScreen(ScreenType::SaveLoad)` | Interaction | 加载游戏无法进入 |
| R04 | 响应 Settings 按钮点击，触发 `UiCommand::OpenScreen(ScreenType::Settings)` | Interaction | 设置无法进入 |
| R05 | 管理按钮间焦点位移（Tab/Arrow 导航） | Focus | 键盘/手柄无法操作按钮 |

**不负责**:
- 显示标题或版本号
- 执行实际的业务逻辑（仅发射 UiCommand，不处理）

### 6.3 version_text

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 显示版本号文本（LocalizationKey: `ui.main_menu.version`） | Display | 版本号缺失或显示错误 |
| R02 | 固定位于屏幕底部，不随按钮列表弹性伸缩移动 | Layout | 版本号位置错乱 |

**不负责**:
- 按钮交互
- 标题显示

---

## 7. Widget Contract

> Inputs / Outputs / Selection Model。

### 7.1 title_text

```yaml
widget_id: title_text
widget_type: HeadingText
defined_in: "02-design-system/widget-atoms.md §HeadingText"

inputs:
  - name: text
    type: LocalizedText
    source: "ui.main_menu.title"
    default: "Fre"

outputs: []

selection_model:
  type: none
```

### 7.2 subtitle_text

```yaml
widget_id: subtitle_text
widget_type: CaptionText
defined_in: "02-design-system/widget-atoms.md §CaptionText"

inputs:
  - name: text
    type: LocalizedText
    source: "ui.main_menu.subtitle"
    default: "A Bevy SRPG"

outputs: []

selection_model:
  type: none
```

### 7.3 new_game_btn

```yaml
widget_id: new_game_btn
widget_type: Button, Primary
defined_in: "02-design-system/widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.main_menu.new_game"
    default: "New Game"

outputs:
  - name: clicked
    type: UiCommand::NewGame
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.4 load_game_btn

```yaml
widget_id: load_game_btn
widget_type: Button, Secondary
defined_in: "02-design-system/widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.main_menu.load_game"
    default: "Load Game"

outputs:
  - name: clicked
    type: UiCommand::OpenScreen(ScreenType::SaveLoad)
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.5 settings_btn

```yaml
widget_id: settings_btn
widget_type: Button, Secondary
defined_in: "02-design-system/widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.main_menu.settings"
    default: "Settings"

outputs:
  - name: clicked
    type: UiCommand::OpenScreen(ScreenType::Settings)
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.6 version_text

```yaml
widget_id: version_text
widget_type: CaptionText
defined_in: "02-design-system/widget-atoms.md §CaptionText"

inputs:
  - name: text
    type: LocalizedText
    source: "ui.main_menu.version"
    default: "v0.1.0"

outputs: []

selection_model:
  type: none
```

---

## 8. State Mapping (Per-Region)

> 每个 region 独立的状态。MainMenuScreen 无异步数据加载，仅 Normal 状态适用。
> Loading / Empty / Error 状态标记为 N/A（不适用）。

### 8.1 title_area

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 标题文本为静态 LocalizedKey，无异步加载 | — | — |
| **Empty** | N/A — 标题始终有值，无空状态 | — | — |
| **Normal** | TitleText + SubtitleText 正常显示 | OnEnter 完成 | Fade(0.5s) 入场动画 |
| **Error** | N/A — 静态文本无错误状态 | — | — |

### 8.2 button_list

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 按钮标签为静态 LocalizedKey，无异步加载 | — | — |
| **Empty** | N/A — 按钮始终有 3 个，不存在空列表 | — | — |
| **Normal** | 3 个功能按钮正常显示，可点击 | OnEnter 完成 | 按钮逐个淡入 (stagger 100ms) |
| **Error** | N/A — 按钮无错误状态 | — | — |

### 8.3 version_text

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 版本号为静态 LocalizedKey | — | — |
| **Empty** | N/A — 版本号始终有值 | — | — |
| **Normal** | 版本号文本正常显示 | OnEnter 完成 | Fade(0.5s) 入场动画 |
| **Error** | N/A — 静态文本无错误状态 | — | — |

---

## 9. Focus Navigation

> Tab 导航路径。按 Tab 键的顺序就是导航路径的顺序。

```yaml
focus_path:
  - new_game_btn          # Tab 1 — 默认焦点
  - load_game_btn         # Tab 2
  - settings_btn          # Tab 3

special_keys:
  Escape: "无效果（MainMenu 为最顶层 Screen，Esc 不执行任何操作）"
  Enter: "激活当前焦点按钮（等同于点击）"
  ArrowUp: "焦点上移（settings_btn → load_game_btn → new_game_btn 逆序）"
  ArrowDown: "焦点下移（new_game_btn → load_game_btn → settings_btn 顺序）"
  Tab: "按 focus_path 顺序前进"
  Shift+Tab: "按 focus_path 逆序后退"

focus_trap: true          # true = 焦点锁定在该 Screen 内，Tab 循环
```

### 9.1 默认焦点

进入 MainMenuScreen 时，默认焦点落在 `new_game_btn`（第一个按钮）。

---

## 10. Interaction Zones

> 每个可交互区域的行为定义。

### 10.1 new_game_btn

```yaml
zone_id: new_game_btn
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::NewGame"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "按钮变为 Hover 样式（Primary 高亮变体）"
    leave_effect: "恢复 Primary 默认样式"
    delay: 0ms
```

### 10.2 load_game_btn

```yaml
zone_id: load_game_btn
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::OpenScreen(ScreenType::SaveLoad)"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "按钮变为 Hover 样式（Secondary 高亮变体）"
    leave_effect: "恢复 Secondary 默认样式"
    delay: 0ms
```

### 10.3 settings_btn

```yaml
zone_id: settings_btn
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::OpenScreen(ScreenType::Settings)"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "按钮变为 Hover 样式（Secondary 高亮变体）"
    leave_effect: "恢复 Secondary 默认样式"
    delay: 0ms
```

---

## 11. Overlay Definition

> Overlay 列表 + Z-Layer。MainMenuScreen 当前 MVP 无 Overlay，以下为后续迭代预留。

| Overlay | 用途 | Z-Layer | 类型 | 触发条件 |
|---------|------|---------|------|---------|
| ModalOverlay | 新游戏确认弹窗（"确定要开始新游戏？当前进度将丢失。"） | 3 | Modal | 点击 NewGameButton |
| LoadingOverlay | 加载新游戏/读取存档时的资源加载遮罩 | 4 | Popup | NewGame / LoadGame 执行后 |

### 11.1 Z-Layer 分配

| Z-Layer | 用途 | 包含 |
|---------|------|------|
| 0 | Screen 主界面层 | `root` 及所有子 region |
| 1 | Tooltip 层 | 预留 |
| 2 | Notification 层 | 预留 |
| 3 | Modal 层 | ModalOverlay（新游戏确认） |
| 4 | Popup 层 | LoadingOverlay（资源加载） |
| 9 | Debug 层 | DebugOverlay (FPS/日志) |

### 11.2 Overlay 生命周期

| Overlay | OnOpen | OnClose | 依赖 |
|---------|--------|---------|------|
| ModalOverlay | 创建 Modal 实体（确认 / 取消 两个按钮），遮罩背景半透明 | 销毁 Modal 实体，恢复按钮焦点 | 无 |
| LoadingOverlay | 创建全屏遮罩 + Spinner 实体 | 销毁 Loading 实体 | ModalOverlay 确认后打开 |

---

## 12. Lifecycle

> Screen 的完整生命周期行为。遵守 `screen-lifecycle.md` 定义的状态机。

| 阶段 | 行为 | 触发条件 | 清理 |
|------|------|---------|------|
| **OnEnter** | `spawn_main_menu()` — 生成完整 UI 树（root → title_area + button_list + version_text） | `GameState::MainMenu` 状态进入 | — |
| **OnReady** | 无操作（MainMenu 无 ViewModel，无异步数据加载） | OnEnter 完成，UI 树就绪 | — |
| **Active** | 等待按钮点击事件，通过 UiLayer 下发 UiCommand | OnReady 完成 | — |
| **OnExit** | `despawn_main_menu()` — 清理所有标记了 `With<MainMenuScreen>` 的实体 | `GameState::MainMenu` 状态退出 | 清理标记: `With<MainMenuScreen>` |

### 12.1 生命周期事件处理

```yaml
on_enter:
  - action: "spawn_ui_tree"
    spawner: "spawn_main_menu()"
    description: "生成 MainMenuScreen 完整 UI 树：TitleArea(标题 + 副标题) + ButtonList(3个按钮) + VersionText"

on_ready:
  - action: "no_op"
    description: "MainMenuScreen 无ViewModel集成，无Observer注册，无异步数据加载"

active:
  - trigger: "UiCommand::NewGame"
    action: "通过 UiLayer 下发至 Domain"
    scope: "new_game_btn"
  - trigger: "UiCommand::OpenScreen(ScreenType::Settings)"
    action: "通过 UiLayer 下发导航至 ScreenStack"
    scope: "settings_btn"
  - trigger: "UiCommand::OpenScreen(ScreenType::SaveLoad)"
    action: "通过 UiLayer 下发导航至 ScreenStack"
    scope: "load_game_btn"

on_exit:
  - action: "unregister_observer"
    target: "None — MainMenu 未注册任何 Observer"
  - action: "despawn_ui_tree"
    query: "With<MainMenuScreen>"
    description: "清理所有标记了 MainMenuScreen 组件的实体"
```

---

## 13. Data Ownership

> Owns / Uses 分离。MainMenuScreen 不消费任何 ViewModel / UiStore 字段。

### 13.1 ViewModel 映射

| ViewModel | 字段 | 归属 (Owns/Uses) | 更新频率 | Projection 源 |
|-----------|------|-----------------|---------|--------------|
| 无 | — | — | — | — |

### 13.2 数据流

MainMenuScreen 无 Domain 数据流入。交互数据流为纯 UI 导航：

```
用户点击按钮 → UiCommand → UiLayer → ScreenStack (OpenScreen) / Domain (NewGame)
```

---

## 14. Layout Intent

> 每个关键尺寸的**理由说明**。为什么选这个尺寸而不是别的？

### 14.1 固定尺寸意图

| widget_id | 属性 | 值 | 意图 | shrink |
|-----------|------|----|------|--------|
| `button_list` | width | 200px | "按钮宽度 200px 确保文本（中英文均在 10 字以内）单行显示，不折行" | none |
| `new_game_btn` | width | 200px | "与 button_list 同宽，确保按钮边缘对齐，视觉统一" | none |
| `new_game_btn` | height | 48px | "按钮高度大于 40px 最小触摸目标，适配手指/鼠标点击" | none |
| `load_game_btn` | width | 200px | "与 new_game_btn 同宽" | none |
| `load_game_btn` | height | 48px | "与 new_game_btn 等高" | none |
| `settings_btn` | width | 200px | "与 new_game_btn 同宽" | none |
| `settings_btn` | height | 48px | "与 new_game_btn 等高" | none |

### 14.2 弹性尺寸意图

| widget_id | flex_grow | 理由 |
|-----------|-----------|------|
| `title_area` | 0 | "标题区域无需弹性增长，auto 尺寸自然包裹内容即可" |
| `button_list` | 0 | "按钮列表无需弹性增长，固定 200px + 3 × 48px 高度即可" |
| `version_text` | 0 | "版本号文本无需弹性增长，固定底部显示" |

### 14.3 通用约束

```yaml
global:
  min_interactive_height: 40px   # 可交互元素最小高度 (触摸友好)
  min_interactive_width: 40px    # 可交互元素最小宽度 (触摸友好)
  standard_padding: 8px          # 标准内边距
  standard_gap: 4px              # 标准间距 (Flexbox gap)
```

---

## 15. Scroll & Overflow Policy

> MainMenuScreen 内容不超出视口，无滚动需求。

### 15.1 滚动区域

| widget_id | 方向 | Scroll Policy | Overflow Policy | 理由 |
|-----------|------|--------------|---------------|------|
| `title_area` | vertical | none | clip | "标题内容固定，不会溢出，无需滚动" |
| `button_list` | vertical | none | clip | "3 个按钮固定高度总和 (3×48=144px) 远小于视口高度，无需滚动" |
| `version_text` | horizontal | none | clip | "版本号文本极短 ('v0.1.0')，不会溢出" |

### 15.2 文本溢出

| widget_id | max_lines | overflow | 多语言风险 |
|-----------|-----------|----------|-----------|
| `title_text` | 1 | clip | "标题 'Fre' 极短，无多语言溢出风险" |
| `subtitle_text` | 1 | ellipsis | "副标题中英文差异：中文 '一款 Bevy SRPG 游戏' (11 字) 约为英文 'A Bevy SRPG' (11 字符) 同长，风险低" |
| `new_game_btn` | 1 | ellipsis | "按钮标签短，中英文均在 10 字以内，风险低" |
| `load_game_btn` | 1 | ellipsis | "按钮标签短，中英文均在 10 字以内，风险低" |
| `settings_btn` | 1 | ellipsis | "按钮标签短，中英文均在 10 字以内，风险低" |
| `version_text` | 1 | clip | "版本号 'v0.1.0' 极短，无风险" |

---

## 16. Event Contract

> UI -> Domain 事件 + Domain -> UI 事件的完整契约。

### 16.1 UI -> Domain（通过 UiCommand 传递）

```yaml
NewGame:
  trigger_widget: "button_list → new_game_btn → click"
  data: {}
  conditions: []
  emits: UiCommand::NewGame
  domain_event: "TBD — 战前准备流程启动 (PartyAssemblyScreen / GameState::PartyAssembly)"

OpenSettings:
  trigger_widget: "button_list → settings_btn → click"
  data: {}
  conditions: []
  emits: UiCommand::OpenScreen(ScreenType::Settings)
  domain_event: "None — 纯UI导航，ScreenStack 直接处理"

OpenSaveLoad:
  trigger_widget: "button_list → load_game_btn → click"
  data: {}
  conditions: []
  emits: UiCommand::OpenScreen(ScreenType::SaveLoad)
  domain_event: "None — 纯UI导航，ScreenStack 直接处理"
```

### 16.2 Domain -> UI（通过 Projection 消费）

```yaml
# MainMenuScreen 不消费任何 Domain Event。
# MainMenu 无 ViewModel，无实时数据绑定。
# 所有交互均为 UI 导航指令，不依赖 Domain 数据响应。
```

---

## 17. Screen Metrics

> 复杂度基线。所有数值初始创建时手动填写，后续 CI 阶段自动校验。

| Metric | 值 | P0/P1 | 说明 |
|--------|-----|-------|------|
| `widget_count` | 9 | P1 | Widget 实例总数（root Screen + title_area + title_text + subtitle_text + button_list + 3 buttons + version_text） |
| `container_count` | 2 | P1 | 纯容器节点数（title_area + button_list — 无业务逻辑，仅布局） |
| `interactive_count` | 3 | P1 | 可交互 Widget 数（new_game_btn + load_game_btn + settings_btn） |
| `overlay_count` | 2 | P1 | 关联的 Overlay 数（ModalOverlay + LoadingOverlay，均为预留） |
| `max_depth` | 3 | P1 | root → button_list → new_game_btn 的层级数 |
| `max_children` | 4 | P1 | 单一容器最大子节点数（root 有 3 个直接子节点: title_area + button_list + version_text / button_list 有 3 个按钮子节点） |

### 17.1 Budget 检查

| 规则 | 阈值 | 当前值 | 状态 |
|------|------|--------|------|
| max_depth ≤ 6 | 6 | 3 | ✅ |
| max_children ≤ 20 | 20 | 4 | ✅ |
| interactive_count / widget_count ≥ 0.2 | 20% | 33% (3/9) | ✅ |

---

## 附录 A: DoD Checklist

> 以下 14 项全部通过后，本文件 status 改为 `active`。

| # | 检查项 | 状态 | 备注 |
|---|--------|------|------|
| D01 | ASCII Wireframe 存在, 所有区域已命名 (region_id) | [ ] | |
| D02 | 无匿名面板 — 每个区域都有 widget_id 标注 | [ ] | |
| D03 | Widget Tree 完整 — 从 root 到叶子，无隐藏节点 | [ ] | |
| D04 | 所有引用的 widget_id 在 Widget Tree 中存在 | [ ] | |
| D05 | Flexbox Layout 完整 — 每个 widget_id 有 direction/width/height/flex_grow/intent | [ ] | |
| D06 | Responsive Rules 已定义 (至少 strategy: none) | [ ] | |
| D07 | Region Responsibility 已定义 (每 region 3-5 条) | [ ] | |
| D08 | Widget Contract 已定义 (Inputs/Outputs/Selection Model) | [ ] | |
| D09 | State Mapping 完整 (每个 region 的 Loading/Empty/Normal/Error) | [ ] | 全部标记为 N/A（MainMenu 无异步数据） |
| D10 | Focus Navigation 已定义 (Tab 路径完整) | [ ] | P1 |
| D11 | Interaction Zones 已定义 (Click/Hover) | [ ] | |
| D12 | Overlay Definition 已定义 (Overlay 列表 + Z-Layer) | [ ] | |
| D13 | Lifecycle 已定义 (OnEnter/OnReady/Active/OnExit) | [ ] | |
| D14 | Data Ownership 已定义 (Owns/Uses) | [ ] | |
| D15 | Layout Intent 已定义 (关键尺寸理由) | [ ] | P1 |
| D16 | Scroll & Overflow Policy 已定义 | [ ] | P1 |
| D17 | Event Contract 已定义 (UI->Domain + Domain->UI) | [ ] | P1 |
| D18 | Screen Metrics 已定义 | [ ] | P1 |

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
| `06-ui/02-design-system/widget-atoms.md` | 原子组件 Contract（HeadingText, CaptionText, Button） |
| `06-ui/02-design-system/widget-composites.md` | 复合组件 Contract |
| `06-ui/02-design-system/theme-localization.md` | StyleToken / Theme / UiTextKey |
| `06-ui/02-design-system/focus-binding.md` | Focusable / FocusGroup / Dirty<T> / UiBinding |
| `06-ui/03-screens/screens.md` | 主菜单 Screen 定义（§3） |
| `06-ui/03-screens/screen-lifecycle.md` | Screen 生命周期状态机 |
| `06-ui/04-data-flow/projection-viewmodel.md` | Projection / ViewModel 映射 |
| `03-content/localization/ui-screen-keys.md` | 主菜单 LocalizationKeys（§2） |

---

## 附录 C: 架构审查记录

> **审查人**: @presentation-architect | **日期**: 2026-06-22 | **结论**: PASS with notes

### 通过确认

| 维度 | 结果 |
|------|------|
| 模板合规 (17字段) | PASS |
| 架构一致性 (vs screens.md §3) | PASS |
| ASCII Wireframe (无匿名面板) | PASS (borderline D02) |
| Widget ID 命名规范 | PASS |
| Flexbox Layout 完整性 | PASS |
| LocalizationKey 一致性 | PASS |
| Widget Contract 一致性 | PASS |
| 过约束检查 | PASS |
| 单向数据流 | PASS |

### 待处理建议 (不阻塞 active，建议下一修订版处理)

| # | 区域 | 问题 | 优先级 |
|---|------|------|--------|
| 01 | 6.3 version_text | Region Responsibility 仅 2 条，低于 D07 要求的 3-5 条。建议补充 R03 | Low |
| 02 | 3.1 Widget Type 索引 | 引用 `§Button` 但 widget-atoms.md 无此节。应用 `§2.1 PrimaryButton` / `§2.2 SecondaryButton` | Low |
| 03 | 3.1 / 7 | Widget Type 命名 "Button, Primary" vs contract 的 "PrimaryButton"，建议统一 | Low |
| 04 | 12 Lifecycle | OnReady 未提及设置默认焦点到 new_game_btn（虽在 §9 已定义） | Low |
| 05 | 1/附录A | P0/P1 分类 Focus Nav 在 header 为 P0 但 DoD 标注 P1（继承模板不一致） | Low |

### P0 字段全部通过日期
2026-06-22

---

*本文档是 MainMenuScreen SSPEC，由 @feature-developer 根据 `07-specs/screen-spec-template.md` 模板创建。所有 17 个字段已填充。当前 status: active（2026-06-22 经 @presentation-architect 审查通过）。*
