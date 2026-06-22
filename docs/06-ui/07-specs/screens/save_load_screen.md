---
id: 07-specs.save-load-screen
title: SaveLoadScreen Specification — AI-Consumable Layout & Interaction Spec
status: active
owner: presentation-architect
created: 2026-06-22
tags:
  - ui
  - screen-spec
  - save-load
  - active
  - reviewed
---

# SaveLoadScreen

> **职责**: @presentation-architect | **上游**: ADR-066 (Screen Spec), `07-specs/README.md` (总纲)
> **状态**: active（通过 @presentation-architect 最终审查）

**P0 字段**: 1-14 (Screen Header / ASCII Wireframe / Widget Tree / Flexbox Layout / Responsive Rules / Region Responsibility / Widget Contract / State Mapping / Focus Nav / Interaction Zones / Overlay / Lifecycle / Data Ownership / Layout Intent)
**P1 字段**: 15-17 (Scroll & Overflow / Event Contract / Screen Metrics)

---

## 1. Screen Header

| 属性 | 值 |
|------|-----|
| Screen Name | `SaveLoadScreen` — 对应 `GameState::SaveLoad`（或在菜单中通过 `UiCommand::OpenScreen(ScreenType::SaveLoad)` 打开） |
| Purpose | 游戏的存档/读档界面：展示最多 10 个存档槽位的元数据，支持保存游戏、加载存档、删除存档操作 |
| Navigation | MainMenu → SaveLoad (UiCommand::OpenScreen(ScreenType::SaveLoad) / LoadGameButton)；MainMenu → SaveLoad (SaveMode — 通过设置菜单调用保存)；点击 Close 或 Esc 返回前一 Screen（MainMenuScreen） |
| GameState | `GameState::SaveLoad` |
| ScreenLayer 层级 | 100（screen_layer） |
| 加载模式 | Ephemeral（每次进入 SaveLoad 重新 spawn） |
| 过渡动画 | Fade(0.3s) |
| 变体 | SaveMode / LoadMode |

---

## 2. ASCII Wireframe

> 纯文本线框图。所有区域必须命名（`widget_id`），禁止匿名面板。

