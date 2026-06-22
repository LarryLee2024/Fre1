---
id: 07-specs.{screen-name}
title: {ScreenName} Specification — AI-Consumable Layout & Interaction Spec
status: draft
owner: presentation-architect
created: {YYYY-MM-DD}
tags:
  - ui
  - screen-spec
  - {tag}
  - draft
---

# {ScreenName}

> **职责**: @presentation-architect | **上游**: ADR-066 (Screen Spec), `07-specs/README.md` (总纲)
> **状态**: 初始 draft，完成后改为 active

**P0 字段**: 1-14 (Screen Header / ASCII Wireframe / Widget Tree / Flexbox Layout / Responsive Rules / Region Responsibility / Widget Contract / State Mapping / Focus Nav / Interaction Zones / Overlay / Lifecycle / Data Ownership / Layout Intent)
**P1 字段**: 15-17 (Scroll & Overflow / Event Contract / Screen Metrics)

---

## 1. Screen Header

| 属性 | 值 |
|------|-----|
| Screen Name | `{ScreenName}` — 对应 `GameState::{Variant}` |
| Purpose | {一句话描述该屏幕的核心功能} |
| Navigation | {如何进入/离开此屏幕，如：MainMenu → NewGame → BattleScreen，Esc → PreviousScreen} |
| GameState | `GameState::{Variant}` |
| ScreenLayer 层级 | {0=主界面层 / 1=Overlay 层} |
| 加载模式 | Persistent / Ephemeral |
| 过渡动画 | {如: Fade(0.3s), Slide(Direction::Up)} |
| 变体 | {如: None / BossBattle / AutoBattle / SaveMode / LoadMode} |

---

## 2. ASCII Wireframe

> 纯文本线框图。所有区域必须命名（`widget_id`），禁止匿名面板。
> 水平线 `---` 表示横向分隔，竖线 `|` 表示纵向分隔。

```
┌────────────────────────────────────────────────────────────────┐
│  [region_id: top_bar]                                          │
│  ┌──────────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ turn_indicator   │  │ phase_label  │  │ end_turn_btn     │  │
│  │ "Turn: {n}"      │  │ "Player Turn"│  │ [End Turn]       │  │
│  └──────────────────┘  └──────────────┘  └──────────────────┘  │
├────────────────────────────────────────────────────────────────┤
│  [region_id: battle_area]                                      │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                                                          │  │
│  │              (战斗场地 — 6层渲染地图)                      │  │
│  │                                                          │  │
│  └──────────────────────────────────────────────────────────┘  │
├────────────────────────────────────────────────────────────────┤
│  [region_id: char_panel]                                       │
│  ┌──────────┬───────────────────────────────────────────────┐  │
│  │ avatar   │ name: "Aria"         lv: 5                   │  │
│  │          │ hp_bar:  ████████░░  80/100                   │  │
│  │          │ mp_bar:  ████░░░░░░  40/50                    │  │
│  │          │ buffs: [icon0][icon1]                         │  │
│  └──────────┴───────────────────────────────────────────────┘  │
├────────────────────────────────────────────────────────────────┤
│  [region_id: action_menu]                                      │
│  ┌──────────┬──────────┬──────────┬──────────┐                │
│  │ attack   │ skill    │ defend   │ wait     │                │
│  │ [btn]    │ [btn]    │ [btn]    │ [btn]    │                │
│  └──────────┴──────────┴──────────┴──────────┘                │
└────────────────────────────────────────────────────────────────┘
```

### 2.1 Region 索引

| widget_id | 类型 | 用途 | 对应 Wireframe 位置 |
|-----------|------|------|-------------------|
| `{region_id}` | Container / Atom / Molecule / Organism | {用途描述} | {Wireframe 中的位置描述} |

---

## 3. Widget Tree

> 标注 `[widget_id: WidgetType]` 的树结构。禁止隐藏节点，必须完整。

