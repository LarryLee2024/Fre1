---
id: 07-specs.battle-screen
title: BattleScreen Specification — AI-Consumable Layout & Interaction Spec
status: draft
owner: presentation-architect
created: 2026-06-22
tags:
  - ui
  - screen-spec
  - battle
  - draft
---

# BattleScreen

> **职责**: @presentation-architect | **上游**: ADR-066 (Screen Spec), `07-specs/README.md` (总纲)
> **状态**: 初始 draft，完成后改为 active

**P0 字段**: 1-14 (Screen Header / ASCII Wireframe / Widget Tree / Flexbox Layout / Responsive Rules / Region Responsibility / Widget Contract / State Mapping / Focus Nav / Interaction Zones / Overlay / Lifecycle / Data Ownership / Layout Intent)
**P1 字段**: 15-17 (Scroll & Overflow / Event Contract / Screen Metrics)

---

## 1. Screen Header

| 属性 | 值 |
|------|-----|
| Screen Name | `BattleScreen` — 对应 `GameState::Combat` |
| Purpose | 战斗主界面，提供回合信息显示、战斗地图渲染、角色状态面板、行动菜单和结束回合功能，是游戏最核心的 Screen |
| Navigation | 战前准备流程完成自动进入；战斗结束自动退出（Victory/Defeat → ResultScreen）；不支持 Esc 返回（战斗不可中途取消） |
| GameState | `GameState::Combat` |
| ScreenLayer 层级 | 0（主界面层） |
| 加载模式 | Ephemeral（每次进入 Combat 重新 spawn，退出时完整 despawn） |
| 过渡动画 | Fade(0.3s) |
| 变体 | None（当前 MVP；后续迭代补充 BossBattle、AutoBattle 变体） |

**P0 架构约束**（来自 code-review 的 4 个 P0 违规修复）:

1. **Factory 构造**: BattleScreen 根容器必须通过 `spawn_panel` 工厂函数创建，禁止使用原始 `commands.spawn((Node{...}, BattleScreen))`。违规位置: `battle/mod.rs:53`
2. **ViewModel 隔离**: UI 系统禁止直接查询 `TurnQueue`（`core::domains::combat` 类型），`unit_id` 必须通过 `BattleHudVm.current_unit_id` 获取。违规位置: `battle/systems.rs:30`
3. **ViewModel 隔离**: UI 系统禁止直接查询 `Res<State<BattlePhase>>`，区域可见性必须由 ViewModel 字段驱动。违规位置: `battle/visibility.rs:20`
4. **LocalizationKey 文本**: 所有用户可见文本必须使用 `spawn_localized_text` / `spawn_localized_button` 工厂，禁止硬编码字符串。违规位置: `battle/mod.rs:73`

---

## 2. ASCII Wireframe

> 纯文本线框图。所有区域必须命名（`widget_id`），禁止匿名面板。
> 水平线 `---` 表示横向分隔，竖线 `|` 表示纵向分隔。

```
┌──────────────────────────────────────────────────────────────────────────────────┐
│  [battle_top_bar: h:64]                                                           │
│  ┌──────────────────────────┐  ┌─────────────────┐              ┌──────────────┐  │
│  │ [battle_turn_indicator]  │  │ [battle_phase_  │              │ [battle_end_ │  │
│  │  "Turn: {turn_number}"   │  │  label]          │              │  turn_btn]   │  │
│  │                          │  │  "Player Turn"   │              │  [End Turn]  │  │
│  └──────────────────────────┘  └─────────────────┘              └──────────────┘  │
├──────────────────────────────────────────────────────────────────────────────────┤
│  [battle_battle_area: flex_grow:1]                                                │
│  ┌──────────────────────────────────────────────────────────────────────────────┐│
│  │                                     ┌──────────┐                             ││
│  │                                     │minimap   │                             ││
│  │                                     │[battle_  │                             ││
│  │        (6-Layer Battle Map)         │minimap]  │                             ││
│  │                                     │          │                             ││
│  │                                     └──────────┘                             ││
│  └──────────────────────────────────────────────────────────────────────────────┘│
├──────────────────────────────────────────────────────────────────────────────────┤
│  [battle_char_panel: h:140]                                                       │
│  ┌──────────┬──────────────────────────────────────────────────────────────────┐  │
│  │ [battle_ │  [battle_char_name]        LV [battle_char_level]                │  │
│  │  char_   │  Aria                          5                                 │  │
│  │  avatar] │  [battle_hp_bar] ████████░░░░░░░░░░░░░░░░░   80/100              │  │
│  │          │  [battle_mp_bar] ████████░░░░░░░░░░░░░░░░░   40/50               │  │
│  │          │  [battle_ap_bar] ██████████████████████████   6/6                │  │
│  │          │  [battle_buff_area] [icon0][icon1][icon2][icon3]                 │  │
│  └──────────┴──────────────────────────────────────────────────────────────────┘  │
├──────────────────────────────────────────────────────────────────────────────────┤
│  [battle_action_menu: h:80]                                                       │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┬──────────┐              │
│  │ [battle_ │ [battle_ │ [battle_ │ [battle_ │ [battle_ │ [battle_ │              │
│  │  attack_  │  skill_  │  defend_  │  item_   │  move_   │  wait_   │              │
│  │  btn]    │  btn]    │  btn]    │  btn]    │  btn]    │  btn]    │              │
│  └──────────┴──────────┴──────────┴──────────┴──────────┴──────────┘              │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### 2.1 Region 索引

| widget_id | 类型 | 用途 | 对应 Wireframe 位置 |
|-----------|------|------|-------------------|
| `battle_screen_root` | Screen | Screen 根容器，全屏，垂直排列四个 region | 整个 Wireframe 外框 |
| `battle_top_bar` | Container | 顶栏，水平排列回合信息、阶段标签和结束回合按钮 | Wireframe 顶行 |
| `battle_turn_indicator` | BodyText | 显示当前回合数字（UiBinding::Turn），绑定 BattleHudVm.turn_number | 顶栏左侧 |
| `battle_phase_label` | CaptionText | 显示当前阶段文本（UiBinding::Phase），绑定 BattleHudVm.phase_key 本地化 | 顶栏中间 |
| `battle_end_turn_btn` | Button, Danger | 结束当前回合按钮，点击触发 UiCommand::EndTurn | 顶栏右侧 |
| `battle_battle_area` | Container | 战斗地图区域，由 Tactical Domain 管理 6 层渲染，不直接组合原子 Widget | Wireframe 第二行（弹性空间） |
| `battle_minimap` | Image | 小地图缩略图，用于战场全局概览 | 战斗区域右上角浮层 |
| `battle_char_panel` | Container | 角色信息面板，水平排列头像（左）和详细状态（右） | Wireframe 第三行 |
| `battle_char_avatar` | Image | 角色头像图片 | 角色面板左侧 |
| `battle_char_name` | HeadingText | 角色名称文本（UiBinding::Name），绑定 BattleHudVm.unit_name 本地化或字符名 | 角色面板右侧顶部 |
| `battle_char_level` | CaptionText | 角色等级文本（UiBinding::CharacterLevel），绑定 BattleHudVm.unit_level | 角色名称右侧 |
| `battle_hp_bar` | ProgressBar | HP 血条，显示 BattleHudVm.hp / BattleHudVm.max_hp | 角色面板第三行 |
| `battle_hp_text` | CaptionText | HP 数值文本，显示 "hp/max_hp"，绑定 BattleHudVm.hp + max_hp | HP 条右侧 |
| `battle_mp_bar` | ProgressBar | MP 条，显示 BattleHudVm.mp / BattleHudVm.max_mp | 角色面板第四行 |
| `battle_mp_text` | CaptionText | MP 数值文本，显示 "mp/max_mp"，绑定 BattleHudVm.mp + max_mp | MP 条右侧 |
| `battle_ap_bar` | ProgressBar | AP 行动点条，显示 BattleHudVm.ap / BattleHudVm.max_ap | 角色面板第五行 |
| `battle_buff_area` | Container | Buff 图标水平排列容器，固定区域容纳最多 4 个 buff 图标 | 角色面板底部 |
| `battle_buff_icons_0` | Icon | 第 1 个 Buff 图标（UiBinding::BuffSlot(0)），指向 BuffDef.icon | buff_area 左一 |
| `battle_buff_icons_1` | Icon | 第 2 个 Buff 图标（UiBinding::BuffSlot(1)） | buff_area 左二 |
| `battle_buff_icons_2` | Icon | 第 3 个 Buff 图标（UiBinding::BuffSlot(2)） | buff_area 左三 |
| `battle_buff_icons_3` | Icon | 第 4 个 Buff 图标（UiBinding::BuffSlot(3)） | buff_area 左四 |
| `battle_action_menu` | Container | 行动菜单底栏，水平排列 6 个行动按钮 | Wireframe 底行 |
| `battle_attack_btn` | Button, Primary | 攻击按钮（LocalizedKey: ui.battle.attack） | 行动菜单第一个 |
| `battle_skill_btn` | Button, Secondary | 技能按钮（LocalizedKey: ui.battle.skill），激活后切换至技能子菜单 | 行动菜单第二个 |
| `battle_defend_btn` | Button, Secondary | 防御按钮（LocalizedKey: ui.battle.defend） | 行动菜单第三个 |
| `battle_item_btn` | Button, Secondary | 物品按钮（LocalizedKey: ui.battle.item），激活后弹出 InventoryOverlay | 行动菜单第四个 |
| `battle_move_btn` | Button, Secondary | 移动按钮（LocalizedKey: ui.battle.move），激活后进入移动选择模式 | 行动菜单第五个 |
| `battle_wait_btn` | Button, Secondary | 待机按钮（LocalizedKey: ui.battle.wait） | 行动菜单第六个 |

---

## 3. Widget Tree

> 标注 `[widget_id: WidgetType]` 的树结构。禁止隐藏节点，必须完整。

```
ScreenRoot                                                  [battle_screen_root: Screen]
├── TopBar                                                   [battle_top_bar: Container]
│   ├── TurnIndicator                                        [battle_turn_indicator: BodyText]
│   ├── PhaseLabel                                           [battle_phase_label: CaptionText]
│   └── EndTurnButton                                        [battle_end_turn_btn: Button, Danger]
├── BattleArea                                               [battle_battle_area: Container]
│   └── MapLayerRenderer                                     [map_layer: Container — Tactical Domain manages]
│       └── MinimapOverlay                                   [battle_minimap: Image]
├── CharacterPanel                                           [battle_char_panel: Container]
│   ├── AvatarImage                                          [battle_char_avatar: Image]
│   └── CharStats                                            [battle_char_stats: Container]
│       ├── NameLine                                         [battle_char_name_line: Container]
│       │   ├── CharName                                     [battle_char_name: HeadingText]
│       │   └── CharLevel                                    [battle_char_level: CaptionText]
│       ├── HpRow                                            [battle_hp_row: Container]
│       │   ├── HpBar                                        [battle_hp_bar: ProgressBar]
│       │   └── HpText                                       [battle_hp_text: CaptionText]
│       ├── MpRow                                            [battle_mp_row: Container]
│       │   ├── MpBar                                        [battle_mp_bar: ProgressBar]
│       │   └── MpText                                       [battle_mp_text: CaptionText]
│       ├── ApRow                                            [battle_ap_row: Container]
│       │   ├── ApBar                                        [battle_ap_bar: ProgressBar]
│       │   └── ApText                                       [battle_ap_text: CaptionText]
│       └── BuffArea                                         [battle_buff_area: Container]
│           ├── BuffSlot0                                    [battle_buff_icons_0: Icon]
│           ├── BuffSlot1                                    [battle_buff_icons_1: Icon]
│           ├── BuffSlot2                                    [battle_buff_icons_2: Icon]
│           └── BuffSlot3                                    [battle_buff_icons_3: Icon]
└── ActionMenu                                               [battle_action_menu: Container]
    ├── AttackButton                                         [battle_attack_btn: Button, Primary]
    ├── SkillButton                                          [battle_skill_btn: Button, Secondary]
    ├── DefendButton                                         [battle_defend_btn: Button, Secondary]
    ├── ItemButton                                           [battle_item_btn: Button, Secondary]
    ├── MoveButton                                           [battle_move_btn: Button, Secondary]
    └── WaitButton                                           [battle_wait_btn: Button, Secondary]