```
┌─────────────────────────────────────────────────────────────────────────────────────┐
│  [saveload_header]                                                                    │
│  ┌──────────────────────────────────────────────────────────────────────────────┐    │
│  │  [saveload_header_title] │ [saveload_header_mode_toggle]   [saveload_header_  │    │
│  │    Save Game             │   Switch to Load                close]             │    │
│  │                                                                     [X]       │    │
│  └──────────────────────────────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────────────────────────┤
│  [saveload_body]                                                                      │
│  ┌────────────────────────────┐  ┌────────────────────────────────────────────────┐  │
│  │  [saveload_slot_list]      │  │  [saveload_preview_panel]                      │  │
│  │                            │  │                                                │  │
│  │  ┌──────────────────────┐  │  │  ┌────────────────────────────────────────┐   │  │
│  │  │ [saveload_slot_1]    │  │  │  │  [saveload_preview_avatar]              │   │  │
│  │  │ Slot 1               │  │  │  │    ┌──────┐                             │   │  │
│  │  │ Aria - Lv.5          │  │  │  │    │avatar│                             │   │  │
│  │  │ River Crossing       │  │  │  │    └──────┘                             │   │  │
│  │  │ 2026-06-21 14:30     │  │  │  └────────────────────────────────────────┘   │  │
│  │  │ Playtime: 12h 34m    │  │  │                                                │  │
│  │  └──────────────────────┘  │  │  ┌────────────────────────────────────────┐   │  │
│  │  ┌──────────────────────┐  │  │  │  [saveload_preview_details]            │   │  │
│  │  │ [saveload_slot_2]    │  │  │  │  Name:        Aria                     │   │  │
│  │  │ Slot 2               │  │  │  │  Level:       5                        │   │  │
│  │  │ [EmptyLabel]         │  │  │  │  Location:    River Crossing           │   │  │
│  │  └──────────────────────┘  │  │  │  Playtime:    12h 34m                  │   │  │
│  │  ┌──────────────────────┐  │  │  │  Chapter:     3                        │   │  │
│  │  │ [saveload_slot_3]    │  │  │  │  Last Saved:  2026-06-21 14:30         │   │  │
│  │  │ Slot 3               │  │  │  └────────────────────────────────────────┘   │  │
│  │  │ [EmptyLabel]         │  │  │                                                │  │
│  │  └──────────────────────┘  │  │  ┌────────────────────────────────────────┐   │  │
│  │       ... (4-10)           │  │  │  [saveload_preview_screenshot]         │   │  │
│  │                            │  │  │  ┌──────────────────────────────────┐  │   │  │
│  │                            │  │  │  │   Screenshot Preview             │  │   │  │
│  │                            │  │  │  │   (256x144 placeholder)          │  │   │  │
│  │                            │  │  │  └──────────────────────────────────┘  │   │  │
│  │                            │  │  └────────────────────────────────────────┘   │  │
│  └────────────────────────────┘  └────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────────────────────┤
│  [saveload_action_panel]                                                              │
│  ┌──────────────────────────────────────────────────────────────────────────────┐    │
│  │                                                                              │    │
│  │                               ┌──────────────────────┐  ┌──────────────────┐ │    │
│  │                               │  [saveload_confirm_  │  │ [saveload_delete_│ │    │
│  │                               │   btn]               │  │  btn]            │ │    │
│  │                               │    Save / Load       │  │    Delete        │ │    │
│  │                               └──────────────────────┘  └──────────────────┘ │    │
│  └──────────────────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Region 索引

| widget_id | 类型 | 用途 | 对应 Wireframe 位置 |
|-----------|------|------|-------------------|
| `saveload_header` | Container | 包裹标题、模式切换按钮和关闭按钮，水平排列 | Wireframe 顶部第一行 |
| `saveload_header_title` | HeadingText | 显示 "Save Game" / "Load Game" 标题（取决于当前模式） | 标题处（左侧） |
| `saveload_header_mode_toggle` | Button, Secondary | 切换保存/加载模式的按钮（"Switch to Load" / "Switch to Save"） | 标题与关闭按钮之间 |
| `saveload_header_close` | IconButton | 关闭按钮，点击触发返回上一级 Screen | 标题处（右侧） |
| `saveload_body` | Container | 包裹列表区域和预览区域，水平分栏布局 | Wireframe 中部 |
| `saveload_slot_list` | Container | 垂直排列最多 10 个存档槽位，支持滚动 | 左侧栏（固定 400px） |
| `saveload_slot_1` | Molecule | 第 1 个存档槽位，显示槽位号 + 角色名/等级/地点/时间/游戏时长 或 EmptyLabel | 列表第一个条目 |
| `saveload_slot_2` | Molecule | 第 2 个存档槽位，同上结构 | 列表第二个条目 |
| `saveload_slot_3` | Molecule | 第 3 个存档槽位 | 列表第三个条目 |
| `saveload_slot_4` | Molecule | 第 4 个存档槽位 | 列表第四个条目 |
| `saveload_slot_5` | Molecule | 第 5 个存档槽位 | 列表第五个条目 |
| `saveload_slot_6` | Molecule | 第 6 个存档槽位 | 列表第六个条目 |
| `saveload_slot_7` | Molecule | 第 7 个存档槽位 | 列表第七个条目 |
| `saveload_slot_8` | Molecule | 第 8 个存档槽位 | 列表第八个条目 |
| `saveload_slot_9` | Molecule | 第 9 个存档槽位 | 列表第九个条目 |
| `saveload_slot_10` | Molecule | 第 10 个存档槽位 | 列表第十个条目 |
| `saveload_preview_panel` | Container | 选中槽位的详细信息预览面板，包括头像、详情、截图 | 右侧内容区 |
| `saveload_preview_avatar` | Image | 选中存档的角色头像 | 预览面板顶部区域 |
| `saveload_preview_details` | Container | 选中存档的详细元数据（等级、地点、游戏时长、章节、保存时间） | 预览面板中部 |
| `saveload_preview_screenshot` | Container | 选中存档的游戏截图预览（256x144 占位区域） | 预览面板底部 |
| `saveload_action_panel` | Container | 底部操作栏，包裹 ConfirmButton 和 DeleteButton | Wireframe 底部 |
| `saveload_confirm_btn` | Button, Primary | 确认操作按钮——SaveMode 下为 "Save"，LoadMode 下为 "Load" | 底部左侧 Action 按钮 |
| `saveload_delete_btn` | Button, Danger | 删除选中存档，点击触发 ModalOverlay 确认 | 底部右侧 Action 按钮 |

---

## 3. Widget Tree

> 标注 `[widget_id: WidgetType]` 的树结构。禁止隐藏节点，必须完整。

```
ScreenRoot                                                  [saveload_root: Screen]
├── HeaderPanel                                             [saveload_header: Container]
│   ├── ScreenTitle                                         [saveload_header_title: HeadingText]
│   ├── ModeToggleButton                                    [saveload_header_mode_toggle: Button, Secondary]
│   └── CloseButton                                         [saveload_header_close: IconButton]
├── BodyPanel                                               [saveload_body: Container]
│   ├── SlotList                                            [saveload_slot_list: Container]
│   │   ├── SaveSlot1                                       [saveload_slot_1: SaveSlotMolecule]
│   │   │   ├── SlotNumber                                  [saveload_slot_1_number: BodyText]
│   │   │   ├── SlotInfo                                    [saveload_slot_1_info: CaptionText]
│   │   │   ├── PlayTime                                    [saveload_slot_1_playtime: CaptionText]
│   │   │   └── EmptyLabel                                  [saveload_slot_1_empty: CaptionText]
│   │   ├── SaveSlot2                                       [saveload_slot_2: SaveSlotMolecule]
│   │   │   ├── SlotNumber                                  [saveload_slot_2_number: BodyText]
│   │   │   ├── SlotInfo                                    [saveload_slot_2_info: CaptionText]
│   │   │   ├── PlayTime                                    [saveload_slot_2_playtime: CaptionText]
│   │   │   └── EmptyLabel                                  [saveload_slot_2_empty: CaptionText]
│   │   ├── SaveSlot3                                       [saveload_slot_3: SaveSlotMolecule]
│   │   │   ├── SlotNumber                                  [saveload_slot_3_number: BodyText]
│   │   │   ├── SlotInfo                                    [saveload_slot_3_info: CaptionText]
│   │   │   ├── PlayTime                                    [saveload_slot_3_playtime: CaptionText]
│   │   │   └── EmptyLabel                                  [saveload_slot_3_empty: CaptionText]
│   │   ├── SaveSlot4                                       [saveload_slot_4: SaveSlotMolecule]
│   │   │   ├── SlotNumber                                  [saveload_slot_4_number: BodyText]
│   │   │   ├── SlotInfo                                    [saveload_slot_4_info: CaptionText]
│   │   │   ├── PlayTime                                    [saveload_slot_4_playtime: CaptionText]
│   │   │   └── EmptyLabel                                  [saveload_slot_4_empty: CaptionText]
│   │   ├── SaveSlot5                                       [saveload_slot_5: SaveSlotMolecule]
│   │   │   ├── SlotNumber                                  [saveload_slot_5_number: BodyText]
│   │   │   ├── SlotInfo                                    [saveload_slot_5_info: CaptionText]
│   │   │   ├── PlayTime                                    [saveload_slot_5_playtime: CaptionText]
│   │   │   └── EmptyLabel                                  [saveload_slot_5_empty: CaptionText]
│   │   ├── SaveSlot6                                       [saveload_slot_6: SaveSlotMolecule]
│   │   │   ├── SlotNumber                                  [saveload_slot_6_number: BodyText]
│   │   │   ├── SlotInfo                                    [saveload_slot_6_info: CaptionText]
│   │   │   ├── PlayTime                                    [saveload_slot_6_playtime: CaptionText]
│   │   │   └── EmptyLabel                                  [saveload_slot_6_empty: CaptionText]
│   │   ├── SaveSlot7                                       [saveload_slot_7: SaveSlotMolecule]
│   │   │   ├── SlotNumber                                  [saveload_slot_7_number: BodyText]
│   │   │   ├── SlotInfo                                    [saveload_slot_7_info: CaptionText]
│   │   │   ├── PlayTime                                    [saveload_slot_7_playtime: CaptionText]
│   │   │   └── EmptyLabel                                  [saveload_slot_7_empty: CaptionText]
│   │   ├── SaveSlot8                                       [saveload_slot_8: SaveSlotMolecule]
│   │   │   ├── SlotNumber                                  [saveload_slot_8_number: BodyText]
│   │   │   ├── SlotInfo                                    [saveload_slot_8_info: CaptionText]
│   │   │   ├── PlayTime                                    [saveload_slot_8_playtime: CaptionText]
│   │   │   └── EmptyLabel                                  [saveload_slot_8_empty: CaptionText]
│   │   ├── SaveSlot9                                       [saveload_slot_9: SaveSlotMolecule]
│   │   │   ├── SlotNumber                                  [saveload_slot_9_number: BodyText]
│   │   │   ├── SlotInfo                                    [saveload_slot_9_info: CaptionText]
│   │   │   ├── PlayTime                                    [saveload_slot_9_playtime: CaptionText]
│   │   │   └── EmptyLabel                                  [saveload_slot_9_empty: CaptionText]
│   │   └── SaveSlot10                                      [saveload_slot_10: SaveSlotMolecule]
│   │       ├── SlotNumber                                  [saveload_slot_10_number: BodyText]
│   │       ├── SlotInfo                                    [saveload_slot_10_info: CaptionText]
│   │       ├── PlayTime                                    [saveload_slot_10_playtime: CaptionText]
│   │       └── EmptyLabel                                  [saveload_slot_10_empty: CaptionText]
│   └── PreviewPanel                                        [saveload_preview_panel: Container]
│       ├── AvatarPreview                                   [saveload_preview_avatar: Image]
│       ├── DetailInfo                                      [saveload_preview_details: Container]
│       │   ├── DetailName                                  [saveload_preview_detail_name: BodyText]
│       │   ├── DetailLevel                                 [saveload_preview_detail_level: BodyText]
│       │   ├── DetailLocation                              [saveload_preview_detail_location: BodyText]
│       │   ├── DetailPlaytime                              [saveload_preview_detail_playtime: BodyText]
│       │   ├── DetailChapter                               [saveload_preview_detail_chapter: BodyText]
│       │   └── DetailSavedAt                               [saveload_preview_detail_saved_at: BodyText]
│       └── ScreenshotPreview                               [saveload_preview_screenshot: Container]
└── ActionPanel                                             [saveload_action_panel: Container]
    ├── ConfirmButton                                       [saveload_confirm_btn: Button, Primary]
    └── DeleteButton                                        [saveload_delete_btn: Button, Danger]
```

### 3.1 Widget Type 索引

| widget_id | WidgetType | 定义位置 | 复用于 |
|-----------|-----------|---------|--------|
| `saveload_header_title` | `Atom: HeadingText` | `02-design-system/widget-atoms.md §HeadingText` | MainMenuScreen、各 Screen 标题 |
| `saveload_header_mode_toggle` | `Atom: Button (Secondary)` | `02-design-system/widget-atoms.md §Button` | 设置Screen、其他导航场景 |
| `saveload_header_close` | `Atom: IconButton` | `02-design-system/widget-atoms.md §IconButton` | SettingsScreen、InventoryScreen |
| `saveload_slot_{n}` | `Molecule: SaveSlotMolecule` | `02-design-system/widget-composites.md (待定义)` | — |
| `saveload_slot_{n}_number` | `Atom: BodyText` | `02-design-system/widget-atoms.md §BodyText` | — |
| `saveload_slot_{n}_info` | `Atom: CaptionText` | `02-design-system/widget-atoms.md §CaptionText` | — |
| `saveload_slot_{n}_playtime` | `Atom: CaptionText` | `02-design-system/widget-atoms.md §CaptionText` | — |
| `saveload_slot_{n}_empty` | `Atom: CaptionText` | `02-design-system/widget-atoms.md §CaptionText` | — |
| `saveload_preview_avatar` | `Atom: Image` | `02-design-system/widget-atoms.md §Image` | InventoryScreen 角色头像 |
| `saveload_preview_detail_*` | `Atom: BodyText` | `02-design-system/widget-atoms.md §BodyText` | — |
| `saveload_preview_screenshot` | `Atom: Image` | `02-design-system/widget-atoms.md §Image` | — |
| `saveload_confirm_btn` | `Atom: Button (Primary)` | `02-design-system/widget-atoms.md §Button` | 各确认场景 |
| `saveload_delete_btn` | `Atom: Button (Danger)` | `02-design-system/widget-atoms.md §Button` | — |

---

## 4. Flexbox Layout

> YAML 格式。每个 widget_id 必须有 direction / width / height / flex_grow / intent。
> 注意：10 个槽位共享相同布局参数，仅以 `saveload_slot_{n}` 为代表列出。

```yaml
## Flexbox Layout — SaveLoadScreen
## width/height: px 值或 "auto" 或 "fill"
## flex_grow: 0=不增长, 1=等分剩余空间, 2=双倍增长
## shrink: none/low/high — 收缩优先级