```
ScreenRoot                                      [root: Screen]
├── TopBar                                      [top_bar: Container]
│   ├── TurnIndicator                           [turn_indicator: BodyText]
│   ├── PhaseLabel                              [phase_label: CaptionText]
│   └── EndTurnButton                           [end_turn_btn: Button, Danger]
├── BattleArea                                  [battle_area: Panel]
│   └── (6-Layer Map Renderer — infra 层实现)    [map_layer: Container]
├── CharacterCard                               [char_panel: Organism]
│   ├── Avatar                                  [avatar: Image]
│   ├── NameLevel                               [name_level: BodyText]
│   ├── HpBar                                   [hp_bar: ProgressBar]
│   ├── MpBar                                   [mp_bar: ProgressBar]
│   └── BuffIcons                               [buff_icons: Container]
│       ├── BuffSlot0                           [buff_icons_0: Icon]
│       └── BuffSlot1                           [buff_icons_1: Icon]
└── ActionMenu                                  [action_menu: Container]
    ├── AttackButton                            [attack_btn: Button, Primary]
    ├── SkillButton                             [skill_btn: Button, Secondary]
    ├── DefendButton                            [defend_btn: Button, Secondary]
    └── WaitButton                              [wait_btn: Button, Secondary]
```

### 3.1 Widget Type 索引

| widget_id | WidgetType | 定义位置 | 复用于 |
|-----------|-----------|---------|--------|
| `{widget_id}` | `{Atom/Molecule/Organism}` | `02-design-system/widget-{atoms/composites}.md` §{n} | `{other_screen}` |

---

## 4. Flexbox Layout

> YAML 格式。每个 widget_id 必须有 direction / width / height / flex_grow / intent。

```yaml
## Flexbox Layout — {ScreenName}
## width/height: px 值或 "auto" 或 "fill"
## flex_grow: 0=不增长, 1=等分剩余空间, 2=双倍增长
## shrink: none/low/high — 收缩优先级

root:
  direction: column
  width: 100%
  height: 100%
  flex_grow: 0
  intent: "Screen 根容器，占满视口"

top_bar:
  direction: row
  width: 100%
  height: 64
  flex_grow: 0
  shrink: none
  intent: "顶栏，固定 64px 高度，容纳回合信息 + 结束回合按钮"

turn_indicator:
  direction: row
  width: 160
  height: "auto"
  flex_grow: 0
  shrink: none
  intent: "回合信息文本，固定宽度避免折行"

phase_label:
  direction: row
  width: 200
  height: "auto"
  flex_grow: 0
  shrink: low
  intent: "阶段标签文本，宽度可压缩但不可完全隐藏"

end_turn_btn:
  direction: row
  width: 120
  height: 40
  flex_grow: 0
  shrink: none
  intent: "结束回合按钮，固定尺寸保证可点击区域 ≥ 40x40"

battle_area:
  direction: column
  width: "fill"
  height: "fill"
  flex_grow: 1
  intent: "战斗场地占满剩余空间，地图越大越好"

char_panel:
  direction: row
  width: 100%
  height: 120
  flex_grow: 0
  shrink: none
  intent: "角色面板，固定 120px 高度容纳头像+状态条"

action_menu:
  direction: row
  width: 100%
  height: 80
  flex_grow: 0
  shrink: none
  intent: "行动菜单底栏，固定 80px 容纳 4 个按钮"
```

---

## 5. Responsive Rules

| 条件 | 行为 | 影响区域 |
|------|------|---------|
| width < 800px | strategy: "none" — 当前不实现响应式 | 全部 |
| width < 480px | {策略说明，如: hide action_menu → replace with hamburger} | {region_id} |

**最小支持分辨率**: 1280 x 720 (16:9)
**设计分辨率**: 1920 x 1080 (16:9)

---

## 6. Region Responsibility

> 每个 region 3-8 条职责。明确该区域"展示什么"和"不做什么"。

### 6.1 {region_id}

| # | 职责 | 类型 | 违反后果 |
|---|------|------|---------|
| R01 | 展示{信息名} | Display | {描述} |
| R02 | 响应{交互类型} | Interaction | {描述} |
| R03 | {其他职责} | {类型} | {描述} |

**不负责**:
- {此 region 不应该做的事}

### 6.2 {region_id}



---

## 7. Widget Contract

> Inputs / Outputs / Selection Model。对于复合组件 (Organism)，引用其定义的 Contract；对于直接组合的原语，在此列明。

### 7.1 {widget_id}