```

### 3.1 Widget Type 索引

| widget_id | WidgetType | 定义位置 | 复用于 |
|-----------|-----------|---------|--------|
| `battle_turn_indicator` | `Atom: BodyText` | `widget-atoms.md §BodyText` | — |
| `battle_phase_label` | `Atom: CaptionText` | `widget-atoms.md §CaptionText` | — |
| `battle_end_turn_btn` | `Atom: Button (Danger)` | `widget-atoms.md §Button` | — |
| `battle_minimap` | `Atom: Image` | `widget-atoms.md §Image` | — |
| `battle_char_avatar` | `Atom: Image` | `widget-atoms.md §Image` | InventoryScreen character portrait |
| `battle_char_name` | `Atom: HeadingText` | `widget-atoms.md §HeadingText` | — |
| `battle_char_level` | `Atom: CaptionText` | `widget-atoms.md §CaptionText` | — |
| `battle_hp_bar` | `Atom: ProgressBar` | `widget-atoms.md §ProgressBar` | SettingsScreen audio volume sliders |
| `battle_hp_text` | `Atom: CaptionText` | `widget-atoms.md §CaptionText` | — |
| `battle_mp_bar` | `Atom: ProgressBar` | `widget-atoms.md §ProgressBar` | — |
| `battle_mp_text` | `Atom: CaptionText` | `widget-atoms.md §CaptionText` | — |
| `battle_ap_bar` | `Atom: ProgressBar` | `widget-atoms.md §ProgressBar` | — |
| `battle_buff_icons_0..3` | `Atom: Icon` | `widget-atoms.md §Icon` | — |
| `battle_attack_btn` | `Atom: Button (Primary)` | `widget-atoms.md §Button` | — |
| `battle_skill_btn` | `Atom: Button (Secondary)` | `widget-atoms.md §Button` | — |
| `battle_defend_btn` | `Atom: Button (Secondary)` | `widget-atoms.md §Button` | — |
| `battle_item_btn` | `Atom: Button (Secondary)` | `widget-atoms.md §Button` | — |
| `battle_move_btn` | `Atom: Button (Secondary)` | `widget-atoms.md §Button` | — |
| `battle_wait_btn` | `Atom: Button (Secondary)` | `widget-atoms.md §Button` | — |

---

## 4. Flexbox Layout

> YAML 格式。每个 widget_id 必须有 direction / width / height / flex_grow / intent。

```yaml
## Flexbox Layout — BattleScreen
## width/height: px 值或 "auto" 或 "fill"
## flex_grow: 0=不增长, 1=等分剩余空间, 2=双倍增长
## shrink: none/low/high — 收缩优先级

battle_screen_root:
  direction: column
  width: 100%
  height: 100%
  flex_grow: 0
  intent: "Screen 根容器，占满视口，垂直排列 TopBar / BattleArea / CharPanel / ActionMenu"

## ── Top Bar ──

battle_top_bar:
  direction: row
  width: 100%
  height: 64
  flex_grow: 0
  shrink: none
  intent: "顶栏，固定 64px 高度，水平排列回合信息 + 阶段标签 + 结束回合按钮"

battle_turn_indicator:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  shrink: none
  intent: "回合数字指示器，auto 宽度适配 'Turn: {n}' 文本，固定不压缩"

battle_phase_label:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  shrink: low
  intent: "阶段标签文本，auto 宽度适配本地化文本，可低优先级压缩"

battle_end_turn_btn:
  direction: row
  width: 120
  height: 40
  flex_grow: 0
  shrink: none
  intent: "结束回合按钮，固定 120x40 保证可点击区域 >= 40x40，Danger 样式突出不可逆操作"

## ── Battle Area ──

battle_battle_area:
  direction: column
  width: fill
  height: fill
  flex_grow: 1
  intent: "战斗地图区域，弹性填充除顶栏 + 角色面板 + 菜单外的所有剩余空间，作为 Tactical Domain 的渲染容器"

battle_minimap:
  direction: column
  width: 160
  height: 120
  flex_grow: 0
  shrink: none
  intent: "小地图缩略图，固定 160x120，绝对定位在 battle_area 右上角"

## ── Character Panel ──

battle_char_panel:
  direction: row
  width: 100%
  height: 140
  flex_grow: 0
  shrink: none
  intent: "角色面板，固定 140px 高度，水平排列头像(左)和状态信息(右)"

battle_char_avatar:
  direction: column
  width: 100
  height: 100
  flex_grow: 0
  shrink: none
  intent: "角色头像，固定 100x100 正方形，位于 CharPanel 左侧"

battle_char_name:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  shrink: none
  intent: "角色名称文本，auto 宽度适配角色名，不压缩"

battle_char_level:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  shrink: none
  intent: "角色等级文本，auto 宽度适配 'Lv.{n}' 格式"

battle_hp_bar:
  direction: row
  width: 200
  height: 16
  flex_grow: 0
  shrink: low
  intent: "HP 血条，固定 200px 宽度确保视觉效果统一，16px 高度保证可见性"

battle_hp_text:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  shrink: none
  intent: "HP 数值文本 'hp/max_hp'，auto 宽度，不压缩"

battle_mp_bar:
  direction: row
  width: 200
  height: 16
  flex_grow: 0
  shrink: low
  intent: "MP 条，与 HP 条同宽同高保证视觉统一"

battle_mp_text:
  direction: row
  width: auto
  height: auto
  flex_grow: 0
  shrink: none
  intent: "MP 数值文本 'mp/max_mp'"

battle_ap_bar:
  direction: row
  width: 200
  height: 12
  flex_grow: 0
  shrink: low
  intent: "AP 行动点条，12px 高度（略低于 HP/MP 条），区分视觉层级"

battle_buff_area:
  direction: row
  width: auto
  height: 24
  flex_grow: 0
  shrink: none
  intent: "Buff 图标容器，auto 宽度，固定 24px 高度，水平排列最多 4 个图标"

battle_buff_icons_0:
  direction: row
  width: 24
  height: 24
  flex_grow: 0
  shrink: none
  intent: "Buff 图标 0，固定 24x24 正方形，显示 buff 状态图标"

battle_buff_icons_1:
  direction: row
  width: 24
  height: 24
  flex_grow: 0
  shrink: none
  intent: "Buff 图标 1，固定 24x24"

battle_buff_icons_2:
  direction: row
  width: 24
  height: 24
  flex_grow: 0
  shrink: none
  intent: "Buff 图标 2，固定 24x24"

battle_buff_icons_3:
  direction: row
  width: 24
  height: 24
  flex_grow: 0
  shrink: none
  intent: "Buff 图标 3，固定 24x24"

## ── Action Menu ──

battle_action_menu:
  direction: row
  width: 100%
  height: 80
  flex_grow: 0
  shrink: none
  intent: "行动菜单底栏，固定 80px 高度，水平排列 6 个行动按钮，居中分布"

battle_attack_btn:
  direction: row
  width: 120
  height: 48
  flex_grow: 0
  shrink: none
  intent: "攻击按钮，固定 120x48，Primary 样式突出主行动，高度 >= 40px 触摸友好"

battle_skill_btn:
  direction: row
  width: 120
  height: 48
  flex_grow: 0
  shrink: none
  intent: "技能按钮，固定 120x48，Secondary 样式"

battle_defend_btn:
  direction: row
  width: 120
  height: 48
  flex_grow: 0
  shrink: none
  intent: "防御按钮，固定 120x48，Secondary 样式"

battle_item_btn:
  direction: row
  width: 120
  height: 48
  flex_grow: 0
  shrink: none
  intent: "物品按钮，固定 120x48，Secondary 样式"

battle_move_btn:
  direction: row
  width: 120
  height: 48
  flex_grow: 0
  shrink: none
  intent: "移动按钮，固定 120x48，Secondary 样式"

battle_wait_btn:
  direction: row
  width: 120
  height: 48
  flex_grow: 0
  shrink: none
  intent: "待机按钮，固定 120x48，Secondary 样式"