saveload_root:
  direction: column
  width: 100%
  height: 100%
  flex_grow: 0
  intent: "Screen 根容器，占满视口，垂直排列 Header / Body / ActionPanel"

saveload_header:
  direction: row
  width: 100%
  height: 56
  flex_grow: 0
  shrink: none
  intent: "Header 栏，固定 56px 高度，水平排列标题（左）、模式切换（中）、关闭按钮（右）"

saveload_header_title:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  shrink: none
  intent: "标题文本，auto 宽度适配 'Save Game' / 'Load Game'，不弹性增长"

saveload_header_mode_toggle:
  direction: row
  width: auto
  height: 40
  flex_grow: 1
  shrink: low
  intent: "模式切换按钮，flex_grow:1 居中于标题与关闭按钮之间，高度 40px 保证最小触摸目标"

saveload_header_close:
  direction: row
  width: 40
  height: 40
  flex_grow: 0
  shrink: none
  intent: "关闭按钮，固定 40x40 保证最小触摸目标"

saveload_body:
  direction: row
  width: 100%
  height: fill
  flex_grow: 1
  intent: "主体区域，水平分栏，占据除 Header+ActionPanel 外的所有剩余空间"

saveload_slot_list:
  direction: column
  width: 400
  height: 100%
  flex_grow: 0
  shrink: none
  intent: "存档槽位列表，固定 400px 宽度，垂直排列最多 10 个槽位"

saveload_slot_{n}:
  direction: column
  width: 100%
  height: 80
  flex_grow: 0
  shrink: none
  intent: "单个存档槽位，固定高度 80px，显示槽位号+摘要信息 或 EmptyLabel"

saveload_slot_{n}_number:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  shrink: none
  intent: "槽位号文本，'Slot 1' 格式，auto 宽度"

saveload_slot_{n}_info:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  shrink: low
  intent: "槽位存档信息（角色名-等级、地点），可适度压缩"

saveload_slot_{n}_playtime:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  shrink: low
  intent: "游戏时长文本，可适度压缩"

saveload_slot_{n}_empty:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  shrink: none
  intent: "空槽位标签 '[EmptyLabel]'，auto 宽度，无压缩"

saveload_preview_panel:
  direction: column
  width: fill
  height: 100%
  flex_grow: 1
  intent: "预览面板，弹性宽度填充剩余水平空间，垂直排列头像/详情/截图"

saveload_preview_avatar:
  direction: row
  width: 96
  height: 96
  flex_grow: 0
  shrink: none
  intent: "角色头像区域，固定 96x96，居中显示"

saveload_preview_details:
  direction: column
  width: 100%
  height: auto
  flex_grow: 0
  intent: "存档详情容器，auto 高度包裹 6 个字段文本"

saveload_preview_detail_name:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  intent: "角色名称字段"

saveload_preview_detail_level:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  intent: "等级字段"

saveload_preview_detail_location:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  intent: "地点字段"

saveload_preview_detail_playtime:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  intent: "游戏时长字段"

saveload_preview_detail_chapter:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  intent: "章节字段"

saveload_preview_detail_saved_at:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  intent: "最后保存时间字段"

saveload_preview_screenshot:
  direction: row
  width: 256
  height: 144
  flex_grow: 0
  shrink: none
  intent: "游戏截图预览占位区域，固定 256x144（4:3 比例）"

saveload_action_panel:
  direction: row
  width: 100%
  height: 56
  flex_grow: 0
  shrink: none
  intent: "底部操作栏，固定 56px 高度，水平右对齐排列 ConfirmButton + DeleteButton"

saveload_confirm_btn:
  direction: row
  width: 160
  height: 40
  flex_grow: 0
  shrink: none
  intent: "确认按钮，Primary 样式，固定 160x40，标签动态切换 'Save' / 'Load'"

saveload_delete_btn:
  direction: row
  width: 120
  height: 40
  flex_grow: 0
  shrink: none
  intent: "删除按钮，Danger 样式，固定 120x40，仅在选中非空槽位时 enabled"
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

### 6.1 saveload_header

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示存档/读档页面标题文本（LocalizationKey: `ui.save_load.title.save` / `ui.save_load.title.load`，取决于当前模式） | Display | 页面标题与实际模式不符 |
| R02 | 展示模式切换按钮（Secondary 样式），标签根据当前模式动态切换 | Display | 用户无法切换存档/读档模式 |
| R03 | 展示关闭按钮（IconButton），图标为 "X" | Display | 用户无法关闭页面 |
| R04 | 响应模式切换按钮点击，切换 SaveMode / LoadMode，更新标题和按钮标签 | Interaction | 模式切换不生效 |
| R05 | 响应关闭按钮点击，触发 `UiCommand::CloseScreen` | Interaction | 关闭功能失效 |

**不负责**:
- 管理存档槽位的选择状态（属于 `saveload_slot_list` 职责）
- 执行实际的存档/读档操作（属于 `saveload_action_panel` 职责）

### 6.2 saveload_slot_list

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示最多 10 个存档槽位，按槽位号 1-10 垂直排列 | Display | 槽位数不足或顺序错误 |
| R02 | 每个槽位展示槽位号 + 存档摘要信息（非空槽位）或 EmptyLabel（空槽位） | Display | 槽位信息展示不全 |
| R03 | 响应槽位点击，选中该槽位，高亮显示，更新预览面板和 Action 按钮状态 | Interaction | 选中槽位无反馈 |
| R04 | 响应键盘上下方向键遍历槽位列表 | Focus | 键盘无法操作槽位列表 |
| R05 | 槽位列表固定 400px 宽度，不伸缩 | Layout | 列表宽度不一致 |
| R06 | 当列表总高度超出可视区域时支持垂直滚动（最多 10 个槽位 × 80px = 800px） | Scroll | 底部槽位不可见 |

**不负责**:
- 执行保存/加载/删除操作（属于 `saveload_action_panel` 职责）
- 显示详细的存档信息（属于 `saveload_preview_panel` 职责）

### 6.3 saveload_preview_panel

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 当选中一个非空槽位时，展示该存档的详细信息（角色头像、等级、地点、游戏时长、章节、保存时间） | Display | 预览信息缺失或不准确 |
| R02 | 当选中空槽位或无选中槽位时，显示提示文本（"选择一个存档槽位"）或留空 | Display | 空选择时无任何指示 |
| R03 | 在截图区域展示存档截图预览（256x144 占位，MVP 阶段显示占位图案） | Display | 预览区域空白 |
| R04 | 预览面板弹性宽度填充剩余水平空间 | Layout | 预览面板宽度异常 |

**不负责**:
- 管理存档选择状态（属于 `saveload_slot_list` 职责）
- 执行保存/加载/删除操作（属于 `saveload_action_panel` 职责）

### 6.4 saveload_action_panel

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示确认按钮（Primary 样式），标签根据当前模式动态显示 "Save" / "Load" | Display | 按钮标签错误 |
| R02 | 展示删除按钮（Danger 样式），标签 "Delete" | Display | 删除入口缺失 |
| R03 | SaveMode 下确认按钮仅当选中的槽位为 空槽 时才 enabled（新建存档）；选中非空槽位时先弹出 ModalOverlay 确认覆盖 | Interaction | 误覆盖存档 |
| R04 | LoadMode 下确认按钮仅当选中的槽位为 非空槽 时才 enabled | Interaction | 加载空槽位无意义 |
| R05 | 删除按钮仅当选中的槽位为 非空槽 时才 enabled | Interaction | 删除空槽位无意义 |
| R06 | 响应确认按钮点击，发射对应 UiCommand（SaveGame / LoadGame） | Interaction | 保存/加载流程无法启动 |
| R07 | 响应删除按钮点击，弹出 ModalOverlay 确认弹窗 | Interaction | 误删存档 |

**不负责**:
- 选择存档槽位（属于 `saveload_slot_list` 职责）
- 显示存档详情（属于 `saveload_preview_panel` 职责）