```yaml
widget_id: {widget_id}
widget_type: {Button / ProgressBar / Text / Panel / ...}
defined_in: "02-design-system/widget-{atoms/composites}.md §{n}"

inputs:
  - name: {prop_name}
    type: {type}
    source: "{ViewModel.field_path}"
    default: {default_value}

outputs:
  - name: {event_name}
    type: UiCommand::{Variant}
    payload: {payload_type}
    trigger: {trigger_condition}

selection_model:
  type: none / single / multi
  max_select: {n}          # multi 模式下最大可选数
  clear_on: {event}        # 何时清空选择
```

---

## 8. State Mapping (Per-Region)

> 每个 region 独立的状态。必须定义 Loading / Empty / Normal / Error 四种状态。
> AI 实现时必须为每种状态提供对应的 UI 展示。

### 8.1 {region_id}

| 状态 | 展示内容 | 触发条件 | 过渡行为 |
|------|---------|---------|---------|
| **Loading** | {骨架屏 / Spinner / 占位文本} | {ViewModel 数据未就绪} | {Spinner → Normal 的过渡} |
| **Empty** | {空状态提示文本 + Icon} | {数据加载完成但为空} | {无数据时的展示} |
| **Normal** | {正常数据展示} | {数据加载完成且有数据} | {常规渲染} |
| **Error** | {错误提示文本 + 重试按钮} | {数据加载失败} | {错误提示 → 用户点击重试} |

### 8.2 {region_id}



---

## 9. Focus Navigation

> Tab 导航路径。按 Tab 键的顺序就是导航路径的顺序。

```yaml
focus_path:
  - {region_id}          # Tab 1
  - {region_id}          # Tab 2
  - {region_id}          # Tab 3
  # ...

special_keys:
  Escape: "返回上一级 / 关闭当前 Screen"
  Enter: "确认选中项"
  ArrowUp/Down: "列表内导航 / Tab 组切换"
  ArrowLeft/Right: "Tab 页切换 / Slider 调节"
  Tab: "按 focus_path 顺序前进"
  Shift+Tab: "按 focus_path 逆序后退"

focus_trap: true          # true = 焦点锁定在该 Screen 内，Tab 循环
```

---

## 10. Interaction Zones

> 每个可交互区域的行为定义。Click / Hover / Drag / Drop。

### 10.1 {region_id}

```yaml
zone_id: {region_id}
interactions:
  - type: click
    button: Left
    effect: "触发 UiCommand::{Variant}"
    cursor: Pointer
    conditions:
      - {条件 1: 描述}
      - {条件 2: 描述}

  - type: hover
    enter_effect: "显示 TooltipOverlay，内容: {ViewModel.field_path}"
    leave_effect: "隐藏 TooltipOverlay"
    delay: 500ms

  - type: drag
    source: {region_id}
    preview: "{拖拽预览 UI 描述}"
    drop_zones:
      - {region_id}

  - type: drop
    accept_types: ["{type}"]
    effect: "触发 UiCommand::{Variant}"
    hover_effect: "高亮放置区域"
```

---

## 11. Overlay Definition

> Overlay 列表 + Z-Layer。Overlay 使用独立层级，不嵌套在任何 Screen 之下。

| Overlay | 用途 | Z-Layer | 类型 | 触发条件 |
|---------|------|---------|------|---------|
| {overlay_name} | {用途描述} | {1-9} | Tooltip / Modal / Notification / Popup / Debug | {触发条件} |
| {overlay_name} | {用途描述} | {1-9} | Tooltip / Modal / Notification / Popup / Debug | {触发条件} |

### 11.1 Z-Layer 分配

| Z-Layer | 用途 | 包含 |
|---------|------|------|
| 0 | Screen 主界面层 | `root` 及所有子 region |
| 1 | Tooltip 层 | TooltipOverlay |
| 2 | Notification 层 | NotificationOverlay (toast) |
| 3 | Modal 层 | ModalOverlay (确认弹窗) |
| 4 | Popup 层 | PopupOverlay (商店/设置) |
| 9 | Debug 层 | DebugOverlay (FPS/日志) |

### 11.2 Overlay 生命周期