```

---

## 5. Responsive Rules

| 条件 | 行为 | 影响区域 |
|------|------|---------|
| width < 1280px | strategy: "none" — 当前不实现响应式 | 全部 |
| height < 720px | strategy: "none" — 当前不实现响应式 | 全部 |
| width < 1024px | **后续迭代**: 隐藏 ActionMenu 为收缩按钮，点击展开 | battle_action_menu |
| width < 800px | **后续迭代**: 隐藏 CharPanel 为浮层按钮，点击展开 | battle_char_panel |

**最小支持分辨率**: 1280 x 720 (16:9)
**设计分辨率**: 1920 x 1080 (16:9)

---

## 6. Region Responsibility

> 每个 region 3-8 条职责。明确该区域"展示什么"和"不做什么"。

### 6.1 battle_top_bar

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示当前回合数字，绑定 BattleHudVm.turn_number，格式为 "ui.battle.turn.label: {n}" | Display | 回合信息不显示或数字错误 |
| R02 | 展示当前阶段文本，绑定 BattleHudVm.phase_key，通过 LocalizationKey 渲染（"ui.battle.phase.player"/"ui.battle.phase.enemy"） | Display | 阶段标签缺失或未本地化 |
| R03 | 响应 EndTurn 按钮点击，触发 UiCommand::EndTurn | Interaction | 玩家无法结束回合 |
| R04 | EndTurn 按钮在非玩家回合时 disabled（或隐藏），防止误操作 | Interaction/Validation | 非玩家回合可点击 EndTurn 导致逻辑错误 |

**不负责**:
- 显示角色状态信息（属于 battle_char_panel 职责）
- 显示战斗地图（属于 battle_battle_area 职责）

### 6.2 battle_battle_area

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 作为 Tactical Domain 6 层地图渲染的容器，提供全尺寸渲染区域 | Display/Container | 地图渲染范围不足或错位 |
| R02 | 渲染小地图缩略图（battle_minimap），提供战场全局概览 | Display | 玩家无法感知战场全局 |
| R03 | 响应战斗地图内的单位选中（SelectTarget），通过 UiCommand::SelectTarget 传递选中实体 | Interaction | 玩家无法选择攻击目标 |
| R04 | 响应地图点击/拖拽进行视角平移和缩放（由 Tactical Camera 处理） | Interaction | 玩家无法自由查看战场 |
| R05 | 在 Loading/Empty/Error 状态下展示对应的占位/错误 UI | Display | 地图加载失败时无反馈 |

**不负责**:
- 渲染 UI 按钮或菜单（属于 action_menu 职责）
- 展示角色状态数值（属于 char_panel 职责）
- 实现地图滚动的逻辑（由 Tactical Domain 管理，BattleScreen 仅提供容器）

### 6.3 battle_char_panel

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示当前选中/行动单位的角色头像图片 | Display | 头像显示错误或缺失 |
| R02 | 展示角色名称文本，绑定 BattleHudVm.unit_name（UiBinding::Name） | Display | 角色名显示错误 |
| R03 | 展示角色等级文本，绑定 BattleHudVm.unit_level（UiBinding::CharacterLevel） | Display | 等级信息不显示或错误 |
| R04 | 展示 HP 血条 + 数值，绑定 BattleHudVm.hp / max_hp（UiBinding::Hp），数值变更时通过 Dirty<BattleHudVm> 刷新 | Display | HP 显示与实际不符 |
| R05 | 展示 MP 条 + 数值，绑定 BattleHudVm.mp / max_mp（UiBinding::Mp） | Display | MP 显示与实际不符 |
| R06 | 展示 AP 行动点条，绑定 BattleHudVm.ap / max_ap（UiBinding::Ap） | Display | AP 显示与实际不符 |
| R07 | 展示 Buff 图标列表（最多 4 个），绑定 BattleHudVm 的 buff_slots（UiBinding::BuffSlot(n)） | Display | Buff 图标缺失或错误 |
| R08 | 无选中单位时展示 Empty 状态（"ui.battle.char_panel.empty" 提示文本） | Display | 无单位时面板空白无提示 |

**不负责**:
- 显示回合信息（属于 top_bar 职责）
- 提供行动选择（属于 action_menu 职责）

### 6.4 battle_action_menu

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示 6 个行动按钮（Attack/Skill/Defend/Item/Move/Wait），所有文本通过 LocalizationKey 渲染 | Display | 按钮文本硬编码未本地化 |
| R02 | Attack 按钮点击触发 UiCommand::CastSkill(slot=0, target_id) — 默认普通攻击 | Interaction | 玩家无法发起攻击 |
| R03 | Skill 按钮点击切换至技能子菜单（弹出 SkillOverlay），展示可用技能列表 | Interaction | 技能系统无法访问 |
| R04 | Defend 按钮点击触发 UiCommand::Defend | Interaction | 防御动作无法执行 |
| R05 | Item 按钮点击弹出 InventoryOverlay，展示可用物品列表 | Interaction | 战斗中无法使用物品 |
| R06 | Move 按钮点击进入移动模式（由 Tactical Domain 处理移动范围高亮和路径选择） | Interaction | 单位无法移动 |
| R07 | Wait 按钮点击触发 UiCommand::Wait，结束当前单位行动 | Interaction | 待机动作无法执行 |
| R08 | 根据 BattleHudVm.available_actions 控制按钮的 enabled/disabled 状态（如 AP 不足时 Attack disabled） | Validation | 玩家可执行非法行动 |

**不负责**:
- 显示角色状态信息（属于 char_panel 职责）
- 渲染战斗地图（属于 battle_area 职责）

---

## 7. Widget Contract

> Inputs / Outputs / Selection Model。对于复合组件 (Organism)，引用其定义的 Contract；对于直接组合的原语，在此列明。

### 7.1 battle_turn_indicator

```yaml
widget_id: battle_turn_indicator
widget_type: BodyText
defined_in: "widget-atoms.md §BodyText"

inputs:
  - name: text
    type: LocalizedText
    source: "BattleHudVm.turn_number"
    default: "Turn: 1"

outputs: []

selection_model:
  type: none
```

### 7.2 battle_phase_label

```yaml
widget_id: battle_phase_label
widget_type: CaptionText
defined_in: "widget-atoms.md §CaptionText"

inputs:
  - name: text
    type: LocalizedText
    source: "BattleHudVm.phase_key"
    default: "ui.battle.phase.player"

outputs: []

selection_model:
  type: none
```

### 7.3 battle_end_turn_btn

```yaml
widget_id: battle_end_turn_btn
widget_type: Button, Danger
defined_in: "widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.battle.end.turn"
    default: "End Turn"
  - name: enabled
    type: bool
    source: "BattleHudVm.is_player_turn"
    default: true

outputs:
  - name: clicked
    type: UiCommand::EndTurn
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.4 battle_char_name

```yaml
widget_id: battle_char_name
widget_type: HeadingText
defined_in: "widget-atoms.md §HeadingText"

inputs:
  - name: text
    type: LocalizedText
    source: "BattleHudVm.unit_name"
    default: "Character"

outputs: []

selection_model:
  type: none
```

### 7.5 battle_char_level

```yaml
widget_id: battle_char_level
widget_type: CaptionText
defined_in: "widget-atoms.md §CaptionText"

inputs:
  - name: text
    type: LocalizedText
    source: "BattleHudVm.unit_level"
    default: "Lv.1"

outputs: []

selection_model:
  type: none
```

### 7.6 battle_hp_bar / battle_mp_bar / battle_ap_bar

```yaml
widget_id: battle_hp_bar
widget_type: ProgressBar
defined_in: "widget-atoms.md §ProgressBar"

inputs:
  - name: value
    type: f32
    source: "BattleHudVm.hp"
    default: 100.0
  - name: max_value
    type: f32
    source: "BattleHudVm.max_hp"
    default: 100.0
  - name: variant
    type: ProgressBarVariant
    source: "local — hp variant (红色系)"
    default: hp

outputs: []

selection_model:
  type: none

---

widget_id: battle_mp_bar
widget_type: ProgressBar
defined_in: "widget-atoms.md §ProgressBar"

inputs:
  - name: value
    type: f32
    source: "BattleHudVm.mp"
    default: 50.0
  - name: max_value
    type: f32
    source: "BattleHudVm.max_mp"
    default: 50.0
  - name: variant
    type: ProgressBarVariant
    source: "local — mp variant (蓝色系)"
    default: mp

outputs: []

selection_model:
  type: none

---

widget_id: battle_ap_bar
widget_type: ProgressBar
defined_in: "widget-atoms.md §ProgressBar"

inputs:
  - name: value
    type: f32
    source: "BattleHudVm.ap"
    default: 6.0
  - name: max_value
    type: f32
    source: "BattleHudVm.max_ap"
    default: 6.0
  - name: variant
    type: ProgressBarVariant
    source: "local — ap variant (黄色系)"
    default: ap

outputs: []

selection_model:
  type: none
```

### 7.7 battle_hp_text / battle_mp_text

```yaml
widget_id: battle_hp_text
widget_type: CaptionText
defined_in: "widget-atoms.md §CaptionText"

inputs:
  - name: text
    type: LocalizedText
    source: "format_args(BattleHudVm.hp, BattleHudVm.max_hp)"
    default: "100/100"

outputs: []

selection_model:
  type: none

---

widget_id: battle_mp_text
widget_type: CaptionText
defined_in: "widget-atoms.md §CaptionText"

inputs:
  - name: text
    type: LocalizedText
    source: "format_args(BattleHudVm.mp, BattleHudVm.max_mp)"
    default: "50/50"

outputs: []

selection_model:
  type: none
```

### 7.8 battle_buff_icons_0

```yaml
widget_id: battle_buff_icons_0
widget_type: Icon
defined_in: "widget-atoms.md §Icon"

inputs:
  - name: icon
    type: TextureKey
    source: "BattleHudVm.buff_icons[0]"
    default: ""
  - name: tooltip
    type: LocalizedText
    source: "BattleHudVm.buff_tooltips[0]"
    default: ""

outputs:
  - name: hovered
    type: OverlayAction::ShowTooltip
    payload: "battle_buff_icons_0.tooltip"
    trigger: OnHover(delay: 500ms)
  - name: unhovered
    type: OverlayAction::HideTooltip
    payload: None
    trigger: OnUnhover

selection_model:
  type: none
```

### 7.9 battle_attack_btn