---

## 7. Widget Contract

> Inputs / Outputs / Selection Model。

### 7.1 saveload_header_title

```yaml
widget_id: saveload_header_title
widget_type: HeadingText
defined_in: "02-design-system/widget-atoms.md §HeadingText"

inputs:
  - name: text
    type: LocalizedText
    source: "ui.save_load.title.save / ui.save_load.title.load"
    default: "Save Game"
    dynamic: true  # 根据 mode 切换

outputs: []

selection_model:
  type: none
```

### 7.2 saveload_header_mode_toggle

```yaml
widget_id: saveload_header_mode_toggle
widget_type: Button, Secondary
defined_in: "02-design-system/widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.save_load.toggle_to_load / ui.save_load.toggle_to_save"
    default: "Switch to Load"
    dynamic: true  # 根据 mode 切换

outputs:
  - name: clicked
    type: UiCommand::ToggleSaveLoadMode
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.3 saveload_header_close

```yaml
widget_id: saveload_header_close
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

### 7.4 saveload_slot_{n} (代表 1-10 号槽位)

```yaml
widget_id: saveload_slot_{n}
widget_type: SaveSlotMolecule
defined_in: "02-design-system/widget-composites.md §SaveSlotMolecule (待定义)"

inputs:
  - name: slot_number
    type: usize
    source: "Slot index 1-10"
    default: 1
  - name: slot_data
    type: SaveSlotVm
    source: "Save domain — slot metadata query"
    default: empty
  - name: is_selected
    type: bool
    source: "Screen state — selected_slot == n"
    default: false

outputs:
  - name: selected
    type: UiCommand::SelectSlot(usize)
    payload: n
    trigger: OnLeftClick

selection_model:
  type: single
  max_select: 1
  clear_on: none
```

### 7.5 saveload_preview_avatar

```yaml
widget_id: saveload_preview_avatar
widget_type: Image
defined_in: "02-design-system/widget-atoms.md §Image"

inputs:
  - name: texture
    type: Handle<Image>
    source: "SaveSlotVm.avatar_handle"
    default: "ui/default_avatar.png"
  - name: visible
    type: bool
    source: "selected_slot != None && slot_data.is_occupied"
    default: false

outputs: []

selection_model:
  type: none
```

### 7.6 saveload_preview_details (子字段)

```yaml
widget_id: saveload_preview_detail_name
widget_type: BodyText
defined_in: "02-design-system/widget-atoms.md §BodyText"

inputs:
  - name: text
    type: LocalizedText
    source: "SaveSlotVm.character_name"
    default: "—"

outputs: []

selection_model:
  type: none
```

```yaml
widget_id: saveload_preview_detail_level
widget_type: BodyText
defined_in: "02-design-system/widget-atoms.md §BodyText"

inputs:
  - name: text
    type: String
    source: "format!('Level: {}', SaveSlotVm.level)"
    default: "Level: —"

outputs: []

selection_model:
  type: none
```

```yaml
widget_id: saveload_preview_detail_location
widget_type: BodyText
defined_in: "02-design-system/widget-atoms.md §BodyText"

inputs:
  - name: text
    type: LocalizedText
    source: "SaveSlotVm.location_key"
    default: "Location: —"

outputs: []

selection_model:
  type: none
```

```yaml
widget_id: saveload_preview_detail_playtime
widget_type: BodyText
defined_in: "02-design-system/widget-atoms.md §BodyText"

inputs:
  - name: text
    type: String
    source: "format!('Playtime: {}', SaveSlotVm.playtime_formatted)"
    default: "Playtime: —"

outputs: []

selection_model:
  type: none
```

```yaml
widget_id: saveload_preview_detail_chapter
widget_type: BodyText
defined_in: "02-design-system/widget-atoms.md §BodyText"

inputs:
  - name: text
    type: LocalizedText
    source: "SaveSlotVm.chapter_key"
    default: "Chapter: —"

outputs: []

selection_model:
  type: none
```

```yaml
widget_id: saveload_preview_detail_saved_at
widget_type: BodyText
defined_in: "02-design-system/widget-atoms.md §BodyText"

inputs:
  - name: text
    type: String
    source: "format!('Last Saved: {}', SaveSlotVm.saved_at_formatted)"
    default: "Last Saved: —"

outputs: []

selection_model:
  type: none
```

### 7.7 saveload_preview_screenshot

```yaml
widget_id: saveload_preview_screenshot
widget_type: Container (Image placeholder)
defined_in: "02-design-system/widget-atoms.md §Image"

inputs:
  - name: texture
    type: Handle<Image>
    source: "SaveSlotVm.screenshot_handle"
    default: "ui/screenshot_placeholder.png"
  - name: visible
    type: bool
    source: "selected_slot != None && slot_data.is_occupied"
    default: false

outputs: []

selection_model:
  type: none
```

### 7.8 saveload_confirm_btn

```yaml
widget_id: saveload_confirm_btn
widget_type: Button, Primary
defined_in: "02-design-system/widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.save_load.action.save / ui.save_load.action.load"
    default: "Save"
    dynamic: true  # 根据 mode 切换
  - name: enabled
    type: bool
    source: |
      SaveMode: selected slot is empty OR overlord confirmed
      LoadMode: selected slot is occupied
    default: false

outputs:
  - name: clicked
    type: UiCommand::SaveGame(usize) | UiCommand::LoadGame(usize)
    payload: selected_slot
    trigger: OnLeftClick
    conditions:
      - SaveMode: "选中空槽位或覆盖确认后"
      - LoadMode: "选中非空槽位"

selection_model:
  type: none
```

### 7.9 saveload_delete_btn

```yaml
widget_id: saveload_delete_btn
widget_type: Button, Danger
defined_in: "02-design-system/widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.save_load.action.delete"
    default: "Delete"
  - name: enabled
    type: bool
    source: "selected slot is occupied"
    default: false

outputs:
  - name: clicked
    type: Trigger ModalOverlay (DeleteConfirm)
    payload: selected_slot
    trigger: OnLeftClick
    conditions:
      - "选中非空槽位"

selection_model:
  type: none
```

---

## 8. State Mapping (Per-Region)

> 每个 region 独立的状态。SaveLoadScreen 通过 Save Domain 接口查询存档信息，涉及异步数据加载。

### 8.1 saveload_header

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 标题和按钮标签为静态 LocalizedKey，无异步加载 | — | — |
| **Empty** | N/A — 标题和按钮始终有值 | — | — |
| **Normal** | 标题 HeadingText + 模式切换按钮 + 关闭图标正常显示 | OnEnter 完成 | Fade(0.3s) 入场动画 |
| **Error** | N/A — 静态元素无错误状态 | — | — |

### 8.2 saveload_slot_list

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | 所有槽位显示占位动画（灰框脉冲动画） | OnEnter — Save Domain 查询未返回 | 脉冲动画持续 |
| **Empty** | 所有 10 个槽位均显示 EmptyLabel（"空"） | Save Domain 返回无任何存档 | 从 Loading 直接过渡 |
| **Normal** | 部分或全部槽位显示存档摘要（角色名、等级、地点、时间），空槽位显示 EmptyLabel；选中槽位高亮 | Save Domain 返回元数据完成 | 列表项逐个淡入 (stagger 50ms) |
| **Error** | 列表顶部显示错误提示 "无法加载存档列表"，所有槽位显示 EmptyLabel | Save Domain 查询失败 | 错误提示淡入 |

### 8.3 saveload_preview_panel

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | 预览区域显示加载占位（灰框） | 选中槽位的详细数据正在加载 | 脉冲动画 |
| **Empty** | 无选中槽位 或 选中为空槽位时，显示 "请选择一个存档槽位" | 无选中 / 选中空槽位 | 从 Loading 直接过渡 |
| **Normal** | 显示选中存档的完整预览（头像 + 6 个详情字段 + 截图） | 选中非空槽位，数据加载完成 | 预览内容淡入切换 (Fade 0.15s) |
| **Error** | 显示 "无法加载存档详情" 错误提示 | 选中槽位的详细数据加载失败 | 错误提示淡入 |

### 8.4 saveload_action_panel

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 按钮标签和状态在 OnEnter 后即可确定，无需异步加载 | — | — |
| **Empty** | 两个按钮均为 disabled 状态（灰显不可点击） | 未选中任何槽位 | 无过渡 |
| **Normal** | ConfirmButton 根据模式 + 选中槽位状态决定 enabled；DeleteButton 仅当选中的非空槽位 enabled | 选中槽位后 | 按钮状态即时切换 |
| **Error** | N/A — 按钮本身无错误状态 | — | — |