| Overlay | OnOpen | OnClose | 依赖 |
|---------|--------|---------|------|
| {overlay_name} | {创建行为} | {清理行为} | {依赖的其他 Overlay} |

---

## 12. Lifecycle

> Screen 的完整生命周期行为。遵守 `screen-lifecycle.md` 定义的状态机。

| 阶段 | 行为 | 触发条件 | 清理 |
|------|------|---------|------|
| **OnEnter** | {spawn UI 树的完整行为描述} | `GameState::{Variant}` 状态进入 | — |
| **OnReady** | {ViewModel 首次数据加载、Observer 注册} | OnEnter 完成，UI 树就绪 | — |
| **Active** | {每帧/事件驱动的更新行为} | OnReady 完成 | — |
| **OnExit** | {despawn UI 树、注销 Observer} | `GameState::{Variant}` 状态退出 | 清理标记: `With<{Screen}>` |

### 12.1 生命周期事件处理

```yaml
on_enter:
  - action: "spawn_ui_tree"
    spawner: "spawn_{screen_name}()"
    description: "生成完整 UI 树，所有 Widget 按 Flexbox Layout 定位"

on_ready:
  - action: "register_observer"
    target: "{Projection}::{Event}"
    handler: "更新 ViewModel 并标记 dirty"
  - action: "load_data"
    target: "{ViewModel}"
    from: "{Projection}"

active:
  - trigger: "ViewModel.dirty"
    action: "刷新对应 Widget 数据"
    scope: "{region_id}"
  - trigger: "UiCommand::{Variant}"
    action: "通过 UiLayer 下发至 Domain"

on_exit:
  - action: "unregister_observer"
    target: "{Projection}::{Event}"
  - action: "despawn_ui_tree"
    query: "With<{Screen}>"
    description: "清理所有标记了 {Screen} 组件的实体"
```

---

## 13. Data Ownership

> Owns / Uses 分离。明确每个 UiStore 字段的归属。

### 13.1 ViewModel 映射

| ViewModel | 字段 | 归属 (Owns/Uses) | 更新频率 | Projection 源 |
|-----------|------|-----------------|---------|--------------|
| `{Vm}::{field}` | {字段类型} | Owns — Screen 独享 | {每帧 / 事件触发 / 按需} | `{Domain}Projection::{method}` |
| `{Vm}::{field}` | {字段类型} | Uses — 多个 Screen 共享 | {每帧 / 事件触发 / 按需} | `{Domain}Projection::{method}` |

### 13.2 数据流

```
Domain Event → {Projection} → {Vm} → {Screen} / {Widget}
                                                        ↓
UiCommand ← UiAction ← Intent ← Input ← (用户交互)
```

---

## 14. Layout Intent

> 每个关键尺寸的**理由说明**。为什么选这个尺寸而不是别的？
> 记录意图是为了防止未来修改时随意改尺寸。

### 14.1 固定尺寸意图

| widget_id | 属性 | 值 | 意图 | shrink |
|-----------|------|----|------|--------|
| `{region_id}` | width | {n}px | "{为什么是这个宽度}" | none/low/high |
| `{region_id}` | height | {n}px | "{为什么是这个高度}" | none/low/high |
| `{region_id}` | min_width | {n}px | "{为什么需要最小宽度}" | none |

### 14.2 弹性尺寸意图

| widget_id | flex_grow | 理由 |
|-----------|-----------|------|
| `{region_id}` | {0/1/2} | "{为什么这个区域需要/不需要弹性增长}" |

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

> 每个可能产生滚动的区域必须定义 policy。
> Overflow 策略: clip — 裁剪 / ellipsis — 省略号 / scroll — 可滚动 / visible — 可见溢出

### 15.1 滚动区域

| widget_id | 方向 | Scroll Policy | Overflow Policy | 理由 |
|-----------|------|--------------|---------------|------|
| `{region_id}` | vertical / horizontal / both | scroll / none | clip / ellipsis / scroll / visible | "{为什么这个区域需要/不需要滚动}" |
| `{region_id}` | vertical / horizontal / both | scroll / none | clip / ellipsis / scroll / visible | "{为什么这个区域需要/不需要滚动}" |

### 15.2 文本溢出