```yaml
widget_id: battle_attack_btn
widget_type: Button, Primary
defined_in: "widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.battle.attack"
    default: "Attack"
  - name: enabled
    type: bool
    source: "BattleHudVm.available_actions.contains('attack')"
    default: true

outputs:
  - name: clicked
    type: UiCommand::CastSkill(slot=0)
    payload: None (default attack)
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.10 battle_skill_btn

```yaml
widget_id: battle_skill_btn
widget_type: Button, Secondary
defined_in: "widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.battle.skill"
    default: "Skill"
  - name: enabled
    type: bool
    source: "BattleHudVm.available_actions.contains('skill')"
    default: true

outputs:
  - name: clicked
    type: UiCommand::OpenOverlay(ScreenType::SkillOverlay)
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.11 battle_defend_btn

```yaml
widget_id: battle_defend_btn
widget_type: Button, Secondary
defined_in: "widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.battle.defend"
    default: "Defend"
  - name: enabled
    type: bool
    source: "BattleHudVm.available_actions.contains('defend')"
    default: true

outputs:
  - name: clicked
    type: UiCommand::Defend
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.12 battle_item_btn

```yaml
widget_id: battle_item_btn
widget_type: Button, Secondary
defined_in: "widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.battle.item"
    default: "Item"
  - name: enabled
    type: bool
    source: "BattleHudVm.available_actions.contains('item')"
    default: true

outputs:
  - name: clicked
    type: UiCommand::OpenOverlay(ScreenType::InventoryOverlay)
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.13 battle_move_btn

```yaml
widget_id: battle_move_btn
widget_type: Button, Secondary
defined_in: "widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.battle.move"
    default: "Move"
  - name: enabled
    type: bool
    source: "BattleHudVm.available_actions.contains('move')"
    default: true

outputs:
  - name: clicked
    type: UiCommand::EnterMoveMode
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

### 7.14 battle_wait_btn

```yaml
widget_id: battle_wait_btn
widget_type: Button, Secondary
defined_in: "widget-atoms.md §Button"

inputs:
  - name: label
    type: LocalizedText
    source: "ui.battle.wait"
    default: "Wait"
  - name: enabled
    type: bool
    source: "BattleHudVm.available_actions.contains('wait')"
    default: true

outputs:
  - name: clicked
    type: UiCommand::Wait
    payload: None
    trigger: OnLeftClick

selection_model:
  type: none
```

---

## 8. State Mapping (Per-Region)

> 每个 region 独立的状态。必须定义 Loading / Empty / Normal / Error 四种状态。
> AI 实现时必须为每种状态提供对应的 UI 展示。

### 8.1 battle_top_bar

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 回合信息和阶段标签为静态数据，无异步加载 | — | — |
| **Empty** | N/A — 回合始终从 1 开始，阶段始终存在，无空状态 | — | — |
| **Normal** | TurnIndicator + PhaseLabel + EndTurnBtn 正常显示，EndTurnBtn 在非玩家回合时 disabled | ViewModel 数据就绪 | Fade(0.3s) 入场动画 |
| **Error** | N/A — 顶栏无数据加载，无错误状态 | — | — |

### 8.2 battle_battle_area

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | 全区域 Spinner + "ui.battle.area.loading" 文本（LocalizationKey），地图数据正在从 Tactical Domain 加载 | OnEnter 后地图数据未就绪 | Spinner 旋转动画 |
| **Empty** | 空地图网格 + "ui.battle.area.empty" 文本（LocalizationKey），地图加载完成但无任何单位或地形数据 | 地图加载完成但无数据 | Loading → Empty 淡入 |
| **Normal** | 6 层地图全渲染 + 单位部署 + Minimap 显示，完整的战斗场景 | 地图数据加载完成且有有效数据 | Empty → Normal 地图 fade-in，单位入场动画 |
| **Error** | 错误提示面板 + 重试按钮 + "ui.battle.area.error" 文本，含重试机制 | 地图数据加载失败 | Loading → Error 过渡，保留错误状态直到用户重试 |

### 8.3 battle_char_panel

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | 骨架屏面板 — 头像灰色占位块 + 名称/等级灰色条 + 3 条灰色进度条 | BattleHudVm 数据未就绪（无选中单位） | 骨架屏脉冲动画 |
| **Empty** | "ui.battle.char_panel.empty" 提示文本（LocalizationKey）+ 空面板背景 | 战斗无单位（极边缘情况，战斗初始化后不应出现） | Loading → Empty 淡入 |
| **Normal** | 完整角色面板：头像 + 名称/等级 + HP/MP/AP 条 + Buff 图标 | ViewModel 数据就绪且有选中单位 | Empty → Normal 数据填充过渡 |
| **Error** | N/A — 角色面板数据与顶栏同源，无独立错误状态 | — | — |

### 8.4 battle_action_menu

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | N/A — 按钮标签为静态 LocalizationKey，无异步加载 | — | — |
| **Empty** | N/A — 按钮始终存在，无空状态 | — | — |
| **Normal** | 6 个行动按钮正常显示，部分按钮根据 available_actions 可能 disabled | ViewModel 数据就绪 | Fade(0.3s) 入场动画 |
| **Error** | N/A — 按钮无错误状态 | — | — |

---

## 9. Focus Navigation

> Tab 导航路径。按 Tab 键的顺序就是导航路径的顺序。

```yaml
focus_path:
  - battle_end_turn_btn          # Tab 1 — 默认焦点
  - battle_attack_btn            # Tab 2
  - battle_skill_btn             # Tab 3
  - battle_defend_btn            # Tab 4
  - battle_item_btn              # Tab 5
  - battle_move_btn              # Tab 6
  - battle_wait_btn              # Tab 7

special_keys:
  Escape: "取消当前选择/关闭浮层（SkillOverlay/InventoryOverlay），焦点回到对应按钮"
  Enter: "激活当前焦点按钮（等同于点击）"
  ArrowLeft: "焦点左移（battle_attack_btn ← battle_skill_btn ← ...）"
  ArrowRight: "焦点右移（battle_attack_btn → battle_skill_btn → ...）"
  Tab: "按 focus_path 顺序前进"
  Shift+Tab: "按 focus_path 逆序后退"

focus_trap: true          # true = 焦点锁定在该 Screen 内，Tab 循环
```

### 9.1 默认焦点

进入 BattleScreen 时，默认焦点落在 `battle_end_turn_btn`。理由：玩家首先查看回合信息确认是否己方回合，End Turn 是回合操作的关键入口。

---

## 10. Interaction Zones

> 每个可交互区域的行为定义。Click / Hover / Drag / Drop。

### 10.1 battle_end_turn_btn

```yaml
zone_id: battle_end_turn_btn
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::EndTurn"
    cursor: Pointer
    conditions:
      - "BattleHudVm.is_player_turn == true (else: disabled 不可点击)"

  - type: hover
    enter_effect: "按钮变为 Danger Hover 样式（高亮红色边框/背景）"
    leave_effect: "恢复 Danger 默认样式"
    delay: 0ms
```

### 10.2 battle_attack_btn

```yaml
zone_id: battle_attack_btn
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::CastSkill(slot=0) — 默认普通攻击，随后进入目标选择模式"
    cursor: Pointer
    conditions:
      - "BattleHudVm.available_actions.contains('attack') (else: disabled)"

  - type: hover
    enter_effect: "按钮变为 Primary Hover 样式"
    leave_effect: "恢复 Primary 默认样式"
    delay: 0ms
```

### 10.3 battle_skill_btn

```yaml
zone_id: battle_skill_btn
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::OpenOverlay(ScreenType::SkillOverlay)，弹出技能选择浮层"
    cursor: Pointer
    conditions:
      - "BattleHudVm.available_actions.contains('skill') (else: disabled)"

  - type: hover
    enter_effect: "按钮变为 Secondary Hover 样式"
    leave_effect: "恢复 Secondary 默认样式"
    delay: 0ms
```

### 10.4 battle_defend_btn

```yaml
zone_id: battle_defend_btn
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::Defend，单位进入防御状态，本次行动结束"
    cursor: Pointer
    conditions:
      - "BattleHudVm.available_actions.contains('defend') (else: disabled)"

  - type: hover
    enter_effect: "按钮变为 Secondary Hover 样式"
    leave_effect: "恢复 Secondary 默认样式"
    delay: 0ms
```

### 10.5 battle_item_btn

```yaml
zone_id: battle_item_btn
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::OpenOverlay(ScreenType::InventoryOverlay)，弹出物品选择浮层"
    cursor: Pointer
    conditions:
      - "BattleHudVm.available_actions.contains('item') (else: disabled)"

  - type: hover
    enter_effect: "按钮变为 Secondary Hover 样式"
    leave_effect: "恢复 Secondary 默认样式"
    delay: 0ms
```

### 10.6 battle_move_btn

```yaml
zone_id: battle_move_btn
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::EnterMoveMode，Tactical Domain 高亮可移动范围，等待玩家点击地图选择目的地"
    cursor: Pointer
    conditions:
      - "BattleHudVm.available_actions.contains('move') (else: disabled)"

  - type: hover
    enter_effect: "按钮变为 Secondary Hover 样式"
    leave_effect: "恢复 Secondary 默认样式"
    delay: 0ms
```

### 10.7 battle_wait_btn

```yaml
zone_id: battle_wait_btn
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::Wait，结束当前单位行动，单位不再执行任何动作"
    cursor: Pointer
    conditions:
      - "BattleHudVm.available_actions.contains('wait') (else: disabled)"

  - type: hover
    enter_effect: "按钮变为 Secondary Hover 样式"
    leave_effect: "恢复 Secondary 默认样式"
    delay: 0ms
```

### 10.8 battle_buff_icons_n

```yaml
zone_id: battle_buff_icons_0
interactions:
  - type: hover
    enter_effect: "显示 TooltipOverlay，内容为 Buff 名称 + 描述（BattleHudVm.buff_tooltips[n]）"
    leave_effect: "隐藏 TooltipOverlay"
    delay: 500ms