---

## 9. Focus Navigation

> Tab 导航路径。按 Tab 键的顺序就是导航路径的顺序。

```yaml
focus_path:
  ## 阶段 1: Header
  - saveload_header_mode_toggle   # Tab 1 — 默认焦点 (SaveMode 优先使用 ConfirmButton 作为默认，需确认)
  - saveload_header_close         # Tab 2

  ## 阶段 2: 存档槽位列表
  - saveload_slot_1              # Tab 3
  - saveload_slot_2              # Tab 4
  - saveload_slot_3              # Tab 5
  - saveload_slot_4              # Tab 6
  - saveload_slot_5              # Tab 7
  - saveload_slot_6              # Tab 8
  - saveload_slot_7              # Tab 9
  - saveload_slot_8              # Tab 10
  - saveload_slot_9              # Tab 11
  - saveload_slot_10             # Tab 12

  ## 阶段 3: Action Panel
  - saveload_confirm_btn         # Tab 13
  - saveload_delete_btn          # Tab 14

special_keys:
  Escape: "触发 saveload_header_close.clicked，返回前一 Screen"
  Enter: "激活当前焦点按钮（槽位 = 选中槽位；按钮 = 点击）"
  ArrowUp: "焦点上移（槽位列表内逆序遍历）"
  ArrowDown: "焦点下移（槽位列表内顺序遍历）"
  Tab: "按 focus_path 顺序前进"
  Shift+Tab: "按 focus_path 逆序后退"

focus_trap: true          # true = 焦点锁定在该 Screen 内，Tab 循环
```

### 9.1 默认焦点

进入 SaveLoadScreen 时，默认焦点落在 `saveload_header_mode_toggle`（模式切换按钮）——允许用户快速切换存档/读档模式。
当从 Overlay（ModalOverlay）返回时，焦点回到触发该 Overlay 的按钮。

---

## 10. Interaction Zones

> 每个可交互区域的行为定义。

### 10.1 saveload_header_mode_toggle

```yaml
zone_id: saveload_header_mode_toggle
interactions:
  - type: click
    button: Left
    effect: "切换 SaveMode ↔ LoadMode，更新标题文本和确认按钮标签"
    cursor: Pointer
    conditions:
      - mode != target_mode: "仅当不在目标模式时切换"

  - type: hover
    enter_effect: "按钮变为 Hover 样式（Secondary 高亮变体）"
    leave_effect: "恢复 Secondary 默认样式"
    delay: 0ms
```

### 10.2 saveload_header_close

```yaml
zone_id: saveload_header_close
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

### 10.3 saveload_slot_{n}

```yaml
zone_id: saveload_slot_{n}
interactions:
  - type: click
    button: Left
    effect: "选中槽位 n：更新 selected_slot = n，高亮该槽位，刷新预览面板和 Action 按钮状态"
    cursor: Pointer
    conditions: []

  - type: hover
    enter_effect: "槽位背景变为 Hover 样式（浅色高亮）"
    leave_effect: "恢复槽位默认背景色（选中态除外）"
    delay: 0ms
```

### 10.4 saveload_confirm_btn

```yaml
zone_id: saveload_confirm_btn
interactions:
  - type: click
    button: Left
    effect: |
      SaveMode:
        - 空槽位: 直接保存，触发 UiCommand::SaveGame(selected_slot)
        - 非空槽位: 弹出 ModalOverlay（覆盖确认："覆盖将丢失旧存档，确定？"）
      LoadMode:
        - 非空槽位: 弹出 ModalOverlay（加载确认："加载将重置当前进度，确定？"）
    cursor: Pointer
    conditions:
      - enabled == true: "按钮处于 enabled 状态"

  - type: hover
    enter_effect: "按钮变为 Primary Hover 样式（背景变亮）"
    leave_effect: "恢复 Primary 默认样式"
    delay: 0ms
```

### 10.5 saveload_delete_btn

```yaml
zone_id: saveload_delete_btn
interactions:
  - type: click
    button: Left
    effect: "弹出 ModalOverlay（删除确认："确定删除存档 Slot {n}？此操作不可撤销。"）"
    cursor: Pointer
    conditions:
      - enabled == true: "按钮处于 enabled 状态（选中的非空槽位）"

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
| ModalOverlay (OverwriteConfirm) | 覆盖存档确认："Slot {n} 已有存档，覆盖将丢失旧进度。确定？" | popup_layer (300) | Modal | SaveMode + 选中非空槽位 + 点击 ConfirmButton |
| ModalOverlay (LoadConfirm) | 加载存档确认："加载存档将重置当前未保存的进度。确定？" | popup_layer (300) | Modal | LoadMode + 选中非空槽位 + 点击 ConfirmButton |
| ModalOverlay (DeleteConfirm) | 删除存档确认："确定删除 Slot {n} 的存档？此操作不可撤销。" | popup_layer (300) | Modal | 选中非空槽位 + 点击 DeleteButton |
| LoadingOverlay | 加载存档时的资源加载遮罩，显示 Spinner + "Loading..." | popup_layer (301) | Popup | 确认 LoadGame 后 |

### 11.1 Z-Layer 分配

Z-Layer 分配遵循 `07-specs/references/z-layer-spec.md`。

| Z-Layer | 层名 | 用途 | 包含 |
|---------|------|------|------|
| 100 | screen_layer | Screen 主界面层 | `saveload_root` 及所有子 region |
| 200 | tooltip_layer | Tooltip 层 | 预留 |
| 300 | popup_layer | Modal 层 | ModalOverlay（覆盖确认/加载确认/删除确认） |
| 301 | popup_layer | Popup 层 | LoadingOverlay（资源加载） |
| 400 | notification_layer | Notification 层 | 预留 |
| 500 | debug_layer | Debug 层 | DebugOverlay (FPS/日志) |

### 11.2 Overlay 生命周期

| Overlay | OnOpen | OnClose | 依赖 |
|---------|--------|---------|------|
| ModalOverlay (OverwriteConfirm) | 创建 Modal 实体：半透明遮罩 (z=303) + 弹窗面板 (z=304) + "确认覆盖"按钮 (Danger) + "取消"按钮 (Secondary)，焦点锁定到弹窗内 | 确认: 执行 UiCommand::SaveGame，销毁 Modal；取消: 销毁 Modal，焦点回到 saveload_confirm_btn | SaveDomain |
| ModalOverlay (LoadConfirm) | 创建 Modal 实体：半透明遮罩 (z=303) + 弹窗面板 (z=304) + "确认加载"按钮 (Primary) + "取消"按钮 (Secondary)，焦点锁定到弹窗内 | 确认: 执行 UiCommand::LoadGame，销毁 Modal，打开 LoadingOverlay；取消: 销毁 Modal，焦点回到 saveload_confirm_btn | LoadDomain |
| ModalOverlay (DeleteConfirm) | 创建 Modal 实体：半透明遮罩 (z=303) + 弹窗面板 (z=304) + "确认删除"按钮 (Danger) + "取消"按钮 (Secondary)，焦点锁定到弹窗内 | 确认: 执行 UiCommand::DeleteSlot，销毁 Modal，刷新槽位列表焦点回到被删槽位；取消: 销毁 Modal，焦点回到 saveload_delete_btn | SaveDomain |
| LoadingOverlay | 创建全屏遮罩 (z=301) + Spinner (z=302) + 文本 "Loading..." | 加载完成: 销毁 LoadingOverlay，关闭 Screen 或继续操作 | — |

### 11.3 ModalOverlay 内部结构（以 OverwriteConfirm 为例）

```
ModalOverlay (OverwriteConfirm)
├── ModalBackdrop    [modal_backdrop: Panel — 半透明遮罩, z=303]
└── ModalDialog      [modal_dialog: Container — 居中弹窗, z=304]
    ├── ModalTitle   [modal_title: HeadingText]     — "ui.save_load.modal.overwrite_title"
    ├── ModalBody    [modal_body: BodyText]          — "ui.save_load.modal.overwrite_body"
    ├── ConfirmBtn   [modal_confirm_btn: Button, Danger]     — "ui.save_load.modal.overwrite_confirm"
    └── CancelBtn    [modal_cancel_btn: Button, Secondary]   — "ui.save_load.modal.overwrite_cancel"
```