| widget_id | max_lines | overflow | 多语言风险 |
|-----------|-----------|----------|-----------|
| `{region_id}` | {n} (或 `unlimited`) | ellipsis / clip / scroll | {该文本在不同语言下的长度风险，如：中文 5 字 ≈ 英文 15 字} |

---

## 16. Event Contract

> UI -> Domain 事件 + Domain -> UI 事件的完整契约。

### 16.1 UI -> Domain（通过 UiCommand 传递）

```yaml
{EventName}:
  trigger_widget: "{region_id} → {sub_region} → {interaction}"
  data:
    {field}: {type}
    {field}: {type}
  conditions:
    - {条件 1}
    - {条件 2}
  emits: UiCommand::{Variant}({payload_type})
  domain_event: {DomainEvent1} | {DomainEvent2}

{EventName}:
  trigger_widget: "{region_id} → {sub_region} → {interaction}"
  data: {}
  conditions: {条件}
  emits: UiCommand::{Variant}
  domain_event: {DomainEvent}
```

### 16.2 Domain -> UI（通过 Projection 消费）

```yaml
{DomainEvent}:
  source: "Domain Event ({Domain} Domain)"
  projection: "{Projection}.{method}()"
  vm_update:
    - {Vm}.{field} ← {value}
  side_effect:
    - "mark_dirty::<{Vm}>()"

{DomainEvent}:
  source: "Domain Event ({Domain} Domain)"
  projection: "{Projection}.{method}()"
  vm_update:
    - {Vm}.{field} += {delta}
  side_effect:
    - "mark_dirty::<{Vm}>()"
```

---

## 17. Screen Metrics

> 复杂度基线。所有数值初始创建时手动填写，后续 CI 阶段自动校验。

| Metric | 值 | P0/P1 | 说明 |
|--------|-----|-------|------|
| `widget_count` | {n} | P1 | Widget 实例总数（叶子节点+容器节点） |
| `container_count` | {n} | P1 | 纯容器节点数（无业务逻辑，仅布局） |
| `interactive_count` | {n} | P1 | 可交互 Widget 数（Button / Slider / Toggle 等） |
| `overlay_count` | {n} | P1 | 关联的 Overlay 数 |
| `max_depth` | {n} | P1 | root → 最深深层叶子 Widget 的层级数 |
| `max_children` | {n} | P1 | 单一容器的最大子节点数 |

### 17.1 Budget 检查

| 规则 | 阈值 | 当前值 | 状态 |
|------|------|--------|------|
| max_depth ≤ 6 | 6 | {n} | ✅ / ❌ |
| max_children ≤ 20 | 20 | {n} | ✅ / ❌ |
| interactive_count / widget_count ≥ 0.2 | 20% | {n}% | ✅ / ❌ |

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
| D11 | Interaction Zones 已定义 (Click/Hover/Drag/Drop) | [ ] | |
| D12 | Overlay Definition 已定义 (Overlay 列表 + Z-Layer) | [ ] | |
| D13 | Lifecycle 已定义 (OnEnter/OnReady/OnExit) | [ ] | |
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
| `07-specs/README.md` | SSPEC 总纲、AI 14 条规则、DoD 14 项清单 |
| `07-specs/references/widget-id-map.md` | Widget ID -> UiBinding 映射总表 |
| `07-specs/references/z-layer-spec.md` | Z-Layer 统一规范 |
| `07-specs/references/layout-intent-library.md` | 跨 Screen 共享的 Layout Intent |
| `06-ui/02-design-system/widget-atoms.md` | 原子组件 Contract |
| `06-ui/02-design-system/widget-composites.md` | 复合组件 Contract |
| `06-ui/02-design-system/theme-localization.md` | StyleToken / Theme / UiTextKey |
| `06-ui/02-design-system/focus-binding.md` | Focusable / FocusGroup / Dirty<T> / UiBinding |
| `06-ui/03-screens/screen-lifecycle.md` | Screen 生命周期状态机 |
| `06-ui/04-data-flow/projection-viewmodel.md` | Projection / ViewModel 映射 |

---

*本文档是 SSPEC 模板，由 @presentation-architect 维护。使用时复制此文件到 `screens/{name}.md`，替换所有 `{placeholder}` 为实际内容。*