```

---

## 11. Overlay Definition

> Overlay 列表 + Z-Layer。Overlay 使用独立层级，不嵌套在任何 Screen 之下。

| Overlay | 用途 | Z-Layer | 类型 | 触发条件 |
|---------|------|---------|------|---------|
| SkillOverlay | 展示当前单位可用技能列表，点击选中技能后执行 CastSkill | 2 | Popup | 点击 battle_skill_btn |
| InventoryOverlay | 展示背包中可在战斗使用的物品列表，点击选中后执行 UseItem | 2 | Popup | 点击 battle_item_btn |
| DamageTextOverlay | 战斗伤害/治疗数字弹出效果，从受击单位位置弹出 | 3 | Popup (CueType) | Domain 事件 DamageApplied |
| TooltipOverlay | Buff 图标 hover 时显示 Buff 名称和描述 | 1 | Tooltip | Hover on battle_buff_icons_n (500ms delay) |
| NotificationOverlay | 战斗中的系统通知（Buff 生效/过期、升级等） | 4 | Notification (toast) | Domain 事件 EffectApplied / LevelUp |
| ModalOverlay | 确认弹窗（如确认技能目标、投降确认等） | 5 | Modal | 特定交互条件 |
| VictoryOverlay | 战斗胜利时的结算覆盖层（经验、掉落物品、等级提升） | 6 | Modal | Domain 事件 BattleEnded(Victory) |
| DefeatOverlay | 战斗失败时的覆盖层（重试/返回标题） | 6 | Modal | Domain 事件 BattleEnded(Defeat) |

### 11.1 Z-Layer 分配

| Z-Layer | 用途 | 包含 |
|---------|------|------|
| 0 | Screen 主界面层 | `battle_screen_root` 及所有子 region |
| 1 | Tooltip 层 | TooltipOverlay (Buff 详情) |
| 2 | Popup 层 | SkillOverlay, InventoryOverlay, DamageTextOverlay |
| 3 | Notification 层 | NotificationOverlay (Battle Log toasts) |
| 4 | Modal 层 | ModalOverlay (确认弹窗) |
| 5 | Result 层 | VictoryOverlay, DefeatOverlay |
| 9 | Debug 层 | DebugOverlay (FPS/日志) |

### 11.2 Overlay 内部结构

#### SkillOverlay

```
SkillOverlay                                              [skill_overlay_container]
├── SkillList                                             [skill_overlay_list: Container]
│   ├── SkillSlot1                                        [skill_overlay_slot_0: SkillSlot — Organism]
│   ├── SkillSlot2                                        [skill_overlay_slot_1: SkillSlot — Organism]
│   ├── SkillSlot3                                        [skill_overlay_slot_2: SkillSlot — Organism]
│   └── ...                                               [skill_overlay_slot_n: SkillSlot — Organism]
└── CloseButton                                           [skill_overlay_close: IconButton]
```

#### DamageTextOverlay

```
DamageTextOverlay                                         [damage_overlay: Container]
├── DamageText1                                           [damage_text_0: FloatingText — CueType::Popup]
├── DamageText2                                           [damage_text_1: FloatingText — CueType::Popup]
└── ...                                                   [damage_text_n: FloatingText — CueType::Popup]
```

#### NotificationOverlay

```
NotificationOverlay                                       [notification_overlay: Container]
├── NotificationToast1                                    [notification_toast_0: Toast — CueType::Toast]
├── NotificationToast2                                    [notification_toast_1: Toast — CueType::Toast]
└── ...                                                   [notification_toast_n: Toast — CueType::Toast]
```

### 11.3 Overlay 生命周期

| Overlay | OnOpen | OnClose | 依赖 |
|---------|--------|---------|------|
| SkillOverlay | 创建 SkillList 浮层，从 BattleHudVm.skills 填充技能槽位列表，焦点切换至第一个技能槽 | 销毁浮层，焦点回到 battle_skill_btn | 无 |
| InventoryOverlay | 创建物品列表浮层，从 BattleInventoryVm 填充可用物品，焦点切换至第一个物品 | 销毁浮层，焦点回到 battle_item_btn | InventoryProjection |
| DamageTextOverlay | 添加 CueType::Popup 实体，从受击单位位置向上飘浮 | 动画完成后自动销毁 | 无 |
| TooltipOverlay | 创建 TooltipPanel，填充 buff 名称 + 描述文本 | hover 离开后销毁 | 无 |
| NotificationOverlay | 添加 CueType::Toast 实体，自动定时消失 | 定时完成/点击关闭 | 无 |
| ModalOverlay | 创建 Modal 弹窗 + 半透明遮罩，锁定下层交互 | 确认/取消后销毁 | 无 |
| VictoryOverlay | 创建全屏结算覆盖层，显示经验/掉落/等级提升动画 | 玩家点击"继续"后销毁 | LootProjection, ProgressionProjection |
| DefeatOverlay | 创建全屏失败覆盖层，显示"重试"/"返回标题"按钮 | 玩家选择操作后销毁 | 无 |

---

## 12. Lifecycle

> Screen 的完整生命周期行为。遵守 `screen-lifecycle.md` 定义的状态机。

| 阶段 | 行为 | 触发条件 | 清理 |
|------|------|---------|------|
| **OnEnter** | `spawn_battle_screen()` — 通过 `spawn_panel` 工厂生成根容器（禁止原始 commands.spawn），插入 BattleScreen + Name 组件，按 Flexbox 布局构造完整 UI 树：TopBar / BattleArea / CharPanel / ActionMenu。从 BattleHudVm 读取初始数据填充各 Widget | `GameState::Combat` 状态进入 | — |
| **OnReady** | 注册 BattleProjection Observer（监听 TurnStarted/TurnEnded/DamageApplied/EffectApplied/BattleEnded 等核心战斗事件）；注册 ZoneVisibility Projection 驱动区域可见性；Tactical Domain 开始加载地图数据 | OnEnter 完成，UI 树就绪 | — |
| **Active** | 等待玩家行动选择（CastSkill/SelectTarget/EndTurn/Defend/Wait/EnterMoveMode），通过 UiCommand 下发至 Domain；BattleHudVm dirty 时刷新 HP/MP/AP/Turn/Phase/Buff 显示；DamageTextOverlay 和 NotificationOverlay 响应 Domain 事件自动弹出 | OnReady 完成 | — |
| **OnExit** | `despawn_battle_screen()` — 清理所有标记了 `With<BattleScreen>` 的实体；注销所有 BattleProjection Observer | `GameState::Combat` 状态退出 | 清理标记: `With<BattleScreen>` |

### 12.1 生命周期事件处理

```yaml
on_enter:
  - action: "spawn_ui_tree"
    spawner: "spawn_battle_screen()"
    description: "通过 spawn_panel 工厂生成 BattleScreen 根容器（禁止原始 commands.spawn），构造完整 UI 树：TopBar(TurnIndicator + PhaseLabel + EndTurnBtn) + BattleArea(Minimap) + CharPanel(Avatar + Name + Level + HP/MP/AP bars + BuffIcons) + ActionMenu(6 action buttons)"
  - action: "bind_viewmodel"
    source: "BattleHudVm"
    description: "将所有 UiBinding 指向 BattleHudVm 字段，通过 Dirty<BattleHudVm> 实现 Widget 刷新"

on_ready:
  - action: "register_observer"
    target: "BattleProjection::on_turn_started"
    handler: "TurnStarted 事件 → 更新 BattleHudVm.turn_number, phase_key, is_player_turn, current_unit_id，标记 Dirty<BattleHudVm>"
  - action: "register_observer"
    target: "BattleProjection::on_turn_ended"
    handler: "TurnEnded 事件 → 更新 BattleHudVm.phase_key, is_player_turn，标记 Dirty<BattleHudVm>"
  - action: "register_observer"
    target: "BattleProjection::on_damage_applied"
    handler: "DamageApplied 事件 → 更新 BattleHudVm.hp，标记 Dirty<BattleHudVm>，触发 DamageTextOverlay 弹出"
  - action: "register_observer"
    target: "BattleProjection::on_effect_applied"
    handler: "EffectApplied 事件 → 更新 BattleHudVm.buff_icons，标记 Dirty<BattleHudVm>，触发 NotificationOverlay"
  - action: "register_observer"
    target: "BattleProjection::on_battle_ended"
    handler: "BattleEnded 事件 → 根据结果弹出 VictoryOverlay / DefeatOverlay"
  - action: "register_observer"
    target: "ZoneVisibilityProjection"
    handler: "更新区域可见性状态，替代直接查询 Res<State<BattlePhase>>"

active:
  - trigger: "battle_end_turn_btn.clicked"
    action: "构建 UiCommand::EndTurn 下发至 Domain"
    scope: "battle_top_bar → Domain"
  - trigger: "battle_attack_btn.clicked"
    action: "构建 UiCommand::CastSkill(slot=0) 下发至 Domain"
    scope: "battle_action_menu → Domain"
  - trigger: "battle_skill_btn.clicked"
    action: "弹出 SkillOverlay（Popup 层），暂停下层交互"
    scope: "battle_action_menu → Overlay"
  - trigger: "battle_item_btn.clicked"
    action: "弹出 InventoryOverlay（Popup 层），展示可用物品列表"
    scope: "battle_action_menu → Overlay"
  - trigger: "battle_move_btn.clicked"
    action: "构建 UiCommand::EnterMoveMode 下发至 Tactical Domain"
    scope: "battle_action_menu → Domain"
  - trigger: "battle_defend_btn.clicked"
    action: "构建 UiCommand::Defend 下发至 Domain"
    scope: "battle_action_menu → Domain"
  - trigger: "battle_wait_btn.clicked"
    action: "构建 UiCommand::Wait 下发至 Domain"
    scope: "battle_action_menu → Domain"
  - trigger: "BattleHudVm.dirty"
    action: "刷新所有 UiBinding 绑定的 Widget 数据（HP/MP/AP/Turn/Phase/Buff 等）"
    scope: "全部 Widget"
  - trigger: "MapData.loaded"
    action: "BattleArea Loading → Normal 状态转换，渲染地图"
    scope: "battle_battle_area"
  - trigger: "MapData.error"
    action: "BattleArea Loading → Error 状态转换，显示错误提示 + 重试按钮"
    scope: "battle_battle_area"