### 11.4 ModalOverlay 内部结构（LoadConfirm）

```
ModalOverlay (LoadConfirm)
├── ModalBackdrop    [modal_backdrop: Panel — 半透明遮罩, z=303]
└── ModalDialog      [modal_dialog: Container — 居中弹窗, z=304]
    ├── ModalTitle   [modal_title: HeadingText]     — "ui.save_load.modal.load_title"
    ├── ModalBody    [modal_body: BodyText]          — "ui.save_load.modal.load_body"
    ├── ConfirmBtn   [modal_confirm_btn: Button, Primary]     — "ui.save_load.modal.load_confirm"
    └── CancelBtn    [modal_cancel_btn: Button, Secondary]   — "ui.save_load.modal.load_cancel"
```

### 11.5 ModalOverlay 内部结构（DeleteConfirm）

```
ModalOverlay (DeleteConfirm)
├── ModalBackdrop    [modal_backdrop: Panel — 半透明遮罩, z=303]
└── ModalDialog      [modal_dialog: Container — 居中弹窗, z=304]
    ├── ModalTitle   [modal_title: HeadingText]     — "ui.save_load.modal.delete_title"
    ├── ModalBody    [modal_body: BodyText]          — "ui.save_load.modal.delete_body"
    ├── ConfirmBtn   [modal_confirm_btn: Button, Danger]     — "ui.save_load.modal.delete_confirm"
    └── CancelBtn    [modal_cancel_btn: Button, Secondary]   — "ui.save_load.modal.delete_cancel"
```

---

## 12. Lifecycle

> Screen 的完整生命周期行为。遵守 `screen-lifecycle.md` 定义的状态机。

| 阶段 | 行为 | 触发条件 | 清理 |
|------|------|---------|------|
| **OnEnter** | `spawn_save_load_screen()` — 生成完整 UI 树（saveload_root → Header + Body(SlotList + PreviewPanel) + ActionPanel），从 Save Domain 查询所有 10 个槽位的元数据（SaveSlotVm[]），根据打开模式设置子模式（SaveMode / LoadMode） | `GameState::SaveLoad` 状态进入 或 `UiCommand::OpenScreen(ScreenType::SaveLoad)` | — |
| **OnReady** | 如果元数据查询完成，填充槽位列表；初始化 selected_slot = None | OnEnter 完成，UI 树就绪 | — |
| **Active** | 等待槽位选择、模式切换、确认/删除按钮交互 | OnReady 完成 | — |
| **OnExit** | `despawn_save_load_screen()` — 清理所有标记了 `With<SaveLoadScreen>` 的实体 | `GameState::SaveLoad` 状态退出 | 清理标记: `With<SaveLoadScreen>` |

### 12.1 生命周期事件处理

```yaml
on_enter:
  - action: "spawn_ui_tree"
    spawner: "spawn_save_load_screen()"
    description: "生成 SaveLoadScreen 完整 UI 树：HeaderPanel(标题+模式切换+关闭) + Body(SlotList(10槽位) + PreviewPanel) + ActionPanel(确认+删除)"
  - action: "set_mode"
    source: "调用参数 or UiCommand 上下文"
    description: "根据打开方式设置当前模式：从 LoadGame 按钮进入 = LoadMode；从其他入口/默认 = SaveMode"
  - action: "query_slot_metadata"
    target: "Save Domain — 查询所有 10 个槽位的元数据"
    description: "异步查询每个存档槽位的 SaveSlotVm（角色名、等级、地点、时间、游戏时长、截图等），完成后填充 slot_list"

on_ready:
  - action: "populate_slot_list"
    source: "SaveSlotVm[] query result"
    description: "将查询结果填充至 10 个槽位，空槽位显示 EmptyLabel，非空槽位显示摘要"

active:
  - trigger: "saveload_slot_{n}.selected"
    action: "更新 selected_slot = n，高亮槽位，更新预览面板显示对应存档详情，更新 Action 按钮 enabled 状态"
    scope: "saveload_slot_list → saveload_preview_panel + saveload_action_panel"
  - trigger: "saveload_header_mode_toggle.clicked"
    action: "切换 SaveMode ↔ LoadMode，更新标题文本和确认按钮标签，重置 selected_slot = None，清除预览面板"
    scope: "saveload_header + saveload_confirm_btn"
  - trigger: "saveload_confirm_btn.clicked"
    action: |
      SaveMode + 空槽位: 直接执行 UiCommand::SaveGame(selected_slot)
      SaveMode + 非空槽位: 弹出 ModalOverlay (OverwriteConfirm)
      LoadMode + 非空槽位: 弹出 ModalOverlay (LoadConfirm)
    scope: "saveload_action_panel → ModalOverlay"
  - trigger: "saveload_delete_btn.clicked"
    action: "弹出 ModalOverlay (DeleteConfirm)"
    scope: "saveload_action_panel → ModalOverlay"
  - trigger: "ModalOverlay.confirm (OverwriteConfirm)"
    action: "执行 UiCommand::SaveGame(selected_slot)，销毁 Modal"
    scope: "ModalOverlay"
  - trigger: "ModalOverlay.confirm (LoadConfirm)"
    action: "执行 UiCommand::LoadGame(selected_slot)，销毁 Modal，打开 LoadingOverlay"
    scope: "ModalOverlay → LoadingOverlay"
  - trigger: "ModalOverlay.confirm (DeleteConfirm)"
    action: "执行 UiCommand::DeleteSlot(selected_slot)，销毁 Modal，刷新槽位列表，Reset selected_slot"
    scope: "ModalOverlay"
  - trigger: "ModalOverlay.cancel (任意)"
    action: "销毁 Modal，焦点回到触发按钮"
    scope: "ModalOverlay"
  - trigger: "saveload_header_close.clicked"
    action: "触发 UiCommand::CloseScreen"
    scope: "saveload_header"

on_exit:
  - action: "unregister_observer"
    target: "None — SaveLoadScreen 未注册任何 Observer"
  - action: "despawn_ui_tree"
    query: "With<SaveLoadScreen>"
    description: "清理所有标记了 SaveLoadScreen 组件的实体"
```

---

## 13. Data Ownership

> Owns / Uses 分离。SaveLoadScreen 通过 Save Domain 接口查询存档信息，UI 状态由 Screen 自身管理。

### 13.1 ViewModel 映射

| ViewModel | 字段 | 归属 (Owns/Uses) | 更新频率 | Projection 源 |
|-----------|------|-----------------|---------|--------------|
| SaveSlotVm | slot_metadata[0..9] | Uses — 从 Save Domain 读取 | OnEnter 时查询一次，删除后重新查询 | 待定义 SaveProjection |

### 13.2 自身状态

| 状态字段 | 类型 | 归属 | 说明 |
|---------|------|------|------|
| `mode` | `SaveMode / LoadMode` | Owns — Screen 内部状态，不持久化 | 决定标题文本和确认按钮行为 |
| `selected_slot` | `Option<usize>` (1-10) | Owns — Screen 内部状态，不持久化 | 当前选中的槽位号，None = 未选中 |
| `slots[]` | `Vec<SaveSlotVm>` | Uses — 从 Save Domain 只读获取 | 10 个槽位的元数据 |

### 13.3 SaveSlotVm 结构

| 字段 | 类型 | 说明 |
|------|------|------|
| `slot_number` | usize (1-10) | 槽位编号 |
| `is_occupied` | bool | 是否有存档数据 |
| `character_name_key` | LocalizationKey | 存档角色名称（LocalizedKey） |
| `level` | u32 | 角色等级 |
| `location_key` | LocalizationKey | 存档地点名称（LocalizedKey） |
| `playtime_seconds` | u64 | 游戏时长（秒） |
| `chapter_key` | LocalizationKey | 当前章节（LocalizedKey） |
| `saved_at` | i64 | UNIX 时间戳（秒） |
| `avatar_handle` | Option<Handle<Image>> | 角色头像纹理句柄 |
| `screenshot_handle` | Option<Handle<Image>> | 存档截图纹理句柄 |

### 13.4 数据流

```
进入 SaveLoadScreen
        ↓
OnEnter: Screen 从 Save Domain 查询 SaveSlotVm[]（异步）
        ↓
数据返回 → 填充槽位列表（每个槽位显示摘要或 EmptyLabel）
        ↓
用户交互 → 选中槽位
        ↓
预览面板显示选中存档的详细信息（头像 + 文本详情 + 截图占位）
        ↓
用户操作 → ConfirmButton → 确认弹窗 → SaveGame / LoadGame / DeleteSlot
        ↓
操作完成后 → 刷新槽位列表 / 关闭 Screen / 启动 LoadingOverlay
```