on_exit:
  - action: "unregister_observer"
    target: "BattleProjection::on_turn_started"
  - action: "unregister_observer"
    target: "BattleProjection::on_turn_ended"
  - action: "unregister_observer"
    target: "BattleProjection::on_damage_applied"
  - action: "unregister_observer"
    target: "BattleProjection::on_effect_applied"
  - action: "unregister_observer"
    target: "BattleProjection::on_battle_ended"
  - action: "unregister_observer"
    target: "ZoneVisibilityProjection"
  - action: "despawn_ui_tree"
    query: "With<BattleScreen>"
    description: "清理所有标记了 BattleScreen 组件的实体（包括未自动销毁的 Overlay 实体）"
```

---

## 13. Data Ownership

> Owns / Uses 分离。明确每个 UiStore 字段的归属。

### 13.1 ViewModel 映射

| ViewModel | 字段 | 归属 (Owns/Uses) | 更新频率 | Projection 源 |
|-----------|------|-----------------|---------|--------------|
| `BattleHudVm` | `turn_number: u32` | Owns — BattleScreen 独享 | 事件触发 (TurnStarted) | `BattleProjection::on_turn_started()` |
| `BattleHudVm` | `phase_key: &'static str` | Owns | 事件触发 (TurnStarted/TurnEnded) | `BattleProjection::on_turn_started()` / `on_turn_ended()` |
| `BattleHudVm` | `is_player_turn: bool` | Owns | 事件触发 (TurnStarted/TurnEnded) | `BattleProjection::on_turn_started()` / `on_turn_ended()` |
| `BattleHudVm` | `current_unit_id: Entity` | Owns | 事件触发 (TurnStarted) | `BattleProjection::on_turn_started()` |
| `BattleHudVm` | `unit_name: &'static str` | Owns | 事件触发 (TurnStarted) | `BattleProjection::on_turn_started()` |
| `BattleHudVm` | `unit_level: u32` | Owns | 事件触发 (TurnStarted) | `BattleProjection::on_turn_started()` |
| `BattleHudVm` | `hp: f32` | Owns | 事件触发 (DamageApplied/HealApplied) | `BattleProjection::on_damage_applied()` |
| `BattleHudVm` | `max_hp: f32` | Owns | 事件触发 (StatChanged) | `BattleProjection::on_stat_changed()` |
| `BattleHudVm` | `mp: f32` | Owns | 事件触发 (MpChanged) | `BattleProjection::on_mp_changed()` |
| `BattleHudVm` | `max_mp: f32` | Owns | 事件触发 (StatChanged) | `BattleProjection::on_stat_changed()` |
| `BattleHudVm` | `ap: f32` | Owns | 事件触发 (ApChanged) | `BattleProjection::on_ap_changed()` |
| `BattleHudVm` | `max_ap: f32` | Owns | 事件触发 (StatChanged) | `BattleProjection::on_stat_changed()` |
| `BattleHudVm` | `buff_icons: [TextureKey; 4]` | Owns | 事件触发 (EffectApplied/EffectExpired) | `BattleProjection::on_effect_applied()` |
| `BattleHudVm` | `buff_tooltips: [&str; 4]` | Owns | 事件触发 (EffectApplied/EffectExpired) | `BattleProjection::on_effect_applied()` |
| `BattleHudVm` | `available_actions: Vec<ActionType>` | Owns | 事件触发 (TurnStarted/ActionExecuted) | `BattleProjection::on_turn_started()` |
| `BattleHudVm` | `visible_zones: Vec<String>` | Owns | 事件触发 (PhaseChanged) — 替代直接查询 Res<State<BattlePhase>> | `ZoneVisibilityProjection` |

### 13.2 数据流

```
Domain Event (TurnStarted/TurnEnded/DamageApplied)
    ↓
BattleProjection (pure function — no direct domain queries)
    ↓
BattleHudVm (UiStore field setter)
    ↓
mark_dirty::<BattleHudVm>()
    ↓
Widget refresh system reads BattleHudVm → updates UiBinding targets (HP/MP/AP/Turn/Phase/Buff)
    ↓
用户交互 → UiCommand → UiLayer → Domain (UiCommand::EndTurn / CastSkill / ...)
```

**P0 合规检查**:
- `TurnQueue` domain query 已移除: `current_unit_id` 通过 TurnStarted 事件的参数设置到 BattleHudVm，UI 不再直接查询 TurnQueue
- `BattlePhase` domain query 已移除: 区域可见性通过 `visible_zones` ViewModel 字段 + ZoneVisibilityProjection 驱动，UI 不再直接查询 `Res<State<BattlePhase>>`

---

## 14. Layout Intent

> 每个关键尺寸的**理由说明**。为什么选这个尺寸而不是别的？
> 记录意图是为了防止未来修改时随意改尺寸。

### 14.1 固定尺寸意图

| widget_id | 属性 | 值 | 意图 | shrink |
|-----------|------|----|------|--------|
| `battle_top_bar` | height | 64px | "顶栏高度 64px 为回合信息和阶段标签提供充足显示空间，同时不会过度挤压战斗地图区域" | none |
| `battle_end_turn_btn` | width | 120px | "按钮宽度 120px 容纳 'End Turn' / '结束回合' 等本地化文本（中英文均在 12 字符以内），单行显示不折行" | none |
| `battle_end_turn_btn` | height | 40px | "按钮高度 >= 40px 满足最小触摸目标要求" | none |
| `battle_minimap` | width | 160px | "小地图宽度 160px 在 1080p 分辨率下占战斗区域约 12%，足够显示战场概览，不会过度遮挡地图" | none |
| `battle_minimap` | height | 120px | "小地图高度 120px，长宽比约 4:3，与常见 SRPG 战场比例一致" | none |
| `battle_char_avatar` | width | 100px | "头像 100x100 在 1080p 下约为视口高度的 9%，大小足以展示角色特征，不会过度占据 CharPanel" | none |
| `battle_char_avatar` | height | 100px | "头像 100x100 正方形，与宽度一致" | none |
| `battle_char_panel` | height | 140px | "CharPanel 140px 高度 = 头像 100px + 上下 padding 40px，容纳头像 + 4 行状态条 + Buff 图标，信息完整且不过度" | none |
| `battle_hp_bar` | width | 200px | "HP 条 200px 宽度确保在 1080p 下精细显示血量变化（每像素约 0.5%），同时保持与面板宽度的比例协调" | low |
| `battle_hp_bar` | height | 16px | "HP 条 16px 高度确保颜色填充变化肉眼可辨，MP 条同高保持视觉统一" | none |
| `battle_ap_bar` | height | 12px | "AP 条 12px 高度略低于 HP/MP 条（16px），区分视觉层级，表示 AP 与 HP/MP 的数值性质不同" | none |
| `battle_action_menu` | height | 80px | "行动菜单 80px 高度 = 按钮 48px + 上下 padding 32px，为 6 个水平排列按钮提供充足空间，同时不会过度挤压战斗区域" | none |
| `battle_attack_btn` | width | 120px | "按钮 120px 宽度确保 'Attack' / '普通攻击' 等本地化文本单行显示，与 EndTurn 按钮同宽保证视觉节奏感" | none |
| `battle_attack_btn` | height | 48px | "按钮 48px 高度 >= 40px 触摸友好标准，略高于 EndTurn 的 40px 以突出主行动按钮" | none |
| `battle_buff_icons_n` | width | 24px | "Buff 图标 24x24 在 1080p 下约为视口高度的 2%，大小足够展示 Buff 图标纹理，不过度占据面板空间" | none |
| `battle_buff_icons_n` | height | 24px | "与宽度一致，正方形图标" | none |

### 14.2 弹性尺寸意图

| widget_id | flex_grow | 理由 |
|-----------|-----------|------|
| `battle_battle_area` | 1 | "战斗地图区域是 BattleScreen 的核心，弹性填充所有剩余空间——地图越大，玩家对战场的感知越清晰" |
| `battle_top_bar` | 0 | "顶栏固定 64px，不需要弹性增长" |
| `battle_char_panel` | 0 | "角色面板固定 140px，不需要弹性增长" |
| `battle_action_menu` | 0 | "行动菜单固定 80px，不需要弹性增长" |

### 14.3 通用约束

```yaml
global:
  min_interactive_height: 40px   # 可交互元素最小高度 (触摸友好)
  min_interactive_width: 40px    # 可交互元素最小宽度 (触摸友好)
  standard_padding: 8px          # 标准内边距
  standard_gap: 4px              # 标准间距 (Flexbox gap)
```

### 14.4 布局策略理由

| 决策 | 理由 |
|------|------|
| BattleArea flex_grow:1 | "战斗地图是战斗体验的核心区域，应占满所有弹性空间。固定顶栏/面板/菜单高度后，剩余空间全部分配给地图" |
| CharPanel 非浮层 | "相比浮层方案，固定底部面板提供持续可见的角色状态，减少玩家的认知负担（不需要主动打开查看 HP/MP）" |
| ActionMenu 底部而非右侧 | "底部菜单符合 SRPG 经典布局（如 Fire Emblem、Final Fantasy Tactics），玩家关注点从上到下：回合信息 → 地图 → 角色 → 操作" |
| ActionMenu 6 按钮而非更少 | "6 个行动按钮覆盖所有核心操作（攻击/技能/防御/物品/移动/待机），无需子菜单即可完成 90% 的常规操作" |
| CharPanel 320px 等效宽度 | "角色面板宽度不固定为 320px（screen.md 原设计），而是屏幕底部全宽 100%，充分利用宽屏显示器横向空间，头像 + 状态条水平排列效率更高" |

---

## 15. Scroll & Overflow Policy

> 每个可能产生滚动的区域必须定义 policy。
> Overflow 策略: clip — 裁剪 / ellipsis — 省略号 / scroll — 可滚动 / visible — 可见溢出

### 15.1 滚动区域

| widget_id | 方向 | Scroll Policy | Overflow Policy | 理由 |
|-----------|------|--------------|---------------|------|
| `battle_battle_area` | both | scroll (by Tactical Camera) | clip | "战斗地图由 Tactical Domain 的摄像机系统管理滚动和缩放，BattleScreen 仅提供容器，不直接处理滚动" |
| `battle_top_bar` | horizontal | none | clip | "顶栏内容（回合信息 + 阶段 + 按钮）在 1280px 宽度内完全可见，无需滚动" |
| `battle_char_panel` | vertical | none | clip | "角色面板固定 140px 高度，内容不超出，无滚动需求" |
| `battle_action_menu` | horizontal | none | clip | "6 个按钮 6x120=720px 在 1280px+ 宽度内完全可见（有间距时 ~800px），无需滚动。响应式策略宽度 < 1024px 时隐藏为收缩按钮" |
| `battle_minimap` | none | none | clip | "小地图固定 160x120，不溢出" |