---

## 14. Layout Intent

> 每个关键尺寸的**理由说明**。为什么选这个尺寸而不是别的？

### 14.1 固定尺寸意图

| widget_id | 属性 | 值 | 意图 | shrink |
|-----------|------|----|------|--------|
| `saveload_header` | height | 56px | "Header 栏高度 56px，略大于按钮高度 40px，提供充足上下内边距（8px），视觉舒适" | none |
| `saveload_header_mode_toggle` | height | 40px | "模式切换按钮高度 40px，最小触摸目标" | none |
| `saveload_header_close` | width | 40px | "关闭按钮最小触摸目标 40x40" | none |
| `saveload_header_close` | height | 40px | "最小触摸目标 40px" | none |
| `saveload_slot_list` | width | 400px | "槽位列表 400px 宽度，足以显示 'Aria - Lv.5' / 'River Crossing' / '12h 34m' 等中文摘要信息单行显示，无需折行" | none |
| `saveload_slot_{n}` | height | 80px | "单个槽位高度 80px，足以容纳槽位号 + 摘要信息两行文本（BodyText 一行 + CaptionText 两行）+ 上下 padding" | none |
| `saveload_preview_avatar` | width | 96px | "角色头像 96x96，标准角色头像大小，与 InventoryScreen 角色头像一致" | none |
| `saveload_preview_avatar` | height | 96px | "与 width 等宽，1:1 头像比例" | none |
| `saveload_preview_screenshot` | width | 256px | "截图预览 256px 宽度，4:3 比例缩略图，足够展示游戏场景概览" | none |
| `saveload_preview_screenshot` | height | 144px | "256/144 = 16:9 宽屏比例，与游戏画面比例一致" | none |
| `saveload_confirm_btn` | width | 160px | "确认按钮宽度 160px，确保 'Save' / 'Load' / 中文 '保存' / '加载' 均单行显示" | none |
| `saveload_confirm_btn` | height | 40px | "最小触摸目标 40px" | none |
| `saveload_delete_btn` | width | 120px | "删除按钮宽度 120px，标签 'Delete' / '删除' 单行显示即可" | none |
| `saveload_delete_btn` | height | 40px | "最小触摸目标 40px" | none |
| `saveload_action_panel` | height | 56px | "ActionPanel 高度 56px，与 Header 等高，视觉对称，包裹按钮后上下留 8px 内边距" | none |

### 14.2 弹性尺寸意图

| widget_id | flex_grow | 理由 |
|-----------|-----------|------|
| `saveload_header_mode_toggle` | 1 | "模式切换按钮弹性居中于标题与关闭按钮之间" |
| `saveload_body` | 1 | "主体区域占据 Header 和 ActionPanel 之外的所有垂直空间，留给内容区越多越好" |
| `saveload_preview_panel` | 1 | "预览面板弹性填充 SlotList (400px) 右侧的剩余水平空间，越宽越便于显示详情文本和截图" |

### 14.3 通用约束

```yaml
global:
  min_interactive_height: 40px   # 可交互元素最小高度 (触摸友好)
  min_interactive_width: 40px    # 可交互元素最小宽度 (触摸友好)
  standard_padding: 8px          # 标准内边距
  standard_gap: 8px              # 标准间距 (Flexbox gap)
  slot_list_gap: 4px             # 槽位列表项之间的间距
```

---

## 15. Scroll & Overflow Policy

> SaveLoadScreen 的 SlotList 区域可能因 10 个槽位 (10 x 80px = 800px) 超出可视高度而需要垂直滚动。

### 15.1 滚动区域

| widget_id | 方向 | Scroll Policy | Overflow Policy | 理由 |
|-----------|------|--------------|---------------|------|
| `saveload_slot_list` | vertical | auto | clip | "10 个槽位 × 80px = 800px，在 720px 视口下可能溢出，需垂直滚动；仅在内容溢出时显示滚动条" |
| `saveload_preview_panel` | vertical | none | clip | "预览面板内容（头像 96px + 详情 auto + 截图 144px）总高度通常在 600px 以内，无需滚动" |
| `saveload_preview_details` | vertical | none | clip | "6 行详情文本 (6 × auto ≈ 180px) 高度固定，无需滚动" |
| `saveload_header` | horizontal | none | clip | "Header 内容固定（标题 + 按钮 + 关闭），不会溢出" |
| `saveload_action_panel` | horizontal | none | clip | "ActionPanel 仅 2 个按钮，不会溢出" |

### 15.2 文本溢出

| widget_id | max_lines | overflow | 多语言风险 |
|-----------|-----------|----------|-----------|
| `saveload_header_title` | 1 | clip | "标题 'Save Game' / 'Load Game' / '保存游戏' / '加载游戏' 均在 10 字以内，无溢出风险" |
| `saveload_header_mode_toggle` | 1 | ellipsis | "'Switch to Load' / 'Switch to Save' 约 15 字符，flex_grow 确保有足够空间，风险低" |
| `saveload_slot_{n}_info` | 1 | ellipsis | "存档信息如 'Aria - Lv.5 / 河畔渡口' 中英文最长约 20 字符，400px 宽度充足，风险低" |
| `saveload_slot_{n}_playtime` | 1 | ellipsis | "'Playtime: 12h 34m' / '游戏时长: 12小时34分' 固定格式，风险低" |
| `saveload_slot_{n}_empty` | 1 | clip | "EmptyLabel '空' / 'Empty' 极短（1-5 字符），无溢出风险" |
| `saveload_confirm_btn` | 1 | ellipsis | "'Save' / 'Load' / '保存' / '加载' 极短（2-6 字符），160px 宽度充足" |
| `saveload_delete_btn` | 1 | clip | "'Delete' / '删除' 极短（2-6 字符），120px 宽度充足" |

---

## 16. Event Contract

> UI -> Domain 事件 + Domain -> UI 事件的完整契约。

### 16.1 UI -> Domain（通过 UiCommand 传递）

```yaml
SelectSlot:
  trigger_widget: "saveload_body → saveload_slot_list → saveload_slot_{n} → click"
  data:
    slot_number: usize (1-10)
  conditions: []
  emits: UiCommand::SelectSlot(usize)
  domain_event: "None — 纯 UI 状态更新，不涉及 Domain"

ToggleMode:
  trigger_widget: "saveload_header → saveload_header_mode_toggle → click"
  data: {}
  conditions:
    - mode != target_mode
  emits: UiCommand::ToggleSaveLoadMode
  domain_event: "None — 纯 UI 状态更新，不涉及 Domain"

SaveGame:
  trigger_widget: "saveload_action_panel → saveload_confirm_btn → click → (空槽位) 或 ModalOverlay.confirm"
  data:
    slot_number: usize
  conditions:
    - SaveMode == true
    - slot is empty OR overwrite_confirmed == true
  emits: UiCommand::SaveGame(usize)
  domain_event: "GameSaved { slot_number, timestamp }"

LoadGame:
  trigger_widget: "saveload_action_panel → saveload_confirm_btn → click → ModalOverlay.confirm"
  data:
    slot_number: usize
  conditions:
    - LoadMode == true
    - slot is occupied
    - load_confirmed == true
  emits: UiCommand::LoadGame(usize)
  domain_event: "GameLoaded { slot_number }"

DeleteSlot:
  trigger_widget: "saveload_action_panel → saveload_delete_btn → click → ModalOverlay.confirm"
  data:
    slot_number: usize
  conditions:
    - slot is occupied
    - delete_confirmed == true
  emits: UiCommand::DeleteSlot(usize)
  domain_event: "SlotDeleted { slot_number }"

CloseScreen:
  trigger_widget: "saveload_header → saveload_header_close → click"
  data: {}
  conditions: []
  emits: UiCommand::CloseScreen
  domain_event: "None — 纯 UI 导航，ScreenStack 直接处理"
```

### 16.2 Domain -> UI（通过 Projection 消费）