### 15.2 文本溢出

| widget_id | max_lines | overflow | 多语言风险 |
|-----------|-----------|----------|-----------|
| `battle_turn_indicator` | 1 | clip | "'Turn: 99' (8 字符) 与 '回合: 99' (6 字符) 长度相近，无风险" |
| `battle_phase_label` | 1 | ellipsis | "'Player Turn' (11 字符) 与 '玩家回合' (4 字符) 中英文差异大但均短于 200px 容器，无风险" |
| `battle_char_name` | 1 | ellipsis | "角色名在不同语言下差异大（如英文 3-15 字符 vs 中文 2-6 字符），容器 auto 宽度适配，但极端角色名可能溢出——使用 ellipsis 截断" |
| `battle_char_level` | 1 | clip | "'Lv.99' (5 字符) / '等级 99' (5 字符) 极短，无风险" |
| `battle_hp_text` | 1 | clip | "'9999/9999' (9 字符) 与 '9999/9999' 一致，无风险" |
| `battle_mp_text` | 1 | clip | "同 HP text，无风险" |
| `battle_attack_btn` | 1 | ellipsis | "'Attack' (6 字符) / '普通攻击' (4 字符) 在 120px 内容器内安全，但极长本地化变体（如德语 'Angreifen' 8 字符）需 ellipsis 兜底" |
| `battle_skill_btn` | 1 | ellipsis | "同 attack 按钮，120px 安全" |
| `battle_defend_btn` | 1 | ellipsis | "同 attack 按钮，120px 安全" |
| `battle_wait_btn` | 1 | ellipsis | "同 attack 按钮，120px 安全" |

---

## 16. Event Contract

> UI -> Domain 事件 + Domain -> UI 事件的完整契约。

### 16.1 UI -> Domain（通过 UiCommand 传递）

```yaml
EndTurn:
  trigger_widget: "battle_top_bar → battle_end_turn_btn → click"
  data: {}
  conditions:
    - "BattleHudVm.is_player_turn == true"
  emits: UiCommand::EndTurn
  domain_event: "CombatEvent::TurnEnded | TurnQueue::advance()"

CastSkill:
  trigger_widget: "battle_action_menu → battle_attack_btn → click"
  data:
    slot: 0 (default attack)
    target_id: "from SelectTarget interaction on battle_battle_area"
  conditions:
    - "BattleHudVm.available_actions.contains('attack')"
    - "AP >= skill.cost"
  emits: UiCommand::CastSkill { slot: u8, target: Entity }
  domain_event: "CombatEvent::SkillUsed | EffectPipeline::execute()"

SelectTarget:
  trigger_widget: "battle_battle_area → map click on unit"
  data:
    target_id: Entity
  conditions:
    - "target is valid (within range, enemy, alive)"
    - "skill selected or attack mode active"
  emits: UiCommand::SelectTarget { target: Entity }
  domain_event: "CombatEvent::TargetSelected | EffectPipeline::resolve_target()"

Defend:
  trigger_widget: "battle_action_menu → battle_defend_btn → click"
  data: {}
  conditions:
    - "BattleHudVm.available_actions.contains('defend')"
  emits: UiCommand::Defend
  domain_event: "CombatEvent::DefendStarted | CombatEvent::ActionExecuted"

Wait:
  trigger_widget: "battle_action_menu → battle_wait_btn → click"
  data: {}
  conditions:
    - "BattleHudVm.available_actions.contains('wait')"
  emits: UiCommand::Wait
  domain_event: "CombatEvent::ActionExecuted | TurnQueue::advance()"

EnterMoveMode:
  trigger_widget: "battle_action_menu → battle_move_btn → click"
  data: {}
  conditions:
    - "BattleHudVm.available_actions.contains('move')"
  emits: UiCommand::EnterMoveMode
  domain_event: "TacticalEvent::MoveRangeHighlighted | TacticalEvent::MovementConfirmed"

OpenSkillOverlay:
  trigger_widget: "battle_action_menu → battle_skill_btn → click"
  data: {}
  conditions:
    - "BattleHudVm.available_actions.contains('skill')"
  emits: UiCommand::OpenOverlay(ScreenType::SkillOverlay)
  domain_event: "None — 纯 UI 导航，ScreenStack 处理"

OpenInventoryOverlay:
  trigger_widget: "battle_action_menu → battle_item_btn → click"
  data: {}
  conditions:
    - "BattleHudVm.available_actions.contains('item')"
  emits: UiCommand::OpenOverlay(ScreenType::InventoryOverlay)
  domain_event: "None — 纯 UI 导航，ScreenStack 处理"
```

### 16.2 Domain -> UI（通过 Projection 消费）

```yaml
TurnStarted:
  source: "Domain Event (Combat Domain)"
  projection: "BattleProjection.on_turn_started()"
  vm_update:
    - BattleHudVm.turn_number ← event.turn_number
    - BattleHudVm.phase_key ← event.phase_key  (LocalizationKey: "ui.battle.phase.player"/"ui.battle.phase.enemy")
    - BattleHudVm.is_player_turn ← event.is_player_turn
    - BattleHudVm.current_unit_id ← event.unit_id
    - BattleHudVm.unit_name ← event.unit_name
    - BattleHudVm.unit_level ← event.unit_level
    - BattleHudVm.hp ← event.hp
    - BattleHudVm.max_hp ← event.max_hp
    - BattleHudVm.mp ← event.mp
    - BattleHudVm.max_mp ← event.max_mp
    - BattleHudVm.ap ← event.ap
    - BattleHudVm.max_ap ← event.max_ap
    - BattleHudVm.available_actions ← event.available_actions
  side_effect:
    - "mark_dirty::<BattleHudVm>()"
    - "Clear DamageTextOverlay"
    - "battle_battle_area → Normal state (if was Loading)"

TurnEnded:
  source: "Domain Event (Combat Domain)"
  projection: "BattleProjection.on_turn_ended()"
  vm_update:
    - BattleHudVm.phase_key ← "ui.battle.phase.enemy" / "ui.battle.phase.player"
    - BattleHudVm.is_player_turn ← false / true
  side_effect:
    - "mark_dirty::<BattleHudVm>()"
    - "Disable EndTurnBtn during enemy phase"

DamageApplied:
  source: "Domain Event (Combat Domain)"
  projection: "BattleProjection.on_damage_applied()"
  vm_update:
    - BattleHudVm.hp ← event.new_hp
  side_effect:
    - "mark_dirty::<BattleHudVm>()"
    - "Spawn DamageTextOverlay (CueType::Popup) at event.target_position showing event.damage_value"
    - "If event.is_critical: extra visual cue (larger text, shake effect)"

HealApplied:
  source: "Domain Event (Combat Domain)"
  projection: "BattleProjection.on_heal_applied()"
  vm_update:
    - BattleHudVm.hp ← event.new_hp
  side_effect:
    - "mark_dirty::<BattleHudVm>()"
    - "Spawn DamageTextOverlay (green) at event.target_position showing event.heal_value"

EffectApplied:
  source: "Domain Event (Combat Domain | Buff Module)"
  projection: "BattleProjection.on_effect_applied()"
  vm_update:
    - BattleHudVm.buff_icons ← update_slot(event.slot, event.icon)
    - BattleHudVm.buff_tooltips ← update_slot(event.slot, event.tooltip_key)
  side_effect:
    - "mark_dirty::<BattleHudVm>()"
    - "Spawn NotificationOverlay: 'ui.battle.buff.applied' with buff_name parameter"

EffectExpired:
  source: "Domain Event (Combat Domain | Buff Module)"
  projection: "BattleProjection.on_effect_expired()"
  vm_update:
    - BattleHudVm.buff_icons ← clear_slot(event.slot)
    - BattleHudVm.buff_tooltips ← clear_slot(event.slot)
  side_effect:
    - "mark_dirty::<BattleHudVm>()"
    - "Spawn NotificationOverlay: 'ui.battle.buff.expired' with buff_name parameter"

BattleEnded:
  source: "Domain Event (Combat Domain)"
  projection: "BattleProjection.on_battle_ended()"
  vm_update:
    - (full ViewModel reset via on_turn_started not applicable — battle is over)
  side_effect:
    - "If result == Victory: Spawn VictoryOverlay (result data: exp_gained, loot_dropped, level_ups)"
    - "If result == Defeat: Spawn DefeatOverlay (retry / return to title)"
```

---

## 17. Screen Metrics

> 复杂度基线。所有数值初始创建时手动填写，后续 CI 阶段自动校验。

| Metric | 值 | P0/P1 | 说明 |
|--------|-----|-------|------|
| `widget_count` | 30 | P1 | Widget 实例总数（root + top_bar + 3 top widgets + battle_area + minimap + char_panel + avatar + name + level + hp_bar + hp_text + mp_bar + mp_text + ap_bar + buff_area + 4 buff_icons + action_menu + 6 action_btns = 26 existing + 2 new = 28, plus 2 internal containers char_stats/name_line → 30） |
| `container_count` | 8 | P1 | 纯容器节点数（top_bar + battle_area + char_panel + char_stats + name_line + buff_area + action_menu + screen_root） |
| `interactive_count` | 8 | P1 | 可交互 Widget 数（end_turn_btn + attack_btn + skill_btn + defend_btn + item_btn + move_btn + wait_btn + buff_icons hover → 8+ interactive zones） |
| `overlay_count` | 8 | P1 | 关联的 Overlay 数（SkillOverlay + InventoryOverlay + DamageTextOverlay + TooltipOverlay + NotificationOverlay + ModalOverlay + VictoryOverlay + DefeatOverlay） |
| `max_depth` | 5 | P1 | root → char_panel → char_stats → name_line → char_name 的层级数 |
| `max_children` | 6 | P1 | battle_action_menu 有 6 个子节点（6 个按钮） |

### 17.1 Budget 检查

| 规则 | 阈值 | 当前值 | 状态 |
|------|------|--------|------|
| max_depth ≤ 6 | 6 | 5 | ✅ |
| max_children ≤ 20 | 20 | 6 | ✅ |
| interactive_count / widget_count ≥ 0.2 | 20% | 26.7% (8/30) | ✅ |

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
| D07 | Region Responsibility 已定义 (每 region 3-8 条) | [ ] | |
| D08 | Widget Contract 已定义 (Inputs/Outputs/Selection Model) | [ ] | |
| D09 | State Mapping 完整 (每个 region 的 Loading/Empty/Normal/Error) | [ ] | |
| D10 | Focus Navigation 已定义 (Tab 路径完整) | [ ] | P1 |
| D11 | Interaction Zones 已定义 (Click/Hover) | [ ] | |
| D12 | Overlay Definition 已定义 (Overlay 列表 + Z-Layer + 内部结构) | [ ] | |
| D13 | Lifecycle 已定义 (OnEnter/OnReady/Active/OnExit) | [ ] | |
| D14 | Data Ownership 已定义 (Owns/Uses) | [ ] | |
| D15 | Layout Intent 已定义 (关键尺寸理由) | [ ] | P1 |
| D16 | Scroll & Overflow Policy 已定义 | [ ] | P1 |
| D17 | Event Contract 已定义 (UI->Domain + Domain->UI) | [ ] | P1 |
| D18 | Screen Metrics 已定义 | [ ] | P1 |

**P0 字段全部通过日期**: {YYYY-MM-DD}
**status 改为 active 日期**: {YYYY-MM-DD}

---

## 附录 B: 引用文档

| 文档 | 用途 |
|------|------|
| `07-specs/README.md` | SSPEC 总纲、AI 14 条规则、DoD 18 项清单 |
| `07-specs/references/widget-id-map.md` | Widget ID -> UiBinding 映射总表（§4.2 BattleScreen 26 IDs + 新增 battle_item_btn, battle_move_btn） |
| `07-specs/references/z-layer-spec.md` | Z-Layer 统一规范 |
| `07-specs/references/layout-intent-library.md` | 跨 Screen 共享的 Layout Intent |
| `06-ui/02-design-system/widget-atoms.md` | 原子组件 Contract（Button, ProgressBar, HeadingText, CaptionText, BodyText, Icon, Image） |
| `06-ui/02-design-system/widget-composites.md` | 复合组件 Contract（CharacterCard, ActionMenu, SkillSlot） |
| `06-ui/02-design-system/theme-localization.md` | StyleToken / Theme / UiTextKey |
| `06-ui/02-design-system/focus-binding.md` | Focusable / FocusGroup / Dirty\<T\> / UiBinding |
| `06-ui/03-screens/screens.md` | BattleScreen 定义（§2） |
| `06-ui/03-screens/screen-lifecycle.md` | Screen 生命周期状态机 |
| `06-ui/04-data-flow/projection-viewmodel.md` | Projection / ViewModel 映射 |
| `03-content/localization/ui-screen-keys.md` | 战斗屏幕 LocalizationKeys（§battle） |

---

## 附录 C: P0 违规架构追溯

> 本附录追踪 code-review (screen-spec-code-gaps.md) 发现的 4 个 P0 违规在 SSPEC 中的处理方式。

### C.1 Factory Bypass (screen-spec-code-gaps.md §1.2)

**原始违规**: `src/ui/screens/battle/mod.rs:53` — 使用 `commands.spawn((Node{...}, BattleScreen))` 而非 `spawn_panel` 工厂。

**SSPEC 修复**: §12 Lifecycle on_enter 明确要求 "通过 spawn_panel 工厂生成 BattleScreen 根容器（禁止原始 commands.spawn）"。所有 Widget 通过 `spawn_*` 工厂函数创建。

### C.2 TurnQueue Domain Query (screen-spec-code-gaps.md §2.1)

**原始违规**: `src/ui/screens/battle/systems.rs:30` — `turn_queue: Option<Res<TurnQueue>>` 直接查询 Combat Domain 组件。

**SSPEC 修复**: §13.2 Data Ownership 中 `current_unit_id` 通过 TurnStarted 事件的参数设置到 BattleHudVm，UI 不再直接查询 TurnQueue。§16.2 Domain->UI Event Contract 中 TurnStarted 事件明确携带 event.unit_id。

### C.3 BattlePhase Domain Query (screen-spec-code-gaps.md §2.1)

**原始违规**: `src/ui/screens/battle/visibility.rs:20` — `battle_phase: Res<State<BattlePhase>>` 直接查询 Domain 状态。

**SSPEC 修复**: §13.1 ViewModel 映射增加 `visible_zones: Vec<String>` 字段，由 `ZoneVisibilityProjection` 驱动。§12.1 on_ready 中注册 `ZoneVisibilityProjection` Observer 替代直接查询 `Res<State<BattlePhase>>`。

### C.4 Hardcoded Turn Indicator Text (screen-spec-code-gaps.md §3.1)

**原始违规**: `src/ui/screens/battle/mod.rs:73` — `"Turn: 3    Phase: Player Turn"` 硬编码调试文本。

**SSPEC 修复**: §7.1 / §7.2 Widget Contract 要求所有文本通过 LocalizationKey 渲染。`battle_turn_indicator` 绑定 `BattleHudVm.turn_number`，`battle_phase_label` 绑定 `BattleHudVm.phase_key`。§16.2 Domain->UI 中 TurnStarted 事件设置正确的 phase_key 值。

---

## 附录 D: 与 screens.md §2 的一致性核对

| screens.md §2 属性 | SSPEC 字段 | 一致性 | 差异说明 |
|-------------------|-----------|--------|---------|
| GameState::Combat | §1 Screen Header | ✅ | 一致 |
| ScreenLayer 0 | §1 Screen Header | ✅ | 一致 |
| Persistent 加载 | §1 Screen Header | ⚠️ | screens.md 标记 Persistent，SSPEC 标记 Ephemeral。理由: 战斗完整生命周期 spawn → despawn 属于标准 Ephemeral 模式，"Persistent" 描述易误解为不释放。实际行为相同（Combat 生命周期内持续存在） |
| Fade(0.3s) | §1 Screen Header | ✅ | 一致 |
| BattleHudVm | §13 Data Ownership | ✅ | 完整字段映射（含 screens.md 未列出的 current_unit_id, available_actions, visible_zones） |
| SkillPanelVm | §13 Data Ownership | ⚠️ | screens.md 提及 SkillPanelVm, CharacterPanelVm，但 SSPEC 将其合并至 BattleHudVm（当前 current_unit 的 skill_panel 和 character_panel 数据由 BattleHudVm 承载，减少 ViewModel 数量）。分离方案在后续迭代中处理 |
| 5 组件组合 | §2 Wireframe / §3 Widget Tree | ✅ | 5 组件都已包含（TopBar + BattleArea + CharacterCard + ActionMenu + EndTurnBtn），且增加了 Item/Move 两个按钮 |
| 3 个 UiCommand | §16 Event Contract | ✅ | CastSkill, SelectTarget, EndTurn 全部保留，增加 Defend/Wait/EnterMoveMode/OpenOverlay |

---

## 附录 E: 新引入 Widget ID 记录

以下 widget_id 为此 BattleScreen SSPEC 新引入（不在 widget-id-map.md §4.2 现有 26 个 ID 中），需要在 widget-id-map.md 注册：

| widget_id | UiBinding | 原因 |
|-----------|-----------|------|
| `battle_item_btn` | `UiBinding::None` | ActionMenu 第六个按钮，物品操作入口（对应 loc::ui::BATTLE_ITEM） |
| `battle_move_btn` | `UiBinding::None` | ActionMenu 第五个按钮，移动操作入口（对应 loc::ui::BATTLE_MOVE） |

相关 localization keys 已在代码库中存在:
- `loc::ui::BATTLE_ITEM` = `"ui.battle.item"` (keys.rs:264)
- `loc::ui::BATTLE_MOVE` = `"ui.battle.move"` (keys.rs:265)

---

## 附录 F: 新提出 LocalizationKey 需求

以下 LocalizationKey 在现有代码库 (`src/infra/localization/generated/keys.rs`) 中不存在，为 BattleScreen SSPEC 实现所需新增：

| Key | 用途 | 默认值 (en) | 默认值 (zh) |
|-----|------|------------|------------|
| `ui.battle.turn.label` | TurnIndicator 文本格式 | "Turn: {n}" | "回合 {n}" |
| `ui.battle.phase.player` | 玩家回合阶段标签 | "Player Turn" | "玩家回合" |
| `ui.battle.phase.enemy` | 敌方回合阶段标签 | "Enemy Turn" | "敌方回合" |
| `ui.battle.area.loading` | 战斗区域加载提示 | "Loading battle map..." | "正在加载战斗地图..." |
| `ui.battle.area.empty` | 战斗区域空状态提示 | "No battle data" | "无战斗数据" |
| `ui.battle.area.error` | 战斗区域错误提示 | "Failed to load battle map" | "战斗地图加载失败" |
| `ui.battle.area.retry` | 重试按钮文本 | "Retry" | "重试" |
| `ui.battle.char_panel.empty` | 角色面板空状态 | "No unit selected" | "未选中单位" |
| `ui.battle.buff.applied` | Buff 应用通知 (含 buff_name 参数) | "{buff_name} applied" | "{buff_name} 已生效" |
| `ui.battle.buff.expired` | Buff 过期通知 (含 buff_name 参数) | "{buff_name} expired" | "{buff_name} 已消失" |
| `ui.battle.result.victory` | 胜利标题 | "Victory!" | "胜利！" |
| `ui.battle.result.defeat` | 失败标题 | "Defeated..." | "战败..." |
| `ui.battle.result.exp_gained` | 经验值获取文本 | "EXP: {n}" | "经验值: {n}" |
| `ui.battle.result.loot` | 战利品文本 | "Loot: {item_name}" | "战利品: {item_name}" |
| `ui.battle.result.continue` | 继续按钮文本 | "Continue" | "继续" |
| `ui.battle.result.retry` | 重试按钮文本 | "Retry" | "重试" |
| `ui.battle.result.return_title` | 返回标题按钮 | "Return to Title" | "返回标题画面" |

---

*本文档是 BattleScreen SSPEC，由 @feature-developer 根据 `07-specs/screen-spec-template.md` 模板创建。所有 17 个字段已填充。当前 status: draft（待 @presentation-architect 审查）。*