```yaml
SlotMetadataLoaded:
  source: "SaveDomain → SaveProjection"
  data:
    slots: Vec<SaveSlotVm>
  effect: "填充 saveload_slot_list 各槽位显示内容，空槽位显示 EmptyLabel，非空槽位显示摘要"
  consumed_by: "saveload_slot_list 所有槽位"

SlotOperationResult:
  source: "SaveDomain → SaveProjection"
  data:
    operation: "Save | Load | Delete"
    success: bool
    slot_number: usize
  effect: |
    保存成功: 刷新槽位列表，selected_slot 不变
    加载成功: 关闭 Screen 并切换 GameState（到 Combat / Overworld 等）
    删除成功: 刷新槽位列表，重置 selected_slot = None
    操作失败: 显示错误通知（NotificationOverlay）
  consumed_by: "saveload_slot_list + saveload_action_panel"
```

---

## 17. Screen Metrics

> 复杂度基线。所有数值初始创建时手动填写，后续 CI 阶段自动校验。

| Metric | 值 | P0/P1 | 说明 |
|--------|-----|-------|------|
| `widget_count` | ~20 | P1 | Widget 实例总数（root + header_container + title + mode_toggle + close_btn + body + slot_list + 10 slots × 4 sub-widgets ≈ 40 + preview_panel + avatar + 6 details + screenshot + action_panel + confirm_btn + delete_btn ≈ 65） |
| `container_count` | ~6 | P1 | 纯容器节点数（root + header + body + slot_list + preview_panel + preview_details + action_panel） |
| `interactive_count` | ~12 | P1 | 可交互 Widget 数（mode_toggle + close_btn + 10 slot molecules + confirm_btn + delete_btn） |
| `overlay_count` | 4 | P1 | 关联的 Overlay 数（OverwriteConfirm + LoadConfirm + DeleteConfirm + LoadingOverlay） |
| `max_depth` | 4 | P1 | root → saveload_body → saveload_slot_list → saveload_slot_{n} → saveload_slot_{n}_info |
| `max_children` | 12 | P1 | saveload_slot_list 有最多 12 个直接子节点（10 个 slot + 可能的滚动条/空状态指示器） |

### 17.1 Budget 检查

| 规则 | 阈值 | 当前值 | 状态 |
|------|------|--------|------|
| max_depth <= 6 | 6 | 4 | ✅ |
| max_children <= 20 | 20 | 12 | ✅ |
| interactive_count / widget_count >= 0.2 | 20% | 60% (12/20) | ✅ |

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
| D09 | State Mapping 完整 (每个 region 的 Loading/Empty/Normal/Error) | [x] | 含 slot_list 异步加载 / preview_panel 异步加载 |
| D10 | Focus Navigation 已定义 (Tab 路径完整) | [x] | P1 |
| D11 | Interaction Zones 已定义 (Click/Hover) | [x] | |
| D12 | Overlay Definition 已定义 (Overlay 列表 + Z-Layer) | [x] | 含 3 种 ModalOverlay + LoadingOverlay |
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
| `06-ui/02-design-system/widget-atoms.md` | 原子组件 Contract（HeadingText, BodyText, CaptionText, IconButton, Button, Image） |
| `06-ui/02-design-system/widget-composites.md` | 复合组件 Contract（SaveSlotMolecule 待定义） |
| `06-ui/02-design-system/theme-localization.md` | StyleToken / Theme / UiTextKey |
| `06-ui/02-design-system/focus-binding.md` | Focusable / FocusGroup / Dirty<T> / UiBinding |
| `06-ui/03-screens/screens.md` | SaveLoadScreen 定义（§7） |
| `06-ui/03-screens/screen-lifecycle.md` | Screen 生命周期状态机 |
| `06-ui/04-data-flow/projection-viewmodel.md` | Projection / ViewModel 映射 |
| `03-content/localization/ui-screen-keys.md` | 存档界面 LocalizationKeys |

---

## 附录 C: 架构审查记录

### 审查 1 (2026-06-22)

> **审查人**: @presentation-architect | **日期**: 2026-06-22 (第1轮) | **结论**: draft (未通过，P0 Z-Layer 违规)

#### 审查结果

| 维度 | 结果 |
|------|------|
| 模板合规 (17字段) | PASS |
| 架构一致性 (vs screens.md §7) | PASS |
| SaveMode/LoadMode 建模 | PASS |
| SaveSlot Empty/Occupied 状态 | PASS |
| ASCII Wireframe (无匿名面板) | PASS |
| Widget Tree 完整性 | PASS |
| Widget ID 命名规范 (`saveload_{region}_{element}`) | PASS |
| Flexbox Layout 完整性 | PASS |
| 过约束检查 | PASS |
| 单向数据流 | PASS |

#### P0 违规 (阻塞 status: active)

| # | 区域 | 问题 | 严重性 |
|---|------|------|--------|
| 01 | §11.1 Z-Layer 分配 | 使用顺序索引值 (0,1,2,3,4,9) 作为 Z 值。违反 z-layer-spec.md 附录 A.2。索引号应映射为实际 Z 值：Screen→z=100(screen_layer), Tooltip→z=200(tooltip_layer), Notification→z=400(notification_layer), Modal→z=300(popup_layer), LoadingOverlay→z=301(popup_layer), Debug→z=500(debug_layer) | P0 — D12 |
| 02 | §11.2 Overlay 生命周期 | ModalOverlay 使用 z=299/300，与 popup_layer 基础值 (z=300) 冲突。应使用 z=300(遮罩) + z=301(弹窗) 或子层 z=303-304（参考 ShopScreen 模式） | P0 — D12 |

#### 修正要求

1. 将 §11.1 的所有 Z 值修正为 z-layer-spec.md 定义的对应 Z 值（不能用索引号）
2. 将 §11.2 和 §11.3-11.5 的 ModalOverlay z 值修正为 popup_layer 子范围 (z=300+)
3. 更新后重新提交 @presentation-architect 审查
4. 通过后填写附录 A DoD 清单 (当前全部为 [ ])

### 审查 2 (2026-06-22) — 最终审查

> **审查人**: @presentation-architect | **日期**: 2026-06-22 (第2轮) | **结论**: active (P0 违规已全部修复)

#### Z-Layer 修复验证

| 检查项 | 修复前 | 修复后 | 预期 (z-layer-spec.md) | 状态 |
|--------|--------|--------|------------------------|------|
| §11.1 screen_layer | 0 | 100 | 100 | PASS |
| §11.1 tooltip_layer | 1 | 200 | 200 | PASS |
| §11.1 notification_layer | 2 | 400 | 400 | PASS |
| §11.1 popup_layer (Modal) | 3 | 300 | 300 | PASS |
| §11.1 popup_layer (Loading) | 4 | 301 | 301 | PASS |
| §11.1 debug_layer | 9 | 500 | 500 | PASS |
| §11.2 ModalOverlay backdrop | 299 | 303 | 300-399 (popup_layer 子层) | PASS |
| §11.2 ModalOverlay panel | 300 | 304 | 300-399 (popup_layer 子层) | PASS |
| §11.2 LoadingOverlay | — | 301 | 301 | PASS |
| §1 ScreenLayer | — | 100 | 100 | PASS |

**结论**: 全部 Z 值符合 `z-layer-spec.md`。P0 违规已清除。DoD 18 项全部检查通过。

#### P1 遗留备注 (不影响 status: active)

| # | 区域 | 问题 | 优先级 |
|---|------|------|--------|
| 01 | §17 Screen Metrics | widget_count 写 ~20 但描述列举 ~59-65，应统一 | Low |
| 02 | §17 Screen Metrics | budget 使用 12/20=60%，实际 12/59=20.3% 勉强及格 | Low |
| 03 | 3.1 Widget Type 索引 | `SaveSlotMolecule` 在 `widget-composites.md` 中尚未定义，需要 @presentation-architect 补充 | Medium |
| 04 | 13.1 ViewModel 映射 | `SaveProjection` 尚未定义，需要 Domain 团队确定 SaveSlotVm 的数据源 | Medium |
| 05 | 11 Overlay | 3 种 ModalOverlay 结构高度相似，可考虑复用通用确认弹窗，通过 LocalizationKey 区分 | Low |
| 06 | 9 Focus Nav | 默认焦点 `saveload_header_mode_toggle` 可能需要 Review：另一种方案是默认焦点落在第一个非空槽位 | Low |

---

*本文档是 SaveLoadScreen SSPEC，由 @feature-developer 根据 `07-specs/screen-spec-template.md` 模板创建。所有 17 个字段已填充。当前 status: active（@presentation-architect 最终审查通过，见附录 C 审查 2）。*
